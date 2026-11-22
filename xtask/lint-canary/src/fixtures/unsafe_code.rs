//! `unsafe` without an ADR.

/// Declaring an unsafe function.
///
/// # Safety
///
/// None needed: the body is empty. The declaration alone must be rejected.
pub unsafe fn unsafe_code() {} // expect-lint: unsafe_code

/// An unsafe block with no `// SAFETY:` comment.
pub fn undocumented_unsafe_blocks(value: &u32) -> u32 {
    unsafe { std::ptr::read(value) } // expect-lint: unsafe_code, clippy::undocumented_unsafe_blocks
}
