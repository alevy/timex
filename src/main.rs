use std::collections::VecDeque;
use libc::c_int;

pub struct UContext {
    ctx: *const usize,
    console_buf: Option<&'static libt::Writer>,
    _stack: Option<Box<[u8]>>,
}

extern {
    fn ucontext_alloc() -> *const usize;
    fn ucontext_new(trampoline: *const extern fn(), stack_ptr: *const u8, stack_len: usize, func: *const usize, link: *const usize) -> *const usize;
    fn ucontext_free(ctx: *const usize);
    fn swapcontext(ouc: *const usize, uc: *const usize) -> c_int;
    fn setcontext(uc: *const usize) -> c_int;
}

impl UContext {
    pub fn alloc() -> Self {
        UContext {
            ctx: unsafe { ucontext_alloc() },
            console_buf: None,
            _stack: None,
        }
    }

    pub fn new(func: fn(), stack: Box<[u8]>, console_buf: Option<&'static libt::Writer>, link: Option<&UContext>) -> Self {
        extern fn trampoline(f: fn()) {
            f()
        }

        let link_ctx = link.map(|l| l.ctx).unwrap_or(::std::ptr::null());
        let ctx = unsafe {
            let t = trampoline as *const extern fn();
            ucontext_new(t, stack.as_ptr(), stack.len(), func as *const usize, link_ctx)
        };
        UContext {
            ctx: ctx,
            console_buf: console_buf,
            _stack: Some(stack),
        }
    }

    pub fn swap_with(&self, to: *const usize) {
        unsafe {
            if swapcontext(self.ctx, to) < 0 {
                panic!("Error swapping context");
            }
        }
    }

    pub fn swap_to(&self) {
        unsafe {
            if setcontext(self.ctx) < 0 {
                panic!("Error swapping context");
            }
        }
    }
}

impl Drop for UContext {
    fn drop(&mut self) {
        unsafe { ucontext_free(self.ctx) };
    }
}

#[repr(C)]
struct ITimerval {
    it_interval: libc::timeval, /* Interval for periodic timer */
    it_value: libc::timeval,    /* Time until next expiration */
}

extern {
    fn setitimer(which: libc::c_int, new_value: &ITimerval, old_value: *const ITimerval) -> libc::c_int;
    fn write(fd: usize, s: *const u8, len: usize);
}

static mut CUR_CTX: Option<UContext> = None;
static mut MAIN_CTX: UContext = UContext { ctx: 0 as *const _, console_buf: None, _stack: None };

extern fn interrupt(_:u32) {
    //println("Interrupted!");
    unsafe {
        if let Some(c) = CUR_CTX.as_ref() {
            c.swap_with(MAIN_CTX.ctx);
        } else {
            MAIN_CTX.swap_to();
        }
    }
}

fn main() -> libloading::Result<()> {
    use libloading as lib;
    use std::env;

    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let mc = unsafe {
        MAIN_CTX = UContext::alloc();
        &MAIN_CTX
    };

    let mut run_queue: VecDeque<UContext> = VecDeque::new();
    for app_path in args[1..].iter() {
        let test1 = lib::Library::new(app_path)?;
        let test1_init = unsafe { test1.get(b"main")? };
        let test1_console = unsafe { test1.get(b"WRITER").map(|t| *t).ok() };
        run_queue.push_back(UContext::new(*test1_init, Box::new([0; 100 * 1024]), test1_console, Some(&mc)));
        std::mem::forget(test1);
    }


    unsafe {
        libc::signal(libc::SIGPROF, interrupt as libc::sighandler_t);
        let interval = ITimerval {
            it_interval: libc::timeval {
                tv_sec: 0,
                tv_usec: 10000,
            },
            it_value: libc::timeval {
                tv_sec: 0,
                tv_usec: 10000,
            },
        };
        setitimer(2 /* ITIMER_PROF */, &interval, ::std::ptr::null());
    }

    loop {
        if let Some(ctx) = run_queue.pop_front() {
            let uctx = ctx.ctx;
            unsafe { CUR_CTX.replace(ctx); }
            mc.swap_with(uctx);
            unsafe { CUR_CTX.take() }.map(|mut c| {
                // drain console buffer
                c.console_buf.as_mut().map(|writer| {
                    let mut buf = [0; 256];
                    let len = writer.read(&mut buf);
                    unsafe {
                        write(1, buf.as_ptr(), len);
                    }
                });
                run_queue.push_back(c)
            });
        } else {
            break;
        }
    }

    println!("Done");
    Ok(())
}

