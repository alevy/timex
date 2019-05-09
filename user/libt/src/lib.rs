#![feature(lang_items)]
#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};
use core::fmt::{self, Write};

use libttypes::Writer;

#[no_mangle]
pub static mut WRITER: Writer = Writer::new();

pub fn _print(args: core::fmt::Arguments) {
    unsafe { &mut WRITER }.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::_print(format_args!($($arg)*))
    });
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
fn eh_personality() {}

