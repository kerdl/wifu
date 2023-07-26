pub mod event;
pub mod error;
pub mod list;
pub mod chosen;
pub use error::UpdateError;

use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;


pub static IS_INITIALIZED: Lazy<Arc<RwLock<bool>>> = Lazy::new(
    || Arc::new(RwLock::new(false))
);
pub static LIST: Lazy<Arc<RwLock<list::Operator>>> = Lazy::new(
    || Arc::new(RwLock::new(list::Operator::default()))
);
pub static CHOSEN: Lazy<Arc<RwLock<chosen::Operator>>> = Lazy::new(
    || Arc::new(RwLock::new(chosen::Operator::default()))
);


pub async fn init() {
    event::init();

    *IS_INITIALIZED.write().await = true;
}