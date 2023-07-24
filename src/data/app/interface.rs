use crate::app;
use crate::win;
use crate::win::wlan;

use std::sync::Arc;
use once_cell::sync::Lazy;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use once_cell::sync::OnceCell;
use windows::core::GUID;

use super::network;


pub static LIST: Lazy<Arc<RwLock<Vec<wlan::Interface>>>> = Lazy::new(
    || Arc::new(RwLock::new(vec![]))
);
pub static CHOSEN_AS_GUID: Lazy<Arc<RwLock<Option<GUID>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);
pub static UPDATE_SENDER: OnceCell<broadcast::Sender<NotificationWithInterface>> = OnceCell::new();
pub static UPDATE_RECV: OnceCell<broadcast::Receiver<NotificationWithInterface>> = OnceCell::new();
pub static ACM_EVENT_LOOP_HANDLE: Lazy<Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);
pub static AUTOPILOT_HANDLE: Lazy<Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);


#[derive(Debug, Clone)]
pub struct NotificationWithInterface {
    pub code: wlan::acm::notification::Code,
    pub interface: wlan::Interface,
}
impl NotificationWithInterface {
    pub async fn from_notification_global(notif: wlan::acm::Notification) -> Self {
        Self {
            code: notif.code,
            interface: LIST.read().await.iter()
                .find(|iface| iface.guid == notif.guid)
                .map(|iface| iface.clone())
                .unwrap()
        }
    }
}

pub async fn init_globals() {
    let wlan = crate::WLAN.get().unwrap();
    let (sender, receiver) = broadcast::channel(64);

    *LIST.write().await = wlan.list_interfaces().unwrap();
    UPDATE_SENDER.set(sender).unwrap();
    UPDATE_RECV.set(receiver).unwrap();

    if !list_is_empty().await {
        choose_global(None, false, false).await;
    }
}

pub async fn spawn_all_handles() {
    spawn_acm_event_loop_global().await;
    spawn_autopilot_global().await;
}

pub async fn close_all_handles() {
    close_acm_event_loop_global().await;
    close_autopilot_global().await;
}

pub async fn get_name_by_guid(guid: &GUID) -> Option<String> {
    let interfaces = LIST.read().await;
    interfaces.iter().find(|iface| &iface.guid == guid).map(|iface| iface.description.clone())
}

pub async fn guid_is_in_list(guid: &GUID) -> bool {
    let interfaces = LIST.read().await;
    interfaces.iter().find(|iface| &iface.guid == guid).is_some()
}

pub async fn is_chosen(guid: &GUID) -> bool {
    let chosen = CHOSEN_AS_GUID.read().await;
    if chosen.is_none() { return false }
    chosen.as_ref().unwrap() == guid
}

pub async fn chose_something() -> bool {
    let chosen = CHOSEN_AS_GUID.read().await;
    chosen.is_some()
}

pub async fn list_is_empty() -> bool {
    LIST.read().await.is_empty()
}

pub async fn acm_event_loop() {
    loop {
        let wlan = crate::WLAN.get().unwrap();
        let notif = wlan.acm_recv().await;
        let notif_with_interface;

        match notif.code {
            wlan::acm::notification::Code::InterfaceArrival => {
                let mut interfaces = LIST.write().await;
                *interfaces = wlan.list_interfaces().unwrap();
                let description = &interfaces.iter().find(
                    |i| i.guid == notif.guid
                ).unwrap().description;
                println!("+ CONNECTED {:?} ({:?})", description, notif.guid);
                std::mem::drop(interfaces);
                notif_with_interface = NotificationWithInterface::from_notification_global(notif.clone()).await;
            },
            wlan::acm::notification::Code::InterfaceRemoval => {
                notif_with_interface = NotificationWithInterface::from_notification_global(notif.clone()).await;
                let mut interfaces = LIST.write().await;
                let description = &interfaces.iter().find(
                    |i| i.guid == notif_with_interface.interface.guid
                ).unwrap().description;
                println!("- DISCONNECTED {:?} ({:?})", description, notif.guid);
                *interfaces = wlan.list_interfaces().unwrap();
            },
            _ => continue
        }

        UPDATE_SENDER.get().unwrap().send(notif_with_interface).unwrap();
    }
}

pub fn spawn_acm_event_loop() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move { acm_event_loop().await })
}

pub async fn spawn_acm_event_loop_global() {
    *ACM_EVENT_LOOP_HANDLE.write().await = Some(spawn_acm_event_loop());
}

pub async fn close_acm_event_loop_global() {
    ACM_EVENT_LOOP_HANDLE.read().await.as_ref().map(|h| h.abort());
    *ACM_EVENT_LOOP_HANDLE.write().await = None;
}

pub async fn autopilot() {
    let mut receiver = UPDATE_RECV.get().unwrap().resubscribe();

    loop {        
        let notif = receiver.recv().await.unwrap();

        match notif.code {
            wlan::acm::notification::Code::InterfaceArrival => {
                let chosen = CHOSEN_AS_GUID.read().await;
                std::mem::drop(chosen);
                choose_global(Some(&notif.interface), true, true).await;
            },
            wlan::acm::notification::Code::InterfaceRemoval => {
                network::clear_list().await;

                let chosen_lock = CHOSEN_AS_GUID.read().await;
                if chosen_lock.is_none() { continue; }
                let chosen = chosen_lock.unwrap();

                let list = LIST.read().await;

                if chosen == notif.interface.guid && list.is_empty() {
                    std::mem::drop(chosen_lock);
                    std::mem::drop(list);
                    unchoose(&notif.interface).await;
                } else if chosen == notif.interface.guid && !list.is_empty() {
                    std::mem::drop(chosen_lock);
                    std::mem::drop(list);
                    choose_global(Some(&notif.interface), true, true).await;
                }
            },
            _ => ()
        }
    }
}

pub fn spawn_autopilot() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move { autopilot().await })
}

pub async fn spawn_autopilot_global() {
    *AUTOPILOT_HANDLE.write().await = Some(spawn_autopilot());
}

pub async fn close_autopilot_global() {
    AUTOPILOT_HANDLE.read().await.as_ref().map(|h| h.abort());
    *AUTOPILOT_HANDLE.write().await = None;
}

app::acm::wait_fn!(
    async fn wait_for_arrival(
        UPDATE_RECV.get().unwrap().resubscribe(),
        wlan::acm::notification::Code::InterfaceArrival
    )
);

pub async fn choose(current: Option<&str>) -> Option<GUID> {
    let priority = crate::CONFIG.get().unwrap().interfaces.priority.as_slice();
    let interfaces = LIST.read().await;

    if interfaces.is_empty() {
        return None
    }

    if priority.is_empty() {
        Some(interfaces.get(0).unwrap().guid)
    } else {
        let interfaces = LIST.read().await;
        
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

pub async fn choose_global(
    current: Option<&wlan::Interface>,
    send_alive: bool,
    restart_network: bool
) {
    let mut old_chosen = CHOSEN_AS_GUID.write().await;
    let new_chosen = choose(current.map(|iface| iface.description.as_str())).await;

    let old_and_new_are_some = old_chosen.is_some() && new_chosen.is_some();
    let old_and_new_are_same = if old_and_new_are_some {
        old_chosen.as_ref().unwrap() == new_chosen.as_ref().unwrap()
    } else {
        false
    };

    if new_chosen.is_none() {
        panic!("interface::choose_global() was called when no interfaces are connected")
    }

    let switch = if old_and_new_are_some { !old_and_new_are_same } else { true };
    if switch {
        *old_chosen = new_chosen;
        let name = get_name_by_guid(&new_chosen.unwrap()).await;
        println!("o CHOSE {:?} ({:?})", name.unwrap(), old_chosen.unwrap());

        if restart_network {
            network::close_all_handles().await;
            network::spawn_all_handles().await;
        }

        if send_alive && *super::IS_DEAD.read().await {
            super::alive(false).await
        }
    }
}

pub async fn unchoose(current: &wlan::Interface) {
    let mut chosen = CHOSEN_AS_GUID.write().await;
    if chosen.is_none() {
        panic!("interface::unchoose() was called when there is no chosen interface")
    }

    println!("x UNCHOSE INTERFACE {:?} ({:?})", current.description, current.guid);
    *chosen = None;

    super::dead(app::DeadReason::NoInterface).await;
}