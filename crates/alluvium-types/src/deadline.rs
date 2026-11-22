//! Deadlines: the instant by which a wait must end.
//!
//! Every network wait has a timeout (rule 9 of the
//! [engineering skill](../../../.agents/skills/alluvium-engineering/SKILL.md)), and a
//! timeout becomes a [`Deadline`] when the wait starts. State machines receive `now` in
//! their inputs and return deadlines in their actions; drivers wait until them through
//! the `Timer` seam ([ADR-0011](../../../docs/adr/0011-time-model-and-clock-injection.md)).

use std::time::Duration;

use crate::MonotonicInstant;

/// A monotonic instant by which something must happen.
///
/// Built from a start and a timeout. A timeout too long to represent saturates to the
/// last representable instant, about 584 years after the clock's origin, which in
/// practice means "never". Kafka's `Timer` saturates the same way, at `Long.MAX_VALUE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Deadline(MonotonicInstant);

// Parity: clients/src/main/java/org/apache/kafka/common/utils/Timer.java@4.4.0 Timer.reset, Timer.isExpired, Timer.remainingMs
impl Deadline {
    /// The deadline `timeout` after `now`, saturating instead of overflowing.
    pub fn after(now: MonotonicInstant, timeout: Duration) -> Deadline {
        Deadline(
            now.checked_add(timeout)
                .unwrap_or(MonotonicInstant::from_nanos(u64::MAX)),
        )
    }

    /// The instant the deadline falls on, for a timer to wait until.
    pub fn instant(self) -> MonotonicInstant {
        self.0
    }

    /// True from the deadline's instant onwards. A zero timeout is expired at once.
    pub fn is_expired(self, now: MonotonicInstant) -> bool {
        now >= self.0
    }

    /// The time left until the deadline: zero at and after it, never negative.
    pub fn remaining(self, now: MonotonicInstant) -> Duration {
        self.0.saturating_duration_since(now)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use proptest::prelude::{any, prop_assert_eq, proptest};

    use super::Deadline;
    use crate::MonotonicInstant;
    use crate::strategies::monotonic_instant;

    fn at(nanos: u64) -> MonotonicInstant {
        MonotonicInstant::from_nanos(nanos)
    }

    #[test]
    fn a_zero_timeout_is_expired_at_once() {
        let deadline = Deadline::after(at(100), Duration::ZERO);
        assert!(deadline.is_expired(at(100)));
        assert_eq!(deadline.remaining(at(100)), Duration::ZERO);
    }

    #[test]
    fn remaining_counts_down_to_zero_and_stays_there() {
        let deadline = Deadline::after(at(100), Duration::from_nanos(50));
        assert_eq!(deadline.instant(), at(150));
        assert!(!deadline.is_expired(at(149)));
        assert_eq!(deadline.remaining(at(149)), Duration::from_nanos(1));
        assert!(deadline.is_expired(at(150)));
        assert_eq!(deadline.remaining(at(150)), Duration::ZERO);
        assert!(deadline.is_expired(at(1_000)));
        assert_eq!(deadline.remaining(at(1_000)), Duration::ZERO);
    }

    #[test]
    fn an_overflowing_timeout_saturates_to_never() {
        let deadline = Deadline::after(at(1), Duration::MAX);
        assert_eq!(deadline.instant(), at(u64::MAX));
        assert!(!deadline.is_expired(at(u64::MAX - 1)));
        assert_eq!(
            Deadline::after(at(u64::MAX), Duration::from_nanos(1)).instant(),
            at(u64::MAX)
        );
    }

    proptest! {
        #[test]
        fn deadlines_match_saturating_arithmetic(start in monotonic_instant(), timeout in any::<u64>(), now in monotonic_instant()) {
            let deadline = Deadline::after(start, Duration::from_nanos(timeout));
            prop_assert_eq!(deadline.instant(), at(start.as_nanos().saturating_add(timeout)));
            prop_assert_eq!(deadline.is_expired(now), deadline.remaining(now).is_zero());
            if !deadline.is_expired(now) {
                prop_assert_eq!(now.checked_add(deadline.remaining(now)), Ok(deadline.instant()));
            }
        }
    }
}
