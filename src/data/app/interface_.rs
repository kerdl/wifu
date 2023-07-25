use crate::app;
use crate::win;
use crate::win::wlan;
use crate::app::acm::NotificationWithInterface;

use std::sync::Arc;
use once_cell::sync::Lazy;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use once_cell::sync::OnceCell;
use windows::core::GUID;

use super::network;



pub static UPDATE_SENDER: OnceCell<broadcast::Sender<NotificationWithInterface>> = OnceCell::new();
pub static UPDATE_RECV: OnceCell<broadcast::Receiver<NotificationWithInterface>> = OnceCell::new();
pub static ACM_EVENT_LOOP_HANDLE: Lazy<Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);
pub static AUTOPILOT_HANDLE: Lazy<Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);


pub async fn init_globals() {
    let wlan = crate::WLAN.get().unwrap();
    let (sender, receiver) = broadcast::channel(64);

    *LIST.write().await = wlan.list_interfaces().unwrap();
    UPDATE_SENDER.set(sender).unwrap();
    UPDATE_RECV.set(receiver).unwrap();

    *IS_INITIALIZED.write().await = true;
}

pub async fn chose_something() -> bool {
    let chosen = CHOSEN_AS_GUID.read().await;
    chosen.is_some()
}

pub async fn scan() -> win::NativeResult<bool> {
    let wlan = crate::WLAN.get().unwrap();
    wlan.scan(CHOSEN_AS_GUID.read().await.as_ref().unwrap()).await
}

pub async fn spawn_event_handles() {
    acm::spawn_event_loop_global().await;
    spawn_autopilot_global().await;
}

pub async fn close_event_handles() {
    acm::close_event_loop_global().await;
    close_autopilot_global().await;
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
            network::close_event_handles().await;
            network::spawn_event_handles().await;
        }

        if send_alive && super::STATE.read().await.is_dead() {
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