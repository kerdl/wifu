pub mod event;
pub mod error;
pub mod list;
pub mod chosen;
pub use error::UpdateError;

use crate::app::wlan::interface;

use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;


pub static LIST: Lazy<Arc<RwLock<list::Operator>>> = Lazy::new(
    || Arc::new(RwLock::new(list::Operator::default()))
);
pub static CHOSEN: Lazy<Arc<RwLock<chosen::Operator>>> = Lazy::new(
    || Arc::new(RwLock::new(chosen::Operator::default()))
);

pub async fn start_necessary() {
    event::autopilot::spawn_event_loop().await;
}

pub async fn start() {
    start_necessary().await;

    interface::CHOSEN.write().await.scan().await.unwrap();
    //CHOSEN.write().await.choose().await.unwrap();

    //event::pinger::spawn_event_loop().await;
}

pub async fn end() {
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
    end().await;
    start().await;
}