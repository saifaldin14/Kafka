//! Reading the time ([ADR-0011](../../../docs/adr/0011-time-model-and-clock-injection.md)).
//!
//! [`Clock`] is passed explicitly as `Arc<dyn Clock>`; there is no global clock. It only
//! reads the time: waiting is the separate `Timer` seam, so code that holds a clock
//! still can't block on it.
//!
//! [`SystemClock`] is the only code in the workspace, apart from `xtask`'s `Stopwatch`,
//! that reads the operating system's clocks. Layer 3–5 code constructs it; layer 0–2
//! code never does, and receives the time in its inputs instead. [`ManualClock`] is the
//! clock for tests and simulation.

use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant, SystemTime};

use crate::{MonotonicInstant, TimeError, WallTime};

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

/// A clock that moves only when told to, for tests and simulation.
///
/// Clones share one clock, so a test can keep a handle and advance the clock that the
/// code under test reads through `Arc<dyn Clock>`.
///
/// Not gated behind `cfg(test)` or a feature: the deterministic simulation in
/// `alluvium-testkit` builds its clock on it, and other crates' tests use it directly.
/// It reads no real clock, so it is as safe in product code as any other value.
#[derive(Debug, Clone)]
pub struct ManualClock {
    readings: Arc<Mutex<Readings>>,
}

#[derive(Debug)]
struct Readings {
    monotonic: MonotonicInstant,
    wall: WallTime,
    /// Nanoseconds past `wall`, always below one millisecond, so that many advances
    /// shorter than a millisecond add up as they would on a real clock.
    wall_sub_milli_nanos: u32,
}

impl ManualClock {
    /// A clock that reads `monotonic` and `wall` until it is moved.
    pub fn new(monotonic: MonotonicInstant, wall: WallTime) -> ManualClock {
        ManualClock {
            readings: Arc::new(Mutex::new(Readings {
                monotonic,
                wall,
                wall_sub_milli_nanos: 0,
            })),
        }
    }

    /// Moves both readings forward by `by`, as time passing does.
    ///
    /// # Errors
    ///
    /// [`TimeError`] if either reading would overflow. The clock is then unchanged.
    pub fn advance(&self, by: Duration) -> Result<(), TimeError> {
        let mut readings = self.lock();
        let monotonic = readings.monotonic.checked_add(by)?;
        let overflow = TimeError::WallOverflow {
            millis: readings.wall.as_millis(),
            duration: by,
        };
        let since_wall = by
            .checked_add(Duration::from_nanos(u64::from(
                readings.wall_sub_milli_nanos,
            )))
            .ok_or(overflow)?;
        // `checked_add` keeps the whole milliseconds; the rest carries to the next advance.
        let wall = readings
            .wall
            .checked_add(since_wall)
            .map_err(|_| overflow)?;
        readings.monotonic = monotonic;
        readings.wall = wall;
        readings.wall_sub_milli_nanos = since_wall.subsec_nanos() % 1_000_000;
        Ok(())
    }

    /// Sets the wall clock, forwards or backwards, without touching monotonic time: what
    /// an NTP step or a VM migration does to a real clock.
    pub fn set_wall(&self, wall: WallTime) {
        let mut readings = self.lock();
        readings.wall = wall;
        readings.wall_sub_milli_nanos = 0;
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Readings> {
        // A panic while the lock is held can't leave the readings half-updated: `advance`
        // and `set_wall` compute every new value before they assign any, and assignment
        // doesn't panic. So a poisoned lock still holds a consistent state.
        self.readings.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Clock for ManualClock {
    fn monotonic(&self) -> MonotonicInstant {
        self.lock().monotonic
    }

    fn wall(&self) -> Result<WallTime, WallClockError> {
        Ok(self.lock().wall)
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

    use proptest::prelude::{Strategy, any, prop_assert, prop_assert_eq, prop_oneof, proptest};

    use super::{Clock, ManualClock, SystemClock, WallClockError, wall_time_of};
    use crate::strategies::{monotonic_instant, wall_time};
    use crate::{MonotonicInstant, TimeError, WallTime};

    fn wall(millis: i64) -> WallTime {
        WallTime::try_from(millis).unwrap()
    }

    #[derive(Debug, Clone)]
    enum Step {
        Advance(Duration),
        SetWall(WallTime),
    }

    fn step() -> impl Strategy<Value = Step> {
        prop_oneof![
            // Mostly short steps, some sub-millisecond, and some large enough to overflow.
            3 => (0..10_000_000_u64).prop_map(|nanos| Step::Advance(Duration::from_nanos(nanos))),
            1 => any::<u64>().prop_map(|nanos| Step::Advance(Duration::from_nanos(nanos))),
            1 => any::<u64>().prop_map(|seconds| Step::Advance(Duration::from_secs(seconds))),
            2 => wall_time().prop_map(Step::SetWall),
        ]
    }

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
    fn manual_clock_reads_what_it_was_set_to() {
        let clock = ManualClock::new(MonotonicInstant::from_nanos(7), wall(1_000));
        assert_eq!(clock.monotonic(), MonotonicInstant::from_nanos(7));
        assert_eq!(clock.wall(), Ok(wall(1_000)));

        clock.advance(Duration::from_millis(5)).unwrap();
        assert_eq!(clock.monotonic(), MonotonicInstant::from_nanos(5_000_007));
        assert_eq!(clock.wall(), Ok(wall(1_005)));
    }

    #[test]
    fn wall_jumps_are_visible_and_leave_monotonic_time_alone() {
        let clock = ManualClock::new(MonotonicInstant::from_nanos(0), wall(1_000));
        clock.set_wall(wall(10));
        assert_eq!(clock.wall(), Ok(wall(10)));
        clock.set_wall(wall(99_999));
        assert_eq!(clock.wall(), Ok(wall(99_999)));
        assert_eq!(clock.monotonic(), MonotonicInstant::from_nanos(0));
    }

    #[test]
    fn sub_millisecond_advances_add_up_on_the_wall_clock() {
        let clock = ManualClock::new(MonotonicInstant::from_nanos(0), WallTime::UNIX_EPOCH);
        for _ in 0..999 {
            clock.advance(Duration::from_micros(1)).unwrap();
        }
        assert_eq!(clock.wall(), Ok(WallTime::UNIX_EPOCH));
        clock.advance(Duration::from_micros(1)).unwrap();
        assert_eq!(clock.wall(), Ok(wall(1)));
    }

    #[test]
    fn clones_share_one_clock() {
        let clock = ManualClock::new(MonotonicInstant::from_nanos(0), WallTime::UNIX_EPOCH);
        let shared: Arc<dyn Clock> = Arc::new(clock.clone());
        clock.advance(Duration::from_secs(1)).unwrap();
        clock.set_wall(wall(42));
        assert_eq!(
            shared.monotonic(),
            MonotonicInstant::from_nanos(1_000_000_000)
        );
        assert_eq!(shared.wall(), Ok(wall(42)));
    }

    #[test]
    fn an_overflowing_advance_changes_nothing() {
        let clock = ManualClock::new(MonotonicInstant::from_nanos(0), wall(i64::MAX));
        assert_eq!(
            clock.advance(Duration::from_millis(1)),
            Err(TimeError::WallOverflow {
                millis: i64::MAX,
                duration: Duration::from_millis(1),
            })
        );
        assert_eq!(clock.monotonic(), MonotonicInstant::from_nanos(0));
        assert_eq!(clock.wall(), Ok(wall(i64::MAX)));

        let clock = ManualClock::new(MonotonicInstant::from_nanos(u64::MAX), WallTime::UNIX_EPOCH);
        assert!(clock.advance(Duration::from_nanos(1)).is_err());
        assert_eq!(clock.wall(), Ok(WallTime::UNIX_EPOCH));
    }

    proptest! {
        #[test]
        fn manual_monotonic_time_never_goes_backwards(
            start in monotonic_instant(),
            start_wall in wall_time(),
            steps in proptest::collection::vec(step(), 0..50),
        ) {
            let clock = ManualClock::new(start, start_wall);
            // A model of both readings in wide integers.
            let mut monotonic = u128::from(start.as_nanos());
            let mut wall_nanos = u128::from(start_wall.as_millis().unsigned_abs()) * 1_000_000;
            for step in steps {
                let before = clock.monotonic();
                match step {
                    Step::Advance(by) => {
                        let fits = monotonic + by.as_nanos() <= u128::from(u64::MAX)
                            && (wall_nanos + by.as_nanos()) / 1_000_000 <= i64::MAX.unsigned_abs().into();
                        prop_assert_eq!(clock.advance(by).is_ok(), fits);
                        if fits {
                            monotonic += by.as_nanos();
                            wall_nanos += by.as_nanos();
                        }
                    }
                    Step::SetWall(to) => {
                        clock.set_wall(to);
                        wall_nanos = u128::from(to.as_millis().unsigned_abs()) * 1_000_000;
                    }
                }
                prop_assert!(clock.monotonic() >= before);
                prop_assert_eq!(u128::from(clock.monotonic().as_nanos()), monotonic);
                prop_assert_eq!(
                    u128::from(clock.wall().unwrap().as_millis().unsigned_abs()),
                    wall_nanos / 1_000_000
                );
            }
        }
    }

    #[test]
    fn clocks_are_shared_as_send_sync_trait_objects() {
        fn shareable<T: Send + Sync + Debug>(_: &T) {}
        let clock: Arc<dyn Clock> = Arc::new(SystemClock::start());
        shareable(&clock);
    }
}
