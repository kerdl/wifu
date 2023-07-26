use crate::app::cfg;
use crate::app::wlan::network::UpdateError;
use crate::app::wlan::interface;
use crate::win::wlan::Network;


pub struct Operator {
    list: Vec<Network>
}
impl Operator {
    pub fn as_slice(&self) -> &[Network] {
        self.list.as_slice()
    }

    pub fn as_ssids(&self) -> Vec<&str> {
        self.list.iter().map(|net| net.ssid.as_str()).collect::<Vec<&str>>()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn get_by_ssid(&self, ssid: &str) -> Option<&Network> {
        self.list.iter().find(|net| net.ssid == ssid)
    }

    pub fn map_with_config(&self) -> Vec<(cfg::Network, Network)> {
        let mut v = vec![];
    
        for net in crate::CONFIG.get().unwrap().wifi.networks.iter() {
            let corresponding_result = self.list.iter()
                .find(|live_net| live_net.ssid == net.ssid)
                .clone();
            if corresponding_result.is_none() { continue }
            let corresponding = corresponding_result.unwrap();
    
            v.push((net.clone(), corresponding.clone()));
        }

        v
    }

    pub fn cfg_networks_available(&self) -> bool {
        !self.map_with_config().is_empty()
    }

    pub fn accessable_ssids(&self) -> Vec<String> {
        self.map_with_config().iter().map(
            |(cfg_net, _live_net)| cfg_net.ssid.clone()
        ).collect::<Vec<String>>()
    }

    pub fn clear(&mut self) {
        self.list = vec![];
    }

    pub async fn update(&mut self) -> Result<(), UpdateError> {
        let wlan = crate::WLAN.get().unwrap();
        let chosen_interface = interface::CHOSEN.read().await;

        if chosen_interface.get().is_none() {
            return Err(UpdateError::Interface(interface::Error::NotChosen))
        }

        let networks = wlan.available_networks(chosen_interface.get().unwrap());

        if let Err(err) = networks {
            return Err(UpdateError::Win(err))
        }

        self.list = networks.unwrap();
    
        Ok(())
    }
}
impl Default for Operator {
    fn default() -> Self {
        Self { list: vec![] }
    }
}