use crate::app::cfg;
use crate::app::{util::priority, wlan::{interface, network::LIST}};
use crate::win;
use crate::win::wlan::network::profile::Key;

pub struct Operator {
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
        let live_network = list.get_by_ssid(&cfgs_network.ssid).unwrap();

        if !iface.profile_exists(&cfgs_network.ssid) {
            let key = cfgs_network.password.as_ref().map(|pwd| Key::from_plain(pwd));
            let profile = live_network.clone().to_profile(key);
            iface.set_profile(profile).await.unwrap();
        }

        iface.connect(&cfgs_network.ssid, &live_network.bss).await
    }

    pub async fn choose(&mut self) -> Option<&str> {
        let list = LIST.read().await;
        let accessable_ssids = list.accessable_ssids();

        let chosen = priority::choose(
            self.chosen.as_ref().map(|s| s.as_str()),
            accessable_ssids.as_slice()
        ).unwrap();

        println!("o NETWORK: CHOSE {}", chosen);

        self.set(chosen.to_string());
        self.get()
    }

    pub async fn unchoose(&mut self) -> Result<(), ()> {
        if self.chosen.is_none() {
            return Err(())
        }

        println!("x NETWORK: UNCHOSE {}", self.get().unwrap());

        self.chosen = None;

        Ok(())
    }
}
impl Default for Operator {
    fn default() -> Self {
        Self { chosen: None }
    }
}