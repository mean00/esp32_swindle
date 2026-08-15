/*
 * This file handles a single gdbstub link over TCP through a Task
 * The write part of the TcpStream is exctracted through its C interface
 * rather than using split
 */

#![allow(dead_code)]
#![allow(static_mut_refs)]
use crate::BiQueue;
use crate::SwindleEvents;
use lazy_static::lazy_static;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::os::fd::AsRawFd;
use std::sync::Mutex;
//
use crate::fsm_led;
use crate::settings;
//
unsafe extern "C" {
    pub unsafe fn rngdbstub_init();
    pub unsafe fn rngdbstub_shutdown();
    pub unsafe fn rngdbstub_run(size: u32, data: *const u8);
    pub unsafe fn rngdbstub_poll();
}
//
// this is the read buffer, allocated once
// it is strictly single threaded but we use Mutex etc.. to avoid static mut ref
static GDB_BUFFER: std::sync::OnceLock<std::sync::Mutex<Box<[u8; 1024]>>> =
    std::sync::OnceLock::new();
//
const TX_BUFFER_SIZE: usize = 256;
struct LocalContext {
    fd: Option<i32>,
    tx_index: u32,
    tx_buffer: [u8; TX_BUFFER_SIZE],
}
//
struct MutexedContext {
    inner: Mutex<LocalContext>,
}
//
impl MutexedContext {
    //
    fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut LocalContext) -> R,
    {
        f(&mut self.inner.lock().unwrap())
    }
}
impl LocalContext {
    //
    pub fn new() -> Self {
        LocalContext {
            fd: None,
            tx_index: 0,
            tx_buffer: [0u8; TX_BUFFER_SIZE],
        }
    }
    //app/src/handle_connection.rsapp/src/handle_connection.rs
    fn stop(&mut self) {
        self.tx_index = 0;
        self.fd = None;
    }
    //
    fn start(&mut self, socket_id: i32) {
        self.fd = Some(socket_id);
        self.tx_index = 0;
    }
    //
    fn push(&mut self, size: u32, data: *const u8) {
        if self.tx_index + size >= TX_BUFFER_SIZE as u32 {
            self.flush();
        }
        if size >= TX_BUFFER_SIZE as u32 {
            self.socket_write(size, data);
        } else {
            unsafe {
                let dest_ptr = self.tx_buffer.as_mut_ptr().add(self.tx_index as usize);
                std::ptr::copy_nonoverlapping(data, dest_ptr, size as usize);
            }
            self.tx_index += size;
        }
    }
    //
    fn socket_write(&mut self, size: u32, data: *const u8) {
        if let Some(ref mut writer) = self.fd {
            unsafe {
                libc::send(*writer, data as *const libc::c_void, size as usize, 0);
            }
        }
    }
    //
    fn flush(&mut self) {
        if self.tx_index != 0 {
            self.socket_write(self.tx_index, self.tx_buffer.as_ptr());
            self.tx_index = 0;
        }
    }
}

//static WRITE_HALF: Mutex<Option<i32>> = Mutex::new(None);
//
lazy_static! {
    static ref WRITE_HALF: MutexedContext = MutexedContext {
        inner: Mutex::new(LocalContext::new()),
    };
}
//
/*
 *
 */
#[unsafe(no_mangle)]
pub extern "C" fn rngdb_output_flush_c() {
    WRITE_HALF.with(|ctx| {
        ctx.flush();
    });
}
/*
 *
 */
#[unsafe(no_mangle)]
pub extern "C" fn rngdb_send_data_c(sz: u32, ptr: *const u8) {
    WRITE_HALF.with(|ctx| {
        ctx.push(sz, ptr);
    });
}
/*
 *
 *
 */
// The swindle C++ c_interface (bmp_interface_usb.cpp) registers this as the
// logger callback (setLogger). This build is a TCP probe with no USB serial
// bridge; the new rsbmp no longer provides the symbol in network mode, so we
// provide it here and forward the log text to the console.
#[unsafe(no_mangle)]
pub extern "C" fn rn_serial_bridge_write(n: i32, data: *const u8) {
    if n > 0 && !data.is_null() {
        let slice = unsafe { std::slice::from_raw_parts(data, n as usize) };
        let _ = std::io::stdout().write_all(slice);
    }
}
/*
 *
 *
 */
fn emit(q: &Mutex<BiQueue>, ev: SwindleEvents) {
    let mut lock = q.lock().unwrap();
    lock.send_queued(ev as u32);
}
/*
 *
 *
 */
pub fn handle_gdb(mut stream: TcpStream, q: &Mutex<BiQueue>) {
    unsafe {
        rngdbstub_init();
    }
    let _ = stream.set_nodelay(true);
    let timeout = std::time::Duration::from_millis(20);
    stream.set_read_timeout(Some(timeout)).ok();
    let socket_id = stream.as_raw_fd();
    WRITE_HALF.with(|ctx| {
        ctx.start(socket_id);
    });

    let mutex = GDB_BUFFER.get_or_init(|| std::sync::Mutex::new(Box::new([0u8; 1024])));
    if let Ok(mut buffer_guard) = mutex.lock() {
        // We get a mutable reference to the underlying array once
        //let buffer: &mut [u8; 1024] = &mut **buffer_guard;
        let buffer: &mut [u8; 1024] = &mut buffer_guard;
        unsafe {
            println!(
                "GDB Handler running on core: {:?}",
                esp_idf_svc::sys::xTaskGetCoreID(std::ptr::null_mut())
            );
        }
        loop {
            match stream.read(buffer) {
                Ok(n) if n > 0 => unsafe {
                    rngdbstub_run(n as u32, buffer.as_ptr());
                },
                Ok(_) => {
                    // n is 0, which means the socket was closed by the client
                    fsm_led::set_color(settings::WS2812_WAITING);
                    emit(q, SwindleEvents::Detach);
                    break;
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    // give back some CPU to the watchdog task
                    std::thread::sleep(std::time::Duration::from_micros(200));
                }
                Err(_) => {
                    // Any other hardware error or connection reset
                    fsm_led::set_color(settings::WS2812_WAITING);
                    emit(q, SwindleEvents::Detach);
                    break;
                }
            }
        }
    } else {
        panic!("Cannot get gdb read buffer");
    }
}
/*
 *
 */
pub fn stop_gdb(q: &Mutex<BiQueue>) {
    unsafe {
        rngdbstub_shutdown();
    }
    WRITE_HALF.with(|ctx| {
        ctx.stop();
    });
    emit(q, SwindleEvents::Detach);
}
// EOF
