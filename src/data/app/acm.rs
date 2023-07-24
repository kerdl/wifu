macro_rules! wait_fn {
    (async fn $name:ident($recv:expr, $notification:path)) => {
        pub async fn $name() {
            loop {
                let mut receiver = $recv;
        
                match receiver.recv().await.unwrap().code {
                    $notification => return,
                    _ => ()
                }
            }
        }
    };
}


pub(crate) use wait_fn;