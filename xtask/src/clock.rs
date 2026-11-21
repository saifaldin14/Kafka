//! The one place `xtask` reads the clock.
//!
//! `xtask` times the external commands it runs for its step summary. It is a dev-layer
//! tool that never runs under simulation, so it reads the real monotonic clock, but
//! only through this module (docs/standards/rust.md#workspace-configuration).

use std::time::{Duration, Instant};

/// Measures wall time from `start` until `elapsed` is called.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Stopwatch(Instant);

impl Stopwatch {
    #[expect(
        clippy::disallowed_methods,
        reason = "xtask is a dev tool outside simulation; this is its only clock read"
    )]
    pub(crate) fn start() -> Self {
        Self(Instant::now())
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "xtask is a dev tool outside simulation; this is its only elapsed-time read"
    )]
    pub(crate) fn elapsed(self) -> Duration {
        self.0.elapsed()
    }
}
