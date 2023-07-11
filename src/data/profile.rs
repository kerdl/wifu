use serde_derive::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub enum Authentication {
    WPA2PSK
}

#[derive(Serialize, Deserialize)]
pub enum Encryption {
    AES
}

#[derive(Serialize, Deserialize)]
pub enum KeyType {
    #[serde(rename = "passPhrase")]
    PassPhrase
}

#[derive(Serialize, Deserialize)]
pub enum ConnectionType {
    ESS
}

#[derive(Serialize, Deserialize)]
pub enum ConnectionMode {
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