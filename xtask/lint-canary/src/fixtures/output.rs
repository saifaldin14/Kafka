//! Writing to the terminal directly instead of through `tracing`.

/// Debug printing left in.
pub fn dbg_macro(value: u32) -> u32 {
    dbg!(value) // expect-lint: clippy::dbg_macro
}

/// Printing to stdout.
pub fn print_stdout() {
    println!("canary"); // expect-lint: clippy::print_stdout
}

/// Printing to stderr.
pub fn print_stderr() {
    eprintln!("canary"); // expect-lint: clippy::print_stderr
}
