use serde_derive::{Serialize, Deserialize};
use windows::Win32::NetworkManagement::WiFi;


#[derive(Serialize, Deserialize)]
pub enum Authentication {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "shared")]
    Shared,
    WPA,
    WPAPSK,
    WPA2,
    WPA2PSK,
    WPA3,
    WPA3ENT192,
    WPA3ENT,
    WPA3SAE,
    OWE,
}
impl Authentication {
    fn from_dot11_auth_algorithm(auth_algo: WiFi::DOT11_AUTH_ALGORITHM) -> Self {
        match auth_algo {
            WiFi::DOT11_AUTH_ALGO_80211_OPEN => Self::Open,
            WiFi::DOT11_AUTH_ALGO_80211_SHARED_KEY => Self::Shared,
            WiFi::DOT11_AUTH_ALGO_WPA => Self::WPA,
            WiFi::DOT11_AUTH_ALGO_WPA_PSK => Self::WPAPSK,
            //WiFi::DOT11_AUTH_ALGO_WPA_NONE => Self::Open,
            //WiFi::DOT11_AUTH_ALGO_RSNA => Self::Open,
            //WiFi::DOT11_AUTH_ALGO_RSNA_PSK => Self::Open,
            WiFi::DOT11_AUTH_ALGO_WPA3 => Self::WPA3,
            WiFi::DOT11_AUTH_ALGO_WPA3_ENT_192 => Self::WPA3ENT192,
            WiFi::DOT11_AUTH_ALGO_WPA3_SAE => Self::WPA3SAE,
            WiFi::DOT11_AUTH_ALGO_OWE => Self::OWE,
            WiFi::DOT11_AUTH_ALGO_WPA3_ENT => Self::WPA3ENT,
            //WiFi::DOT11_AUTH_ALGO_IHV_START => Self::WPA3SAE,
            //WiFi::DOT11_AUTH_ALGO_IHV_END => Self::OWE
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Encryption {
    AES
}

#[derive(Serialize, Deserialize)]
pub enum KeyType {
    #[serde(rename = "networkKey")]
    NetworkKey,
    #[serde(rename = "passPhrase")]
    PassPhrase
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub enum ConnectionMode {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "manual")]
    Manual
}


#[derive(Serialize, Deserialize)]
pub struct MacRandomization {
    #[serde(rename = "enableRandomization")]
    enable_randomization: bool,
}
impl Default for MacRandomization {
    fn default() -> Self {
        Self { enable_randomization: false }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SharedKey {
    #[serde(rename = "keyType")]
    pub key_type: KeyType,
    pub protected: bool,
    #[serde(rename = "keyMaterial")]
    pub key_material: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthEncryption {
    pub authentication: Authentication,
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

#[derive(Serialize, Deserialize)]
pub struct Security {
    #[serde(rename = "authEncryption")]
    pub auth_encryption: AuthEncryption,
    #[serde(rename = "sharedKey")]
    pub shared_key: SharedKey
}

#[derive(Serialize, Deserialize)]
pub struct MSM {
    pub security: Security
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct WLANProfile {
    pub name: String,
    #[serde(rename = "SSIDConfig")]
    pub ssid_config: SSIDConfig,
    #[serde(rename = "connectionType")]
    pub connection_type: ConnectionType,
    #[serde(rename = "connectionMode")]
    pub connection_mode: ConnectionMode,
    #[serde(rename = "autoSwitch")]
    pub auto_switch: bool,
    #[serde(rename = "MSM")]
    pub msm: MSM,
    #[serde(rename = "MacRandomization")]
    pub mac_randomization: MacRandomization,
}