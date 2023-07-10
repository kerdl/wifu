#[derive(Debug)]
pub enum RwError {
    ReadError(String),
    DeserializeError(String)
}