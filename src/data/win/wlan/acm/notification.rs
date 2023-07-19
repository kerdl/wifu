use crate::data::app::error as app_err;
use crate::data::win::wlan::notification as wlan_notif;

use num_traits::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};
use windows::core::GUID;
use windows::Win32::NetworkManagement::WiFi;


#[derive(Debug, Clone, FromPrimitive, ToPrimitive)]
pub enum Code {
    Start = 0,
    AutoconfEnabled = 1,
    AutoconfDisabled = 2,
    BackgroundScanEnabled = 3,
    BackgroundScanDisabled = 4,
    BssTypeChange = 5,
    PowerSettingChange = 6,
    ScanComplete = 7,
    ScanFail = 8,
    ConnectionStart = 9,
    ConnectionComplete = 10,
    ConnectionAttemptFail = 11,
    FilterListChange = 12,
    InterfaceArrival = 13,
    InterfaceRemoval = 14,
    ProfileChange = 15,
    ProfileNameChange = 16,
    ProfilesExhausted = 17,
    NetworkNotAvailable = 18,
    NetworkAvailable = 19,
    Disconnecting = 20,
    Disconnected = 21,
    AdhocNetworkStateChange = 22,
    ProfileUnblocked = 23,
    ScreenPowerChange = 24,
    ProfileBlocked = 25,
    ScanListRefresh = 26,
    OperationalStateChange = 27,
    End = 28
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub code: Code,
    pub guid: GUID
}
impl Notification {
    pub fn from_l2_notification_data(data: WiFi::L2_NOTIFICATION_DATA) -> app_err::Result<Self> {
        if data.NotificationSource != wlan_notif::Source::Acm.to_u32().unwrap() {
            return Err(app_err::Error::NotAnAcmNotification)
        }

        let this = Self {
            code: Code::from_u32(data.NotificationCode).unwrap(),
            guid: data.InterfaceGuid
        };

        Ok(this)
    }
}