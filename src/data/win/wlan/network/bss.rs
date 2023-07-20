use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use windows::Win32::NetworkManagement::WiFi;


#[derive(Debug, FromPrimitive)]
pub enum Bss {
    Infrastructure = 1,
    Independent = 2,
    Any = 3
}
impl Bss {
    pub fn from_dot11_bss_type(bss: WiFi::DOT11_BSS_TYPE) -> Self {
        Self::from_i32(bss.0).unwrap()
    }
}