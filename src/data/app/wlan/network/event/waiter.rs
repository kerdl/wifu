use crate::app;
use crate::app::wlan::event;
use crate::app::wlan::interface;
use crate::app::wlan::network::{LIST, CHOSEN};
use crate::win::wlan::acm::notification::Code as AcmNotifCode;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use once_cell::sync::Lazy;


pub static HANDLE: Lazy<Arc<RwLock<Option<JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);


pub async fn event_loop() {
    loop {
        interface::CHOSEN.read().await.scan().await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

event::looping::works!(async fn works(HANDLE));
event::looping::spawner!(async fn spawn_event_loop(HANDLE, event_loop, works));
event::looping::closer!(async fn close_event_loop(HANDLE));
