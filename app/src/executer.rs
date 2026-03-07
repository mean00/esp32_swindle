#![allow(dead_code)]
//
use crate::SwindleStateTrait;
use esp_idf_svc::eventloop::EspSystemEventLoop;

use crate::fsm::SwindleEvents;
use crate::qevent::BiQueue;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use std::net::TcpListener;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
//
use crate::fsm_led;
use crate::settings;
//
//
use crate::handle_connection::stop_gdb;
//
//
unsafe extern "C" {
    fn pins_init();
}
//
use esp_idf_svc::hal::cpu::Core;
use esp_idf_svc::hal::task::thread::ThreadSpawnConfiguration; //
//
use esp_idf_svc::sys as idf;
//
pub enum WifiCommand {
    StartDhcp,
    Stop,
    Provision,
    StartServer,
    KillGdb,
}

pub enum GdbControl {
    StartServer,
    StopGdb,
}

//
pub struct SwindleExecutor {
    cmd_tx: Sender<WifiCommand>,
    //gdb_tx: Sender<GdbControl>,
    //gdb_rx: Receiver<GdbControl>,
}

impl SwindleExecutor {
    /*
     * This thread will handle gdb connection
     * The same thread will be reused over and over
     */
    fn gdb_handler(
        _tx: Sender<GdbControl>,
        rx: Receiver<GdbControl>,
        event_queue: &'static Mutex<BiQueue>,
    ) {
        println!("Gdb slave started...");
        unsafe {
            pins_init();
        }
        // do the init here so that the fast io inits are on the same core as the slave
        while let Ok(cmd) = rx.recv() {
            match cmd {
                GdbControl::StartServer => {
                    println!("GDB Slave: Bound to port 8080, waiting for connection...");
                    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
                    fsm_led::set_color(settings::WS2812_WAITING);

                    // We use blocking accept since we are on our own thread
                    for stream in listener.incoming() {
                        match stream {
                            Ok(stream) => {
                                println!("New Gdb Connection ");
                                fsm_led::set_color(settings::WS2812_ATTACHED);
                                Self::emit(event_queue, SwindleEvents::Attach);
                                crate::handle_connection::handle_gdb(stream, event_queue);
                                fsm_led::set_color(settings::WS2812_WAITING);
                            }
                            Err(e) => {
                                println!("Error accepting connection: {}", e);
                                break;
                            }
                        }
                    }
                }
                _ => panic!("Unsupported cmd received by gdb slave"),
            }
        }
    }

    pub fn new(
        wifi_static: &'static mut EspWifi<'static>,
        lop: EspSystemEventLoop,
        event_queue: &'static Mutex<BiQueue>,
    ) -> Self {
        let (cmd_tx, cmd_rx) = channel::<WifiCommand>();
        let (gdb_tx, gdb_rx) = channel::<GdbControl>();
        let thread_loop = lop.clone();
        let timer_service = esp_idf_svc::timer::EspTaskTimerService::new().unwrap();

        // if its a dual core, pin it to core 1 so that it does not jump
        // from one core to the other . Fast GPIO would fail (ESP32S3)
        #[cfg(esp32s3)]
        let core = Core::Core1;
        #[cfg(esp32c6)]
        let core = Core::Core0;

        ThreadSpawnConfiguration::set(&ThreadSpawnConfiguration {
            name: Some(c"gdb".to_bytes_with_nul()),
            pin_to_core: Some(core),
            ..Default::default()
        })
        .unwrap();

        let tx_clone = gdb_tx.clone();
        let _gdb_handler = thread::Builder::new()
            .stack_size(10240)
            .spawn(move || {
                Self::gdb_handler(tx_clone, gdb_rx, event_queue);
            })
            .unwrap();

        // 🛑 CRITICAL: Reset the thread configuration back to default!
        ThreadSpawnConfiguration::set(&ThreadSpawnConfiguration::default()).unwrap();

        // Spawn the thread in charge of WIFI + listen
        thread::Builder::new()
            .stack_size(7168) // Give it enough stack for WiFi/TLS
            .name("swindle_executor_worker".into())
            .spawn(
                move || {
                    // Wrap the raw wifi into AsyncWifi inside the thread
                    let mut async_wifi =
                        AsyncWifi::wrap(wifi_static, thread_loop, timer_service).unwrap();

                    while let Ok(cmd) = cmd_rx.recv() {
                        // Use the native ESP-IDF block_on instead of tokio
                        esp_idf_svc::hal::task::block_on(async {
                            match cmd {
                                WifiCommand::StartDhcp => {
                                    Self::handle_dhcp(&mut async_wifi, event_queue).await;
                                }
                                WifiCommand::Provision => {
                                    Self::provision(event_queue).await;
                                }
                                WifiCommand::Stop => {
                                    let _ = async_wifi.stop().await;
                                }
                                WifiCommand::StartServer => {
                                    // Start the socket server logic
                                    println!("Executer: start_server");
                                    gdb_tx.send(GdbControl::StartServer).unwrap();
                                }
                                WifiCommand::KillGdb => {
                                    println!("Executer: stop gdb");
                                    stop_gdb(event_queue);
                                }
                            } // match
                        }) //async
                    } //cmd
                }, // spawn move
            )
            .unwrap();
        Self { cmd_tx }
    }
    //
    fn emit(q: &Mutex<BiQueue>, ev: SwindleEvents) {
        let mut lock = q.lock().unwrap();
        lock.send_queued(ev as u32);
    }
    //
    async fn provision(event_queue: &'static Mutex<BiQueue>) {
        crate::provisioning::provision(event_queue)
    }
    //
    async fn handle_dhcp(wifi: &mut AsyncWifi<&mut EspWifi<'static>>, q: &Mutex<BiQueue>) {
        println!("Worker: Starting Wifi & DHCP ");
        // The async sequence
        if !wifi.is_started().unwrap() {
            println!("Starting WiFi ");
            crate::wifi_util::set_country_code();
            crate::wifi_util::set_wifi_tx_power();
            if wifi.start().await.is_ok() {
                // You can still do connect() if needed, or just return
                println!("Wifi started...");
                // Disable Power Save mode to prevent DHCP retries due to Modem Sleep missing DHCP offers
                unsafe {
                    esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE);
                }
            } else {
                panic!("cannot start wifi"); // FIXME
            }
        } else {
            println!("Wifi already started");
        }

        println!("Worker: Starting DHCP...");

        if wifi.connect().await.is_ok() {
            match wifi.wait_netif_up().await {
                Ok(_) => {
                    println!("Worker: Got IP!");
                    let netif = wifi.wifi().sta_netif();
                    if let Ok(ip_info) = netif.get_ip_info() {
                        println!("Worker: Got IP! Address: {}", ip_info.ip);
                    }
                    Self::emit(q, SwindleEvents::IpReady);
                }
                Err(e) => {
                    println!("Worker: DHCP Failed: {:?}", e);
                    let _ = wifi.disconnect().await;
                    Self::emit(q, SwindleEvents::NetworkLoss);
                }
            }
        } else {
            println!("Worker: Failed to start/connect");
            let _ = wifi.disconnect().await;
            Self::emit(q, SwindleEvents::NetworkLoss);
        }
    }
}
// 2. Implement the trait
impl SwindleStateTrait for SwindleExecutor {
    fn start_dhcp(&self) {
        println!("FSM: Requesting DHCP for SSID"); //": {}", credentials::SSID);
        let _ = self.cmd_tx.send(WifiCommand::StartDhcp);
    }
    //
    fn reset_provisioning(&self) {
        unsafe {
            // Stop WiFi first (safe and recommended)
            let _ = idf::esp_wifi_disconnect();
            let _ = idf::esp_wifi_stop();

            let _ret = idf::wifi_prov_mgr_reset_provisioning();
        }
        fsm_led::set_color(settings::WS2812_RESET);
    }

    fn start_ble_provisioning(&self) {
        println!("Executor: Entering BLE provisioning mode...");
        let _ = self.cmd_tx.send(WifiCommand::Provision);
    }

    fn start_sockets(&self) {
        println!("Executor: Opening TCP/UDP sockets...");
        let _ = self.cmd_tx.send(WifiCommand::StartServer);
    }

    fn stop_sockets(&self) {
        println!("Executor: Closing all active sockets.");
        let _ = self.cmd_tx.send(WifiCommand::KillGdb);
    }

    fn start_swindle(&self) {
        println!("Executor: Swindle protocol ACTIVE.");
    }
    fn has_provisioning(&self) -> bool {
        crate::provisioning::is_provisioned()
    }

    fn stop_swindle(&self) {
        println!("Executor: Swindle protocol STOPPED.");
    }
}
// EOF
