use core::ffi::{c_int, c_void};
//use std::slice;
//use std::str;
//

#[unsafe(no_mangle)]
pub extern "C" fn usbCdc_Logger(_n: i32, _data: *const u8) {}

#[unsafe(no_mangle)]
pub extern "C" fn x__wrap_longjmp() {}

// Declare the "real" longjmp, just in case we need to pass through
unsafe extern "C" {
    unsafe fn __real_longjmp(env: *mut c_void, val: c_int) -> !;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wrap_longjmp(env: *mut c_void, val: c_int) -> ! {
    unsafe { __real_longjmp(env, val) }
}

#[unsafe(no_mangle)]
pub extern "C" fn gdb_if_init() {}

unsafe extern "C" {
    static __init_array_start: usize;
    static __init_array_end: usize;
}
#[unsafe(no_mangle)]
pub fn init_global_ctors() {
    unsafe {
        let mut ptr = &__init_array_start as *const _ as *const extern "C" fn();
        let end = &__init_array_end as *const _ as *const extern "C" fn();

        while ptr < end {
            let constructor = *ptr;
            constructor();
            ptr = ptr.add(1);
        }
    }
}
// EOF
