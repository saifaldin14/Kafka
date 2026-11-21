//! Reading the clock or sleeping directly. Time is injected through
//! `alluvium_types::Clock` so simulation controls it.

use std::future::{Ready, ready};
use std::time::{Duration, SystemTimeError};

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

/// Measuring elapsed time on the monotonic clock directly.
pub fn std_instant_elapsed(start: std::time::Instant) -> Duration {
    start.elapsed() // expect-lint: clippy::disallowed_methods
}

/// Measuring elapsed time on the wall clock directly.
///
/// # Errors
///
/// If `start` is in the future.
pub fn std_system_time_elapsed(start: std::time::SystemTime) -> Result<Duration, SystemTimeError> {
    start.elapsed() // expect-lint: clippy::disallowed_methods
}

/// Waiting for a deadline on tokio's timer.
pub fn tokio_sleep_until(deadline: tokio::time::Instant) -> tokio::time::Sleep {
    tokio::time::sleep_until(deadline) // expect-lint: clippy::disallowed_methods
}

/// Bounding a future with tokio's timer.
pub fn tokio_timeout() -> tokio::time::Timeout<Ready<()>> {
    tokio::time::timeout(Duration::ZERO, ready(())) // expect-lint: clippy::disallowed_methods
}

/// Bounding a future by a deadline on tokio's timer.
pub fn tokio_timeout_at(deadline: tokio::time::Instant) -> tokio::time::Timeout<Ready<()>> {
    tokio::time::timeout_at(deadline, ready(())) // expect-lint: clippy::disallowed_methods
}

/// A periodic tick on tokio's timer.
pub fn tokio_interval() -> tokio::time::Interval {
    tokio::time::interval(Duration::from_millis(1)) // expect-lint: clippy::disallowed_methods
}

/// A periodic tick from a start time on tokio's timer.
pub fn tokio_interval_at(start: tokio::time::Instant) -> tokio::time::Interval {
    tokio::time::interval_at(start, Duration::from_millis(1)) // expect-lint: clippy::disallowed_methods
}
