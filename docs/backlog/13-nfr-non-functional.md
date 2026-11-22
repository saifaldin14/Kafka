# NFR: Non-functional requirements

> Crates: whole system · Kafka counterpart: Kafka's operational characteristics · Milestones: M0-M9

Explicit, measured targets for performance, scalability, durability, availability, efficiency and maintainability, and the tests that prove them.

## [To Do] NFR-F01 Performance targets
P0 · M2 · Feature · deps: none
Scope: Spike: define throughput and latency targets and standard workloads; publish them in docs/nfr.md.

### [To Do] NFR-0001 Spike: performance targets, standard workloads and metrics
P0 · M2 · Spike · deps: none · labels: nfr, perf
Done when: Workloads: W1 produce throughput with 1 KiB records and `acks=all`; W2 end-to-end latency at 50% of capacity; W3 catch-up consumption from disk (cold read); W4 many partitions with small records; W5 compressed produce with lz4 and zstd; W6 fan-out to several consumer groups. Metrics: MB/s, records/s, p50/p99/p99.9/p99.99 produce and end-to-end latency, CPU seconds per GB, RSS. Targets are set by the spike, relative to Kafka 4.4.0 on the same hardware. An accepted ADR "Performance targets" and a new docs/nfr.md "Performance" section list each workload with its parameters, metrics and target (for example "throughput ≥ Kafka's, p99 ≤ Kafka's"), linking the workload files in `benches/omb/` as the single definition.

### [To Do] NFR-0002 Measure Kafka 4.4.0 on the reference environment for every standard workload (W1-W6) and record the results in docs/nfr.md
P0 · M2 · Task · deps: NFR-0001, TEST-F11 · labels: nfr, perf
Done when: Each workload has at least three runs with variance reported; raw results are stored as JSON next to the workload files; the Kafka version, JVM and configuration are recorded.

### [To Do] NFR-0003 Measure single-node Alluvium (replication factor 1) on the standard workloads that M2 supports and publish the comparison with Kafka 4.4.0
P0 · M2 · Task · deps: NFR-0002 · labels: nfr, perf
Done when: The side-by-side report (TEST-F11's comparison tool) is in docs/nfr.md with a pass or fail against each target; every miss has a linked issue with a profile (flame graph) attached.

### [To Do] NFR-0004 Hot-path budgets: per-operation thresholds for request header decode, produce record-batch validation, the log append path and fetch reads, enforced by the benchmark regression job
P1 · M2 · Task · deps: NFR-0001, TEST-F11 · labels: nfr, perf · ref: docs/standards/rust.md#performance
Done when: Each hot path named in rust.md "Performance" has a criterion benchmark with a budget entry in `benches/thresholds.toml` (with the reason); a planted 20% slowdown fails the benchmark job.

### [To Do] NFR-0005 Latency breakdown by request phase (queue, local, remote, throttle, response queue, response send) recorded during the W2 latency workload
P1 · M2 · Task · deps: NFR-0003, SRV-F08 · labels: nfr, perf
Done when: The W2 report includes the per-phase percentiles from the request metrics; a phase that dominates p99 is called out with its cause; the method is documented so regressions can be located the same way.

### [To Do] NFR-0006 Measure three-node Alluvium with replication factor 3 and `acks=all` on W1, W2 and W6 against a three-node Kafka 4.4.0 cluster
P0 · M3 · Task · deps: NFR-0003 · labels: nfr, perf
Done when: Results and the pass/fail per target are in docs/nfr.md; replication latency (time for the high watermark to pass an offset) is reported separately.

### [To Do] NFR-0007 Release performance gate: run the standard workloads on the release candidate and compare with the previous release and with Kafka 4.4.0
P1 · M9 · Task · deps: NFR-0006 · labels: nfr, perf
Done when: The release checklist (OPS-F01) requires the report; a throughput drop or p99 increase beyond the ADR's tolerance blocks the release unless an issue explains and accepts it.

## [To Do] NFR-F02 Scalability targets
P0 · M2 · Feature · deps: none
Scope: Partitions per broker and cluster, connections per broker, topics, groups, brokers per cluster.

### [To Do] NFR-0008 Spike: scalability targets per broker and per cluster
P0 · M2 · Spike · deps: none · labels: nfr · ref: tests/kafkatest/tests/core/replica_scale_test.py @4.4.0
Done when: Dimensions and candidate targets to validate: partitions per broker (100,000), partitions per cluster (1,000,000), connections per broker (50,000), topics per cluster, consumer groups per cluster and members per group, brokers per cluster, and metadata propagation time at those sizes; the spike confirms or changes the candidates with Kafka 4.4.0 measurements. An accepted ADR "Scalability targets" and a docs/nfr.md "Scalability" section list each dimension with its target, how it is measured, and the resource budget at that size (memory, file descriptors, threads).

### [To Do] NFR-0009 Per-unit resource cost model: measured memory, open file descriptors and background work per partition (idle and active), per connection and per topic on a single node
P0 · M2 · Task · deps: NFR-0008 · labels: nfr
Done when: docs/nfr.md publishes the per-unit costs measured with 1k, 10k and 50k partitions and 10k connections, and the extrapolation to the targets; any cost that makes a target unreachable has an issue naming the component.

### [To Do] NFR-0010 File descriptor budget with 100,000 partitions per broker
P0 · M2 · Task · deps: NFR-0009, STOR-F16 · labels: nfr
Done when: Open segment and index files stay within the configured limit (lazy opening and closing as the storage design decides), and hitting the process limit is reported, not fatal. A test with 100,000 small partitions on one node keeps the open-file count below the documented budget; lowering `ulimit -n` below it produces an error-level event naming the limit instead of a crash.

### [To Do] NFR-0011 Metadata scale in simulation: controller and metadata image with 1,000,000 partitions and 100 brokers: replay time, snapshot size, image memory, and delta application time for a single leader change
P0 · M3 · Task · deps: NFR-0008, TEST-F06 · labels: nfr
Done when: The simulation test runs in the nightly job and asserts the budgets from the ADR; results are recorded in docs/nfr.md.

### [To Do] NFR-0012 Connection scale test: 50,000 mostly idle client connections to one broker, checking memory per connection, accept latency and that `max.connections` limits apply cleanly
P1 · M2 · Task · deps: NFR-0009, NET-F02 · labels: nfr
Done when: The test runs on the reference environment; memory per connection is within budget; connections beyond the limit are refused with the documented behavior and a metric increment.

### [To Do] NFR-0013 Consumer group scale test: the group-count and members-per-group targets on one coordinator, measuring rebalance time and coordinator memory
P1 · M5 · Task · deps: NFR-0009, GRP-F01 · labels: nfr
Done when: Results against the targets are in docs/nfr.md; rebalance time at the target member count is reported for the classic and consumer protocols.

## [To Do] NFR-F03 Durability targets
P0 · M3 · Feature · deps: NFR-F01
Scope: Data-loss guarantees per acks and fsync mode and failure count; verification plan.

### [To Do] NFR-0014 Spike: durability guarantees matrix per acks, fsync policy and failure
P0 · M3 · Spike · deps: none · labels: nfr · ref: ADR-0006
Done when: Matrix dimensions: `acks` (0, 1, all); fsync policy (Kafka default with no `flush.*` set, and the stricter policies STOR-F17 offers); replication factor and `min.insync.replicas`; number and kind of failures (process crash, machine power loss, disk loss); unclean leader election; each cell says whether acknowledged data can be lost, matching Kafka 4.4.0. An accepted ADR "Durability guarantees" with the matrix, published in docs/nfr.md "Durability"; each "no loss" cell names the invariant that guarantees it; the matrix never claims more than the fsync policy provides (security.md).

### [To Do] NFR-0015 Durability verification plan: map each matrix cell to the test that proves it (simulation fault plan, crash-consistency test, or chaos run), listed in docs/nfr.md
P0 · M3 · Task · deps: NFR-0014 · labels: nfr, test
Done when: Every cell has a named test or an issue for the missing test; the plan is reviewed by the owners of STOR, REPL and TEST.

### [To Do] NFR-0016 Durability matrix simulation suite: one simulation test per matrix row, including the rows where loss is allowed (for example `acks=1` with leader crash before replication), which must show loss under some seed so we know the checker can detect it
P0 · M3 · Task · deps: NFR-0015, REPL-F15 · labels: nfr, test
Done when: "No loss" rows pass across the nightly seed count; "loss allowed" rows find a loss within a bounded number of seeds; the suite is listed in docs/nfr.md next to the matrix.

### [To Do] NFR-0017 Correlated power loss test: all replicas lose unsynced data at once (simulated machine power loss) under the default fsync policy, documenting the resulting loss window exactly as Kafka's
P0 · M3 · Task · deps: NFR-0016 · labels: nfr, test
Done when: The test asserts the documented bound (only records not yet fsynced on any replica can be lost) and that recovery truncates consistently on all replicas; docs/nfr.md explains the operator choice (stricter `flush.*` settings) with this test as evidence.

### [To Do] NFR-0018 Durability chaos runs: the matrix rows that real processes can exercise (process kills, network partitions) run with the chaos runner and the lost-write checker
P1 · M3 · Task · deps: NFR-0015, TEST-F12 · labels: nfr, test
Done when: Each listed row has a nightly chaos run that passes; a failing run opens an issue with the history.

## [To Do] NFR-F04 Availability targets
P0 · M3 · Feature · deps: NFR-F01
Scope: Leader failover, controller failover, restart and rolling-restart impact targets.

### [To Do] NFR-0019 Spike: availability targets for failover, restart and rolling restart
P0 · M3 · Spike · deps: none · labels: nfr
Done when: Metrics: partition unavailability after a leader crash and after controlled shutdown, controller failover time, broker restart time to rejoin the ISR, client error rate and p99 latency during a rolling restart, unavailability when one broker is partitioned; numbers set by the spike against Kafka 4.4.0 measurements. An accepted ADR "Availability targets" and a docs/nfr.md "Availability" section with each metric defined precisely (start and end events) and its target.

### [To Do] NFR-0020 Availability probe: a client workload that writes and reads each partition at a fixed rate and produces a per-partition availability timeline (gaps between successful operations)
P0 · M3 · Task · deps: NFR-0019, TEST-F12 · labels: nfr, test
Done when: The probe's timeline is part of chaos reports; its resolution (default 10 ms) is configurable; a unit test computes gaps on a synthetic history.

### [To Do] NFR-0021 Failover times in simulation: leader crash, controlled shutdown and controller leader crash, measuring simulated time from the event to the first successful produce, across seeds
P0 · M3 · Task · deps: NFR-0019, TEST-F06 · labels: nfr, test
Done when: The p50 and p99 over the nightly seed set meet the ADR targets (simulated time with the default timeouts); results are in docs/nfr.md; a regression fails the nightly job.

### [To Do] NFR-0022 Failover times on real processes: the same scenarios with the availability probe on a three-node cluster, compared with Kafka 4.4.0 under the same faults
P0 · M3 · Task · deps: NFR-0020 · labels: nfr, test
Done when: Results for both systems are in docs/nfr.md with pass/fail per target.

### [To Do] NFR-0023 Rolling restart impact: client error rate and p99 latency for producers and consumers during a full rolling restart of a three-node cluster
P1 · M9 · Task · deps: NFR-0022, TEST-F14 · labels: nfr, test
Done when: The measured impact meets the ADR target and is recorded in docs/nfr.md; the upgrade tests (TEST-F14) report the same metrics.

## [To Do] NFR-F05 Resource efficiency
P1 · M2 · Feature · deps: NFR-F01
Scope: Memory per partition and connection, idle CPU, startup memory; budgets and tests.

### [To Do] NFR-0024 Spike: resource efficiency budgets: heap memory per idle partition, per active partition, per connection and per consumer group; idle CPU (wake-ups and CPU time with no traffic); RSS after startup with no data; set against Kafka 4.4.0 measurements
P1 · M2 · Spike · deps: none · labels: nfr
Done when: An accepted ADR "Resource budgets" and a docs/nfr.md "Efficiency" section list each budget, how it is measured and the test that enforces it.

### [To Do] NFR-0025 Heap measurement for budget tests: a counting global allocator for test binaries from a maintained crate (no `unsafe` in our code), reporting live bytes and allocation counts around a code region
P1 · M2 · Task · deps: NFR-0024 · labels: nfr, test
Done when: A test shows the reported live bytes for allocating a known `Vec` within 1%; the crate is justified per security.md#dependencies and used only as a dev-dependency.

### [To Do] NFR-0026 Per-partition memory budget test: create 10,000 idle partitions in-process and assert the heap growth per partition stays within the ADR budget
P1 · M2 · Task · deps: NFR-0025, STOR-F16, REPL-F12 · labels: nfr, test
Done when: The test fails if the per-partition cost exceeds the budget, printing the measured cost; it runs in CI (scaled down if needed, with the scale stated).

### [To Do] NFR-0027 Per-connection memory budget test: open 1,000 idle connections (in-process network driver) and assert the heap growth per connection stays within budget
P1 · M2 · Task · deps: NFR-0025, NET-F02 · labels: nfr, test
Done when: The test fails with the measured cost when over budget; buffers allocated lazily on first request are verified to be freed after idle periods if the design says so.

### [To Do] NFR-0028 Idle wake-up budget in simulation: a three-node cluster with 1,000 partitions and no traffic, counting timer firings and messages per simulated minute per node
P1 · M3 · Task · deps: NFR-0024, TEST-F06 · labels: nfr, test
Done when: The counts stay within the ADR budget across seeds; the top sources of wake-ups are printed by component, so regressions point to their cause.

### [To Do] NFR-0029 Bounded-resource inventory: docs/nfr.md lists every queue, buffer, cache and pool in the server with its bound, the config key that sets it, and the behavior when it is full
P1 · M2 · Task · deps: NFR-0024 · labels: nfr, security · ref: docs/architecture.md
Done when: The table covers every bounded channel and cache in `alluvium-server` and `alluvium-network` (checked against a grep for channel and cache constructors); missing bounds found during the audit become bug issues.

## [To Do] NFR-F06 Startup and recovery time
P1 · M2 · Feature · deps: NFR-F02
Scope: Targets and tests for restart with many partitions and after unclean shutdown.

### [To Do] NFR-0030 Spike: startup and recovery targets: time to ready after a clean shutdown with 10,000 and 100,000 partitions, time to ready after an unclean shutdown as a function of unflushed data, and time for a restarted broker to rejoin all ISRs
P1 · M2 · Spike · deps: none · labels: nfr
Done when: An accepted ADR "Startup and recovery targets" and a docs/nfr.md "Startup and recovery" section with each target and its measurement (from process start to `/health/ready`).

### [To Do] NFR-0031 Startup phase timing: each startup phase (config, log loading, metadata replay, listener start) emits its start and end events with `elapsed_ms`, and a test extracts the breakdown
P1 · M2 · Task · deps: NFR-0030, SRV-F03 · labels: nfr, perf
Done when: The breakdown for a 10,000-partition start is printed by the test and recorded in docs/nfr.md; phases come from the lifecycle events, not from new instrumentation.

### [To Do] NFR-0032 Clean restart test: 10,000 partitions (CI scale) and 100,000 partitions (reference environment) restarted after a clean shutdown, measuring time to ready against the target
P1 · M2 · Task · deps: NFR-0031, STOR-F16 · labels: nfr, perf
Done when: The CI-scale test fails when time to ready exceeds its budget; the reference-scale result is in docs/nfr.md.

### [To Do] NFR-0033 Unclean shutdown recovery test: kill the process during sustained produce with 10,000 partitions and measure recovery time to ready, including segment validation
P1 · M2 · Task · deps: NFR-0031, STOR-F09 · labels: nfr, perf
Done when: Recovery time is within the target; the recovered logs pass `alluvium verify log`; the effect of `num.recovery.threads.per.data.dir` on recovery time is measured and documented.

## [To Do] NFR-F07 Overload behavior
P0 · M3 · Feature · deps: NFR-F01
Scope: Load shedding, bounded memory under overload, fairness; tests.

### [To Do] NFR-0034 Spike: overload behavior at every saturation point
P0 · M3 · Spike · deps: none · labels: nfr, security
Done when: Saturation points: accept queue, connection limits, request queue, request memory pool, delayed-operation purgatory, fetch session cache, replica fetcher, disk throughput; for each, the behavior (mute, reject with which error code, throttle, shed) and the fairness property between clients. An accepted ADR "Overload behavior" with one row per saturation point, matching Kafka 4.4.0 where Kafka defines the behavior and recording each deviation; published in docs/nfr.md "Overload".

### [To Do] NFR-0035 Produce flood test: clients offering twice the node's measured capacity for 10 minutes, asserting bounded RSS (within the budget), no crash, and that admitted requests complete within the request timeout
P0 · M3 · Task · deps: NFR-0034 · labels: nfr, test
Done when: RSS stays under the budget throughout; muting and queue-full metrics increase as the ADR specifies; after the flood stops, latency returns to baseline within the stated time.

### [To Do] NFR-0036 Connection storm test: connection attempts at ten times the normal rate from many sources, checking `max.connections`, `max.connections.per.ip` and connection-rate limits, with established clients unaffected
P0 · M3 · Task · deps: NFR-0034 · labels: nfr, test, security
Done when: Established clients keep their p99 latency within the ADR bound; excess connections are refused as specified with metrics and events; no file descriptor exhaustion occurs.

### [To Do] NFR-0037 Large-request flood: many clients sending requests close to `socket.request.max.bytes` at once, checking that memory is bounded by `queued.max.request.bytes` and that oversized frames are rejected before allocation
P0 · M3 · Task · deps: NFR-0034 · labels: nfr, test, security
Done when: Peak memory stays within the request memory bound plus the documented overhead; an oversized frame closes the connection without allocating its body (checked with the allocation meter).

### [To Do] NFR-0038 Slow and stalled readers: clients that stop reading responses (large fetches) must not grow broker memory without bound
P0 · M3 · Task · deps: NFR-0034 · labels: nfr, test, security
Done when: Per-connection response buffering stays within its bound; the connection is muted and then closed per the ADR; other clients are unaffected.

### [To Do] NFR-0039 Fairness test: one aggressive client and many well-behaved clients on the same broker, measuring the well-behaved clients' latency and throughput share
P1 · M3 · Task · deps: NFR-0035 · labels: nfr, test
Done when: The well-behaved clients' p99 latency and throughput stay within the ADR's fairness bound; the result is in docs/nfr.md.

### [To Do] NFR-0040 Overload in simulation: deterministic versions of the produce flood and slow-reader tests on a simulated cluster, with checkers for bounded queue depths
P1 · M3 · Task · deps: NFR-0034, TEST-F06 · labels: nfr, test
Done when: Queue depth checkers pass across seeds; a planted unbounded queue (test-only) is caught by the checker.

## [To Do] NFR-F08 Compatibility policy
P1 · M2 · Feature · deps: none
Scope: Supported client matrix, Kafka version tracking, deprecation policy.

### [To Do] NFR-0041 Compatibility policy page `docs/compatibility.md` with the supported client matrix
P1 · M2 · Task · deps: none · labels: nfr, docs · ref: ADR-0010
Done when: The matrix covers the Java client versions Kafka 4.4.0 brokers support (per KIP-896's removal of old protocol versions), librdkafka and franz-go version ranges, and best-effort clients, plus what "supported" means. Every supported client family names the oldest and newest versions tested in CI; the page links to parity.md for API versions instead of repeating them; DOCS-F10 can build on it.

### [To Do] NFR-0042 Spike: Kafka version tracking policy: when Alluvium adopts a newer Kafka release as its compatibility surface, how new API versions and KIPs are evaluated, and how schema vendoring and differential baselines move
P1 · M2 · Spike · deps: NFR-0041 · labels: nfr · ref: ADR-0010
Done when: An accepted ADR "Tracking Kafka releases" with the cadence, the decision process and the steps (vendor schemas, codegen diff, differential baseline image update, parity.md update).

### [To Do] NFR-0043 Deprecation policy for Alluvium-specific surfaces (`alluvium.*` config keys, native APIs, CLI flags, metrics and event names): notice period, warnings emitted when a deprecated item is used, and removal rules
P1 · M2 · Task · deps: NFR-0041 · labels: nfr, docs
Done when: The policy is in docs/compatibility.md; it states that Kafka-compatible surfaces follow Kafka's own deprecations only; event and metric renames are listed as breaking changes, as debuggability.md says.

### [To Do] NFR-0044 Client version matrix in CI: run the compatibility smoke suite with the oldest and newest supported version of each client family
P1 · M2 · Task · deps: NFR-0041, TEST-F08 · labels: nfr, test
Done when: The nightly compatibility job runs the matrix; each pin is recorded in `images.lock`; docs/compatibility.md's tested versions are read from the same file (no second list).

## [In Progress] NFR-F09 Maintainability metrics
P2 · M0 · Feature · deps: FND-F06
Scope: Build time, crate size and doc coverage tracking.

### [Done] NFR-0045 Build time tracking: CI records clean and incremental build times on main (`cargo build --timings` report as an artifact) and writes the totals to the job summary
P2 · M0 · Task · deps: none · labels: nfr
Done when: The timings artifact is available for every main build; the summary shows the change against the previous main build; nothing fails on build time.
Notes: Proven on main by two consecutive runs. [45021811502](https://github.com/saifaldin14/Kafka/actions/runs/45021811502) (7b92126, clean 25.5 s, incremental 2.3 s) had no previous build, so its summary shows `none`. [45022611108](https://github.com/saifaldin14/Kafka/actions/runs/45022611108) (d32bacc, clean 21.1 s, incremental 3.1 s) downloaded the first run's `totals.json` without error, so its summary shows the change: −4.4 s and +0.8 s. Both runs uploaded the `build-times` artifact with the `--timings` reports. No step has a threshold, so a slow build never fails.

### [To Do] NFR-0046 `cargo xtask metrics`: lines of code per crate (excluding generated code and tests counted separately), modules over 600 lines (rust.md's split signal), and transitive dependency count per crate, as JSON and a table
P2 · M1 · Task · deps: none · labels: nfr · ref: docs/standards/rust.md#modules-and-documentation
Done when: The command runs offline; unit tests on a fixture tree check the counts; the CI job summary on main shows the table; modules over 600 lines are listed as information, not failures.

### [To Do] NFR-0047 Documentation coverage report from rustdoc's coverage output (nightly), per crate, in the nightly job
P3 · M1 · Task · deps: NFR-0046 · labels: nfr, docs
Done when: The nightly job publishes per-crate documented-item percentages for public and private items; the report is information only.

## [To Do] NFR-F10 Portability
P2 · M2 · Feature · deps: none
Scope: Supported OS and architecture matrix, Linux-first features, macOS development support.

### [To Do] NFR-0048 Spike: supported platforms and Linux-only features ADR
P2 · M2 · Spike · deps: none · labels: nfr
Done when: Scope: tier 1 production platforms (Linux x86_64 and aarch64, minimum kernel and glibc), development-only platforms (macOS aarch64), unsupported platforms (Windows), and Linux-only features with their fallbacks (`sendfile`, `fallocate`, later `io_uring`). An accepted ADR "Supported platforms" and a docs/nfr.md "Portability" section with the matrix; `deny.toml` `[graph] targets` and the CI matrix are updated to match.

### [To Do] NFR-0049 Platform-specific code rule: every `#[cfg(target_os = …)]` lives in the modules the ADR names (network zero-copy, storage file operations), each with a fallback and a test on every tier 1 and development platform
P2 · M2 · Task · deps: NFR-0048 · labels: nfr
Done when: A `check-manifests` rule rejects `cfg(target_os` outside the allowed modules, with fixture tests; existing uses comply.

### [To Do] NFR-0050 Linux aarch64 CI: add an arm64 Linux leg to the test job
P2 · M2 · Task · deps: NFR-0048 · labels: nfr, test
Done when: The full test suite passes on arm64 Linux in CI; the leg is in `ci-success`'s `needs:`.

### [To Do] NFR-0051 macOS durability note and behavior: `fsync` on macOS does not flush the drive cache (`F_FULLFSYNC` does); decide and implement which call the storage layer uses on macOS and document that macOS is development-only for durability
P2 · M2 · Task · deps: NFR-0048, STOR-F17 · labels: nfr, docs
Done when: The choice is implemented in the storage file operations with a test on macOS; docs/nfr.md "Portability" states the durability difference.

### [To Do] NFR-0052 Supported file systems: ext4 and xfs on Linux are supported and tested; others are best effort; test the storage crash-consistency suite on both
P2 · M2 · Task · deps: NFR-0048 · labels: nfr, test
Done when: A CI or nightly job runs the storage suite on ext4 and xfs (loop devices or runner disks); docs/nfr.md lists the supported file systems and mount options.

## [To Do] NFR-F11 Large-scale testing
P0 · M9 · Feature · deps: NFR-F02, TEST-F06
Scope: Simulation and real tests at target scale.

### [To Do] NFR-0053 Large-scale test environment: scripts that provision and tear down a cluster of the target size (brokers, controllers, client machines) with monitoring, reproducibly
P0 · M9 · Task · deps: none · labels: nfr
Done when: The environment can be created from the repository in one command and torn down completely; its cost per run is documented; credentials come from the CI secret store, never from the repository.

### [To Do] NFR-0054 100,000 partitions per broker on real hardware: create them, run low-rate produce across all of them, perform leader changes and a broker restart, and check memory, file descriptors and startup time against the targets
P0 · M9 · Task · deps: NFR-0053 · labels: nfr, test · ref: tests/kafkatest/tests/core/replica_scale_test.py @4.4.0
Done when: Every NFR-F02 and NFR-F06 target at this size passes, or a linked issue explains the gap; results are in docs/nfr.md.

### [To Do] NFR-0055 1,000,000 partitions per cluster on real hardware: controller failover time, metadata propagation time to all brokers, snapshot size and loading time
P0 · M9 · Task · deps: NFR-0053 · labels: nfr, test
Done when: The measurements meet the NFR-F02 targets or have linked issues; results are in docs/nfr.md alongside the simulation figures from NFR-F02.

### [To Do] NFR-0056 Topic churn at scale: create and delete topics continuously at the target topic count for one hour, checking controller latency, metadata growth and leaked resources
P1 · M9 · Task · deps: NFR-0053 · labels: nfr, test
Done when: Controller request latency stays within target; metadata image size returns to baseline after deletion; no files or memory are left behind (NFR-F12 checks).

### [To Do] NFR-0057 Client scale at target: the target connection count per broker and the target consumer group count, with realistic client churn
P1 · M9 · Task · deps: NFR-0053 · labels: nfr, test
Done when: Targets are met or have linked issues; results are in docs/nfr.md.

### [To Do] NFR-0058 Large-scale results report: one docs/nfr.md section summarizing every scale target with the latest measured value, date and commit
P0 · M9 · Task · deps: NFR-0054, NFR-0055, NFR-0056, NFR-0057 · labels: nfr, docs
Done when: Each target row shows pass or fail with a link to its run; the release checklist requires the report to be current for a GA release.

## [To Do] NFR-F12 Resource leak detection
P1 · M9 · Feature · deps: TEST-F13
Scope: Long-run memory, file-descriptor and task leak checks.

### [To Do] NFR-0059 Leak detection from soak data: regression on RSS, heap (allocator statistics), file descriptor, thread and task counts after warm-up, failing when the slope exceeds the threshold over 24 hours
P1 · M9 · Task · deps: none · labels: nfr, test
Done when: Thresholds are in docs/nfr.md; the analysis is unit-tested on synthetic flat, leaking and noisy series; the soak report shows the slopes.

### [To Do] NFR-0060 Churn leak tests: repeat topic create/delete, connection open/close and group join/leave cycles thousands of times on one node, then check that file descriptors, supervised task count (`/debug/tasks`) and heap return to the baseline within tolerance
P1 · M9 · Task · deps: NFR-0059 · labels: nfr, test
Done when: Each churn test fails with the leaked resource named; a planted leak (a test-only build that keeps a task per connection) is detected.

### [To Do] NFR-0061 Metric series leak test: after deleting topics and disconnecting clients, the number of exported metric series returns to the baseline (no per-topic or per-client series left behind)
P1 · M9 · Task · deps: NFR-0060 · labels: nfr, test · ref: docs/standards/debuggability.md#metrics
Done when: The test compares `/metrics` series counts before and after churn; any leftover series is printed by name.

### [To Do] NFR-0062 Per-entity state cleanup test: flight recorders, caches, fetch sessions and quota entries for deleted partitions, groups and clients are removed
P1 · M9 · Task · deps: NFR-0060 · labels: nfr, test
Done when: After churn, the diagnostics endpoint lists no deleted entities and heap is back to the baseline; each leftover kind fails the test with its name.

### [To Do] NFR-0063 LeakSanitizer run: the test suite on a nightly toolchain with `-Zsanitizer=leak` in the nightly job
P2 · M9 · Task · deps: NFR-0059 · labels: nfr, test
Done when: The job runs the suite and reports leaks with stacks; known false positives are suppressed in one file with a reason per entry.

## [To Do] NFR-F13 Time handling
P1 · M3 · Feature · deps: TYPE-F06
Scope: Clock skew, wall-clock jumps, monotonic vs wall usage audit.

### [To Do] NFR-0064 Time usage audit: classify every clock read as monotonic or wall clock, compared with Kafka
P1 · M3 · Task · deps: none · labels: nfr · ref: clients/src/main/java/org/apache/kafka/common/utils/Time.java @4.4.0
Done when: Monotonic: timeouts, durations, rates, session expiry; wall clock: record timestamps, time-based retention, token expiry; compared with Kafka's use of `Time.milliseconds()` and `nanoseconds()` for the same purpose. docs/nfr.md "Time handling" holds the table with file, purpose, clock kind and Kafka reference; every mismatch with Kafka's choice has an issue or an ADR reference.

### [To Do] NFR-0065 Wall-clock jump tests in simulation: forward and backward jumps of hours on one node, checking that timeouts and sessions use monotonic time (unaffected) and that wall-clock-based behavior (time-based retention, `LogAppendTime`) matches Kafka 4.4.0
P1 · M3 · Task · deps: NFR-0064, TEST-F06 · labels: nfr, test
Done when: Tests pass across seeds; the expected Kafka behavior for each wall-clock case (for example, a forward jump making time-based retention delete segments) is documented with a `// Parity:` citation.

### [To Do] NFR-0066 Clock skew tests: nodes with several minutes of wall-clock skew between them, covering replication, leader election, delegation token expiry checks and timestamp validation (`log.message.timestamp.*` settings) of client-supplied timestamps
P1 · M3 · Task · deps: NFR-0064, TEST-F06 · labels: nfr, test
Done when: Behavior matches Kafka 4.4.0 for each case and the tests pass across seeds; a skew-related limitation (if any) is documented in docs/nfr.md.

### [To Do] NFR-0067 Real-process clock skew run: a three-node chaos run with one node's wall clock offset by 10 minutes, with the standard checkers
P2 · M3 · Task · deps: NFR-0066, TEST-F12 · labels: nfr, test
Done when: Checkers pass; the run is part of the nightly chaos set.
