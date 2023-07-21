use num_traits::FromPrimitive;
use num_derive::{FromPrimitive, ToPrimitive};
use windows::core::GUID;
use windows::Win32::NetworkManagement::WiFi;


#[derive(Debug, Clone, FromPrimitive, ToPrimitive)]
pub enum State {
    NotReady = 0,
    Connected = 1,
    AdHocNetworkFormed = 2,
    Disconnecting = 3,
    Disconnected = 4,
    Associating = 5,
    Discovering = 6,
    Authenticating = 7,
}
impl State {
    pub fn from_wlan_interface_state(state: WiFi::WLAN_INTERFACE_STATE) -> Self {
        Self::from_i32(state.0).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Interface {
    pub guid: GUID,
    pub description: String,
    pub state: State,
}
impl Interface {
    pub fn from_wlan_interface_info(info: WiFi::WLAN_INTERFACE_INFO) -> Self {
        let interface_desc_slice = info.strInterfaceDescription.as_slice();
        let interface_desc_u16cstr = widestring::U16CStr::from_slice_truncate(interface_desc_slice).unwrap();

        Self { 
            guid: info.InterfaceGuid,
            description: interface_desc_u16cstr.to_string().unwrap(),
            state: State::from_wlan_interface_state(info.isState)
        }
    }
}