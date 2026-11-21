//! `unsafe` without an ADR.

/// Declaring an unsafe function.
///
/// # Safety
///
/// None needed: the body is empty. The declaration alone must be rejected.
pub unsafe fn unsafe_code() {} // expect-lint: unsafe_code
