use crate::app;
use crate::app::wlan::event;
use crate::app::wlan::interface;
use crate::app::wlan::network::{LIST, CHOSEN};
use crate::win::wlan::acm::notification::Code as AcmNotifCode;

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use once_cell::sync::Lazy;


pub static HANDLE: Lazy<Arc<RwLock<Option<JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);


pub async fn event_loop() {
    loop {
        interface::CHOSEN.read().await.scan().await.unwrap();

        let any_cfg_networks = LIST.read().await.cfg_networks_available();

        if any_cfg_networks {
            if CHOSEN.write().await.choose().await.is_none() {
                continue
            } else {
                return close_event_loop().await;
            }
        }
    }
}

event::looping::spawner!(async fn spawn_event_loop(HANDLE, event_loop));
event::looping::closer!(async fn close_event_loop(HANDLE));
