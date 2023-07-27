use crate::app;
use crate::app::wlan::event;
use crate::app::wlan::interface;
use crate::app::wlan::network::{LIST, CHOSEN, event::{pinger, waiter}};
use crate::win::wlan::acm::notification::Code as AcmNotifCode;

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use once_cell::sync::Lazy;


pub static HANDLE: Lazy<Arc<RwLock<Option<JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);


pub async fn event_loop() {
    let mut receiver = interface::event::RECEIVER.get().unwrap().resubscribe();

    loop {
        let notif = receiver.recv().await.unwrap();

        if !interface::event::is_relevant(&notif.interface.guid).await {
            continue
        }

        match notif.code {
            AcmNotifCode::ScanListRefresh => {
                println!("network::autopilot: updating network list");
                LIST.write().await.update().await.unwrap();

                println!("network::autopilot: getting app state");
                let app_state = app::STATE.read().await;

                println!("network::autopilot: getting dead_because_no_network");
                let dead_because_no_network = {
                    app_state.is_dead()
                    && app_state.get_dead_reason().unwrap().is_no_network()
                };
                println!("network::autopilot: getting cfg_networks_available");
                let cfg_networks_available = LIST.read().await.cfg_networks_available();

                if dead_because_no_network && cfg_networks_available {
                    println!("network::autopilot: dead_because_no_network && cfg_networks_available");
                    std::mem::drop(app_state);

                    if waiter::works().await {
                        println!("network::autopilot: waiter works, closing");
                        waiter::close_event_loop().await;
                    }

                    println!("network::autopilot calls choose");
                    CHOSEN.write().await.choose().await.unwrap();
                    app::STATE.write().await.alive().unwrap();
                    pinger::spawn_event_loop().await;
                } else if !cfg_networks_available && !CHOSEN.read().await.is_chosen() && app_state.can_die() {
                    println!("network::autopilot: !cfg_networks_available && !CHOSEN.read().await.is_chosen() && app_state.can_die()");
                    std::mem::drop(app_state);
                    println!("network autopilot calls dead");
                    app::STATE.write().await.dead(app::DeadReason::NoNetwork).unwrap();
                    waiter::spawn_event_loop().await;
                } else if cfg_networks_available && app_state.is_dead() && app_state.get_dead_reason().unwrap().is_uninitialized() {
                    println!("network::autopilot: cfg_networks_available && app_state.is_dead() && app_state.get_dead_reason().unwrap().is_uninitialized()");
                    std::mem::drop(app_state);
                    println!("network autopilot calls choose");
                    CHOSEN.write().await.choose().await.unwrap();
                    app::STATE.write().await.alive().unwrap();
                    pinger::spawn_event_loop().await;
                } else if cfg_networks_available && app_state.is_alive() && !CHOSEN.read().await.is_chosen() {
                    println!("network::autopilot: cfg_networks_available && app_state.is_alive() && !CHOSEN.read().await.is_chosen()");
                    std::mem::drop(app_state);
                    println!("network autopilot calls choose");
                    CHOSEN.write().await.choose().await.unwrap();
                    pinger::spawn_event_loop().await;
                }
            },
            _ => ()
        }
    }
}

event::looping::works!(async fn works(HANDLE));
event::looping::spawner!(async fn spawn_event_loop(HANDLE, event_loop, works));
event::looping::closer!(async fn close_event_loop(HANDLE));
