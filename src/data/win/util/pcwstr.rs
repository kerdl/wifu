use crate::data::win::SafePCWSTR;
use windows::core::PCWSTR;


pub fn from_u16cstring(u16cs: widestring::U16CString) -> PCWSTR {
    PCWSTR::from_raw(u16cs.as_ptr())
}

pub fn from_u16cstring_safe(u16cs: &widestring::U16CString) -> SafePCWSTR {
    SafePCWSTR(PCWSTR::from_raw(u16cs.as_ptr()))
}