use crate::win::wlan::Network;
use crate::app::wlan::network::UpdateError;
use crate::app::wlan::interface;


pub struct Operator {
    list: Vec<Network>
}
impl Operator {
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