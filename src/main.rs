use std::collections::VecDeque;
use std::cell::RefCell;
use libc::c_int;

pub struct UContext {
    ctx: *const usize,
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
            _stack: None,
        }
    }

    pub fn new(func: fn(), stack: Box<[u8]>, link: Option<&UContext>) -> Self {
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
            _stack: Some(stack),
        }
    }

    pub fn swap_with(&self, to: &UContext) {
        unsafe {
            if swapcontext(self.ctx, to.ctx) < 0 {
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
        //unsafe { ucontext_free(self.ctx) };
    }
}

fn f1() {
    for i in 0.. {
        let s = format!("f1 {}!\n", i);
        println(s.as_str());
    }
}

fn f2() {
    for i in 0.. {
        let s = format!("f2 {}!\n", i);
        println(s.as_str());
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

fn println(s: &str) {
    unsafe {
        write(0, s.as_ptr(), s.len())
    }
}

thread_local!(
    static CTXS: RefCell<VecDeque<UContext>> = RefCell::new(VecDeque::new());
    static CUR_CTX: RefCell<Option<UContext>> = RefCell::new(None);
);

static mut MAIN_CTX: UContext = UContext { ctx: 0 as *const _, _stack: None };

extern fn interrupt(_:u32) {
    //println("Interrupted!");
    unsafe {
        let ctx = CUR_CTX.with(|ctx| {
            ctx.borrow().as_ref().map(|ctx| UContext { ctx: ctx.ctx, _stack: None })
        });
        if let Some(c) = ctx {
            c.swap_with(&MAIN_CTX);
        } else {
            MAIN_CTX.swap_to();
        }
    }
}

fn main() {
    println("Hello, world!");

    unsafe {
        libc::signal(libc::SIGPROF, interrupt as libc::sighandler_t);
        let interval = ITimerval {
            it_interval: libc::timeval {
                tv_sec: 0,
                tv_usec: 1000,
            },
            it_value: libc::timeval {
                tv_sec: 0,
                tv_usec: 1000,
            },
        };
        setitimer(2 /* ITIMER_PROF */, &interval, ::std::ptr::null());
    }

    let mc = unsafe {
        MAIN_CTX = UContext::alloc();
        &MAIN_CTX
    };

    CTXS.with(|ctxs| {
        let mut c = ctxs.borrow_mut();
        c.push_back(UContext::new(f1, Box::new([0; 100 * 1024]), Some(&mc)));
        c.push_back(UContext::new(f2, Box::new([0; 100 * 1024]), Some(&mc)));
    });

    for _ in 0.. {
        if let Some(ctx) = CTXS.with(|ctxs| ctxs.borrow_mut().pop_front()) {
            let cop = CUR_CTX.with(|cur| {
                let cop = UContext { ctx: ctx.ctx, _stack: None };
                cur.borrow_mut().replace(ctx);
                cop
            });
            CUR_CTX.with(|cur| {
                mc.swap_with(&cop);
                CTXS.with(|ctxs| {
                    if let Some(c) = cur.borrow_mut().take() {
                        ctxs.borrow_mut().push_back(c);
                    }
                })
            });
        } else {
            break;
        }
    }

    println("Done");
}

