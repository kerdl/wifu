use crate::{win::wlan::{Network, self, network::profile::Key}, app};

use std::{sync::Arc, time::Duration};
use tokio::sync::{RwLock, broadcast};
use once_cell::sync::{OnceCell, Lazy};

use super::{interface::{NotificationWithInterface, self}, pinger};


pub static LIST: Lazy<Arc<RwLock<Vec<Network>>>> = Lazy::new(
    || Arc::new(RwLock::new(vec![]))
);
pub static CHOSEN_AS_SSID: Lazy<Arc<RwLock<Option<String>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);
pub static ACM_EVENT_LOOP_HANDLE: Lazy<Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);
pub static SCANNER_HANDLE: Lazy<Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);
pub static AUTOPILOT_HANDLE: Lazy<Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);
pub static UPDATE_SENDER: OnceCell<broadcast::Sender<NotificationWithInterface>> = OnceCell::new();
pub static UPDATE_RECV: OnceCell<broadcast::Receiver<NotificationWithInterface>> = OnceCell::new();


pub async fn init_globals() {
    let (sender, receiver) = broadcast::channel(64);

    UPDATE_SENDER.set(sender).unwrap();
    UPDATE_RECV.set(receiver).unwrap();

    if interface::chose_something().await {
        update_list().await;
    }
}

pub async fn update_list() {
    let wlan = crate::WLAN.get().unwrap();
    let chosen = interface::CHOSEN_AS_GUID.read().await;
    *LIST.write().await = wlan.available_networks(chosen.as_ref().unwrap()).unwrap();
}

pub async fn list_is_empty() -> bool {
    LIST.read().await.is_empty()
}

pub async fn clear_list() {
    *LIST.write().await = vec![]
}

pub async fn spawn_all_handles() {
    spawn_acm_event_loop_global().await;
    spawn_scanner_global().await;
    spawn_autopilot_global().await;
}

pub async fn close_all_handles() {
    close_acm_event_loop_global().await;
    close_scanner_global().await;
    close_autopilot_global().await;
}

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

pub async fn scanner() {
    let config = crate::CONFIG.get().unwrap();
    let wlan = crate::WLAN.get().unwrap();

    loop {
        let chosen = interface::CHOSEN_AS_GUID.read().await;
        let guid = chosen.as_ref().unwrap();
        let name = interface::get_name_by_guid(guid).await.unwrap();
        let result = wlan.scan(guid).await;

        if let Err(err) = result.as_ref() {
            println!("x SCAN error {:?} ({:?})", err, name)
        } else if !result.unwrap() {
            println!("x SCAN interrupted ({:?})", name)
        } else {
            println!("o SCAN completed ({:?})", name)
        }

        std::mem::drop(chosen);

        tokio::time::sleep(Duration::from_millis(config.wifi.scan.interval_ms)).await;
    }
}

pub fn spawn_scanner() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move { scanner().await })
}

pub async fn spawn_scanner_global() {
    *SCANNER_HANDLE.write().await = Some(spawn_scanner());
}

pub async fn close_scanner_global() {
    SCANNER_HANDLE.read().await.as_ref().map(|h| h.abort());
    *SCANNER_HANDLE.write().await = None;
}

pub async fn autopilot() {
    loop {
        if list_is_empty().await {
            wait_for_scan_list_refresh().await
        }

        println!("o PINGER: starting");

        //let list = LIST.read().await;

        //super::util::priority::choose(current, priority);

        if pinger::PINGER.read().await.has_no_ips() {
            wait_for_connection_complete().await;
        }

        println!("{:?}", crate::WLAN.get().unwrap().list_profiles(interface::CHOSEN_AS_GUID.read().await.as_ref().unwrap()));

        pinger::PINGER.read().await.start().await;

        println!("o PINGER: switching WI-FI...");
        choose_global(false).await;
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
    async fn wait_for_scan_list_refresh(
        UPDATE_RECV.get().unwrap().resubscribe(),
        wlan::acm::notification::Code::ScanListRefresh
    )
);
app::acm::wait_fn!(
    async fn wait_for_connection_complete(
        UPDATE_RECV.get().unwrap().resubscribe(),
        wlan::acm::notification::Code::ConnectionComplete
    )
);

pub async fn choose(current: Option<&str>) -> Option<String> {
    let wlan = crate::WLAN.get().unwrap();
    let chosen_interface = interface::CHOSEN_AS_GUID.read().await;
    let cfg_networks = crate::CONFIG.get().unwrap().wifi.networks.iter().map(|n| n.ssid.to_owned()).collect::<Vec<String>>();
    let available_networks = LIST.read().await;

    if available_networks.is_empty() {
        return None
    }

    let interfaces = LIST.read().await;
    
    let chosen_str = super::util::priority::choose(current, cfg_networks.as_slice()).unwrap();
    let chosen = interfaces.iter()
        .find(|net| net.ssid == chosen_str)
        .map(|net| net.ssid.to_owned());

    chosen
}

pub async fn choose_global(send_alive: bool) {
    let wlan = crate::WLAN.get().unwrap();
    let config = crate::CONFIG.get().unwrap();
    let chosen = CHOSEN_AS_SSID.read().await;
    let current = chosen.as_ref().map(|string| string.as_str());
    let mut old_chosen = CHOSEN_AS_SSID.write().await;
    let new_chosen = choose(current).await;

    let old_and_new_are_some = old_chosen.is_some() && new_chosen.is_some();
    let old_and_new_are_same = if old_and_new_are_some {
        old_chosen.as_ref().unwrap() == new_chosen.as_ref().unwrap()
    } else {
        false
    };

    if new_chosen.is_none() {
        panic!("network::choose_global() was called when there are no networks available")
    }

    let switch = if old_and_new_are_some { !old_and_new_are_same } else { true };
    if switch {
        *old_chosen = new_chosen;
        println!("o CHOSE {:?}", old_chosen.as_ref().unwrap());

        let list = LIST.read().await;
        let chosen_interface_lock = interface::CHOSEN_AS_GUID.read().await;
        let chosen_interface = chosen_interface_lock.as_ref().unwrap();
        let chosen_network_str = old_chosen.as_ref().unwrap().as_str();
        let chosen_network = list.iter().find(|net| &net.ssid == chosen_network_str);
        let chosen_network_clone = chosen_network.as_ref().unwrap().clone().clone();
        let chosen_network_cfg = config.wifi.networks.iter().find(|net| &net.ssid == chosen_network_str).unwrap();
        let chosen_network_key = chosen_network_cfg.password.as_ref().map(|pwd| Key::from_plain(pwd));
        let chosen_network_profile = chosen_network_clone.clone().to_profile(chosen_network_key);

        if !wlan.profile_exists(chosen_interface, chosen_network_str) {
            wlan.set_profile(chosen_interface, chosen_network_profile.clone()).unwrap();
        }

        wlan.connect(chosen_interface, &chosen_network_profile.name, &chosen_network_clone.bss).await;

        if send_alive && *super::IS_DEAD.read().await {
            super::alive(false).await;
        }
    }
}

pub async fn unchoose(current: &wlan::Network) {
    let mut chosen = CHOSEN_AS_SSID.write().await;
    if chosen.is_none() {
        panic!("network::unchoose() was called when there is no chosen network")
    }

    println!("x UNCHOSE WI-FI {:?}", current.ssid);
    *chosen = None;

    super::dead(app::DeadReason::NoNetwork).await;
}