fn main() {
    cc::Build::new()
        .file("src/ucontext.c")
        .compile("hello");   // outputs `libhello.a`
}
