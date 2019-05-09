#![feature(lang_items)]
#![no_std]

use libt::println;

#[no_mangle]
pub fn main() {
    for i in 0.. {
        let _ = println!("t2 {}", i);
    }
}
