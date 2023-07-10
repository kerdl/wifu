pub mod data;
pub mod handlers;

use std::net::{ToSocketAddrs, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use winping::{AsyncPinger, Buffer};
use windows::Win32::NetworkManagement::WiFi;
use windows::Win32::Foundation::HANDLE;
use widestring::U16CStr;


lazy_static! {
    pub static ref DATA_PATH: PathBuf = PathBuf::from("./wifu-data");
    pub static ref CFG_PATH: PathBuf = DATA_PATH.join("cfg.json");
}

pub static CONFIG: OnceCell<data::cfg::Config> = OnceCell::new();


#[tokio::main]
async fn main() {
    data::init_fs().await;
    let config = CONFIG.get().unwrap();

    println!("{:?}", config);

    let wlan_handle = unsafe {
        // 1 = Client version for Windows XP with SP3 and Wireless LAN API for Windows XP with SP2
        // 2 = Client version for Windows Vista and Windows Server 2008
        let client_version = 2;
        let mut handle: HANDLE = std::mem::zeroed();
        let mut negotiated_version: u32 = std::mem::zeroed();

        let result = WiFi::WlanOpenHandle(client_version, None, &mut negotiated_version, &mut handle);
        println!("negotiated_version={}", negotiated_version);
        println!("WlanOpenHandle -> {}", result);

        handle
    };

    println!("wlan_handle={:?}", wlan_handle);

    let wlan_interfaces = unsafe {
        let mut list = std::mem::zeroed();

        let result = WiFi::WlanEnumInterfaces(wlan_handle, None, &mut list);
        println!("WlanEnumInterfaces -> {}", result);

        *list
    };

    if wlan_interfaces.dwNumberOfItems < 1 {
        println!("no wlan devices detected");
        return
    }
    
    println!("wlan_interfaces={:?}", wlan_interfaces);

    let first_interface = wlan_interfaces.InterfaceInfo.get(0).unwrap();
    let desc = U16CStr::from_slice_truncate(first_interface.strInterfaceDescription.as_slice()).unwrap().to_string().unwrap();
    println!("{}", desc);

    unsafe { WiFi::WlanCloseHandle(wlan_handle, None) };
    return;

    let mut ips: Vec<SocketAddr> = vec![];
    let mut pinger = AsyncPinger::new();
    let mut buf = Buffer::new();

    pinger.set_timeout(config.ping.timeout_ms);

    loop {
        for domain in CONFIG.get().unwrap().ping.domains.iter() {
            let domain_ips = format!("{}:80", domain).to_socket_addrs();
            if let Err(err) = domain_ips {
                println!("{}: {}", domain, err);
                continue
            }
            let mut domain_ips = domain_ips.unwrap();
    
            let Some(first_ip) = domain_ips.nth(0) else { continue; };
            ips.push(first_ip);
        }

        if ips.is_empty() {
            println!("no ips were found for domains specified in the config, retrying in 5s");
            tokio::time::sleep(Duration::from_secs(5)).await;
        } else {
            break
        }
    }

    println!("will ping these ips: {:?}", ips);

    'infinite: loop {
        let mut errors = 0;

        'ips: for ip in ips.iter() {
            println!("{}: starting ping", ip);

            'ip: loop {
                let answer = pinger.send(ip.ip(), buf).await;
                buf = answer.buffer;

                match answer.result {
                    Ok(rtt) => {
                        println!("{}: rtt={}", ip, rtt);
                        errors = 0;
                    },
                    Err(err) => {
                        println!("{}: err={}", ip, err);
                        errors += 1;
                        break 'ip
                    },
                }

                tokio::time::sleep(Duration::from_millis(config.ping.interval_ms)).await;
            }

            if errors == config.ping.max_errors {
                println!("switch wifi")
            }
        }
    }
}
