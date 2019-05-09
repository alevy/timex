#![no_std]

use libt::println;

#[no_mangle]
pub static mut WRITER: libt::Writer = libt::Writer::new();

#[no_mangle]
pub fn main() {
    for i in 0.. {
        println!("t1 {}", i);
    }
}
