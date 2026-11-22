# Debuggability standard

Any problem in Alluvium should be diagnosable from what the system already records, without adding print statements and redeploying. There are five sources of evidence, and every feature must feed all of them that apply:

1. **Traces and logs**: what happened, in which request, for which partition.
2. **Metrics**: how often, how slow, how full.
3. **Flight recorders**: the recent state transitions of a specific partition, group or quorum.
4. **On-disk state**, readable with offline tools.
5. **Deterministic reproduction**: a simulation seed or a recorded input sequence.

Design decisions are in [ADR-0005](../adr/0005-observability-model.md).

## Tracing

Built on `tracing`. The binary installs the subscriber: JSON lines in production, human-readable output in development. The filter uses `RUST_LOG` syntax from the `ALLUVIUM_LOG` environment variable or the `alluvium.log.filter` config. It can be changed at runtime through the diagnostics endpoint, and through Kafka's `BROKER_LOGGER` config resource (`IncrementalAlterConfigs`), as in Kafka.

### Span hierarchy

```
node{node_id, roles}
└── connection{conn_id, listener, peer_addr}
    └── request{api, api_version, correlation_id, client_id, principal}
        └── partition{topic, partition}          # per-partition work inside a request
driver{component, topic?, partition?, group_id?}  # background drivers (fetchers, coordinators, raft)
```

Every log line inside a request therefore carries the connection, request and partition fields with no extra effort. To follow one client request end to end, filter on `conn_id` and `correlation_id`.

### Standard field names

Use exactly these names, so a single query works across all components:

| Field | Meaning |
|---|---|
| `node_id`, `broker_id` | Node identity; broker identity in broker-specific contexts |
| `conn_id` | Connection id (`<local addr>-<peer addr>-<seq>`, like Kafka's) |
| `listener` | Listener name |
| `peer_addr` | Remote socket address |
| `api`, `api_version` | Kafka API name (`Produce`) and version |
| `correlation_id`, `client_id` | From the request header |
| `principal` | Authenticated principal (`User:alice`) |
| `topic`, `topic_id`, `partition` | Partition identity |
| `offset`, `fetch_offset`, `base_offset`, `last_offset` | Positions |
| `log_end_offset`, `high_watermark`, `last_stable_offset`, `log_start_offset` | Partition watermarks |
| `leader_epoch`, `leader_id`, `isr` | Replication state |
| `segment_base_offset` | Segment identity |
| `group_id`, `member_id`, `generation_id`, `member_epoch` | Group state |
| `producer_id`, `producer_epoch`, `sequence`, `transactional_id` | Producer state |
| `error_code` | Kafka error name (`NOT_LEADER_OR_FOLLOWER`) |
| `reason` | Reason enum for a transition or decision |
| `elapsed_ms` | Duration of the operation being reported |

New fields are added to this table in the same PR that introduces them.

### Log levels

| Level | Use for | Examples |
|---|---|---|
| `error` | Operator action needed, or data at risk | Log directory failure, corrupted segment found, invariant about to abort |
| `warn` | Unexpected but handled; degraded | ISR below `min.insync.replicas`, TLS handshake failures from a peer, request timed out |
| `info` | Lifecycle and cluster-level state transitions | Startup/shutdown steps, leader or ISR change, segment roll, retention deletion, group rebalance, quorum leader change |
| `debug` | Per-request decisions and component internals | Request completed (one line with timings), delayed operation completed, fetch session updates |
| `trace` | Per-batch and per-record detail | Index lookups, batch validation steps |

Never log at `info` or above once per request or per record. That turns logs into noise and costs throughput.

### Events for state transitions

Every state change that matters for diagnosis is an **event** with a stable name, the old and new values, and a `reason`:

```rust
info!(
    event = "isr_shrink",
    topic = %tp.topic, partition = tp.partition.get(),
    leader_epoch = epoch.get(),
    old_isr = ?old, new_isr = ?new,
    reason = %IsrChangeReason::FollowerLagged { broker_id, lag_ms },
);
```

- Event names are `snake_case`, stable, and listed in each crate's `events.rs` along with what triggers them. Operators alert on them, so renaming one is a breaking change.
- `reason` is an enum. Its `Display` output is stable `snake_case` plus its fields. Kafka uses the same idea (for example `LogStartOffsetIncrementReason`).
- Layer 0–2 state machines *return* transitions. The driver logs them, records them in the flight recorder and updates metrics. This keeps the pure code free of side effects and ensures no transition goes unrecorded.

## Flight recorders

Each partition, group, transaction coordinator partition, Raft quorum and connection keeps a bounded ring buffer of its recent transitions (default 256 entries). Each entry holds: clock timestamp, input kind, summarized old state → new state, and reason.

- Readable live at `/debug/...` on the diagnostics endpoint.
- Written to the log when the component hits an error, and by the panic hook (together with the backtrace and active span stack) before the process aborts.
- Fixed memory: entries are small enums, not strings, and are formatted only when read.

## Metrics

Built on `prometheus-client` with an **explicitly passed registry**. There is no global state, so tests can assert on metrics.

- Names follow `alluvium_<subsystem>_<name>_<unit>`. Counters end in `_total`, durations in `_seconds`, sizes in `_bytes`.
- Every metric that corresponds to a Kafka metric is listed in `docs/metrics.md` (created with the first metric) next to its Kafka JMX name, so existing dashboards can be translated.
- **Label cardinality is bounded.** `topic` labels are allowed. `partition` labels only appear when `alluvium.metrics.per.partition.enabled` is set. A `client_id` label is allowed only on quota metrics, with a cap on distinct values. Never use unbounded labels (offsets, member ids, connection ids).
- **Every queue** exports depth, capacity and rejections (or mute events).
- **Every API** exports request count, error count by `error_code`, and latency histograms split by phase, mirroring Kafka's request metrics: queue time, local time, remote time (waiting in purgatory), throttle time, response queue time, response send time, total. These phases are what tell you where latency comes from.

## Diagnostics endpoint

A separate HTTP listener. It binds to `127.0.0.1` by default. If it binds anywhere else, it requires mTLS or a bearer token, and the server refuses to start without one ([security](security.md)). It is read-only except for the log level.

| Path | Returns |
|---|---|
| `/metrics` | Prometheus exposition |
| `/health/live`, `/health/ready` | Liveness; readiness (caught up on metadata, logs loaded) |
| `/debug/config` | Effective config with each value's source (default, file, env, dynamic); secrets redacted |
| `/debug/tasks` | Supervised tasks: name, state, age |
| `/debug/partitions/{topic}/{partition}` | Replication and log state plus flight recorder |
| `/debug/groups/{group_id}` | Group state plus flight recorder |
| `/debug/raft` | Quorum state plus flight recorder |
| `/debug/log-level` | `GET` the current filter; `PUT` a new filter |

Development builds with the `tokio-console` feature also expose task internals to `tokio-console`.

## Offline tools

Part of the `alluvium` binary. Each prints human-readable output by default, and JSON with `--json`.

| Command | Purpose | Kafka equivalent |
|---|---|---|
| `alluvium dump log <segment>` | Batches and records (`--payload` to include values; off by default) | `kafka-dump-log.sh` |
| `alluvium dump index <file>` | Offset or time index entries | `kafka-dump-log.sh` |
| `alluvium dump producer-state <snapshot>` | Producer state snapshot | `kafka-dump-log.sh` |
| `alluvium dump metadata <log dir>` | Metadata log records and snapshots | `kafka-metadata-shell.sh` |
| `alluvium verify log <partition dir>` | CRCs, offset monotonicity, index ↔ log consistency | `kafka-dump-log.sh --verify-index-only` |
| `alluvium check-config <file>` | Validate config and print effective values with sources | none |
| `alluvium debug-bundle` | Collect redacted config, versions, metrics snapshot, flight recorders and log-dir listing for a bug report | none |

On-disk formats are documented in `docs/formats/` (created with storage in M2) precisely enough to decode with a hex dump.

## Reproduction

- Every simulation test prints its seed on failure, along with the exact command to replay it: `ALLUVIUM_SIM_SEED=<seed> cargo nextest run <test>`.
- Production RNG keys and hash seeds are the opposite: never logged, printed, exported or shown on a debug endpoint, since they would let an attacker flood hash maps or predict timeouts. `SeededRng` and `HashSeed` print no state in `Debug` ([ADR-0012](../adr/0012-seeded-randomness-and-hashing.md)).
- Because core logic is sans-I/O, drivers can **journal the inputs** to a state machine (off by default; metadata only, never payloads). A journal can be replayed locally against the same code to reproduce a production state exactly.

## Printing values

- Every type implements `Debug` (lint-enforced).
- `Secret<T>` prints `Secret(***)` in every form: `{:?}`, `{:#?}`, `{}`, inside a derived `Debug`, and as a `?` or `%` field of a `tracing` event or span. The property test `alluvium_types::secret::tests::no_printing_reveals_any_part_of_the_secret` checks all of them for random values.
- Payload bytes are printed only through [`alluvium_types::PayloadSummary`](../../crates/alluvium-types/src/payload.rs), which shows their length and checksum and never their contents. Wrap every payload-bearing field in log output with it.
- Never log credentials, SASL tokens, record keys, values or headers. Topic names, group ids, client ids and principals may be logged, as Kafka does.
- Strings a client chooses (group, instance, transactional and client ids, and any other free-form name) print through `alluvium_types::LogSafe`, and the `Debug` and `Display` of their types use it. It escapes control characters, U+2028 and U+2029, and the bidirectional formatting characters as `\n`, `\r`, `\t` or `\u{hex}`, so a value can't start a fake log line or make the rest of a line read backwards. Everything else, including `\` and printable Unicode, prints unchanged. Topic names need no escaping: their alphabet is `[a-zA-Z0-9._-]`.
