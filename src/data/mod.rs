pub mod cfg;
pub mod error;


pub async fn init_fs() {
    if !crate::DATA_PATH.exists() {
        tokio::fs::create_dir(crate::DATA_PATH.as_path()).await.unwrap();
    }

    if !crate::CFG_PATH.exists() {
        crate::CONFIG.set(cfg::Config::default_and_save().await.unwrap()).unwrap();
    } else {
        crate::CONFIG.set(cfg::Config::load().await.unwrap()).unwrap()
    }
}