use std::fmt::Display;

use super::error::RwError;
use serde_derive::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WiFiNetwork {
    pub ssid: String,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WiFiScan {
    pub interval_ms: u64,
}
impl Default for WiFiScan {
    fn default() -> Self {
        Self {
            interval_ms: 30000
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WiFiPriority {
    List,
    SignalStrength
}

#[derive(Debug, Clone)]
pub enum WiFiInvalidReason {
    NoNetworks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WiFi {
    pub networks: Vec<WiFiNetwork>,
    pub priority: WiFiPriority,
    pub scan: WiFiScan,
}
impl WiFi {
    pub fn is_valid(&self) -> Result<(), Vec<WiFiInvalidReason>> {
        let mut reasons = vec![];

        if self.networks.is_empty() {
            reasons.push(WiFiInvalidReason::NoNetworks)
        }

        if reasons.is_empty() {
            Ok(())
        } else {
            Err(reasons)
        }
    }
}
impl Default for WiFi {
    fn default() -> Self {
        Self {
            networks: vec![],
            priority: WiFiPriority::SignalStrength,
            scan: WiFiScan::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interfaces {
    pub priority: Vec<String>,
}
impl Default for Interfaces {
    fn default() -> Self {
        Self {
            priority: vec![]
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DomainsMode {
    FirstIpFromEach,
    AllIpsFromEach,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Domains {
    pub list: Vec<String>,
    pub mode: DomainsMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ping {
    pub domains: Domains,
    pub timeout_ms: u32,
    pub interval_ms: u64,
    pub max_errors: u32,
}
impl Default for Ping {
    fn default() -> Self {
        Self {
            domains: Domains {
                list: vec![
                    "google.com".to_string(),
                    "amazon.com".to_string(),
                    "microsoft.com".to_string()
                ],
                mode: DomainsMode::FirstIpFromEach
            },
            timeout_ms: 1500,
            interval_ms: 1000,
            max_errors: 3
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfigInvalidReason {
    WiFi(Vec<WiFiInvalidReason>)
}

#[derive(Debug, Clone)]
pub struct ConfigInvalidReasons {
    pub reasons: Vec<ConfigInvalidReason>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub ping: Ping,
    pub interfaces: Interfaces,
    pub wifi: WiFi
}
impl Config {
    pub async fn load() -> Result<Self, RwError> {
        let bytes = tokio::fs::read(crate::CFG_PATH.as_path()).await;
        if let Err(err) = bytes {
            return Err(RwError::ReadError(err.to_string()))
        }
        let bytes = bytes.unwrap();

        let this = serde_json::from_slice(&bytes);
        if let Err(err) = this {
            return Err(RwError::DeserializeError(err.to_string()))
        }
        let this = this.unwrap();

        Ok(this)
    }

    pub async fn save(&self) -> tokio::io::Result<()> {
        tokio::fs::write(crate::CFG_PATH.as_path(), serde_json::to_vec_pretty(&self).unwrap()).await
    }

    pub async fn default_and_save() -> tokio::io::Result<Self> {
        let this = Self::default();
        this.save().await?;
        Ok(this)
    }

    pub fn is_valid(&self) -> Result<(), ConfigInvalidReasons> {
        let mut reasons = vec![];

        if let Err(wifi_reasons) = self.wifi.is_valid() {
            reasons.push(ConfigInvalidReason::WiFi(wifi_reasons))
        }

        if reasons.is_empty() {
            Ok(())
        } else {
            Err(ConfigInvalidReasons { reasons })
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            ping: Ping::default(),
            interfaces: Interfaces::default(),
            wifi: WiFi::default()
        }
    }
}