#![feature(lang_items)]
#![no_std]

use core::panic::PanicInfo;
use libt::Writer;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
fn eh_personality() {}

#[no_mangle]
pub static mut WRITER: Writer = Writer::new();

#[no_mangle]
pub fn main() {
    use core::fmt::Write;
    for i in 0.. {
        let _ = write!(unsafe { &mut WRITER }, "barbaz {}\n", i);
    }
}
