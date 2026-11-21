//! Panicking shortcuts that are banned outright: input must produce errors, and
//! unfinished code must not compile.

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
