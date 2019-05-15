#![feature(lang_items)]
#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};

pub mod ucontext;

pub use ucontext::pause;

pub struct WaitFreeBuffer {
    pub wcur: usize,
    pub rcur: AtomicUsize,
    pub buf: [u8; 256],
}

impl WaitFreeBuffer {
    pub const fn new() -> Self {
        Self {
            buf: [0; 256],
            wcur: 0,
            rcur: AtomicUsize::new(0),
        }
    }

    pub fn read(&self, out: &mut [u8]) -> usize {
        let mut rcur = self.rcur.load(Ordering::Relaxed);
        let mut len = 0;
        for b in out.iter_mut() {
            if rcur == self.wcur {
                break;
            }
            rcur = (rcur + 1) % self.buf.len();
            *b = self.buf[rcur];
            len += 1;
        }

        self.rcur.store(rcur, Ordering::Relaxed);
        len
    }
}
