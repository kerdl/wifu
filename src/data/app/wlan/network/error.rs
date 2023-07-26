use crate::win;
use crate::app::wlan::interface;


#[derive(Debug, Clone)]
pub enum UpdateError {
    Win(win::NativeError),
    Interface(interface::Error)
}