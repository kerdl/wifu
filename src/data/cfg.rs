use crate::data::error::RwError;

use serde_derive::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct WiFiNetwork {
    pub ssid: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WiFis {
    pub networks: Vec<WiFiNetwork>,
}
impl Default for WiFis {
    fn default() -> Self {
        Self {
            networks: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Ping {
    pub domains: Vec<String>,
    pub timeout_ms: u32,
    pub interval_ms: u64,
    pub max_errors: u32,
    //pub tcp_streams: Vec<Ipv4Addr>
}
impl Default for Ping {
    fn default() -> Self {
        Self {
            domains: vec![
                "google.com".to_string(),
                "amazon.com".to_string(),
                "microsoft.com".to_string()
            ],
            timeout_ms: 2000,
            interval_ms: 500,
            max_errors: 3
            //tcp_streams: vec![]
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub ping: Ping,
    pub interfaces: Interfaces,
    pub wifis: WiFis
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
}
impl Default for Config {
    fn default() -> Self {
        Self {
            ping: Ping::default(),
            interfaces: Interfaces::default(),
            wifis: WiFis::default()
        }
    }
}