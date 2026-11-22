# GRP: Group coordinator

> Crates: `alluvium-group-coordinator` (layer 2) · Kafka counterpart: `group-coordinator/` · Milestones: M5-M7

Consumer groups: the coordinator runtime over `__consumer_offsets`, classic and consumer (KIP-848) protocols, server-side assignors, offset storage, admin APIs and migration between protocols.

## [To Do] GRP-F01 Coordinator runtime
P0 · M5 · Feature · deps: STOR-F14, REPL-F12, CTRL-F04
Scope: Partitioned state machines over internal topic partitions, timeline (snapshot-at-offset) data structures, writes completing at the HW, load and unload.

### [To Do] GRP-0001 Decide the sans-I/O coordinator runtime design shared by the group, transaction and share coordinators
P0 · M5 · Spike · deps: none · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java @4.4.0
Done when: ADR `coordinator-runtime` accepted in docs/adr/ covering: shard state-machine interface, one event loop per coordinator partition, timeline snapshots keyed by log offset, responses released only at the high watermark, worker mapping of `group.coordinator.threads`, bounded event queues and the error returned when a queue is full (documented deviation if any).

### [To Do] GRP-0002 Timeline snapshot registry: create, idempotent create, revert to and delete snapshots keyed by offset
P0 · M5 · Task · deps: GRP-0001 · ref: server-common/src/main/java/org/apache/kafka/timeline/SnapshotRegistry.java @4.4.0
Done when: `SnapshotRegistry` supports `idempotent_create_snapshot`, `revert_to_snapshot`, `delete_snapshots_up_to` and `reset`; reverting to an unknown epoch panics as an invariant violation; unit tests ported from `SnapshotRegistryTest`.

### [To Do] GRP-0003 Timeline hash map and set with reads at a snapshot epoch
P0 · M5 · Task · deps: GRP-0002 · ref: server-common/src/main/java/org/apache/kafka/timeline/TimelineHashMap.java @4.4.0
Done when: `TimelineHashMap`/`TimelineHashSet` support get, insert, remove and iteration both at the latest state and at a given epoch; iteration order is deterministic; snapshot deltas are freed by `delete_snapshots_up_to`; unit tests cover read-at-epoch after later writes.

### [To Do] GRP-0004 Timeline scalar values: object, integer and long with revert support
P0 · M5 · Task · deps: GRP-0002 · ref: server-common/src/main/java/org/apache/kafka/timeline/TimelineObject.java @4.4.0
Done when: `TimelineObject<T>`, `TimelineInteger` and `TimelineLong` read the value at a given epoch and revert with the registry; unit tests cover set, revert and read-at-epoch.

### [To Do] GRP-0005 Property test of timeline structures against a copy-on-snapshot model
P0 · M5 · Task · deps: GRP-0003, GRP-0004 · labels: test · ref: server-common/src/main/java/org/apache/kafka/timeline/SnapshottableHashTable.java @4.4.0
Done when: a proptest applies random insert/remove/snapshot/revert/delete-up-to sequences and asserts every read-at-epoch equals a model built from cloned `BTreeMap`s; regressions committed under `proptest-regressions/`.

### [To Do] GRP-0006 Coordinator shard interface and result type
P0 · M5 · Task · deps: GRP-0001 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorShard.java @4.4.0
Done when: the shard trait exposes `replay(offset, producer_id, producer_epoch, record)`, `replay_end_transaction_marker`, `on_loaded`, `on_unloaded` and `on_metadata_update`; `CoordinatorResult` carries records, response, replay flag and atomicity like Kafka's `CoordinatorResult`; module docs state the single-writer invariant.

### [To Do] GRP-0007 Snapshottable coordinator wrapper tracking last written and last committed offsets
P0 · M5 · Task · deps: GRP-0002, GRP-0006 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/SnapshottableCoordinator.java @4.4.0
Done when: writing advances `last_written_offset` and takes a snapshot; committing advances `last_committed_offset` and deletes older snapshots; `revert_last_written_offset` restores state exactly; unit tests prove reads at the committed offset never see uncommitted writes.

### [To Do] GRP-0008 Per-partition coordinator state machine: INITIAL, LOADING, ACTIVE, FAILED, CLOSED
P0 · M5 · Task · deps: GRP-0006 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java CoordinatorState @4.4.0
Done when: allowed transitions match `CoordinatorState.canTransitionFrom` (LOADING from INITIAL or FAILED, ACTIVE from LOADING or ACTIVE, FAILED from LOADING or ACTIVE, CLOSED from any); an illegal transition panics; each transition is returned as a typed transition with a reason enum; unit test per allowed and rejected transition.

### [To Do] GRP-0009 Bounded per-partition event accumulator with one in-flight event per coordinator partition
P0 · M5 · Task · deps: GRP-0001 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/EventAccumulator.java @4.4.0
Done when: events for one partition run strictly in order and never concurrently; internal events can be enqueued at the front; capacity comes from the runtime config struct and overflow returns the error chosen in the ADR; closing fails queued events with NOT_COORDINATOR; property test proves per-key FIFO and mutual exclusion.

### [To Do] GRP-0010 Write operations: run on the active shard, replay records, defer the response until the high watermark
P0 · M5 · Task · deps: GRP-0007, GRP-0008, GRP-0009 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java scheduleWriteOperation @4.4.0
Done when: a write op runs only when the partition is ACTIVE; LOADING returns COORDINATOR_LOAD_IN_PROGRESS and FAILED/CLOSED/unknown returns NOT_COORDINATOR; generated records are replayed then appended to the open batch; the response completes only after the high watermark passes the batch's last offset; an op that produces no records completes after pending writes finish.

### [To Do] GRP-0011 Batch building, linger and flush for coordinator writes
P0 · M5 · Task · deps: GRP-0010 · labels: perf · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java maybeAllocateNewBatch @4.4.0
Done when: batches are bounded by the partition's `max.message.bytes`; a record that does not fit flushes the batch first; `group.coordinator.append.linger.ms` schedules a front-of-queue flush, and when unset the runtime flushes adaptively at the end of the queue; compression follows `offsets.topic.compression.codec`; buffers above `group.coordinator.cached.buffer.max.bytes` are discarded rather than cached.

### [To Do] GRP-0012 Atomic writes and failure rollback
P0 · M5 · Task · deps: GRP-0011 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java append @4.4.0
Done when: records of an atomic op land in one batch (a new batch is started if needed, oversized atomic writes go alone); a replay or append failure reverts the shard to the batch base offset and fails every deferred event of that batch with the mapped error; unit tests inject failures at each step.

### [To Do] GRP-0013 High watermark listener completing deferred events in offset order
P0 · M5 · Task · deps: GRP-0010 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java HighWatermarkListener @4.4.0
Done when: high watermark updates from the partition writer advance `last_committed_offset`, complete deferred events with offsets at or below it in order, and free snapshots; a lower high watermark than previously seen is ignored; tests cover out-of-order notifications.

### [To Do] GRP-0014 Read operations against the last committed state
P0 · M5 · Task · deps: GRP-0007 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java scheduleReadOperation @4.4.0
Done when: read ops observe state at `last_committed_offset`; `schedule_read_all` fans out to all active partitions and reports per-partition errors; unit test shows an uncommitted write is invisible to a concurrent read.

### [To Do] GRP-0015 Transactional writes and transaction completion in the coordinator runtime
P0 · M6 · Task · deps: GRP-0012, STOR-F15 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java scheduleTransactionalWriteOperation @4.4.0
Done when: transactional write ops flush the open batch first and append a transactional batch with the producer id, epoch and verification guard; `schedule_transaction_completion` writes a COMMIT or ABORT control batch with the coordinator epoch and calls `replay_end_transaction_marker`; tests cover commit, abort and failure revert.

### [To Do] GRP-0016 Write timeout for coordinator operations
P0 · M5 · Task · deps: GRP-0010 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java OperationTimeout @4.4.0
Done when: a write op not completed within `offsets.commit.timeout.ms` fails with REQUEST_TIMED_OUT (mapped to COORDINATOR_NOT_AVAILABLE by the error mapper); the timeout uses the injected clock; tested with a stalled high watermark.

### [To Do] GRP-0017 Keyed coordinator timers that run expirations as write operations
P0 · M5 · Task · deps: GRP-0010 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorTimerImpl.java @4.4.0
Done when: `schedule(key, delay, retry_on_failure, op)` replaces an existing timer with the same key, `cancel(key)` and `is_scheduled(key)` work; expirations run as write ops on the shard and retry after failure when requested; timers are cleared on unload; tests use the simulated clock.

### [To Do] GRP-0018 Coordinator executor for offloaded work whose results are applied as write operations
P0 · M5 · Task · deps: GRP-0010 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorExecutorImpl.java @4.4.0
Done when: tasks are keyed and de-duplicated (`is_scheduled`), run outside the shard event loop on a bounded pool sized by `group.coordinator.background.threads`, and apply their result through a write op; tasks are cancelled on unload and stale results are dropped; tests cover cancellation during unload.

### [To Do] GRP-0019 Coordinator error mapping in one exhaustive match
P0 · M5 · Task · deps: GRP-0010, ERR-F02 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorOperationExceptionHelper.java handleOperationException @4.4.0
Done when: NETWORK_EXCEPTION maps to COORDINATOR_LOAD_IN_PROGRESS; UNKNOWN_TOPIC_OR_PARTITION, NOT_ENOUGH_REPLICAS and REQUEST_TIMED_OUT map to COORDINATOR_NOT_AVAILABLE; NOT_LEADER_OR_FOLLOWER and KAFKA_STORAGE_ERROR map to NOT_COORDINATOR; MESSAGE_TOO_LARGE, RECORD_LIST_TOO_LARGE and INVALID_FETCH_SIZE map to UNKNOWN_SERVER_ERROR; unexpected errors are logged once at error level; one test per mapping.

### [To Do] GRP-0020 Metadata image delivery to active coordinator shards
P0 · M5 · Task · deps: GRP-0010, SRV-F14 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java onMetadataUpdate @4.4.0
Done when: the runtime registers as a metadata publisher (SRV-F14), passes each image and delta to every ACTIVE shard as an internal event in order, and new shards receive the current image on load; tests cover topic creation, deletion and partition count changes.

### [To Do] GRP-0021 Partition writer adapter over the replica manager for internal coordinator topics
P0 · M5 · Task · deps: GRP-0010, REPL-F03 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/PartitionWriter.java @4.4.0
Done when: appends go to the local leader of the coordinator partition with `acks=all`, carry the verification guard for transactional writes, return the last appended offset, expose the partition's `max.message.bytes`, and register high-watermark listeners; NOT_LEADER_OR_FOLLOWER and storage errors surface as typed errors.

### [To Do] GRP-0022 Tokio driver for the coordinator runtime with supervised named workers
P0 · M5 · Task · deps: GRP-0009, GRP-0013, GRP-0017, GRP-0018, GRP-0021, SRV-F02 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/MultiThreadedEventProcessor.java @4.4.0
Done when: `group.coordinator.threads` named worker tasks run under the task supervisor (SRV-F02) with a span per event; partitions hash to workers; shutdown stops intake, fails pending events with NOT_COORDINATOR, closes shards and logs each step with elapsed time; no blocking calls on Tokio workers.

### [To Do] GRP-0023 Coordinator partition transition events and flight recorder
P0 · M5 · Task · deps: GRP-0008, GRP-0022, OBS-F06, OBS-F07 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java transitionTo @4.4.0
Done when: LOADING, ACTIVE, FAILED and CLOSED transitions emit `coordinator_state_change` events with topic, partition, leader epoch and reason; each coordinator partition has a bounded flight recorder of transitions; events are listed in the crate's `events.rs`.

### [To Do] GRP-0024 Coordinator runtime metrics
P1 · M5 · Task · deps: GRP-0022, OBS-F04 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntimeMetricsImpl.java @4.4.0
Done when: Prometheus equivalents of event-queue-size, event-queue-time-ms, event-processing-time-ms, event-purgatory-time-ms, batch-flush-time-ms, batch-linger-time-ms, batch-flush-rate, thread-idle-ratio-avg, partition-load-time-avg/max, num-partitions by state and batch-buffer-cache size/discard count are exported with docs/metrics.md rows naming the Kafka JMX names.

### [To Do] GRP-0025 Simulation test of the runtime under replication faults
P0 · M5 · Task · deps: GRP-0022, TEST-F05, TEST-F06 · labels: test
Done when: a simulation with seeded faults proves responses never complete before the high watermark, failed appends revert state exactly, and leadership loss fails all pending events with NOT_COORDINATOR; the seed is printed on failure.

### [To Do] GRP-0026 Benchmark coordinator write throughput, latency and memory
P2 · M5 · Task · deps: GRP-0022, TEST-F11 · labels: perf
Done when: criterion benchmarks report ops/s and p99 latency for 1-record and 100-record writes at linger 0 and 5 ms, and peak snapshot memory per partition; the numbers and budgets are recorded in docs/nfr.md.

### [To Do] GRP-0027 Module documentation for the coordinator runtime
P1 · M5 · Task · deps: GRP-0022 · labels: docs
Done when: `//!` docs describe responsibility, layer, Kafka counterpart and invariants (single writer per coordinator partition, reads at the committed offset, responses at the high watermark); docs/parity.md lists the runtime as implemented.

### [To Do] GRP-0182 Input journaling hook for coordinator shards
P2 · M5 · Task · deps: SRV-F15
Done when: with `alluvium.journal.enable` set, the coordinator runtime journals inputs to each shard (metadata only, no offsets metadata payloads); the replay tool reproduces shard state from the journal in a test.

## [To Do] GRP-F02 `__consumer_offsets` topic
P0 · M5 · Feature · deps: GRP-F01
Scope: Auto-creation, `offsets.topic.*` configs, record key and value formats.

### [To Do] GRP-0028 Vendor the group coordinator record schemas and generate their types
P0 · M5 · Task · deps: PROTO-F05 · labels: decoder · ref: group-coordinator/src/main/resources/common/message/OffsetCommitKey.json @4.4.0
Done when: every JSON schema under group-coordinator/src/main/resources/common/message is copied unmodified into `schemas/kafka-4.4.0/` with the checksum manifest, and codegen produces key and value types for all versions; generated code passes the codegen drift check.

### [To Do] GRP-0029 Coordinator record key/value serde with version prefixes and tombstones
P0 · M5 · Task · deps: GRP-0028 · labels: decoder · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorRecordSerde.java @4.4.0
Done when: keys encode a 2-byte record type then the key; values encode a 2-byte version then the value; a null value decodes as a tombstone; an unknown record type yields a typed `UnknownRecordType` error that loading skips with a warning; an unsupported value version is an error; round-trip property tests for every record type.

### [To Do] GRP-0030 Fuzz target and golden bytes for coordinator record decoding
P0 · M5 · Task · deps: GRP-0029, TEST-F07 · labels: decoder, test
Done when: a cargo-fuzz target decodes arbitrary key/value bytes without panics or unbounded allocation; golden records written by Kafka 4.4.0 decode to the expected values and re-encode byte for byte.

### [To Do] GRP-0031 Offset commit and classic group metadata record construction
P0 · M5 · Task · deps: GRP-0029 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorRecordHelpers.java newOffsetCommitRecord @4.4.0
Done when: offset commit values use version 1 when an expire timestamp is present and version 4 (with tagged `topicId`) otherwise; classic group metadata values use the current version with member metadata, assignments and `currentStateTimestamp`; tombstone constructors exist for each key type; unit tests check exact bytes against Kafka golden files.

### [To Do] GRP-0032 Map a group id to its `__consumer_offsets` partition exactly like Kafka
P0 · M5 · Task · deps: GRP-0028 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java partitionFor @4.4.0
Done when: the partition is `abs(java_string_hash(group_id)) % offsets.topic.num.partitions` using Java `String.hashCode` over UTF-16 code units and Kafka's `Utils.abs` (`i32::MIN` maps to 0); golden tests cover ASCII, non-BMP and empty-adjacent ids against values computed by Kafka.

### [To Do] GRP-0033 `__consumer_offsets` topic configuration
P0 · M5 · Task · deps: CFG-F09 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java groupMetadataTopicConfigs @4.4.0
Done when: creation uses `offsets.topic.num.partitions` and `offsets.topic.replication.factor`, and topic configs `cleanup.policy=compact`, `compression.type=producer` and `segment.bytes=offsets.topic.segment.bytes`; a unit test asserts the exact config map.

### [To Do] GRP-0034 Auto-create `__consumer_offsets` on first coordinator lookup
P0 · M5 · Task · deps: GRP-0033, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/DefaultAutoTopicCreationManager.java @4.4.0
Done when: a lookup for a missing topic sends one de-duplicated CreateTopics request through the controller channel with the internal principal and returns COORDINATOR_NOT_AVAILABLE until a leader exists; a creation failure such as INVALID_REPLICATION_FACTOR is logged once with its reason and lookups keep returning COORDINATOR_NOT_AVAILABLE; tests cover in-flight de-duplication.

### [To Do] GRP-0035 Prove coordinator state survives log compaction
P0 · M5 · Task · deps: GRP-0031 · labels: test
Done when: a property test replays a random record history both raw and after compaction (with tombstones kept within `delete.retention.ms`) and asserts identical loaded group and offset state.

### [To Do] GRP-0036 Document the `__consumer_offsets` record formats
P1 · M5 · Task · deps: GRP-0031 · labels: docs
Done when: docs/formats/consumer-offsets.md lists every record type id, key and value version, field and tombstone rule, with a hex example per record type checked by a doc test.

### [To Do] GRP-0037 Expose the offsets record decoder for offline dump tooling
P1 · M5 · Task · deps: GRP-0029 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorRecordSerde.java @4.4.0
Done when: a library function renders decoded keys and values as text and JSON (no payload bytes beyond the record fields) for use by `alluvium dump log` (CLI-F04); unit test on golden records.

## [To Do] GRP-F03 FindCoordinator
P0 · M5 · Feature · deps: GRP-F01
Scope: Group, transaction and share key types; batched lookups (v4+).

### [To Do] GRP-0038 Resolve a group coordinator from the metadata image
P0 · M5 · Task · deps: GRP-0032, GRP-0034, META-F09 · ref: core/src/main/scala/kafka/server/KafkaApis.scala getCoordinator @4.4.0
Done when: the coordinator for a group key is the leader of its `__consumer_offsets` partition with the endpoint for the request's listener; a missing topic triggers auto-creation; a partition error, no leader or no endpoint on that listener returns COORDINATOR_NOT_AVAILABLE; unit tests for each case.

### [To Do] GRP-0039 Coordinator key type dispatch
P0 · M5 · Task · deps: GRP-0038 · ref: core/src/main/scala/kafka/server/KafkaApis.scala getCoordinator @4.4.0
Done when: `CoordinatorType` (GROUP=0, TRANSACTION=1, SHARE=2) is matched exhaustively, and FindCoordinator's `KeyType` reads as it through `xtask/codegen/known_values.toml` (PROTO-0062); an unknown key type returns INVALID_REQUEST; SHARE below FindCoordinator v6 returns INVALID_REQUEST; TRANSACTION and SHARE return COORDINATOR_NOT_AVAILABLE until TXN-F03 and SHARE-F05 add their arms; tests per key type.

### [To Do] GRP-0040 Batched coordinator lookups for many keys
P0 · M5 · Task · deps: GRP-0039 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFindCoordinatorRequestV4AndAbove @4.4.0
Done when: the lookup service resolves a list of keys in request order with a per-key error and node, so the v4+ handler (API-F10) needs no extra logic; an empty key list returns an empty result.

### [To Do] GRP-0041 Listener-aware coordinator endpoints with diagnostics
P1 · M5 · Task · deps: GRP-0038
Done when: the returned host and port come from the leader's endpoint for the request's listener name; a missing listener on the leader is reported once per leader epoch as a `coordinator_endpoint_missing` warn event with listener and broker id; tested with a two-listener cluster image.

### [To Do] GRP-0042 Test that coordinator lookups follow `__consumer_offsets` leadership
P0 · M5 · Task · deps: GRP-0040 · labels: test
Done when: an integration test moves partition leadership and asserts lookups return the new leader, while the old leader's coordinator returns NOT_COORDINATOR for writes.

### [To Do] GRP-0043 Document coordinator lookup semantics
P2 · M5 · Task · deps: GRP-0040 · labels: docs
Done when: module docs describe key types, partition mapping, auto-creation and the error table, linking docs/formats/consumer-offsets.md.

## [To Do] GRP-F04 Classic group protocol
P0 · M5 · Feature · deps: GRP-F01
Scope: Join/Sync/Heartbeat/Leave state machine, static membership, member id requirement, timeouts, initial rebalance delay.

### [To Do] GRP-0044 Classic group and member model
P0 · M5 · Task · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/classic/ClassicGroup.java @4.4.0
Done when: `ClassicGroup` holds state and previous state, generation id, protocol type and name, leader, members, pending members, static member map (instance id to member id), assignments and `currentStateTimestamp`; `ClassicGroupMember` holds session/rebalance timeouts, client id/host, protocols and assignment; invariants (leader is a member, static map consistent) are debug-asserted.

### [To Do] GRP-0045 Classic group state enum with valid previous states
P0 · M5 · Task · deps: GRP-0044 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/classic/ClassicGroupState.java @4.4.0
Done when: EMPTY, PREPARING_REBALANCE, COMPLETING_REBALANCE, STABLE and DEAD carry Kafka's names and valid previous states; `transition_to` panics on an invalid transition; each transition returns a transition record with reason for the driver.

### [To Do] GRP-0046 Transition Empty to PreparingRebalance on the first join
P0 · M5 · Task · deps: GRP-0045 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java prepareRebalance @4.4.0
Done when: the first member joining an Empty group moves it to PreparingRebalance, schedules the initial rebalance delay and records reason `member_joined`; unit test asserts state, timer key and event.

### [To Do] GRP-0047 Transition PreparingRebalance to CompletingRebalance when the join phase completes
P0 · M5 · Task · deps: GRP-0045, GRP-0058 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/classic/ClassicGroup.java initNextGeneration @4.4.0
Done when: on join completion with members, the generation increments, the protocol is selected, the leader is elected (current leader if rejoined, else the first joined member), all awaiting joins get responses (leader receives member metadata) and the group enters CompletingRebalance; pending-sync expiration is scheduled.

### [To Do] GRP-0048 Transition PreparingRebalance to Empty when no members remain
P0 · M5 · Task · deps: GRP-0045 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java completeClassicGroupJoin @4.4.0
Done when: join completion with no members increments the generation, moves to Empty and writes the empty group metadata record; an append failure is mapped with `appendGroupMetadataErrorToResponseError`; unit test covers both outcomes.

### [To Do] GRP-0049 Transition CompletingRebalance to Stable after the leader's assignment is persisted
P0 · M5 · Task · deps: GRP-0067 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java classicGroupSyncToClassicGroup @4.4.0
Done when: once the group metadata record with the leader's assignment is committed and the group is still in CompletingRebalance at the same generation, assignments are propagated to all awaiting syncs and the group becomes Stable; the completed-rebalance sensor is recorded.

### [To Do] GRP-0050 Transition CompletingRebalance to PreparingRebalance
P0 · M5 · Task · deps: GRP-0045 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java prepareRebalance @4.4.0
Done when: a member joining with changed metadata, a member leaving, a pending-sync expiry or a failed assignment append resets assignments, completes awaiting syncs with REBALANCE_IN_PROGRESS and moves to PreparingRebalance with the matching reason.

### [To Do] GRP-0051 Transition Stable to PreparingRebalance
P0 · M5 · Task · deps: GRP-0045 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java classicGroupJoinExistingMember @4.4.0
Done when: a leader rejoin, a follower rejoin with changed protocols, a new member join, a member leave or a session timeout moves a Stable group to PreparingRebalance with reason `leader_rejoined`, `metadata_changed`, `member_joined`, `member_left` or `session_timeout`; one test per trigger.

### [To Do] GRP-0052 Transition any classic group state to Dead
P0 · M5 · Task · deps: GRP-0045 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java createGroupTombstoneRecordsAndCancelTimers @4.4.0
Done when: deletion, expiry of an Empty group with no offsets, conversion to another group type and coordinator unload move the group to Dead, cancel its timers and complete awaiting joins/syncs (NOT_COORDINATOR on unload, REBALANCE_IN_PROGRESS on conversion); tested per trigger.

### [To Do] GRP-0053 JoinGroup request validation
P0 · M5 · Task · deps: GRP-0044 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java joinGroup @4.4.0
Done when: an inactive coordinator returns COORDINATOR_NOT_AVAILABLE, an empty group id returns INVALID_GROUP_ID, and a session timeout outside [`group.min.session.timeout.ms`, `group.max.session.timeout.ms`] returns INVALID_SESSION_TIMEOUT before any state change; tests per error.

### [To Do] GRP-0054 JoinGroup routing by group type
P0 · M5 · Task · deps: GRP-0044 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java classicGroupJoin @4.4.0
Done when: a join to a non-empty consumer group goes to the consumer-group path (GRP-F09); joins to classic, empty consumer and empty streams groups use the classic path (converting empty groups); joins to share groups and non-empty streams groups return INCONSISTENT_GROUP_PROTOCOL; a known member id for a missing group returns UNKNOWN_MEMBER_ID.

### [To Do] GRP-0055 Enforce `group.max.size` on join
P0 · M5 · Task · deps: GRP-0044 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java acceptJoiningMember @4.4.0
Done when: a new member beyond `group.max.size` is removed and gets GROUP_MAX_SIZE_REACHED with an empty member id; existing members, pending members and static replacements are still accepted while at capacity; tests at size limit and limit minus one.

### [To Do] GRP-0056 Require a known member id for dynamic members (KIP-394)
P0 · M5 · Task · deps: GRP-0044, GRP-0063 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java classicGroupJoinNewDynamicMember @4.4.0
Done when: on JoinGroup v4+ a dynamic member with an empty member id is added to pending members with a generated id `<client id>-<uuid>`, gets MEMBER_ID_REQUIRED with that id, and expires after its session timeout if it does not rejoin; on v0-v3 the member is added directly; rejoin with the pending id is accepted.

### [To Do] GRP-0057 Protocol type and protocol compatibility checks on join
P0 · M5 · Task · deps: GRP-0044 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/classic/ClassicGroup.java supportsProtocols @4.4.0
Done when: a join whose protocol type differs from the group's or with no protocol in common with all members returns INCONSISTENT_GROUP_PROTOCOL; the first member of an Empty group sets the protocol type; tests for each case.

### [To Do] GRP-0058 Protocol selection by member vote with deterministic tie-break
P0 · M5 · Task · deps: GRP-0044 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/classic/ClassicGroup.java selectProtocol @4.4.0
Done when: each member votes for its most preferred protocol among those supported by all members, the protocol with most votes wins, and ties break deterministically by the rule documented in module docs (Kafka uses hash-map order); tests include a tie.

### [To Do] GRP-0059 JoinGroup from an existing member in each state
P0 · M5 · Task · deps: GRP-0057 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java classicGroupJoinExistingMember @4.4.0
Done when: Dead returns COORDINATOR_NOT_AVAILABLE; `validateMember` returns UNKNOWN_MEMBER_ID for an unknown member or instance id and FENCED_INSTANCE_ID for an instance id mapped to another member id; CompletingRebalance with unchanged protocols returns the current generation (members list only to the leader); Stable follower with unchanged protocols returns the current generation; Empty returns UNKNOWN_MEMBER_ID.

### [To Do] GRP-0060 Static member join and replacement (KIP-345)
P0 · M5 · Task · deps: GRP-0059, GRP-0070 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java updateStaticMemberThenRebalanceOrCompleteJoin @4.4.0
Done when: a static member joining with an unknown member id replaces the existing mapping: the old member's awaiting join or sync completes with FENCED_INSTANCE_ID; in Stable the new mapping is persisted without a rebalance and the response sets `skipAssignment=true` for the leader (KIP-814); a failed append reverts the replacement and returns the mapped error; in other states the member is updated and the rebalance continues.

### [To Do] GRP-0061 Initial rebalance delay for groups leaving Empty
P0 · M5 · Task · deps: GRP-0046, GRP-0063 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java tryCompleteInitialRebalanceElseSchedule @4.4.0
Done when: the first rebalance waits `group.initial.rebalance.delay.ms` and extends by that delay while new members keep joining, bounded by the remaining rebalance timeout; once no member joined in the last delay the join completes; tests with the simulated clock.

### [To Do] GRP-0062 Rebalance timeout and removal of members that did not rejoin
P0 · M5 · Task · deps: GRP-0047, GRP-0063 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java completeClassicGroupJoin @4.4.0
Done when: the join phase completes when all known members rejoined or after the maximum rebalance timeout among members; dynamic members that did not rejoin are removed; static members are kept; if the elected leader has not rejoined the join waits for session timeouts; tests for each case.

### [To Do] GRP-0063 Classic heartbeat, join and sync timer keys
P0 · M5 · Task · deps: GRP-0044, GRP-0017 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java rescheduleClassicGroupMemberHeartbeat @4.4.0
Done when: per-member heartbeat expiration, per-group join completion and per-group pending-sync expiration use keyed coordinator timers with Kafka's key formats; heartbeats, commits in Stable or PreparingRebalance and joins reschedule the member's session expiry.

### [To Do] GRP-0064 Session timeout expiry of classic members
P0 · M5 · Task · deps: GRP-0063 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java expireClassicGroupMemberHeartbeat @4.4.0
Done when: a member whose session timeout elapses is removed; a pending member is dropped without a rebalance; removing a member of a Stable or CompletingRebalance group triggers a rebalance and of a PreparingRebalance group may complete the join; tests per state.

### [To Do] GRP-0065 Pending sync expiry
P0 · M5 · Task · deps: GRP-0047, GRP-0063 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java expirePendingSync @4.4.0
Done when: members that do not send SyncGroup within the rebalance timeout after CompletingRebalance (or Stable for followers) are removed and the group rebalances with reason `sync_timeout`; members that sync in time are not affected.

### [To Do] GRP-0066 SyncGroup validation
P0 · M5 · Task · deps: GRP-0044 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java validateSyncGroup @4.4.0
Done when: an inactive coordinator returns COORDINATOR_NOT_AVAILABLE, an empty group id INVALID_GROUP_ID, a missing group UNKNOWN_MEMBER_ID, Dead COORDINATOR_NOT_AVAILABLE, member validation UNKNOWN_MEMBER_ID or FENCED_INSTANCE_ID, a generation mismatch ILLEGAL_GENERATION, a protocol type or name mismatch (v5+) INCONSISTENT_GROUP_PROTOCOL, and PreparingRebalance REBALANCE_IN_PROGRESS; one test per error.

### [To Do] GRP-0067 SyncGroup leader assignment and follower waits
P0 · M5 · Task · deps: GRP-0066, GRP-0070 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java classicGroupSyncToClassicGroup @4.4.0
Done when: in CompletingRebalance followers wait for the assignment; the leader's assignment is filled with empty assignments for missing members (logged at warn) and persisted in one group metadata record; append failure resets assignments with the mapped error and rebalances; in Stable a member gets its current assignment with protocol type and name.

### [To Do] GRP-0068 Classic Heartbeat handling
P0 · M5 · Task · deps: GRP-0063 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java classicGroupHeartbeatToClassicGroup @4.4.0
Done when: a missing group returns UNKNOWN_MEMBER_ID, Dead COORDINATOR_NOT_AVAILABLE, member validation UNKNOWN_MEMBER_ID or FENCED_INSTANCE_ID, a generation mismatch ILLEGAL_GENERATION, PreparingRebalance REBALANCE_IN_PROGRESS (session still extended), CompletingRebalance and Stable NONE, Empty UNKNOWN_MEMBER_ID; COORDINATOR_LOAD_IN_PROGRESS is answered as NONE like Kafka's service.

### [To Do] GRP-0069 LeaveGroup with batched members (v3+)
P0 · M5 · Task · deps: GRP-0063 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java classicGroupLeaveToClassicGroup @4.4.0
Done when: each member identity (member id and/or instance id) gets its own error: UNKNOWN_MEMBER_ID or FENCED_INSTANCE_ID; pending members are dropped; a missing group returns UNKNOWN_MEMBER_ID at top level; if any member was removed the group rebalances once with the client reason (v5) in the event; v0-v2 single-member semantics preserved.

### [To Do] GRP-0070 Persist and replay classic group metadata
P0 · M5 · Task · deps: GRP-0044, GRP-0031 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorRecordHelpers.java newGroupMetadataRecord @4.4.0
Done when: group metadata records are written on generation completion, on assignment and on static replacement; replay rebuilds members, generation, protocol and assignments, and a tombstone removes the group; replay of a group record for a group of another type is a typed error; round-trip tests.

### [To Do] GRP-0071 Property test of classic group invariants under random request sequences
P0 · M5 · Task · deps: GRP-0067, GRP-0068, GRP-0069, GRP-0060 · labels: test
Done when: random join/sync/heartbeat/leave/timeout sequences keep invariants: the leader is a member, generation never decreases, only valid transitions occur, and every member in Stable has exactly the assignment from its generation's leader.

### [To Do] GRP-0072 Port Kafka's classic protocol tests that encode tricky semantics
P1 · M5 · Task · deps: GRP-0071, TEST-F10 · labels: test · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java @4.4.0
Done when: the classic-group cases from `GroupMetadataManagerTest` listed in the TEST-F10 inventory are ported with `// Ported from:` attributions and pass.

## [To Do] GRP-F05 Consumer group protocol
P0 · M5 · Feature · deps: GRP-F01, GRP-F06
Scope: KIP-848 heartbeat, member epochs, target assignment, reconciliation, regex subscriptions, rack awareness.

### [To Do] GRP-0073 Consumer group and member model on timeline structures
P0 · M5 · Task · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/consumer/ConsumerGroup.java @4.4.0
Done when: the group holds group epoch, target assignment epoch, metadata hash, members, target assignments, partition epochs, subscribed topic name counts, regex subscriptions and resolved regexes, server assignor counts; members hold epoch, previous epoch, state, assigned partitions with assignment epochs, partitions pending revocation, instance id, rack id, rebalance timeout, client id/host, subscriptions and classic metadata; all reads support the committed epoch.

### [To Do] GRP-0074 Consumer group record replay
P0 · M5 · Task · deps: GRP-0073, GRP-0029 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java replay @4.4.0
Done when: ConsumerGroupMetadata, MemberMetadata, TargetAssignmentMetadata, TargetAssignmentMember, CurrentMemberAssignment and RegularExpression records and their tombstones replay into the model; the deprecated ConsumerGroupPartitionMetadata record is accepted and tombstoned (KIP-1101); replay tests for each record and tombstone.

### [To Do] GRP-0075 ConsumerGroupHeartbeat request validation
P0 · M5 · Task · deps: GRP-0073 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java throwIfConsumerGroupHeartbeatRequestIsInvalid @4.4.0
Done when: INVALID_REQUEST is returned for an empty member id (v1+, member epoch > 0, or leave), empty group id, empty instance or rack id, epoch 0 without rebalance timeout, non-empty owned partitions on join, no subscribed names or regex on join, epoch -2 without instance id and epochs below -2; an unknown server assignor returns UNSUPPORTED_ASSIGNOR; one test per rule.

### [To Do] GRP-0076 Gate the consumer protocol on `group.version` and `group.coordinator.rebalance.protocols`
P0 · M5 · Task · deps: GRP-0075, META-F06 · ref: server-common/src/main/java/org/apache/kafka/server/common/GroupVersion.java @4.4.0
Done when: ConsumerGroupHeartbeat returns UNSUPPORTED_VERSION when `group.version` < 1 or `consumer` is not in `group.coordinator.rebalance.protocols`; the gate updates when the finalized feature level changes; tests for both conditions.

### [To Do] GRP-0077 Member join with epoch 0 and group creation
P0 · M5 · Task · deps: GRP-0074 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java getOrMaybeCreateConsumerGroup @4.4.0
Done when: epoch 0 creates the group if absent; a non-zero epoch for a missing group returns GROUP_ID_NOT_FOUND; a group of another type returns GROUP_ID_NOT_FOUND; v0 requests with an empty member id get a server-generated UUID; the member is created with its subscription and the group epoch is bumped.

### [To Do] GRP-0078 Enforce `group.consumer.max.size`
P0 · M5 · Task · deps: GRP-0077 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java throwIfConsumerGroupIsFull @4.4.0
Done when: a new member beyond `group.consumer.max.size` gets GROUP_MAX_SIZE_REACHED; existing members and a static member replacing its previous incarnation are not counted as new; tests at the limit.

### [To Do] GRP-0079 Member epoch fencing
P0 · M5 · Task · deps: GRP-0077 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java throwIfConsumerGroupMemberEpochIsInvalid @4.4.0
Done when: an unknown member id with a non-zero epoch returns UNKNOWN_MEMBER_ID; an epoch greater than the member's returns FENCED_MEMBER_EPOCH; a smaller epoch is accepted only if it equals the previous epoch and the owned partitions are a subset of the assigned partitions, otherwise FENCED_MEMBER_EPOCH; tests per case.

### [To Do] GRP-0080 Static members in consumer groups
P0 · M5 · Task · deps: GRP-0079 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java getOrMaybeSubscribeStaticConsumerGroupMember @4.4.0
Done when: a static member rejoining with epoch 0 replaces the previous member that left with epoch -2, inheriting its assignment and target; the replacement while the previous member is active returns UNRELEASED_INSTANCE_ID; a member id mismatch for the instance returns FENCED_INSTANCE_ID; an unknown instance id with non-zero epoch returns UNKNOWN_MEMBER_ID; tests per case.

### [To Do] GRP-0081 Subscription changes and group epoch bumps
P0 · M5 · Task · deps: GRP-0077, GRP-0082 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java updateSubscriptionMetadata @4.4.0
Done when: a change of subscribed topic names, of the preferred server assignor, or of the metadata hash of subscribed topics bumps the group epoch and persists ConsumerGroupMetadata; subscription type (homogeneous/heterogeneous) is recomputed; unchanged subscriptions do not bump; tests per trigger.

### [To Do] GRP-0082 Topic and group metadata hashes (KIP-1101)
P0 · M5 · Task · deps: GRP-0073 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/Utils.java computeTopicHash @4.4.0
Done when: the topic hash is XXH3-64 over the magic byte, topic id, name, partition count and per-partition sorted racks, the group hash combines topic hashes sorted by name, a missing topic hashes to 0, results equal Kafka's on golden inputs, and hashes are cached per topic and invalidated on metadata deltas; the hashing dependency is justified per the dependency policy.

### [To Do] GRP-0083 Regex subscription validation and bookkeeping (v1+)
P0 · M5 · Task · deps: GRP-0081 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java maybeUpdateRegularExpressions @4.4.0
Done when: a new regex is compiled with an RE2-compatible engine and an invalid one returns INVALID_REGULAR_EXPRESSION; subscriber counts per regex are maintained; the last subscriber leaving tombstones the resolved regex record; a change to an already resolved regex bumps the group epoch immediately.

### [To Do] GRP-0084 Asynchronous regex resolution with topic DESCRIBE authorization
P0 · M5 · Task · deps: GRP-0083, GRP-0018, SEC-F08 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/consumer/TopicRegexResolver.java @4.4.0
Done when: resolution runs on the coordinator executor at most once at a time per group, at most every 10 s and at least every `group.consumer.regex.refresh.interval.ms`; matched topics are filtered to those the member's principal may DESCRIBE; results are persisted as ConsumerGroupRegularExpression records with version and timestamp and bump the group epoch when the set changes; tests with an authorizer that denies some topics.

### [To Do] GRP-0085 Document RE2J versus Rust regex dialect differences for subscriptions
P1 · M5 · Spike · deps: GRP-0083
Done when: ADR `regex-subscription-dialect` accepted in docs/adr/ listing syntax accepted by RE2J but not the Rust engine (and vice versa), the chosen handling for each, and a differential test corpus of patterns compared against Kafka.

### [To Do] GRP-0086 Target assignment computation and persistence
P0 · M5 · Task · deps: GRP-0081, GRP-0109 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/TargetAssignmentBuilder.java @4.4.0
Done when: when the group epoch exceeds the target epoch the target is recomputed with the group's assignor, only changed member targets are written followed by TargetAssignmentMetadata with the new epoch, a static member replacement keeps its target, and assignor errors fail the heartbeat with UNKNOWN_SERVER_ERROR logged once.

### [To Do] GRP-0087 Assignment interval for consumer groups
P0 · M5 · Task · deps: GRP-0086 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java computeDelayedTargetAssignment @4.4.0
Done when: target recomputation runs at most once per `consumer.assignment.interval.ms` (group config, bounded by `group.consumer.min/max.assignment.interval.ms`, default `group.consumer.assignment.interval.ms`) with a delayed computation timer; a heartbeat within the interval reconciles against the existing target; tests with the simulated clock.

### [To Do] GRP-0088 Offload assignor runs to the coordinator executor
P0 · M5 · Task · deps: GRP-0086, GRP-0018 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java maybeUpdateTargetAssignment @4.4.0
Done when: with `consumer.assignor.offload.enable` (default from `group.consumer.assignor.offload.enable`) the assignor runs on the executor and the result is applied only if the group epoch it was computed for is still current; stale results are discarded with a debug event; tests cover a racing epoch bump.

### [To Do] GRP-0089 Reconciliation from STABLE: compute revocations against the new target
P0 · M5 · Task · deps: GRP-0086 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/consumer/CurrentAssignmentBuilder.java @4.4.0
Done when: a STABLE member behind the target moves to UNREVOKED_PARTITIONS with partitions pending revocation if it owns partitions not in its target, otherwise advances to the target epoch (STABLE or UNRELEASED_PARTITIONS); the CurrentMemberAssignment record is written; unit tests per path.

### [To Do] GRP-0090 Reconciliation from UNREVOKED_PARTITIONS on revocation acknowledgement
P0 · M5 · Task · deps: GRP-0089 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/consumer/CurrentAssignmentBuilder.java @4.4.0
Done when: when owned partitions in a heartbeat no longer include the pending revocations the member advances to the target epoch and the released partitions' epochs are cleared; a missing acknowledgement keeps the member in place; tests per case.

### [To Do] GRP-0091 Reconciliation from UNRELEASED_PARTITIONS as other members release partitions
P0 · M5 · Task · deps: GRP-0089 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/consumer/CurrentAssignmentBuilder.java @4.4.0
Done when: a member waiting for partitions still owned by others receives each partition once its previous owner releases it and becomes STABLE when its full target is assigned; tests with two members swapping partitions.

### [To Do] GRP-0092 Reconciliation edge cases: UNKNOWN state and forced consistency on subscription change
P0 · M5 · Task · deps: GRP-0089 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/consumer/CurrentAssignmentBuilder.java @4.4.0
Done when: a member in UNKNOWN state gets FENCED_MEMBER_EPOCH; when the subscription changed, assigned partitions outside the new subscription are revoked even without a new target; tests per case.

### [To Do] GRP-0093 Revocation (rebalance) timeout fences slow members
P0 · M5 · Task · deps: GRP-0090, GRP-0017 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java scheduleConsumerGroupRebalanceTimeout @4.4.0
Done when: a member in UNREVOKED_PARTITIONS that does not acknowledge within its rebalance timeout is fenced (removed, group epoch bumped, records tombstoned) and its next heartbeat gets FENCED_MEMBER_EPOCH (or UNKNOWN_MEMBER_ID); the timer is cancelled on acknowledgement.

### [To Do] GRP-0094 Partition ownership epochs invariant
P0 · M5 · Task · deps: GRP-0089 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/ModernGroup.java @4.4.0
Done when: each partition maps to at most one owning member epoch; adding an owned partition that is already owned panics as an invariant violation; releasing clears the epoch; a property test over random reconciliations proves no partition is owned twice.

### [To Do] GRP-0095 ConsumerGroupHeartbeat response content
P0 · M5 · Task · deps: GRP-0089 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java consumerGroupHeartbeat @4.4.0
Done when: the response carries member id, member epoch and heartbeat interval (`consumer.heartbeat.interval.ms` group config or `group.consumer.heartbeat.interval.ms`); the assignment is included on join, on a full request (rebalance timeout, subscription and owned partitions all set) or when assigned partitions changed; tests per case.

### [To Do] GRP-0096 Consumer member session timeout
P0 · M5 · Task · deps: GRP-0077, GRP-0017 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java scheduleConsumerGroupSessionTimeout @4.4.0
Done when: each heartbeat reschedules the member's session timer from `consumer.session.timeout.ms` (group config, else `group.consumer.session.timeout.ms`); expiry fences the member, tombstones its member, target and current-assignment records and bumps the group epoch; static members that left with epoch -2 expire the same way.

### [To Do] GRP-0097 Consumer member leave with epoch -1 and static leave with epoch -2
P0 · M5 · Task · deps: GRP-0077 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java consumerGroupLeave @4.4.0
Done when: epoch -1 removes the member, tombstones its records and bumps the group epoch; epoch -2 for a static member keeps its assignment for a replacement until the session timeout; responses echo the leave epoch; tests per path.

### [To Do] GRP-0098 Rack information for server-side assignment
P1 · M5 · Task · deps: GRP-0082 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/SubscribedTopicDescriberImpl.java @4.4.0
Done when: member rack ids and partition replica racks are passed to assignors through the group spec and topic describer; a partition rack change alters the topic hash and bumps the group epoch; module docs record that the 4.4.0 uniform and range assignors do not place by rack.

### [To Do] GRP-0099 Property test of consumer group invariants under random heartbeat sequences
P0 · M5 · Task · deps: GRP-0090, GRP-0091, GRP-0096, GRP-0097, GRP-0080 · labels: test
Done when: random heartbeats, leaves, timeouts and subscription changes keep: member epoch <= target epoch <= group epoch, no partition owned twice, state derivation consistent, and every committed state reloads identically from its records.

### [To Do] GRP-0100 Port Kafka's consumer protocol tests that encode tricky semantics
P1 · M5 · Task · deps: GRP-0099, TEST-F10 · labels: test · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java @4.4.0
Done when: consumer-group cases from `GroupMetadataManagerTest` and `CurrentAssignmentBuilderTest` listed in the TEST-F10 inventory are ported with attributions and pass.

### [To Do] GRP-0101 Consumer group state: Empty to Assigning
P0 · M5 · Task · deps: GRP-0073 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/consumer/ConsumerGroup.java maybeUpdateGroupState @4.4.0
Done when: the group state is derived after every change; the first member join (group epoch > target epoch) moves Empty to Assigning; a `group_state_change` transition with reason `member_joined` is returned; unit test.

### [To Do] GRP-0102 Consumer group state: Assigning to Reconciling
P0 · M5 · Task · deps: GRP-0101, GRP-0086 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/consumer/ConsumerGroup.java maybeUpdateGroupState @4.4.0
Done when: when the target epoch catches up with the group epoch while some member is not at the target epoch or not STABLE, the group becomes Reconciling with reason `target_assignment_updated`; unit test.

### [To Do] GRP-0103 Consumer group state: Reconciling to Stable and Stable to Assigning
P0 · M5 · Task · deps: GRP-0102, GRP-0089 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/consumer/ConsumerGroup.java maybeUpdateGroupState @4.4.0
Done when: the group becomes Stable when all members are STABLE at the target epoch, and returns to Assigning on any group epoch bump; transitions carry reasons `members_reconciled` and the bump reason; unit tests.

### [To Do] GRP-0104 Consumer group state: to Empty and Empty to Dead
P0 · M5 · Task · deps: GRP-0101, GRP-0097 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/consumer/ConsumerGroup.java @4.4.0
Done when: the last member leaving or being fenced makes the group Empty; deletion, offset expiry of an Empty group and conversion make it Dead and remove it; unit tests per trigger.

## [To Do] GRP-F06 Server-side assignors
P0 · M5 · Feature · deps: GRP-F01
Scope: Uniform and range assignors, homogeneous and heterogeneous subscriptions, stickiness.

### [To Do] GRP-0105 Server-side assignor interface and inputs
P0 · M5 · Task · deps: none · ref: group-coordinator/group-coordinator-api/src/main/java/org/apache/kafka/coordinator/group/api/assignor/ConsumerGroupPartitionAssignor.java @4.4.0
Done when: the assignor trait takes a group spec (members with subscribed topic ids, rack id and current assignment; subscription type) and a subscribed-topic describer (partition count and racks) and returns a group assignment; failures are a typed `PartitionAssignorError`; at least the uniform and range implementations exist, justifying the trait.

### [To Do] GRP-0106 Uniform assignor for homogeneous subscriptions
P0 · M5 · Task · deps: GRP-0105 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/assignor/UniformHomogeneousAssignmentBuilder.java @4.4.0
Done when: partitions are spread so member counts differ by at most one while keeping as many current assignments as possible; results equal Kafka's for the ported `UniformHomogeneousAssignmentBuilderTest` cases.

### [To Do] GRP-0107 Uniform assignor for heterogeneous subscriptions
P0 · M5 · Task · deps: GRP-0105 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/assignor/UniformHeterogeneousAssignmentBuilder.java @4.4.0
Done when: each topic's partitions go only to its subscribers, balance is maximised as in Kafka's heterogeneous builder, stickiness is preserved, and ported `UniformHeterogeneousAssignmentBuilderTest` cases pass.

### [To Do] GRP-0108 Range assignor for homogeneous and heterogeneous subscriptions
P0 · M5 · Task · deps: GRP-0105 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/assignor/RangeAssignor.java @4.4.0
Done when: co-partitioned ranges are assigned per topic in member order with Kafka's sticky range behavior, for both subscription types, and ported `RangeAssignorTest` cases pass.

### [To Do] GRP-0109 Assignor selection for a consumer group
P0 · M5 · Task · deps: GRP-0105, CFG-F09 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/consumer/ConsumerGroup.java computePreferredServerAssignor @4.4.0
Done when: `group.consumer.assignors` defines available assignors (default `uniform,range`, first is default); the group uses the server assignor most preferred by its members, else the default, with a deterministic tie-break documented; a change of the selected assignor bumps the group epoch.

### [To Do] GRP-0110 Deterministic assignor output and golden comparison with Kafka
P0 · M5 · Task · deps: GRP-0106, GRP-0107, GRP-0108 · labels: test
Done when: assignors iterate members and topics in sorted order so the same input always yields the same output; golden outputs produced by Kafka 4.4.0's assignors for 50 fixed inputs match exactly.

### [To Do] GRP-0111 Property tests for assignor correctness, balance and stickiness
P0 · M5 · Task · deps: GRP-0106, GRP-0107, GRP-0108 · labels: test
Done when: for random groups every subscribed partition is assigned exactly once to a subscribed member, uniform balance bounds hold, and an unchanged group keeps its assignment; failing seeds are persisted.

### [To Do] GRP-0112 Benchmark server-side assignors at scale
P2 · M5 · Task · deps: GRP-0106, GRP-0107, GRP-0108, TEST-F11 · labels: perf
Done when: criterion benchmarks for 1 000 members × 10 000 partitions (homogeneous and heterogeneous) record latency and allocations, with budgets in docs/nfr.md and a CI regression threshold.

### [To Do] GRP-0113 Document server-side assignors
P2 · M5 · Task · deps: GRP-0109 · labels: docs
Done when: module docs describe each assignor's algorithm, stickiness, determinism rules and the selection order, with links to KIP-848.

## [To Do] GRP-F07 Offset commit and fetch
P0 · M5 · Feature · deps: GRP-F01
Scope: OffsetCommit/OffsetFetch semantics per version, require-stable, OffsetDelete, offset retention.

### [To Do] GRP-0114 Committed offsets store on timeline structures
P0 · M5 · Task · deps: GRP-0031 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java @4.4.0
Done when: offsets are stored per group, topic and partition with committed offset, leader epoch, metadata, commit timestamp, optional expire timestamp, record offset and topic id; replay of OffsetCommit records and tombstones updates it; offset count is tracked per shard; replay tests.

### [To Do] GRP-0115 OffsetCommit group resolution
P0 · M5 · Task · deps: GRP-0114, GRP-0044, GRP-0073 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java validateOffsetCommit @4.4.0
Done when: a missing group with generation/epoch < 0 creates a simple classic group; a missing group with >= 0 returns GROUP_ID_NOT_FOUND on v9+ and ILLEGAL_GENERATION below v9; a share group returns GROUP_ID_NOT_FOUND; tests per version boundary.

### [To Do] GRP-0116 OffsetCommit validation for classic groups
P0 · M5 · Task · deps: GRP-0115 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/classic/ClassicGroup.java validateOffsetCommit @4.4.0
Done when: Dead returns COORDINATOR_NOT_AVAILABLE; generation < 0 on an Empty group is accepted; member identity errors return UNKNOWN_MEMBER_ID or FENCED_INSTANCE_ID; a generation mismatch returns ILLEGAL_GENERATION; no member in a non-Empty group returns UNKNOWN_MEMBER_ID; CompletingRebalance returns REBALANCE_IN_PROGRESS; commits in Stable or PreparingRebalance extend the member session.

### [To Do] GRP-0117 OffsetCommit validation for consumer groups with per-partition assignment epochs
P0 · M5 · Task · deps: GRP-0115 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/consumer/ConsumerGroup.java validateOffsetCommit @4.4.0
Done when: epoch < 0 with no members is accepted; an unknown member returns UNKNOWN_MEMBER_ID; a consumer-protocol member below v9 gets UNSUPPORTED_VERSION; an epoch above the member's returns STALE_MEMBER_EPOCH (ILLEGAL_GENERATION for classic-protocol members); an older epoch is accepted per partition only if the partition is assigned or pending revocation with assignment epoch <= the received epoch, else STALE_MEMBER_EPOCH/ILLEGAL_GENERATION.

### [To Do] GRP-0118 OffsetCommit per-partition validation and record writes
P0 · M5 · Task · deps: GRP-0116, GRP-0117 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java commitOffset @4.4.0
Done when: metadata longer than `offset.metadata.max.bytes` returns OFFSET_METADATA_TOO_LARGE for that partition only; v2-v4 `retention_time_ms` sets an expire timestamp (value v1), other versions write value v4 with topic id; committed leader epoch is stored (v6+); the response lists every partition; tests per version.

### [To Do] GRP-0119 Topic-id-based offset commits (v10)
P0 · M5 · Task · deps: GRP-0118 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java commitOffset @4.4.0
Done when: v10 commits resolve topic ids against the metadata image and an unknown topic id returns UNKNOWN_TOPIC_ID for its partitions; commits store both the name and the id; tests cover a recreated topic with a new id.

### [To Do] GRP-0120 OffsetFetch group and member validation
P0 · M5 · Task · deps: GRP-0114, GRP-0044, GRP-0073 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java validateOffsetFetch @4.4.0
Done when: a missing group returns GROUP_ID_NOT_FOUND only where Kafka does (otherwise empty results), a classic Dead group GROUP_ID_NOT_FOUND, a share group GROUP_ID_NOT_FOUND; for consumer groups (v9+) an unknown member returns UNKNOWN_MEMBER_ID and an epoch mismatch STALE_MEMBER_EPOCH (ILLEGAL_GENERATION for classic members); null member id with epoch < 0 is accepted.

### [To Do] GRP-0121 OffsetFetch for listed partitions and for all partitions
P0 · M5 · Task · deps: GRP-0120 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java fetchOffsets @4.4.0
Done when: listed partitions without an offset return offset -1, leader epoch -1 and empty metadata; a null topic list returns all committed offsets; v8+ fetches several groups with per-group errors; v10 resolves topic ids and treats a mismatched topic id as no offset; reads use the last committed offset.

### [To Do] GRP-0122 OffsetFetch error remapping for older consumers
P0 · M5 · Task · deps: GRP-0121 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java handleOffsetFetchException @4.4.0
Done when: UNKNOWN_TOPIC_OR_PARTITION, NOT_ENOUGH_REPLICAS and REQUEST_TIMED_OUT from the runtime return NOT_COORDINATOR for OffsetFetch (not COORDINATOR_NOT_AVAILABLE); other errors use the common mapping; tests per error.

### [To Do] GRP-0123 OffsetFetch `require_stable` read point (v7+)
P0 · M5 · Task · deps: GRP-0121 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java fetchOffsets @4.4.0
Done when: with `require_stable` the fetch reads the latest written state instead of the last committed state; the UNSTABLE_OFFSET_COMMIT path for pending transactional offsets is delivered by TXN-F07; module docs explain both.

### [To Do] GRP-0124 OffsetDelete semantics
P0 · M5 · Task · deps: GRP-0114, GRP-0044, GRP-0073 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java deleteOffsets @4.4.0
Done when: a missing group or a classic Dead group returns GROUP_ID_NOT_FOUND; a non-empty classic group not using the consumer protocol returns NON_EMPTY_GROUP; partitions of topics the group is subscribed to return GROUP_SUBSCRIBED_TO_TOPIC; other partitions get tombstones; share groups return GROUP_ID_NOT_FOUND; tests per case.

### [To Do] GRP-0125 Offset retention and expiry
P0 · M5 · Task · deps: GRP-0114, GRP-0017 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java cleanupExpiredOffsets @4.4.0
Done when: every `offsets.retention.check.interval.ms` offsets are expired with `offsets.retention.minutes`: an explicit expire timestamp wins; classic Empty groups use the state timestamp (or commit timestamp); classic groups using the consumer protocol in Stable and consumer groups expire only unsubscribed topics by commit timestamp; tombstones are written and an Empty group with no offsets left becomes Dead.

### [To Do] GRP-0126 Remove offsets of deleted topics
P0 · M5 · Task · deps: GRP-0114, GRP-0020 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java onTopicsDeleted @4.4.0
Done when: a metadata delta with deleted topics writes tombstones for those topics' offsets in every group of the shard; test with two groups and a recreated topic.

### [To Do] GRP-0127 Delete all offsets of a group for group deletion
P0 · M5 · Task · deps: GRP-0114 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java deleteAllOffsets @4.4.0
Done when: the operation tombstones every offset of the group and returns the count; pending transactional offsets are handled by TXN-F07; unit test.

### [To Do] GRP-0128 Property test of the offset store against a model
P0 · M5 · Task · deps: GRP-0118, GRP-0121, GRP-0124, GRP-0125 · labels: test
Done when: random commits, deletes, expiries, reverts and fetches at committed and latest read points match a simple model; persisted regressions committed.

## [To Do] GRP-F08 Group administration
P0 · M5 · Feature · deps: GRP-F04, GRP-F05
Scope: DescribeGroups, ListGroups filters, DeleteGroups, ConsumerGroupDescribe.

### [To Do] GRP-0129 ListGroups with state and type filters
P0 · M5 · Task · deps: GRP-0044, GRP-0073, GRP-0014 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java listGroups @4.4.0
Done when: groups from all shards are listed with group id, protocol type, state and type; v4+ states filter is case-insensitive; v5+ types filter accepts classic, consumer, share and streams; a failing shard returns its error code at top level as Kafka does; tests per filter.

### [To Do] GRP-0130 DescribeGroups for classic and consumer groups
P0 · M5 · Task · deps: GRP-0044, GRP-0073 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java describeGroups @4.4.0
Done when: classic groups return state, protocol type and name and members (metadata and assignment only where Kafka includes them); consumer groups return their classic-compatible view; a missing group returns Dead state with GROUP_ID_NOT_FOUND on v6+ and NONE below; groups of other types return GROUP_ID_NOT_FOUND.

### [To Do] GRP-0131 ConsumerGroupDescribe
P0 · M5 · Task · deps: GRP-0073 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java consumerGroupDescribe @4.4.0
Done when: each group returns epochs, state, assignor and members with instance id, rack, client id/host, subscriptions, regex, assignment and target assignment with topic names resolved; v1 adds member type (classic or consumer, KIP-1099); a missing or non-consumer group returns GROUP_ID_NOT_FOUND.

### [To Do] GRP-0132 DeleteGroups
P0 · M5 · Task · deps: GRP-0127, GRP-0052, GRP-0104 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorShard.java deleteGroups @4.4.0
Done when: per group: missing returns GROUP_ID_NOT_FOUND, non-empty returns NON_EMPTY_GROUP, otherwise offsets and group records are tombstoned atomically, timers cancelled and the group becomes Dead; results are per group; share and streams specifics are added by their epics.

### [To Do] GRP-0133 GROUP config resource definitions and validation
P0 · M7 · Task · deps: CFG-F04, CTRL-F11 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfigManager.java @4.4.0
Done when: GROUP config keys are defined with types and defaults from the broker `group.*` settings, values outside broker min/max bounds (for example session and heartbeat intervals) return INVALID_CONFIG at alter time, applied values reach the coordinators through the metadata publisher, and DescribeConfigs shows them with sources; tests per consumer key.

### [To Do] GRP-0134 Apply dynamic consumer group configs
P0 · M7 · Task · deps: GRP-0096, GRP-0133 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java @4.4.0
Done when: `consumer.session.timeout.ms`, `consumer.heartbeat.interval.ms`, `consumer.assignment.interval.ms` and `consumer.assignor.offload.enable` group configs override broker defaults for new timers and responses; changes take effect on the next heartbeat; tests per key.

### [To Do] GRP-0135 Document group administration semantics
P2 · M5 · Task · deps: GRP-0129, GRP-0130, GRP-0131, GRP-0132 · labels: docs
Done when: module docs list each admin operation's errors and visibility rules (committed reads), linked from docs/parity.md.

## [To Do] GRP-F09 Protocol migration
P1 · M5 · Feature · deps: GRP-F04, GRP-F05
Scope: Online classic↔consumer group conversion per migration policy.

### [To Do] GRP-0136 Migration policy parsing and checks
P1 · M5 · Task · deps: CFG-F09 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/ConsumerGroupMigrationPolicy.java @4.4.0
Done when: `group.consumer.migration.policy` accepts `bidirectional`, `upgrade`, `downgrade` and `disabled` case-insensitively with `is_upgrade_enabled`/`is_downgrade_enabled`; invalid values fail config validation; unit tests.

### [To Do] GRP-0137 Validate an online classic-to-consumer upgrade
P1 · M5 · Task · deps: GRP-0136 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java validateOnlineUpgrade @4.4.0
Done when: upgrade is refused (GROUP_ID_NOT_FOUND with a reason) when the policy disallows it, the classic group exceeds `group.consumer.max.size`, or its protocol type is not `consumer`; a malformed embedded subscription returns INCONSISTENT_GROUP_PROTOCOL; tests per rule.

### [To Do] GRP-0138 Convert a non-empty classic group to a consumer group
P1 · M5 · Task · deps: GRP-0137 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java convertToConsumerGroup @4.4.0
Done when: a consumer-protocol join to a classic group completes pending joins and syncs with REBALANCE_IN_PROGRESS, builds members from classic metadata (subscription and assignment decoded with bounded parsing), writes classic tombstones plus consumer group records atomically, and reverts on append failure; tests for success and failure.

### [To Do] GRP-0139 JoinGroup from classic-protocol members in a consumer group
P1 · M5 · Task · deps: GRP-0138 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java classicGroupJoinToConsumerGroup @4.4.0
Done when: classic members join with subscriptions decoded from their protocol metadata, unsupported protocols return INCONSISTENT_GROUP_PROTOCOL, join and sync timeouts use per-member timers, the response's generation equals the member epoch and its assignment derives from the target; tests per path.

### [To Do] GRP-0140 SyncGroup and Heartbeat from classic-protocol members in a consumer group
P1 · M5 · Task · deps: GRP-0139 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java classicGroupSyncToConsumerGroup @4.4.0
Done when: SyncGroup returns the serialized ConsumerProtocolAssignment, ILLEGAL_GENERATION on generation mismatch and INCONSISTENT_GROUP_PROTOCOL on protocol mismatch; Heartbeat returns REBALANCE_IN_PROGRESS while the member must rejoin; members not using the classic protocol get UNKNOWN_MEMBER_ID.

### [To Do] GRP-0141 LeaveGroup from classic-protocol members in a consumer group
P1 · M5 · Task · deps: GRP-0139 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java classicGroupLeaveToConsumerGroup @4.4.0
Done when: batched leaves remove matching members with per-member UNKNOWN_MEMBER_ID or FENCED_INSTANCE_ID errors, bump the group epoch once, and may trigger a downgrade; tests per case.

### [To Do] GRP-0142 Validate an online consumer-to-classic downgrade
P1 · M5 · Task · deps: GRP-0136 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java validateOnlineDowngradeWithFencedMembers @4.4.0
Done when: downgrade happens only when the policy allows it, all remaining members use the classic protocol and their count is within `group.max.size`; both the fenced-member and the replaced-static-member paths are covered by tests.

### [To Do] GRP-0143 Convert a consumer group to a classic group
P1 · M5 · Task · deps: GRP-0142, GRP-0139 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java convertToClassicGroup @4.4.0
Done when: the classic group is built from the remaining members with generation equal to the group epoch and current assignments, consumer group tombstones and the group metadata record are written atomically, and a rebalance starts so classic members rejoin; tests for success and append failure.

### [To Do] GRP-0144 Empty-group conversion between types
P1 · M5 · Task · deps: GRP-0136 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java maybeDeleteEmptyConsumerGroup @4.4.0
Done when: a classic JoinGroup to an empty consumer group deletes it and creates a classic group; a consumer heartbeat to an empty classic group deletes it and creates a consumer group; both are logged with reason `empty_group_converted`; tests per direction.

### [To Do] GRP-0145 Rolling upgrade and downgrade test with Java consumers
P1 · M5 · Task · deps: GRP-0138, GRP-0143, TEST-F08 · labels: test
Done when: a compatibility test rolls Java 4.4.0 consumers from `group.protocol=classic` to `consumer` and back under policy `bidirectional`, asserting no partition is consumed by two members at once and committed offsets are preserved.

### [To Do] GRP-0146 Document protocol migration
P2 · M5 · Task · deps: GRP-0143 · labels: docs
Done when: a docs page describes the migration policy values, conversion conditions and errors, and the rolling upgrade procedure.

## [To Do] GRP-F10 Coordinator loading on leadership change
P0 · M5 · Feature · deps: GRP-F01
Scope: Load and unload state on partition leadership changes; error codes while loading.

### [To Do] GRP-0147 Load a coordinator partition from its log
P0 · M5 · Task · deps: GRP-0029, GRP-0007 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorLoaderImpl.java @4.4.0
Done when: the loader reads from log start to the log end offset with a buffer of `offsets.load.buffer.size` (grown to fit a larger batch with a warning), replays data records and COMMIT/ABORT control records, skips unknown record types with a warning, advances last written and committed offsets as the high watermark allows, and reports records and elapsed time.

### [To Do] GRP-0148 Loading is robust against corrupted records
P0 · M5 · Task · deps: GRP-0147, TEST-F07 · labels: decoder, test
Done when: a fuzz/property test feeds malformed batches and values to the loader and asserts a typed load error (FAILED state) and never a panic or unbounded allocation.

### [To Do] GRP-0149 Schedule loading on becoming leader
P0 · M5 · Task · deps: GRP-0147, GRP-0008 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java scheduleLoadOperation @4.4.0
Done when: a leader change for a coordinator partition with a newer leader epoch creates the context and loads it; requests during LOADING get COORDINATOR_LOAD_IN_PROGRESS; on completion the state becomes ACTIVE and `on_loaded` runs; a load request with an older epoch is ignored.

### [To Do] GRP-0150 Unload on resignation or partition deletion
P0 · M5 · Task · deps: GRP-0149 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java scheduleUnloadOperation @4.4.0
Done when: unload with a newer or equal epoch fails deferred events and the current batch with NOT_COORDINATOR, cancels timers and executor tasks, runs `on_unloaded` (classic joins and syncs complete with NOT_COORDINATOR) and closes the context; an unload with a stale epoch is ignored.

### [To Do] GRP-0151 Failed loads
P0 · M5 · Task · deps: GRP-0149 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java withActiveContextOrThrow @4.4.0
Done when: a load error moves the partition to FAILED, logs once with the error, returns NOT_COORDINATOR to requests and retries on the next leadership change; test with an injected read error.

### [To Do] GRP-0152 Rebuild timers and checks after load
P0 · M5 · Task · deps: GRP-0149, GRP-0063, GRP-0096 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java onLoaded @4.4.0
Done when: `on_loaded` reschedules classic heartbeats, consumer and streams session timers, rebalance timeouts for members with pending revocations, and prepares a rebalance for classic groups larger than `group.max.size`; tests per group type.

### [To Do] GRP-0153 Provide the metadata image to newly loaded shards
P0 · M5 · Task · deps: GRP-0149, GRP-0020 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java scheduleLoadOperation @4.4.0
Done when: a freshly loaded shard receives the current image before serving requests so topic-dependent logic (hashes, regexes, deleted topics) is correct; test with a topic deleted while unloaded.

### [To Do] GRP-0154 Simulation of leadership changes during load and writes
P0 · M5 · Task · deps: GRP-0150, GRP-0151 · labels: test
Done when: a seeded simulation with leadership flapping asserts no request is served by a stale coordinator, and that state after reload equals state before unload for committed data.

### [To Do] GRP-0155 Benchmark coordinator load time
P2 · M5 · Task · deps: GRP-0147, TEST-F11, NFR-F06 · labels: perf
Done when: a benchmark loads 1 million offset records and 100 000 group records and reports duration and peak memory against the startup budget from NFR-F06.

## [To Do] GRP-F11 Group metrics and events
P1 · M5 · Feature · deps: GRP-F01, OBS-F04
Scope: Rebalance counts and durations, group states, member events with reasons.

### [To Do] GRP-0156 Group count gauges by type and state
P1 · M5 · Task · deps: GRP-0024 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/metrics/GroupCoordinatorMetrics.java @4.4.0
Done when: Prometheus gauges equivalent to group-count{protocol}, classic-group-count{state} and consumer-group-count{state} are exported from shard counters with docs/metrics.md rows naming the Kafka JMX names; tests assert values after transitions.

### [To Do] GRP-0157 Rebalance metrics
P1 · M5 · Task · deps: GRP-0024 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/metrics/GroupCoordinatorMetricsShard.java @4.4.0
Done when: equivalents of group-completed-rebalance-rate/count and consumer-group-rebalance-rate/count are exported, plus a rebalance duration histogram for classic groups; docs/metrics.md rows exist.

### [To Do] GRP-0158 Offset metrics
P1 · M5 · Task · deps: GRP-0024 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/metrics/GroupCoordinatorMetrics.java @4.4.0
Done when: equivalents of offset-commit-rate/count, offset-expiration-rate/count, offset-deletion-rate/count and offset-count are exported with docs/metrics.md rows; tests assert counts after commits, expiries and deletes.

### [To Do] GRP-0159 Classic group events with reasons
P1 · M5 · Task · deps: GRP-0051, GRP-0049, OBS-F07 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java prepareRebalance @4.4.0
Done when: `group_rebalance_started` (reason: member_joined, member_left, leader_rejoined, metadata_changed, session_timeout, sync_timeout, assignment_failed), `group_rebalance_completed` (generation, members, elapsed_ms), `static_member_replaced` and `group_state_change` events are emitted by the driver and listed in `events.rs`.

### [To Do] GRP-0160 Consumer group events with reasons
P1 · M5 · Task · deps: GRP-0103, GRP-0096, OBS-F07 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java consumerGroupHeartbeat @4.4.0
Done when: `member_joined`, `member_left`, `member_fenced` (reason: session_timeout, rebalance_timeout, epoch_mismatch, static_replaced), `group_epoch_bumped` (reason: subscription_changed, metadata_changed, member_joined, member_left, assignor_changed), `target_assignment_computed` (epoch, assignor, elapsed_ms) and `group_state_change` are emitted and listed in `events.rs`.

### [To Do] GRP-0161 Group type conversion events
P1 · M5 · Task · deps: GRP-0138, GRP-0143, GRP-0144, OBS-F07 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java convertToConsumerGroup @4.4.0
Done when: `group_type_converted` events with group id, from and to types, member count and reason (online_upgrade, online_downgrade, empty_group_converted) are emitted by the driver, recorded in the group's flight recorder and listed in `events.rs`.

### [To Do] GRP-0162 Per-group flight recorders
P1 · M5 · Task · deps: GRP-0159, GRP-0160, OBS-F06
Done when: each group's transitions are recorded in a bounded ring buffer (default 256 entries) with a cap on tracked groups per shard; recorders are dumped on coordinator errors and dropped when a group is deleted; memory bound tested.

### [To Do] GRP-0163 `/debug/groups/{group_id}` diagnostics
P1 · M5 · Task · deps: GRP-0162, OBS-F08
Done when: the endpoint returns group type, state, epochs or generation, members with assignments, offset count and the flight recorder, as JSON and text, for groups on local coordinators; unknown groups return 404; redaction test confirms no secrets.

### [To Do] GRP-0164 Standard tracing fields on coordinator spans
P1 · M5 · Task · deps: GRP-0022, OBS-F03
Done when: coordinator operations run in spans carrying `group_id`, `member_id`, `generation_id` or `member_epoch` as applicable; the field check (OBS-F03) passes.

### [To Do] GRP-0165 Bounded label cardinality for group metrics
P1 · M5 · Task · deps: GRP-0156, GRP-0157, GRP-0158 · labels: test
Done when: a test asserts no group metric carries group id, member id or partition labels and that label sets stay constant as groups are added.

### [To Do] GRP-0166 Assignment and regex resolution timing histograms
P2 · M5 · Task · deps: GRP-0086, GRP-0084
Done when: `alluvium_group_target_assignment_seconds{assignor}` and `alluvium_group_regex_resolution_seconds` histograms are exported and documented as Alluvium-only metrics.

## [To Do] GRP-F12 Group simulation testing
P0 · M5 · Feature · deps: GRP-F04, GRP-F05, TEST-F06
Scope: Rebalance correctness, no double assignment, offset durability under faults.

### [To Do] GRP-0167 Simulated classic and consumer-protocol clients for group simulation
P0 · M5 · Task · deps: GRP-0067, GRP-0095 · labels: test
Done when: testkit clients implement the classic join/sync/heartbeat loop and the consumer heartbeat/reconcile loop as deterministic state machines driven by the simulated network and clock.

### [To Do] GRP-0168 Invariant checker: no partition consumed by two members at once
P0 · M5 · Task · deps: GRP-0167, TEST-F05 · labels: test
Done when: after every step the checker asserts no partition is owned by two members of a group (consumer protocol epochs and classic generations after sync) and prints flight recorders on violation.

### [To Do] GRP-0169 Invariant checker: acknowledged offset commits survive coordinator failover
P0 · M5 · Task · deps: GRP-0167, TEST-F05 · labels: test
Done when: every acknowledged commit is visible after coordinator failover and reload, unless later overwritten, deleted or expired; violation reports include the seed.

### [To Do] GRP-0170 Classic protocol simulation under faults
P0 · M5 · Task · deps: GRP-0168, GRP-0169 · labels: test
Done when: fault plans with member crashes, session timeouts, network partitions and coordinator failover during join and sync run over the CI seed set; groups reach Stable within the documented bound after faults stop.

### [To Do] GRP-0171 Consumer protocol simulation under faults
P0 · M5 · Task · deps: GRP-0168, GRP-0169 · labels: test
Done when: the same fault plans plus revocation timeouts, static member restarts and regex topic churn run over the CI seed set with invariants holding and convergence to Stable.

### [To Do] GRP-0172 Mixed-protocol simulation under migration policy `bidirectional`
P0 · M5 · Task · deps: GRP-0171, GRP-0143 · labels: test
Done when: classic and consumer-protocol members join and leave randomly while groups convert in both directions, with the ownership and offset invariants holding.

### [To Do] GRP-0173 Real-client rebalance compatibility tests
P0 · M5 · Task · deps: TEST-F08 · labels: test
Done when: Java 4.4.0 consumers (classic and consumer protocols), librdkafka and franz-go pass scale-out, scale-in, static-member restart, cooperative-sticky and offset reset scenarios against Alluvium.

### [To Do] GRP-0174 Differential tests of group protocol responses against Kafka
P0 · M5 · Task · deps: TEST-F09 · labels: test
Done when: scripted join/sync/heartbeat/leave/commit/fetch and consumer heartbeat sequences produce identical error codes and states on Kafka 4.4.0 and Alluvium, ignoring member ids and timestamps per the documented rules.

### [To Do] GRP-0175 Seed management for group simulations
P1 · M5 · Task · deps: GRP-0170, GRP-0171 · labels: test
Done when: group simulations run a fixed seed set plus fresh seeds in CI and thousands nightly, and any failing seed is added to the fixed set with its replay command.

## [To Do] GRP-F13 Group limits
P1 · M5 · Feature · deps: GRP-F04, GRP-F05
Scope: Group size limits, session and heartbeat bounds, protection against oversized metadata.

### [To Do] GRP-0176 Classic group size limit across config changes and pending members
P1 · M5 · Task · deps: GRP-0055, GRP-0152 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java onLoaded @4.4.0
Done when: pending members count toward `group.max.size`, a flood of MEMBER_ID_REQUIRED joins cannot grow pending members beyond the limit, and groups loaded over the limit rebalance down to it; tests for each.

### [To Do] GRP-0177 Consumer group size limit edge cases
P1 · M5 · Task · deps: GRP-0078 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java throwIfConsumerGroupIsFull @4.4.0
Done when: static replacement, classic members inside consumer groups and conversions respect `group.consumer.max.size`; tests at the limit for each path.

### [To Do] GRP-0178 Session and heartbeat bounds
P1 · M5 · Task · deps: GRP-0053, GRP-0096
Done when: classic session timeouts are bounded by `group.min/max.session.timeout.ms`; the broker defaults `group.consumer.session.timeout.ms` and `group.consumer.heartbeat.interval.ms` must lie within their min/max keys and the heartbeat interval below the session timeout, failing startup with the key names; tests at each bound.

### [To Do] GRP-0179 Protection against oversized group metadata
P1 · M5 · Task · deps: GRP-0067, GRP-0138 · labels: security · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java appendGroupMetadataErrorToResponseError @4.4.0
Done when: protocol metadata and assignments that make the group record exceed the partition's `max.message.bytes` fail with the mapped error (UNKNOWN_SERVER_ERROR) without corrupting state; embedded ConsumerProtocol subscriptions and assignments are parsed with bounded lengths and malformed input returns INCONSISTENT_GROUP_PROTOCOL; fuzz target for the embedded protocol parser.

### [To Do] GRP-0180 Bounded regex compilation
P1 · M5 · Task · deps: GRP-0083 · labels: security
Done when: regex compilation uses an explicit size limit, patterns exceeding it return INVALID_REGULAR_EXPRESSION, resolution time per group is bounded, and the limit is documented as a deviation in the regex ADR.

### [To Do] GRP-0181 Memory accounting for coordinator state
P2 · M5 · Task · deps: GRP-0073, GRP-0044, GRP-0114, NFR-F05 · labels: nfr
Done when: approximate bytes per shard for groups, members and offsets are exported as a gauge and a test proves 100 000 groups of 10 members with 100 offsets each stay within the budget recorded in docs/nfr.md.
