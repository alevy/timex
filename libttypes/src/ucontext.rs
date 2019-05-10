use libc::c_int;

pub type Ctx = *const usize;

extern {
    pub fn ucontext_alloc() -> *const usize;
    pub fn ucontext_new(start: *const extern fn(Ctx, usize, *const u8), stack_ptr: *const u8, stack_len: usize, link: *const usize, argc: usize, argv: *const u8) -> *const usize;
    pub fn ucontext_free(ctx: *const usize);
    pub fn swapcontext(ouc: *const usize, uc: *const usize) -> c_int;
    pub fn setcontext(uc: *const usize) -> c_int;
}


