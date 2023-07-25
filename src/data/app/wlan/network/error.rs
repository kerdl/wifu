use crate::win;
use crate::app::wlan::interface;


pub enum UpdateError {
    Win(win::NativeError),
    Interface(interface::Error)
}