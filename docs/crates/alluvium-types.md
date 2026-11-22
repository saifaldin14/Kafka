# `alluvium-types`: deep dive

## Responsibility and layer

`alluvium-types` is the workspace's shared vocabulary. It holds:
- identifier newtypes that make a swapped or out-of-range value a type error;
- the two kinds of time and the `Clock` that reads them;
- the seeded RNG and the hash maps keyed from it;
- the wrappers that keep secrets, record payloads and client strings from leaking into or forging logs.

It owns values, not behavior. Codecs live in `alluvium-protocol` and `alluvium-records`. Decisions about these values (who leads, what to fence) live in the layer 1–2 state machines.

It is layer 0 in the [code map](../architecture.md#4-code-map) and depends on no other Alluvium crate. Every crate may depend on it. Its outside dependencies are `chacha20`, `crc32c`, `getrandom`, `siphasher`, `thiserror` and `zeroize`, plus `proptest` behind the `proptest-strategies` feature. Each one is justified in its commit and governed by [`deny.toml`](../../deny.toml).

[`src/lib.rs`](../../crates/alluvium-types/src/lib.rs) lists every public type, grouped by purpose. Start there. This document is the map of where each thing lives and how it is checked.

| Module | Owns |
|---|---|
| [`id`](../../crates/alluvium-types/src/id.rs) | `NodeId`, `BrokerId`, `PartitionIndex`, `CorrelationId`, and `IdError`, which every numeric id and epoch returns |
| [`producer`](../../crates/alluvium-types/src/producer.rs) | `ProducerId`, `ProducerEpoch`, `Sequence` |
| [`epoch`](../../crates/alluvium-types/src/epoch.rs) | `LeaderEpoch`, `BrokerEpoch`, `MemberEpoch`, `GenerationId` |
| [`offset`](../../crates/alluvium-types/src/offset.rs) | `Offset`, `OffsetRange`, `OffsetSpec` |
| [`time`](../../crates/alluvium-types/src/time.rs), [`clock`](../../crates/alluvium-types/src/clock.rs), [`deadline`](../../crates/alluvium-types/src/deadline.rs), [`timer`](../../crates/alluvium-types/src/timer.rs) | `MonotonicInstant`, `WallTime`, `Clock`, `SystemClock`, `ManualClock`, `Deadline`, and the `Timer` trait that layer 3–5 code waits through ([ADR-0011](../adr/0011-time-model-and-clock-injection.md)) |
| [`rng`](../../crates/alluvium-types/src/rng.rs), [`collections`](../../crates/alluvium-types/src/collections.rs) | `SeededRng`, `SeededHashMap`, `SeededHashSet`, `HashSeed` ([ADR-0012](../adr/0012-seeded-randomness-and-hashing.md)) |
| [`uuid`](../../crates/alluvium-types/src/uuid.rs) | `Uuid` and `TopicId` |
| [`topic`](../../crates/alluvium-types/src/topic.rs), [`client_ids`](../../crates/alluvium-types/src/client_ids.rs), [`listener`](../../crates/alluvium-types/src/listener.rs) | `TopicName`; `GroupId`, `GroupInstanceId`, `TransactionalId`, `ClientId`; `ListenerName` |
| [`partition`](../../crates/alluvium-types/src/partition.rs) | `TopicPartition`, `TopicIdPartition` |
| [`partitioner`](../../crates/alluvium-types/src/partitioner.rs), [`key_range`](../../crates/alluvium-types/src/key_range.rs) | `murmur2`, `partition_for_key`; `KeyHash`, `KeyRange`, `check_coverage` ([ADR-0013](../adr/0013-key-hash-space.md)) |
| [`namespace`](../../crates/alluvium-types/src/namespace.rs) | `NamespaceId`, `NamespacedName` ([ADR-0014](../adr/0014-namespace-identity.md)) |
| [`delivery`](../../crates/alluvium-types/src/delivery.rs), [`record_attributes`](../../crates/alluvium-types/src/record_attributes.rs), [`cleanup_policy`](../../crates/alluvium-types/src/cleanup_policy.rs), [`byte_size`](../../crates/alluvium-types/src/byte_size.rs) | `Acks`, `IsolationLevel`; `TimestampType`, `CompressionType`; `CleanupPolicy`; `ByteSize` |
| [`secret`](../../crates/alluvium-types/src/secret.rs), [`payload`](../../crates/alluvium-types/src/payload.rs), [`log_safe`](../../crates/alluvium-types/src/log_safe.rs) | `Secret<T>`, `PayloadSummary`, `LogSafe` ([printing values](../standards/debuggability.md#printing-values)) |
| [`strategies`](../../crates/alluvium-types/src/strategies.rs) | proptest strategies, behind `proptest-strategies` ([shared strategies](../standards/testing.md#shared-strategies)), for the ids, epochs, offsets, times, UUIDs, names, partitions, key hashes and ranges, and namespace ids; and wire values for `Acks` and `IsolationLevel` |

## Kafka counterpart

The crate mirrors value rules from Kafka 4.4.0's clients and coordinators: sentinels, validation, printing and arithmetic. Each mirrored rule carries a `// Parity:` citation at the code that implements it, or `// Ported from:` where the code is a close port. `grep -nE "Parity:|Ported from:" crates/alluvium-types/src/*.rs` lists them all. The sources, by module:

| Module | Kafka 4.4.0 source |
|---|---|
| `id` | [`Node.java`][node] (`-1` is "no node") |
| `producer` | [`RecordBatch.java`][record-batch] (no-producer sentinels), [`DefaultRecordBatch.java`][default-record-batch] (sequence wrap-around), [`TransactionMetadata.java`][txn-metadata] (epoch exhaustion) |
| `epoch` | [`RecordBatch.java`][record-batch], [`ConsumerGroupHeartbeatRequest.java`][heartbeat], [`JoinGroupRequest.java`][join-group] |
| `offset` | [`OffsetFetchResponse.java`][offset-fetch] (`INVALID_OFFSET`), [`ListOffsetsRequest.java`][list-offsets] (special timestamps) |
| `clock`, `deadline` | [`SystemTime.java`][system-time], [`Timer.java`][timer] |
| `uuid` | [`Uuid.java`][uuid] (base64url form, signed ordering, reserved ids). Its tests are ported from [`UuidTest.java`][uuid-test] |
| `topic` | [`Topic.java`][topic] (name rules, collisions, internal topics). Its tests are ported from [`TopicTest.java`][topic-test] |
| `client_ids` | [`GroupCoordinatorService.java`][group-coordinator-service], [`JoinGroupRequest.java`][join-group], [`TransactionCoordinator.scala`][txn-coordinator] |
| `listener` | [`ListenerName.java`][listener-name] |
| `partition` | [`TopicPartition.java`][topic-partition], [`TopicIdPartition.java`][topic-id-partition] |
| `partitioner` | [`Utils.java`][utils] `murmur2` (closely ported, with its [`UtilsTest.java`][utils-test] vectors) and `toPositive`; [`BuiltInPartitioner.java`][built-in-partitioner] `partitionForKey` |
| `delivery` | [`IsolationLevel.java`][isolation-level], [`ReplicaManager.scala`][replica-manager] (`acks` values) |
| `record_attributes` | [`TimestampType.java`][timestamp-type], [`CompressionType.java`][compression-type] |
| `cleanup_policy` | [`ConfigDef.java`][config-def] `ValidList`, [`LogConfig.java`][log-config] |
| `payload` | [`Crc32C.java`][crc32c] |
| `secret` | [`Password.java`][password] |

The closely ported code in `partitioner`, `uuid` and `topic` carries Kafka's attribution header ([attributing Kafka material](../../CONTRIBUTING.md#attributing-kafka-material)) and is listed in [`NOTICE`](../../NOTICE).

Deliberate differences, each opt-in or outside Kafka's wire surface:
- The key hash space and key ranges ([ADR-0013](../adr/0013-key-hash-space.md)) and namespaces ([ADR-0014](../adr/0014-namespace-identity.md)) have no Kafka counterpart. A Kafka client only ever sees the default namespace.
- `Uuid` parsing counts bytes where Kafka counts UTF-16 units. They agree for ASCII, and both reject any other input; see the comment in `Uuid::from_str`.
- `Secret` is stricter than Kafka's [`Password`][password], which only hides its value in `toString()`. A `Secret` also wipes its value on drop, can't be compared or hashed, and wraps any zeroizable type.
- Client-chosen ids order by UTF-8 bytes, Java strings by UTF-16 units. The two differ only between characters above U+FFFF and those from U+E000 to U+FFFF, and nothing Kafka returns is sorted by these ids (`client_ids` module doc).

## State machines

None. The crate holds values and pure functions; the state machines that use them live in layers 1–2 ([ADR-0002](../adr/0002-sans-io-state-machines-driven-by-tokio.md)). Two types have state:
- `ManualClock` moves only when a test or simulation tells it to.
- `SeededRng` advances its stream as it is drawn from.

Neither has transitions to report.

## Invariants

| Invariant | Where it is enforced |
|---|---|
| An id holds a valid value only | Private fields. Values with invalid inputs are built only through validating constructors (`TryFrom`, `from_wire_nullable`, `MemberEpoch::from_wire`), which return `IdError`, `OffsetError` and the like. Types for which every input is valid convert infallibly: `CorrelationId` through `From<i32>`, `Uuid` through `from_bits`. Unit and property tests cover the rejections, and `strategies` produces only valid values |
| A wire sentinel is never stored as an id | `from_wire_nullable` maps `-1` (`NodeId`, `BrokerId`, `ProducerId`, `Offset`) and the zero UUID (`TopicId`) to `None`. `TopicId::try_from` rejects zero |
| Arithmetic on decoded values never overflows silently | `Offset::checked_add`, `checked_sub` and `next`, and `MonotonicInstant`/`WallTime::checked_add`, return errors. `Deadline::after` saturates, as Kafka's `Timer` does. `Sequence` wraps exactly as `DefaultRecordBatch.incrementSequence` does |
| Monotonic time and wall time are never converted into each other | No conversion exists. Two `compile_fail` doctests in `time` prove it (they run in the `doctest` CI step) |
| Only `SystemClock` and `SeededRng::from_os_entropy` read the operating system | clippy's `disallowed-methods` bans every other clock and entropy reader. The three sanctioned calls (two in `clock`, one in `rng`) are `#[expect]`ed, and [`xtask/tests/disallowed_exceptions.rs`](../../xtask/tests/disallowed_exceptions.rs) fails if a file's count of exceptions changes |
| A seed replays the same random stream forever | Golden tests in `rng`: `fixed_seeds_give_fixed_streams`, `forks_give_fixed_streams` and `forking_never_shifts_any_other_stream` |
| `murmur2` and `partition_for_key` equal Kafka's, bit for bit | `kafka_test_vectors`, and 10,000 golden inputs each in [`tests/golden/`](../../crates/alluvium-types/tests/golden/), generated by running Kafka 4.4.0's own code. `partition_for_key`'s `expect("invariant: …")` relies on `toPositive` clearing the sign bit |
| Key ranges are half-open, inside `[0, 2^31)`, and a range set covers the space exactly once | `KeyRange::new` rejects inverted and out-of-space ranges. `check_coverage` reports the first gap, overlap or empty range, and property tests in `key_range` cover removing, duplicating and shifting a range |
| A topic name or group instance id passes exactly Kafka's rules, and a namespace id exactly [ADR-0014](../adr/0014-namespace-identity.md)'s (1 to 63 of `[a-z0-9-]`) | Property tests in `topic`, `client_ids` and `namespace` compare any string against a model of the rules, alongside the ported `TopicTest` cases |
| Secrets are never printed and are zeroized on drop | `Secret`'s `Debug` and `Display` redact. Tests print a secret every way, including through a `tracing` span, and check that no part of it appears. Other tests check that dropping, including inside a container and from a clone, zeroizes it |
| Record payloads never appear in output | `PayloadSummary` prints only the length and the CRC32C. A property test checks that the output reveals nothing else |
| A client-chosen string can't forge a log line | `LogSafe` escapes line breaks, controls and bidirectional overrides. Property tests check that nothing needing escape survives, and that other text is unchanged |

## Events, metrics and flight recorder

None. The crate has no state transitions to report and no hot path of its own: it is called from other crates' hot paths, which own the events and metrics. The criterion benchmarks in [`benches/types.rs`](../../crates/alluvium-types/benches/types.rs) track the two per-record costs it adds: hashing a key with `murmur2`, and printing a `PayloadSummary`.

## Errors

Fallible constructors return a `thiserror` type from their module, some shared across types:
- `IdError`, with its `IdKind`, for every numeric id and epoch;
- `TopicNameError` for topic names, and for group instance ids, which follow the same rules;
- `OffsetError`, `TimeError`, `KeyRangeError`, `UuidError` (with `Base64Problem`), `NamespaceIdError`, `CleanupPolicyError`, `ByteSizeError`, `DeliveryError` and `AttributeError`;
- the structs `BlankListenerName`, and `NotInDefaultNamespace`, which carries the namespace;
- `WallClockError` and `EntropyError` for the two operating-system readers.

Variants carry the rejected value or its bounds where that is safe to log, and some, like `TopicNameError::Empty`, need no field. Client input that could be large is never echoed whole:
- `TopicNameError` omits the name, which can be 32 KiB; the handler, which has the name, builds Kafka's message.
- `UuidError` echoes at most 24 characters, Kafka's input limit.

The string fields that are echoed (`cleanup.policy` tokens, attribute names) print through `LogSafe`, so they can't forge a log line, but they aren't truncated.

There is no Kafka error-code mapping in this crate. The same rejected value means different Kafka errors depending on where it came from, for example `INVALID_TOPIC_EXCEPTION` from a `CreateTopics` request versus a corrupt metadata record. So the decoder or handler that rejects it owns the mapping ([ADR-0004](../adr/0004-error-model-and-panic-policy.md)).

## Tests

| Level | Where | Covers |
|---|---|---|
| Unit | `mod tests` at the bottom of each module except `strategies` | Kafka's examples, and the ported `UuidTest`, `TopicTest` and `UtilsTest` cases (each marked `// Ported from:`). Together with the property tests, they cover every error variant |
| Property | `proptest!` blocks in 20 of the modules | Round trips, rejection of every invalid value, and models of Kafka's string rules over arbitrary input. Failing cases persist under [`proptest-regressions/`](../../crates/alluvium-types/proptest-regressions/) and are committed |
| Golden | [`tests/golden/`](../../crates/alluvium-types/tests/golden/) and the `rng` tests | Kafka's own `murmur2` and `partitionForKey` outputs, and the RNG streams |
| Doctests | Examples in doc comments | Usage examples, and the `compile_fail` proofs in `time` |
| Benchmarks | [`benches/types.rs`](../../crates/alluvium-types/benches/types.rs) | `murmur2` and `PayloadSummary` throughput, on seeded inputs |
| Simulation, compatibility | None | The crate has no distributed behavior or client-visible surface of its own |
| Fuzz | None | The crate decodes no bytes from the network or disk. Its string parsers (`Uuid`, `TopicName`, `NamespaceId`, the client ids) are property-tested over arbitrary strings instead |

Commands:

```sh
cargo nextest run -p alluvium-types         # unit, property and golden tests
cargo test -p alluvium-types --doc          # doctests
cargo bench -p alluvium-types               # benchmarks
crates/alluvium-types/tests/golden/generate.sh  # regenerate the goldens from Kafka 4.4.0 (needs Docker)
```

A failing property test prints its minimal input, and proptest saves its seed in the module's `proptest-regressions/*.txt` file. Commit that file with the fix, so the case runs first from then on.

## Debugging

The crate's types show up in other crates' bugs more often than they cause their own. Start from the symptom:

- **An id or name was rejected.** The error's `Display` names the rule, and the value where it is safe to print. Find the constructor that returned it in the module table above, and the `// Parity:` citation beside it for the Kafka rule it follows.
- **A partition choice or key hash differs from Kafka's.** Rerun `generate.sh` to check the goldens still match Kafka 4.4.0, then compare `murmur2` with Kafka's `Utils.murmur2` line by line.
- **A run doesn't replay from its seed.** Check that nothing reads the OS clock or entropy (`cargo xtask ci --step clippy` and the disallowed-exceptions test) and that no `std` `HashMap` with `RandomState` is iterated. The `rng` golden tests show whether the stream itself changed.
- **A secret, payload or forged line appears in a log.** It was printed without its wrapper: `Secret`, `PayloadSummary` or `LogSafe`. Add the failing print to that module's tests first ([debugging guide](../guides/debugging.md)).

[node]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/Node.java
[record-batch]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/record/internal/RecordBatch.java
[default-record-batch]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java
[txn-metadata]: https://github.com/apache/kafka/blob/4.4.0/transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java
[heartbeat]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/requests/ConsumerGroupHeartbeatRequest.java
[join-group]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/requests/JoinGroupRequest.java
[offset-fetch]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/requests/OffsetFetchResponse.java
[list-offsets]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/requests/ListOffsetsRequest.java
[system-time]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/utils/SystemTime.java
[timer]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/utils/Timer.java
[uuid]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/Uuid.java
[uuid-test]: https://github.com/apache/kafka/blob/4.4.0/clients/src/test/java/org/apache/kafka/common/UuidTest.java
[topic]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/internals/Topic.java
[topic-test]: https://github.com/apache/kafka/blob/4.4.0/clients/src/test/java/org/apache/kafka/common/internals/TopicTest.java
[group-coordinator-service]: https://github.com/apache/kafka/blob/4.4.0/group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java
[txn-coordinator]: https://github.com/apache/kafka/blob/4.4.0/core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala
[listener-name]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/network/ListenerName.java
[topic-partition]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/TopicPartition.java
[topic-id-partition]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/TopicIdPartition.java
[utils]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/utils/Utils.java
[utils-test]: https://github.com/apache/kafka/blob/4.4.0/clients/src/test/java/org/apache/kafka/common/utils/UtilsTest.java
[built-in-partitioner]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/clients/producer/internals/BuiltInPartitioner.java
[isolation-level]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/IsolationLevel.java
[replica-manager]: https://github.com/apache/kafka/blob/4.4.0/core/src/main/scala/kafka/server/ReplicaManager.scala
[timestamp-type]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/record/TimestampType.java
[compression-type]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/record/internal/CompressionType.java
[config-def]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/config/ConfigDef.java
[log-config]: https://github.com/apache/kafka/blob/4.4.0/storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java
[crc32c]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/utils/internals/Crc32C.java
[password]: https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/config/types/Password.java
