#![feature(start, lang_items)]
#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};
use core::fmt::Write;

use libttypes::Writer;

pub use libttypes::ucontext::Ctx;

#[export_name = "WRITER"]
pub static mut WRITER: Writer = Writer::new();

pub fn _print(args: core::fmt::Arguments) {
    unsafe { &mut WRITER }.write_fmt(args).unwrap();
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

pub fn sleep(mut ms: usize) -> bool{
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
    unsafe {
        ARGS
    }
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
    ($f:expr) => (
        #[export_name = "_start"]
        pub fn _start(parent: Ctx, ctx: Ctx, argc: usize, argv: *const u8) {
            use $crate::Ctx;
            unsafe {
                $crate::crt0(parent, ctx, argc, argv);
            }
            $f
        }
    )
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
fn eh_personality() {}

