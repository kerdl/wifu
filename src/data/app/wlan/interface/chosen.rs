use crate::app::interface::LIST;
use crate::win::{self, guid};

use windows::core::GUID;


pub struct Operator {
    chosen: Option<GUID>,
    name: Option<String>
}
impl Operator {
    pub fn get(&self) -> Option<&GUID> {
        self.chosen.as_ref()
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| s.as_str())
    }

    fn set_guid(&mut self, guid: GUID) {
        self.chosen = Some(guid);
    }

    fn set_name(&mut self, name: String) {
        self.name = Some(name)
    }

    pub fn as_string(&self) -> Option<String> {
        self.chosen.map(|chosen| guid::to_string(&chosen))
    }

    pub fn is_chosen(&self) -> bool {
        self.chosen.is_some()
    }

    pub fn is_guid_chosen(&self, guid: &GUID) -> bool {
        self.chosen.map(|chosen| &chosen == guid).unwrap_or(false)
    }

    pub async fn scan(&self) -> win::NativeResult<bool> {
        let wlan = crate::WLAN.get().unwrap();

        if let Some(chosen) = &self.chosen {
            wlan.scan(chosen).await
        } else {
            Err(win::NativeError::NotFound)
        }
    }

    pub async fn choose(&mut self) -> Option<&GUID> {
        let list = LIST.read().await;
        let priority_sorted = list.sorted_priority_string_guids();

        if priority_sorted.is_empty() {
            return None
        }

        let interface = list.get_by_str_guid(priority_sorted.get(0).unwrap());

        if let Some(iface) = interface {
            let is_same = self.chosen.as_ref().map(|guid| guid == &iface.guid).unwrap_or(false);
            if is_same {
                return self.get()
            }

            self.set_guid(iface.guid);
            self.set_name(iface.description);

            let name = list.get_name_by_guid(&iface.guid).unwrap();
            let guid_string = guid::to_string(&iface.guid);
            println!("o INTERFACE: CHOSE {} (GUID: {})", name, guid_string);

            self.get()
        } else {
            None
        }
    }

    pub async fn unchoose(&mut self) -> Result<(), ()> {
        if self.chosen.is_none() {
            return Err(())
        }

        let guid_string = self.as_string().unwrap();
        println!("x INTERFACE: UNCHOSE {} (GUID: {})", self.name().unwrap(), guid_string);

        self.chosen = None;

        Ok(())
    }
}
impl Default for Operator {
    fn default() -> Self {
        Self { chosen: None, name: None }
    }
}
