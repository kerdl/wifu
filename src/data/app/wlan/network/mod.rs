pub mod event;
pub mod error;
pub mod list;
pub mod chosen;
pub use error::UpdateError;

use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;


pub static IS_INITIALIZED: Lazy<Arc<RwLock<bool>>> = Lazy::new(
    || Arc::new(RwLock::new(false))
);
pub static LIST: Lazy<Arc<RwLock<list::Operator>>> = Lazy::new(
    || Arc::new(RwLock::new(list::Operator::default()))
);
pub static CHOSEN: Lazy<Arc<RwLock<chosen::Operator>>> = Lazy::new(
    || Arc::new(RwLock::new(chosen::Operator::default()))
);


pub async fn init() {
    event::init();
    LIST.write().await.update_warned().await.unwrap();
}

// --------- Available network list operations ---------
pub async fn update_list() {
    let wlan = crate::WLAN.get().unwrap();
    let chosen = interface::CHOSEN_AS_GUID.read().await;
    *LIST.write().await = wlan.available_networks(chosen.as_ref().unwrap()).unwrap();
}

pub async fn list_is_empty() -> bool {
    let list = LIST.read().await;
    println!("network::list_is_empty(): {:?}", list);
    list.is_empty()
}

pub async fn clear_list() {
    *LIST.write().await = vec![]
}

pub async fn get_by_name(name: &str) -> Option<Network> {
    LIST.read().await.iter().find(|net| net.ssid == name).map(|net| net.clone())
}

pub async fn map_list_with_config() -> Vec<(super::cfg::WiFiNetwork, wlan::Network)> {
    let list = LIST.read().await;
    let mut v = vec![];

    for net in crate::CONFIG.get().unwrap().wifi.networks.iter() {
        let corresponding_result = list.iter()
            .find(|live_net| live_net.ssid == net.ssid)
            .clone();
        if corresponding_result.is_none() { continue }
        let corresponding = corresponding_result.unwrap();

        v.push((net.clone(), corresponding.clone()));
    }

    v
}

pub async fn cfg_networks_available() -> bool {
    !map_list_with_config().await.is_empty()
}


pub async fn cfg_network_wait() {
    println!("cfg_network_wait()");

    loop {
        println!("cfg_network_wait(): scanning...");
        interface::scan().await.unwrap();

        let any_cfg_networks = cfg_networks_available().await;
        println!("cfg_network_wait(): any_cfg_networks: {}", any_cfg_networks);

        if any_cfg_networks {
            println!("cfg_network_wait(): choosing global...");
            let result = choose_global(true).await;
            println!("cfg_network_wait(): choosing global is {}", result);
            return close_cfg_network_wait_global().await;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

pub fn spawn_cfg_network_wait() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move { cfg_network_wait().await })
}

pub async fn spawn_cfg_network_wait_global() {
    *CFG_NETWORK_WAIT_HANDLE.write().await = Some(spawn_cfg_network_wait());
}

pub async fn close_cfg_network_wait_global() {
    CFG_NETWORK_WAIT_HANDLE.read().await.as_ref().map(|h| h.abort());
    *CFG_NETWORK_WAIT_HANDLE.write().await = None;
}

app::acm::wait_fn!(
    async fn wait_for_scan_complete(
        UPDATE_RECV.get().unwrap().resubscribe(),
        wlan::acm::notification::Code::ScanComplete
    )
);

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
    let maps = map_list_with_config().await;

    if maps.is_empty() {
        return None
    }

    let live_known_networks = maps.iter().map(|map| map.1.ssid.clone()).collect::<Vec<String>>();
    
    let chosen_str = super::util::priority::choose(
        current,
        live_known_networks.as_slice()
    ).unwrap();
    let chosen = get_by_name(chosen_str).await?;

    Some(chosen.ssid)
}

pub async fn choose_global(send_alive: bool) -> bool {
    loop {
        let wlan = crate::WLAN.get().unwrap();
        let config = crate::CONFIG.get().unwrap();
        let mut chosen = CHOSEN_AS_SSID.write().await;
        let chosen_as_str = chosen.as_ref().map(|string| string.as_str());

        let new_chosen = choose(chosen_as_str).await;
    
        let old_and_new_are_some = chosen.is_some() && new_chosen.is_some();
        let old_and_new_are_same = if old_and_new_are_some {
            chosen.as_ref().unwrap() == new_chosen.as_ref().unwrap()
        } else {
            false
        };
    
        if new_chosen.is_none() {
            println!("choose_global(): new_chosen is None");

            if chosen.is_some() {
                println!("choose_global(): unchoosing current");
                std::mem::drop(chosen);
                unchoose().await;
            } else {
                println!("choose_global(): setting dead state");
                super::dead(app::DeadReason::NoNetwork).await;
            }

            return false
        }
    
        let switch = if old_and_new_are_some { !old_and_new_are_same } else { true };

        if switch {
            println!("choose_global(): switching...");

            *chosen = new_chosen;
            println!("o CHOSE {:?}", chosen.as_ref().unwrap());
    
            let list = LIST.read().await;
            let chosen_interface_lock = interface::CHOSEN_AS_GUID.read().await;
            let chosen_interface = chosen_interface_lock.as_ref().unwrap();
            let chosen_network_str = chosen.as_ref().unwrap().as_str();
            let chosen_network = list.iter().find(|net| &net.ssid == chosen_network_str);
            let chosen_network_clone = chosen_network.as_ref().unwrap().clone().clone();
            let chosen_network_cfg = config.wifi.networks.iter().find(|net| &net.ssid == chosen_network_str).unwrap();
            let chosen_network_key = chosen_network_cfg.password.as_ref().map(|pwd| Key::from_plain(pwd));
            let chosen_network_profile = chosen_network_clone.clone().to_profile(chosen_network_key);
    
            if !wlan.profile_exists(chosen_interface, chosen_network_str) {
                wlan.set_profile(chosen_interface, chosen_network_profile.clone()).unwrap();
            }
    
            println!("choose_global(): connecting...");
            let result = wlan.connect(
                chosen_interface,
                &chosen_network_profile.name,
                &chosen_network_clone.bss
            ).await;
    
            if result.is_err() || !result.unwrap() {
                println!("choose_global(): error connecting, rechoosing network");
                continue;
            }
    
            if send_alive && super::STATE.read().await.is_dead() {
                super::alive(false).await;
            }
        }
    
        return true
    }
}

pub async fn unchoose() {
    let mut chosen = CHOSEN_AS_SSID.write().await;
    if chosen.is_none() {
        panic!("network::unchoose() was called when there is no chosen network")
    }

    println!("x UNCHOSE WI-FI {:?}", chosen.as_ref().unwrap());
    *chosen = None;

    super::dead(app::DeadReason::NoNetwork).await;
}