pub mod acm;
pub mod interface;
pub mod network;
pub mod notification;

use crate::data::win;
use crate::data::win::wlan::acm::notification::Code as AcmNotifCode;
use crate::data::win::wlan::acm::notification::Notification as AcmNotif;
use interface::Interface;
use network::Network;
use rand::Rng;

use std::collections::HashMap;
use tokio::sync::broadcast;
use once_cell::sync::Lazy;
use num_traits::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};
use windows::Win32::NetworkManagement::WiFi;
use windows::Win32::Foundation::HANDLE;
use windows::core::GUID;


static mut ACM_NOTIFY_SENDERS: Lazy<HashMap<u32, broadcast::Sender<AcmNotif>>> = Lazy::new(|| HashMap::new());


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
    pub async fn new(client_version: ClientVersion) -> win::Result<Self> {
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
            return Err(win::Error::from_u32(handle_result).unwrap())
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

        this.register_acm_notifs().await?;

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

        sender.send(notification).unwrap();
    }

    async fn register_acm_notifs(&self) -> win::Result<()> {
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
            return Err(win::Error::from_u32(result).unwrap())
        }

        Ok(())
    }
}
// --------- User actions ---------
impl Wlan {
    pub fn list_interfaces(&self) -> win::Result<Vec<Interface>> {
        let mut output: Vec<Interface> = vec![];
        let mut list: *mut WiFi::WLAN_INTERFACE_INFO_LIST = unsafe { std::mem::zeroed() };
        let list_result = unsafe { WiFi::WlanEnumInterfaces(self.handle, None, &mut list) };

        if list_result != win::SUCCESS {
            return Err(win::Error::from_u32(list_result).unwrap())
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

    pub async fn scan(&mut self, guid: &GUID) -> win::Result<()> {
        let result = unsafe {
            WiFi::WlanScan(self.handle, guid, None, None, None)
        };

        if result != win::SUCCESS {
            return Err(win::Error::from_u32(result).unwrap())
        }

        let mut acm_notify_receiver = {
            self.session.acm_notify_receiver.resubscribe()
        };

        while let Ok(notif) = acm_notify_receiver.recv().await {
            match notif.code {
                AcmNotifCode::ScanComplete => return Ok(()),
                AcmNotifCode::ScanFail => {println!("SCAN FAILED!"); return Ok(())},
                _ => println!("scan() code recv: {:?}", notif.code)
            }
        }

        Ok(())
    }

    pub fn available_networks(&self, guid: &GUID) -> win::Result<Vec<Network>>{
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
            return Err(win::Error::from_u32(result).unwrap())
        }

        let networks = Network::from_wlan_available_network_list(raw_networks);

        unsafe {
            WiFi::WlanFreeMemory(raw_networks as *const core::ffi::c_void)
        }

        Ok(networks)
    }

    pub fn get_profile(&self, guid: &GUID, name: &str) -> win::Result<network::profile::Profile> {
        let mut pwstr_profile = unsafe { std::mem::zeroed() };
        let wides = widestring::U16CString::from_str(name).unwrap();
        let name_pcwstr = windows::core::PCWSTR::from_raw(wides.as_ptr());

        let result = unsafe {
            WiFi::WlanGetProfile(
                self.handle,
                guid,
                name_pcwstr,
                None,
                &mut pwstr_profile,
                None,
                None
            )
        };

        if result != win::SUCCESS {
            return Err(win::Error::from_u32(result).unwrap())
        }

        let string_profile = win::util::string::from_pwstr(&pwstr_profile).unwrap();
        let profile = network::profile::Profile::deserialize_str(&string_profile).unwrap();

        unsafe {
            WiFi::WlanFreeMemory(pwstr_profile.as_ptr() as *const core::ffi::c_void)
        };

        Ok(profile)
    }

    pub fn set_profile(&self, profile: network::profile::Profile) -> win::Result<()> {
        let profile_string = profile.genuine_serialize_to_string();

        unimplemented!()
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