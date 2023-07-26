use num_derive::{FromPrimitive, ToPrimitive};


#[derive(Debug, Clone, FromPrimitive, ToPrimitive)]
pub enum NativeError {
    AccessDenied = 5,
    NotEnoughMemory = 8,
    InvalidParameter = 87,
    NotFound = 1168,
    RemoteSessionLimitExceeded = 1220,
}

#[derive(Debug)]
pub enum Error {
    Native(NativeError),
}

pub type NativeResult<T> = std::result::Result<T, NativeError>;
