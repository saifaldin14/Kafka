//! Ending the process without an orderly shutdown.

/// Exiting from library code.
pub fn exit() {
    // `std::process::exit` is also in clippy.toml's `disallowed-methods`.
    std::process::exit(1); // expect-lint: clippy::exit, clippy::disallowed_methods
}
