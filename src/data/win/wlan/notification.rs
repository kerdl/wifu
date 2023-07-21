use num_derive::{FromPrimitive, ToPrimitive};


#[derive(FromPrimitive, ToPrimitive)]
pub enum Source {
    None = 0,
    Onex = 4,
    Acm = 8,
    Msm = 16,
    Security = 32,
    Ihv = 64,
    Hnwk = 128,
    All = 65535
}
