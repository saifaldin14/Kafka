//! Holding a blocking lock or a `RefCell` borrow across an `.await`, which can deadlock
//! the runtime or panic when another task borrows the cell.

use std::cell::RefCell;
use std::sync::{Mutex, PoisonError};

/// A `std::sync::Mutex` guard held while awaiting. No runtime is needed: the lint
/// inspects the future's captured state.
pub async fn await_holding_lock(lock: &Mutex<u32>) -> u32 {
    let guard = lock.lock().unwrap_or_else(PoisonError::into_inner); // expect-lint: clippy::await_holding_lock
    std::future::ready(()).await;
    *guard
}

/// A `RefCell` borrow held while awaiting.
pub async fn await_holding_refcell_ref(cell: &RefCell<u32>) -> u32 {
    let borrowed = cell.borrow(); // expect-lint: clippy::await_holding_refcell_ref
    std::future::ready(()).await;
    *borrowed
}
