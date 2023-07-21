use num_derive::{FromPrimitive, ToPrimitive};


#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum Error {
    NotAnAcmNotification
}

#[derive(Debug)]
pub enum RwError {
    ReadError(String),
    DeserializeError(String)
}

pub type Result<T> = std::result::Result<T, Error>;
