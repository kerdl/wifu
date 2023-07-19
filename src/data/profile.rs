use serde_derive::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};
use strum_macros::{EnumString, Display};
use windows::Win32::NetworkManagement::WiFi;


pub const XMLNS_PROFILE_V1: &str = "http://www.microsoft.com/networking/WLAN/profile/v1";
pub const XMLNS_PROFILE_V3: &str = "http://www.microsoft.com/networking/WLAN/profile/v3";


#[derive(Debug, EnumString, Display)]
pub enum Authentication {
    /// ## Network has no password
    #[strum(to_string = "open")]
    Open,
    #[strum(to_string = "shared")]
    Shared,
    WPA,
    WPAPSK,
    WPA2,
    /// ## WPA2-Personal network
    WPA2PSK,
    WPA3,
    WPA3ENT192,
    WPA3ENT,
    /// ## WPA3-Personal network
    WPA3SAE,
    OWE,
}
impl Authentication {
    pub fn from_dot11_auth_algorithm(auth_algo: WiFi::DOT11_AUTH_ALGORITHM) -> Self {
        match auth_algo {
            WiFi::DOT11_AUTH_ALGO_80211_OPEN => Self::Open,  // checked
            //WiFi::DOT11_AUTH_ALGO_80211_SHARED_KEY => Self::Shared,
            //WiFi::DOT11_AUTH_ALGO_WPA => Self::WPA,
            WiFi::DOT11_AUTH_ALGO_WPA_PSK => Self::WPAPSK,   // checked
            WiFi::DOT11_AUTH_ALGO_RSNA_PSK => Self::WPA2PSK, // checked
            //WiFi::DOT11_AUTH_ALGO_WPA3 => Self::WPA3,
            WiFi::DOT11_AUTH_ALGO_WPA3_SAE => Self::WPA3SAE, // checked
            //WiFi::DOT11_AUTH_ALGO_OWE => Self::OWE,
            //WiFi::DOT11_AUTH_ALGO_WPA3_ENT => Self::WPA3ENT,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, EnumString, Display)]
pub enum Encryption {
    #[strum(to_string = "none")]
    None,
    /// ## WEP Encryption
    /// Used in conbination with `Authentication::Open`
    WEP,
    TKIP,
    AES,
    GCMP256
}
impl Encryption {
    pub fn from_dot11_cipher_algorithm(cipher_algo: WiFi::DOT11_CIPHER_ALGORITHM) -> Self {
        match cipher_algo {
            WiFi::DOT11_CIPHER_ALGO_NONE => Self::None, // checked
            WiFi::DOT11_CIPHER_ALGO_WEP40 => Self::WEP,
            WiFi::DOT11_CIPHER_ALGO_TKIP => Self::TKIP,
            WiFi::DOT11_CIPHER_ALGO_CCMP => Self::AES, // checked
            WiFi::DOT11_CIPHER_ALGO_WEP104 => Self::WEP,
            WiFi::DOT11_CIPHER_ALGO_BIP => Self::AES,
            WiFi::DOT11_CIPHER_ALGO_GCMP => Self::AES,
            WiFi::DOT11_CIPHER_ALGO_GCMP_256 => Self::GCMP256,
            WiFi::DOT11_CIPHER_ALGO_CCMP_256 => Self::AES,
            WiFi::DOT11_CIPHER_ALGO_BIP_GMAC_128 => Self::AES,
            WiFi::DOT11_CIPHER_ALGO_BIP_GMAC_256 => Self::AES,
            WiFi::DOT11_CIPHER_ALGO_BIP_CMAC_256 => Self::AES,
            WiFi::DOT11_CIPHER_ALGO_WPA_USE_GROUP => Self::AES,
            WiFi::DOT11_CIPHER_ALGO_RSN_USE_GROUP => Self::AES,
            WiFi::DOT11_CIPHER_ALGO_WEP => Self::WEP, // checked
            WiFi::DOT11_CIPHER_ALGO_IHV_START => Self::AES,
            WiFi::DOT11_CIPHER_ALGO_IHV_END => Self::AES,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, EnumString, Display)]
pub enum KeyType {
    #[strum(to_string = "networkKey")]
    NetworkKey,
    #[strum(to_string = "passPhrase")]
    PassPhrase
}

#[derive(Debug, EnumString, Display)]
pub enum ConnectionType {
    IBSS,
    ESS
}
impl ConnectionType {
    pub fn from_dot11_bss_type(bss_type: WiFi::DOT11_BSS_TYPE) -> Self {
        match bss_type {
            WiFi::dot11_BSS_type_infrastructure => Self::ESS,
            WiFi::dot11_BSS_type_independent => Self::IBSS,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, EnumString, Display)]
pub enum ConnectionMode {
    #[strum(to_string = "auto")]
    Auto,
    #[strum(to_string = "manual")]
    Manual
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct MacRandomization {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    #[serde(rename = "enableRandomization")]
    pub enable_randomization: bool,
}
impl Default for MacRandomization {
    fn default() -> Self {
        Self {
            xmlns: XMLNS_PROFILE_V3.to_string(),
            enable_randomization: false
        }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct SharedKey {
    #[serde(rename = "keyType")]
    #[serde_as(as = "DisplayFromStr")]
    pub key_type: KeyType,
    pub protected: bool,
    #[serde(rename = "keyMaterial")]
    pub key_material: String,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthEncryption {
    #[serde_as(as = "DisplayFromStr")]
    pub authentication: Authentication,
    #[serde_as(as = "DisplayFromStr")]
    pub encryption: Encryption,
    #[serde(rename = "useOneX")]
    pub use_one_x: bool,
}
impl AuthEncryption {
    pub fn wpa2psk_aes() -> Self {
        Self {
            authentication: Authentication::WPA2PSK,
            encryption: Encryption::AES,
            use_one_x: false
        }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Security {
    #[serde(rename = "authEncryption")]
    pub auth_encryption: AuthEncryption,
    #[serde(rename = "sharedKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_key: Option<SharedKey>
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct MSM {
    pub security: Security
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct SSID {
    pub hex: String,
    pub name: String,
}
impl SSID {
    pub fn from_string(string: String) -> Self {
        Self {
            hex: hex::encode_upper(&string),
            name: string,
        }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct SSIDConfig {
    #[serde(rename = "SSID")]
    pub ssid: SSID
}
impl SSIDConfig {
    pub fn from_string(string: String) -> Self {
        Self {
            ssid: SSID::from_string(string)
        }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct WLANProfile {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    pub name: String,
    #[serde(rename = "SSIDConfig")]
    pub ssid_config: SSIDConfig,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "connectionType")]
    pub connection_type: ConnectionType,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "connectionMode")]
    pub connection_mode: ConnectionMode,
    #[serde(rename = "autoSwitch")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_switch: Option<bool>,
    #[serde(rename = "MSM")]
    pub msm: MSM,
    #[serde(rename = "MacRandomization")]
    pub mac_randomization: MacRandomization,
}