mod data;
pub use data::app;
pub use data::win;

use app::log::Logger;
use app::wlan::interface;
use app::wlan::network;

use std::path::PathBuf;
use std::sync::Arc;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use log::{debug, error};


lazy_static! {
    pub static ref DATA_PATH: PathBuf = PathBuf::from("./wifu-data");
    pub static ref CFG_PATH: PathBuf = DATA_PATH.join("cfg.json");
    pub static ref CHANNEL: (tokio::sync::mpsc::Sender<()>, tokio::sync::RwLock<tokio::sync::mpsc::Receiver<()>>) = {
        let (tx, rx) = tokio::sync::mpsc::channel::<()>(100);
        (tx, tokio::sync::RwLock::new(rx))
    };
}

static LOGGER: Logger = Logger;
pub static CONFIG: OnceCell<data::app::cfg::Config> = OnceCell::new();
pub static WLAN: OnceCell<Arc<win::Wlan>> = OnceCell::new();


#[tokio::main]
async fn main() {
    Logger::init().unwrap();
    app::init_fs().await;

    let config = CONFIG.get().unwrap();
    if let Err(reasons) = config.is_valid() {
        error!("x CONFIG is invalid due to the following reasons: {:?}", reasons);
        return;
    }

    WLAN.set(Arc::new(win::Wlan::new(win::wlan::ClientVersion::Second).unwrap())).unwrap();

    interface::start().await;
    network::start().await;
    
    if !interface::CHOSEN.write().await.is_chosen() {
        debug!("main calls dead because no interface");
        app::STATE.write().await.dead(app::DeadReason::NoInterface).unwrap()
    } else if !network::LIST.read().await.cfg_networks_available() {
        debug!("main calls dead because no cfg_networks_available");
        app::STATE.write().await.dead(app::DeadReason::NoNetwork).unwrap();
        network::event::waiter::spawn_event_loop().await;
    }

    std::thread::park();
}
