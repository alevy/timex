use std::collections::VecDeque;
use libttypes::ucontext::*;

pub struct Process {
    ctx: Ctx,
    _stack: Option<Box<[u8]>>,
}

impl Process {
    pub fn new(func: *const extern fn(*const usize, usize, *const u8), mut stack: Box<[u8]>, cmd: &[u8], link_ctx: Ctx) -> Self {
        for (s, d) in cmd.iter().zip(stack.iter_mut()) {
            *d = *s;
        }
        let ctx = unsafe {
            let t = func;
            ucontext_new(t, stack.as_ptr(), stack.len(), link_ctx, cmd.len(), stack.as_ptr())
        };
        Process {
            ctx: ctx,
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

extern fn interrupt(_: u32) {
    unsafe {
        if let Some(c) = CUR_PROCESS.as_ref() {
            libttypes::ucontext::swapcontext(c.ctx, MAIN_CTX);
        }
    }
}

fn main() -> libloading::Result<()> {
    use libloading as lib;
    use std::env;
    use std::sync::mpsc::{channel, Sender};

    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let (console_tx, console_rx): (Sender<&'static libttypes::WaitFreeBuffer>, _) = channel();

    let mc = unsafe {
        MAIN_CTX = libttypes::ucontext::ucontext_alloc();
        MAIN_CTX
    };

    let mut run_queue: VecDeque<Process> = VecDeque::new();
    for (i, app_path) in args[1..].iter().enumerate() {
        let test1 = lib::Library::new(app_path)?;
        let test1_init = unsafe { test1.get(b"_start")? };
        let test1_console = unsafe { test1.get(b"CONSOLE_BUF").map(|t| *t).expect("No console buf") };
        console_tx.send(test1_console).expect("Couldn't send console to I/O processor");
        run_queue.push_back(Process::new(*test1_init, Box::new([0; 100 * 1024]), format!("Test{}", i).as_bytes(), mc));
        std::mem::forget(test1);
    }

    std::thread::spawn(move || {
        for console_buf in console_rx.iter() {
            // drain console buffer
            let mut buf = [0; 256];
            let len = console_buf.read(&mut buf);
            unsafe {
                write(1, buf.as_ptr(), len);
            }
            console_tx.send(console_buf).unwrap();
            // TODO(alevy): This is a (very bad) approximation of a pad. It's totally made up, but
            // at least it's based on the length of the buffer. It's probably an overestimate, but
            // who knows.
            std::thread::sleep(std::time::Duration::from_micros((256 - len) as u64));
        }
    });


    let tenms = ITimerval {
        it_interval: libc::timeval {
            tv_sec: 0,
            tv_usec: 10000,
        },
        it_value: libc::timeval {
            tv_sec: 0,
            tv_usec: 10000,
        },
    };
    unsafe {
        setitimer(0 /* ITIMER_PROF */, &tenms, ::std::ptr::null());
        libc::signal(libc::SIGALRM, interrupt as libc::sighandler_t);
    }

    loop {
        if let Some(ctx) = run_queue.pop_front() {
            let uctx = ctx.ctx;
            unsafe {
                CUR_PROCESS.replace(ctx);
                libttypes::ucontext::swapcontext(mc, uctx);
                if let Some(ctx) = CUR_PROCESS.take() {
                    run_queue.push_back(ctx);
                }
            }
        } else {
            break;
        }
    }

    println!("Done");
    Ok(())
}

