use num_derive::{FromPrimitive, ToPrimitive};


#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum Error {
    NotAnAcmNotification
}

pub type Result<T> = std::result::Result<T, Error>;
