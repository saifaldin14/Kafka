//! Waiting: the one way async code in layers 3–5 waits until an instant
//! ([ADR-0011](../../../docs/adr/0011-time-model-and-clock-injection.md)).
//!
//! State machines in layers 0–2 never wait. Their inputs carry `now`, and they return
//! the instants they need to be woken at, usually from a [`Deadline`](crate::Deadline),
//! in their actions. Their drivers wait for those instants through a [`Timer`], passed
//! explicitly as `Arc<dyn Timer>` like the [`Clock`](crate::Clock) it pairs with.
//!
//! The trait names no async runtime, so this crate stays runtime-free. Production uses
//! a Tokio-backed timer in `alluvium-server`. Simulation uses one in `alluvium-testkit`
//! whose futures complete when simulated time passes their instant, so a run that
//! waits for hours replays in milliseconds. A timeout around another future is a
//! `select!` against [`Timer::sleep_until`].

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use crate::MonotonicInstant;

/// Waits until a monotonic instant.
///
/// An implementation reads the same monotonic time as the node's [`Clock`](crate::Clock),
/// so a deadline means the same thing to both.
pub trait Timer: Send + Sync + fmt::Debug {
    /// A future that completes once the timer's monotonic time has reached `deadline`,
    /// at once if it already has. Dropping the future cancels the wait.
    fn sleep_until(&self, deadline: MonotonicInstant) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

#[cfg(test)]
mod tests {
    use std::future::{Future, poll_fn};
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll, Waker};
    use std::time::Duration;

    use super::Timer;
    use crate::{Clock, Deadline, ManualClock, MonotonicInstant, WallTime};

    /// A timer over a shared `ManualClock`: its futures are ready once the clock reaches
    /// their instant. Polling it needs no runtime.
    #[derive(Debug)]
    struct ManualTimer(ManualClock);

    impl Timer for ManualTimer {
        fn sleep_until(
            &self,
            instant: MonotonicInstant,
        ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
            let clock = self.0.clone();
            Box::pin(poll_fn(move |_| {
                if clock.monotonic() >= instant {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }))
        }
    }

    #[test]
    fn a_sleep_completes_once_time_reaches_its_instant_without_a_runtime() {
        let clock = ManualClock::new(MonotonicInstant::from_nanos(0), WallTime::UNIX_EPOCH);
        let timer: Arc<dyn Timer> = Arc::new(ManualTimer(clock.clone()));
        let deadline = Deadline::after(clock.monotonic(), Duration::from_secs(5));
        let mut sleep = timer.sleep_until(deadline.instant());
        let mut context = Context::from_waker(Waker::noop());

        assert_eq!(sleep.as_mut().poll(&mut context), Poll::Pending);
        clock.advance(Duration::from_secs(4)).unwrap();
        assert_eq!(sleep.as_mut().poll(&mut context), Poll::Pending);
        clock.advance(Duration::from_secs(1)).unwrap();
        assert_eq!(sleep.as_mut().poll(&mut context), Poll::Ready(()));

        let mut past = timer.sleep_until(MonotonicInstant::from_nanos(0));
        assert_eq!(past.as_mut().poll(&mut context), Poll::Ready(()));
    }
}
