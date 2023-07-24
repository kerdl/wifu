pub mod interface;
pub mod network;
pub mod pinger;
pub mod acm;
pub mod util;
pub mod cfg;
pub mod error;
pub use pinger::{Pinger, PingErr, PingOk};
pub use error::{Result, Error};

use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

/// ## Indicates that the app is currently not working
/// 
/// - `true`: No wireless interfaces are connected
/// - `false`: At least one wireless interface is connected
pub static IS_DEAD: Lazy<Arc<RwLock<bool>>> = Lazy::new(|| Arc::new(RwLock::new(true)));


pub enum DeadReason {
    NoInterface,
    NoNetwork,
}

pub async fn dead(reason: DeadReason) {
    *IS_DEAD.write().await = true;

    match reason {
        DeadReason::NoInterface => {
            println!("! DEAD: no available wireless interfaces");
            println!("? FIX: connect at least one wireless interface (USB, PCIe, virtual, etc.)")
        },
        DeadReason::NoNetwork => {
            println!("! DEAD, could not connect to any available network");
            println!("? FIX: check that at least one network defined in ./wifu-data/cfg.json is reachable")
        }
    }

    network::close_all_handles().await;
}

pub async fn alive(spawn_network: bool) {
    *IS_DEAD.write().await = false;
    println!("o ALIVE");

    if spawn_network {
        network::spawn_all_handles().await;
    }
}

pub async fn run() {
    if interface::list_is_empty().await {
        dead(DeadReason::NoInterface).await
    } else {
        alive(true).await
    }
}

pub async fn init_fs() {
    if !crate::DATA_PATH.exists() {
        tokio::fs::create_dir(crate::DATA_PATH.as_path()).await.unwrap();
    }

    if !crate::CFG_PATH.exists() {
        crate::CONFIG.set(cfg::Config::default_and_save().await.unwrap()).unwrap();
    } else {
        crate::CONFIG.set(cfg::Config::load().await.unwrap()).unwrap()
    }
}
