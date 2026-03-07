use crate::stubs_to_do::init_global_ctors;

unsafe extern "C" {
 //   fn pins_init();
    fn gdb_if_init();
    fn platform_init();
    fn _Z10LoggerInitv();
    fn _Z12lnEspSysInitv();
    //static mut swd_delay_cnt: u32;

}
pub fn swindle_sys_init() {
    init_global_ctors();
    // Init esprit runtime
    unsafe {
        _Z12lnEspSysInitv();
        platform_init();
  //      pins_init();
        gdb_if_init();
    }
}
