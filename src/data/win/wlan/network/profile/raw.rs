use crate::data::win::wlan::network::Bss;
use crate::data::win::wlan::network;

use serde::Serialize as SerializeTrait;
use serde_derive::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};
use strum_macros::{EnumString, Display};
use num_derive::FromPrimitive;
use windows::Win32::NetworkManagement::WiFi;


pub const XMLNS_PROFILE_V1: &str = "http://www.microsoft.com/networking/WLAN/profile/v1";
pub const XMLNS_PROFILE_V3: &str = "http://www.microsoft.com/networking/WLAN/profile/v3";


#[derive(Debug, Clone, EnumString, Display, FromPrimitive)]
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
    pub fn from_network_auth(auth: network::Authentication) -> Self {
        match auth {
            network::Authentication::Open => Self::Open,
            network::Authentication::WpaPsk => Self::WPAPSK,
            network::Authentication::RsnaPsk => Self::WPA2PSK,
            network::Authentication::Wpa3Sae => Self::WPA3SAE,
            _ => unimplemented!()
        }
    }
}

#[derive(Debug, Clone, EnumString, Display, FromPrimitive)]
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
    pub fn from_network_cipher(cipher: network::Encryption) -> Self {
        match cipher {
            network::Encryption::None => Self::None,
            network::Encryption::Ccmp => Self::AES,
            network::Encryption::Wep => Self::WEP,
            _ => unimplemented!()
        }
    }
}

#[derive(Debug, Clone, EnumString, Display)]
pub enum KeyType {
    #[strum(to_string = "networkKey")]
    NetworkKey,
    #[strum(to_string = "passPhrase")]
    PassPhrase
}

#[derive(Debug, Clone, EnumString, Display)]
pub enum ConnectionType {
    IBSS,
    ESS
}
impl ConnectionType {
    pub fn from_dot11_bss_type(bss: WiFi::DOT11_BSS_TYPE) -> Self {
        match bss {
            WiFi::dot11_BSS_type_infrastructure => Self::ESS,
            WiFi::dot11_BSS_type_independent => Self::IBSS,
            _ => unreachable!()
        }
    }

    pub fn from_bss(bss: Bss) -> Self {
        match bss {
            Bss::Infrastructure => Self::ESS,
            Bss::Independent => Self::IBSS,
            _ => unimplemented!()
        }
    }
}

#[derive(Debug, Clone, EnumString, Display)]
pub enum ConnectionMode {
    #[strum(to_string = "auto")]
    Auto,
    #[strum(to_string = "manual")]
    Manual
}
impl Default for ConnectionMode {
    fn default() -> Self {
        Self::Manual
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedKey {
    #[serde(rename = "keyType")]
    #[serde_as(as = "DisplayFromStr")]
    pub key_type: KeyType,
    pub protected: bool,
    #[serde(rename = "keyMaterial")]
    pub key_material: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Security {
    #[serde(rename = "authEncryption")]
    pub auth_encryption: AuthEncryption,
    #[serde(rename = "sharedKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_key: Option<SharedKey>
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MSM {
    pub security: Security
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
// --------- Serialization ---------
impl WLANProfile {
    /// ## Serialize to `Writer` exactly how Windows does it
    /// 
    /// Windows uses `\t` (tabulation) instead of ` ` (whitespaces)
    /// to indent in WI-FI profiles, while default `quick_xml` config
    /// does not indent at all.
    ///
    /// This function provides such Windows-like config.
    pub fn genuine_serialize<W: std::fmt::Write>(&self, writer: &mut W) {
        let mut ser = quick_xml::se::Serializer::new(writer);
        ser.indent('\t', 1);
        self.serialize(ser).unwrap();
    }

    /// ## Serialize to `String` exactly how Windows does it
    /// 
    /// Windows uses `\t` (tabulation) instead of ` ` (whitespaces)
    /// to indent in WI-FI profiles, while default `quick_xml` config
    /// does not indent at all.
    ///
    /// This function provides such Windows-like config.
    pub fn genuine_serialize_to_string(&self) -> String {
        let mut string = "".to_string();
        self.genuine_serialize(&mut string);
        string
    }
}
// --------- Deserialization ---------
impl WLANProfile {
    pub fn deserialize_str(string: &str) -> Result<Self, quick_xml::DeError> {
        quick_xml::de::from_str(string)
    }
}
impl WLANProfile {
    pub fn to_friendly(self) -> super::Profile {
        super::Profile::from_raw(self)
    }

    pub fn from_friendly(friendly: super::Profile) -> Self {
        Self {
            xmlns: XMLNS_PROFILE_V1.to_string(),
            name: friendly.name,
            ssid_config: SSIDConfig {
                ssid: SSID {
                    hex: hex::encode_upper(&friendly.ssid),
                    name: friendly.ssid
                }
            },
            connection_type: friendly.connection.kind,
            connection_mode: friendly.connection.mode,
            auto_switch: friendly.auto_switch,
            msm: MSM {
                security: Security {
                    auth_encryption: AuthEncryption {
                        authentication: friendly.security.auth,
                        encryption: friendly.security.cipher,
                        use_one_x: false
                    },
                    shared_key: friendly.security.key.map(
                        |key| SharedKey {
                            key_type: key.kind,
                            protected: key.is_encrypted,
                            key_material: key.content
                        }
                    )
                }
            },
            mac_randomization: MacRandomization {
                xmlns: XMLNS_PROFILE_V3.to_string(),
                enable_randomization: friendly.mac.randomization
            },
        }
    }
}