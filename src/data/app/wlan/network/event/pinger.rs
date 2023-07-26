use crate::app::pinger::PINGER;
use crate::app::wlan::event;
use crate::app::wlan::interface;
use crate::app::wlan::network::{LIST, CHOSEN};

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use once_cell::sync::Lazy;


pub static HANDLE: Lazy<Arc<RwLock<Option<JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);


pub async fn event_loop() {
    loop {
        if PINGER.read().await.has_no_ips() {
            interface::CHOSEN.read().await.scan().await.unwrap();

            if CHOSEN.write().await.choose().await.is_none() {
                return close_event_loop().await
            }

            PINGER.write().await.update_ips()
        }

        PINGER.read().await.start().await;

        interface::CHOSEN.read().await.scan().await.unwrap();

        if CHOSEN.write().await.choose().await.is_none() {
            return close_event_loop().await
        }
    }
}

event::looping::works!(async fn works(HANDLE));
event::looping::spawner!(async fn spawn_event_loop(HANDLE, event_loop, works));
event::looping::closer!(async fn close_event_loop(HANDLE));
