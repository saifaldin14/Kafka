# Alluvium architecture

This document describes how Alluvium is organized and why. Read it before changing code. When the code and this document disagree, one of them is wrong; fix it in the same change.

## 1. Goals and non-goals

**Goals**

- Wire and semantic compatibility with Apache Kafka 4.4.0 for client-facing APIs, as a permanent protocol surface ([ADR-0010](adr/0010-alluvium-is-its-own-platform.md)).
- Core types and interfaces that don't inherit Kafka's limitations, so the planned enhancements fit without rework (§10).
- A single cluster mode: KRaft. Nodes run as broker, controller, or both.
- Every component testable in deterministic simulation and debuggable from its logs, metrics, flight recorder and on-disk state.

**Non-goals**

- ZooKeeper mode, and the ZooKeeper-era APIs (keys 4–7) removed in Kafka 4.0.
- Message formats v0 and v1, which Kafka 4.0 dropped ([KIP-724](https://cwiki.apache.org/confluence/display/KAFKA/KIP-724%3A+Drop+support+for+message+formats+v0+and+v1)).
- Mixed clusters of Kafka and Alluvium nodes.
- Reimplementing the Kafka Streams or Kafka Connect runtimes. They are client-side and run against Alluvium through the protocol. Built-in processing is a separate, native feature (X-SQL, X-WASM).
- JMX. Metrics are exported in Prometheus format, with a documented mapping from Kafka metric names.

## 2. Bird's-eye view

```
          Kafka clients (Java, librdkafka, franz-go, …) and admin tools
                                   │  Kafka wire protocol (TCP, TLS, SASL)
┌──────────────────────────────────▼──────────────────────────────────────┐
│ network       listeners · framing · TLS · SASL · per-connection ordering│
│               · request size limits · backpressure (connection muting)  │
├─────────────────────────────────────────────────────────────────────────┤
│ server        request router (ApiKey → handler) · authorization ·       │
│               quotas · drivers that run the state machines below        │
├──────────────┬───────────────┬─────────────────┬────────────────────────┤
│ replication  │ coordinators  │ metadata image  │ controller (if role)   │
│ partition    │ group, txn,   │ (read-only view │ + raft quorum          │
│ state, ISR,  │ share         │  of cluster     │ metadata log           │
│ HW, fetchers │               │  metadata)      │                        │
├──────────────┴───────────────┴─────────────────┴────────────────────────┤
│ storage       logs · segments · indexes · recovery · retention ·        │
│               compaction · producer state · (tiered storage)            │
├─────────────────────────────────────────────────────────────────────────┤
│ protocol · records · types   (pure codecs and domain types)             │
└─────────────────────────────────────────────────────────────────────────┘
```

Metadata flows one way. The controller quorum commits metadata records to the Raft log. Brokers replay that log into an immutable **metadata image**, and every broker component reads only the image. There is no other path for cluster state.

## 3. Design principles

These shape every module. The detailed rules are in [standards/](standards/).

1. **Sans-I/O state machines.** Logic is pure: `(state, input) → (state', actions)`. Drivers perform the actions: send bytes, write files, set timers. ([ADR-0002](adr/0002-sans-io-state-machines-driven-by-tokio.md))
2. **Deterministic simulation** is the main test for distributed behavior. Any failure is a seed you can replay. ([ADR-0003](adr/0003-deterministic-simulation-testing.md))
3. **One owner per piece of mutable state.** Each partition, group and transaction has exactly one writer. Shared state is read through immutable snapshots (such as the metadata image) or published values.
4. **Bounded resources.** Every queue, buffer, cache and per-client allocation has a limit and a defined behavior when it is reached.
5. **Explicit over clever.** Plain structs and enums, generated code you can read, no macro magic beyond codegen, no hidden global state. The metrics registry, clock and config are passed in explicitly.

## 4. Code map

The workspace is a set of crates under `crates/`. **A crate is created when its first real code lands**, not ahead of time. The table is the target map. Layer numbers drive the dependency rules in §5.

| Layer | Crate | Responsibility | Kafka counterpart |
|---|---|---|---|
| 0 | `alluvium-types` | Domain newtypes (`BrokerId`, `TopicId`, `TopicName`, `Offset`, `LeaderEpoch`, `ProducerId`…), `Clock` trait, `Secret<T>` | various |
| 0 | `alluvium-protocol` | Wire primitives (varints, compact strings, tagged fields), generated request/response types, `ApiKey`, `ErrorCode`, version ranges | `clients/…/common/protocol`, `common/message` |
| 0 | `alluvium-records` | Record batch v2: parse, build, validate, CRC32C, compression (gzip, snappy, lz4, zstd) | `common/record`, `LogValidator` |
| 1 | `alluvium-storage` | Partition log: segments, offset/time indexes, recovery, retention, compaction, producer state, leader-epoch cache. File I/O goes through a narrow `fs` trait, so tests can inject faults | `storage/…/internals/log` |
| 1 | `alluvium-raft` | KRaft consensus state machine (vote, fetch, snapshots, voter changes) | `raft/` |
| 1 | `alluvium-metadata` | Metadata record types (generated), `MetadataImage`/`MetadataDelta`, replay | `metadata/…/image` |
| 1 | `alluvium-security` | SASL mechanism state machines, principals, ACL authorizer, SCRAM credentials | `common/security`, `StandardAuthorizer` |
| 2 | `alluvium-controller` | Controller state machine: topics, partitions, leader election, ISR/ELR, broker registration, features, ACL/config records | `metadata/…/controller` |
| 2 | `alluvium-replication` | Partition leader/follower state, ISR expand/shrink, high watermark, delayed produce/fetch (purgatory) logic | `cluster/Partition`, `ReplicaManager` |
| 2 | `alluvium-group-coordinator` | Classic, consumer (KIP-848), and streams (KIP-1071) group protocols; committed offsets | `group-coordinator/` |
| 2 | `alluvium-txn-coordinator` | Transaction state machine, producer id allocation, markers | `transaction-coordinator/` |
| 2 | `alluvium-share-coordinator` | Share groups (KIP-932) state | `share-coordinator/` |
| 3 | `alluvium-config` | Typed config, Kafka-compatible key names, validation, source tracking (default/file/env/dynamic), dynamic updates | `KafkaConfig`, `LogConfig` |
| 3 | `alluvium-observability` | `tracing` setup, metrics registry and exporter, flight recorder, diagnostics endpoint | `kafka.metrics`, JMX |
| 3 | `alluvium-network` | TCP listeners, TLS (rustls), framing, SASL driving, connection state and limits, per-connection response ordering, muting | `network/SocketServer`, `Selector` |
| 4 | `alluvium-server` | Node assembly and lifecycle, request router and handlers (`handlers/<api>.rs`), drivers for all layer 1–2 state machines, quotas | `KafkaApis`, `BrokerServer`, `ControllerServer` |
| 5 | `alluvium` (binary) | CLI: `server`, `dump log`, `dump index`, `dump metadata`, `verify log`, `check-config` | `kafka-server-start`, `kafka-dump-log` |
| dev | `alluvium-testkit` | Simulation harness: simulated clock, network and disk; fault injection; seeded runs; cluster builders | test utilities |
| dev | `xtask` | Codegen, layer checks, CI entry point, compatibility test runner | Gradle tasks |
| dev | `lint-canary` | Fixtures that break the workspace lints on purpose, proving each one fires (`xtask/lint-canary`) | checkstyle tests |

`fuzz/` holds the cargo-fuzz targets. `tests/compat/` holds the compatibility and differential suites.

The enhancement track adds crates when their first code lands. Each one's layer is decided in its epic's design ADR:
- `alluvium-api` and `alluvium-client` (native APIs and SDK);
- `alluvium-schema` (schema registry);
- `alluvium-queue` (queue semantics);
- `alluvium-kv` (key index);
- `alluvium-diskless` (object-storage topics);
- `alluvium-link` (geo-replication);
- `alluvium-transform` (WASM);
- `alluvium-sql`;
- `alluvium-autopilot`;
- `alluvium-embedded`.

## 5. Layering rules

Machine-checked by `cargo xtask check-layers` ([ADR-0009](adr/0009-crate-layering-and-dependency-rules.md)).

Layers, bottom to top:

| Layer | Role | Rule of thumb |
|---|---|---|
| 0 | Codecs and domain types | Pure functions and data |
| 1 | Primitives: log storage, Raft, metadata image, security | One concept each, no knowledge of the others |
| 2 | Domain logic: controller, replication, coordinators | Pure state machines composed from layer 0–1 types |
| 3 | Infrastructure: config, observability, network | Tokio allowed from here up |
| 4 | Server: drivers, handlers, lifecycle | The only place where everything meets |
| 5 | Binary | Argument parsing and `main` |

1. A crate may depend only on crates in **lower** layers. Same-layer dependencies are forbidden unless listed in `xtask`'s allow-list with an ADR reference. There are none today.
2. **Layers 0–2 must not depend on Tokio**, any async runtime, or any networking crate. They must not perform I/O (except `alluvium-storage` through its `fs` trait), read the clock, or spawn threads. `alluvium-types` defines the system clock and the OS-entropy RNG constructor so that each has one implementation, but only layer 3–5 code constructs them, and `check-layers` rejects any other layer 0–2 crate that names them; layer 0–2 state machines receive time and randomness from their drivers ([ADR-0011](adr/0011-time-model-and-clock-injection.md), [ADR-0012](adr/0012-seeded-randomness-and-hashing.md)). They may use the `tracing` facade for debug/trace events. State transitions are *returned* to the driver, which records them (flight recorder, info-level event, metrics). Layer 0–2 code never touches the metrics registry.
3. Each layer 0–2 crate defines its own plain config struct (for example `storage::LogConfig`). `alluvium-config` maps Kafka config keys onto those structs, so lower layers never parse config.
4. `dev` crates may depend on anything. Nothing except tests may depend on `dev` crates.
5. Shared code goes down a layer into the crate that owns the concept, never into a "common" crate.

## 6. Runtime and concurrency model

Chosen in [ADR-0002](adr/0002-sans-io-state-machines-driven-by-tokio.md).

- **Tokio multi-threaded runtime** for network and request handling. The runtime is created in exactly one place, the binary.
- **Drivers.** Each layer 1–2 state machine has a driver in `alluvium-server`. The driver owns the machine and feeds it inputs (requests, peer messages, timer ticks, completed I/O). It executes the returned actions and records transitions in the flight recorder.
- **Single writer per partition.** All mutations of a partition's log and replication state go through one owner. Readers (fetches) read published offsets (high watermark, last stable offset, log end offset) without taking the writer's lock. *Whether the owner is an actor task or a lock is decided by the spike REPL-F01, which ends in an ADR.*
- **Blocking work** (file I/O, fsync, compression of large batches) never runs on Tokio worker threads. The storage driver owns where it runs (spike STOR-F18).
- **Tasks** are spawned only through a supervisor that names them, attaches a span, tracks them for shutdown, and treats a task panic as a process-fatal bug ([ADR-0004](adr/0004-error-model-and-panic-policy.md)).
- **Shutdown order** is the reverse of startup:
  1. Stop accepting connections.
  2. Drain in-flight requests (bounded time).
  3. Stop fetchers and coordinators.
  4. Flush and close logs.
  5. Close the Raft client.

  Each step logs its start, end and duration.
- **Backpressure.** Per-connection in-flight requests are capped. When the request queue is full, connections are muted (stop reading), as Kafka does. Queues are bounded channels sized from config.

## 7. Request lifecycle

**Produce (`acks=all`)**

1. `network` reads a size-prefixed frame (rejecting it if it is over `socket.request.max.bytes`), decodes the header, and opens the `request` span.
2. `server` authorizes `WRITE` on each topic and decodes the body (`alluvium-protocol`).
3. Records are validated by `alluvium-records`: CRC, batch structure, compression, timestamps.
4. The handler sends the append to each partition's owner. `alluvium-replication` checks that the partition is the leader and that ISR size ≥ `min.insync.replicas`, then appends through `alluvium-storage`.
5. If the high watermark has not reached the appended offset, the request waits as a delayed produce. It completes when followers' fetches advance the high watermark, or it times out with `REQUEST_TIMED_OUT`.
6. The response is queued on the connection. Responses are written in request order per connection.

**Fetch (consumer)**

1. Same network and authorization steps (`READ`).
2. For each partition: resolve the fetch offset to a segment and position (offset index, then scan), and read up to the high watermark, or the last stable offset for `read_committed`.
3. If not enough bytes are available and `max_wait_ms > 0`, the request waits as a delayed fetch until data arrives or the wait expires.
4. On plaintext connections, segment bytes go from file to socket without copying through user space (`sendfile`). With TLS they pass through rustls.

**Follower replication** uses the same Fetch API. The follower's fetch offset tells the leader that follower's log end offset, which drives ISR membership and high watermark advancement.

## 8. Invariants

These are checked with `debug_assert!` in state machines, with property tests, and with simulation checkers. A violation is a bug, never "handled".

- Offsets in a partition are assigned contiguously and never reused. Log end offset only grows, except for truncation during leader-epoch reconciliation.
- On the leader: log start offset ≤ high watermark ≤ log end offset.
- The high watermark is the minimum log end offset over the maximal ISR (including caught-up replicas waiting to join). It never decreases on a leader.
- Consumers never read past the high watermark. `read_committed` consumers never read past the last stable offset.
- Leader epochs are monotonic per partition. A replica rejects requests from a stale epoch.
- A produce acknowledged with `acks=all` was in the log of every ISR member at acknowledgement time, and the ISR size was at least `min.insync.replicas`.
- For an idempotent producer, a sequence for (producer id, epoch, partition) is appended at most once.
- Responses on one connection are sent in the order the requests arrived.
- Metadata seen by a broker component comes from one image version at a time. No component mixes two versions within one request.

## 9. Cross-cutting concerns

| Concern | Where it's defined |
|---|---|
| Rust rules, lints, async, error types | [standards/rust.md](standards/rust.md) |
| Logging, tracing fields, metrics, flight recorder, dump tools | [standards/debuggability.md](standards/debuggability.md) |
| Threat model, input limits, TLS/SASL/ACLs, dependencies | [standards/security.md](standards/security.md) |
| Test levels, simulation, fuzzing, compatibility | [standards/testing.md](standards/testing.md) |
| Configuration: Kafka key names, typed, validated, each value's source visible in `check-config` and `DescribeConfigs` | `alluvium-config` (M0) |

## 10. Beyond Kafka: foundation hooks

Alluvium is designed to lift Kafka's limitations ([ADR-0010](adr/0010-alluvium-is-its-own-platform.md)). The enhancements themselves come after the parity track, but these extension points are built during it, so the enhancements don't force a redesign. Each is a backlog feature labelled `foundation-for:<epic>`. Each has exactly one implementation until its enhancement lands, which is the sanctioned exception to "no speculative abstraction".

| Kafka limitation | Foundation (parity track) | Enhancement |
|---|---|---|
| Adding partitions breaks key ordering; partitions can't be removed | Key hash space and key ranges (TYPE-F11); key-range mapping records (META-F08) | Elastic partitions (X-ELASTIC) |
| Not a queue: no per-message ack, delay, retry, DLQ or priority | Per-record delivery state store (SHARE-F08) | Queue semantics (X-QUEUE) |
| Storage tied to brokers; costly cross-zone replication | Segment store abstraction (STOR-F02) | Diskless topics (X-DISKLESS), tiered storage (TIER) |
| No multi-tenancy | Namespace identity (TYPE-F10); namespace records (META-F07) | Multi-tenancy (X-TENANT) |
| Kafka protocol only | Protocol-agnostic service layer (SRV-F10) | Native HTTP/gRPC APIs (X-NATIVE) |
| Not a database: no key lookups | Append observer hook (STOR-F24) | Key lookups (X-KV) |
| ~1 MB messages; no schema enforcement; no end-to-end tracing | Reserved record-header conventions (REC-F13) | Large messages (X-LARGE), schema validation (X-SCHEMA), tracing (X-OBSX) |

Open design decisions are backlog `Spike` tasks that end in an ADR. Examples: storage I/O execution (STOR-F18), partition ownership (REPL-F01), fsync defaults (STOR-F17), GSSAPI (SEC-F06), and the design spike that opens each enhancement epic.

## 11. Repository layout (target)

```
.
├── AGENTS.md  CLAUDE.md  README.md  CONTRIBUTING.md
├── Cargo.toml              # workspace, [workspace.dependencies], [workspace.lints]
├── rust-toolchain.toml     # pinned toolchain
├── clippy.toml  deny.toml  # disallowed methods/types; dependency policy
├── .agents/skills/alluvium-engineering/SKILL.md
├── .github/                # CI workflows, PR template
├── crates/<crate>/         # see §4
├── fuzz/                   # cargo-fuzz targets, one per decoder
├── tests/compat/           # real-client and differential (vs Kafka 4.4.0) suites
├── schemas/kafka-4.4.0/    # vendored Kafka JSON message schemas (Apache-2.0)
├── xtask/
└── docs/
```
