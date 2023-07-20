pub mod auth;
pub mod cipher;
pub use auth::Authentication;
pub use cipher::Encryption;


#[derive(Debug)]
pub struct Security {
    pub enabled: bool,
    pub auth: Authentication,
    pub cipher: Encryption
}