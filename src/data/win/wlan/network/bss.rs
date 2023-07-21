use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use windows::Win32::NetworkManagement::WiFi;


#[derive(Debug, Clone, FromPrimitive, ToPrimitive)]
pub enum Bss {
    Infrastructure = 1,
    Independent = 2,
    Any = 3
}
impl Bss {
    pub fn from_dot11_bss_type(bss: WiFi::DOT11_BSS_TYPE) -> Self {
        Self::from_i32(bss.0).unwrap()
    }

    pub fn to_dot11_bss_type(&self) -> WiFi::DOT11_BSS_TYPE {
        WiFi::DOT11_BSS_TYPE(self.to_i32().unwrap())
    }
}