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
    let mut receiver = super::RECEIVER.get().unwrap().resubscribe();

    loop {        
        let notif = receiver.recv().await.unwrap();

        match notif.code {
            AcmNotifCode::InterfaceArrival => {
                let mut chosen = interface::CHOSEN.write().await;
                chosen.choose();
            },
            AcmNotifCode::InterfaceRemoval => {
                network::LIST.write().await.clear();

                let list = interface::LIST.read().await;
                let chosen = interface::CHOSEN.read().await;

                if chosen.is_guid_chosen(&notif.interface.guid) && list.is_empty() {
                    std::mem::drop(list);
                    std::mem::drop(chosen);
                    interface::CHOSEN.write().await.unchoose().await.unwrap();
                } else if !list.is_empty() {
                    std::mem::drop(list);
                    std::mem::drop(chosen);
                    interface::CHOSEN.write().await.choose().await.unwrap();
                } else {
                    app::dead(app::DeadReason::NoInterface).await;
                }
            },
            _ => ()
        }
    }
}

event::looping::spawner!(async fn spawn_event_loop(HANDLE, event_loop));
event::looping::closer!(async fn close_event_loop(HANDLE));
