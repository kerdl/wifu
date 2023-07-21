use std::net::{ToSocketAddrs, SocketAddr};


pub fn http_to_ips(domain: &str) -> std::io::Result<std::vec::IntoIter<SocketAddr>> {
    format!("{}:80", domain).to_socket_addrs()
}