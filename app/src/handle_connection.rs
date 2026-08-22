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
    fn push(&mut self, size: u32, data: *const u8) -> bool {
        // Case 1: The incoming data is larger than the buffer capacity.
        // Flush any existing buffered data first, then send the new data directly.
        if size >= TX_BUFFER_SIZE as u32 {
            if !self.flush() {
                return false;
            }
            return self.socket_write(size, data);
        }

        // Case 2: The data fits in the buffer overall, but there isn't enough room right now.
        // Flush the buffer to make space.
        if self.tx_index + size >= TX_BUFFER_SIZE as u32 {
            if !self.flush() {
                return false;
            }
        }

        // Case 3: The data is now guaranteed to safely fit in the buffer.
        unsafe {
            let dest_ptr = self.tx_buffer.as_mut_ptr().add(self.tx_index as usize);
            std::ptr::copy_nonoverlapping(data, dest_ptr, size as usize);
        }
        self.tx_index += size;
        
        true
    }
    //
    fn socket_write(&mut self, mut size: u32, mut data: *const u8) -> bool {
        if let Some(ref mut writer) = self.fd {
            while size > 0 {
                let sent = unsafe {
                    libc::send(*writer, data as *const libc::c_void, size as usize, 0)
                };
                if sent < 0 {
                    let err = std::io::Error::last_os_error();
                    if err.kind() == std::io::ErrorKind::Interrupted {
                        continue; // EINTR, safe to retry
                    }
                    // Fatal network error (e.g., EPIPE, ECONNRESET). 
                    // Invalidate socket to prevent further pointless write attempts.
                    self.fd = None;
                    return false;
                } else if sent == 0 {
                    // Connection cleanly closed by peer
                    self.fd = None;
                    return false;
                }
                
                size -= sent as u32;
                data = unsafe { data.add(sent as usize) };
            }
            true
        } else {
            false // Socket was already dead/invalidated
        }
    }
    //
    fn flush(&mut self) -> bool {
        if self.tx_index != 0 {
            let success = self.socket_write(self.tx_index, self.tx_buffer.as_ptr());
            // Always reset the index even on failure, so we don't infinitely retry failed data
            self.tx_index = 0;
            success
        } else {
            true
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
    let _success = WRITE_HALF.with(|ctx| {
        ctx.flush()
    });
    // NOTE: Failure cannot be propagated further because this C-FFI API is 
    // expected by the swindle backend to return `void`. 
    // (If `_success` is false, `self.fd` is internally set to None, safely ignoring future writes).
}
/*
 *
 */
#[unsafe(no_mangle)]
pub extern "C" fn rngdb_send_data_c(sz: u32, ptr: *const u8) {
    let _success = WRITE_HALF.with(|ctx| {
        ctx.push(sz, ptr)
    });
    // NOTE: Failure cannot be propagated further because this C-FFI API is 
    // expected by the swindle backend to return `void`.
    // (If `_success` is false, `self.fd` is internally set to None, safely ignoring future writes).
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
            log::info!(
                "GDB Handler running on core: {:?}",
                esp_idf_svc::sys::xTaskGetCoreID(std::ptr::null_mut())
            );
        }
        loop {
            match stream.read(buffer) {
                Ok(n) if n > 0 => unsafe {
                    rngdbstub_run(n as u32, buffer.as_ptr());
                    // The target may already have halted (e.g. single-step, or
                    // it hit a breakpoint between packets). rngdbstub_poll()
                    // sends the stop reply immediately in that case; it is a
                    // no-op unless the target is actually running.
                    rngdbstub_poll();
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
                    // No GDB data pending (socket read timeout is 20 ms above).
                    // While the target is running this is the ONLY place that
                    // polls it: if it halted on a breakpoint/watchpoint/fault
                    // the stop reply (T05/T11/...) must be sent or GDB will
                    // wait forever after `continue`. Without this call the
                    // poll never happens on the TCP build (rngdbstub_poll was
                    // declared but never invoked), which is exactly why
                    // breakpoints appeared to "not work".
                    unsafe {
                        rngdbstub_poll();
                    }
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
