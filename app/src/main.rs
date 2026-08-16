use esp_idf_svc::wifi::EspWifi;

use esp_idf_svc::eventloop::EspSystemEventLoop;
//--
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use num_traits::{FromPrimitive, ToPrimitive};
//
use esp_idf_svc::hal::peripherals::Peripherals;
//
use esp_idf_hal::gpio::{PinDriver, Pull};
//
#[allow(unused_imports)]
use rsbmp::native; // keep rsbmp linked: rngdbstub_init/run/shutdown are referenced from handle_connection via extern "C"
//
use swindle_if::swindle_sys_init;
//
mod fsm;
mod qevent;
use qevent::BiQueue;
use rust_sfsm::StateMachine;
//
use core::ffi::{c_int, c_void};
//
mod swindle_state_trait;
//
mod stubs_to_do;
mod swindle_if;
//
mod executer;
//
//mod credentials;
mod fsm_led;
mod settings;
mod wifi_util;
//
mod handle_connection;
//
mod provisioning;
//
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::sys::{ESP_OK, esp_err_t, esp_netif_set_hostname};
use std::ffi::CString;
//
use crate::swindle_state_trait::SwindleStateTrait;
use executer::SwindleExecutor;
use fsm::SwindleEvents;
use std::sync::Mutex;
//
//
unsafe extern "C" {
    unsafe fn bmp_get_frequency_c() -> u32;
    unsafe fn bmp_set_frequency_c(f: u32);
    unsafe fn __real_longjmp(env: *mut c_void, val: c_int) -> !;
}
fn bmp_set_frequency(f: u32) {
    unsafe { bmp_set_frequency_c(f) }
}
fn bmp_get_frequency() -> u32 {
    unsafe { bmp_get_frequency_c() }
}
//
fn emit(q: &Mutex<BiQueue>, ev: SwindleEvents) {
    let mut lock = q.lock().unwrap();
    lock.send_queued(ev as u32);
}
//
fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::hal::sys::link_patches();
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Startup");
    let asked: u32 = 1500 * 1000;
    bmp_set_frequency(asked); // 1.5 Megabits
    let fq = bmp_get_frequency(); // 1.5 Megabits
    log::info!("FQ asked {} got {}", asked, fq);

    swindle_sys_init();
    // Blink the LED to say we are starting
    fsm_led::init();
    for _i in 0..2 {
        fsm_led::set_color(settings::WS2812_PROVISIONING);
        std::thread::sleep(std::time::Duration::from_millis(50));
        fsm_led::set_color(settings::WS2812_OFF);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    let biqueue: qevent::BiQueue = BiQueue::new();
    let static_queue: &'static Mutex<BiQueue> = Box::leak(Box::new(Mutex::new(biqueue)));

    let internal_wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;

    let netif_handle = internal_wifi.sta_netif().handle();
    let hostname = CString::new("swindle_esp32").unwrap();
    unsafe {
        let err: esp_err_t = esp_netif_set_hostname(netif_handle, hostname.as_ptr());
        if err != ESP_OK {
            panic!("Failed to set hostname: {}", err);
        }
    }
    let wifi_static = Box::leak(Box::new(internal_wifi));
    let executor = SwindleExecutor::new(wifi_static, sys_loop.clone(), static_queue);
    let static_executor: &'static SwindleExecutor = Box::leak(Box::new(executor));

    let mut runner = fsm::SwindleState::new(static_executor);

    let mut initial_event: SwindleEvents = SwindleEvents::Start;

    unsafe {
        esp_idf_svc::hal::sys::gpio_reset_pin(esp_idf_svc::hal::sys::gpio_num_t_GPIO_NUM_1);
    }
    let user_button = PinDriver::input(peripherals.pins.gpio1, Pull::Up)?;
    std::thread::sleep(std::time::Duration::from_millis(50));

    if user_button.is_low() {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        if user_button.is_low() {
            initial_event = SwindleEvents::ResetConfiguration;
        }
    }

    provisioning::init();

    log::info!("Starting event loop...");
    // Start the even loop
    {
        let mut q = static_queue.lock().unwrap();
        q.take_ownership();
        q.send_queued(initial_event.to_u32().unwrap());
    }
    let mut count: u32 = 0;
    loop {
        let raw_event = {
            let mut q = static_queue.lock().unwrap();
            q.receive()
        };
        // just continue, it's just a timeout
        if raw_event == 0 {
            count += 1;
            if count > 10 {
                count = 0;
                log::info!(".");
            }
            continue;
        }
        if let Some(event) = fsm::SwindleEvents::from_u32(raw_event) {
            log::info!("Successfully received event: {:?}", event);
            runner.handle_event(&event);
        } else {
            log::info!("Received an unknown event ID: {}", raw_event);
        }
        // check for outbox, i.e. the FSM sending message to itself
        if let Some(outgoing_msg) = runner.take_outbox() {
            emit(static_queue, outgoing_msg);
        }
    }
}
//----------------------
