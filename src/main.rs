pub mod data;
pub use data::app;
pub use data::win;

use app::interface;
use data::app::network;
use data::app::pinger;
use win::wlan::network::profile::Key;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;


lazy_static! {
    pub static ref DATA_PATH: PathBuf = PathBuf::from("./wifu-data");
    pub static ref CFG_PATH: PathBuf = DATA_PATH.join("cfg.json");
    pub static ref CHANNEL: (tokio::sync::mpsc::Sender<()>, tokio::sync::RwLock<tokio::sync::mpsc::Receiver<()>>) = {
        let (tx, rx) = tokio::sync::mpsc::channel::<()>(100);
        (tx, tokio::sync::RwLock::new(rx))
    };
}

pub static CONFIG: OnceCell<data::app::cfg::Config> = OnceCell::new();
pub static WLAN: OnceCell<Arc<win::Wlan>> = OnceCell::new();


#[tokio::main]
async fn main() {
    app::init_fs().await;

    let config = CONFIG.get().unwrap();
    if let Err(reasons) = config.is_valid() {
        println!("x CONFIG is invalid due to the following reasons: {:?}", reasons);
        return;
    }
    let pinger = &pinger::PINGER;

    WLAN.set(Arc::new(win::Wlan::new(win::wlan::ClientVersion::Second).unwrap())).unwrap();
    let wlan = WLAN.get().unwrap();

    interface::init_globals().await;
    network::init_globals().await;

    interface::spawn_all_handles().await;

    let interfaces = interface::LIST.as_ref();
    let chosen_interface = interface::CHOSEN_AS_GUID.as_ref();
    let interfaces_update_sender = interface::UPDATE_SENDER.get().unwrap();
    let interfaces_update_recv = interface::UPDATE_RECV.get().unwrap();

    app::run().await;

    std::thread::park();

    loop {
        let mut list_interface_error_printed = false;
        let mut no_interfaces_error_printed = false;

        let mut interfaces;
        loop {
            let result = wlan.list_interfaces();
            if let Err(err) = result.as_ref() {
                if list_interface_error_printed { continue; }
                println!("could not list wireless interfaces, error: {:?}, will retry", err);
                list_interface_error_printed = true;
                continue;
            }
            interfaces = result.unwrap();

            list_interface_error_printed = false;

            if interfaces.len() < 1 {
                if no_interfaces_error_printed { continue; }
                println!("no wireless interfaces found, connect one");
                no_interfaces_error_printed = true;
                continue;
            }
            break;
        }

        return;
    
        let interface = {
            let mut selected = None;
    
            for preference in config.interfaces.priority.iter() {
                let iface = interfaces.iter().find(|iface| &iface.description == preference);
                if iface.is_none() {
                    continue
                }
    
                selected = iface;
                break
            }
    
            if selected.is_none() {
    
            }
        };
    }

    loop {
        let wlan = win::Wlan::new(win::wlan::ClientVersion::Second).unwrap();
        println!("wlan={:#?}", wlan);
        let ifs = wlan.list_interfaces().unwrap();
        println!("ifs={:#?}", ifs);

        if ifs.is_empty() {
            println!("no wlan adapters available");
            continue
        }

        let iface = &ifs[0];

        let scan = wlan.scan(&iface.guid).await;
        if let Err(err) = scan {
            println!("scan failed with error {:?}", err);
            continue
        }
        let scan = scan.unwrap();
        println!("scan={:#?}", scan);

        tokio::time::sleep(Duration::from_secs(3)).await;

        let networks = wlan.available_networks(&iface.guid);
        if let Err(err) = networks {
            println!("network listing failed with error {:?}", err);
            std::thread::park();
            continue
        }
        let networks = networks.unwrap();
        println!("networks={:#?}", networks);
    
        let network = networks.iter().find(|n| n.ssid == "***REMOVED***");
        if network.is_none() {
            println!("no desired network available");
            continue
        }
        let network = network.unwrap();
        println!("network={:?}", network);
    
        let mut profile = wlan.get_profile(&iface.guid, &network.ssid);
        if profile.is_err() {
            match profile.as_ref().unwrap_err() {
                win::NativeError::NotFound => println!("no profile for desired network, setting..."),
                _ => { profile.unwrap(); }
            }

            let new_profile = network.clone().to_profile(Some(Key::from_plain("***REMOVED***")));
            wlan.set_profile(&iface.guid, new_profile.clone()).unwrap();
            profile = Ok(new_profile)
        }
        let profile = profile.unwrap();
        println!("profile={:?}", profile);
    
        println!("connecting...");
        let connection = wlan.connect(&iface.guid, &profile.name, &network.bss).await;
        println!("connection={:?}", connection);
        
        println!("disconnecting...");
        let disconnection = wlan.disconnect(&iface.guid).await;
        println!("disconnection={:?}", disconnection);

        //std::thread::park();
    }
}
