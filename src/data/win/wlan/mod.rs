pub mod acm;
pub mod interface;
pub mod notification;

use crate::data::win;
use crate::data::win::wlan::acm::notification::Code as AcmNotifCode;
use crate::data::win::wlan::acm::notification::Notification as AcmNotif;
use interface::Interface;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use num_traits::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};
use windows::Win32::NetworkManagement::WiFi;
use windows::Win32::Foundation::HANDLE;
use windows::core::GUID;


static mut SESSIONS: Lazy<RwLock<Sessions>> = Lazy::new(|| RwLock::new(Sessions::default()));


#[derive(Debug)]
pub struct Sessions {
    map: HashMap<u32, Arc<RwLock<Session>>>,
}
impl Sessions {
    pub fn add(&mut self, session: Session) {
        self.map.insert(session.id, Arc::new(RwLock::new(session)));
    }

    pub fn get(&self, handle: u32) -> Option<Arc<RwLock<Session>>> {
        self.map.get(&handle).map(|s| s.clone())
    }

    pub fn end(&mut self, handle: u32) {
        self.map.remove(&handle);
    }
}
impl Default for Sessions {
    fn default() -> Self {
        Self { map: HashMap::new() }
    }
}


#[derive(Debug)]
pub struct Session {
    id: u32,
    acm_notify_sender: broadcast::Sender<AcmNotif>,
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
        let id = rand::random::<u32>();

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

        let session = Session {
            id,
            acm_notify_sender,
            acm_notify_receiver,
        };

        let mut this = Self {
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
        session_ptr: *mut core::ffi::c_void
    ) {
        let session = &mut *(session_ptr as *mut Session);
        println!("session={:?}", session);
        let notification = AcmNotif::from_l2_notification_data(*notify).unwrap();

        session.acm_notify_sender.send(notification).unwrap();
    }

    async fn register_acm_notifs(&mut self) -> win::Result<()> {
        let result = unsafe {
            WiFi::WlanRegisterNotification(
                self.handle,
                WiFi::WLAN_NOTIFICATION_SOURCE_ACM,
                None,
                Some(Self::acm_notif_callback),
                Some(&mut self.session as *mut _ as *mut core::ffi::c_void),
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

    pub async fn scan(&mut self, guid: GUID) -> win::Result<()> {
        let result = unsafe {
            WiFi::WlanScan(self.handle, &guid, None, None, None)
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
                _ => ()
            }
        }

        Ok(())
    }

    pub fn available_networks() {

    }
}
impl Drop for Wlan {
    fn drop(&mut self) {
        unsafe {
            WiFi::WlanCloseHandle(self.handle, None);
        }
    }
}