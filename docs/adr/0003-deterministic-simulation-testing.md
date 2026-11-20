# ADR-0003: Deterministic simulation testing

- **Status:** Accepted
- **Date:** 2026-11-20

## Context

Distributed bugs appear under rare interleavings: a message delayed during an election, a crash between write and fsync. Real-time integration tests hit these rarely and can't reproduce them. FoundationDB and TigerBeetle showed that deterministic simulation finds and reproduces such bugs reliably.

ADR-0002 makes this cheap. State machines are already deterministic, so the simulator only needs to control time, message delivery, disk behavior and randomness.

## Decision

1. `alluvium-testkit` provides a simulator: a simulated clock, network and disk, one seeded RNG, fault plans, and invariant checkers.
2. Simulation is the **primary** test level for Raft, the controller, replication and coordinators. Real-time multi-process tests are used only for compatibility and differential testing.
3. Every run prints its seed. `ALLUVIUM_SIM_SEED=<n>` replays it exactly. Failing seeds are added to the fixed regression set.
4. CI runs fixed seeds plus new random seeds on each PR. A nightly job runs thousands of seeds.
5. Determinism is protected by lints: wall-clock access, unseeded randomness and `RandomState` hash maps are disallowed ([rust.md](../standards/rust.md#workspace-configuration)).

## Consequences

- Distributed bugs become ordinary failing tests with a seed.
- Engineers must keep components deterministic. The lints catch the common mistakes. Iteration order is the subtle one, handled by `BTreeMap` and seeded hashers.
- The simulator is a real piece of software that needs maintenance and its own tests.

## Alternatives considered

- **`turmoil` only.** Useful for simulating Tokio networking in driver tests, and may be used there. Not sufficient alone: it doesn't model disk faults, and it tests through the runtime rather than at the state-machine level.
- **Jepsen-style external testing only.** Valuable later as an independent check, but slow, and failures are hard to reproduce.
