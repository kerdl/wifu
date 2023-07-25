pub mod wlan;
pub mod pinger;
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
/// or no networks available
/// - `false`: At least one wireless interface is connected
/// and at leas one network is available
pub static STATE: Lazy<Arc<RwLock<State>>> = Lazy::new(|| Arc::new(RwLock::new(State::default())));


#[derive(Clone)]
pub enum DeadReason {
    NoInterface,
    NoNetwork,
}

pub enum State {
    Dead(DeadReason),
    Alive
}
impl State {
    pub fn is_dead(&self) -> bool {
        match self {
            Self::Dead(_) => true,
            Self::Alive => false,
        }
    }

    pub fn is_alive(&self) -> bool {
        !self.is_dead()
    }

    pub fn dead_because_no_interface(&self) -> bool {
        match self {
            Self::Alive => false,
            Self::Dead(reason) => match reason {
                DeadReason::NoInterface => true,
                DeadReason::NoNetwork => false,
            }
        }
    }

    pub fn dead_because_no_network(&self) -> bool {
        match self {
            Self::Alive => false,
            Self::Dead(reason) => match reason {
                DeadReason::NoInterface => false,
                DeadReason::NoNetwork => true,
            }
        }
    }
}
impl Default for State {
    fn default() -> Self {
        Self::Dead(DeadReason::NoInterface)
    }
}

pub async fn dead(reason: DeadReason) {
    *STATE.write().await = State::Dead(reason.clone());

    network::close_pinger_global().await;

    match reason {
        DeadReason::NoInterface => {
            println!("! DEAD: no available wireless interfaces");
            println!("? FIX: connect at least one wireless interface (USB, PCIe, virtual, etc.)")
        },
        DeadReason::NoNetwork => {
            println!("! DEAD, could not connect to any available network");
            println!("? FIX: check that at least one network defined in ./wifu-data/cfg.json is reachable");
            network::spawn_cfg_network_wait_global().await;
        }
    }
}

pub async fn alive(spawn_pinger: bool) {
    *STATE.write().await = State::Alive;
    println!("o ALIVE");

    if spawn_pinger {
        //network::choose_global(false).await;
        println!("alive(): spawning pinger");
        network::spawn_pinger_global().await;
    }
}

pub async fn run() {
    if interface::list_is_empty().await {
        dead(DeadReason::NoInterface).await
    } else if !network::cfg_networks_available().await {
        dead(DeadReason::NoNetwork).await
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
