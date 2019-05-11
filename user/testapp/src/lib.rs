#![no_std]

use libt::*;
use core::str::from_utf8;

start!({
    let name = from_utf8(args()).unwrap_or(&"");
    for i in 0.. {
        println!("{} {}", name, i);
    }
});
