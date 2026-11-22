//! Panicking shortcuts: input must produce errors, an invariant check takes the
//! sanctioned `expect("invariant: …")` form (see `suppression`), and unfinished code
//! must not compile.

/// `unwrap` on a value that may come from input.
///
/// # Panics
///
/// If `value` is `None`.
pub fn unwrap_used(value: Option<u32>) -> u32 {
    value.unwrap() // expect-lint: clippy::unwrap_used
}

/// An unfinished function.
pub fn todo() -> u32 {
    todo!() // expect-lint: clippy::todo
}

/// A function that will never be finished.
pub fn unimplemented() -> u32 {
    unimplemented!() // expect-lint: clippy::unimplemented
}

/// `panic!` where an error belongs.
///
/// # Panics
///
/// Always.
pub fn panic() -> u32 {
    panic!("this should have been an error") // expect-lint: clippy::panic
}

/// `expect` without the sanctioned invariant form.
///
/// # Panics
///
/// If `value` is `None`.
pub fn expect_used(value: Option<u32>) -> u32 {
    value.expect("a value") // expect-lint: clippy::expect_used
}
