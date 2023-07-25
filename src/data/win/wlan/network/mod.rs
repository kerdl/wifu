pub mod profile;
pub mod security;
pub mod phy;
pub mod bss;
pub use security::{auth::{self, Authentication}, cipher::{self, Encryption}};
pub use bss::Bss;
pub use profile::Profile;

use security::Security;
use windows::Win32::NetworkManagement::WiFi;


pub struct SafeConnectionParameters(pub WiFi::WLAN_CONNECTION_PARAMETERS);
unsafe impl Send for SafeConnectionParameters {}

#[derive(Debug, Clone)]
pub enum UnconnectableReason {
    Unknown
}

#[derive(Debug, Clone)]
pub struct Network {
    pub profile: Option<String>,
    pub ssid: String,
    pub bss: Bss,
    pub bssids: u32,
    pub connectable: bool,
    pub unconnectable_reason: Option<UnconnectableReason>,
    pub phys: Vec<phy::Type>,
    pub signal_quality: u32,
    pub security: Security,
}
// --------- Constructors ---------
impl Network {
    pub fn from_wlan_available_network(network: WiFi::WLAN_AVAILABLE_NETWORK) -> Self {
        let profile = String::from_utf16(network.strProfileName.as_slice())
            .ok()
            .filter(|string| string.is_empty());
        let ssid = unsafe {
            let slice = std::ptr::slice_from_raw_parts(
                network.dot11Ssid.ucSSID.as_ptr(),
                network.dot11Ssid.uSSIDLength as usize
            ).as_ref().unwrap();
            String::from_utf8_lossy(slice).to_string()
        };
        let bss = Bss::from_dot11_bss_type(network.dot11BssType);
        let bssids = network.uNumberOfBssids;
        let connectable = network.bNetworkConnectable.as_bool();
        let unconnectable_reason = if connectable {
            None
        } else {
            Some(UnconnectableReason::Unknown)
        };
        let phys = unsafe {
            let mut v = vec![];
            for idx in 0..network.uNumberOfPhyTypes {
                let raw_phy_type = *network.dot11PhyTypes.as_ptr().add(idx as usize);
                v.push(phy::Type::from_dot11_phy_type(raw_phy_type))
            }
            v
        };
        let signal_quality = network.wlanSignalQuality;
        let security = Security {
            enabled: network.bSecurityEnabled.as_bool(),
            auth: Authentication::from_dot11_auth_algorithm(
                network.dot11DefaultAuthAlgorithm
            ),
            cipher: Encryption::from_dot11_cipher_algorithm(
                network.dot11DefaultCipherAlgorithm
            ),
        };

        Self {
            profile,
            ssid,
            bss,
            bssids,
            connectable,
            unconnectable_reason,
            phys,
            signal_quality,
            security,
        }
    } 

    pub fn from_wlan_available_network_list(networks: *mut WiFi::WLAN_AVAILABLE_NETWORK_LIST) -> Vec<Self> {
        let mut output = vec![];

        unsafe {
            for idx in 0..(*networks).dwNumberOfItems {
                let raw_network_ptr = (*networks).Network.as_ptr().add(idx as usize);
                let network = Self::from_wlan_available_network(*raw_network_ptr);
                output.push(network);
            }
        }

        output
    }
}
// --------- Converters ---------
impl Network {
    pub fn to_profile(self, key: Option<profile::Key>) -> Profile {
        Profile::from_network(self, key)
    }
}