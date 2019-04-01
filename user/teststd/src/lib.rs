#![feature(lang_items)]

use core::panic::PanicInfo;

extern {
    fn println(s: &str);
}

#[no_mangle]
pub fn main() {
    for i in 0.. {
        unsafe { println(format!("foobar {}\n", i).as_str()); }
    }
}
