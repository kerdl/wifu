use num_derive::{FromPrimitive, ToPrimitive};


#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum Error {
    AccessDenied = 5,
    NotEnoughMemory = 8,
    InvalidParameter = 87,
    NotFound = 1168,
    RemoteSessionLimitExceeded = 1220,
}

pub type Result<T> = std::result::Result<T, Error>;
