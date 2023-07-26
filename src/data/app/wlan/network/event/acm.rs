use crate::app::wlan::event;
use crate::app::wlan::interface;
use crate::app::wlan::network::{LIST, CHOSEN};
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

        if !interface::event::is_relevant(&notif.guid).await {
            continue
        }

        let notif_with_interface = NotificationWithInterface::from_notification_global(notif.clone()).await;

        match notif.code {
            AcmNotifCode::ScanComplete => {},
            AcmNotifCode::ScanFail => {},
            AcmNotifCode::ScanListRefresh => {
                LIST.write().await.update().await;
                println!("{:?}", LIST.read().await.as_ssids())
            }
            AcmNotifCode::ConnectionStart => {},
            AcmNotifCode::ConnectionComplete => {},
            AcmNotifCode::ConnectionAttemptFail => {},
            AcmNotifCode::Disconnecting => {},
            AcmNotifCode::Disconnected => {},
            _ => continue
        }

        SENDER.get().unwrap().send(notif_with_interface).unwrap();
    }
}

event::looping::spawner!(async fn spawn_event_loop(HANDLE, event_loop));
event::looping::closer!(async fn close_event_loop(HANDLE));
