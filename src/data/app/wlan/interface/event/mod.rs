pub mod acm;
pub mod autopilot;

use crate::app::wlan::acm::NotificationWithInterface;

use tokio::sync::broadcast::{channel, Receiver};
use once_cell::sync::OnceCell;


pub static RECEIVER: OnceCell<Receiver<NotificationWithInterface>> = OnceCell::new();


pub fn init() {
    let (sender, receiver) = channel(64);
    acm::set_sender(sender);
    RECEIVER.set(receiver).unwrap()
}