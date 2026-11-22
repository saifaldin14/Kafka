//! Leaking values instead of dropping them, and shapes that waste memory: a shared
//! buffer that can't release its spare capacity, and a future too large for the stack.

use std::rc::Rc;

/// Forgetting a value that owns heap memory.
pub fn mem_forget(value: String) {
    std::mem::forget(value); // expect-lint: clippy::mem_forget
}

/// Sharing a growable buffer through `Rc`, which keeps its spare capacity forever.
/// Private, because clippy doesn't lint the exported API (`avoid-breaking-exported-api`).
#[derive(Debug)]
struct RcBuffer {
    bytes: Rc<Vec<u8>>, // expect-lint: clippy::rc_buffer
}

/// Uses `RcBuffer`, so it isn't dead code.
pub fn rc_buffer(bytes: Vec<u8>) -> usize {
    RcBuffer {
        bytes: Rc::new(bytes),
    }
    .bytes
    .len()
}

/// A future holding 20 KB across an `.await`, in two arrays each small enough for the
/// stack.
pub async fn large_future() -> u8 {
    let first = [0_u8; 10_000];
    let second = [0_u8; 10_000];
    std::future::ready(()).await;
    first[0] ^ second[0]
}

/// Awaiting the large future in place instead of boxing it.
pub async fn large_futures() -> u8 {
    large_future().await // expect-lint: clippy::large_futures
}
