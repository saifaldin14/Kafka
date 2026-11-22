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
10. **Port Kafka's tests where they encode tricky semantics.** Name the source in a comment: `// Ported from: core/src/test/scala/unit/kafka/cluster/PartitionTest.scala@4.4.0 testXyz`. A ported test is Kafka material, so its file also takes the header in [Attributing Kafka material](../../CONTRIBUTING.md#attributing-kafka-material) and is listed in `NOTICE` ([ADR-0008](../adr/0008-protocol-code-generated-from-kafka-schemas.md)).
11. **Every fixed fuzz crash adds a regression unit test** in the crate that owns the broken rule, and adds its input to `fuzz/seeds/<target>/`. Fuzzing finds a bug once, and the test keeps it fixed even when no fuzzer runs ([crash triage](../guides/debugging.md#fuzz-crashes)).

## Shared strategies

Proptest strategies for a crate's types live in that crate, in a `strategies` module behind its `proptest-strategies` feature. Other crates enable the feature on their dev-dependency: `alluvium-types = { workspace = true, features = ["proptest-strategies"] }`. They don't live in `alluvium-testkit`, because testkit depends on those crates: a dev-dependency back to testkit would compile two copies of the types, and values from one copy wouldn't type-check against the other. Strategies produce valid values only and favour boundaries (0, 1, the maximum, and the wire sentinels where those are valid).

Every strategy for a plain wire value is built from the boundary-bias helpers in `alluvium_types::strategies`, so a boundary bug in any field is found the same way:

| Helper | Values |
|---|---|
| `boundary_i8`, `boundary_i16`, `boundary_i32`, `boundary_i64` | Any value, half the time -1 (Kafka's usual "none" sentinel), 0, 1 or an extreme (and `i32::MAX` for `i64`) |
| `boundary_u16`, `boundary_u32` | Any value, half the time 0, 1, the signed maximum or the maximum |
| `non_negative_i32`, `non_negative_i64` | Values a count or id can hold, biased towards 0, 1, `i32::MAX` and the maximum |
| `string_up_to(max_bytes)`, `bytes_up_to(max_len)` | Empty, exactly the maximum length (for strings once ending in a multi-byte character), and anything in between |

`alluvium_types::strategies::composite` has strategies for values that must hold together, each tested against its invariant: ordered partition `watermarks`, an idempotent producer's `producer_batches` (checked against Kafka's `ProducerAppendInfo` rules), and `colliding_topic_partitions`. Where a test needs the broken case, an `invalid_` strategy breaks the invariant in one labelled way, such as `invalid_producer_batches` returning a `ProducerBatchViolation`.

`alluvium_types::strategies::adversarial` attacks decoders with raw bytes in Kafka's encodings:
- `lying_int32_length` and `lying_compact_length`: length prefixes larger than the body, negative, `i32::MAX`, `0x80000000` and `0xFFFFFFFF`;
- `truncated(valid)`: valid input cut short;
- `bad_varint(max_bytes)`: overlong, too long and unterminated varints, each labelled with its `VarintShape`;
- `nested(prefix, depth)`: nesting deeper than a limit.

Every decoder's property tests use them. A decoder must return an error or a value, never panic, and never reserve memory for a declared size it hasn't checked: a test in that module shows a toy decoder that trusts the declared length failing such a property.

Generated message strategies come from the code generator (PROTO-0081) and record batch strategies from REC-F14. Both are built from these helpers rather than from `any::<T>()`.

## State-machine property tests

A sans-I/O state machine is tested against a reference model: a few lines of plain Rust that state what the machine must do, without its optimizations.
- Generate a `Vec` of operations with weighted choices (`vec(prop_oneof![3 => …, 1 => …], 0..50)`).
- Apply each operation to both with `alluvium_types::strategies::state_machine::compare`. It compares what the two observe after every operation and fails at the first difference, naming the step.
- Operations aren't filtered by the current state: the machine must reject an operation that is invalid now, as the model does.

On failure:
- proptest shrinks the sequence to the shortest one of the simplest operations that still fails;
- its message prints the operations as `Debug` does, which for plain enums is valid Rust;
- the seed is recorded in `proptest-regressions/`.

Paste the operations into a unit test that calls `compare` on that slice, so the failure stays fixed (rule 1).

We don't use `proptest-state-machine`. Its state-dependent generation would let tests skip invalid operations, but those are exactly what a Kafka state machine must reject. It also turns on proptest's `fork` and `timeout` features for every crate's tests, adding about 35 crates to the lockfile. Plain proptest already shrinks a sequence of operations.

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

Benchmarks come in two kinds ([ADR-0019](../adr/0019-benchmark-regression-gating.md)). Both measure the same inputs, from a shared `benches/inputs/` module in the crate:

- **Gated benchmarks** count instructions under valgrind's callgrind, with gungraun.
  - They are each crate's `benches/<area>_instructions.rs`, and they cover hot paths.
  - Every pull request runs them at its base and at its head in one CI job (`bench`). It fails when a count grows by more than its threshold in [`benches/thresholds.toml`](../../benches/thresholds.toml): 3% unless an override, with its reason, says otherwise.
  - Counts are exact, so a failure is a real change in work done, never noise.
  - Run them with `cargo xtask bench [--base <ref>]`. They need Linux and valgrind; on macOS, use the devcontainer.
- **Wall-time benchmarks** use criterion.
  - They are each crate's `benches/<area>.rs`, run with `cargo bench -p <crate> --bench <area>`.
  - They are for local work and for the numbers a performance claim needs. CI doesn't run them: wall time on shared runners varies by tens of percent between machines.

Conventions for both:

- A gated benchmark function contains no loop: it calls one `#[inline(never)]` function that does the work. callgrind counts from each entry to the benchmark function, and takes a loop that jumps back to a function's first instruction for a recursive call. So a loop compiled into the benchmark function itself turns counting off and on, and most of the work goes uncounted.
- Benchmark ids are `<area>/<operation>/<case>`, such as `types/murmur2/size_4096`. In criterion: one group per operation, with the input size as the parameter. In gungraun: the group is the area, the function the operation, and the `#[bench::…]` id the case. A group name is unique across the workspace.
- Inputs are deterministic, drawn from `SeededRng::from_seed` with a fixed seed, so two runs measure the same work.
- Every criterion group declares its throughput (`Throughput::Bytes` or `Throughput::Elements`), so results compare across sizes and machines.
- A PR that claims a performance change includes the before and after numbers and the CPU they ran on.
- `criterion` is built without its default features: no plotting and no rayon, only `cargo bench` support.

## Coverage

Line coverage is tracked but not used as a target. The question for review is "which behaviors and failure modes are tested?", not "what percentage is covered?".
