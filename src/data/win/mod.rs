pub mod wlan;
pub mod guid;
pub mod error;

pub use wlan::Wlan;
pub use error::{Result, Error};


pub const SUCCESS: u32 = 0;
