//! Reading OS entropy outside `SeededRng::from_os_entropy`
//! ([ADR-0012](../../../../docs/adr/0012-seeded-randomness-and-hashing.md)).

use std::mem::MaybeUninit;

/// Filling a buffer from the OS.
///
/// # Errors
///
/// If the OS can't provide entropy.
pub fn getrandom_fill(buffer: &mut [u8]) -> Result<(), getrandom::Error> {
    getrandom::fill(buffer) // expect-lint: clippy::disallowed_methods
}

/// Filling an uninitialised buffer from the OS.
///
/// # Errors
///
/// If the OS can't provide entropy.
pub fn getrandom_fill_uninit(
    buffer: &mut [MaybeUninit<u8>],
) -> Result<&mut [u8], getrandom::Error> {
    getrandom::fill_uninit(buffer) // expect-lint: clippy::disallowed_methods
}

/// A random `u32` from the OS.
///
/// # Errors
///
/// If the OS can't provide entropy.
pub fn getrandom_u32() -> Result<u32, getrandom::Error> {
    getrandom::u32() // expect-lint: clippy::disallowed_methods
}

/// A random `u64` from the OS.
///
/// # Errors
///
/// If the OS can't provide entropy.
pub fn getrandom_u64() -> Result<u64, getrandom::Error> {
    getrandom::u64() // expect-lint: clippy::disallowed_methods
}
