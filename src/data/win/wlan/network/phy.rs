use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use windows::Win32::NetworkManagement::WiFi;


#[derive(Debug, FromPrimitive)]
pub enum Type {
    Any = 0,
    Fhss = 1,
    Dsss = 2,
    IrBaseBand = 3,
    Ofdm = 4,
    Hrdsss = 5,
    Erp = 6,
    Ht = 7,
    Vht = 8,
    Dmg = 9,
    He = 10,
    Eht = 11,
    IhvStart = -2147483648,
    IhvEnd = -1
}
impl Type {
    pub fn from_dot11_phy_type(phy: WiFi::DOT11_PHY_TYPE) -> Self {
        Self::from_i32(phy.0).unwrap()
    }
}
