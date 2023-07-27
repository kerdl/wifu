pub mod event;
pub mod error;
pub mod list;
pub mod chosen;
pub use error::UpdateError;

use crate::app::wlan::interface;

use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use log::debug;


pub static LIST: Lazy<Arc<RwLock<list::Operator>>> = Lazy::new(
    || Arc::new(RwLock::new(list::Operator::default()))
);
pub static CHOSEN: Lazy<Arc<RwLock<chosen::Operator>>> = Lazy::new(
    || Arc::new(RwLock::new(chosen::Operator::default()))
);

pub async fn start_necessary() {
    debug!("network::start_necessary()");
    event::autopilot::spawn_event_loop().await;
}

pub async fn start() {
    debug!("network::start()");
    start_necessary().await;

    debug!("network::start(): scanning on chosen interface {:?}", interface::CHOSEN.read().await.get());
    interface::CHOSEN.write().await.scan().await.unwrap();
    debug!("network::start(): updating network list on chosen interface");
    LIST.write().await.update().await.unwrap();
    //CHOSEN.write().await.choose().await.unwrap();

    //event::pinger::spawn_event_loop().await;
}

pub async fn end() {
    debug!("network::end()");

    if event::autopilot::works().await {
        event::autopilot::close_event_loop().await;
    }

    if event::pinger::works().await {
        event::pinger::close_event_loop().await;
    }

    if event::waiter::works().await {
        event::waiter::close_event_loop().await;
    }

    if CHOSEN.read().await.is_chosen() {
        CHOSEN.write().await.unchoose().await.unwrap()
    }

    LIST.write().await.clear()
}

pub async fn restart() {
    debug!("network::restart()");
    end().await;
    start().await;
}