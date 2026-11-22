//! Reading the time ([ADR-0011](../../../docs/adr/0011-time-model-and-clock-injection.md)).
//!
//! [`Clock`] is passed explicitly as `Arc<dyn Clock>`; there is no global clock. It only
//! reads the time: waiting is the separate `Timer` seam, so code that holds a clock
//! still can't block on it.
//!
//! [`SystemClock`] is the only code in the workspace, apart from `xtask`'s `Stopwatch`,
//! that reads the operating system's clocks. Layer 3–5 code constructs it; layer 0–2
//! code never does, and receives the time in its inputs instead.

use std::fmt;
use std::time::{Duration, Instant, SystemTime};

use crate::{MonotonicInstant, WallTime};

/// A source of the current time.
pub trait Clock: Send + Sync + fmt::Debug {
    /// The current monotonic reading. Never less than an earlier reading of the same
    /// clock.
    fn monotonic(&self) -> MonotonicInstant;

    /// The current wall-clock time. It can jump backwards or forwards between readings.
    ///
    /// # Errors
    ///
    /// [`WallClockError`] if the clock is set outside what [`WallTime`] can hold.
    fn wall(&self) -> Result<WallTime, WallClockError>;
}

/// Why the wall clock couldn't be read as a [`WallTime`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum WallClockError {
    /// The system clock is set before 1970-01-01T00:00:00Z.
    #[error("the system clock is {before:?} before the Unix epoch")]
    BeforeEpoch {
        /// How far before the epoch.
        before: Duration,
    },
    /// The system clock is set further after the epoch than `i64` milliseconds reach.
    #[error("the system clock is {since:?} after the Unix epoch, beyond the range of wall time")]
    OutOfRange {
        /// How far after the epoch.
        since: Duration,
    },
}

/// The operating system's clocks.
///
/// Monotonic readings count from when the clock was started, so they are small numbers
/// that mean nothing outside this process.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock {
    origin: Instant,
}

impl SystemClock {
    /// A clock whose monotonic origin is now.
    pub fn start() -> SystemClock {
        SystemClock {
            origin: os_monotonic_now(),
        }
    }

    /// The [`Instant`] that a reading of this clock stands for, so that the production
    /// timer can wait until it. `None` if the platform's `Instant` can't represent it,
    /// which for a saturated "never" deadline means the wait never ends.
    pub fn to_std_instant(&self, instant: MonotonicInstant) -> Option<Instant> {
        self.origin
            .checked_add(Duration::from_nanos(instant.as_nanos()))
    }
}

// Parity: clients/src/main/java/org/apache/kafka/common/utils/SystemTime.java@4.4.0 SystemTime.nanoseconds, SystemTime.milliseconds
impl Clock for SystemClock {
    fn monotonic(&self) -> MonotonicInstant {
        let elapsed = os_monotonic_now().saturating_duration_since(self.origin);
        // `u64` nanoseconds cover 584 years of uptime, so saturating is unreachable in
        // practice and still never goes backwards.
        MonotonicInstant::from_nanos(u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX))
    }

    fn wall(&self) -> Result<WallTime, WallClockError> {
        wall_time_of(os_wall_now())
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "SystemClock is the one sanctioned reader of the OS monotonic clock (ADR-0011)"
)]
fn os_monotonic_now() -> Instant {
    Instant::now()
}

#[expect(
    clippy::disallowed_methods,
    reason = "SystemClock is the one sanctioned reader of the OS wall clock (ADR-0011)"
)]
fn os_wall_now() -> SystemTime {
    SystemTime::now()
}

/// Converts a system time to wall time, without panicking on any setting of the clock.
fn wall_time_of(time: SystemTime) -> Result<WallTime, WallClockError> {
    let since = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|error| WallClockError::BeforeEpoch {
            before: error.duration(),
        })?;
    WallTime::UNIX_EPOCH
        .checked_add(since)
        .map_err(|_| WallClockError::OutOfRange { since })
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use super::{Clock, SystemClock, WallClockError, wall_time_of};
    use crate::{MonotonicInstant, WallTime};

    #[test]
    fn a_clock_before_the_epoch_is_an_error_not_a_panic() {
        let before = SystemTime::UNIX_EPOCH - Duration::from_secs(1);
        assert_eq!(
            wall_time_of(before),
            Err(WallClockError::BeforeEpoch {
                before: Duration::from_secs(1)
            })
        );
    }

    #[test]
    fn wall_time_is_whole_milliseconds_since_the_epoch() {
        let time = SystemTime::UNIX_EPOCH + Duration::from_micros(1_500_999);
        assert_eq!(wall_time_of(time), Ok(WallTime::try_from(1_500).unwrap()));
        assert_eq!(
            wall_time_of(SystemTime::UNIX_EPOCH),
            Ok(WallTime::UNIX_EPOCH)
        );
    }

    #[test]
    fn a_clock_beyond_i64_milliseconds_is_an_error() {
        // 10^17 seconds is past i64::MAX milliseconds (about 9.2 * 10^15 seconds), and
        // still representable as a SystemTime on the platforms CI runs.
        let since = Duration::from_secs(100_000_000_000_000_000);
        let far = SystemTime::UNIX_EPOCH.checked_add(since).unwrap();
        assert_eq!(wall_time_of(far), Err(WallClockError::OutOfRange { since }));
    }

    #[test]
    fn system_clock_readings_map_back_to_std_instants() {
        let clock = SystemClock::start();
        let first = clock.monotonic();
        let second = clock.monotonic();
        assert!(second >= first);

        let origin = clock
            .to_std_instant(MonotonicInstant::from_nanos(0))
            .unwrap();
        let later = clock
            .to_std_instant(MonotonicInstant::from_nanos(5_000))
            .unwrap();
        assert_eq!(later - origin, Duration::from_micros(5));
        assert_eq!(origin, clock.origin);
    }

    #[test]
    fn system_wall_clock_reads_a_plausible_time() {
        // 2024-01-01T00:00:00Z: any machine running these tests is past it.
        let wall = SystemClock::start().wall().unwrap();
        assert!(wall.as_millis() >= 1_704_067_200_000, "{wall}");
    }

    #[test]
    fn clocks_are_shared_as_send_sync_trait_objects() {
        fn shareable<T: Send + Sync + Debug>(_: &T) {}
        let clock: Arc<dyn Clock> = Arc::new(SystemClock::start());
        shareable(&clock);
    }
}
