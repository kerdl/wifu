use std::{net::SocketAddr, time::Duration};
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use winping::AsyncPinger;

use super::cfg;


pub static PINGER: Lazy<Arc<RwLock<Pinger>>> = Lazy::new(
    || Arc::new(RwLock::new(Pinger::from_config(crate::CONFIG.get().unwrap().ping.clone())))
);

pub struct PingOk {
    pub buf: winping::Buffer,
    pub rtt: u32,
}

pub struct PingErr {
    pub buf: winping::Buffer,
    pub err: winping::Error
}


pub struct Pinger {
    pinger: AsyncPinger,
    pub config: super::cfg::Ping,
    pub ips: Vec<SocketAddr>,
}
impl Pinger {
    pub fn from_config(config: super::cfg::Ping) -> Self {
        let ips = Self::gather_ips(&config.domains);
        let mut pinger = AsyncPinger::new();
        pinger.set_timeout(config.timeout_ms);

        Self { pinger, config, ips }
    }

    pub fn has_no_ips(&self) -> bool {
        self.ips.is_empty()
    }

    fn gather_ips(domains: &cfg::Domains) -> Vec<SocketAddr> {
        let mut ips = vec![];

        for domain in domains.list.iter() {
            let domain_ips = super::util::domain::http_to_ips(domain);

            if domain_ips.is_err() || domain_ips.as_ref().unwrap().as_slice().is_empty() {
                continue;
            }
            let domain_ips = domain_ips.unwrap();

            match domains.mode {
                super::cfg::DomainsMode::FirstIpFromEach => {
                    ips.push(domain_ips.as_slice()[0])
                },
                super::cfg::DomainsMode::AllIpsFromEach => {
                    ips.extend_from_slice(domain_ips.as_slice())
                }
            }
        }

        ips
    }

    pub fn update_ips(&mut self) {
        self.ips = Self::gather_ips(&self.config.domains)
    }

    async fn ping_ip_once(&self, addr: &SocketAddr, buf: winping::Buffer) -> Result<PingOk, PingErr> {
        let answer = self.pinger.send(addr.ip(), buf).await;
        let buf = answer.buffer;

        match answer.result {
            Ok(rtt) => {
                println!("{}: rtt={}", addr, rtt);
                return Ok(PingOk { buf, rtt })
            },
            Err(err) => {
                println!("{}: err={}", addr, err);
                return Err(PingErr { buf, err })
            },
        }
    }

    pub async fn start(&self) -> () {
        let mut errors = 0;
        let mut buf = winping::Buffer::new();


        let mut addr_idx: i32 = -1;
        'addrs: loop {
            addr_idx += 1;

            let Some(addr) = self.ips.get(addr_idx as usize) else {
                addr_idx = 0;
                continue 'addrs;
            };

            println!("pinging ip: {}", addr);
            
            'addr: loop {
                match self.ping_ip_once(addr, buf).await {
                    Ok(result) => {
                        errors = 0;
                        buf = result.buf;
                        tokio::time::sleep(Duration::from_millis(self.config.interval_ms)).await;
                    },
                    Err(err) => {
                        errors += 1;
                        buf = err.buf;
                        break 'addr;
                    },
                }
            }

            if errors >= self.config.max_errors {
                break 'addrs;
            }
        }

        println!("trigger wifi switch")
    }
}
