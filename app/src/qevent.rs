//use esp_idf_hal::prelude::*;
use esp_idf_hal::task::notify;
use esp_idf_sys as _; // required to link to IDF
use esp_idf_sys::{QueueHandle_t, xQueueGenericCreate, xQueueGenericSend, xQueueReceive};
use esp_idf_sys::{TaskHandle_t, xTaskGetCurrentTaskHandle};
use std::num::NonZeroU32;
/*
 */
const PD_TRUE: i32 = 1;
//const PORT_MAX_DELAY: u32 = u32::MAX;
const BIQUEUE_QUEUE_FLAG: u32 = 1u32 << 31;

struct MonoQueue {
    queue: QueueHandle_t,
}
impl MonoQueue {
    //
    pub fn new() -> Self {
        unsafe {
            let my_queue = xQueueGenericCreate(
                10,
                core::mem::size_of::<u32>() as u32, // we're sending u32 messages
                0,                                  // queue type = queue (not mutex/semaphore)
            );
            if my_queue.is_null() {
                panic!("cannot create queue");
            }
            MonoQueue { queue: my_queue }
        }
    }
    //
    pub fn send(&mut self, msg: u32) {
        unsafe {
            let sent = xQueueGenericSend(
                self.queue,
                &msg as *const u32 as *const _,
                0, // no block / timeout
                0, // send to back
            );
            if sent != PD_TRUE {
                panic!("Queue send failed");
            }
        }
    }
    //
    pub fn receive(&mut self) -> u32 {
        let mut msg: u32 = 0;
        unsafe {
            let ptr = &mut msg as *mut u32 as *mut ::core::ffi::c_void;
            let r = xQueueReceive(self.queue, ptr, 0);
            if r == PD_TRUE {
                return msg;
            }
        }
        0
    }
}
/*
 *
 */
pub struct BiQueue {
    queue: MonoQueue,
    me: Option<TaskHandle_t>,
}
// it is thread safe
unsafe impl Send for BiQueue {}
unsafe impl Sync for BiQueue {}

impl BiQueue {
    //
    pub fn new() -> Self {
        BiQueue {
            me: None,
            queue: MonoQueue::new(),
        }
    }
    //
    pub fn take_ownership(&mut self) {
        unsafe {
            self.me = Some(xTaskGetCurrentTaskHandle());
        }
    }
    //
    pub fn send_queued(&mut self, ev: u32) {
        self.queue.send(ev);
        self.send_unqueued(BIQUEUE_QUEUE_FLAG);
    }
    //
    pub fn send_unqueued(&mut self, ev: u32) {
        unsafe {
            if let Some(h) = self.me {
                notify(h, NonZeroU32::new_unchecked(ev));
            } else {
                panic!("BiQueue handle not initialized");
            }
        }
    }
    //
    fn clear_bits(&mut self, bits: u32) {
        unsafe {
            let mut bits_received: u32 = 0;
            esp_idf_sys::xTaskGenericNotifyWait(
                0,
                0,
                bits,
                &mut bits_received as *mut u32,
                0, // Wait time in ticks
            );
        }
    }
    fn wait_bits(&mut self) -> u32 {
        unsafe {
            let mut bits_received: u32 = 0;
            esp_idf_sys::xTaskGenericNotifyWait(
                0,
                0,
                0,
                &mut bits_received as *mut u32,
                100, // Wait time in ticks
            );
            bits_received
        }
    }
    //
    pub fn receive(&mut self) -> u32 {
        loop {
            // queue empty ?
            let msg = self.queue.receive();
            if msg != 0 {
                // clear the BIQUE FLAG
                self.clear_bits(BIQUEUE_QUEUE_FLAG);
                return msg;
            }
            let bits = self.wait_bits();
            if (bits & BIQUEUE_QUEUE_FLAG) != 0 {
                continue;
            }
            self.clear_bits(bits);
            return bits;
        }
    }
}
// EOF
