#![feature(lang_items)]
#![no_std]

use core::panic::PanicInfo;
use libt::println;

#[no_mangle]
pub static mut WRITER: libt::Writer = libt::Writer::new();

#[no_mangle]
pub fn main() {
    for i in 0.. {
        let _ = println!("t2 {}", i);
    }
}
