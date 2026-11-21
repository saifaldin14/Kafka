//! Reading the clock or sleeping directly. Time is injected through
//! `alluvium_types::Clock` so simulation controls it.

use std::time::Duration;

/// Reading the monotonic clock.
pub fn std_instant_now() -> std::time::Instant {
    std::time::Instant::now() // expect-lint: clippy::disallowed_methods
}

/// Reading the wall clock.
pub fn std_system_time_now() -> std::time::SystemTime {
    std::time::SystemTime::now() // expect-lint: clippy::disallowed_methods
}

/// Blocking a thread to wait.
pub fn std_thread_sleep() {
    std::thread::sleep(Duration::ZERO); // expect-lint: clippy::disallowed_methods
}

/// Waiting on tokio's timer.
pub fn tokio_sleep() -> tokio::time::Sleep {
    tokio::time::sleep(Duration::ZERO) // expect-lint: clippy::disallowed_methods
}

/// Reading tokio's clock.
pub fn tokio_instant_now() -> tokio::time::Instant {
    tokio::time::Instant::now() // expect-lint: clippy::disallowed_methods
}
