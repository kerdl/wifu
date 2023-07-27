use log::debug;
use windows::Win32::NetworkManagement::WiFi;


#[derive(Debug, Clone)]
pub enum Authentication {
    Open,
    SharedKey,
    Wpa,
    WpaPsk,
    WpaNone,
    Rsna,
    RsnaPsk,
    Wpa3,
    Wpa3Sae,
    Owe,
    Wpa3Ent,
    IhvStart,
    IhvEnd
}
impl Authentication {
    pub fn from_dot11_auth_algorithm(auth_algo: WiFi::DOT11_AUTH_ALGORITHM) -> Self {
        match auth_algo {
            WiFi::DOT11_AUTH_ALGO_80211_OPEN => Self::Open,
            WiFi::DOT11_AUTH_ALGO_80211_SHARED_KEY => Self::SharedKey,
            WiFi::DOT11_AUTH_ALGO_WPA => Self::Wpa,
            WiFi::DOT11_AUTH_ALGO_WPA_PSK => Self::WpaPsk,
            WiFi::DOT11_AUTH_ALGO_WPA_NONE => Self::WpaNone,
            WiFi::DOT11_AUTH_ALGO_RSNA => Self::Rsna,
            WiFi::DOT11_AUTH_ALGO_RSNA_PSK => Self::RsnaPsk,
            WiFi::DOT11_AUTH_ALGO_WPA3 => Self::Wpa3,
            WiFi::DOT11_AUTH_ALGO_WPA3_SAE => Self::Wpa3Sae,
            WiFi::DOT11_AUTH_ALGO_OWE => Self::Owe,
            WiFi::DOT11_AUTH_ALGO_WPA3_ENT => Self::Wpa3Ent,
            WiFi::DOT11_AUTH_ALGO_IHV_START => Self::IhvStart,
            WiFi::DOT11_AUTH_ALGO_IHV_END => Self::IhvEnd,
            _ => {
                debug!("auth_algo unknown!! {:?}", auth_algo);
                unreachable!()
            }
        }
    }
}
