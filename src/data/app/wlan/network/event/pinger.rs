pub async fn pinger() {
    loop {
        if pinger::PINGER.read().await.has_no_ips() {
            interface::scan().await.unwrap();
            if !choose_global(false).await {
                return close_pinger_global().await
            }
            pinger::PINGER.write().await.update_ips()
        }

        pinger::PINGER.read().await.start().await;

        interface::scan().await.unwrap();
        if !choose_global(false).await {
            return close_pinger_global().await
        }
    }
}

pub fn spawn_pinger() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move { pinger().await })
}

pub async fn spawn_pinger_global() {
    *PINGER_HANDLE.write().await = Some(spawn_pinger());
}

pub async fn close_pinger_global() {
    PINGER_HANDLE.read().await.as_ref().map(|h| h.abort());
    *PINGER_HANDLE.write().await = None;
}