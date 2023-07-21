use crate::win;
use crate::win::wlan;

use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use once_cell::sync::OnceCell;
use windows::core::GUID;


pub static LIST: OnceCell<Arc<RwLock<Vec<wlan::Interface>>>> = OnceCell::new();
pub static CHOSEN_AS_GUID: OnceCell<Arc<RwLock<Option<GUID>>>> = OnceCell::new();
pub static UPDATE_SENDER: OnceCell<broadcast::Sender<NotificationWithInterface>> = OnceCell::new();
pub static UPDATE_RECV: OnceCell<broadcast::Receiver<NotificationWithInterface>> = OnceCell::new();


#[derive(Debug, Clone)]
pub struct NotificationWithInterface {
    pub code: wlan::acm::notification::Code,
    pub interface: Option<wlan::Interface>,
}
impl NotificationWithInterface {
    pub async fn from_notification_global(notif: wlan::acm::Notification) -> Self {
        Self {
            code: notif.code,
            interface: LIST.get().unwrap().read().await.iter()
                .find(|iface| iface.guid == notif.guid)
                .map(|iface| iface.clone())
        }
    }
}

pub async fn get_name_by_guid(guid: &GUID) -> Option<String> {
    let interfaces = LIST.get().unwrap().read().await;
    interfaces.iter().find(|iface| &iface.guid == guid).map(|iface| iface.description.clone())
}

pub async fn acm_event_loop() {
    loop {
        let wlan = crate::WLAN.get().unwrap();
        let notif = wlan.acm_recv().await;
        let notif_with_interface = NotificationWithInterface::from_notification_global(notif.clone()).await;

        let mut interfaces = LIST.get().unwrap().write().await;

        match notif_with_interface.code {
            wlan::acm::notification::Code::InterfaceArrival => {
                *interfaces = wlan.list_interfaces().unwrap();
                let description = &interfaces.iter().find(
                    |i| i.guid == notif.guid
                ).unwrap().description;
                println!("+ CONNECTED {:?} ({:?})", description, notif.guid);
            },
            wlan::acm::notification::Code::InterfaceRemoval => {
                let description = &interfaces.iter().find(
                    |i| i.guid == notif_with_interface.interface.as_ref().unwrap().guid
                ).unwrap().description;
                println!("- DISCONNECTED {:?} ({:?})", description, notif.guid);
                *interfaces = wlan.list_interfaces().unwrap();
            },
            _ => ()
        }

        std::mem::drop(interfaces);
        UPDATE_SENDER.get().unwrap().send(notif_with_interface).unwrap();
    }
}

pub fn spawn_acm_event_loop() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move { acm_event_loop().await })
}

pub async fn autopilot() {
    let mut receiver = UPDATE_RECV.get().unwrap().resubscribe();

    loop {        
        let notif = receiver.recv().await.unwrap();

        match notif.code {
            wlan::acm::notification::Code::InterfaceArrival => {
                let chosen = CHOSEN_AS_GUID.get().unwrap().read().await;
                std::mem::drop(chosen);
                choose_global(notif.interface.as_ref()).await;
            },
            wlan::acm::notification::Code::InterfaceRemoval => {
                let chosen_lock = CHOSEN_AS_GUID.get().unwrap().read().await;
                if chosen_lock.is_none() { continue; }
                let chosen = chosen_lock.unwrap();

                let list = LIST.get().unwrap().read().await;

                if chosen == notif.interface.as_ref().unwrap().guid && list.is_empty() {
                    std::mem::drop(chosen_lock);
                    std::mem::drop(list);
                    unchoose(notif.interface.as_ref().unwrap()).await;
                } else if chosen == notif.interface.as_ref().unwrap().guid && !list.is_empty() {
                    std::mem::drop(chosen_lock);
                    std::mem::drop(list);
                    choose_global(notif.interface.as_ref()).await;
                }
            },
            _ => ()
        }
    }
}

pub fn spawn_autopilot() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move { autopilot().await })
}

pub async fn wait_for_arrival() {
    loop {
        let mut receiver = UPDATE_RECV.get().unwrap().resubscribe();

        match receiver.recv().await.unwrap().code {
            wlan::acm::notification::Code::InterfaceArrival => return,
            _ => ()
        }
    }
}

pub async fn choose(current: Option<&str>) -> Option<GUID> {
    let priority = crate::CONFIG.get().unwrap().interfaces.priority.as_slice();
    let interfaces = LIST.get().unwrap().read().await;

    if interfaces.is_empty() {
        return None
    }

    if priority.is_empty() {
        Some(interfaces.get(0).unwrap().guid)
    } else {
        let interfaces = LIST.get().unwrap().read().await;
        
        let chosen_str = super::util::priority::choose(current, priority).unwrap();
        let mut chosen = interfaces.iter()
            .find(|iface| &win::guid::to_string(&iface.guid) == chosen_str)
            .map(|iface| iface.guid);

        if chosen.is_none() {
            chosen = Some(interfaces.get(0).unwrap().guid);
        }

        chosen
    }
}

pub async fn choose_global(current: Option<&wlan::Interface>) {
    let mut chosen = CHOSEN_AS_GUID.get().unwrap().write().await;
    let new_chosen = choose(current.map(|iface| iface.description.as_str())).await;

    if chosen.is_some() && new_chosen.is_some() && chosen.as_ref().unwrap() == new_chosen.as_ref().unwrap() {
        return
    }  else if chosen.is_some() && new_chosen.is_some() && chosen.as_ref().unwrap() != new_chosen.as_ref().unwrap() {
        *chosen = new_chosen;
        println!("o CHOSE {:?} ({:?})", get_name_by_guid(&new_chosen.unwrap()).await, chosen.unwrap());
    } else if chosen.is_none() && new_chosen.is_some() {
        *chosen = new_chosen;
        println!("o CHOSE {:?} ({:?})", get_name_by_guid(&new_chosen.unwrap()).await, chosen.unwrap());
    } else {
        panic!("interface::choose_global() was called when no interfaces are connected")
    }
}

pub async fn unchoose(current: &wlan::Interface) {
    let mut chosen = CHOSEN_AS_GUID.get().unwrap().write().await;
    if chosen.is_none() {
        panic!("interface::unchoose() was called when there is no chosen interface")
    }

    println!("x UNCHOSE {:?} ({:?})", current.description, current.guid);
    *chosen = None;
}