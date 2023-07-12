pub mod data;
pub mod handlers;

use std::alloc::Layout;
use std::net::{ToSocketAddrs, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use windows::core::PCWSTR;
use winping::{AsyncPinger, Buffer};
use windows::Win32::NetworkManagement::WiFi;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::Cryptography;
use widestring::{U16CStr, U16CString};


lazy_static! {
    pub static ref DATA_PATH: PathBuf = PathBuf::from("./wifu-data");
    pub static ref CFG_PATH: PathBuf = DATA_PATH.join("cfg.json");
    pub static ref CHANNEL: (tokio::sync::mpsc::Sender<()>, tokio::sync::RwLock<tokio::sync::mpsc::Receiver<()>>) = {
        let (tx, rx) = tokio::sync::mpsc::channel::<()>(100);
        (tx, tokio::sync::RwLock::new(rx))
    };
}

pub static CONFIG: OnceCell<data::cfg::Config> = OnceCell::new();


#[tokio::main]
async fn main() {
    data::init_fs().await;
    let config = CONFIG.get().unwrap();

    println!("{:?}", config);

    let wlan_handle = unsafe {
        // 1 = Client version for Windows XP with SP3 and Wireless LAN API for Windows XP with SP2
        // 2 = Client version for Windows Vista and Windows Server 2008
        let client_version = 2;
        let mut handle: HANDLE = std::mem::zeroed();
        let mut negotiated_version: u32 = std::mem::zeroed();

        let result = WiFi::WlanOpenHandle(client_version, None, &mut negotiated_version, &mut handle);
        println!("negotiated_version={}", negotiated_version);
        println!("WlanOpenHandle -> {}", result);

        handle
    };

    println!("wlan_handle={:?}", wlan_handle);

    let wlan_interfaces = unsafe {
        let mut list = std::mem::zeroed();

        let result = WiFi::WlanEnumInterfaces(wlan_handle, None, &mut list);
        println!("WlanEnumInterfaces -> {}", result);

        list
    };
    let wlan_interfaces_deref = unsafe { *wlan_interfaces };

    if wlan_interfaces_deref.dwNumberOfItems < 1 {
        println!("no wlan devices detected");
        return
    }
    
    println!("wlan_interfaces={:?}", wlan_interfaces_deref);

    let mut first_interface = wlan_interfaces_deref.InterfaceInfo.get(0).unwrap().to_owned();
    let desc = U16CStr::from_slice_truncate(first_interface.strInterfaceDescription.as_slice()).unwrap().to_string().unwrap();
    println!("{}", desc);

    unsafe {
        println!("cleaning wlan_interfaces memory...");
        WiFi::WlanFreeMemory(wlan_interfaces as *const core::ffi::c_void);
    }

    let wifi_network = config.wifis.networks.get(0).unwrap();

    let mut password_in = Cryptography::CRYPT_INTEGER_BLOB {
        cbData: wifi_network.password.len() as u32,
        pbData: wifi_network.password.as_ptr() as *mut u8,
    };
    unsafe {
        println!("password_in={:?}", std::ptr::slice_from_raw_parts(password_in.pbData, password_in.cbData as usize).as_ref());
    }
    

    let mut raw_encrypted_password = Cryptography::CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    unsafe {
        let result = Cryptography::CryptProtectData(
            &mut password_in,
            std::mem::zeroed::<PCWSTR>(),
            std::mem::zeroed(),
            None,
            std::mem::zeroed(),
            Cryptography::CRYPTPROTECT_VERIFY_PROTECTION,
            &mut raw_encrypted_password
        );
        println!("CryptProtectData -> {:?}", result);
    };

    println!("CryptProtectData encrypted_out -> {:?}", raw_encrypted_password);

    unsafe {
        println!("password_in={:?}", std::ptr::slice_from_raw_parts(password_in.pbData, password_in.cbData as usize).as_ref());
    }
    
    let encrypted_password = unsafe {
        let content_ptr = std::ptr::slice_from_raw_parts(raw_encrypted_password.pbData, raw_encrypted_password.cbData as usize);
        let content_ref = content_ptr.as_ref().unwrap();
        let string = hex::encode_upper(content_ref);
        println!("encrypted wifi password: {}", string);
        string
    };

    unsafe {
        unsafe extern "system" fn scan_list_refresh(notify: *mut WiFi::L2_NOTIFICATION_DATA, context: *mut core::ffi::c_void) {
            println!("scan_list_refresh! \nNOTIFY: {:?} \nCONTEXT: {:?}\n\n", *notify, *context);
            (&CHANNEL.0).blocking_send(()).unwrap();
        }

        WiFi::WlanRegisterNotification(
            wlan_handle,
            WiFi::wlan_notification_acm_scan_list_refresh.0 as u32,
            None,
            Some(scan_list_refresh),
            None,
            None,
            None
        );

        let start = tokio::time::Instant::now();
        let result = WiFi::WlanScan(wlan_handle, &mut first_interface.InterfaceGuid, None, None, None);
        println!("WlanScan -> {}", result);

        println!("waiting for lock release!");

        (&CHANNEL.1).write().await.recv().await;
        println!("first lock released! NIG took {:?}", start.elapsed());

        loop {
            let t = tokio::time::timeout(Duration::from_millis(300), (&CHANNEL.1).write().await.recv()).await;
            if let Err(err) = t {
                println!("scan timeout {}, continue", err.to_string());
                break
            }
        }

        println!("lock released! NIG took {:?}", start.elapsed());
    }

    let networks = unsafe {
        let mut networks_out = std::mem::zeroed();

        let result = WiFi::WlanGetAvailableNetworkList(
            wlan_handle,
            &mut first_interface.InterfaceGuid,
            WiFi::WLAN_AVAILABLE_NETWORK_INCLUDE_ALL_MANUAL_HIDDEN_PROFILES,
            None,
            &mut networks_out
        );

        println!("WlanGetAvailableNetworkList -> {}", result);
        println!("networks: {:?}", *networks_out);

        networks_out
    };
    
    let mut networks_vec = vec![];

    unsafe {
        for i in 0..(*networks).dwNumberOfItems {
            let wifi = (*networks).Network.as_ptr().add(i as usize);
            networks_vec.push(*wifi);
        }
    }

    fn get_name(data: WiFi::DOT11_SSID) -> String {
        let slice = unsafe {
            std::ptr::slice_from_raw_parts(
                data.ucSSID.as_ptr(),
                data.uSSIDLength as usize
            ).as_ref().unwrap()
        };

        String::from_utf8_lossy(slice).to_string()
    }

    println!("networks_vec={:?}", networks_vec.iter().map(|n| get_name(n.dot11Ssid)).collect::<Vec<String>>());

    let selected_network = networks_vec.into_iter().find(|n| get_name(n.dot11Ssid) == wifi_network.ssid);
    if selected_network.is_none() {
        println!("{} was not found in scanned wifis", wifi_network.ssid);
        return;
    }
    let mut selected_network = selected_network.unwrap();


    let profile = data::profile::WLANProfile {
        name: wifi_network.ssid.clone(),
        ssid_config: data::profile::SSIDConfig::from_string(wifi_network.ssid.clone()),
        connection_type: data::profile::ConnectionType::from_dot11_bss_type(selected_network.dot11BssType),
        connection_mode: data::profile::ConnectionMode::Manual,
        auto_switch: false,
        msm: data::profile::MSM {
            security: data::profile::Security {
                auth_encryption: data::profile::AuthEncryption {
                    authentication: data::profile::Authentication::from_dot11_auth_algorithm(selected_network.dot11DefaultAuthAlgorithm),
                    encryption: data::profile::Encryption::from_dot11,
                    use_one_x: false,
                },
                shared_key: data::profile::SharedKey {
                    key_type: data::profile::KeyType::PassPhrase,
                    protected: true,
                    key_material: encrypted_password,
                }
            }
        },
        mac_randomization: data::profile::MacRandomization {
            enable_randomization: false,
        }
    };

    unsafe {
        let mut params = WiFi::WLAN_CONNECTION_PARAMETERS {
            wlanConnectionMode: WiFi::wlan_connection_mode_temporary_profile,
            strProfile: PCWSTR::from_raw(U16CString::from_str(wifi_network.ssid.as_str()).unwrap().as_ptr()),
            ..Default::default()
        };
    
        let result = WiFi::WlanConnect(wlan_handle, &mut first_interface.InterfaceGuid, &mut params, None);
        println!("WlanConnect -> {}", result);
    }

    unsafe {
        println!("raw_encrypted_password.pbData={:?}", raw_encrypted_password.pbData);
        println!("*raw_encrypted_password.pbData={:?}", *raw_encrypted_password.pbData);
        println!("LocalFree...");
        let result = windows::Win32::System::Memory::LocalFree(windows::Win32::Foundation::HLOCAL(raw_encrypted_password.pbData as isize));
        println!("raw_encrypted_password LocalFree -> {:?}", result);
        //let last_err = windows::Win32::Foundation::GetLastError();
        //println!("GetLastError -> {:?}", last_err);
        //println!("cleaning password_in_ptr memory...");
        //Cryptography::CryptMemFree(Some(password_in_ptr as *const core::ffi::c_void));
        //println!("cleaning encrypted_out_ptr memory...");
        //Cryptography::CryptMemFree(Some(encrypted_out_ptr as *const core::ffi::c_void));
        WiFi::WlanFreeMemory(networks as *const core::ffi::c_void);
        WiFi::WlanCloseHandle(wlan_handle, None)
    };
    std::thread::park();
    return;

    let mut ips: Vec<SocketAddr> = vec![];
    let mut pinger = AsyncPinger::new();
    let mut buf = Buffer::new();

    pinger.set_timeout(config.ping.timeout_ms);

    loop {
        for domain in CONFIG.get().unwrap().ping.domains.iter() {
            let domain_ips = format!("{}:80", domain).to_socket_addrs();
            if let Err(err) = domain_ips {
                println!("{}: {}", domain, err);
                continue
            }
            let mut domain_ips = domain_ips.unwrap();
    
            let Some(first_ip) = domain_ips.nth(0) else { continue; };
            ips.push(first_ip);
        }

        if ips.is_empty() {
            println!("no ips were found for domains specified in the config, retrying in 5s");
            tokio::time::sleep(Duration::from_secs(5)).await;
        } else {
            break
        }
    }

    println!("will ping these ips: {:?}", ips);

    'infinite: loop {
        let mut errors = 0;

        'ips: for ip in ips.iter() {
            println!("{}: starting ping", ip);

            'ip: loop {
                let answer = pinger.send(ip.ip(), buf).await;
                buf = answer.buffer;

                match answer.result {
                    Ok(rtt) => {
                        println!("{}: rtt={}", ip, rtt);
                        errors = 0;
                    },
                    Err(err) => {
                        println!("{}: err={}", ip, err);
                        errors += 1;
                        break 'ip
                    },
                }

                tokio::time::sleep(Duration::from_millis(config.ping.interval_ms)).await;
            }

            if errors == config.ping.max_errors {
                println!("switch wifi")
            }
        }
    }
}
