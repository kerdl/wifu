pub mod acm;
pub mod interface;
pub mod network;
pub mod notification;
pub use interface::Interface;
pub use network::Network;
use widestring::U16CStr;

use crate::data::win;
use crate::data::win::SafePCWSTR;
use crate::data::win::wlan::acm::notification::Code as AcmNotifCode;
use crate::data::win::wlan::acm::notification::Notification as AcmNotif;

use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::broadcast;
use once_cell::sync::Lazy;
use num_traits::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};
use rand::Rng;
use windows::Win32::NetworkManagement::WiFi;
use windows::Win32::Foundation::HANDLE;
use windows::core::GUID;


static mut ACM_NOTIFY_SENDERS: Lazy<HashMap<u32, broadcast::Sender<AcmNotif>>> = Lazy::new(
    || HashMap::new()
);


#[derive(Debug)]
pub struct Session {
    id: u32,
    acm_notify_receiver: broadcast::Receiver<AcmNotif>,
}


#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum ClientVersion {
    /// ## Client version for Windows XP with SP3 and Wireless LAN API for Windows XP with SP2
    First = 1,
    Second = 2
}


#[derive(Debug)]
pub struct Wlan {
    session: Session,
    handle: HANDLE,
    negotiated_version: ClientVersion,
}
// --------- Constructors ---------
impl Wlan {
    pub fn new(client_version: ClientVersion) -> win::NativeResult<Self> {
        let mut handle = unsafe { std::mem::zeroed() };
        let mut negotiated_version = unsafe { std::mem::zeroed() };
        let id = rand::thread_rng().gen::<u32>();

        let handle_result = unsafe {
            WiFi::WlanOpenHandle(
                client_version.to_u32().unwrap(),
                None,
                &mut negotiated_version,
                &mut handle
            )
        };
        
        if handle_result != win::SUCCESS {
            return Err(win::NativeError::from_u32(handle_result).unwrap())
        }

        let (acm_notify_sender, acm_notify_receiver) = {
            broadcast::channel::<AcmNotif>(64)
        };

        unsafe { ACM_NOTIFY_SENDERS.insert(id, acm_notify_sender) };

        let session = Session {
            id,
            acm_notify_receiver,
        };

        let this = Self {
            session,
            handle,
            negotiated_version: ClientVersion::from_u32(negotiated_version).unwrap(),
        };

        this.register_acm_notifs()?;

        Ok(this)
    }
}
// --------- Getters ---------
impl Wlan {
    pub fn negotiated_version(&self) -> &ClientVersion {
        &self.negotiated_version
    }
}
// --------- Callbacks ---------
impl Wlan {
    unsafe extern "system" fn acm_notif_callback(
        notify: *mut WiFi::L2_NOTIFICATION_DATA,
        sender_ptr: *mut core::ffi::c_void
    ) {
        let sender = &*(sender_ptr as *const broadcast::Sender<AcmNotif>);
        let notification = AcmNotif::from_l2_notification_data(*notify).unwrap();

        //println!("acm notification! {:?}", notification);

        sender.send(notification).unwrap();
    }

    fn register_acm_notifs(&self) -> win::NativeResult<()> {
        let result = unsafe {
            WiFi::WlanRegisterNotification(
                self.handle,
                WiFi::WLAN_NOTIFICATION_SOURCE_ACM,
                None,
                Some(Self::acm_notif_callback),
                Some(ACM_NOTIFY_SENDERS.get(&self.session.id).unwrap() as *const broadcast::Sender<AcmNotif> as *const core::ffi::c_void),
                None,
                None
            )
        };

        if result != win::SUCCESS {
            return Err(win::NativeError::from_u32(result).unwrap())
        }

        Ok(())
    }
}
// --------- User actions ---------
impl Wlan {
    /// ## Get all available wireless interfaces
    /// 
    /// This includes interfaces such as USB dongles,
    /// PCIe wireless adapters, virtual interfaces, etc.
    /// Anything that works with WI-FI, really.
    /// 
    /// Each interface has its own unique `GUID`, which
    /// acts as an identifier for all other functions,
    /// such as `scan`, `get_profile` and `connect`.
    ///
    /// ## Returns
    /// `Result` wraps 2 values:
    /// - An error, returned by a `WlanEnumInterfaces` function.
    /// - A `Vec` of all available interfaces.
    pub fn list_interfaces(&self) -> win::NativeResult<Vec<Interface>> {
        let mut output: Vec<Interface> = vec![];
        let mut list: *mut WiFi::WLAN_INTERFACE_INFO_LIST = unsafe { std::mem::zeroed() };
        let list_result = unsafe { WiFi::WlanEnumInterfaces(self.handle, None, &mut list) };

        if list_result != win::SUCCESS {
            return Err(win::NativeError::from_u32(list_result).unwrap())
        }

        unsafe {
            for idx in 0..(*list).dwNumberOfItems {
                let interface = (*list).InterfaceInfo.as_ptr().add(idx as usize);
                output.push(Interface::from_wlan_interface_info(*interface))
            }
        }

        unsafe {
            WiFi::WlanFreeMemory(list as *const core::ffi::c_void)
        };

        Ok(output)
    }

    /// ## Scan for WI-FI networks
    /// 
    /// Windows periodically performs scans automatically,
    /// but this functions allows to perform it manually,
    /// "right here, right now".
    /// 
    /// Usually, a full scan takes 1-2 seconds.
    /// 
    /// ## Parameters
    /// - `guid`: A GUID of the interface to perform
    /// a scan on.
    /// 
    /// ## Returns
    /// `Result` wraps 2 values:
    /// - An error, returned by a `WlanScan` function.
    /// Means that scan had failed from the very beginning.
    /// - A `bool` value. `true` means that scan was successful,
    /// and `false` means that it failed in the process.
    /// One of the reasons could be unplugging USB WI-FI
    /// adapter during the scan.
    pub async fn scan(&self, guid: &GUID) -> win::NativeResult<bool> {
        let result = unsafe {
            WiFi::WlanScan(self.handle, guid, None, None, None)
        };

        if result != win::SUCCESS {
            return Err(win::NativeError::from_u32(result).unwrap())
        }

        loop {
            let mut acm_notify_receiver = {
                self.session.acm_notify_receiver.resubscribe()
            };

            let timeout = tokio::time::timeout(
                Duration::from_secs(4),
                async move { acm_notify_receiver.recv().await }
            ).await;

            if timeout.is_err() {
                break
            }
            let notif = timeout.unwrap().unwrap();

            match notif.code {
                AcmNotifCode::ScanComplete => return Ok(true),
                AcmNotifCode::ScanFail => return Ok(false),
                _ => ()
            }
        }

        Ok(false)
    }

    /// ## Currently available WI-FI networks
    /// Windows stores all currently available networks
    /// and allows to list them using `WlanGetAvailableNetworkList`.
    /// 
    /// Note that to get the freshest list you should first
    /// perform a scan using the `Wlan::scan` function, which
    /// takes about 1-2 seconds to complete.
    /// 
    /// ## Parameters
    /// - `guid`: A GUID of the interface from which
    /// the available networks will return.
    /// 
    /// ## Returns
    /// `Result` wraps 2 values:
    /// - An error, returned by a `WlanGetAvailableNetworkList` function.
    /// - A `Vec` of all currently available networks.
    pub fn available_networks(&self, guid: &GUID) -> win::NativeResult<Vec<Network>>{
        let mut raw_networks = unsafe { std::mem::zeroed() };
        let result = unsafe {
            WiFi::WlanGetAvailableNetworkList(
                self.handle,
                guid,
                WiFi::WLAN_AVAILABLE_NETWORK_INCLUDE_ALL_MANUAL_HIDDEN_PROFILES,
                None,
                &mut raw_networks
            )
        };

        if result != win::SUCCESS {
            return Err(win::NativeError::from_u32(result).unwrap())
        }

        let networks = Network::from_wlan_available_network_list(raw_networks);

        unsafe {
            WiFi::WlanFreeMemory(raw_networks as *const core::ffi::c_void)
        }

        Ok(networks)
    }

    /// ## Get saved WI-FI network
    /// 
    /// When you connect to some WI-FI, Windows saves
    /// this network's details (such as its SSID, password, encryption type, etc.)
    /// and allows to access them using the `WlanGetProfile` function.
    /// 
    /// However, the password is stored in encrypted form.
    /// There are 2 ways to decrypt it:
    /// - Export the profile in `cmd` with the `key=clear` flag.
    /// Example: `netsh wlan export profile name="WiFiSSID" key=clear folder=c:\Wifi`
    /// - Call `CryptUnprotectData` function with admin rights
    /// and `winlogon.exe`'s token privelege.
    /// More about it: `https://github.com/l4tr0d3ctism/WifikeyDecryptor`
    ///
    /// ## Parameters
    /// - `guid`: A GUID of the interface from which
    /// the profile will return.
    /// - `name`: The name of a profile.
    /// Matches the network SSID.
    /// 
    /// ## Returns
    /// `Result` wraps 2 values:
    /// - An error, returned by a `WlanGetProfile` function.
    /// - A deserialized profile.
    pub fn get_profile(&self, guid: &GUID, name: &str) -> win::NativeResult<network::profile::Profile> {
        let mut profile_pwstr = unsafe { std::mem::zeroed() };

        let name_u16cs = widestring::U16CString::from_str(name).unwrap();
        let name_pcwstr = windows::core::PCWSTR::from_raw(name_u16cs.as_ptr());

        let result = unsafe {
            WiFi::WlanGetProfile(
                self.handle,
                guid,
                name_pcwstr,
                None,
                &mut profile_pwstr,
                None,
                None
            )
        };

        if result != win::SUCCESS {
            return Err(win::NativeError::from_u32(result).unwrap())
        }

        let string_profile = win::util::string::from_pwstr(&profile_pwstr).unwrap();
        let profile = network::profile::Profile::deserialize_str(&string_profile).unwrap();

        unsafe {
            WiFi::WlanFreeMemory(profile_pwstr.as_ptr() as *const core::ffi::c_void)
        };

        Ok(profile)
    }

    pub fn profile_exists(&self, guid: &GUID, name: &str) -> bool {
        if let Err(win::NativeError::NotFound) = self.get_profile(guid, name) {
            return false
        }

        true
    }

    pub fn list_profiles(&self, guid: &GUID) -> win::NativeResult<Vec<network::Profile>> {
        let mut list = unsafe { std::mem::zeroed() };
        let mut parsed_list = vec![];

        let result = unsafe {
            WiFi::WlanGetProfileList(
                self.handle,
                guid,
                None,
                &mut list
            )
        };

        if result != win::SUCCESS {
            return Err(win::NativeError::from_u32(result).unwrap())
        }

        unsafe {
            for idx in 0..(*list).dwNumberOfItems {
                let profile_info = (*list).ProfileInfo.as_ptr().add(idx as usize);
                let u16cs = U16CStr::from_slice_truncate(
                    (*profile_info).strProfileName.as_slice()
                ).unwrap();
                let profile_name = u16cs.to_string().unwrap();

                let profile_result = self.get_profile(guid, &profile_name);
                if profile_result.is_err() {
                    continue;
                }
                let profile = profile_result.unwrap();

                parsed_list.push(profile)
            }
        }

        Ok(parsed_list)
    }

    pub fn set_profile(&self, guid: &GUID, profile: network::Profile) -> win::NativeResult<()> {
        let mut reason_code = 0;
        let profile_string = profile.genuine_serialize_to_string();
        let profile_u16cs = widestring::U16CString::from_str(&profile_string).unwrap();
        let profile_pcwstr = windows::core::PCWSTR::from_raw(profile_u16cs.as_ptr());

        let result = unsafe {
            WiFi::WlanSetProfile(
                self.handle,
                guid,
                0,
                profile_pcwstr,
                None,
                false,
                None,
                &mut reason_code
            )
        };

        if result != win::SUCCESS {
            return Err(win::NativeError::from_u32(result).unwrap())
        }

        if reason_code != 0 {
            panic!("how to handle reason codes idk")
        }

        Ok(())
    }

    fn wlan_connection_params_safe(
        profile_pcwstr: SafePCWSTR,
        bss: &network::Bss
    ) -> network::SafeConnectionParameters {
        let params = WiFi::WLAN_CONNECTION_PARAMETERS {
            wlanConnectionMode: WiFi::wlan_connection_mode_profile,
            strProfile: profile_pcwstr.0,
            dot11BssType: bss.to_dot11_bss_type(),
            ..Default::default()
        };
        
        network::SafeConnectionParameters(params)
    }

    fn connect_safe(
        &self,
        guid: &GUID,
        params: network::SafeConnectionParameters
    ) -> u32 {
        unsafe {
            WiFi::WlanConnect(
                self.handle,
                guid,
                &params.0,
                None
            )
        }
    }

    pub async fn connect(
        &self,
        guid: &GUID,
        profile: &str,
        bss: &network::Bss,
    ) -> win::NativeResult<bool> {
        let profile_u16cs = widestring::U16CString::from_str(profile).unwrap();
        let profile_pcwstr = super::util::from_u16cstring_safe(&profile_u16cs);

        let params = Self::wlan_connection_params_safe(profile_pcwstr, bss);
        
        let result = self.connect_safe(guid, params);

        if result != win::SUCCESS {
            return Err(win::NativeError::from_u32(result).unwrap())
        }

        loop {
            let timeout = tokio::time::timeout(
                Duration::from_secs(5),
                async move { self.session.acm_notify_receiver.resubscribe().recv().await }
            ).await;
            if timeout.is_err() { break }
            let notif = timeout.unwrap().unwrap();

            match notif.code {
                AcmNotifCode::ConnectionStart => (),
                AcmNotifCode::ConnectionComplete => return Ok(true),
                AcmNotifCode::ConnectionAttemptFail => return Ok(false),
                _ => println!("Wlan::connect() recv {:?}", notif.code)
            }
        }

        Ok(false)
    }

    pub async fn disconnect(&self, guid: &GUID) -> win::NativeResult<bool> {
        let result = unsafe {
            WiFi::WlanDisconnect(self.handle, guid, None)
        };

        if result != win::SUCCESS {
            return Err(win::NativeError::from_u32(result).unwrap())
        }

        loop {
            let timeout = tokio::time::timeout(
                Duration::from_secs(5),
                async move { self.session.acm_notify_receiver.resubscribe().recv().await }
            ).await;
            if timeout.is_err() { break }
            let notif = timeout.unwrap().unwrap();

            match notif.code {
                AcmNotifCode::Disconnecting => (),
                AcmNotifCode::Disconnected => return Ok(true),
                AcmNotifCode::ConnectionAttemptFail => return Ok(false),
                _ => println!("Wlan::disconnect() code recv: {:?}", notif.code)
            }
        }

        Ok(false)
    }

    pub async fn acm_recv(&self) -> AcmNotif {
        let mut acm_notify_receiver = {
            self.session.acm_notify_receiver.resubscribe()
        };

        let notif = acm_notify_receiver.recv().await.unwrap();
        //println!("{:?}", notif);
        notif
    }
}
impl Drop for Wlan {
    fn drop(&mut self) {
        unsafe {
            WiFi::WlanCloseHandle(self.handle, None);
            ACM_NOTIFY_SENDERS.remove(&self.session.id).unwrap();
        }
    }
}
