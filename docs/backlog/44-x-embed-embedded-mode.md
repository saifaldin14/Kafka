# X-EMBED: Embedded and lightweight mode

> Crates: `alluvium-embedded` (new), `alluvium` binary · Kafka counterpart: none (Alluvium enhancement) · Milestones: M18

Make Alluvium practical for small systems and tests: one-command dev mode, in-memory storage, an embeddable library, small-footprint and edge modes.

## [To Do] X-EMBED-F01 Dev mode
P0 · M18 · Feature · deps: SRV-F01
Scope: `alluvium dev`: one ephemeral node with all features in one command.

### [To Do] X-EMBED-0001 Embedded and lightweight mode ADR: dev mode, in-memory storage, embedded API and runtime ownership, host-process panic policy, footprint targets, edge sync, security posture
P0 · M18 · Spike · deps: none · ref: ADR-0006
Done when: ADR accepted deciding: what `alluvium dev` enables and its loopback-only posture under ADR-0006, that memory-backed nodes are single-node only, that the embedded node runs on the caller's Tokio runtime (amending architecture §6), how ADR-0004's abort policy applies in a host process, small-profile memory and thread targets, and the edge uplink/downlink model built on cluster links.

### [To Do] X-EMBED-0002 `alluvium dev` command: `--port`, `--data-dir`, `--persist`, `--topics`, `--seed-file`, `--ready-file` and `--listen-external` in the CLI framework
P0 · M18 · Task · deps: X-EMBED-0001, CLI-F01, CLI-F02
Done when: `alluvium dev --help` documents every flag; invalid combinations (for example `--persist` without `--data-dir`) exit with code 2 and a clear message; CLI parsing tests.

### [To Do] X-EMBED-0003 Automatic formatting: random cluster id, single node with combined roles, single-voter KRaft, `meta.properties` carrying a dev marker
P0 · M18 · Task · deps: X-EMBED-0002, SRV-F12
Done when: A fresh dev node formats and starts without any config file; restart with `--persist` reuses the cluster id; tests.

### [To Do] X-EMBED-0004 Ephemeral data directory: created per run, removed on clean exit unless `--persist`; directories left by crashed runs cleaned at the next start
P0 · M18 · Task · deps: X-EMBED-0003
Done when: SIGINT and SIGTERM remove the directory; a killed run's directory is removed by the next run (identified by the dev marker and a dead PID); tests.

### [To Do] X-EMBED-0005 Dev defaults: single-node internal topic settings, no initial rebalance delay, auto topic creation on
P0 · M18 · Task · deps: X-EMBED-0003
Done when: sets `offsets.topic.replication.factor=1`, `transaction.state.log.replication.factor=1`, `transaction.state.log.min.isr=1`, `share.coordinator.state.topic.replication.factor=1`, `share.coordinator.state.topic.min.isr=1` and `group.initial.rebalance.delay.ms=0`; defaults listed with rationale in the dev mode guide; explicit `--config key=value` overrides win; check-config shows the dev source; transactions and share groups work on one node (test).

### [To Do] X-EMBED-0006 Ephemeral ports: `--port 0` binds free ports and writes every chosen address to the `--ready-file` as JSON after readiness
P0 · M18 · Task · deps: X-EMBED-0002
Done when: two dev nodes with `--port 0` run concurrently; ready-file JSON schema documented and tested.

### [To Do] X-EMBED-0007 Readiness: ready file written and a `ready` line printed only after `/health/ready` is true; exit codes documented
P0 · M18 · Task · deps: X-EMBED-0006, OBS-F09
Done when: startup failures exit non-zero before writing the ready file; tests for success and for a port already in use.

### [To Do] X-EMBED-0008 Startup banner via the CLI output module: bootstrap servers, native endpoints, console URL and example produce and consume commands; human-readable logs by default
P1 · M18 · Task · deps: X-EMBED-0007
Done when: banner golden test; `--json` prints the ready-file content instead.

### [To Do] X-EMBED-0009 Seed data: `--topics name:partitions[,...]` and `--seed-file` (topics and configs) applied at startup with clear errors
P1 · M18 · Task · deps: X-EMBED-0007
Done when: seeded topics exist before the ready file is written; a bad seed file exits non-zero naming the line.

### [To Do] X-EMBED-0010 All features in one command: native API, schema registry and web console listeners enabled on loopback when compiled in, each switchable off with `--no-<feature>`
P1 · M18 · Task · deps: X-EMBED-0007, X-NATIVE-F03, X-SCHEMA-F09, X-OBSX-F10
Done when: ready file lists each enabled endpoint; tests with each feature on and off.

### [To Do] X-EMBED-0011 Dev mode security posture: loopback-only by default; `--listen-external` requires TLS or the explicit plaintext opt-in
P0 · M18 · Task · deps: X-EMBED-0002 · labels: security · ref: ADR-0006
Done when: plaintext needs `alluvium.allow.plaintext.external=true`; the banner warns that dev mode is not for production; tests: default binds only 127.0.0.1; `--listen-external` without TLS or the explicit plaintext key refuses to start with a message naming both options.

### [To Do] X-EMBED-0012 Production guard: `alluvium server` refuses a data directory with the dev marker unless `--allow-dev-data` is given
P2 · M18 · Task · deps: X-EMBED-0003 · labels: security
Done when: test starting `alluvium server` on a dev directory exits with a clear error.

### [To Do] X-EMBED-0013 Integration test: `alluvium dev --port 0` with the Java client and librdkafka producing, consuming and committing offsets, then SIGINT cleanup
P0 · M18 · Task · deps: X-EMBED-0007, X-EMBED-0005, X-EMBED-0004, TEST-F08 · labels: test
Done when: runs in `cargo xtask compat`; asserts the data directory is gone after exit.

### [To Do] X-EMBED-0014 Startup time benchmark: process start to ready under the ADR target, tracked in the benchmark job
P1 · M18 · Task · deps: X-EMBED-0007, TEST-F11 · labels: perf
Done when: regression above the tolerance fails the benchmark job.

## [To Do] X-EMBED-F02 In-memory storage backend
P0 · M18 · Feature · deps: STOR-F01
Scope: fs implementation backed by memory for development and tests.

### [To Do] X-EMBED-0015 `alluvium.storage.backend` = `disk` (default) or `memory`, validated at startup; `memory` refused unless the node is the only broker and the only voter
P0 · M18 · Task · deps: X-EMBED-0001, CFG-F11 · labels: config, security
Done when: tests: multi-node or multi-voter configurations with `memory` refuse to start with a reason; default stays `disk`.

### [To Do] X-EMBED-0016 Harden the STOR-F01 in-memory fs for serving traffic: chunked `Bytes` storage without copies on read, single writer and concurrent readers per file, fsync as a documented no-op
P0 · M18 · Task · deps: X-EMBED-0015, STOR-F01
Done when: fs conformance suite passes; reads return shared `Bytes` slices (no copy, test with pointer identity of the backing buffer); concurrency test with readers during appends.

### [To Do] X-EMBED-0017 Memory bound: `alluvium.storage.memory.max.bytes` across all files; reaching it takes the disk-full path with the same Kafka error codes
P0 · M18 · Task · deps: X-EMBED-0016, STOR-F25
Done when: produce beyond the bound fails exactly as a full disk does (test asserts the error code); metadata writes keep their reserved headroom.

### [To Do] X-EMBED-0018 Optional eviction under memory pressure: `alluvium.storage.memory.evict.on.full` deletes the oldest segments of non-compacted topics before rejecting writes, emitting `memory_eviction` events
P1 · M18 · Task · deps: X-EMBED-0017
Done when: log start offset advances as with retention; compacted and internal topics never evicted; test.

### [To Do] X-EMBED-0019 KRaft metadata log and snapshots on the memory backend; a restarted memory node starts empty with a new cluster id
P0 · M18 · Task · deps: X-EMBED-0016, RAFT-F06
Done when: restart test shows a clean new cluster and no reuse of old epochs; documented in the guide.

### [To Do] X-EMBED-0020 Fetch path for the memory backend: responses built from `Bytes` slices instead of sendfile
P0 · M18 · Task · deps: X-EMBED-0016, NET-F06 · labels: perf
Done when: fetch responses written without copying segment bytes (vectored write test); TLS path unchanged.

### [To Do] X-EMBED-0021 Metrics: `alluvium_storage_memory_bytes`, `alluvium_storage_memory_limit_bytes`, `alluvium_storage_memory_evictions_total`
P0 · M18 · Task · deps: X-EMBED-0017 · labels: observability
Done when: values asserted in tests; rows in docs/metrics.md.

### [To Do] X-EMBED-0022 Produce, consume, group and transaction compatibility subset passes on the memory backend
P0 · M18 · Task · deps: X-EMBED-0017, X-EMBED-0019, X-EMBED-0020, TEST-F08 · labels: test
Done when: subset of `cargo xtask compat` runs against a memory-backed dev node in CI.

### [To Do] X-EMBED-0023 Benchmark: produce and fetch throughput and latency on the memory backend versus the disk backend on tmpfs
P1 · M18 · Task · deps: X-EMBED-0020, TEST-F11 · labels: perf
Done when: results published; memory backend kept only if it shows a measured benefit for tests and dev use.

## [To Do] X-EMBED-F03 Embedded library
P1 · M18 · Feature · deps: SRV-F01
Scope: Start a node in-process from Rust.

### [To Do] X-EMBED-0024 `alluvium-embedded` crate (top layer beside the binary) with an `EmbeddedNode` builder: storage (memory or directory), ephemeral listeners and config overrides
P1 · M18 · Task · deps: X-EMBED-0001
Done when: crate registered in the layer table and the docs/architecture.md code map; `//!` docs; builder unit tests reject invalid combinations with typed errors.

### [To Do] X-EMBED-0025 Runtime ownership: the node runs on the caller's Tokio multi-thread runtime and never creates its own; architecture §6 updated per the ADR
P1 · M18 · Task · deps: X-EMBED-0024
Done when: starting inside a current-thread runtime returns a typed error; doc test shows use with `#[tokio::main]`.

### [To Do] X-EMBED-0026 Lifecycle: `start().await`, `bootstrap_servers()`, `shutdown().await` with ordered shutdown; dropping without shutdown cancels tasks and logs a warning
P1 · M18 · Task · deps: X-EMBED-0025, SRV-F03
Done when: shutdown releases ports and files (test restarts on the same port); drop path tested.

### [To Do] X-EMBED-0027 Observability inside a host: no global tracing subscriber installed, node spans flow into the host's subscriber, metrics registry handle exposed to the host
P1 · M18 · Task · deps: X-EMBED-0026 · labels: observability
Done when: test with a host subscriber captures node spans; the host can scrape the returned registry.

### [To Do] X-EMBED-0028 Panic policy in a host process: invariant violations still abort per ADR-0004; the panic hook chains to the host's existing hook instead of replacing it
P1 · M18 · Task · deps: X-EMBED-0026, SRV-F16 · ref: ADR-0004
Done when: test installs a host hook and asserts it still runs; documented in the crate docs.

### [To Do] X-EMBED-0029 `EmbeddedCluster`: N nodes in one process (combined or separate roles) on ephemeral ports, with no shared global state between nodes
P1 · M18 · Task · deps: X-EMBED-0026
Done when: two three-node clusters run concurrently in one test process; killing a node's leader triggers a new election.

### [To Do] X-EMBED-0030 Test helpers: create topics, wait for leaders and wait for readiness, built on the native SDK
P2 · M18 · Task · deps: X-EMBED-0026, X-NATIVE-F12
Done when: helpers documented with examples; no sleeps (they await metadata changes).

### [To Do] X-EMBED-0031 Cargo features to slim the embedded build (TLS, native API, console optional) with CI checks of each feature combination
P1 · M18 · Task · deps: X-EMBED-0024
Done when: CI builds the minimal and full feature sets; binary size reported for each.

### [To Do] X-EMBED-0032 Public API stability: semver policy, `cargo semver-checks` in CI and docs.rs documentation with examples
P1 · M18 · Task · deps: X-EMBED-0024
Done when: CI fails on an unintended breaking change; every public item documented.

### [To Do] X-EMBED-0033 Tests: embedded node in `#[tokio::test]` with a Kafka client; three-node embedded cluster survives a leader kill without acks=all loss
P1 · M18 · Task · deps: X-EMBED-0029, X-EMBED-0027 · labels: test
Done when: suite runs under nextest in parallel without port conflicts.

### [To Do] X-EMBED-0034 Benchmark: embedded node start time and memory
P2 · M18 · Task · deps: X-EMBED-0026, TEST-F11 · labels: perf
Done when: results tracked in the benchmark job.

## [To Do] X-EMBED-F04 Small-footprint profile
P1 · M18 · Feature · deps: NFR-F05
Scope: Low-memory, few-thread defaults.

### [To Do] X-EMBED-0035 `alluvium.runtime.worker.threads` (default: number of cores) consumed by the binary's runtime builder
P1 · M18 · Task · deps: X-EMBED-0001 · labels: config
Done when: key registered via CFG-F11; thread count asserted in a test; invalid values rejected.

### [To Do] X-EMBED-0036 Small-footprint profile `alluvium.profile=small`: thread counts, queue sizes, fetch session cache and flight recorder size
P1 · M18 · Task · deps: X-EMBED-0035, X-LOWLAT-F07 · labels: config
Done when: sets `num.network.threads`, `num.io.threads`, `background.threads`, `log.cleaner.threads`, `num.recovery.threads.per.data.dir`, `alluvium.runtime.worker.threads`, `queued.max.requests` and `max.incremental.fetch.session.cache.slots`; profile table generated into docs; explicit values win; tests assert effective values.

### [To Do] X-EMBED-0037 Small-profile storage defaults: smaller `log.segment.bytes` and `log.index.size.max.bytes` to cut per-partition memory
P1 · M18 · Task · deps: X-EMBED-0036
Done when: per-partition memory measured before and after with 100 partitions; numbers recorded in the ADR.

### [To Do] X-EMBED-0038 Budgets: idle RSS with 100 partitions and thread count under the ADR targets in the small profile, measured in CI
P1 · M18 · Task · deps: X-EMBED-0036, X-EMBED-0037, NFR-F05 · labels: nfr, test
Done when: CI test fails when RSS or thread count exceeds the budget; trend recorded.

### [To Do] X-EMBED-0039 Slim build: a `minimal` cargo feature set and a binary size budget gate
P2 · M18 · Task · deps: X-EMBED-0035 · labels: nfr
Done when: CI reports binary size per feature set and fails above the budget.

### [To Do] X-EMBED-0040 Low-resource platform CI: aarch64 container with a 256 MiB memory limit runs the smoke suite under the small profile
P2 · M18 · Task · deps: X-EMBED-0038, NFR-F10 · labels: test
Done when: CI job passes with no OOM kill; memory high-water mark recorded.

## [To Do] X-EMBED-F05 Testcontainers modules
P1 · M18 · Feature · deps: X-EMBED-F01
Scope: Modules for Java, Go, Python and Rust.

### [To Do] X-EMBED-0041 Dev image variant `alluvium:<version>-dev`: entrypoint `alluvium dev`, readiness log line, health check, multi-arch, built in the OPS-F03 pipeline
P1 · M18 · Task · deps: OPS-F03
Done when: image starts and becomes healthy in a container test; SBOM produced; non-root user.

### [To Do] X-EMBED-0042 Late-bound advertised listener: `alluvium dev --advertised-listener-file <path>` waits for the mapped host and port written after container start, as Kafka's Testcontainers module does with its starter script
P1 · M18 · Task · deps: X-EMBED-0041
Done when: container test maps a random host port and a client on the host connects through it; timeout with a clear error when the file never appears.

### [To Do] X-EMBED-0043 Shared module spec: options common to all languages (native API port, schema registry URL, console, seed topics, image override, reuse) in one spec file
P1 · M18 · Task · deps: X-EMBED-0042
Done when: each language module's tests check option coverage against the spec.

### [To Do] X-EMBED-0044 Java module `AlluviumContainer` for Testcontainers for Java, published to Maven Central
P1 · M18 · Task · deps: X-EMBED-0043
Done when: JUnit tests with kafka-clients 4.4.0 produce, consume and commit offsets; module API mirrors `KafkaContainer` (`getBootstrapServers()`).

### [To Do] X-EMBED-0045 Go module for testcontainers-go
P1 · M18 · Task · deps: X-EMBED-0043
Done when: go tests with franz-go produce and consume; module published with a semver tag.

### [To Do] X-EMBED-0046 Python module for testcontainers-python
P1 · M18 · Task · deps: X-EMBED-0043
Done when: pytest tests with confluent-kafka-python produce and consume; package published to PyPI.

### [To Do] X-EMBED-0047 Rust module (`alluvium-testcontainers` crate implementing the testcontainers-rs `Image` trait)
P1 · M18 · Task · deps: X-EMBED-0043
Done when: tests produce and consume with the native SDK and a Kafka-protocol client; crate published.

### [To Do] X-EMBED-0048 CI matrix running every module's tests on Linux and macOS Docker
P1 · M18 · Task · deps: X-EMBED-0044, X-EMBED-0045, X-EMBED-0046, X-EMBED-0047 · labels: test
Done when: all four modules green on both platforms in CI.

### [To Do] X-EMBED-0049 Release pipeline: modules versioned with Alluvium, image pinned by digest, published on release
P2 · M18 · Task · deps: X-EMBED-0048
Done when: release dry run publishes to staging registries; digests match the released image.

### [To Do] X-EMBED-0050 Docs per language, including migration from Kafka's Testcontainers modules (`KafkaContainer` to `AlluviumContainer`)
P2 · M18 · Task · deps: X-EMBED-0044, X-EMBED-0045, X-EMBED-0046, X-EMBED-0047 · labels: docs
Done when: each language page has a copy-paste test example verified in CI.

## [To Do] X-EMBED-F06 Edge mode
P2 · M18 · Feature · deps: X-EMBED-F01, X-GEO-F02
Scope: A single node syncing to a central cluster.

### [To Do] X-EMBED-0051 Edge configuration: `alluvium.edge.enabled` (default false), `alluvium.edge.id`, the central cluster link name and topic mappings from local to central topics (default prefix `edge.<edge-id>.`)
P2 · M18 · Task · deps: CFG-F11 · labels: config
Done when: validation rejects missing link names, invalid ids and mapping collisions; keys documented.

### [To Do] X-EMBED-0052 Outbound-only connectivity: the edge initiates every connection to central through the cluster link; TLS required for a non-loopback central; credentials via config providers
P2 · M18 · Task · deps: X-EMBED-0051 · labels: security
Done when: plaintext to a non-loopback central refuses to start; credentials never logged; test with mTLS.

### [To Do] X-EMBED-0053 Sync step, uplink: forward local partitions in order to central with an idempotent producer, batching and `alluvium.edge.uplink.compression`
P2 · M18 · Task · deps: X-EMBED-0052
Done when: records arrive at central in per-partition order; integration test with three local partitions.

### [To Do] X-EMBED-0054 Sync step, checkpoint: persist (local partition, last forwarded offset) after central acknowledgement; restart resumes from the checkpoint
P2 · M18 · Task · deps: X-EMBED-0053
Done when: checkpoint written atomically (tmp, fsync, rename); crash test shows resume without gaps.

### [To Do] X-EMBED-0055 Sync step, duplicate marking: an `alluvium.edge.source` header (edge id, partition, offset) on uplinked records for downstream deduplication; at-least-once across restarts documented
P2 · M18 · Task · deps: X-EMBED-0053
Done when: header format follows the REC-F13 conventions; duplicates after a crash carry identical headers (test).

### [To Do] X-EMBED-0056 Sync step, downlink: mirror selected central topics into read-only local topics through the cluster link
P2 · M18 · Task · deps: X-EMBED-0052, X-GEO-F03
Done when: local produce to a downlinked topic is rejected as for mirror topics; test.

### [To Do] X-EMBED-0057 Sync step, offline buffering: retention never deletes unsynced data; `alluvium.edge.max.unsynced.bytes` with policy `block` (produce fails with the error chosen in the ADR) or `drop-oldest` (counted)
P2 · M18 · Task · deps: X-EMBED-0054, STOR-F13
Done when: disconnected edge keeps every unsynced record until the bound; each policy tested.

### [To Do] X-EMBED-0058 Sync step, reconnect: exponential backoff with jitter up to `alluvium.edge.reconnect.max.backoff.ms` and an uplink bandwidth cap `alluvium.edge.uplink.max.bytes.per.sec`
P2 · M18 · Task · deps: X-EMBED-0053
Done when: backoff bounded (test with the simulated clock); bandwidth cap respected within one window.

### [To Do] X-EMBED-0059 Partition mapping and ordering: same partition count or key re-hash mapping, validated at startup; per-partition order preserved
P2 · M18 · Task · deps: X-EMBED-0053
Done when: mismatched partition counts without re-hash refuse to start; order checked in tests.

### [To Do] X-EMBED-0060 Observability: sync lag in records and time per topic, event `edge_link_state_changed` with reason, metrics `alluvium_edge_unsynced_bytes`, `alluvium_edge_uplink_records_total`
P2 · M18 · Task · deps: X-EMBED-0053, X-EMBED-0054 · labels: observability
Done when: values asserted in tests; rows in docs/metrics.md.

### [To Do] X-EMBED-0061 Simulation: intermittent connectivity, edge crash and central failover
P2 · M18 · Task · deps: X-EMBED-0057, X-EMBED-0058, X-EMBED-0055, TEST-F06 · labels: test
Done when: checker: every locally acknowledged record reaches central in per-partition order, with any duplicate carrying the same source header; many seeds nightly.

### [To Do] X-EMBED-0062 Integration test: an edge node in the small profile syncing to a three-node cluster through a faulty network proxy
P2 · M18 · Task · deps: X-EMBED-0061, X-EMBED-F04 · labels: test
Done when: test passes with link drops and latency injected; no unsynced data lost.

### [To Do] X-EMBED-0063 Edge mode guide: topology, configuration, guarantees and sizing the offline buffer
P2 · M18 · Task · deps: X-EMBED-0062 · labels: docs
Done when: guarantees section states exactly the at-least-once and ordering properties tested.

## [To Do] X-EMBED-F07 Embedded mode testing and documentation
P0 · M18 · Feature · deps: X-EMBED-F01
Scope: Tests and guide.

### [To Do] X-EMBED-0064 Off by default: `alluvium server` with default config never uses dev defaults, the memory backend, the small profile or edge sync
P0 · M18 · Task · deps: X-EMBED-F02, X-EMBED-F04, X-EMBED-F06 · labels: test
Done when: golden snapshot of the default effective config is unchanged by this epic; a default node creates no edge state and selects the disk backend.

### [To Do] X-EMBED-0065 Compatibility and differential suites pass against `alluvium dev`, against a memory-backed node, and against a normal server with edge sync configured but no mapped topics
P0 · M18 · Task · deps: X-EMBED-F02, X-EMBED-F03, X-EMBED-F06, TEST-F08, TEST-F09 · labels: test
Done when: `cargo xtask compat` passes in all three setups; differential diffs unchanged except for documented single-node differences (replication factor).

### [To Do] X-EMBED-0066 Dev mode guide: flags, defaults, ready file, features and limitations
P0 · M18 · Task · deps: X-EMBED-F01 · labels: docs
Done when: guide examples run in CI.

### [To Do] X-EMBED-0067 Embedded library guide: runtime requirements, lifecycle, multi-node test clusters and panic behavior
P1 · M18 · Task · deps: X-EMBED-F03 · labels: docs
Done when: examples compiled as doc tests.

### [To Do] X-EMBED-0068 Small-footprint guide: profile contents, measured footprint and platform notes
P1 · M18 · Task · deps: X-EMBED-F04 · labels: docs
Done when: numbers sourced from the CI budget job.

### [To Do] X-EMBED-0069 Migration and opt-in guide: replacing Kafka test setups (Testcontainers `KafkaContainer`, spring-kafka `EmbeddedKafka`, docker-compose) with Alluvium dev mode or the embedded library
P1 · M18 · Task · deps: X-EMBED-0066, X-EMBED-0067 · labels: docs
Done when: side-by-side examples for each replaced setup, verified in CI.

### [To Do] X-EMBED-0070 Published footprint and startup numbers (dev mode, small profile, embedded) from CI benchmarks
P1 · M18 · Task · deps: X-EMBED-F04, TEST-F11 · labels: perf
Done when: page regenerated from the benchmark job results.
