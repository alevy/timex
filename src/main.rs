use std::collections::VecDeque;
use libttypes::ucontext::*;

pub struct Process {
    ctx: Ctx,
    console_buf: Option<&'static libttypes::Writer>,
    _stack: Option<Box<[u8]>>,
}

impl Process {
    pub fn alloc() -> Self {
        Process {
            ctx: unsafe { ucontext_alloc() },
            console_buf: None,
            _stack: None,
        }
    }

    pub fn new(func: *const extern fn(*const usize, usize, *const u8), mut stack: Box<[u8]>, cmd: &[u8], console_buf: Option<&'static libttypes::Writer>, link_ctx: Ctx) -> Self {
        for (s, d) in cmd.iter().zip(stack.iter_mut()) {
            *d = *s;
        }
        let ctx = unsafe {
            let t = func;
            ucontext_new(t, stack.as_ptr(), stack.len(), link_ctx, cmd.len(), stack.as_ptr())
        };
        Process {
            ctx: ctx,
            console_buf: console_buf,
            _stack: Some(stack),
        }
    }
}

impl Drop for Process {
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

static mut CUR_PROCESS: Option<Process> = None;
static mut MAIN_CTX: Ctx = 0 as Ctx;

extern fn interrupt(_:u32) {
    unsafe {
        if let Some(c) = CUR_PROCESS.as_ref() {
            libttypes::ucontext::swapcontext(c.ctx, MAIN_CTX);
        } else {
            libttypes::ucontext::setcontext(MAIN_CTX);
        }
    }
}

fn main() -> libloading::Result<()> {
    use libloading as lib;
    use std::env;

    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let mc = unsafe {
        MAIN_CTX = libttypes::ucontext::ucontext_alloc();
        MAIN_CTX
    };

    let mut run_queue: VecDeque<Process> = VecDeque::new();
    for (i, app_path) in args[1..].iter().enumerate() {
        let test1 = lib::Library::new(app_path)?;
        let test1_init = unsafe { test1.get(b"_start")? };
        let test1_console = unsafe { test1.get(b"WRITER").map(|t| *t).ok() };
        run_queue.push_back(Process::new(*test1_init, Box::new([0; 100 * 1024]), format!("Test{}", i).as_bytes(), test1_console, mc));
        std::mem::forget(test1);
    }


    unsafe {
        libc::signal(libc::SIGPROF, interrupt as libc::sighandler_t);
    }
    let mut interval = ITimerval {
        it_interval: libc::timeval {
            tv_sec: 0,
            tv_usec: 10000,
        },
        it_value: libc::timeval {
            tv_sec: 0,
            tv_usec: 10000,
        },
    };
    let null_time = ITimerval {
        it_interval: libc::timeval {
            tv_sec: 0,
            tv_usec: 00000,
        },
        it_value: libc::timeval {
            tv_sec: 0,
            tv_usec: 00000,
        },
    };

    loop {
        if let Some(ctx) = run_queue.pop_front() {
            let uctx = ctx.ctx;
            unsafe {
                CUR_PROCESS.replace(ctx);
                setitimer(2 /* ITIMER_PROF */, &interval, ::std::ptr::null());
                libttypes::ucontext::swapcontext(mc, uctx);
                setitimer(2 /* ITIMER_PROF */, &null_time, &mut interval);
                CUR_PROCESS.take()
            }.map(|mut c| {
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

