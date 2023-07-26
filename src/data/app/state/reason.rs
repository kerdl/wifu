#[derive(Debug, Clone)]
pub enum DeadReason {
    Uninitialized,
    NoInterface,
    NoNetwork,
}
impl DeadReason {
    pub fn is_uninitialized(&self) -> bool {
        if let Self::Uninitialized = *self { true } else { false }
    }

    pub fn is_no_interface(&self) -> bool {
        if let Self::NoInterface = *self { true } else { false }
    }

    pub fn is_no_network(&self) -> bool {
        if let Self::NoNetwork = *self { true } else { false }
    }
}
