pub mod wlan;
pub mod pinger;
pub mod util;
pub mod cfg;
pub mod state;
pub mod error;
pub mod log;
pub use pinger::{Pinger, PingErr, PingOk};
pub use state::{STATE, State, DeadReason};
pub use error::{Result, Error};

use wlan::{interface, network};


pub async fn init_fs() -> bool {
    if !crate::DATA_PATH.exists() {
        tokio::fs::create_dir(crate::DATA_PATH.as_path()).await.unwrap();
    }

    if !crate::CFG_PATH.exists() {
        crate::CONFIG.set(cfg::Config::default_and_save().await.unwrap()).unwrap();
        return true;
    } else {
        crate::CONFIG.set(cfg::Config::load().await.unwrap()).unwrap();
        return false;
    }
}
