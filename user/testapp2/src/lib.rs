#![feature(lang_items)]
#![no_std]

use libt::*;

start!({
    for i in 0.. {
        println!("t2 {}", i);
    }
});
