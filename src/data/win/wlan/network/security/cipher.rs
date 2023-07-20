use num_derive::FromPrimitive;
use strum_macros::{EnumString, Display};
use windows::Win32::NetworkManagement::WiFi;


#[derive(Debug)]
pub enum Encryption {
    None,
    Wep40,
    Tkip,
    Ccmp,
    Wep104,
    Bip,
    Gcmp,
    Gcmp256,
    Ccmp256,
    BipGmac128,
    BipGmac256,
    BipCmac256,
    WpaUseGroup,
    RsnUseGroup,
    Wep,
    IhvStart,
    IhvEnd
}
impl Encryption {
    pub fn from_dot11_cipher_algorithm(cipher: WiFi::DOT11_CIPHER_ALGORITHM) -> Self {
        match cipher {
            WiFi::DOT11_CIPHER_ALGO_NONE => Self::None,
            WiFi::DOT11_CIPHER_ALGO_WEP40 => Self::Wep40,
            WiFi::DOT11_CIPHER_ALGO_TKIP => Self::Tkip,
            WiFi::DOT11_CIPHER_ALGO_CCMP => Self::Ccmp,
            WiFi::DOT11_CIPHER_ALGO_WEP104 => Self::Wep104,
            WiFi::DOT11_CIPHER_ALGO_BIP => Self::Bip,
            WiFi::DOT11_CIPHER_ALGO_GCMP => Self::Gcmp,
            WiFi::DOT11_CIPHER_ALGO_GCMP_256 => Self::Gcmp256,
            WiFi::DOT11_CIPHER_ALGO_CCMP_256 => Self::Ccmp256,
            WiFi::DOT11_CIPHER_ALGO_BIP_GMAC_128 => Self::BipGmac128,
            WiFi::DOT11_CIPHER_ALGO_BIP_GMAC_256 => Self::BipGmac256,
            WiFi::DOT11_CIPHER_ALGO_BIP_CMAC_256 => Self::BipCmac256,
            WiFi::DOT11_CIPHER_ALGO_WPA_USE_GROUP => Self::WpaUseGroup,
            WiFi::DOT11_CIPHER_ALGO_WEP => Self::Wep,
            WiFi::DOT11_CIPHER_ALGO_IHV_START => Self::IhvStart,
            WiFi::DOT11_CIPHER_ALGO_IHV_END => Self::IhvEnd,
            _ => unreachable!()
        }
    }
}