# Testing standard

Tests exist to prove behavior and to make bugs reproducible. A test that can't fail for a real bug, or that fails at random, has negative value.

Decisions are in [ADR-0003](../adr/0003-deterministic-simulation-testing.md).

## Test levels

| Level | Where | What it's for | Required when |
|---|---|---|---|
| **Unit** | `#[cfg(test)] mod tests` beside the code | One function or state machine transition | Any logic |
| **Property** (`proptest`) | Beside the code | Codecs round-trip; state machine invariants hold for arbitrary input sequences | Every codec; every state machine |
| **Fuzz** (`cargo-fuzz`) | `fuzz/fuzz_targets/` | No panics, hangs or unbounded allocation on arbitrary bytes | Every decoder (network and disk) |
| **Simulation** | `crates/<crate>/tests/sim_*.rs` using `alluvium-testkit` | Distributed behavior under message loss, delay, reordering, partitions, crashes and disk faults, with invariant checkers | Raft, controller, replication, coordinators, anything with timers or peers |
| **Crash consistency** | `alluvium-storage` tests with the fault-injecting `fs` | Recovery after torn writes, lost unsynced data and crashes at every write point | Any storage write path |
| **Compatibility** | `tests/compat/` | Real Kafka clients (Java client and CLI tools, librdkafka, franz-go) against Alluvium | Every client-visible API before it is advertised |
| **Differential** | `tests/compat/differential/` | The same scenario against Kafka 4.4.0 and Alluvium; responses compared field by field (ignoring fields that legitimately differ, such as node ids and timestamps) | Every API; this is the parity proof |
| **Benchmark** (`criterion`) | `crates/<crate>/benches/` ([conventions](#benchmarks)) | Hot-path performance; regression detection | Hot paths and any change justified by performance |

## Rules

1. **A bug fix starts with a failing test** that reproduces the bug at the lowest level that can show it.
2. **No real time and no sleeps.** Tests use the simulated clock. Waiting for "long enough" is a flaky test waiting to happen.
3. **No real randomness.** Randomized tests take a seed, print it on failure, and replay it via `ALLUVIUM_SIM_SEED` or proptest's persisted regressions (committed under `proptest-regressions/`).
4. **Test behavior, not implementation.** Assert on outputs, emitted actions, state visible through the public API, and invariants, not on private fields or call counts.
5. **Don't mock our own state machines.** Use the real machine under a simulated driver. Mocks are for external systems only.
6. **Every test is independent and parallel-safe.** nextest runs each test in its own process. Tests use temporary directories and never fixed ports.
7. **Flaky tests are bugs.** Quarantine one with `#[ignore = "flaky: #issue"]` only together with an open issue. Fix it within the milestone. That is why the nextest profiles in [`.config/nextest.toml`](../../.config/nextest.toml) never retry: a retry would hide the flake. Tests are reported as slow after 60 s and stopped after 300 s. CI runs them on ubuntu-24.04 and macos-15 (arm64) for every PR and push, and keeps each run's JUnit report as an artifact.
8. **Names describe the behavior**: `fetch_beyond_high_watermark_returns_no_records`, not `test_fetch_3`.
9. **Cover each supported API version.** Version-dependent behavior is tested for every version in the advertised range.
10. **Port Kafka's tests where they encode tricky semantics.** Name the source in a comment: `// Ported from: core/src/test/scala/unit/kafka/cluster/PartitionTest.scala@4.4.0 testXyz`. Follow the attribution rules in [ADR-0008](../adr/0008-protocol-code-generated-from-kafka-schemas.md).

## Shared strategies

Proptest strategies for a crate's types live in that crate, in a `strategies` module behind its `proptest-strategies` feature. Other crates enable the feature on their dev-dependency: `alluvium-types = { workspace = true, features = ["proptest-strategies"] }`. They don't live in `alluvium-testkit`, because testkit depends on those crates: a dev-dependency back to testkit would compile two copies of the types, and values from one copy wouldn't type-check against the other. Strategies produce valid values only and favour boundaries (0, 1, the maximum, and the wire sentinels where those are valid).

## Simulation harness (`alluvium-testkit`)

- Drives real layer 1–2 state machines (and, where needed, real drivers) with a simulated clock, network and disk, all controlled by one seeded RNG.
- A fault plan sets drop, delay and reorder rates, network partitions, node crashes and restarts, disk write failures, torn writes and fsync loss.
- **Checkers** run after every step and assert the [architecture invariants](../architecture.md#8-invariants): for example, no acknowledged `acks=all` write is ever lost, high watermarks never move backwards on a leader, and there is at most one leader per epoch.
- On failure it prints the seed, the step number, and the flight recorders of the involved components.
- In CI, each simulation test runs with a fixed set of seeds plus some new random seeds per run. Nightly runs use thousands of seeds. Any failing seed is added to the fixed set.

## Compatibility and differential tests

- Run with `cargo xtask compat`. Real Kafka 4.4.0 and the client tools run in containers whose images are pinned by digest.
- Client matrix: the Apache Kafka Java client 4.4.0, librdkafka (current stable), franz-go (current stable), and the `kafka-*.sh` admin tools.
- An API's backlog feature is marked **Done** only when its compatibility and differential tests pass.

## Benchmarks

- Each crate has one criterion target, `crates/<crate>/benches/<area>.rs`, declared in its `Cargo.toml` with `harness = false`. Run it with `cargo bench -p <crate>`. The first is `alluvium-types`' [`benches/types.rs`](../../crates/alluvium-types/benches/types.rs).
- Benchmark ids are `<area>/<operation>/<size>`, such as `types/murmur2/4096`: one group per operation, with the input size as the parameter.
- Inputs are deterministic, drawn from `SeededRng::from_seed` with a fixed seed, so two runs measure the same work.
- Every group declares its throughput (`Throughput::Bytes` or `Throughput::Elements`), so results compare across sizes and machines.
- A PR that claims a performance change includes the before and after numbers and the CPU they ran on.
- `criterion` is built without its default features: no plotting and no rayon, only `cargo bench` support. Benchmarks don't run in CI yet; regression gating is TEST-0095.

## Coverage

Line coverage is tracked but not used as a target. The question for review is "which behaviors and failure modes are tested?", not "what percentage is covered?".
