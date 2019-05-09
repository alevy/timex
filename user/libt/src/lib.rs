#![feature(lang_items)]
#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};
use core::fmt::{self, Write};

pub struct Writer {
    wcur: usize,
    rcur: AtomicUsize,
    buf: [u8; 256]
}

impl Writer {
    pub const fn new() -> Writer {
        Writer {
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

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for byte in s.as_bytes().iter() {
            let wcur = (self.wcur + 1) % self.buf.len();
            while wcur == self.rcur.load(Ordering::Relaxed) { }
            self.buf[wcur] = *byte;
            self.wcur = wcur;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        write!(unsafe {&mut WRITER }, $($arg)*).unwrap()
    });
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

#[cfg(feature = "nostd")]
mod base {
    use core::panic::PanicInfo;
    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        loop {}
    }

    #[lang = "eh_personality"]
    fn eh_personality() {}
}

