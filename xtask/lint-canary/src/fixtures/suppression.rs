//! Lint suppression: only `#[expect]` with a reason is allowed, so every exception
//! explains itself and fails the build once it is no longer needed.

/// A bare `#[allow]`.
#[allow(dead_code)] // expect-lint: clippy::allow_attributes, clippy::allow_attributes_without_reason
pub fn bare_allow() {}

/// `#[expect]` without a reason. The expectation itself is met.
pub fn expect_without_reason() {
    #[expect(dead_code)] // expect-lint: clippy::allow_attributes_without_reason
    fn unused() {}
}

/// The sanctioned wrapper: the one place that may call a disallowed method, with
/// a single `#[expect]` naming why. Produces no diagnostics.
#[expect(
    clippy::disallowed_methods,
    reason = "the one sanctioned wrapper for the monotonic clock"
)]
pub fn sanctioned_wrapper() -> std::time::Instant {
    std::time::Instant::now()
}

/// The sanctioned invariant check: `expect("invariant: …")` under an `#[expect]`
/// with a reason. Produces no diagnostics.
///
/// # Panics
///
/// If `value` is `None`, which the caller guarantees never happens.
#[expect(
    clippy::expect_used,
    reason = "invariant: the caller always passes a value"
)]
pub fn sanctioned_invariant(value: Option<u32>) -> u32 {
    value.expect("invariant: the caller always passes a value")
}
