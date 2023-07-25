use crate::win::wlan::acm::Notification as AcmNotif;
use crate::win::wlan::acm::notification::Code as AcmNotifCode;
use crate::win::wlan::Interface;
use crate::app::wlan::interface;


#[derive(Debug, Clone)]
pub struct NotificationWithInterface {
    pub code: AcmNotifCode,
    pub interface: Interface,
}
impl NotificationWithInterface {
    pub async fn from_notification_global(notif: AcmNotif) -> Self {
        Self {
            code: notif.code,
            interface: interface::LIST.read().await
                .get_by_guid(&notif.guid).await
                .unwrap()
        }
    }
}


macro_rules! wait_fn {
    (async fn $name:ident($recv:expr, $notification:path)) => {
        pub async fn $name() {
            loop {
                let mut receiver = $recv;

                let notif = receiver.recv().await.unwrap();
                let chosen = crate::data::app::interface::CHOSEN_AS_GUID.read().await;

                if notif.interface.guid != *chosen.as_ref().unwrap() {
                    continue
                }
        
                match notif.code {
                    $notification => return,
                    _ => ()
                }
            }
        }
    };
}


pub(crate) use wait_fn;