use crate::win::guid;
use crate::app::util::priority;

use windows::core::GUID;


pub struct Operator {
    chosen: Option<GUID>
}
impl Operator {
    pub fn get(&self) -> Option<&GUID> {
        self.chosen.as_ref()
    }

    fn set(&self, guid: GUID) {
        self.chosen = Some(guid)
    }

    pub fn as_string(&self) -> Option<String> {
        self.chosen.map(|chosen| guid::to_string(&chosen))
    }

    pub fn as_str(&self) -> Option<&str> {
        self.as_string().map(|s| s.as_str())
    }

    pub fn is_chosen(&self) -> bool {
        self.chosen.is_some()
    }

    pub fn is_guid_chosen(&self, guid: &GUID) -> bool {
        self.chosen.map(|chosen| &chosen == guid).unwrap_or(false)
    }

    pub async fn choose(&mut self) -> Option<&GUID> {
        let list = super::LIST.read().await;
        let config = crate::CONFIG.get().unwrap();

        let list_as_guid_string = list.as_guid_strings();

        let prioritized = if config.interfaces.priority.is_empty() {
            priority::choose(self.as_str(), &list_as_guid_string).unwrap()
        } else {
            priority::choose(self.as_str(), &config.interfaces.priority).unwrap()
        };

        let interface = list.get_by_str_guid(prioritized);

        if let Some(iface) = interface {
            self.set(iface.guid);
            self.get()
        } else {
            None
        }
    }

    pub async fn unchoose(&mut self) -> Result<(), ()> {
        if self.chosen.is_none() {
            return Err(())
        }

        self.chosen = None;

        Ok(())
    }
}
impl Default for Operator {
    fn default() -> Self {
        Self { chosen: None }
    }
}
