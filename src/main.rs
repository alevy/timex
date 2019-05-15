use libttypes::ucontext::*;
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::{Duration, Instant};

pub struct Process {
    ctx: Ctx,
    _stack: Option<Box<[u8]>>,
}

impl Process {
    pub fn new(
        func: *const extern "C" fn(*const usize, usize, *const u8),
        mut stack: Box<[u8]>,
        cmd: &[u8],
        link_ctx: Ctx,
    ) -> Self {
        for (s, d) in cmd.iter().zip(stack.iter_mut()) {
            *d = *s;
        }
        let ctx = unsafe {
            let t = func;
            ucontext_new(
                t,
                stack.as_ptr(),
                stack.len(),
                link_ctx,
                cmd.len(),
                stack.as_ptr(),
            )
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

extern "C" {
    fn setitimer(
        which: libc::c_int,
        new_value: &ITimerval,
        old_value: *const ITimerval,
    ) -> libc::c_int;
    fn write(fd: usize, s: *const u8, len: usize);
}

static mut CUR_PROCESS: Option<Process> = None;
static mut MAIN_CTX: Ctx = 0 as Ctx;

extern "C" fn interrupt(_: u32) {
    unsafe {
        if let Some(c) = CUR_PROCESS.as_ref() {
            swapcontext(c.ctx, MAIN_CTX);
        }
    }
}

fn main() -> libloading::Result<()> {
    use libloading as lib;
    use std::env;

    let args: Vec<String> = env::args().collect();

    let (console_tx, console_rx): (Sender<&'static libttypes::WaitFreeBuffer>, _) = channel();

    let mc = unsafe {
        MAIN_CTX = ucontext_alloc();
        MAIN_CTX
    };

    let mut run_queue: VecDeque<Process> = VecDeque::new();
    for (i, app_path) in args[1..].iter().enumerate() {
        let test1 = lib::Library::new(app_path)?;
        let test1_init = unsafe { test1.get(b"_start")? };
        let test1_console = unsafe {
            test1
                .get(b"CONSOLE_BUF")
                .map(|t| *t)
                .expect("No console buf")
        };
        console_tx
            .send(test1_console)
            .expect("Couldn't send console to I/O processor");
        run_queue.push_back(Process::new(
            *test1_init,
            Box::new([0; 100 * 1024]),
            format!("Test{}", i).as_bytes(),
            mc,
        ));
        std::mem::forget(test1);
    }

    thread::spawn(move || {
        for console_buf in console_rx.iter() {
            // drain console buffer
            let mut buf = [0; 256];
            let start = Instant::now();
            let len = console_buf.read(&mut buf);
            unsafe {
                write(1, buf.as_ptr(), len);
            }
            // TODO(alevy): how long is long enough that the buffer will always be written, but not
            // _super_ duper long.
            let exec_duration = start.elapsed();
            let sleep_duration = Duration::from_micros(1000) - exec_duration;
            thread::sleep(sleep_duration);
            console_tx.send(console_buf).unwrap();
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
        setitimer(0 /* ITIMER_PROF */, &tenms, std::ptr::null());
        libc::signal(libc::SIGALRM, interrupt as libc::sighandler_t);
    }

    loop {
        if let Some(ctx) = run_queue.pop_front() {
            let uctx = ctx.ctx;
            unsafe {
                CUR_PROCESS.replace(ctx);
                swapcontext(mc, uctx);
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
