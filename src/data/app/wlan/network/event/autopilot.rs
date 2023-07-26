use crate::app;
use crate::app::wlan::event;
use crate::app::wlan::interface;
use crate::app::wlan::network::{LIST, CHOSEN, event::pinger};
use crate::win::wlan::acm::notification::Code as AcmNotifCode;

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use once_cell::sync::Lazy;


pub static HANDLE: Lazy<Arc<RwLock<Option<JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);


pub async fn event_loop() {
    let mut receiver = super::RECEIVER.get().unwrap().resubscribe();

    loop {
        let notif = receiver.recv().await.unwrap();

        if !interface::event::is_relevant(&notif.interface.guid).await {
            continue
        }

        match notif.code {
            AcmNotifCode::ScanListRefresh => {
                let app_state = app::STATE.read().await;

                let dead_because_no_network = {
                    app_state.is_dead()
                    && app_state.get_dead_reason().unwrap().is_no_network()
                };
                let cfg_networks_available = LIST.read().await.cfg_networks_available();

                if dead_because_no_network && cfg_networks_available {
                    CHOSEN.write().await.choose().await;
                    pinger::spawn_event_loop().await;
                }
            },
            _ => ()
        }
    }
}

event::looping::spawner!(async fn spawn_event_loop(HANDLE, event_loop));
event::looping::closer!(async fn close_event_loop(HANDLE));
