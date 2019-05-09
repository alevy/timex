fn main() {
    cc::Build::new()
        .file("src/ucontext.c")
        .compile("ucontext");   // outputs `libcontext.a`
}
