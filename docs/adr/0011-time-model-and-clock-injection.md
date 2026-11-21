# ADR-0011: Time model and clock injection

- **Status:** Accepted
- **Date:** 2026-11-21

## Context

[ADR-0002](0002-sans-io-state-machines-driven-by-tokio.md) says time comes from an injected `Clock`, and [ADR-0003](0003-deterministic-simulation-testing.md) needs the simulator to control it. Neither says what a time value is, which clock a timeout uses, or how async code waits without calling Tokio's timers directly.

Kafka 4.4.0 separates two readings in `clients/src/main/java/org/apache/kafka/common/utils/Time.java`:

- `milliseconds()`: wall-clock time, `System.currentTimeMillis()` in `SystemTime`. Record timestamps, retention and most timeouts use it.
- `nanoseconds()`: a monotonic reading with an arbitrary origin, `System.nanoTime()`, documented as usable "only to measure elapsed time and not related to any other notion of system or wall-clock time".

`Timer` (same package) builds deadlines on `milliseconds()`, saturates at `Long.MAX_VALUE` instead of overflowing, and never lets the cached time go backwards. Because Kafka's timeouts are mostly on wall-clock time, a wall-clock jump (NTP step, VM migration) can expire sessions or delay timeouts. `Time.sleep` and `Time.waitObject` let code block on time, and `MockTime` replaces all of this in tests.

Kafka's wire format carries wall-clock times as `int64` milliseconds since the Unix epoch, with `-1` meaning "no timestamp" in record batches.

## Decision

1. **Two time types, never converted into each other.**
   - `MonotonicInstant`: nanoseconds since the clock's own origin, as `u64`. It never goes backwards and means nothing outside its clock. Used for every deadline, timeout, interval and elapsed-time measurement.
   - `WallTime`: milliseconds since the Unix epoch, as a non-negative `i64`, the unit Kafka puts on the wire and in configs. Used for record timestamps, time-based retention, timestamps sent to or received from clients, and log and flight-recorder timestamps. Wire sentinels (`-1`, "no timestamp") become `None` at the decoder, as with ids.
   - Spans of time are `std::time::Duration`, a plain value that reads no clock. Config values in milliseconds are converted to `Duration` in `alluvium-config`.
   - Arithmetic is checked: adding a `Duration` returns an error on overflow. A deadline that would overflow saturates to "never", as Kafka's `Timer` saturates to `Long.MAX_VALUE`.
   - No `From` between the two types, and no "monotonic to wall" helper: code that needs both reads both.

2. **Timeouts use monotonic time.** Session, request, election, transaction and delivery timeouts are deadlines on `MonotonicInstant`, not on wall time as in Kafka. Wall time is used only where Kafka's *observable* behavior is defined by wall time: record timestamps, `log.retention.ms` against those timestamps, `ListOffsets` by timestamp, and timestamps returned to clients. Clients see the same behavior; the difference is that a wall-clock jump can't expire a session or stall a timeout. This is internal and needs no compatibility ADR.

3. **The `Clock` trait reads time and nothing else.** In `alluvium-types`:
   ```rust
   pub trait Clock: Send + Sync + fmt::Debug {
       fn monotonic(&self) -> MonotonicInstant;
       fn wall(&self) -> Result<WallTime, WallClockError>;
   }
   ```
   `wall` returns an error instead of panicking when the system clock is before the Unix epoch. The trait is object-safe and passed explicitly as `Arc<dyn Clock>`; there is no global clock. It has no `sleep`: waiting is a separate concern (point 5).

4. **Implementations, and the only sanctioned clock reads.**
   - `SystemClock` (`alluvium-types`): the only product code that calls `std::time::Instant::now` and `std::time::SystemTime::now`, each under one `#[expect(clippy::disallowed_methods, reason = "…")]`. It records its origin `Instant` at construction, so a `MonotonicInstant` can be turned back into a `std::time::Instant` for the production timer.
   - `ManualClock` (`alluvium-types`, not test-gated): time moves only when told. `advance` moves both readings; `set_wall` jumps wall time either way without touching monotonic time. Clones share state.
   - `SimClock` (`alluvium-testkit`): built on `ManualClock` and advanced only by the simulation scheduler (TEST-F01).
   - `xtask`'s `Stopwatch` stays the dev-tool exception for timing CI steps. It never runs under simulation.

5. **Waiting goes through one seam.** State machines in layers 0–2 never hold a clock or wait: their inputs carry `now: MonotonicInstant` (and `WallTime` when an input needs it), and they return deadlines as `MonotonicInstant` values in their actions. Async code in layers 3–5 waits through a `Timer` trait defined in `alluvium-types`, with no runtime dependency:
   ```rust
   pub trait Timer: Send + Sync + fmt::Debug {
       fn sleep_until(&self, deadline: MonotonicInstant) -> Pin<Box<dyn Future<Output = ()> + Send>>;
   }
   ```
   - Production: `TokioTimer` in `alluvium-server`, the only call site of `tokio::time::sleep_until`, under one `#[expect]`. It converts the deadline with `SystemClock`'s origin. The binary injects it into the network layer and the drivers, as it does the clock.
   - Simulation: a timer in `alluvium-testkit` whose futures complete when the scheduler advances `SimClock` past their deadline. How the simulator polls driver futures (a custom executor, `turmoil`, or Tokio's paused time) is TEST-0002's decision; this seam works with each of them.
   - The task supervisor (SRV-F02) and the delayed-operation timing wheel (SRV-F07) wait only through `Timer` and read only `Clock`.

6. **The lints cover every direct path to the clock.** Besides the existing bans, `clippy.toml` disallows `std::time::Instant::elapsed`, `std::time::SystemTime::elapsed`, `tokio::time::sleep_until`, `tokio::time::timeout`, `tokio::time::timeout_at`, `tokio::time::interval` and `tokio::time::interval_at`. Each has a lint-canary fixture. A timeout around a future is written as a `select!` against `Timer::sleep_until`.

7. **Layers 0–2 define but never construct the system clock.** `SystemClock` lives in `alluvium-types` so that every crate shares one implementation, and each banned function has one exception. Only layer 3–5 code may construct it; architecture §5 says so, and a `check-layers` source rule enforces it.

## Consequences

- Every timeout in a state machine is a value in an input or an action, so a replayed input sequence reproduces it exactly, and simulation can skip hours in milliseconds.
- A wall-clock jump on a node changes record timestamps and time-based retention, as in Kafka, but expires no session and stalls no timeout. NFR-F13's clock-skew tests check both halves.
- Every async wait allocates a boxed future. That cost is acceptable for connection and request timeouts; bulk delayed operations go through SRV-F07's timing wheel, which waits on one deadline at a time.
- Plan changes:
  - TYPE-0024: the type names become `MonotonicInstant` and `WallTime`.
  - TYPE-0025: excludes `xtask`'s `Stopwatch` from its one-call-site check.
  - New tasks:
    - TYPE-0048: the `Timer` trait.
    - SRV-0149: `TokioTimer`.
    - TEST-0131: the simulation timer.
    - FND-0065: the `check-layers` source rule.
  - FND-0064 adds the new clippy bans and their fixtures.

## Alternatives considered

- **One time type in Unix milliseconds, as Kafka mostly uses.** Rejected: wall time can jump, which turns NTP corrections into spurious session expiries; and millisecond resolution is too coarse for latency measurement.
- **Use `std::time::Instant` as the monotonic type.** Rejected: it can't be constructed at an arbitrary value, so `ManualClock` and simulation couldn't produce it.
- **Put `sleep` on `Clock`, as Kafka's `Time` does.** Rejected: `alluvium-types` would need an async runtime for the production implementation, and layer 0–2 code would gain a way to wait, which ADR-0002 forbids.
- **Let async code call `tokio::time` directly and rely on Tokio's paused time in tests.** Rejected: it ties simulation to one executor before TEST-0002 has compared the options, and paused time is per runtime, not per simulated node.
- **Put `SystemClock` in `alluvium-server`, so layering alone keeps it out of layers 0–2.** Rejected: the network layer (3) needs a clock too and can't depend on layer 4. A source rule is cheaper than a new crate.
