# ADR-0002: Sans-I/O state machines driven by Tokio

- **Status:** Accepted
- **Date:** 2026-11-20

## Context

Kafka's hardest bugs live in the interaction of timers, network messages and disk: leader elections, ISR changes, rebalances, transaction fencing. When that logic is interleaved with I/O and threads, the bugs can't be reproduced, and debugging means reading logs and guessing.

A broker needs an async runtime for tens of thousands of connections. Tokio is the standard choice in Rust: mature, widely understood, well instrumented (`tracing`, `tokio-console`), and easy to hire for. A thread-per-core design (as in Redpanda/Seastar) can be faster but is harder to debug and has a smaller library ecosystem.

## Decision

1. **Domain logic is sans-I/O.** Every layer 1–2 component is a synchronous, deterministic state machine. Its inputs are explicit (a request, a peer message, a timer tick carrying the current time, a completed I/O), and its outputs are explicit actions (send, append, schedule a timer, respond) plus transitions with reasons. It never performs I/O, reads a clock, spawns work or uses unseeded randomness.
2. **Drivers own the I/O.** In `alluvium-server`, a driver per component runs on Tokio. It feeds inputs, executes actions, and records transitions in the flight recorder, logs and metrics.
3. **Tokio multi-threaded runtime**, created only in the binary. Runtime types never appear in layers 0–2.
4. **Time comes from an injected `Clock`**, and randomness from an injected seeded RNG, including in drivers. Simulation can then control both.

## Consequences

- Any state machine bug can be reproduced by replaying its inputs: in a unit test, in simulation (ADR-0003), or from an input journal.
- State machines are testable without a runtime, which keeps tests fast and deterministic.
- There is more explicit plumbing (action enums, driver loops) than in code that does I/O inline. We accept that cost for debuggability.
- Performance work concentrates in drivers and storage, without touching the logic.
- If thread-per-core is ever needed, only the drivers change.

## Alternatives considered

- **Async logic interleaved with I/O.** Rejected: not reproducible, and hard to test without timing-dependent tests.
- **Thread-per-core (glommio, monoio).** Rejected for now: harder debugging, fewer libraries, Linux-only `io_uring` focus. The sans-I/O design keeps this option open.
- **The actor framework of a library.** Rejected: hides control flow and adds a dependency for something plain tasks and channels do.
