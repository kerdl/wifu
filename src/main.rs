mod data;
pub use data::app;
pub use data::win;

use app::wlan::interface;
use app::wlan::network;
use app::pinger;
use win::wlan::network::profile::Key;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;


lazy_static! {
    pub static ref DATA_PATH: PathBuf = PathBuf::from("./wifu-data");
    pub static ref CFG_PATH: PathBuf = DATA_PATH.join("cfg.json");
    pub static ref CHANNEL: (tokio::sync::mpsc::Sender<()>, tokio::sync::RwLock<tokio::sync::mpsc::Receiver<()>>) = {
        let (tx, rx) = tokio::sync::mpsc::channel::<()>(100);
        (tx, tokio::sync::RwLock::new(rx))
    };
}

pub static CONFIG: OnceCell<data::app::cfg::Config> = OnceCell::new();
pub static WLAN: OnceCell<Arc<win::Wlan>> = OnceCell::new();


#[tokio::main]
async fn main() {
    app::init_fs().await;

    let config = CONFIG.get().unwrap();
    if let Err(reasons) = config.is_valid() {
        println!("x CONFIG is invalid due to the following reasons: {:?}", reasons);
        return;
    }

    WLAN.set(Arc::new(win::Wlan::new(win::wlan::ClientVersion::Second).unwrap())).unwrap();

    interface::start().await;

    app::STATE.write().await.choose().await;

    std::thread::park();
}
