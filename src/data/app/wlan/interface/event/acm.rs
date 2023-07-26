//! ## Main loop of ACM notifications
//! 
//! - Catches all events from `crate::WLAN`
//! - If this event is `InterfaceArrival` or `InterfaceRemoval`,
//!   updates global connected interfaces list
//! - Redirects events to the local channel

use crate::app::wlan::event;
use crate::app::wlan::interface::LIST;
use crate::app::wlan::acm::NotificationWithInterface;
use crate::win::wlan::acm::notification::Code as AcmNotifCode;

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast::Sender;
use tokio::task::JoinHandle;
use once_cell::sync::{Lazy, OnceCell};


pub static HANDLE: Lazy<Arc<RwLock<Option<JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);
static SENDER: OnceCell<Sender<NotificationWithInterface>> = OnceCell::new();


pub fn set_sender(sender: Sender<NotificationWithInterface>) {
    SENDER.set(sender).unwrap()
}

pub async fn event_loop() {
    loop {
        let wlan = crate::WLAN.get().unwrap();
        let notif = wlan.acm_recv().await;
        let notif_with_interface;

        match notif.code {
            AcmNotifCode::InterfaceArrival => {
                if LIST.write().await.update_warned().await.is_err() {
                    continue;
                }

                let description = LIST.read().await
                    .get_name_by_guid(&notif.guid).unwrap();

                println!("+ INTERFACE: CONNECTED {:?} (GUID {:?})", description, notif.guid);

                notif_with_interface = {
                    NotificationWithInterface::from_notification_global(notif.clone()).await
                };
            },
            AcmNotifCode::InterfaceRemoval => {
                notif_with_interface = {
                    NotificationWithInterface::from_notification_global(notif.clone()).await
                };
                
                let description = LIST.read().await
                    .get_name_by_guid(&notif.guid).unwrap();

                println!("- INTERFACE: DISCONNECTED {:?} (GUID {:?})", description, notif.guid);

                if LIST.write().await.update_warned().await.is_err() {
                    continue;
                }
            },
            _ => continue
        }

        SENDER.get().unwrap().send(notif_with_interface).unwrap();
    }
}

event::looping::spawner!(async fn spawn_event_loop(HANDLE, event_loop));
event::looping::closer!(async fn close_event_loop(HANDLE));
