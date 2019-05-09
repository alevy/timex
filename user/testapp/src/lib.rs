#![no_std]

use libt::println;

#[no_mangle]
pub fn main() {
    for i in 0.. {
        println!("t1 {}", i);
    }
}
