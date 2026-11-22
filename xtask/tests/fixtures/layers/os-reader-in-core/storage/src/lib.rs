//! Fixture crate for `cargo xtask check-layers` tests. Docs may name `SystemClock`.

/// Receives the time from its driver instead of a `SystemClock` (a comment, accepted).
pub fn append(now_ms: i64) -> i64 {
    now_ms // never SeededRng::from_os_entropy
}

mod driver;
