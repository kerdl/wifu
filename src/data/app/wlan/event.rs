pub enum Initiator {
    NetworkChoose,
}

pub mod looping {
    macro_rules! works {
        (async fn $name:ident($handle:path)) => {
            pub async fn $name() -> bool {
                let handle = $handle.read().await;
                handle.as_ref().map(|h| !h.is_finished()).unwrap_or(false)
            }
        };
    }

    macro_rules! spawner {
        (async fn $name:ident($handle:path, $function:path, $works_fn:path)) => {
            pub async fn $name() {
                if $works_fn().await {
                    panic!("{}(): can't spawn more than 1", stringify!($name))
                }
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
    
    pub(crate) use works;
    pub(crate) use spawner;
    pub(crate) use closer;
}
