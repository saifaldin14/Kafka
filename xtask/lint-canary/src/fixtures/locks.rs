//! Holding a blocking lock across an `.await`, which can deadlock the runtime.

use std::sync::{Mutex, PoisonError};

/// A `std::sync::Mutex` guard held while awaiting. No runtime is needed: the lint
/// inspects the future's captured state.
pub async fn await_holding_lock(lock: &Mutex<u32>) -> u32 {
    let guard = lock.lock().unwrap_or_else(PoisonError::into_inner); // expect-lint: clippy::await_holding_lock
    std::future::ready(()).await;
    *guard
}
