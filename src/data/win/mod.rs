pub mod wlan;
pub mod util;
pub mod guid;
pub mod error;

pub use wlan::Wlan;
pub use error::{NativeResult, NativeError};

use windows::core::PCWSTR;


pub const SUCCESS: u32 = 0;


pub struct SafePCWSTR(pub PCWSTR);
unsafe impl Send for SafePCWSTR {}
