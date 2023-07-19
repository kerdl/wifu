use num_derive::{FromPrimitive, ToPrimitive};


#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum Error {
    InvalidParameter = 87,
    NotEnoughMemory = 8,
    RemoteSessionLimitExceeded = 1220
}

pub type Result<T> = std::result::Result<T, Error>;
