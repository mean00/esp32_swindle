unsafe extern "C" {
    unsafe fn ws2812_init(pin: u8);
    unsafe fn ws2812_set_color(color: u32);
}

fn dim(x: u32) -> u32 {
    x >> 3
}
/*
 *
 */
pub fn init(pin: u8) {
    unsafe {
        ws2812_init(pin);
    }
}
/*
 *
 *
 */
pub fn set_color(color: u32) {
    let mut r = (color >> 16) & 0xffu32;
    let mut g = (color >> 8) & 0xffu32;
    let mut b = (color) & 0xffu32;
    r = dim(r);
    g = dim(g);
    b = dim(b);
    unsafe {
        ws2812_set_color((r << 16) + (g << 8) + (b));
    }
}
// EOF
