use windows::core::{PCWSTR, PWSTR};


pub fn from_pcwstr(pcwstr: &PCWSTR) -> Result<String, std::string::FromUtf16Error> {
    unsafe { String::from_utf16(pcwstr.as_wide()) }
}

pub fn from_pwstr(pwstr: &PWSTR) -> Result<String, std::string::FromUtf16Error> {
    unsafe { String::from_utf16(pwstr.as_wide()) }
}