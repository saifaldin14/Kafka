# TYPE: Domain types

> Crates: `alluvium-types` (layer 0) · Kafka counterpart: `common/TopicPartition`, `common/Uuid`, `server/util/Time` · Milestones: M0-M2

Shared domain vocabulary: identifier newtypes, validated names, time and randomness injection, secrets, and the type-level foundations for namespaces and key ranges.

## [Done] TYPE-F01 Identifier newtypes
P0 · M0 · Feature · deps: FND-F02
Scope: `BrokerId`, `NodeId`, `PartitionIndex`, `ProducerId`, `ProducerEpoch`, `Sequence`, `LeaderEpoch`, `BrokerEpoch`, `MemberEpoch`, `GenerationId`, `CorrelationId` with checked construction and `Debug`/`Display`.

### [Done] TYPE-0001 Create `alluvium-types` (layer 0) with `NodeId` and `BrokerId`: `i32` newtypes, `TryFrom<i32>` rejecting negatives, `get()`, and nullable wire mapping (`-1` → `None`)
P0 · M0 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/Node.java @4.4.0
Done when: The crate lives at `crates/alluvium-types` (the workspace switches to `members = ["crates/*", "xtask"]`), inherits `[workspace.lints]`, and its `lib.rs` `//!` doc covers responsibility, layer 0, Kafka counterparts at 4.4.0, invariants and entry-point types; `from_wire_nullable(-1)` is `Ok(None)` and other negatives return `IdError::Negative { kind, value }`; `Debug` prints `NodeId(3)` and `Display` prints `3`; the doc comment explains when to use `BrokerId` and when `NodeId` (glossary terms); `check-layers` passes.

### [Done] TYPE-0002 Shared proptest strategies: add `proptest` to `[workspace.dependencies]` and a `proptest-strategies` Cargo feature exposing `alluvium_types::strategies`, starting with `NodeId`/`BrokerId` biased towards 0, 1 and `i32::MAX`
P0 · M0 · Task · deps: TYPE-0001 · labels: test · ref: docs/standards/testing.md#test-levels
Done when: docs/standards/testing.md states where shared strategies live (in the owning crate behind `proptest-strategies`, not in `alluvium-testkit`, because a dev-dependency cycle would compile two copies of the types); property tests show `TryFrom<i32>` accepts exactly the non-negative values; the `proptest` justification follows security.md#dependencies; the module is absent without the feature.

### [Done] TYPE-0003 `PartitionIndex` (non-negative `i32`, `checked_next`) and `CorrelationId` (any `i32`, no validation, as in the request header)
P0 · M0 · Task · deps: TYPE-0002 · ref: clients/src/main/java/org/apache/kafka/common/TopicPartition.java @4.4.0
Done when: Unit tests cover 0, `i32::MAX` and negatives for `PartitionIndex`, and the overflow of `checked_next` at `i32::MAX`; `CorrelationId` round-trips every `i32`, including negatives; strategies for both are added; `Display` prints the plain number.

### [Done] TYPE-0004 `ProducerId` (non-negative `i64`, `NO_PRODUCER_ID = -1` → `None`) and `ProducerEpoch` (non-negative `i16`, `-1` → `None`) with `is_exhausted()` (epoch ≥ `i16::MAX - 1`) and `checked_bump()`
P0 · M0 · Task · deps: TYPE-0002 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java isEpochExhausted @4.4.0
Done when: `// Parity:` cites `TransactionMetadata.isEpochExhausted` and `RecordBatch.NO_PRODUCER_ID`/`NO_PRODUCER_EPOCH`; tests cover epochs 32765, 32766 and 32767 for `is_exhausted` and `checked_bump` (which returns `None` at `i16::MAX`); strategies cover the boundaries.

### [Done] TYPE-0005 `Sequence` (non-negative `i32`, `NO_SEQUENCE = -1` → `None`) with `wrapping_add(n)` and `wrapping_sub(n)` matching `DefaultRecordBatch.incrementSequence`/`decrementSequence` (wrap from `i32::MAX` to 0)
P0 · M0 · Task · deps: TYPE-0002 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java incrementSequence @4.4.0
Done when: Ported cases cover wrap-around at `i32::MAX`; a property test shows `wrapping_sub(wrapping_add(s, n), n) == s` for all valid `s` and `n`; `// Parity:` cites both Kafka methods.

### [Done] TYPE-0006 `LeaderEpoch` (non-negative `i32`, `NO_PARTITION_LEADER_EPOCH = -1` → `None`, total order) and `BrokerEpoch` (non-negative `i64`, `-1` → `None`)
P0 · M0 · Task · deps: TYPE-0002 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/RecordBatch.java NO_PARTITION_LEADER_EPOCH @4.4.0
Done when: Comparisons are by numeric value (the rule "a replica rejects a stale epoch" relies on them); unit tests cover the sentinels, the boundaries and other negatives; strategies are added.

### [Done] TYPE-0007 `MemberEpoch` with the KIP-848 sentinels as an enum (`Join` = 0, `Leave` = -1, `LeaveStatic` = -2, `Epoch(n)` for n > 0) and `GenerationId` (`UNKNOWN_GENERATION_ID = -1` → `None`)
P0 · M0 · Task · deps: TYPE-0002 · ref: clients/src/main/java/org/apache/kafka/common/requests/ConsumerGroupHeartbeatRequest.java @4.4.0
Done when: Wire conversion round-trips every `i32` that Kafka accepts, and other negative values return a typed error; `// Parity:` cites `ConsumerGroupHeartbeatRequest` (`JOIN_GROUP_MEMBER_EPOCH`, `LEAVE_GROUP_MEMBER_EPOCH`, `LEAVE_GROUP_STATIC_MEMBER_EPOCH`) and `JoinGroupRequest.UNKNOWN_GENERATION_ID`; strategies cover every sentinel.

## [Done] TYPE-F02 Offsets and positions
P0 · M0 · Feature · deps: FND-F02
Scope: `Offset`, offset ranges, checked offset arithmetic, sentinel values (latest/earliest) as explicit enums.

### [Done] TYPE-0008 `Offset` (non-negative `i64`) with `checked_add`/`checked_sub` returning `OffsetError::Overflow { offset, delta }`, `next()`, and nullable wire mapping (`-1` → `None`, as in `OffsetFetchResponse.INVALID_OFFSET`)
P0 · M0 · Task · deps: TYPE-0002 · ref: clients/src/main/java/org/apache/kafka/common/requests/OffsetFetchResponse.java INVALID_OFFSET @4.4.0
Done when: Property tests show that checked arithmetic never panics and agrees with `i128` reference arithmetic over the whole `i64` range; `next()` at `i64::MAX` returns an error; `Display` prints the plain number; strategies are biased to 0, 1, `i32::MAX` and `i64::MAX`.

### [Done] TYPE-0009 `OffsetRange`: half-open `[start, end)` with `contains`, `len() -> u64`, `is_empty`, `split_at(offset)` and `intersect`, rejecting `start > end` at construction
P0 · M0 · Task · deps: TYPE-0008
Done when: Property tests compare every operation with a naive model over small ranges and check that `split_at` gives two adjacent ranges whose union is the original; the `//!` doc states that ranges are half-open and why (no `end + 1` overflow).

### [Done] TYPE-0010 `OffsetSpec` enum for ListOffsets timestamps, with wire mapping: `Latest` (-1), `Earliest` (-2), `MaxTimestamp` (-3), `EarliestLocal` (-4), `LatestTiered` (-5), `EarliestPendingUpload` (-6), `Timestamp(ms)` (≥ 0)
P0 · M0 · Task · deps: TYPE-0002 · ref: clients/src/main/java/org/apache/kafka/common/requests/ListOffsetsRequest.java @4.4.0
Done when: A property test over all `i64` shows that decode then encode is the identity for accepted values and that every other negative returns a typed error; `// Parity:` cites the `ListOffsetsRequest` constants; the doc states that which spec each API version may send is checked by the handler (API epic), not here.

## [To Do] TYPE-F03 Kafka-compatible UUID
P0 · M0 · Feature · deps: FND-F02
Scope: `Uuid` and `TopicId` with Kafka's base64url display, reserved zero/metadata ids, generation from the injected RNG.

### [To Do] TYPE-0011 `Uuid`: Kafka's base64url `Display`, `FromStr` and signed ordering
P0 · M0 · Task · deps: TYPE-0002 · labels: decoder, security · ref: clients/src/main/java/org/apache/kafka/common/Uuid.java toString/fromString/compareTo @4.4.0
Done when: `Display` is unpadded base64url (22 characters); `FromStr` follows `Uuid.fromString` (rejects input over 24 characters before decoding, accepts optional padding, requires exactly 16 bytes); `Ord` matches Kafka's signed `compareTo`. Ported `UuidTest` cases pass; property tests show string round-trips for all values and that ordering equals signed comparison of the most significant then least significant bits; errors are a typed `UuidError` that echoes at most the first 24 characters of the input; no panic on any input; no new dependency unless justified per security.md#dependencies.

### [To Do] TYPE-0012 Reserved UUIDs and `TopicId`: `Uuid::ZERO` (null/unset, `ZERO_UUID`), `Uuid::ONE` (`ONE_UUID`), and a `TopicId` newtype with `TopicId::METADATA` (`METADATA_TOPIC_ID`) that cannot hold `ZERO`
P0 · M0 · Task · deps: TYPE-0011 · ref: clients/src/main/java/org/apache/kafka/common/Uuid.java RESERVED @4.4.0
Done when: `TopicId::try_from(Uuid::ZERO)` fails; nullable wire mapping turns `ZERO` into `None`; `TopicId::METADATA` displays as `AAAAAAAAAAAAAAAAAAAAAQ`; `// Parity:` cites `Uuid.RESERVED`.

### [To Do] TYPE-0013 `Uuid::random(&mut rng)` from the injected RNG: version-4/IETF-variant bits like `java.util.UUID.randomUUID`, retrying while the value is reserved or its string form contains `-`, as `Uuid.randomUuid` does
P0 · M0 · Task · deps: TYPE-0012, TYPE-0029 · ref: clients/src/main/java/org/apache/kafka/common/Uuid.java randomUuid @4.4.0
Done when: A fixed seed always produces the same sequence (golden values are committed); a property test over many seeds never yields a reserved UUID or a string containing `-`, and every value has version 4 and the IETF variant; nothing reads OS entropy.

## [To Do] TYPE-F04 Validated names
P0 · M0 · Feature · deps: TYPE-F01
Scope: `TopicName`, `GroupId`, `TransactionalId`, `ClientId`, `ListenerName` with Kafka's validation rules, internal-topic detection and `.`/`_` collision detection.

### [To Do] TYPE-0014 `TopicName` validated by `TryFrom<&str>`/`TryFrom<String>` with Kafka's rules in Kafka's order: empty, `.`, `..`, longer than 249, characters outside `[a-zA-Z0-9._-]`; `TopicNameError` has one variant per rule
P0 · M0 · Task · deps: none · labels: decoder, security · ref: clients/src/main/java/org/apache/kafka/common/internals/Topic.java detectInvalidTopic @4.4.0
Done when: Ported `TopicTest` cases pass; property tests show all strings over the legal alphabet with length 1-249 (except `.` and `..`) are accepted, and any string containing an illegal character or of length 250+ is rejected with the same rule Kafka reports first; strategies for valid names, invalid names per rule, and the 249/250 boundary are added; `// Parity:` cites `Topic.detectInvalidTopic`; mapping to `INVALID_TOPIC_EXCEPTION` is left to the handlers.

### [To Do] TYPE-0015 Internal topic constants and `TopicName::is_internal()`: `__consumer_offsets`, `__transaction_state` and `__share_group_state` are internal; `__cluster_metadata` is a separate constant that `is_internal` does not include, as in `Topic.isInternal`
P0 · M0 · Task · deps: TYPE-0014 · ref: clients/src/main/java/org/apache/kafka/common/internals/Topic.java isInternal @4.4.0
Done when: Tests assert exactly these three names are internal and that `__cluster_metadata` and look-alikes (`__consumer_offsets_`, `__Consumer_offsets`) are not; the doc explains the `__cluster_metadata` difference with a `// Parity:` citation; the `CLUSTER_METADATA_TOPIC_PARTITION` constant is added with TYPE-F05.

### [To Do] TYPE-0016 Topic name collision detection: `has_collision_chars()`, `collision_key()` (`.` replaced by `_`) and `collides_with(&other)`, mirroring `Topic.hasCollisionChars`, `unifyCollisionChars` and `hasCollision`
P0 · M0 · Task · deps: TYPE-0014 · ref: clients/src/main/java/org/apache/kafka/common/internals/Topic.java hasCollision @4.4.0
Done when: A property test shows `a.collides_with(b)` exactly when the collision keys are equal; tests cover `a.b` and `a_b`, identical names, and names without collision characters; strategies generate colliding pairs for CTRL topic-creation tests; the doc explains that the reason is metric-name collisions.

### [To Do] TYPE-0017 Log-safe display for client-supplied strings: one escaping routine used by the `Debug`/`Display` of every client-controlled name type, escaping control characters and non-printable code points so a value can't forge log lines
P0 · M0 · Task · deps: TYPE-0001 · labels: security · ref: docs/standards/debuggability.md#printing-values
Done when: A property test shows the output never contains `\n`, `\r` or other control characters, for any input; printable ASCII and Unicode are unchanged; the escaping format is documented in debuggability.md "Printing values"; TYPE's group, transactional, client and instance id types use it.

### [To Do] TYPE-0018 `GroupId` (any string; `is_empty()` because coordinators reject empty ids with `INVALID_GROUP_ID`) and `GroupInstanceId` (validated with the topic-name rules, as `JoinGroupRequest.validateGroupInstanceId` does)
P0 · M0 · Task · deps: TYPE-0014, TYPE-0017 · ref: clients/src/main/java/org/apache/kafka/common/requests/JoinGroupRequest.java validateGroupInstanceId @4.4.0
Done when: An empty `GroupId` can be built (the rejection belongs to the group coordinator, citing `GroupCoordinatorService.isGroupIdNotEmpty`); `GroupInstanceId` rejects exactly what `TopicName` rejects; both use the log-safe display; strategies are added.

### [To Do] TYPE-0019 `TransactionalId` (non-empty: the nullable wire value maps `null` to `None`, and an empty string is a typed error the handler maps to Kafka's response) and `ClientId` (any string, including empty)
P0 · M0 · Task · deps: TYPE-0017 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala handleInitProducerId @4.4.0
Done when: Tests cover null, empty and non-empty transactional ids, with a `// Parity:` citation of the `TransactionCoordinator` null/empty branches; `ClientId` accepts every string, including control characters, and displays them escaped; strategies are added.

### [To Do] TYPE-0020 `ListenerName::normalised(&str)`: rejects blank values (Kafka's `Utils.isBlank`) and converts to uppercase as `toUpperCase(Locale.ROOT)` does; equality is case-sensitive after normalization
P0 · M0 · Task · deps: TYPE-0001 · ref: clients/src/main/java/org/apache/kafka/common/network/ListenerName.java normalised @4.4.0
Done when: Tests cover `plaintext` → `PLAINTEXT`, the empty string, whitespace-only input, and a non-ASCII case documented against Java's behavior; `// Parity:` cites `ListenerName.normalised`.

## [To Do] TYPE-F05 Partition identities
P0 · M0 · Feature · deps: TYPE-F03, TYPE-F04
Scope: `TopicPartition`, `TopicIdPartition`, ordering and display conventions.

### [To Do] TYPE-0021 `TopicPartition { topic: TopicName, partition: PartitionIndex }`: `Display` as `<topic>-<partition>` (Kafka's `toString`), `Ord` by topic then partition, and the `__cluster_metadata-0` constant (`Topic.CLUSTER_METADATA_TOPIC_PARTITION`)
P0 · M0 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/TopicPartition.java toString @4.4.0
Done when: Tests assert the display format and that a `BTreeSet` of partitions iterates in (topic, partition) order; strategies generate sets of partitions over few topics, for collisions; `// Parity:` cites `TopicPartition.toString`.

### [To Do] TYPE-0022 `TopicIdPartition { topic_id: TopicId, partition: PartitionIndex, topic: Option<TopicName> }` for contexts where only the id is known (Fetch v13+); `Display` as `<topic_id>:<topic or null>-<partition>`; equality over all fields as in Kafka
P0 · M0 · Task · deps: TYPE-0021 · ref: clients/src/main/java/org/apache/kafka/common/TopicIdPartition.java @4.4.0
Done when: Tests cover the display with and without a name and equality when the name differs; the doc explains when the name is `None` and that resolving it is the metadata image's job (META); strategies are added.

## [In Progress] TYPE-F06 Clock injection
P0 · M0 · Feature · deps: FND-F02
Scope: `Clock` trait, `SystemClock` (the one sanctioned `Instant::now`), `ManualClock` for tests, wall vs monotonic time types.

### [Done] TYPE-0023 Spike: time model and clock injection ADR
P0 · M0 · Spike · deps: none · ref: ADR-0002
Done when: Scope: the `Clock` trait surface (monotonic and wall readings), the monotonic and wall-clock types, the representation of Kafka's millisecond timestamps, and how drivers wait on timers under `SystemClock` and in simulation without `tokio::time::sleep`. An accepted ADR "Time model and clock injection" decides the trait methods, the types and their units, the sanctioned `#[expect]` call sites, and the timer approach that SRV-F02/SRV-F07 and TEST-F01 will use, citing Kafka's `Time.milliseconds()`/`nanoseconds()` split.
Notes: Accepted as [ADR-0011](../adr/0011-time-model-and-clock-injection.md). It added TYPE-0048, SRV-0149, TEST-0131, FND-0064 and FND-0065.

### [Done] TYPE-0024 Time value types from ADR-0011: `MonotonicInstant` (`u64` nanoseconds since the clock's origin, never goes backwards) and `WallTime` (non-negative `i64` Unix epoch milliseconds) (Kafka record and config timestamps), with checked arithmetic and no implicit conversion between them
P0 · M0 · Task · deps: TYPE-0023, TYPE-0001 · ref: ADR-0011, clients/src/main/java/org/apache/kafka/common/utils/Time.java @4.4.0
Done when: Adding a `Duration` to either type is checked and returns an error at overflow; `saturating_duration_since` never panics; there is no `From` conversion between the two types (a compile-fail doc test shows it); `Display` for wall time prints the millisecond value and an RFC 3339 UTC form.

### [Done] TYPE-0025 `Clock` trait and `SystemClock`: the only callers of `Instant::now` and `SystemTime::now`, each with one `#[expect(clippy::disallowed_methods, reason = "…")]`
P0 · M0 · Task · deps: TYPE-0024 · ref: clients/src/main/java/org/apache/kafka/common/utils/SystemTime.java @4.4.0
Done when: A grep check in the test suite finds no other `#[expect(clippy::disallowed_methods` for time functions in the workspace except `xtask`'s `Stopwatch` (the dev-tool exception in ADR-0011); `SystemClock` can convert a `MonotonicInstant` back to a `std::time::Instant` (for `TokioTimer`); a wall clock before the Unix epoch returns a typed error instead of panicking; `Clock` is `Send + Sync + Debug`; the module doc links to the ADR.

### [Done] TYPE-0026 `ManualClock` for tests and simulation: starts at a given monotonic and wall time, `advance(Duration)` moves both, and `set_wall(…)` makes the wall clock jump forwards or backwards without touching monotonic time
P0 · M0 · Task · deps: TYPE-0025 · labels: test · ref: clients/src/testFixtures/java/org/apache/kafka/common/utils/MockTime.java @4.4.0
Done when: Tests show the monotonic reading never decreases under any sequence of `advance`/`set_wall` (a property test), that wall jumps are visible, and that clones share state; it is usable from other crates' tests without enabling a feature, and the doc explains why it is not test-gated (the simulation in `alluvium-testkit` builds on it).

### [Done] TYPE-0027 `Deadline`: a monotonic instant plus a timeout with saturating construction (a timeout that overflows means "never"), `remaining(now)` and `is_expired(now)`, as Kafka's `Timer` saturates
P0 · M0 · Task · deps: TYPE-0024 · ref: clients/src/main/java/org/apache/kafka/common/utils/Timer.java @4.4.0
Done when: Tests cover a zero timeout, `Duration::MAX`, an expired deadline, and `remaining` at and after expiry (zero, never negative); `// Parity:` cites `Timer`; the doc ties it to the rule "every network wait has a timeout".

### [To Do] TYPE-0048 `Timer` trait: `sleep_until(MonotonicInstant)` returning a boxed `Send` future, with no async-runtime dependency, the one way layer 3–5 code waits (ADR-0011)
P0 · M2 · Task · deps: TYPE-0024 · ref: ADR-0011
Done when: The trait is `Send + Sync + Debug` and object-safe (`Arc<dyn Timer>` compiles); its doc says state machines never wait and links ADR-0011; `alluvium-types` gains no dependency; a unit test polls a hand-written implementation's future to completion without a runtime.

## [In Progress] TYPE-F07 Seeded randomness and deterministic collections
P0 · M0 · Feature · deps: FND-F02
Scope: Injected RNG trait, `SeededHashMap`/`SeededHashSet`, process-random seed in production, fixed seed in simulation.

### [Done] TYPE-0028 Spike: seeded randomness and hashing ADR
P0 · M0 · Spike · deps: none · labels: security · ref: ADR-0003
Done when: Scope: the PRNG algorithm (stable output across platforms and crate versions), the RNG trait surface, deterministic per-component sub-streams, the keyed hasher for `SeededHashMap` (resistant to hash flooding), and the seed sources in production and simulation. An accepted ADR "Seeded randomness and hashing" names the crates (justified per security.md#dependencies), the single sanctioned OS-entropy call site, and the rule that hash seeds are never logged (DoS protection) while simulation seeds always are.
Notes: Accepted as [ADR-0012](../adr/0012-seeded-randomness-and-hashing.md). It added the `getrandom` bans to FND-0064.

### [To Do] TYPE-0029 Injected RNG: a deterministic RNG type with `from_seed(u64)` and `fork(label)` for independent sub-streams, plus `SeededRng::from_os_entropy()` (`getrandom::fill`) as the only entropy source, with one `#[expect(clippy::disallowed_methods, reason = "…")]`
P0 · M0 · Task · deps: TYPE-0028, TYPE-0001 · ref: ADR-0012
Done when: The type is `SeededRng` (ChaCha12 via `chacha20`, with `from_seed`'s SplitMix64 key expansion and `below` as ADR-0012 specifies); `chacha20` and `getrandom` are justified per security.md#dependencies; `Debug` prints no key or position; golden tests commit the first values for fixed seeds and forks and pass on Linux and macOS; `fork` with the same parent seed and label always gives the same stream, and adding a new fork doesn't change existing ones; no other code calls `rand::rng` (the lint canary already proves the lint fires).

### [To Do] TYPE-0030 `alluvium_types::collections::{SeededHashMap, SeededHashSet}` using SipHash-1-3 (`siphasher`), keyed by a `HashSeed` drawn from the injected RNG and passed to `with_seed`/`with_capacity_and_seed`
P0 · M0 · Task · deps: TYPE-0029 · labels: security · ref: ADR-0012
Done when: `siphasher` is justified per security.md#dependencies; `HashSeed`'s `Debug` prints `HashSeed(***)`; the same seed gives the same iteration order across runs (test); different seeds give different hashes for the same key (test); the module doc says iteration order must never reach output or decisions (use `BTreeMap` where order is observable); the clippy messages in clippy.toml point to these types, and the names match.

## [In Progress] TYPE-F08 Secrets
P0 · M0 · Feature · deps: FND-F02
Scope: `Secret<T>` with redacted `Debug`/`Display`, zeroize on drop, explicit `expose_secret()`.

### [Done] TYPE-0031 `Secret<T: Zeroize>`: `Debug` and `Display` print `Secret(***)`, the value is reachable only through `expose_secret()`, the value is zeroized on drop, and there is no `Serialize` impl
P0 · M0 · Task · deps: TYPE-0001 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/types/Password.java @4.4.0
Done when: `zeroize` is added to `[workspace.dependencies]` with the security.md#dependencies justification; `Clone` exists only where `T: Clone + Zeroize`, and both copies zeroize; the doc explains the difference from Kafka's `Password` (`[hidden]`) and the security.md rule that secrets never come from command-line arguments.

### [To Do] TYPE-0032 Redaction tests for `Secret<T>`: `{:?}`, `{:#?}`, `{}`, a derived `Debug` of a struct containing a secret, and capture by a `tracing` test subscriber of `?secret` and `%secret` fields
P0 · M0 · Task · deps: TYPE-0031 · labels: security, test · ref: docs/standards/security.md#secrets
Done when: For random secret values (a property test), none of the outputs contains the secret or any substring of 4 or more of its characters; `tracing` is a dev-dependency only; the test is named in debuggability.md "Printing values".

### [To Do] TYPE-0033 Zeroize-on-drop test for `Secret<T>` without `unsafe`: a test type whose `Zeroize` impl records the call proves drop zeroizes, including after `Clone` and when the secret sits inside a dropped container
P0 · M0 · Task · deps: TYPE-0031 · labels: security, test · ref: docs/standards/security.md#secrets
Done when: The test fails if `Drop` stops calling `zeroize` (shown in the PR by temporarily removing it); no `unsafe` and no memory inspection are used.

## [In Progress] TYPE-F09 Payload display
P1 · M0 · Feature · deps: FND-F02
Scope: `PayloadSummary` that prints length and CRC32C instead of bytes, for logging payload-bearing types.

### [Done] TYPE-0034 `PayloadSummary<'a>(&'a [u8])` whose `Debug` and `Display` print `Bytes { len: <n>, crc32c: 0x<8 hex> }`, computing the CRC only when formatted
P1 · M0 · Task · deps: TYPE-0001 · labels: security · ref: docs/standards/debuggability.md#printing-values
Done when: `crc32c` is added to `[workspace.dependencies]` (it is in rust.md's preferred list); the CRC of `b"123456789"` is `0xe3069283`, checked with a `// Parity:` citation of `clients/src/main/java/org/apache/kafka/common/utils/internals/Crc32C.java`; the empty payload prints `len: 0`; the doc tells engineers to wrap every payload-bearing field in log output with it.

### [To Do] TYPE-0035 Payload non-disclosure tests: for random payloads, the summary output never contains the payload bytes as text, hex or base64, and formatting a 1 MiB payload stays within a time budget set by a criterion benchmark
P1 · M0 · Task · deps: TYPE-0034 · labels: security, test · ref: docs/standards/debuggability.md#printing-values
Done when: The property test covers ASCII, UTF-8 and binary payloads; the benchmark result is in the PR; debuggability.md links to `PayloadSummary` instead of describing the format a second time.

## [To Do] TYPE-F10 Namespace identity
P1 · M0 · Feature · deps: TYPE-F04 · labels: foundation-for:X-TENANT
Scope: `NamespaceId`, namespace-qualified names and the default-namespace mapping that keeps plain Kafka names valid.

### [To Do] TYPE-0036 Spike: namespace identity ADR, limited to the types
P1 · M0 · Spike · deps: none · labels: foundation-for:X-TENANT · ref: ADR-0010
Done when: Scope: `NamespaceId` representation and validation, the default namespace, how a namespace-qualified topic, group or transactional id relates to Kafka's 249-character limit, and the promise that every valid Kafka name is valid unchanged in the default namespace. An accepted ADR "Namespace identity" records these decisions and states that X-TENANT-F01 owns the full tenancy model and may supersede it; nothing in it changes behavior visible to Kafka clients (ADR-0010).

### [To Do] TYPE-0037 `NamespaceId` type per the ADR, with `NamespaceId::DEFAULT`, validated construction, a typed error and log-safe display
P1 · M0 · Task · deps: TYPE-0036 · labels: foundation-for:X-TENANT
Done when: Property tests over the validation rules pass; `DEFAULT` round-trips through `Display`/`FromStr`; META-F07 can use the type unchanged (its record fields are listed in the PR).

### [To Do] TYPE-0038 `NamespacedName` for topics, groups and transactional ids, with the default-namespace mapping: a plain Kafka name maps to `(DEFAULT, name)` and back without loss
P1 · M0 · Task · deps: TYPE-0037 · labels: foundation-for:X-TENANT
Done when: A property test over all valid `TopicName`, `GroupId` and `TransactionalId` values shows the round-trip is the identity and the display of a default-namespace name equals the plain Kafka name; non-default names can't be converted to a plain Kafka name without an explicit call that returns a typed error.

## [To Do] TYPE-F11 Key hash space and key ranges
P1 · M0 · Feature · deps: FND-F02 · labels: foundation-for:X-ELASTIC
Scope: `KeyHash`, `KeyRange`, range arithmetic (split/merge/contains), murmur2-compatible hashing for Kafka partitioners.

### [To Do] TYPE-0039 `murmur2(&[u8]) -> i32`: a port of Kafka's `Utils.murmur2` (seed `0x9747b28c`, `m = 0x5bd1e995`, `r = 24`, little-endian 4-byte blocks, tail of 1-3 bytes, final mix) with wrapping arithmetic
P1 · M0 · Task · deps: TYPE-0001 · labels: foundation-for:X-ELASTIC · ref: clients/src/main/java/org/apache/kafka/common/utils/Utils.java murmur2 @4.4.0
Done when: The `UtilsTest.testMurmur2` vectors pass (`"21"` → -973932308, `"foobar"` → -790332482, `"a-little-bit-long-string"` → -985981536, `"a-little-bit-longer-string"` → -1486304829, `"lkjh234lh9fiuh90y23oiuhsafujhadof229phr9h19h89h8"` → -58897971, `[a,b,c]` → 479470107); a golden file of 10,000 random inputs of length 0-1024 hashed by Kafka 4.4.0 matches, and its generator script (JShell against `kafka-clients` 4.4.0 in a digest-pinned image) is committed; no panic on any input; the file header carries the attribution required for a close port.

### [To Do] TYPE-0040 `partition_for_key(key, num_partitions: NonZeroU32) -> PartitionIndex` = `toPositive(murmur2(key)) % num_partitions`, where `toPositive(x) = x & 0x7fffffff` (`Utils.toPositive`)
P1 · M0 · Task · deps: TYPE-0039 · labels: foundation-for:X-ELASTIC · ref: clients/src/main/java/org/apache/kafka/clients/producer/internals/BuiltInPartitioner.java partitionForKey @4.4.0
Done when: A golden file of 10,000 (key, partition count 1-64) pairs computed by Kafka 4.4.0's `BuiltInPartitioner.partitionForKey` matches; tests cover the empty key and `i32::MIN` hashes; the doc says keyless records use the client's sticky partitioner and are never hashed; `// Parity:` cites both Kafka methods.

### [To Do] TYPE-0041 Spike: key hash space ADR
P1 · M0 · Spike · deps: TYPE-0040 · labels: foundation-for:X-ELASTIC · ref: ADR-0010
Done when: Options: `KeyHash` as Kafka's positive murmur2 (31 bits, `[0, 2^31)`) so key ranges line up with `partitionForKey`, or a wider hash; also how null keys are handled and how ranges map to a fixed Kafka partition count for the virtual partition view (X-ELASTIC-F10). An accepted ADR "Key hash space" fixes the hash function, the space bounds, and the compatibility argument, and states that X-ELASTIC-F01 may extend it but not change the hash.

### [To Do] TYPE-0042 `KeyHash` (a value in the ADR's hash space, `KeyHash::of(key)`) and `KeyRange` (half-open over the hash space) with `contains`, `split_at`, `merge` of adjacent ranges, `is_adjacent` and `FULL`
P1 · M0 · Task · deps: TYPE-0041 · labels: foundation-for:X-ELASTIC
Done when: Property tests show `split_at` followed by `merge` is the identity, that `split_at` rejects split points outside the range or at its ends, that `merge` of non-adjacent ranges returns a typed error, and that exactly one of two split halves contains any hash.

### [To Do] TYPE-0043 Key range coverage validation: check that a set of `KeyRange`s partitions the hash space exactly, and report the first gap or overlap with its bounds
P1 · M0 · Task · deps: TYPE-0042 · labels: foundation-for:X-ELASTIC
Done when: Property tests generate valid partitions by random splits (accepted) and then remove, duplicate or shift one range (rejected with the right gap or overlap); META-F08 can call it to validate key-range mapping records.

## [To Do] TYPE-F12 Small domain enums and size types
P1 · M0 · Feature · deps: FND-F02
Scope: `IsolationLevel`, `Acks`, `TimestampType`, `CompressionType`, `CleanupPolicy`, `ByteSize`, with Kafka wire values.

### [To Do] TYPE-0044 `IsolationLevel` (`ReadUncommitted` = 0, `ReadCommitted` = 1, wire `i8`) and `Acks` (`None` = 0, `Leader` = 1, `All` = -1, wire `i16`)
P1 · M0 · Task · deps: TYPE-0002 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala isValidRequiredAcks @4.4.0
Done when: Tests over every `i8`/`i16` value show only the listed values decode and all others return typed errors (the produce handler maps an invalid `acks` to `INVALID_REQUIRED_ACKS`, citing `ReplicaManager.isValidRequiredAcks`); `// Parity:` cites `IsolationLevel.forId`; strategies include the invalid values.

### [To Do] TYPE-0045 `TimestampType` (`CreateTime` = 0, `LogAppendTime` = 1, with Kafka's names `CreateTime`/`LogAppendTime`) and `CompressionType` (`None` = 0, `Gzip` = 1, `Snappy` = 2, `Lz4` = 3, `Zstd` = 4, with the names `none`, `gzip`, `snappy`, `lz4`, `zstd`)
P1 · M0 · Task · deps: TYPE-0002 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/CompressionType.java forId/forName @4.4.0
Done when: Id and name round-trips are tested for every variant; unknown ids and names return typed errors; the doc states that Kafka's `NO_TIMESTAMP_TYPE` (message formats v0/v1, not supported) is `Option::None` where it can appear, and that the topic-level `producer` compression value belongs to CFG; `// Parity:` cites `TimestampType` and `CompressionType`.

### [To Do] TYPE-0046 `CleanupPolicy`: the set of `{Delete, Compact}` built from a list of tokens (`delete`, `compact`, both in either order, or an empty list meaning no cleanup) with a canonical `Display`
P1 · M0 · Task · deps: TYPE-0002 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java @4.4.0
Done when: Tests cover each accepted list, unknown tokens (rejected with the token named), duplicates handled as Kafka's `ConfigDef.ValidList` does at 4.4.0, and the empty list (infinite retention, per `TopicConfig.CLEANUP_POLICY_DOC`); parsing the config string into tokens is CFG's job.

### [To Do] TYPE-0047 `ByteSize(u64)` with checked arithmetic, checked conversion from Kafka's `i32`/`i64` size fields (negative values rejected), and `Display` as exact bytes plus an IEC approximation (`1048576 (1 MiB)`)
P1 · M0 · Task · deps: TYPE-0002
Done when: Property tests show conversions never panic and reject exactly the negative inputs; arithmetic overflow returns a typed error; the doc states that config keys keep Kafka's plain byte numbers (no unit suffixes) for compatibility.
