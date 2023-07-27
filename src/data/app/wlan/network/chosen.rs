use crate::app::cfg;
use crate::app::{util::priority, wlan::{interface, network::LIST}};
use crate::win;
use crate::win::wlan::network::profile::Key;

use log::{debug, info};

pub struct Operator {
    choosing: bool,
    chosen: Option<String>
}
impl Operator {
    pub fn get(&self) -> Option<&str> {
        self.chosen.as_ref().map(|s| s.as_str())
    }

    fn set(&mut self, ssid: String) {
        self.chosen = Some(ssid)
    }

    pub fn is_chosen(&self) -> bool {
        self.chosen.is_some()
    }

    pub fn is_choosing(&self) -> bool {
        self.choosing
    }

    pub fn is_ssid_chosen(&self, ssid: &str) -> bool {
        self.chosen.as_ref().map(|chosen| chosen == ssid).unwrap_or(false)
    }

    pub fn configs_network(&self) -> Option<&cfg::Network> {
        let config = crate::CONFIG.get().unwrap();

        self.chosen.as_ref().map(
            |chosen| config.wifi.networks.iter().find(|net| &net.ssid == chosen)
        ).flatten()
    }

    pub async fn connect(&self) -> win::NativeResult<bool> {
        assert!(self.chosen.is_some());

        let iface = interface::CHOSEN.read().await;
        let list = LIST.read().await;
 
        let cfgs_network = self.configs_network().unwrap();
        let live_network = list.get_by_ssid(&cfgs_network.ssid);
        if live_network.is_none() {
            return Ok(false)
        }
        let live_network = live_network.unwrap();

        if !iface.profile_exists(&cfgs_network.ssid) {
            let key = cfgs_network.password.as_ref().map(|pwd| Key::from_plain(pwd));
            let profile = live_network.clone().to_profile(key);
            iface.set_profile(profile).await.unwrap();
        }

        iface.connect(&cfgs_network.ssid, &live_network.bss).await
    }

    pub async fn choose(&mut self) -> Option<&str> {
        self.choosing = true;

        loop {
            let accessable_ssids = LIST.read().await.accessable_ssids();

            let mut current = self.chosen.as_ref().map(|s| s.as_str());
            debug!("network::choose(): initial current={:?}", current);

            current = priority::choose(
                current,
                accessable_ssids.as_slice()
            ).ok();
            debug!("network::choose(): after priority current={:?}", current);

            if current.is_none() {
                debug!("network::choose(): current is none, returning");
                return None
            }

            self.set(current.unwrap().to_string());
            debug!("network::choose(): set current");
    
            let result = self.connect().await;
            if result.is_err() || result.is_ok() && !result.unwrap() {
                debug!("network::choose(): connection failed, retrying");
                continue
            }

            debug!("network::choose(): chosen adapter is {:?}", interface::CHOSEN.read().await.get_interface().await);

            info!("o NETWORK: CHOSE {}", self.get().unwrap());

            break
        }

        self.choosing = false;
        self.get()
    }

    pub async fn unchoose(&mut self) -> Result<(), ()> {
        if self.chosen.is_none() {
            return Err(())
        }

        info!("x NETWORK: UNCHOSE {}", self.get().unwrap());

        self.chosen = None;

        Ok(())
    }
}
impl Default for Operator {
    fn default() -> Self {
        Self { chosen: None, choosing: false }
    }
}