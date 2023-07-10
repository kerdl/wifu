use std::{net::{Ipv4Addr, ToSocketAddrs, IpAddr}, process::Output};
use tokio::process::{Command, Child};


pub struct Pinger {
    pub ips: Vec<IpAddr>
}
impl Pinger {
    
}
impl Default for Pinger {
    fn default() -> Self {
        Self { ips: vec![] }
    }
}