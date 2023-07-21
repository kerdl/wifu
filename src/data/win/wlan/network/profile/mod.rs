pub mod raw;

use crate::data::win::wlan::Network;


#[derive(Debug, Clone)]
pub struct Mac {
    pub randomization: bool,
}
impl Default for Mac {
    fn default() -> Self {
        Self { randomization: false }
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub kind: raw::ConnectionType,
    pub mode: raw::ConnectionMode,
}

#[derive(Debug, Clone)]
pub struct Key {
    pub kind: raw::KeyType,
    pub is_encrypted: bool,
    pub content: String,
}
impl Key {
    pub fn from_plain(password: impl ToString) -> Self {
        Self {
            kind: raw::KeyType::PassPhrase,
            is_encrypted: false,
            content: password.to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Security {
    pub auth: raw::Authentication,
    pub cipher: raw::Encryption,
    pub key: Option<Key>
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub ssid: String,
    pub connection: Connection,
    pub auto_switch: Option<bool>,
    pub security: Security,
    pub mac: Mac
}
impl Profile {
    pub fn deserialize_str(string: &str) -> Result<Self, quick_xml::DeError> {
        raw::WLANProfile::deserialize_str(string).map(|raw| raw.to_friendly())
    }

    pub fn from_network(network: Network, key: Option<Key>) -> Self {
        Self {
            name: network.ssid.clone(),
            ssid: network.ssid,
            connection: Connection {
                kind: raw::ConnectionType::from_bss(network.bss),
                mode: raw::ConnectionMode::default()
            },
            auto_switch: Some(false),
            security: Security {
                auth: raw::Authentication::from_network_auth(network.security.auth),
                cipher: raw::Encryption::from_network_cipher(network.security.cipher),
                key
            },
            mac: Mac::default()
        }
    }
}
// --------- Serialization ---------
impl Profile {
    /// ## Serialize to `Writer` exactly how Windows does it
    /// 
    /// Windows uses `\t` (tabulation) instead of ` ` (whitespaces)
    /// to indent in WI-FI profiles, while default `quick_xml` config
    /// does not indent at all.
    ///
    /// This function provides such Windows-like config.
    pub fn genuine_serialize<W: std::fmt::Write>(self, writer: &mut W) {
        self.to_raw().genuine_serialize(writer)
    }

    /// ## Serialize to `String` exactly how Windows does it
    /// 
    /// Windows uses `\t` (tabulation) instead of ` ` (whitespaces)
    /// to indent in WI-FI profiles, while default `quick_xml` config
    /// does not indent at all.
    ///
    /// This function provides such Windows-like config.
    pub fn genuine_serialize_to_string(self) -> String {
        self.to_raw().genuine_serialize_to_string()
    }
}
// --------- Constructors ---------
impl Profile {
    pub fn from_raw(raw: raw::WLANProfile) -> Self {
        Self {
            name: raw.name,
            ssid: raw.ssid_config.ssid.name,
            connection: Connection {
                kind: raw.connection_type,
                mode: raw.connection_mode
            },
            auto_switch: raw.auto_switch,
            security: Security {
                auth: raw.msm.security.auth_encryption.authentication,
                cipher: raw.msm.security.auth_encryption.encryption,
                key: raw.msm.security.shared_key.map(
                    |key| Key {
                        kind: key.key_type,
                        is_encrypted: key.protected,
                        content: key.key_material
                    }
                )
            },
            mac: Mac {
                randomization: raw.mac_randomization.enable_randomization
            }
        }
    }
}
// --------- Converters ---------
impl Profile {
    pub fn to_raw(self) -> raw::WLANProfile {
        raw::WLANProfile::from_friendly(self)
    }
}
