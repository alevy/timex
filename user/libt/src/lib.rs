#![feature(start, lang_items)]
#![no_std]

use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, Ordering};

use libttypes::{pause, WaitFreeBuffer};

pub use libttypes::ucontext::Ctx;

#[export_name = "CONSOLE_BUF"]
pub static mut CONSOLE_BUF: WaitFreeBuffer = WaitFreeBuffer::new();

struct Writer<'a>(&'a mut WaitFreeBuffer);

impl<'a> Write for Writer<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for byte in s.as_bytes().iter() {
            let wcur = (self.0.wcur + 1) % self.0.buf.len();
            while wcur == self.0.rcur.load(Ordering::Relaxed) {
                unsafe {
                    pause();
                }
            }
            self.0.buf[wcur] = *byte;
            self.0.wcur = wcur;
        }
        Ok(())
    }
}

pub fn _print(args: core::fmt::Arguments) {
    Writer(unsafe { &mut CONSOLE_BUF }).write_fmt(args).unwrap();
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

pub fn sleep(mut ms: usize) -> bool {
    let dummy = AtomicBool::new(false);
    while ms > 0 {
        ms -= 1;
        for _ in 0..1000000 {
            dummy.store(true, Ordering::Relaxed);
        }
    }
    dummy.load(Ordering::Relaxed)
}

static mut ARGS: &'static [u8] = &[];

static mut CTX: Ctx = 0 as Ctx;
static mut PARENT: Ctx = 0 as Ctx;

pub fn args() -> &'static [u8] {
    unsafe { ARGS }
}

pub fn wait() {
    unsafe {
        if libttypes::ucontext::swapcontext(CTX, PARENT) < 0 {
            panic!("Error swapping context");
        }
    }
}

pub unsafe fn crt0(parent: Ctx, ctx: Ctx, argc: usize, argv: *const u8) {
    ARGS = core::slice::from_raw_parts(argv, argc);
    CTX = ctx;
    PARENT = parent;
}

#[macro_export]
macro_rules! start {
    ($f:expr) => {
        #[export_name = "_start"]
        pub fn _start(parent: Ctx, ctx: Ctx, argc: usize, argv: *const u8) {
            use $crate::Ctx;
            unsafe {
                $crate::crt0(parent, ctx, argc, argv);
            }
            $f
        }
    };
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
fn eh_personality() {}
