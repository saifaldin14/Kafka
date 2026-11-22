//! Points in time ([ADR-0011](../../../docs/adr/0011-time-model-and-clock-injection.md)).
//!
//! Two types that are never converted into each other:
//!
//! - [`MonotonicInstant`]: for every deadline, timeout, interval and elapsed time. It
//!   never goes backwards, and means nothing outside the clock that produced it.
//! - [`WallTime`]: for timestamps Kafka defines by the wall clock: record timestamps,
//!   time-based retention, `ListOffsets` by timestamp, and times sent to clients.
//!
//! Spans of time are [`Duration`]. Code that needs both kinds of time reads both from
//! its clock; there is deliberately no conversion. These lines don't compile:
//!
//! ```compile_fail,E0277
//! use alluvium_types::{MonotonicInstant, WallTime};
//! let wall: WallTime = MonotonicInstant::from_nanos(0).into();
//! ```
//!
//! ```compile_fail,E0277
//! use alluvium_types::{MonotonicInstant, WallTime};
//! let instant: MonotonicInstant = WallTime::UNIX_EPOCH.into();
//! ```

use std::fmt;
use std::time::Duration;

/// A reading of a monotonic clock: nanoseconds since that clock's origin.
///
/// Readings from one clock never decrease. Readings from different clocks (different
/// processes, or different simulated nodes) can't be compared meaningfully.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonotonicInstant(u64);

/// Wall-clock time: milliseconds since the Unix epoch, never negative.
///
/// Millisecond resolution is what Kafka puts on the wire and in configs. Kafka's
/// "no timestamp" (`-1`) is not a `WallTime`; decoders turn it into `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WallTime(i64);

/// Why a time value was rejected or an operation on one failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TimeError {
    /// Wall time is never before the Unix epoch.
    #[error("negative wall time {millis} ms")]
    NegativeWallTime {
        /// The rejected value.
        millis: i64,
    },
    /// `instant + duration` doesn't fit in `u64` nanoseconds (about 584 years).
    #[error("monotonic instant {nanos} ns plus {duration:?} overflows")]
    MonotonicOverflow {
        /// The instant, in nanoseconds since its clock's origin.
        nanos: u64,
        /// The duration that was added.
        duration: Duration,
    },
    /// `wall time + duration` doesn't fit in `i64` milliseconds.
    #[error("wall time {millis} ms plus {duration:?} overflows")]
    WallOverflow {
        /// The wall time, in milliseconds since the Unix epoch.
        millis: i64,
        /// The duration that was added.
        duration: Duration,
    },
}

impl MonotonicInstant {
    /// The instant `nanos` nanoseconds after the clock's origin.
    ///
    /// Clocks produce instants; this is for clocks, and for tests and simulations that
    /// start a manual clock at a chosen reading.
    pub const fn from_nanos(nanos: u64) -> MonotonicInstant {
        MonotonicInstant(nanos)
    }

    /// Nanoseconds since the clock's origin.
    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    /// `self + duration`.
    ///
    /// # Errors
    ///
    /// [`TimeError::MonotonicOverflow`] if the result doesn't fit in `u64` nanoseconds.
    pub fn checked_add(self, duration: Duration) -> Result<MonotonicInstant, TimeError> {
        u64::try_from(duration.as_nanos())
            .ok()
            .and_then(|nanos| self.0.checked_add(nanos))
            .map(MonotonicInstant)
            .ok_or(TimeError::MonotonicOverflow {
                nanos: self.0,
                duration,
            })
    }

    /// The time from `earlier` to `self`, or zero if `earlier` is later.
    pub fn saturating_duration_since(self, earlier: MonotonicInstant) -> Duration {
        Duration::from_nanos(self.0.saturating_sub(earlier.0))
    }
}

impl WallTime {
    /// 1970-01-01T00:00:00Z.
    pub const UNIX_EPOCH: WallTime = WallTime(0);

    /// Milliseconds since the Unix epoch, as Kafka writes them.
    pub fn as_millis(self) -> i64 {
        self.0
    }

    /// `self + duration`. Wall time has millisecond resolution, so any part of
    /// `duration` below a millisecond is dropped.
    ///
    /// # Errors
    ///
    /// [`TimeError::WallOverflow`] if the result doesn't fit in `i64` milliseconds.
    pub fn checked_add(self, duration: Duration) -> Result<WallTime, TimeError> {
        i64::try_from(duration.as_millis())
            .ok()
            .and_then(|millis| self.0.checked_add(millis))
            .map(WallTime)
            .ok_or(TimeError::WallOverflow {
                millis: self.0,
                duration,
            })
    }

    /// The time from `earlier` to `self`, or zero if `earlier` is later.
    ///
    /// Wall time can jump backwards, so the zero case is expected, not an error. Measure
    /// elapsed time with [`MonotonicInstant`] instead.
    pub fn saturating_duration_since(self, earlier: WallTime) -> Duration {
        // Both values are non-negative, so the difference can't overflow, and it is
        // negative only when `earlier` is later.
        u64::try_from(self.0 - earlier.0).map_or(Duration::ZERO, Duration::from_millis)
    }
}

impl TryFrom<i64> for WallTime {
    type Error = TimeError;

    fn try_from(millis: i64) -> Result<WallTime, TimeError> {
        if millis < 0 {
            Err(TimeError::NegativeWallTime { millis })
        } else {
            Ok(WallTime(millis))
        }
    }
}

/// The milliseconds, then the time in RFC 3339 UTC with milliseconds, for example
/// `1700000000000 (2023-11-14T22:13:20.000Z)`.
///
/// RFC 3339 has four-digit years. After year 9999 the year is written with a `+` and as
/// many digits as it needs (ISO 8601's expanded form, as `java.time.Instant` prints it).
impl fmt::Display for WallTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const MILLIS_PER_DAY: i64 = 86_400_000;
        let days = self.0.div_euclid(MILLIS_PER_DAY);
        let millis_of_day = self.0.rem_euclid(MILLIS_PER_DAY);
        let (year, month, day) = civil_from_days(days);
        let hour = millis_of_day / 3_600_000;
        let minute = millis_of_day / 60_000 % 60;
        let second = millis_of_day / 1_000 % 60;
        let milli = millis_of_day % 1_000;
        write!(f, "{} (", self.0)?;
        if year > 9999 {
            write!(f, "+")?;
        }
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{milli:03}Z)"
        )
    }
}

/// The proleptic Gregorian `(year, month, day)` of a day count since 1970-01-01.
///
/// Howard Hinnant's `civil_from_days` (<https://howardhinnant.github.io/date_algorithms.html>):
/// it counts in 400-year eras starting on 0000-03-01, so the leap day is the last day
/// of each counted year.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    // Days from 0000-03-01 to 1970-01-01. `days` comes from a non-negative `i64` of
    // milliseconds, so it is at most about 1.1e11 and this can't overflow.
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 {
        month_from_march + 3
    } else {
        month_from_march - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};

    use super::{MonotonicInstant, TimeError, WallTime, civil_from_days};
    use crate::strategies::{monotonic_instant, wall_time};

    fn wall(millis: i64) -> WallTime {
        WallTime::try_from(millis).unwrap()
    }

    #[test]
    fn negative_wall_time_is_rejected() {
        assert_eq!(
            WallTime::try_from(-1),
            Err(TimeError::NegativeWallTime { millis: -1 })
        );
        assert_eq!(
            WallTime::try_from(i64::MIN),
            Err(TimeError::NegativeWallTime { millis: i64::MIN })
        );
        assert_eq!(WallTime::try_from(0), Ok(WallTime::UNIX_EPOCH));
    }

    #[test]
    fn adding_at_the_limit_fails_with_the_operands() {
        let last = MonotonicInstant::from_nanos(u64::MAX);
        assert_eq!(
            last.checked_add(Duration::from_nanos(1)),
            Err(TimeError::MonotonicOverflow {
                nanos: u64::MAX,
                duration: Duration::from_nanos(1),
            })
        );
        assert_eq!(last.checked_add(Duration::ZERO), Ok(last));
        assert!(
            MonotonicInstant::from_nanos(0)
                .checked_add(Duration::MAX)
                .is_err()
        );

        let last = wall(i64::MAX);
        assert_eq!(
            last.checked_add(Duration::from_millis(1)),
            Err(TimeError::WallOverflow {
                millis: i64::MAX,
                duration: Duration::from_millis(1),
            })
        );
        assert!(WallTime::UNIX_EPOCH.checked_add(Duration::MAX).is_err());
    }

    #[test]
    fn wall_time_drops_sub_millisecond_durations() {
        assert_eq!(
            wall(10).checked_add(Duration::from_micros(1_999)),
            Ok(wall(11))
        );
        assert_eq!(wall(10).checked_add(Duration::from_nanos(1)), Ok(wall(10)));
    }

    #[test]
    fn display_prints_millis_and_rfc_3339_utc() {
        // Checked against Python's `datetime` up to year 9999, and `java.time.Instant`
        // for `i64::MAX`.
        let cases = [
            (0, "0 (1970-01-01T00:00:00.000Z)"),
            (951_782_400_000, "951782400000 (2000-02-29T00:00:00.000Z)"),
            (
                1_234_567_890_123,
                "1234567890123 (2009-02-13T23:31:30.123Z)",
            ),
            (
                1_700_000_000_000,
                "1700000000000 (2023-11-14T22:13:20.000Z)",
            ),
            (
                4_107_542_399_999,
                "4107542399999 (2100-02-28T23:59:59.999Z)",
            ),
            (
                4_107_542_400_000,
                "4107542400000 (2100-03-01T00:00:00.000Z)",
            ),
            (
                253_402_300_799_999,
                "253402300799999 (9999-12-31T23:59:59.999Z)",
            ),
            (
                253_402_300_800_000,
                "253402300800000 (+10000-01-01T00:00:00.000Z)",
            ),
            (
                i64::MAX,
                "9223372036854775807 (+292278994-08-17T07:12:55.807Z)",
            ),
        ];
        for (millis, expected) in cases {
            assert_eq!(wall(millis).to_string(), expected);
        }
    }

    #[test]
    fn every_day_from_1970_to_10000_follows_the_previous_one() {
        // An independent, naive calendar: step one day at a time with the Gregorian
        // leap-year rule, and compare with the era arithmetic.
        let is_leap = |year: i64| year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let month_length = |year: i64, month: i64| match month {
            2 if is_leap(year) => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        let mut expected = (1970, 1, 1);
        let mut days = 0;
        while expected != (10_001, 1, 1) {
            assert_eq!(civil_from_days(days), expected, "day {days}");
            let (year, month, day) = expected;
            expected = if day < month_length(year, month) {
                (year, month, day + 1)
            } else if month < 12 {
                (year, month + 1, 1)
            } else {
                (year + 1, 1, 1)
            };
            days += 1;
        }
    }

    proptest! {
        #[test]
        fn monotonic_add_matches_wide_arithmetic(start in monotonic_instant(), nanos in any::<u64>(), seconds in any::<u64>()) {
            let duration = Duration::new(seconds, u32::try_from(nanos % 1_000_000_000).unwrap());
            let expected = u128::from(start.as_nanos()) + duration.as_nanos();
            match start.checked_add(duration) {
                Ok(sum) => prop_assert_eq!(u128::from(sum.as_nanos()), expected),
                Err(_) => prop_assert!(expected > u128::from(u64::MAX)),
            }
        }

        #[test]
        fn wall_add_matches_wide_arithmetic(start in wall_time(), millis in any::<u64>()) {
            let duration = Duration::from_millis(millis);
            let expected = i128::from(start.as_millis()) + i128::from(millis);
            match start.checked_add(duration) {
                Ok(sum) => prop_assert_eq!(i128::from(sum.as_millis()), expected),
                Err(_) => prop_assert!(expected > i128::from(i64::MAX)),
            }
        }

        #[test]
        fn duration_since_saturates_at_zero(a in monotonic_instant(), b in monotonic_instant(), c in wall_time(), d in wall_time()) {
            let expected = Duration::from_nanos(a.as_nanos().max(b.as_nanos()) - b.as_nanos());
            prop_assert_eq!(a.saturating_duration_since(b), expected);
            let expected = c.as_millis().max(d.as_millis()) - d.as_millis();
            prop_assert_eq!(c.saturating_duration_since(d).as_millis(), u128::try_from(expected).unwrap());
        }
    }
}
