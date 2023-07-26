use crate::app;
use crate::app::wlan::event;
use crate::app::wlan::interface;
use crate::app::wlan::network;
use crate::win::wlan::acm::notification::Code as AcmNotifCode;

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use once_cell::sync::Lazy;


pub static HANDLE: Lazy<Arc<RwLock<Option<JoinHandle<()>>>>> = Lazy::new(
    || Arc::new(RwLock::new(None))
);


pub async fn event_loop() {
    let mut receiver = super::RECEIVER.get().unwrap().resubscribe();

    loop {        
        let notif = receiver.recv().await.unwrap();

        match notif.code {
            AcmNotifCode::InterfaceArrival => {
                let chosen_something_else = interface::CHOSEN.write().await.choose().await.is_some();

                if chosen_something_else {
                    if app::STATE.read().await.is_dead() {
                        app::STATE.write().await.alive().unwrap();
                    }

                    network::restart().await
                }
            },
            AcmNotifCode::InterfaceRemoval => {
                network::LIST.write().await.clear();

                let list = interface::LIST.read().await;
                let chosen = interface::CHOSEN.read().await;

                let that_was_the_only_interface = {
                    chosen.is_guid_chosen(&notif.interface.guid) && list.is_empty()
                };
                let have_other_interfaces = {
                    !list.is_empty()
                };

                if that_was_the_only_interface {
                    std::mem::drop(list);
                    std::mem::drop(chosen);
                    interface::CHOSEN.write().await.unchoose().await.unwrap();
                    network::end().await;
                    println!("interface autopilot calls dead");
                    app::STATE.write().await.dead(app::DeadReason::NoInterface).unwrap();
                } else if have_other_interfaces {
                    std::mem::drop(list);
                    std::mem::drop(chosen);

                    let chosen_something_else = interface::CHOSEN.write().await.choose().await.is_some();

                    if chosen_something_else {
                        if app::STATE.read().await.is_dead() {
                            app::STATE.write().await.alive().unwrap();
                        }
    
                        network::restart().await
                    }
                } else {
                    println!("interface autopilot calls dead");
                    app::STATE.write().await.dead(app::DeadReason::NoInterface).unwrap();
                }
            },
            _ => ()
        }
    }
}

event::looping::works!(async fn works(HANDLE));
event::looping::spawner!(async fn spawn_event_loop(HANDLE, event_loop, works));
event::looping::closer!(async fn close_event_loop(HANDLE));
