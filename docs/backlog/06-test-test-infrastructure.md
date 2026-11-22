# TEST: Test infrastructure

> Crates: `alluvium-testkit`, `fuzz/`, `tests/compat/`, `benches/` · Kafka counterpart: Kafka's test utilities, system tests (ducktape) · Milestones: M1-M9

The tools that make behavior provable: deterministic simulation, fault injection, fuzzing, real-client compatibility tests, differential tests against Kafka 4.4.0, benchmarks, chaos and soak testing.

## [To Do] TEST-F01 Simulated clock and scheduler
P0 · M2 · Feature · deps: TYPE-F06
Scope: Deterministic simulated time driving timers for state machines and drivers.

### [To Do] TEST-0001 Create `alluvium-testkit` (dev layer) with `SimClock`: a `Clock` implementation built on `ManualClock` whose time moves only when the simulation scheduler advances it
P0 · M2 · Task · deps: none · labels: test · ref: ADR-0003
Done when: The crate `//!` doc covers responsibility, the dev layer ("only tests may depend on it"), the Kafka counterparts (`MockTime`, Trogdor) and the determinism invariants; `check-layers` rejects a normal (non-dev) dependency on it from any product crate; unit tests show that reading the clock never advances it.

### [To Do] TEST-0002 Spike: how simulation drives Tokio-based drivers (layer 4): a custom deterministic executor, `turmoil`, or Tokio's paused-time current-thread runtime, measured against the TYPE-F06 time-model ADR
P0 · M2 · Spike · deps: TEST-0001 · labels: test · ref: ADR-0003
Done when: An accepted ADR "Driver-level simulation" picks one approach for driver tests, shows a prototype test that is bit-for-bit reproducible across 100 runs with one seed, and states which tests stay at the pure state-machine level.

### [To Do] TEST-0003 Simulation scheduler: a bounded timer and event queue ordered by (deadline, insertion sequence), with `schedule`, `cancel` by token and `pop_next` that advances `SimClock` to the event's time
P0 · M2 · Task · deps: TEST-0001 · labels: test · ref: ADR-0003
Done when: A property test against a reference model (sorted vector) shows the pop order matches for arbitrary schedule/cancel sequences, including ties; scheduling past the capacity returns a typed error naming the capacity; time never moves backwards.

### [To Do] TEST-0004 Simulation run loop: `run_until(condition)`, `run_for(simulated duration)` and `step()`, with a step counter and a hard step limit that fails the run with the seed instead of looping forever
P0 · M2 · Task · deps: TEST-0003 · labels: test · ref: ADR-0003
Done when: Tests show `run_for` stops at the exact simulated time, a never-true condition stops at the step limit with a `StepLimitExceeded { steps, sim_time }` failure, and no real time passes (a test simulates 24 hours in under a second).

### [To Do] TEST-0005 Per-node clocks in the simulation: each node gets its own wall-clock offset and drift rate, and wall-clock jumps forwards or backwards can be scheduled as events, while monotonic time stays shared and never jumps
P1 · M2 · Task · deps: TEST-0004 · labels: test, nfr
Done when: Tests show two nodes reading different wall times at the same simulated instant, a scheduled 10-minute backwards jump visible only on the target node, and monotonic readings unaffected; NFR-F13's skew tests use this API.

### [To Do] TEST-0131 Simulation `Timer`: futures from `sleep_until` complete when the scheduler advances `SimClock` past their deadline (ADR-0011)
P0 · M2 · Task · deps: TEST-0003 · labels: test · ref: ADR-0011
Done when: Tests show a future stays pending until the scheduler reaches its deadline, futures with equal deadlines wake in registration order, and a dropped future's timer is cancelled without leaking a queue slot; it works with whichever executor TEST-0002 chooses.

## [To Do] TEST-F02 Simulated network
P0 · M3 · Feature · deps: TEST-F01
Scope: Message delivery with configurable drop, delay, duplication, reordering and partitions between nodes.

### [To Do] TEST-0006 `SimNetwork` core: addressed endpoints and connections with per-connection FIFO delivery (TCP semantics), a seeded latency distribution per link, and bounded in-flight capacity per connection
P0 · M3 · Task · deps: none · labels: test · ref: ADR-0003
Done when: With no faults, every message is delivered once and in order per connection (property test over random traffic); a full connection queue applies backpressure to the sender instead of growing; the same seed gives the same delivery schedule.

### [To Do] TEST-0007 Network fault: message loss per link at a configured probability, which on a TCP-like connection resets the connection so later messages are not delivered out of order
P0 · M3 · Task · deps: TEST-0006 · labels: test · ref: trogdor/src/main/java/org/apache/kafka/trogdor/fault/DegradedNetworkFaultSpec.java @4.4.0
Done when: Tests show that loss at probability 1.0 delivers nothing, that after a loss the connection is reset and both ends get a disconnect event, and that the loss count is exposed in the simulation's link statistics.

### [To Do] TEST-0008 Network fault: delay distributions per link (fixed, uniform, and long-tail with occasional spikes) within connection FIFO order
P0 · M3 · Task · deps: TEST-0006 · labels: test · ref: trogdor/src/main/java/org/apache/kafka/trogdor/fault/DegradedNetworkFaultSpec.java @4.4.0
Done when: A test with long-tail delay measures the simulated latency percentiles against the configured distribution; delay never reorders messages within one connection.

### [To Do] TEST-0009 Network fault: duplicate delivery at the message level, for state-machine tests that model retries (not TCP connections)
P1 · M3 · Task · deps: TEST-0006 · labels: test
Done when: Duplication is available only on message-level links, rejected with a typed error on connection-level links (TCP never duplicates); tests show a duplicated Raft-style message is delivered twice.

### [To Do] TEST-0010 Network fault: reordering across connections between the same nodes, and within a link for message-level links
P1 · M3 · Task · deps: TEST-0006 · labels: test
Done when: Tests show messages on two connections between the same pair can arrive in either order under a seed, while per-connection order is still never violated.

### [To Do] TEST-0011 Network partitions: symmetric splits into groups, one-way (asymmetric) partitions, isolating one node, and healing, all scheduled at simulated times
P0 · M3 · Task · deps: TEST-0006 · labels: test · ref: trogdor/src/main/java/org/apache/kafka/trogdor/fault/NetworkPartitionFaultSpec.java @4.4.0
Done when: Tests show that during a symmetric partition no message crosses groups, that a one-way partition blocks only one direction, that pending messages across a new partition are dropped with connection resets, and that healing restores delivery.

### [To Do] TEST-0012 Connection reset fault: drop a chosen connection, losing its in-flight messages and notifying both ends, as a scheduled event or at a random rate
P0 · M3 · Task · deps: TEST-0006 · labels: test
Done when: Tests show in-flight messages are lost, both endpoints observe the disconnect at the same simulated time, and a new connection can be opened afterwards.

### [To Do] TEST-0013 Slow links: a bytes-per-second limit per link that delays delivery in proportion to message size
P2 · M3 · Task · deps: TEST-0006 · labels: test, perf
Done when: A test sending 10 MB over a 1 MB/s simulated link takes 10 simulated seconds (within the latency settings); senders see backpressure once the connection's in-flight bound is reached.

### [To Do] TEST-0014 Network event trace: every send, delivery, drop, reset and partition change is recorded in the run's bounded trace with step, simulated time, link and message kind
P1 · M3 · Task · deps: TEST-0007, TEST-0011 · labels: test · ref: docs/standards/debuggability.md#reproduction
Done when: A failure report includes the last N network events (N configurable, default 200); the trace never contains message payloads, only kinds and sizes; a test checks the trace of a short partitioned run.

## [To Do] TEST-F03 Simulated disk with faults
P0 · M2 · Feature · deps: TEST-F01, STOR-F01
Scope: `fs` implementation with write failures, torn writes, lost unsynced data, slow I/O and full disk.

### [To Do] TEST-0015 `SimFs`: an in-memory implementation of the storage `fs` trait that models the page cache, keeping durable (fsynced) and volatile (unsynced) contents per file
P0 · M2 · Task · deps: none · labels: test · ref: ADR-0003
Done when: Writes become durable only after `fsync`; reads see volatile data; a property test against a simple reference model shows read-after-write correctness for random operation sequences; the STOR-F01 trait tests pass against it.

### [To Do] TEST-0016 Crash with lost unsynced data: `crash()` discards volatile state, either all unsynced writes or a seeded prefix of them per file, and the next "boot" sees only what survived
P0 · M2 · Task · deps: TEST-0015 · labels: test
Done when: Tests show both modes; with a fixed seed the surviving prefix is reproducible; a write that was fsynced is never lost; the mode is chosen by the fault plan.

### [To Do] TEST-0017 Directory durability: file creation, rename and delete are durable only after the parent directory is fsynced, so a crash can undo them
P0 · M2 · Task · deps: TEST-0015 · labels: test
Done when: Tests show that a rename without a directory fsync is undone by a crash and survives after one, matching the POSIX rules that checkpoint files rely on; the rule is documented in the `SimFs` module doc.

### [To Do] TEST-0018 Torn writes: at a crash, the last unsynced write of a file may persist only partially, at sector granularity (512 B or 4 KiB, configured), with the torn tail zeroed or filled with garbage
P0 · M2 · Task · deps: TEST-0016 · labels: test
Done when: Tests show each mode for writes that span sectors; the torn length is reproducible from the seed; storage recovery tests (STOR-F20) can request torn writes at a specific write.

### [To Do] TEST-0019 I/O errors: `EIO` on write, read, open or fsync, either on the Nth matching operation or at a probability, filtered by path pattern
P0 · M2 · Task · deps: TEST-0015 · labels: test · ref: trogdor/src/main/java/org/apache/kafka/trogdor/fault/FilesUnreadableFaultSpec.java @4.4.0
Done when: Tests show each operation type failing; a failed fsync drops the file's dirty data (as Linux does), so a later fsync succeeding does not make it durable; the error is visible to the caller as the trait's typed error.

### [To Do] TEST-0020 Disk full: a capacity limit in bytes per simulated disk, returning `ENOSPC` on writes and preallocation that would exceed it
P0 · M2 · Task · deps: TEST-0015 · labels: test
Done when: Tests show writes up to the limit succeed and the next one fails without a partial write, and that deleting files frees space; STOR-F25 disk-full tests use it.

### [To Do] TEST-0021 Slow I/O: per-operation latency (fixed, distribution or stall) through the simulation scheduler, so a slow fsync blocks the caller in simulated time
P1 · M2 · Task · deps: TEST-0015, STOR-F18 · labels: test, perf
Done when: Tests show a 2-second fsync stall delays completion by 2 simulated seconds and that other nodes keep running meanwhile; it relies on the storage I/O execution model (STOR-F18) completing operations asynchronously, as the doc states.

### [To Do] TEST-0022 Silent corruption: flip bits in durable data at a chosen file offset or at random, to prove CRC checks on recovery, follower fetch and `verify log` catch it
P1 · M2 · Task · deps: TEST-0015 · labels: test, security · ref: docs/standards/security.md#durability-as-a-security-property
Done when: Tests show a flipped bit is returned by reads exactly as stored; a fault plan can inject it after a crash; a storage recovery test uses it to show truncation at the first invalid batch.

### [To Do] TEST-0023 Crash-point enumeration: run a workload once to record every write and fsync point, then re-run it crashing at each point (and with a torn last write) and run a recovery check after each
P0 · M2 · Task · deps: TEST-0016, TEST-0018 · labels: test
Done when: The helper reports the point index and seed for any failing recovery check; a self-test with a deliberately non-atomic toy writer fails at the expected point; STOR-F20 uses this helper.

## [To Do] TEST-F04 Seeded runner and fault plans
P0 · M2 · Feature · deps: TEST-F01, TYPE-F07
Scope: Single-seed reproducibility, `ALLUVIUM_SIM_SEED`, printed replay command, declarative fault plans.

### [To Do] TEST-0024 Simulation randomness: one root seed, with deterministic named sub-streams for the network, the disk, each node and the workload, so adding randomness in one component doesn't change the others
P0 · M2 · Task · deps: none · labels: test · ref: ADR-0003
Done when: A test shows that adding draws to the disk stream leaves the network schedule unchanged for the same seed; streams use the TYPE-F07 RNG.

### [To Do] TEST-0025 Seed selection and printing: `ALLUVIUM_SIM_SEED` replays one seed; otherwise a seed is drawn from OS entropy through the TYPE-F07 sanctioned call; the seed is printed when the test starts and again on failure
P0 · M2 · Task · deps: TEST-0024 · labels: test · ref: docs/standards/testing.md#rules
Done when: The seed appears in the test output for passing runs (captured by nextest) and failing runs; an invalid `ALLUVIUM_SIM_SEED` fails with a clear message instead of picking a random seed.

### [To Do] TEST-0026 Replay command printing: a failing run prints the exact command `ALLUVIUM_SIM_SEED=<seed> cargo nextest run -p <crate> --test <binary> -- --exact <test path>`
P0 · M2 · Task · deps: TEST-0025 · labels: test, docs · ref: docs/standards/debuggability.md#reproduction
Done when: A deliberately failing self-test's printed command, when run, reproduces the same failure at the same step; the crate, binary and test path come from the test environment, not hand-written strings.

### [To Do] TEST-0027 Multi-seed runs: each simulation test runs a committed regression seed list (`tests/sim-seeds/<test>.txt`) plus N new random seeds (`ALLUVIUM_SIM_SEEDS`, default small on PRs), stopping at the first failure
P0 · M2 · Task · deps: TEST-0025 · labels: test · ref: ADR-0003
Done when: The regression list format and the procedure to add a failing seed are documented in testing.md; a self-test shows that a seed added to the list runs first; the runner prints how many seeds ran.

### [To Do] TEST-0028 Determinism check: run the same seed twice and compare hashes of the full event trace, failing with the first diverging step
P0 · M2 · Task · deps: TEST-0004, TEST-0024 · labels: test · ref: ADR-0003
Done when: A self-test with a toy component that iterates a `RandomState` hash map (behind a test-only switch) is detected as non-deterministic; CI runs the determinism check on every simulation test with two seeds per PR.

### [To Do] TEST-0029 Fault plan DSL: a declarative, validated `FaultPlan` with fault rates (network loss, delay, duplication, reordering; disk errors, torn writes, slow I/O) and scheduled events (partitions, heals, crashes, restarts, clock jumps)
P0 · M2 · Task · deps: TEST-0024 · labels: test · ref: ADR-0003
Done when: Plans can be written as Rust values and as TOML files with the same schema; validation rejects probabilities outside [0, 1], overlapping crash/restart of the same node, and events after the end of the run, naming the field; the plan is printed with the seed on failure.

### [To Do] TEST-0030 Standard fault plans: `calm`, `lossy_network`, `partitions`, `crash_restart`, `disk_faults` and `everything`, each documented with what it exercises
P0 · M3 · Task · deps: TEST-0029, TEST-0011, TEST-0016 · labels: test
Done when: Every plan runs against a toy replicated register in testkit's self-tests; testing.md lists the plans; simulation tests of RAFT, CTRL and REPL can reference them by name.

### [To Do] TEST-0031 Randomized fault plans ("swarm testing"): derive the set of enabled faults and their rates from the seed, so different seeds explore different fault mixes
P1 · M3 · Task · deps: TEST-0030 · labels: test · ref: ADR-0003
Done when: The derived plan is printed with the seed and is reproducible; a self-test shows a bug that needs two specific faults together is found within a bounded number of seeds that a fixed plan misses.

### [To Do] TEST-0032 Nightly simulation job: thousands of seeds per simulation test on a schedule, uploading failing seeds with their replay commands and opening or updating an issue per failing test
P0 · M3 · Task · deps: TEST-0027, TEST-0028 · labels: test · ref: ADR-0003
Done when: The workflow runs on a schedule only, with `issues: write` limited to this job; a planted failing seed shows up as an issue with the replay command; failing seeds are added to the regression lists by the fix PR (procedure in testing.md).

### [To Do] TEST-0033 Liveness check after faults: once the fault plan ends and the network heals, a test can require progress (for example a committed write) within a bounded simulated time
P1 · M3 · Task · deps: TEST-0004, TEST-0029 · labels: test
Done when: A self-test with a toy component that never recovers after a partition fails with `NoProgressAfterHeal { healed_at, deadline }` and the seed; the bound is configurable per test.

## [To Do] TEST-F05 Invariant checkers
P0 · M2 · Feature · deps: TEST-F04
Scope: Framework for per-step invariant checks and failure reports with flight recorders.

### [To Do] TEST-0034 Invariant checker framework: checkers registered with the simulation run after every step, receive a read-only view of the simulated system, and return a typed violation
P0 · M2 · Task · deps: none · labels: test · ref: ADR-0003
Done when: A self-test registers a checker that fails at step 42 and the run stops there with the checker's name and message; checkers cannot change state (read-only view type); the order of checkers is deterministic.

### [To Do] TEST-0035 Failure report: seed, replay command, step, simulated time, fault plan, violation, the last N trace events, and the flight recorders of the components the violation names
P0 · M2 · Task · deps: TEST-0034, OBS-F06 · labels: test, docs · ref: docs/standards/debuggability.md#flight-recorders
Done when: A self-test's report contains each section; flight recorders are rendered through OBS's formatter, so the report looks like `/debug` output; no payload bytes appear (payload fields are formatted only through `PayloadSummary`).

### [To Do] TEST-0036 Client operation history: simulated clients record invoke/ok/fail/unknown events with simulated timestamps (produce with acks and assigned offset, fetch with returned records, commit), as Jepsen histories do
P0 · M2 · Task · deps: TEST-0034 · labels: test
Done when: The history is bounded by run size (documented) and serialized as JSON lines on failure; a self-test shows that an operation timing out is recorded as `unknown`, not `fail`.

### [To Do] TEST-0037 History checker: no acknowledged `acks=all` write is lost; every such record is eventually readable at its acknowledged offset with the same key and value checksum
P0 · M3 · Task · deps: TEST-0036 · labels: test
Done when: A self-test with a toy log that drops an acknowledged write is caught with the partition, offset and acknowledgement step; `acks=0`/`acks=1` writes are excluded, as the durability matrix (NFR-F03) allows.

### [To Do] TEST-0038 History checkers: offsets in a partition are contiguous and never reused, and every consumer observes each partition's records in offset order without reading beyond what was committed to it
P0 · M3 · Task · deps: TEST-0036 · labels: test
Done when: Self-tests with toy logs that reuse an offset, skip one, or serve records out of order are each caught with the partition and offsets involved.

### [To Do] TEST-0039 Fault plan minimization: after a failure, repeatedly remove or weaken scheduled faults while the failure still reproduces with the same seed, and print the smallest plan found
P2 · M3 · Task · deps: TEST-0035, TEST-0029 · labels: test
Done when: A self-test whose failure needs one partition among ten scheduled events is reduced to that single event; minimization stops after a configured number of attempts and always prints the best plan so far.

### [To Do] TEST-0040 Checker self-test suite: a deliberately buggy toy replicated register in testkit's tests that every standard checker must catch under a named fault plan and seed
P0 · M2 · Task · deps: TEST-0035 · labels: test · ref: ADR-0003
Done when: The suite runs in CI; disabling any checker makes its self-test fail; testing.md explains that new checkers need a self-test with a planted bug.

## [To Do] TEST-F06 Simulated cluster builder
P0 · M3 · Feature · deps: TEST-F02, TEST-F05
Scope: Build N-node clusters with roles, crash and restart nodes, inspect state.

### [To Do] TEST-0041 `SimCluster` builder: N nodes with roles (broker, controller, combined), node ids, per-node config overrides, each on its own `SimFs` disk and simulated clock, connected by `SimNetwork`
P0 · M3 · Task · deps: SRV-F01, NET-F14 · labels: test · ref: ADR-0003
Done when: A test builds a 3-controller, 3-broker cluster and a 3-node combined cluster, runs until the metadata quorum has a leader, and reports per-node roles; configs go through the same validation as production (CFG).

### [To Do] TEST-0042 Node lifecycle in simulation: crash (lose volatile state, keep the disk subject to `SimFs` crash rules), restart from disk, graceful shutdown, pause/resume (a GC-pause-like stall), and adding or removing a node
P0 · M3 · Task · deps: TEST-0041 · labels: test · ref: trogdor/src/main/java/org/apache/kafka/trogdor/fault/ProcessStopFaultSpec.java @4.4.0
Done when: Tests show a crashed node restarts from its durable state and rejoins; a paused node's timers fire late after resume; all lifecycle events are in the trace and can be scheduled by fault plans.

### [To Do] TEST-0043 State inspection API for checkers: per node, the metadata image version, partition replication state (leader, epoch, ISR, high watermark, log end offset), Raft quorum state and flight recorders, read through the same views the diagnostics endpoint uses
P0 · M3 · Task · deps: TEST-0041 · labels: test
Done when: A test reads each view after a leader change; inspection never mutates state or advances time; the views are the read-only types checkers receive.

### [To Do] TEST-0044 Simulated Kafka clients: producer (acks 0/1/all, retries, idempotence flag), consumer (assign and fetch) and admin (create topic) that speak the real protocol codecs over `SimNetwork` and record histories
P0 · M3 · Task · deps: TEST-0041, TEST-0036 · labels: test
Done when: A test produces 1,000 records with `acks=all` and consumes them back with the history checkers passing; the clients follow Kafka's retry and metadata-refresh behavior on `NOT_LEADER_OR_FOLLOWER` (cited); they never block on real time.

### [To Do] TEST-0045 Workload generators: steady produce/consume, bursty produce, topic creation and deletion churn, and partition reassignment, parameterized by the seed
P1 · M3 · Task · deps: TEST-0044 · labels: test
Done when: Each workload runs for 10 simulated minutes on a 3-node cluster under the `calm` plan with checkers passing; workload parameters are printed with the seed.

### [To Do] TEST-0046 Guide `docs/guides/simulation-testing.md`: writing a simulation test, choosing a fault plan, adding a checker, reading a failure report and replaying a seed, with one complete example test from the repository
P1 · M3 · Task · deps: TEST-0044, TEST-0030 · labels: docs, test · ref: docs/standards/testing.md#simulation-harness-alluvium-testkit
Done when: The example in the guide is a real test that runs in CI (the guide links to it, it doesn't copy it); testing.md and debugging.md link to the guide.

### [To Do] TEST-0047 Simulator throughput benchmark: simulated steps per second and simulated-time-to-wall-time ratio for a 3-node cluster workload, tracked by the benchmark job
P1 · M3 · Task · deps: TEST-0045 · labels: test, perf
Done when: The criterion benchmark result is recorded in the PR; the nightly seed count is derived from it so the nightly job fits its time budget (stated in the nightly workflow comment).

## [In Progress] TEST-F07 Fuzzing infrastructure
P0 · M1 · Feature · deps: FND-F06
Scope: cargo-fuzz layout, corpus management, short CI runs per PR, nightly long runs, crash triage workflow.

### [Done] TEST-0048 cargo-fuzz layout in `fuzz/` (its own workspace, nightly toolchain pinned by date in `fuzz/rust-toolchain.toml`) with the first targets `types_uuid_parse` and `types_topic_name`
P0 · M1 · Task · deps: TYPE-F03, TYPE-F04 · labels: test, security, decoder · ref: docs/standards/security.md#untrusted-input
Done when: `cargo +nightly fuzz run types_uuid_parse -- -max_total_time=60` runs clean; each target asserts no panic and, when parsing succeeds, that display then parse round-trips; target naming (`<crate>_<decoder>`) and the assertion pattern are documented in `fuzz/README.md`; the main workspace excludes `fuzz/`.

### [Done] TEST-0049 Fuzzing resource limits: every run uses `-rss_limit_mb`, `-malloc_limit_mb`, `-timeout` and `-max_len` values from one table, so unbounded allocation and hangs are reported as crashes
P0 · M1 · Task · deps: TEST-0048 · labels: test, security · ref: docs/standards/security.md#untrusted-input
Done when: A planted target that allocates from an untrusted length (test-only, not committed) is reported as a malloc-limit crash; the limits and why they were chosen are documented in `fuzz/README.md`.

### [Done] TEST-0050 `cargo xtask fuzz <target> [--time 60s]` and `cargo xtask fuzz --list`, wrapping cargo-fuzz (version pinned in the xtask tool table) with the standard limits
P0 · M1 · Task · deps: TEST-0049 · labels: test · ref: docs/standards/rust.md#tooling-commands-contract-implemented-in-m0
Done when: The command fails with the target list on an unknown target, and with the install hint when cargo-fuzz or the nightly toolchain is missing; the rust.md tooling table entry matches the behavior.

### [Done] TEST-0051 Spike: fuzz corpus storage and CI fuzzing service: a separate corpus repository, ClusterFuzzLite with its storage, or GitHub Actions cache plus artifacts; also which seed inputs live in this repo
P0 · M1 · Spike · deps: TEST-0048 · labels: test
Done when: An accepted ADR "Fuzz corpus storage" names where corpora live, how CI reads and writes them with least-privilege tokens, retention, and a size budget; `.gitignore`'s `/fuzz/corpus/` entry is kept or changed accordingly.

### [In Review] TEST-0052 Corpus management per ADR-0018: `cargo xtask fuzz corpus pull`, committed seed inputs under `fuzz/seeds/<target>/`, and weekly corpus minimization (`cargo xtask fuzz corpus minimize`, libFuzzer `-merge=1`)
P0 · M1 · Task · deps: TEST-0051, TEST-0050 · labels: test
Done when: A fresh clone can pull the corpus and run a target from it; the minimization job reduces the corpus without losing coverage (libFuzzer's coverage and feature counts before and after, from the same binary, in the job log). Only CI writes corpora (ADR-0018), so there is no push command.

### [In Review] TEST-0053 PR fuzzing job: every fuzz target runs for 60 s per PR (targets sharded across a matrix), starting from the stored corpus, failing the PR on a crash with the reproducer uploaded
P0 · M1 · Task · deps: TEST-0050, TEST-0052 · labels: test, security · ref: docs/standards/security.md#untrusted-input
Done when: A planted crash in a PR fails the job and the artifact reproduces it with `cargo xtask fuzz <target> <artifact>`; the job's total wall time is reported in the PR description.

### [To Do] TEST-0054 Nightly fuzzing job: each target for one hour (configurable), saving the grown corpus and opening or updating one issue per crashing target with the minimized reproducer
P0 · M1 · Task · deps: TEST-0053 · labels: test, security · ref: docs/standards/security.md#untrusted-input
Done when: The job runs only on schedule, with write permissions limited to corpus storage and issues; a planted crash creates one issue (not one per run) with the reproducer and the `cargo xtask fuzz` command.

### [To Do] TEST-0055 Crash triage workflow in docs/guides/debugging.md: minimize with `cargo fuzz tmin`, reproduce, write the failing unit test in the owning crate (named after the crash), fix, and add the input to `fuzz/seeds/`
P0 · M1 · Task · deps: TEST-0054 · labels: test, docs · ref: docs/guides/debugging.md
Done when: The guide walks through one real or planted crash end to end; the issue template from FND-F11 links to it; the rule "every fixed fuzz crash adds a regression unit test" is in testing.md.

## [To Do] TEST-F08 Compatibility test harness
P0 · M2 · Feature · deps: FND-F06
Scope: Docker-based harness with pinned images: Java client 4.4.0 test app, librdkafka, franz-go, `kafka-*.sh` tools.

### [To Do] TEST-0056 Spike: compatibility harness architecture ADR
P0 · M2 · Spike · deps: none · labels: test · ref: docs/standards/testing.md#compatibility-and-differential-tests
Done when: Scope: orchestration (xtask driving Docker Compose, or testcontainers from Rust tests), how Alluvium runs (locally built image or host binary), networking and port allocation without fixed ports, and one line-based JSON command protocol shared by all client apps. An accepted ADR "Compatibility harness" records these choices, with a prototype that runs one produce/consume scenario against Kafka 4.4.0 in a container from `cargo nextest`.

### [To Do] TEST-0057 Container image pinning: `tests/compat/images.lock` lists every image as name, tag and `sha256` digest; the harness refuses an image reference without a digest; Dependabot or a scheduled job proposes digest updates
P0 · M2 · Task · deps: TEST-0056 · labels: test, security · ref: docs/standards/testing.md#compatibility-and-differential-tests
Done when: A test shows an unpinned reference is rejected with the image name; each pin in the file has a comment saying what the image is used for; the update procedure is documented in `tests/compat/README.md`.

### [To Do] TEST-0058 Harness runner crate: starts the containers and an Alluvium node per test with temporary directories and dynamically allocated ports, sends commands to client apps over the JSON protocol, and collects results as typed values
P0 · M2 · Task · deps: TEST-0056, TEST-0057 · labels: test
Done when: Tests run in parallel under nextest without port clashes (two concurrent runs in CI); a crashed container fails the test with its exit code and last log lines; every wait has a timeout.

### [To Do] TEST-0059 Kafka 4.4.0 target: start Apache Kafka 4.4.0 (official image, pinned digest) as a single combined KRaft node in the harness, so any scenario can run against Kafka as well as Alluvium
P0 · M2 · Task · deps: TEST-0058 · labels: test
Done when: The harness's own smoke scenario (create topic, produce, consume) passes against Kafka 4.4.0 for every client app, proving the harness independently of Alluvium; Kafka's config for tests is in one file with comments.

### [To Do] TEST-0060 Java client test app (kafka-clients 4.4.0) with produce, consume and admin commands
P0 · M2 · Task · deps: TEST-0058 · labels: test · ref: clients/src/main/java/org/apache/kafka/clients/producer/KafkaProducer.java @4.4.0
Done when: The build is pinned with a Gradle lockfile and the image by digest; commands: produce (acks, compression, headers, idempotence flag), consume (assign, and subscribe with the classic protocol), admin (create, delete and describe topics, describe cluster). The app passes the smoke scenario against Kafka 4.4.0; every command reports Kafka exceptions as structured errors (class name and error code), never as process crashes; the command protocol is documented in `tests/compat/PROTOCOL.md`.

### [To Do] TEST-0061 Java test app: consumer group commands for the classic and consumer (KIP-848) protocols: subscribe, poll loops with rebalance callbacks recorded, commit sync/async, seek, and static membership
P0 · M5 · Task · deps: TEST-0060 · labels: test · ref: clients/src/main/java/org/apache/kafka/clients/consumer/KafkaConsumer.java @4.4.0
Done when: Group scenarios (two members joining, one leaving) pass against Kafka 4.4.0 with the rebalance events recorded in the result; the new commands are documented in `PROTOCOL.md`.

### [To Do] TEST-0062 Java test app: transactional producer commands (init, begin, send, send offsets to transaction, commit, abort) and `read_committed` consumption
P0 · M6 · Task · deps: TEST-0060 · labels: test · ref: clients/src/main/java/org/apache/kafka/clients/producer/KafkaProducer.java @4.4.0
Done when: A transactional scenario (one committed and one aborted transaction) passes against Kafka 4.4.0, with the `read_committed` consumer seeing only committed records; the commands are documented in `PROTOCOL.md`.

### [To Do] TEST-0063 Java test app: share consumer commands (subscribe, poll, acknowledge with accept/release/reject) for share-group compatibility tests
P1 · M8 · Task · deps: TEST-0061 · labels: test · ref: clients/src/main/java/org/apache/kafka/clients/consumer/KafkaShareConsumer.java @4.4.0
Done when: A share-group scenario with two consumers passes against Kafka 4.4.0; the commands are documented in `PROTOCOL.md`.

### [To Do] TEST-0064 librdkafka test app via a pinned image (confluent-kafka-python whose bundled librdkafka is the current stable release, or a small C program against a pinned librdkafka build) implementing the same JSON commands for produce, consume and admin
P0 · M2 · Task · deps: TEST-0058 · labels: test
Done when: The app passes the smoke scenario against Kafka 4.4.0; the librdkafka version is printed in every result and recorded in `images.lock`; unsupported commands return a structured "unsupported" error.

### [To Do] TEST-0065 franz-go test app (Go module versions pinned in `go.sum`, image pinned) implementing the same JSON commands for produce, consume and admin
P0 · M2 · Task · deps: TEST-0058 · labels: test
Done when: The app passes the smoke scenario against Kafka 4.4.0; the franz-go version is printed in every result; unsupported commands return a structured "unsupported" error.

### [To Do] TEST-0066 `kafka-*.sh` runner: run any Kafka 4.4.0 CLI tool from the pinned Kafka image against a target, capturing exit code, stdout and stderr, with output normalization for host names, ports and generated ids
P0 · M2 · Task · deps: TEST-0059 · labels: test
Done when: `kafka-topics.sh --create/--describe/--list` and `kafka-broker-api-versions.sh` run against Kafka 4.4.0 with normalized output stored as goldens; CLI-F06 adds one test per tool on top of this runner.

### [To Do] TEST-0067 Alluvium target: build the `alluvium` binary into a local test image (not the release image) and start it as a single node in the harness, with its logs and `/debug` output captured
P0 · M2 · Task · deps: TEST-0058, CLI-F01 · labels: test
Done when: The image is built from the CLI-F01 binary and the harness starts it with the `alluvium server` invocation that CLI-F02 defines (depending on CLI-F02 directly would create a cycle, because SRV and NET compatibility tests depend on TEST-F08); until that command exists the target fails at startup with the binary's own error, and the smoke scenario may fail until the APIs exist; the harness itself is proven by the run against Kafka; the test image build is cached in CI.

### [To Do] TEST-0068 `cargo xtask compat [--client java|librdkafka|franz-go|tools] [--target alluvium|kafka] [--filter <pattern>]`
P0 · M2 · Task · deps: TEST-0067 · labels: test · ref: docs/standards/rust.md#tooling-commands-contract-implemented-in-m0
Done when: The command checks Docker is available (failing with the `doctor` hint) and runs the selected suites; `--target kafka` runs the same scenarios against Kafka 4.4.0; the rust.md tooling table matches.

### [To Do] TEST-0069 Failure artifacts: when a compatibility test fails, collect Alluvium logs, flight recorders from `/debug`, client app output and container logs into one directory per test
P0 · M2 · Task · deps: TEST-0067 · labels: test, docs · ref: docs/standards/debuggability.md
Done when: A planted failing test produces the directory with every item; CI uploads it as an artifact; a test scans the collected files and finds none of the scenario's record values.

### [To Do] TEST-0070 CI compatibility job: on every PR on Linux, run `cargo xtask compat` for every client against Alluvium, with Docker layer and app build caches
P0 · M2 · Task · deps: TEST-0068, TEST-0069 · labels: test
Done when: The job is added to `ci-success`'s `needs:`; its run time is in the PR description; images are pulled by digest only.

### [To Do] TEST-0071 Guide `tests/compat/README.md`: adding a scenario, adding a command to the client apps (all apps or an explicit "unsupported"), running against Kafka first, updating image pins
P1 · M2 · Task · deps: TEST-0070 · labels: docs · ref: docs/standards/testing.md#compatibility-and-differential-tests
Done when: testing.md links to the guide; the guide's example scenario is a real test in the suite.

## [To Do] TEST-F09 Differential test harness
P0 · M2 · Feature · deps: TEST-F08
Scope: Run the same scenario against Kafka 4.4.0 and Alluvium; normalize and diff responses with documented ignore rules.

### [To Do] TEST-0072 Differential scenario DSL: a Rust builder for a sequence of steps (client app commands and raw protocol requests), run the same way against Kafka 4.4.0 and Alluvium, with every step's response captured as structured data
P0 · M2 · Task · deps: none · labels: test · ref: docs/standards/testing.md#compatibility-and-differential-tests
Done when: A scenario can mix client commands and raw requests; each captured response records the API, version and correlation id; a scenario run against Kafka twice gives identical captures after normalization.

### [To Do] TEST-0073 Raw protocol driver: send exact request structs at a chosen API version over TCP and decode the responses with the generated codecs, for behaviors clients never trigger (old versions, invalid fields, edge cases)
P0 · M2 · Task · deps: TEST-0072, PROTO-F06 · labels: test, decoder
Done when: The driver can send every request type at every version the codec supports; undecodable responses are captured as raw bytes with the decode error, not dropped; every read has a timeout.

### [To Do] TEST-0074 Dual-target runner: start Kafka 4.4.0 and Alluvium with equivalent configuration (translation table in one file), run the scenario against both, and produce a pair of captures
P0 · M2 · Task · deps: TEST-0072 · labels: test
Done when: The configuration translation table is a single file with a comment per entry; both targets start with fresh state per scenario; a scenario failure on either target is reported with that target named.

### [To Do] TEST-0075 Response normalizer: field-path rules that replace values which legitimately differ (node ids, cluster id, topic ids, host names and ports, timestamps, throttle times) with placeholders, applied consistently so the same value maps to the same placeholder
P0 · M2 · Task · deps: TEST-0074 · labels: test
Done when: Identity-preserving placeholders keep relationships (a topic id seen twice maps to the same placeholder both times); unit tests cover each rule; leader epochs, offsets and error codes are never normalized.

### [To Do] TEST-0076 Ignore rules file and documentation: `tests/compat/differential/ignore-rules.toml` (API, version range, field path, reason, optional ADR) plus `IGNORE.md` explaining the policy (no whole-response ignores; every deviation needs an ADR)
P0 · M2 · Task · deps: TEST-0075 · labels: test, docs · ref: docs/standards/testing.md#compatibility-and-differential-tests
Done when: The runner refuses a rule without a reason or with a path that matches nothing in any scenario (unused rule); IGNORE.md is the single explanation, and the rules file links to it.

### [To Do] TEST-0077 Diff report: for each mismatch, the scenario step, API and version, field path, both values, and the request that produced it, in readable text and JSON
P0 · M2 · Task · deps: TEST-0076 · labels: test
Done when: A planted mismatch (an error code changed in a test-only build) produces a report naming the exact field; the report is uploaded as a CI artifact.

### [To Do] TEST-0078 Differential self-check: run every scenario Kafka-vs-Kafka (two fresh Kafka 4.4.0 nodes), which must give no differences after normalization
P0 · M2 · Task · deps: TEST-0077 · labels: test
Done when: A failure here means the normalizer misses a legitimately varying field or a scenario is non-deterministic; the self-check runs in the nightly job; it passes on the initial scenario set.

### [To Do] TEST-0079 Version sweep: run a raw-request scenario at every version in the API's advertised range, as testing.md rule 9 requires
P0 · M2 · Task · deps: TEST-0073, TEST-0077 · labels: test · ref: docs/standards/testing.md#rules
Done when: A scenario declares its API and the runner expands it over the range read from Alluvium's `ApiVersions`; a version advertised by Alluvium but missing from Kafka 4.4.0 fails the sweep.

### [To Do] TEST-0080 Multi-node differential runs: three-node Kafka 4.4.0 and three-node Alluvium clusters for replication, leadership and reassignment scenarios
P0 · M3 · Task · deps: TEST-0074 · labels: test
Done when: A scenario that stops the leader of a partition gives matching normalized results (new leader epoch increments, ISR changes and error codes seen by clients) on both clusters; node ids are normalized by role, not by number.

### [To Do] TEST-0081 CI differential job: run the differential suite on PRs (Linux) and the full suite with the self-check nightly
P0 · M2 · Task · deps: TEST-0077 · labels: test
Done when: The PR job is in `ci-success`'s `needs:`; mismatch reports are uploaded as artifacts; total run time is reported in the PR.

## [To Do] TEST-F10 Kafka test porting program
P1 · M2 · Feature · deps: none
Scope: Inventory Kafka's unit and integration tests per area and track which encode semantics we must port.

### [To Do] TEST-0082 Kafka test porting inventory format and location
P1 · M2 · Task · deps: none · labels: test, docs · ref: docs/standards/testing.md#rules
Done when: Each module gets `docs/testing/kafka-test-inventory/<module>.md` with one row per test class: path at 4.4.0, area, verdict (port / covered by differential / covered by simulation / not applicable, with a reason) and the owning backlog feature. The format is described in testing.md next to rule 10; the `// Ported from:` comment format links to it; a sample row for `TopicTest` (ported in TYPE-F04) is included.

### [To Do] TEST-0083 Inventory of the `clients` module's broker-relevant tests: `common/record`, `common/protocol`, `common/message`, `common/requests`, `common/compress`, `common/internals`, `common/utils` and `common/security`
P1 · M2 · Task · deps: TEST-0082 · labels: test · ref: clients/src/test/java/org/apache/kafka/common @4.4.0
Done when: Every test class under those packages at tag 4.4.0 has a row with a verdict; classes marked "port" are linked to their PROTO/REC/SEC/TYPE features; the file is complete when a script listing the classes finds no unlisted one.

### [To Do] TEST-0084 Inventory of the `storage` module tests (log, segments, indexes, recovery, cleaner, producer state, remote storage)
P1 · M2 · Task · deps: TEST-0082 · labels: test · ref: storage/src/test @4.4.0
Done when: Every test class at tag 4.4.0 has a verdict and an owning STOR or TIER feature; tests needing simulation (crash, recovery) are marked for STOR-F20.

### [To Do] TEST-0085 Inventory of the `raft` module tests, including the `KafkaRaftClient*Test` classes and `raft/src/test/java/org/apache/kafka/raft/RaftEventSimulationTest.java`
P1 · M3 · Task · deps: TEST-0082 · labels: test · ref: raft/src/test @4.4.0
Done when: Every test class has a verdict; Kafka's own simulation scenarios are mapped to RAFT-F13 simulation tests or fault plans.

### [To Do] TEST-0086 Inventory of the `metadata` module tests (image, delta, replay, controller: `QuorumControllerTest`, `ReplicationControlManagerTest` and related)
P1 · M2 · Task · deps: TEST-0082 · labels: test · ref: metadata/src/test @4.4.0
Done when: Every test class has a verdict and an owning META or CTRL feature.

### [To Do] TEST-0087 Inventory of `core` unit tests (`unit/kafka/server`, `unit/kafka/cluster`, `unit/kafka/network`, `unit/kafka/coordinator`) and integration tests (`integration/kafka/api`, `integration/kafka/server`)
P1 · M2 · Task · deps: TEST-0082 · labels: test · ref: core/src/test/scala/unit/kafka @4.4.0
Done when: Every test class has a verdict; integration tests are mapped to compatibility or differential scenarios where they test client-visible behavior; `PartitionTest` and `ReplicaManagerTest` rows list the specific cases to port.

### [To Do] TEST-0088 Inventory of the `server` and `server-common` module tests
P1 · M2 · Task · deps: TEST-0082 · labels: test · ref: server/src/test @4.4.0
Done when: Every test class has a verdict and an owning feature.

### [To Do] TEST-0089 Inventory of the `group-coordinator` and `coordinator-common` module tests
P1 · M5 · Task · deps: TEST-0082 · labels: test · ref: group-coordinator/src/test @4.4.0
Done when: Every test class has a verdict and an owning GRP feature; `GroupMetadataManagerTest` rows list the specific cases to port.

### [To Do] TEST-0090 Inventory of the `transaction-coordinator` module tests and `core/src/test/scala/unit/kafka/coordinator/transaction`
P1 · M6 · Task · deps: TEST-0082 · labels: test · ref: transaction-coordinator/src/test @4.4.0
Done when: Every test class has a verdict and an owning TXN feature.

### [To Do] TEST-0091 Inventory of the `share-coordinator` module tests and the share-group tests in `core/src/test/java/kafka/server/share`
P1 · M8 · Task · deps: TEST-0082 · labels: test · ref: share-coordinator/src/test @4.4.0
Done when: Every test class has a verdict and an owning SHARE feature.

### [To Do] TEST-0092 Inventory of the `tools` module tests, mapped to CLI-F06 tool compatibility tests
P1 · M7 · Task · deps: TEST-0082 · labels: test · ref: tools/src/test @4.4.0
Done when: Every test class has a verdict; behaviors that depend on broker responses are mapped to `kafka-*.sh` runner scenarios.

### [To Do] TEST-0093 Porting progress report: `cargo xtask test-inventory` summarizes verdicts per module and lists "port" rows whose `// Ported from:` comment is not found in the code yet
P2 · M2 · Task · deps: TEST-0082, FND-F04 · labels: test
Done when: The report runs in CI as information only (not a gate); a unit test on fixture inventories and source files checks the counts.

## [In Progress] TEST-F11 Benchmark infrastructure
P1 · M0 · Feature · deps: FND-F06
Scope: Criterion micro-benchmarks, benchmark CI job with regression thresholds, macro benchmarks (OpenMessaging Benchmark, perf-test tools).

### [Done] TEST-0094 Criterion conventions and the first benchmark: `benches/` per crate with `harness = false`, deterministic inputs, throughput annotations and naming `<area>/<operation>/<size>`, starting with `alluvium-types` murmur2 over 16 B-64 KiB keys
P1 · M0 · Task · deps: TYPE-F11 · labels: perf, test · ref: docs/standards/rust.md#performance
Done when: `cargo bench -p alluvium-types` runs the benchmark; the conventions are in testing.md; `criterion` is added to `[workspace.dependencies]` with a justification; the murmur2 result is in the PR.
Notes: Pulled into M0 because TYPE-0035 (M0) needs its benchmark budget measured with criterion, and benchmarks should follow conventions from the first one.

### [To Do] TEST-0095 Spike: benchmark regression gating: wall-time criterion runs on shared runners (noisy), a dedicated runner, or instruction counts (for example iai-callgrind) for the gate, with criterion kept for local work
P1 · M1 · Spike · deps: TEST-0094 · labels: perf
Done when: An accepted ADR "Benchmark regression gating" picks the method, the regression threshold and its statistical basis, measured on 20 repeated runs of the murmur2 benchmark on the chosen runner.

### [To Do] TEST-0096 Benchmark CI job per the ADR: run benchmarks on the PR and on its base commit in the same job, compare, and fail when a benchmark regresses beyond its threshold
P1 · M1 · Task · deps: TEST-0095 · labels: perf
Done when: Thresholds live in `benches/thresholds.toml` (default plus per-benchmark overrides with a reason); a planted slowdown fails the job with the benchmark name and both results; the comparison is written to the job summary.

### [To Do] TEST-0097 Benchmark history: store main-branch results per commit (as the ADR decides) and plot trends per benchmark
P2 · M2 · Task · deps: TEST-0096 · labels: perf
Done when: History for the last 90 days of main is viewable; the storage needs no write access from PR jobs.

### [To Do] TEST-0098 Macro benchmark environment spec `docs/benchmarks.md`: reference hardware, OS and kernel settings, file system, JVM settings for Kafka, warm-up and run length, and how to capture flame graphs
P1 · M2 · Task · deps: TEST-0094 · labels: perf, docs
Done when: The spec is precise enough to reproduce a run on rented hardware; NFR-F01's workloads refer to it instead of repeating it.

### [To Do] TEST-0099 Kafka perf-test runner: run `kafka-producer-perf-test.sh`, `kafka-consumer-perf-test.sh` and `EndToEndLatency` from the pinned Kafka 4.4.0 image against Alluvium or Kafka, and parse their output into structured results
P1 · M2 · Task · deps: TEST-0098, TEST-0066 · labels: perf · ref: tools/src/main/java/org/apache/kafka/tools/ProducerPerformance.java @4.4.0
Done when: `cargo xtask bench macro --tool producer-perf --target kafka|alluvium` produces JSON with throughput and latency percentiles; parsing is unit-tested on captured outputs from Kafka 4.4.0.

### [To Do] TEST-0100 OpenMessaging Benchmark setup: pinned OMB version with its Kafka driver, driver configs for Alluvium and Kafka 4.4.0, and the NFR-F01 standard workloads as OMB workload files in `benches/omb/`
P1 · M2 · Task · deps: TEST-0098 · labels: perf
Done when: One standard workload runs end to end against both targets on the reference environment, with results stored as JSON plus a short report; the workload files are the single definition NFR-F01 refers to.

### [To Do] TEST-0101 Side-by-side report: run the same macro workload against Kafka 4.4.0 and Alluvium on the same hardware and produce a comparison table (throughput, latency percentiles, CPU, memory)
P1 · M2 · Task · deps: TEST-0099, TEST-0100 · labels: perf
Done when: `cargo xtask bench compare` produces the table from two result files; the first report is attached to NFR-F01's measurement task.

## [To Do] TEST-F12 Chaos testing
P1 · M3 · Feature · deps: TEST-F08
Scope: Multi-process clusters under real network and process faults; Jepsen-style no-loss and ordering checks.

### [To Do] TEST-0102 Multi-process chaos cluster: start N Alluvium nodes (and the client apps) in containers on a dedicated Docker network from the compatibility harness, with each node's data directory on a volume that survives restarts
P1 · M3 · Task · deps: none · labels: test
Done when: A 3-node cluster starts, serves a produce/consume smoke scenario, and is torn down with all logs collected; the cluster config is generated from one template.

### [To Do] TEST-0103 Process faults: SIGKILL, SIGSTOP/SIGCONT (pause) and graceful restart of chosen nodes, scheduled from a seeded chaos plan
P1 · M3 · Task · deps: TEST-0102 · labels: test · ref: trogdor/src/main/java/org/apache/kafka/trogdor/fault/ProcessStopFaultSpec.java @4.4.0
Done when: Each fault is logged with its wall-clock time and target; a test kills the partition leader and the workload continues after election; the plan and seed are printed for replay (best effort, since real processes aren't deterministic).

### [To Do] TEST-0104 Network degradation with `tc netem` inside node containers (with `NET_ADMIN`): delay with jitter, loss, duplication and reordering per node
P1 · M3 · Task · deps: TEST-0102 · labels: test · ref: trogdor/src/main/java/org/apache/kafka/trogdor/fault/DegradedNetworkFaultSpec.java @4.4.0
Done when: A test applies 100 ms ± 20 ms delay to one node and measures it from a client; faults are always removed at the end of a test, even on failure.

### [To Do] TEST-0105 Network partitions between nodes with `iptables` rules (symmetric, one-way and node isolation), plus Toxiproxy between clients and brokers for per-connection faults (latency, bandwidth, timeouts, reset_peer)
P1 · M3 · Task · deps: TEST-0102 · labels: test · ref: trogdor/src/main/java/org/apache/kafka/trogdor/fault/NetworkPartitionFaultSpec.java @4.4.0
Done when: A test isolates the controller leader and observes a new quorum leader; Toxiproxy (version pinned in `images.lock`) is controlled through its API from the harness; all rules are removed on teardown.

### [To Do] TEST-0106 Clock faults for real processes: run a node with a wall-clock offset or jump (libfaketime in the node image) for NFR-F13's skew tests
P2 · M3 · Task · deps: TEST-0102 · labels: test, nfr
Done when: A test starts a node 5 minutes ahead and the cluster keeps working; the faketime library version is pinned in the image.

### [To Do] TEST-0107 Jepsen-style history for real clients: client apps record invoke/ok/fail/info events with wall-clock timestamps into JSON lines, using the same format as the simulation history
P1 · M3 · Task · deps: TEST-0102 · labels: test
Done when: The history from a 10-minute chaos run is collected as an artifact; the format is shared with TEST-F05 so the same checkers run on both.

### [To Do] TEST-0108 Run the no-lost-acknowledged-write and per-partition ordering checkers on chaos histories, treating `info` (indeterminate) operations as possibly applied
P1 · M3 · Task · deps: TEST-0107, TEST-0037, TEST-0038 · labels: test
Done when: A planted bug (a test build that acknowledges before replicating) is caught in a chaos run with process kills; the checker output names the lost records with offsets.

### [To Do] TEST-0109 Transaction checkers for chaos histories: no duplicates with idempotence, no aborted or uncommitted records seen by `read_committed` consumers, and atomic visibility of each transaction
P1 · M6 · Task · deps: TEST-0108, TEST-0062 · labels: test
Done when: Planted bugs (a test build that exposes aborted records) are caught; the checkers also run on simulation histories.

### [To Do] TEST-0110 Chaos run command: `cargo xtask chaos --plan <file> --duration <d> --workload <name>` running a seeded fault schedule, the workload and the checkers, then writing a report
P1 · M3 · Task · deps: TEST-0103, TEST-0104, TEST-0105, TEST-0108 · labels: test
Done when: The report lists the faults injected, the checker results and the availability timeline; a nightly job runs a 30-minute chaos run on a runner with enough resources, opening an issue on failure.

### [To Do] TEST-0111 Spike: whether to also run the upstream Jepsen Kafka test suite against Alluvium as an independent check
P3 · M6 · Spike · deps: TEST-0110 · labels: test
Done when: An accepted ADR records the effort, what Jepsen would cover beyond our checkers, and the decision.

## [To Do] TEST-F13 Soak and endurance testing
P1 · M9 · Feature · deps: TEST-F12
Scope: Multi-day runs checking memory, file-descriptor and latency stability.

### [To Do] TEST-0112 Soak environment and workload: a multi-node cluster on reference hardware running a mixed workload (produce/consume at a fixed rate, consumer groups, transactions, topic churn, periodic rolling restarts) for at least 72 hours
P1 · M9 · Task · deps: none · labels: test, nfr
Done when: The environment can be recreated from scripts in the repository; the workload definition is versioned with the code; one 72-hour run completes and its report is linked.

### [To Do] TEST-0113 Soak metric capture and stability assertions: RSS, open file descriptors, threads and supervised task counts, request latency percentiles and error rates sampled over the run, with assertions on trend (slope after warm-up) instead of absolute values
P1 · M9 · Task · deps: TEST-0112 · labels: test, nfr
Done when: A run fails when a metric's slope exceeds the NFR-F12 threshold; the report plots each metric; the assertions are unit-tested on synthetic series (flat, leaking, noisy).

### [To Do] TEST-0114 Scheduled soak runs: weekly on main and before every release, with results stored and compared to the previous run
P1 · M9 · Task · deps: TEST-0113 · labels: test
Done when: The release checklist (OPS-F01) requires a passing soak for the release commit; a regression between runs is flagged in the report.

### [To Do] TEST-0115 Soak with chaos: a 24-hour run with a low rate of process kills and network faults on top of the soak workload
P2 · M9 · Task · deps: TEST-0114 · labels: test
Done when: Checkers and stability assertions pass for the whole run; faults are listed in the report.

## [To Do] TEST-F14 Upgrade and downgrade testing
P0 · M9 · Feature · deps: TEST-F08
Scope: Rolling upgrades between Alluvium versions with live clients; metadata.version transitions.

### [To Do] TEST-0116 Upgrade test matrix: which Alluvium version pairs are tested (previous minor to current, previous patch to current, and downgrade within the same `metadata.version`), with binaries taken from release artifacts
P0 · M9 · Task · deps: OPS-F01 · labels: test
Done when: The matrix is defined in one file and documented in testing.md; until the first release, the "previous" version is the last tagged pre-release; the harness downloads artifacts by checksum.

### [To Do] TEST-0117 Rolling upgrade test: upgrade each node of a 3-node cluster one at a time while Java and librdkafka clients produce and consume, then check histories
P0 · M9 · Task · deps: TEST-0116, TEST-0102 · labels: test · ref: tests/kafkatest/tests/core/upgrade_test.py @4.4.0
Done when: No acknowledged `acks=all` write is lost and every client recovers after each node restart; the client-observed unavailability per restart is reported for NFR-F04.

### [To Do] TEST-0118 `metadata.version` transition test: after a rolling upgrade, run `kafka-features.sh upgrade --metadata <version>` and verify feature levels, then exercise behavior gated by the new level
P0 · M9 · Task · deps: TEST-0117, META-F06 · labels: test · ref: tests/kafkatest/tests/core/upgrade_test.py @4.4.0
Done when: Feature levels before and after match `kafka-features.sh describe` expectations; an invalid downgrade is rejected with Kafka's error; the test runs against Kafka 4.4.0 too for the same transition where applicable.

### [To Do] TEST-0119 On-disk compatibility fixtures: data directories (log segments, indexes, checkpoints, metadata log and snapshots) written by each released version, committed in compressed form and opened by the current version
P0 · M9 · Task · deps: TEST-0116 · labels: test
Done when: The current version starts on each fixture, serves the stored records, and `alluvium verify log` passes; adding fixtures is part of the release checklist.

### [To Do] TEST-0120 Downgrade test: roll back from the current version to the previous one within the same `metadata.version`, with live clients
P0 · M9 · Task · deps: TEST-0117 · labels: test
Done when: The previous version starts on data written by the current one and no acknowledged write is lost; an attempted downgrade across an incompatible `metadata.version` fails at startup with a clear error.

### [To Do] TEST-0121 Run the upgrade and downgrade suites on release branches and nightly on main
P1 · M9 · Task · deps: TEST-0117, TEST-0118, TEST-0119, TEST-0120 · labels: test
Done when: A release cannot be tagged without a passing run (release checklist); failures upload all node logs.

## [To Do] TEST-F15 Kafka system-test subset
P3 · M9 · Feature · deps: TEST-F08
Scope: Evaluate running selected Kafka ducktape system tests against Alluvium.

### [To Do] TEST-0122 Spike: running selected Kafka ducktape system tests against Alluvium
P3 · M9 · Spike · deps: none · labels: test · ref: tests/kafkatest/services/kafka/kafka.py @4.4.0
Done when: Questions: what swapping `KafkaService` for an Alluvium service takes; which tests are candidates (`tests/kafkatest/tests/client/*`, `core/replication_test.py`, `core/round_trip_fault_test.py`, `core/compatibility_test_new_broker_test.py`); their value beyond our own suites. An accepted ADR records the candidate list with the expected value of each, the effort, and the decision.

### [To Do] TEST-0123 If the ADR accepts it: an `AlluviumService` for ducktape that starts and stops Alluvium nodes and exposes the bootstrap servers the Kafka tests expect
P3 · M9 · Task · deps: TEST-0122 · labels: test · ref: tests/kafkatest/services/kafka/kafka.py @4.4.0
Done when: One selected test (`client_compatibility_produce_consume_test.py`) passes against Alluvium in the ducktape Docker environment; the service lives under `tests/ducktape/` with its own README.

### [To Do] TEST-0124 Run the selected ducktape tests on a schedule and record results per test
P3 · M9 · Task · deps: TEST-0123 · labels: test
Done when: Results are published as a job artifact; failures are triaged into parity issues, or recorded with the reason in the ADR's list.

## [To Do] TEST-F16 Test data generators
P0 · M1 · Feature · deps: TYPE-F01, TYPE-F02
Scope: Shared proptest building blocks and strategies for domain types. Record strategies live in REC-F14 and protocol message strategies are generated by PROTO.

### [To Do] TEST-0125 Strategy conventions across crates: each crate exports strategies for its own types behind a `proptest-strategies` feature (as `alluvium-types` does), biased towards Kafka boundary values (−1 sentinels, 0, `i32::MAX`, empty and maximum-length strings)
P0 · M1 · Task · deps: none · labels: test · ref: docs/standards/testing.md#test-levels
Done when: testing.md lists the convention and the boundary-bias helpers; the helpers live in `alluvium_types::strategies` and are used by the TYPE strategies; the doc states that generated message strategies come from codegen (PROTO) and record batch strategies from REC-F14, both built on these helpers.

### [To Do] TEST-0126 Composite domain strategies: consistent partition watermark tuples (`log_start_offset ≤ high_watermark ≤ log_end_offset`), producer id/epoch/sequence streams with valid and invalid transitions, and topic-partition sets with colliding names
P0 · M1 · Task · deps: TEST-0125 · labels: test
Done when: Each strategy has a test asserting its generated values satisfy (or, for the invalid variants, violate) the stated invariant; STOR, REPL and TXN tests can use them.

### [To Do] TEST-0127 Adversarial byte strategies for decoder property tests: length prefixes that lie (larger than the remaining bytes, negative, 2^31 - 1), inputs truncated at every position, overlong and unterminated varints, and nesting deeper than a limit
P0 · M1 · Task · deps: TEST-0125 · labels: test, decoder, security · ref: docs/standards/security.md#untrusted-input
Done when: Each generator is tested to produce the adversarial shape it claims; PROTO and REC decoder property tests use them, and a decoder that allocates from an unchecked length fails a test built on them (shown with a toy decoder in the test suite).

### [To Do] TEST-0128 State-machine property test helper: generate weighted operation sequences, apply them to a sans-I/O state machine and to a simple reference model, compare after every operation, and shrink failures to a minimal sequence
P0 · M1 · Task · deps: TEST-0125 · labels: test · ref: docs/standards/testing.md#test-levels
Done when: A self-test with a planted bug in a toy machine shrinks to the minimal failing sequence; the failing sequence and seed are printed in a form that pastes into a unit test; the choice between writing the helper and using `proptest-state-machine` is justified per security.md#dependencies.

## [To Do] TEST-F17 Coverage reporting
P2 · M2 · Feature · deps: FND-F06
Scope: llvm-cov reports in CI, tracked per crate (not a gate).

### [To Do] TEST-0129 `cargo xtask coverage` with cargo-llvm-cov (version in the xtask tool table) running the nextest suite and writing HTML and LCOV reports, excluding generated code under `src/generated/`
P2 · M2 · Task · deps: none · labels: test · ref: docs/standards/testing.md#coverage
Done when: The command produces reports locally on Linux and macOS; excluded paths are listed in one place with the reason.

### [To Do] TEST-0130 Coverage CI job: run on main and on PRs, publish per-crate line coverage in the job summary and the HTML report as an artifact, never failing the build
P2 · M2 · Task · deps: TEST-0129 · labels: test · ref: docs/standards/testing.md#coverage
Done when: The job is not in `ci-success`'s `needs:` (testing.md: coverage is not a gate); the summary table lists each crate's coverage and its change against main.
