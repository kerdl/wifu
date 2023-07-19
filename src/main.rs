pub mod data;
pub mod handlers;

use data::win;

use std::net::{ToSocketAddrs, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use serde::Serialize;
use winping::{AsyncPinger, Buffer};
use windows::Win32::NetworkManagement::WiFi;
use windows::Win32::Foundation::HANDLE;


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

    loop {
        let mut wlan = win::Wlan::new(win::wlan::ClientVersion::Second).await.unwrap();
        println!("wlan={:#?}", wlan);
        let ifs = wlan.list_interfaces().unwrap();
        println!("ifs={:#?}", ifs);
        let scan = wlan.scan(ifs[0].guid).await.unwrap();
        println!("scan={:#?}", scan);
        std::thread::park()
    }
    

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

    println!("wlan_interfaces_deref={:?}", wlan_interfaces_deref);


    if wlan_interfaces_deref.dwNumberOfItems < 1 {
        println!("no wlan devices detected");
        return
    }
    
    println!("wlan_interfaces={:?}", wlan_interfaces_deref);

    let mut first_interface = wlan_interfaces_deref.InterfaceInfo.get(0).unwrap().to_owned();
    let desc = widestring::U16CStr::from_slice_truncate(first_interface.strInterfaceDescription.as_slice()).unwrap().to_string().unwrap();
    println!("{}", desc);

    unsafe {
        println!("cleaning wlan_interfaces memory...");
        WiFi::WlanFreeMemory(wlan_interfaces as *const core::ffi::c_void);
    }

    let wifi_network = config.wifis.networks.get(0).unwrap();
    let wifi_network_ssid_u16 = widestring::U16CString::from_str(&wifi_network.ssid).unwrap();
    let wifi_network_ssid_pcwstr = windows::core::PCWSTR::from_raw(wifi_network_ssid_u16.as_ptr());

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
    let selected_network = selected_network.unwrap();

    println!("!!!!!!!!!!!!! selected_network={:?}", selected_network);

    let profile = data::profile::WLANProfile {
        xmlns: data::profile::XMLNS_PROFILE_V1.to_string(),
        name: wifi_network.ssid.clone(),
        ssid_config: data::profile::SSIDConfig::from_string(wifi_network.ssid.clone()),
        connection_type: data::profile::ConnectionType::from_dot11_bss_type(selected_network.dot11BssType),
        connection_mode: data::profile::ConnectionMode::Manual,
        auto_switch: None,
        msm: data::profile::MSM {
            security: data::profile::Security {
                auth_encryption: data::profile::AuthEncryption {
                    authentication: data::profile::Authentication::from_dot11_auth_algorithm(selected_network.dot11DefaultAuthAlgorithm),
                    encryption: data::profile::Encryption::from_dot11_cipher_algorithm(selected_network.dot11DefaultCipherAlgorithm),
                    use_one_x: false,
                },
                // the encryption of this password is performed by WlanSetProfile
                shared_key: if wifi_network.password.is_some() {
                    let k = data::profile::SharedKey {
                        key_type: data::profile::KeyType::PassPhrase,
                        protected: false,
                        key_material: wifi_network.password.as_ref().unwrap().clone()
                    };
                    Some(k)
                } else {
                    None
                },
            }
        },
        mac_randomization: data::profile::MacRandomization {
            xmlns: data::profile::XMLNS_PROFILE_V3.to_string(),
            enable_randomization: false,
        }
    };

    println!("profile: {:#?}", profile);

    unsafe {
        let mut xml_profile: String = "".to_string();
        let mut ser = quick_xml::se::Serializer::new(&mut xml_profile);
        ser.indent('\t', 1);
        profile.serialize(ser).unwrap();
        xml_profile = format!("<?xml version=\"1.0\"?>\n{}\n", xml_profile);

        tokio::fs::write("./generated-profile.xml", xml_profile.as_bytes()).await.unwrap();

        let u16cstring = widestring::U16CString::from_str(&xml_profile).unwrap();
        println!("u16cstring={:?}", u16cstring);
        let pcwstr = windows::core::PCWSTR::from_raw(u16cstring.as_ptr());
        println!("pcwstr={:?}", pcwstr);
        println!("pcwstr.as_wide()={:?}", pcwstr.as_wide());

        println!("XML PROFILE: {}", String::from_utf16_lossy(pcwstr.as_wide()));

        let mut existing_profile = std::mem::zeroed();
        let existing_profile_result = WiFi::WlanGetProfile(
            wlan_handle,
            &first_interface.InterfaceGuid,
            wifi_network_ssid_pcwstr,
            None,
            &mut existing_profile,
            None,
            None
        );
        println!("existing_profile_result={}", existing_profile_result);

        if existing_profile_result == windows::Win32::Foundation::ERROR_NOT_FOUND.0 {
            println!("existing profile not found, setting...");
            let mut reason_code = 0;

            let result = WiFi::WlanSetProfile(
                wlan_handle,
                &first_interface.InterfaceGuid,
                0,
                pcwstr,
                None,
                false,
                None,
                &mut reason_code
            );

            println!("reason_code={}", reason_code);
            println!("WlanSetProfile -> {}", result);
        } else {
            println!("existing_profile={:?}", String::from_utf16_lossy(existing_profile.as_wide()));
        }

        let mut params = WiFi::WLAN_CONNECTION_PARAMETERS {
            wlanConnectionMode: WiFi::wlan_connection_mode_profile,
            strProfile: wifi_network_ssid_pcwstr,
            pDot11Ssid: std::ptr::null_mut(),
            pDesiredBssidList: std::ptr::null_mut(),
            dot11BssType: selected_network.dot11BssType,
            //dwFlags: WiFi::WLAN_CONNECTION_HIDDEN_NETWORK,
            dwFlags: 0,
            //..Default::default()
        };
    
        tokio::time::sleep(Duration::from_secs(1)).await;
        let result = WiFi::WlanConnect(
            wlan_handle,
            &first_interface.InterfaceGuid,
            &params,
            None
        );
        println!("WlanConnect -> {}", result);
    }

    let last_err = unsafe { windows::Win32::Foundation::GetLastError() };
    println!("GetLastError -> {:?}", last_err);

    unsafe {
        tokio::time::sleep(Duration::from_secs(1)).await;
        WiFi::WlanDisconnect(wlan_handle, &first_interface.InterfaceGuid, None);
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
