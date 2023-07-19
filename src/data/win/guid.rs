use windows::core::GUID;


pub fn to_string(guid: &GUID) -> String {
    format!("{:?}", guid)
}