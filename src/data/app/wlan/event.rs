pub mod looping {
    macro_rules! spawner {
        (async fn $name:ident($handle:path, $function:path)) => {
            pub async fn $name() {
                let mut handle = $handle.write().await;
                *handle = Some(tokio::spawn(async move { $function().await }));
            }
        };
    }
    
    macro_rules! closer {
        (async fn $name:ident($handle:path)) => {
            pub async fn $name() {
                let mut handle = $handle.write().await;
    
                if handle.is_none() {
                    println!("{}(): no handle", stringify!($name))
                }
            
                handle.as_ref().map(|h| h.abort());
                *handle = None;
            }
        };
    }
    
    pub(crate) use spawner;
    pub(crate) use closer;
}
