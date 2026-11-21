//! Leaking values instead of dropping them.

/// Forgetting a value that owns heap memory.
pub fn mem_forget(value: String) {
    std::mem::forget(value); // expect-lint: clippy::mem_forget
}
