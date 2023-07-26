pub mod acm;
pub mod autopilot;

use crate::app::wlan::acm::NotificationWithInterface;

use tokio::sync::broadcast::{channel, Receiver};
use once_cell::sync::OnceCell;
use windows::core::GUID;


pub static RECEIVER: OnceCell<Receiver<NotificationWithInterface>> = OnceCell::new();


pub fn init() {
    let (sender, receiver) = channel(64);
    acm::set_sender(sender);
    RECEIVER.set(receiver).unwrap()
}

pub async fn is_relevant(guid: &GUID) -> bool {
    if !super::CHOSEN.read().await.is_guid_chosen(guid) {
        false
    } else {
        true
    }
}