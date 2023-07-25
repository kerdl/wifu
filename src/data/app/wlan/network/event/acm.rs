use crate::app::wlan::event;
use crate::app::wlan::network::LIST;
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


pub async fn acm_event_loop() {
    loop {
        let wlan = crate::WLAN.get().unwrap();
        let notif = wlan.acm_recv().await;
        if !interface::guid_is_in_list(&notif.guid).await {
            continue
        }
        let notif_with_interface = NotificationWithInterface::from_notification_global(notif.clone()).await;

        if !interface::is_chosen(&notif.guid).await { continue }

        match notif.code {
            wlan::acm::notification::Code::ScanComplete => {},
            wlan::acm::notification::Code::ScanFail => {},
            wlan::acm::notification::Code::ScanListRefresh => {
                update_list().await;
                println!("{:?}", LIST.read().await.iter().map(|n| n.ssid.to_owned()).collect::<Vec<String>>())
                //println!("{:#?}", LIST.read().await);
            }
            wlan::acm::notification::Code::ConnectionStart => {},
            wlan::acm::notification::Code::ConnectionComplete => {},
            wlan::acm::notification::Code::ConnectionAttemptFail => {},
            wlan::acm::notification::Code::Disconnecting => {},
            wlan::acm::notification::Code::Disconnected => {},
            _ => continue
        }

        UPDATE_SENDER.get().unwrap().send(notif_with_interface).unwrap();
    }
}

event::spawner!(async fn spawn_event_loop(HANDLE, event_loop));
event::closer!(async fn close_event_loop(HANDLE));
