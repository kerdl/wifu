use crate::app;
use crate::app::wlan::event;
use crate::app::wlan::interface;
use crate::app::wlan::network;
use crate::win::wlan::acm::notification::Code as AcmNotifCode;

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use once_cell::sync::Lazy;


pub static HANDLE: Lazy<Arc<RwLock<Option<JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);


pub async fn event_loop() {
    let mut receiver = UPDATE_RECV.get().unwrap().resubscribe();

    loop {
        let notif = receiver.recv().await.unwrap();
        if !interface::is_chosen(&notif.interface.guid).await { continue; }

        match notif.code {
            wlan::acm::notification::Code::ScanListRefresh => {
                let dead_because_no_networks = app::STATE.read().await.dead_because_no_network();
                let cfg_networks_available = cfg_networks_available().await;

                if dead_because_no_networks && cfg_networks_available {
                    choose_global(true).await;
                    spawn_pinger_global().await;
                }
            },
            _ => ()
        }
    }
}

event::spawner!(async fn spawn_event_loop(HANDLE, event_loop));
event::closer!(async fn close_event_loop(HANDLE));
