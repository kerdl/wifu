use crate::win;
use crate::win::guid;
use crate::win::wlan::Interface;

use windows::core::GUID;


pub struct Operator {
    list: Vec<Interface>
}
impl Operator {
    pub fn as_slice(&self) -> &[Interface] {
        self.list.as_slice()
    }

    pub fn as_guids(&self) ->Vec<GUID> {
        self.list.as_slice().iter()
            .map(|iface| iface.guid)
            .collect::<Vec<GUID>>()
    }

    pub fn as_guid_strings(&self) -> Vec<String> {
        self.as_guids().iter()
            .map(|guid| guid::to_string(guid))
            .collect::<Vec<String>>()
    }

    pub fn get_by_guid(&self, guid: &GUID) -> Option<Interface> {
        self.list.iter()
            .find(|iface| iface.guid == *guid)
            .map(|iface| iface.clone())
    }

    pub fn get_by_str_guid(&self, guid: &str) -> Option<Interface> {
        self.list.iter()
            .find(|iface| guid::to_string(&iface.guid) == guid)
            .map(|iface| iface.clone())
    }

    pub fn get_name_by_guid(&self, guid: &GUID) -> Option<String> {
        Some(self.get_by_guid(guid)?.description)
    }

    pub fn contains_guid(&self, guid: &GUID) -> bool {
        self.list.iter().find(|iface| &iface.guid == guid).is_some()
    }

    pub fn contains_str_guid(&self, guid: &str) -> bool {
        self.list.iter().find(|iface| guid::to_string(&iface.guid) == guid).is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn sorted_priority(&self) -> Vec<Interface> {
        let config = crate::CONFIG.get().unwrap();
        let mut prioritized = vec![];

        for guid_str in config.interfaces.priority.iter() {
            let result = self.get_by_str_guid(guid_str);
            if result.is_none() { continue }
            prioritized.push(result.unwrap())
        }

        let prioritized_guids = prioritized.iter()
            .map(|iface| iface.guid)
            .collect::<Vec<GUID>>();

        let mut others = self.list.clone();
        others.retain(|iface| !prioritized_guids.contains(&iface.guid));

        prioritized.extend_from_slice(&others);

        prioritized
    }

    pub fn sorted_priority_guids(&self) -> Vec<GUID> {
        self.sorted_priority().iter()
            .map(|iface| iface.guid)
            .collect::<Vec<GUID>>()
    }

    pub fn sorted_priority_string_guids(&self) -> Vec<String> {
        self.sorted_priority_guids().iter()
            .map(|guid| guid::to_string(guid))
            .collect::<Vec<String>>()
    }

    pub async fn disconnect_all_except(&self, guid: &GUID) {
        println!("interface::disconnect_all_except({:?})", guid);
        let wlan = crate::WLAN.get().unwrap();

        for iface in self.list.iter() {
            if &iface.guid == guid {
                println!("interface::disconnect_all_except() not disconnecting {:?}", guid);
                continue;
            }

            if let Err(err) = wlan.disconnect(&iface.guid).await {
                println!("interface::disconnect_all_except(): warning, cannot disconnect {:?} ({:?})", &iface.guid, err)
            }
        }
    }

    pub fn update(&mut self) -> win::NativeResult<()> {
        let wlan = crate::WLAN.get().unwrap();
        self.list = wlan.list_interfaces()?;
    
        Ok(())
    }
    
    pub async fn update_warned(&mut self) -> win::NativeResult<()> {
        let result = self.update();
    
        if let Err(err) = result.as_ref() {
            println!("x INTERFACE list could not be updated: {:?}", err);
        }
    
        result
    }
}
impl Default for Operator {
    fn default() -> Self {
        Self { list: vec![] }
    }
}