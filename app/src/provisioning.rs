use crate::qevent::BiQueue;
use esp_idf_svc::sys as idf;
use idf::ESP_OK;
use std::ffi::c_int;
use std::ptr;
use std::sync::Mutex;
//
//
//
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
enum ProvEvent {
    Init = idf::network_prov_cb_event_t_NETWORK_PROV_INIT as c_int,
    Start = idf::network_prov_cb_event_t_NETWORK_PROV_START as c_int,
    CredRecv = idf::network_prov_cb_event_t_NETWORK_PROV_WIFI_CRED_RECV as c_int,
    CredFail = idf::network_prov_cb_event_t_NETWORK_PROV_WIFI_CRED_FAIL as c_int,
    ProvSuccess = idf::network_prov_cb_event_t_NETWORK_PROV_WIFI_CRED_SUCCESS as c_int,
    ProvEnd = idf::network_prov_cb_event_t_NETWORK_PROV_END as c_int,
}

impl ProvEvent {
    pub fn from_raw(raw: idf::network_prov_cb_event_t) -> Option<Self> {
        Some(match raw {
            idf::network_prov_cb_event_t_NETWORK_PROV_INIT => Self::Init,
            idf::network_prov_cb_event_t_NETWORK_PROV_START => Self::Start,
            idf::network_prov_cb_event_t_NETWORK_PROV_WIFI_CRED_RECV => Self::CredRecv,
            idf::network_prov_cb_event_t_NETWORK_PROV_WIFI_CRED_FAIL => Self::CredFail,
            idf::network_prov_cb_event_t_NETWORK_PROV_WIFI_CRED_SUCCESS => Self::ProvSuccess,
            idf::network_prov_cb_event_t_NETWORK_PROV_END => Self::ProvEnd,
            // … add more
            _ => return None,
        })
    }
}

fn print_nvm_config() {
    unsafe {
        let mut wifi_config: idf::wifi_config_t = core::mem::zeroed();
        let err = idf::esp_wifi_get_config(idf::wifi_interface_t_WIFI_IF_STA, &mut wifi_config);

        if err == idf::ESP_OK {
            let ssid_slice = &wifi_config.sta.ssid[0..wifi_config
                .sta
                .ssid
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(32)];
            let ssid = core::str::from_utf8(ssid_slice).unwrap_or("<invalid>");
            log::info!("Stored SSID via esp_wifi_get_config: '{}'", ssid);
            // Password is in wifi_config.sta.password (but zeroed if privacy features are on)
        } else {
            log::warn!("esp_wifi_get_config failed: {}", err);
        }
    }
}

/*
 *
 */
unsafe extern "C" fn ev_callback(
    _user_data: *mut std::ffi::c_void,
    event: idf::network_prov_cb_event_t,
    _event_data: *mut std::ffi::c_void,
) {
    log::info!(" RECEIVED Raw EVENT : {}", event);
    let prov_event = match ProvEvent::from_raw(event) {
        Some(x) => x,
        _ => {
            log::info!("unsupported");
            return;
        }
    };

    log::info!(" EVENT : {:?}", prov_event);
    match prov_event {
        ProvEvent::Init | ProvEvent::Start | ProvEvent::CredRecv => {}
        ProvEvent::ProvSuccess => {
            print_nvm_config();
        }
        ProvEvent::CredFail | ProvEvent::ProvEnd => {
            log::info!("Rebooting....\n");
            unsafe {
                idf::esp_restart();
            }
        }
    } //_ => log::info!("unsupported event"),
}
//
#[allow(dead_code)]
pub fn clear_provision() {
    /*
    unsafe {
        esp_idf_sys::nvs_flash_erase();
    }
    unsafe {
        esp_idf_sys::nvs_flash_init();
    }
    */
    unsafe {
        let mut handle: idf::nvs_handle_t = 0;
        log::info!("Erasing NVM \n");

        // Open NVS in read-write mode (namespace is usually "wifi" for provisioning)
        let err = idf::nvs_open(
            c"wifi".as_ptr(),
            idf::nvs_open_mode_t_NVS_READWRITE,
            &mut handle,
        );

        if err != ESP_OK {
            log::info!("NVM Erase failure!\n");
            return;
        }

        idf::nvs_erase_all(handle);
        idf::nvs_commit(handle);
        idf::nvs_close(handle);
        log::info!("NVM Erased\n");
    }
}
//
pub fn is_provisioned() -> bool {
    print_nvm_config();
    let mut provisioned = false;
    unsafe {
        idf::network_prov_mgr_is_wifi_provisioned(&mut provisioned);
    }
    log::info!("Device provisioned ? <{provisioned}>");
    provisioned
}
//
pub fn init() {
    unsafe {
        // 2. Provisioning manager config
        let prov_cfg = idf::network_prov_mgr_config_t {
            scheme: idf::network_prov_scheme_ble,
            scheme_event_handler: idf::network_prov_event_handler_t {
                event_cb: None,
                user_data: ptr::null_mut(),
            },
            app_event_handler: idf::network_prov_event_handler_t {
                event_cb: Some(ev_callback),
                user_data: ptr::null_mut(),
            },
            network_prov_wifi_conn_cfg: idf::network_prov_wifi_conn_cfg_t {
                wifi_conn_attempts: 0,
            },
        };
        idf::network_prov_mgr_init(prov_cfg);
    }
}
//
#[allow(dead_code)]
pub fn provision(_event_queue: &'static Mutex<BiQueue>) {
    // 3. Check if already provisioned
    log::info!("=== Device NOT provisioned ===");
    log::info!("Starting BLE provisioning... Use the official Espressif Provisioning app");
    /*
    unsafe {
        esp_idf_sys::nvs_flash_erase();
    }
    unsafe {
        esp_idf_sys::nvs_flash_init();
    }
    */
    let pop = c"abcd1234";
    let service_name = c"SWINDLE_ESP32";
    unsafe {
        idf::network_prov_mgr_start_provisioning(
            idf::network_prov_security_NETWORK_PROV_SECURITY_1,
            pop.as_ptr() as *const core::ffi::c_void, // POP pointer
            service_name.as_ptr(),
            ptr::null(), // service_key = NULL for BLE
        );
    }
    log::info!("BLE provisioning active – connect with the app now!");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    //Self::emit(event_queue, SwindleEvents::Configured);
}
// EOF
