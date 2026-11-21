# SHARE: Share groups

> Crates: `alluvium-share-coordinator` (layer 2) · Kafka counterpart: `share-coordinator/`, `server/share` · Milestones: M8-M9

KIP-932 share groups, and the per-record delivery state store that later powers native queue semantics.

## [To Do] SHARE-F01 Share group membership
P0 · M8 · Feature · deps: GRP-F01
Scope: ShareGroupHeartbeat, member epochs, assignment.

### [To Do] SHARE-0001 Share group and member model
P0 · M8 · Task · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroup.java @4.4.0
Done when: share groups hold group epoch, target assignment epoch, members (member id, rack id, client id/host, subscribed topic names, member epoch, assignment), target assignments and subscribed topic counts on timeline structures; ShareGroupState is EMPTY, STABLE or DEAD (UNKNOWN only for describe); unit tests.

### [To Do] SHARE-0002 Share group record replay
P0 · M8 · Task · deps: SHARE-0001 · ref: group-coordinator/src/main/resources/common/message/ShareGroupMetadataValue.json @4.4.0
Done when: ShareGroupMetadata, ShareGroupMemberMetadata, ShareGroupTargetAssignmentMetadata/Member, ShareGroupCurrentMemberAssignment and ShareGroupStatePartitionMetadata records and tombstones replay into the model; round-trip and replay tests per record type.

### [To Do] SHARE-0003 ShareGroupHeartbeat request validation
P0 · M8 · Task · deps: SHARE-0001 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java throwIfShareGroupHeartbeatRequestIsInvalid @4.4.0
Done when: an empty member id, group id or rack id returns INVALID_REQUEST; epoch 0 without subscribed topic names returns INVALID_REQUEST; an epoch below -1 returns INVALID_REQUEST; one test per rule.

### [To Do] SHARE-0004 Gate share groups on `share.version`
P0 · M8 · Task · deps: SHARE-0003, META-F06 · ref: server-common/src/main/java/org/apache/kafka/server/common/ShareVersion.java @4.4.0
Done when: ShareGroupHeartbeat, ShareFetch and ShareAcknowledge return UNSUPPORTED_VERSION when the finalized `share.version` is 0, and lowering the level stops share partitions and sessions as Kafka's `onShareVersionToggle` does; tests.

### [To Do] SHARE-0005 Share member join and group creation
P0 · M8 · Task · deps: SHARE-0002 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java getOrMaybeSubscribeShareGroupMember @4.4.0
Done when: epoch 0 creates the group if absent; a group of another type returns GROUP_ID_NOT_FOUND; a non-zero epoch for a missing group returns GROUP_ID_NOT_FOUND; the member is created with its subscription and the group epoch is bumped; tests.

### [To Do] SHARE-0006 Enforce `group.share.max.size`
P0 · M8 · Task · deps: SHARE-0005 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java throwIfShareGroupIsFull @4.4.0
Done when: a new member beyond `group.share.max.size` gets GROUP_MAX_SIZE_REACHED and existing members are unaffected; test at the limit.

### [To Do] SHARE-0007 Share member epoch fencing
P0 · M8 · Task · deps: SHARE-0005 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java throwIfShareGroupMemberEpochIsInvalid @4.4.0
Done when: an unknown member with a non-zero epoch returns UNKNOWN_MEMBER_ID, and an epoch greater or smaller than the member's returns FENCED_MEMBER_EPOCH; tests.

### [To Do] SHARE-0008 Share subscription changes and group epoch bumps
P0 · M8 · Task · deps: SHARE-0005, GRP-F05 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java shareGroupHeartbeat @4.4.0
Done when: a change of subscribed topic names or of the subscribed topics' metadata hash bumps the group epoch and persists ShareGroupMetadata; unchanged subscriptions do not bump; tests.

### [To Do] SHARE-0009 Simple share assignor
P0 · M8 · Task · deps: GRP-F06 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/assignor/SimpleAssignor.java @4.4.0
Done when: the `simple` assignor balances partitions so member counts differ by at most one, prefers keeping existing assignments, handles homogeneous and heterogeneous subscriptions like Kafka's builders, ignores racks, and ported `SimpleAssignorTest` cases pass.

### [To Do] SHARE-0010 Share target assignment and direct reconciliation
P0 · M8 · Task · deps: SHARE-0008, SHARE-0009 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupAssignmentBuilder.java @4.4.0
Done when: when the group epoch exceeds the target epoch the target is recomputed with the share assignor (selected from `group.share.assignors`), at most once per `share.assignment.interval.ms` (bounded by `group.share.min/max.assignment.interval.ms`), optionally offloaded per `share.assignor.offload.enable`; members move straight to the target epoch without revocation; tests.

### [To Do] SHARE-0011 ShareGroupHeartbeat response content
P0 · M8 · Task · deps: SHARE-0010 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java shareGroupHeartbeat @4.4.0
Done when: responses carry member id, member epoch, heartbeat interval (`share.heartbeat.interval.ms` group config or `group.share.heartbeat.interval.ms`) and the assignment when it changed or on join; tests.

### [To Do] SHARE-0012 Share member session timeout and leave
P0 · M8 · Task · deps: SHARE-0005 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java scheduleShareGroupSessionTimeout @4.4.0
Done when: each heartbeat reschedules the member's session timer from `share.session.timeout.ms` (bounded by `group.share.min/max.session.timeout.ms`); expiry or a leave with epoch -1 removes the member, tombstones its records, bumps the group epoch and notifies share sessions so the member's acquired records are released; tests.

### [To Do] SHARE-0013 Initialize share-group state for newly subscribed partitions
P0 · M8 · Task · deps: SHARE-0010, SHARE-F05 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java maybeCreateInitializeShareGroupStateRequest @4.4.0
Done when: partitions newly subscribed by the group are recorded as initializing in ShareGroupStatePartitionMetadata, an InitializeShareGroupState request is sent through the persister with start offsets from `share.auto.offset.reset`, success marks them initialized and failure uninitializes them for retry after `group.share.initialize.retry.interval.ms`; tests for success, failure and retry.

### [To Do] SHARE-0014 Share group state transitions
P0 · M8 · Task · deps: SHARE-0005, SHARE-0012 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroup.java @4.4.0
Done when: the first member makes the group Stable, the last member leaving makes it Empty, and deletion makes it Dead; each transition is returned with a reason for events; unit tests.

### [To Do] SHARE-0015 Share groups react to topic deletion
P0 · M8 · Task · deps: SHARE-0013 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java onMetadataUpdate @4.4.0
Done when: a deleted topic is removed from initializing and initialized sets in ShareGroupStatePartitionMetadata and its share state deletion is triggered; tests.

### [To Do] SHARE-0016 Property test of share group membership invariants
P0 · M8 · Task · deps: SHARE-0010, SHARE-0012 · labels: test
Done when: random heartbeats, leaves and timeouts keep member epoch <= target epoch <= group epoch and every subscribed partition assigned to at least one member when members exist.

## [To Do] SHARE-F02 Share partition
P0 · M8 · Feature · deps: SHARE-F08
Scope: Start and end offsets, in-flight record states, acquisition locks, delivery counts, max attempts.

### [To Do] SHARE-0017 Share partition lifecycle states
P0 · M8 · Task · deps: none · ref: core/src/main/java/kafka/server/share/SharePartition.java initializedOrThrowException @4.4.0
Done when: EMPTY, INITIALIZING, ACTIVE, FAILED and FENCED are modelled; requests to an INITIALIZING or FENCED partition fail with LEADER_NOT_AVAILABLE and to a FAILED partition with an internal error that removes it; transitions are returned with reasons; unit tests.

### [To Do] SHARE-0018 Initialize a share partition from persisted state
P0 · M8 · Task · deps: SHARE-0017, SHARE-0093 · ref: core/src/main/java/kafka/server/share/SharePartition.java maybeInitialize @4.4.0
Done when: initialization emits a read-state action and, on its result, loads start offset, state epoch, leader epoch and state batches into the cache; persister errors map as Kafka does (coordinator unavailable errors to COORDINATOR_NOT_AVAILABLE, GROUP_ID_NOT_FOUND, UNKNOWN_TOPIC_OR_PARTITION, NOT_LEADER_OR_FOLLOWER, FENCED_STATE_EPOCH fences, others UNKNOWN_SERVER_ERROR); tests per error.

### [To Do] SHARE-0019 Start offset for uninitialized share partitions
P0 · M8 · Task · deps: SHARE-0018 · ref: core/src/main/java/kafka/server/share/SharePartition.java startOffsetDuringInitialization @4.4.0
Done when: an uninitialized persisted start offset is resolved with `share.auto.offset.reset`: `earliest` to the log start offset, `latest` to the log end offset, `by_duration:<ISO-8601 duration>` to the offset for now minus the duration; tests per strategy.

### [To Do] SHARE-0020 Share partition registry on the broker
P0 · M8 · Task · deps: SHARE-0017 · ref: core/src/main/java/kafka/server/share/SharePartitionCache.java @4.4.0
Done when: share partitions are created on first fetch only on the partition leader, keyed by group and topic-id partition, and removed when leadership is lost, the topic or group is deleted, or the partition is fenced; tests per removal cause.

### [To Do] SHARE-0021 Per-partition fetch lock
P0 · M8 · Task · deps: SHARE-0017 · ref: core/src/main/java/kafka/server/share/SharePartition.java maybeAcquireFetchLock @4.4.0
Done when: at most one share fetch at a time owns a partition's fetch lock (by fetch id), the lock is released on completion or failure, and lock hold time is measured; tests with two concurrent fetches.

### [To Do] SHARE-0022 Next fetch offset computation
P0 · M8 · Task · deps: SHARE-0018, SHARE-0089 · ref: core/src/main/java/kafka/server/share/SharePartition.java nextFetchOffset @4.4.0
Done when: the next fetch offset is the first AVAILABLE offset in the cache, or the end offset plus one when none is available, recomputed only when flagged; tests.

### [To Do] SHARE-0023 Acquire newly fetched records
P0 · M8 · Task · deps: SHARE-0022, SHARE-0021, SHARE-0092 · ref: core/src/main/java/kafka/server/share/SharePartition.java acquireNewBatchRecords @4.4.0
Done when: records beyond the end offset become ACQUIRED with delivery count 1 for the fetching member with a lock deadline from `share.record.lock.duration.ms`; acquisition stops at the fetch's max records and at `share.partition.max.record.locks` in-flight records; tests at both limits.

### [To Do] SHARE-0024 Re-acquire available records for redelivery
P0 · M8 · Task · deps: SHARE-0023 · ref: core/src/main/java/kafka/server/share/SharePartition.java acquire @4.4.0
Done when: AVAILABLE records in the cache are re-acquired with an incremented delivery count, whole-batch when the range matches and per-offset otherwise; records with an ongoing transition are skipped; records close to the delivery limit are delivered in smaller ranges as Kafka's throttling does; tests.

### [To Do] SHARE-0025 Acquire modes (KIP-1206)
P0 · M8 · Task · deps: SHARE-0024 · ref: core/src/main/java/kafka/server/share/SharePartition.java isRecordLimitMode @4.4.0
Done when: `batch_optimized` acquires whole batches and may exceed max records, while `record_limit` stops exactly at max records splitting batches; the mode comes from ShareFetch v2 (v1 uses batch_optimized); tests per mode.

### [To Do] SHARE-0026 Read-committed share partitions
P0 · M8 · Task · deps: SHARE-0023, TXN-F11 · ref: core/src/main/java/kafka/server/share/SharePartition.java filterAbortedTransactionalAcquiredRecords @4.4.0
Done when: with `share.isolation.level=read_committed` fetches stop at the last stable offset, records of aborted transactions and control batches are archived instead of delivered, and `read_uncommitted` delivers up to the high watermark; tests with committed and aborted transactions.

### [To Do] SHARE-0027 Handle log start offset movement
P0 · M8 · Task · deps: SHARE-0090 · ref: core/src/main/java/kafka/server/share/SharePartition.java updateCacheAndOffsets @4.4.0
Done when: when the log start offset passes the share start offset (retention, DeleteRecords or OFFSET_OUT_OF_RANGE on fetch) AVAILABLE records below it are archived, the start offset moves and the change is persisted; tests.

### [To Do] SHARE-0028 Acquisition lock expiry
P0 · M8 · Task · deps: SHARE-0092, SHARE-0029 · ref: core/src/main/java/kafka/server/share/SharePartition.java releaseAcquisitionLockOnTimeout @4.4.0
Done when: an expired lock releases its records to AVAILABLE (or archives them at the delivery limit), persists the change, counts the expiry and wakes delayed share fetches for the partition; tests with the simulated clock.

### [To Do] SHARE-0029 Persist state changes with commit or rollback
P0 · M8 · Task · deps: SHARE-0093, SHARE-0087 · ref: core/src/main/java/kafka/server/share/SharePartition.java writeShareGroupState @4.4.0
Done when: state changes produce a write-state action with state epoch, leader epoch, start offset, delivery-complete count and state batches; success commits the pending transitions and failure rolls them back; FENCED_STATE_EPOCH, NOT_LEADER_OR_FOLLOWER, GROUP_ID_NOT_FOUND and UNKNOWN_TOPIC_OR_PARTITION fence and remove the partition; tests per error.

### [To Do] SHARE-0033 Port Kafka's share partition tests
P1 · M8 · Task · deps: SHARE-0024, SHARE-0028, SHARE-0029, TEST-F10 · labels: test · ref: core/src/main/java/kafka/server/share/SharePartition.java @4.4.0
Done when: the `SharePartitionTest` cases in the TEST-F10 inventory are ported with attributions and pass.

## [To Do] SHARE-F03 ShareFetch
P0 · M8 · Feature · deps: SHARE-F02
Scope: Share sessions, acquisition, record limits.

### [To Do] SHARE-0034 ShareFetch request validation
P0 · M8 · Task · deps: SHARE-0004 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleShareFetchRequest @4.4.0
Done when: a shared validation used by the ShareFetch handler (API-F78) returns INVALID_REQUEST for a null group id or invalid member id; v2 requests with `IsRenewAck` must have max bytes, min bytes, max records and max wait all 0, otherwise INVALID_REQUEST naming the fields; tests per rule.

### [To Do] SHARE-0035 Share session cache
P0 · M8 · Task · deps: none · ref: server/src/main/java/org/apache/kafka/server/share/session/ShareSessionCache.java @4.4.0
Done when: sessions are keyed by group id and member id, hold their partitions and epoch, are bounded by `group.share.max.share.sessions`, are removed when their connection disconnects or their member leaves, and notify listeners when a group has no sessions left; tests.

### [To Do] SHARE-0036 Share session epochs for ShareFetch
P0 · M8 · Task · deps: SHARE-0035 · ref: core/src/main/java/kafka/server/share/SharePartitionManager.java newContext @4.4.0
Done when: epoch 0 replaces any existing session with a new one (SHARE_SESSION_LIMIT_REACHED when the cache is full); epoch -1 requires an existing session (else SHARE_SESSION_NOT_FOUND), allows no acknowledgements with a fetch of new partitions as Kafka does, and closes it; other epochs require an existing session (SHARE_SESSION_NOT_FOUND) with the matching epoch (INVALID_SHARE_SESSION_EPOCH), apply added and forgotten partitions and advance the epoch; tests per case.

### [To Do] SHARE-0037 Delay SHARE_SESSION_LIMIT_REACHED responses
P0 · M8 · Task · deps: SHARE-0036 · ref: core/src/main/java/kafka/server/share/SharePartitionManager.java createIdleShareFetchTimerTask @4.4.0
Done when: when no session slot is available the response is sent after the request's max wait instead of immediately, so clients do not spin; test with the simulated clock.

### [To Do] SHARE-0038 Delayed share fetch
P0 · M8 · Task · deps: SHARE-0036, SHARE-0023, SRV-F07 · ref: core/src/main/java/kafka/server/share/DelayedShareFetch.java @4.4.0
Done when: a share fetch acquires fetch locks for its partitions, reads from the log and acquires records; if fewer than min bytes are acquired it waits up to max wait in the delayed-operation framework, woken by new data, released records or lock expiry; locks are always released; tests per completion trigger.

### [To Do] SHARE-0039 Per-partition byte budgets and partition rotation
P0 · M8 · Task · deps: SHARE-0038 · ref: server/src/main/java/org/apache/kafka/server/share/fetch/PartitionMaxBytesStrategy.java @4.4.0
Done when: request max bytes are split across partitions with Kafka's uniform strategy and partitions are rotated between fetches so none starves; tests against Kafka's examples.

### [To Do] SHARE-0040 Share fetch response building
P0 · M8 · Task · deps: SHARE-0038, SHARE-0025 · ref: core/src/main/java/kafka/server/share/ShareFetchUtils.java @4.4.0
Done when: each partition returns its acquired record ranges with delivery counts, the record batches, and the acquisition lock timeout; max records and batch size from the request bound acquisition; OFFSET_OUT_OF_RANGE from the log moves the share start offset and retries; tests.

### [To Do] SHARE-0041 ShareFetch per-partition errors
P0 · M8 · Task · deps: SHARE-0038 · ref: core/src/main/java/kafka/server/share/SharePartitionManager.java fetchMessages @4.4.0
Done when: unknown partitions return UNKNOWN_TOPIC_OR_PARTITION (UNKNOWN_TOPIC_ID for unknown ids), non-leaders NOT_LEADER_OR_FOLLOWER with current leader information, initializing or fenced partitions LEADER_NOT_AVAILABLE, and persister failures their mapped error; tests per error.

### [To Do] SHARE-0042 Acknowledgements carried in ShareFetch
P0 · M8 · Task · deps: SHARE-0036, SHARE-F04 · ref: core/src/main/java/kafka/server/share/SharePartitionManager.java acknowledge @4.4.0
Done when: acknowledgement batches in a ShareFetch are validated and applied before fetching, with results in the response's acknowledge error fields; a fetch that only renews (`IsRenewAck`) returns without fetching; tests.

### [To Do] SHARE-0043 Share fetches from tiered storage
P1 · M9 · Task · deps: SHARE-0038, TIER-F04 · ref: core/src/main/java/kafka/server/share/PendingRemoteFetches.java @4.4.0
Done when: share fetches whose offsets are only in remote storage use the remote fetch path and complete when remote reads finish, with locks held until then; tests with a fake remote store.

### [To Do] SHARE-0044 Port Kafka's share fetch tests
P1 · M8 · Task · deps: SHARE-0040, SHARE-0041, TEST-F10 · labels: test · ref: core/src/main/java/kafka/server/share/SharePartitionManager.java @4.4.0
Done when: `SharePartitionManagerTest` and `DelayedShareFetchTest` cases in the TEST-F10 inventory are ported with attributions and pass.

## [To Do] SHARE-F04 ShareAcknowledge
P0 · M8 · Feature · deps: SHARE-F02
Scope: Accept, release, reject and renew acknowledgements.

### [To Do] SHARE-0045 Acknowledgement batch validation
P0 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala validateAcknowledgementBatches @4.4.0
Done when: a batch with first offset above last offset, overlapping a previous batch, with no types, with more than one type whose count differs from the range length, with a type outside 0..3 (0..4 from v2) or with RENEW(4) outside a renew request returns INVALID_REQUEST for its partition; the function is shared by the ShareFetch and ShareAcknowledge handlers (API-F78, API-F79); tests per rule.

### [To Do] SHARE-0046 Share session epochs for ShareAcknowledge
P0 · M8 · Task · deps: SHARE-0035 · ref: core/src/main/java/kafka/server/share/SharePartitionManager.java acknowledgeSessionUpdate @4.4.0
Done when: epoch 0 returns INVALID_SHARE_SESSION_EPOCH; a missing session returns SHARE_SESSION_NOT_FOUND; epoch -1 applies the acknowledgements and closes the session; other epochs must match (INVALID_SHARE_SESSION_EPOCH) and advance it; tests.

### [To Do] SHARE-0047 Apply acknowledgements to a share partition
P0 · M8 · Task · deps: SHARE-0045, SHARE-0029 · ref: core/src/main/java/kafka/server/share/SharePartition.java acknowledge @4.4.0
Done when: ACCEPT moves ACQUIRED records to ACKNOWLEDGED, RELEASE to AVAILABLE, REJECT to ARCHIVED (ARCHIVING with DLQ) and GAP applies to gap offsets; records not acquired by the member return INVALID_RECORD_STATE; offsets outside in-flight records return INVALID_REQUEST; changes are persisted and the response waits for persistence, rolling back on failure; tests per type and error.

### [To Do] SHARE-0048 Renew acknowledgements (KIP-1222)
P0 · M8 · Task · deps: SHARE-0047, SHARE-0092 · ref: core/src/main/java/kafka/server/share/SharePartition.java acknowledge @4.4.0
Done when: RENEW(4) on records acquired by the member extends their lock deadline without changing state or delivery count, only when `share.renew.acknowledge.enable` is true for the group; ShareAcknowledge v2 and ShareFetch v2 renew requests return the new acquisition lock timeout; tests.

### [To Do] SHARE-0049 Release records when a member leaves or its session closes
P0 · M8 · Task · deps: SHARE-0047 · ref: core/src/main/java/kafka/server/share/SharePartition.java releaseAcquiredRecords @4.4.0
Done when: records ACQUIRED by the member return to AVAILABLE (or archive at the delivery limit) and are persisted when its session closes, its connection drops or it leaves the group; tests per trigger.

### [To Do] SHARE-0050 ShareAcknowledge per-partition errors
P0 · M8 · Task · deps: SHARE-0047 · ref: core/src/main/java/kafka/server/share/SharePartitionManager.java acknowledge @4.4.0
Done when: unknown partitions return UNKNOWN_TOPIC_OR_PARTITION, partitions without a share partition on this broker NOT_LEADER_OR_FOLLOWER, and persister failures their mapped error, each partition independently; failed acknowledgements are counted per group; tests.

### [To Do] SHARE-0051 Port Kafka's acknowledgement tests
P1 · M8 · Task · deps: SHARE-0047, SHARE-0048, TEST-F10 · labels: test · ref: core/src/main/java/kafka/server/share/SharePartition.java @4.4.0
Done when: acknowledgement cases from `SharePartitionTest` and `SharePartitionManagerTest` in the TEST-F10 inventory are ported with attributions and pass.

## [To Do] SHARE-F05 Share state persistence
P0 · M8 · Feature · deps: SHARE-F02
Scope: Share coordinator and `__share_group_state`; Initialize/Read/Write/Delete/ReadSummary APIs.

### [To Do] SHARE-0052 Vendor and generate the share coordinator record schemas
P0 · M8 · Task · deps: PROTO-F05 · labels: decoder · ref: share-coordinator/src/main/resources/common/message/ShareSnapshotValue.json @4.4.0
Done when: ShareSnapshotKey/Value and ShareUpdateKey/Value schemas are vendored unmodified with checksums, types and serde are generated, and round-trip property tests pass.

### [To Do] SHARE-0053 Fuzz target and golden bytes for share state records
P0 · M8 · Task · deps: SHARE-0052, TEST-F07 · labels: decoder, test
Done when: a cargo-fuzz target decodes arbitrary share state keys and values without panics; golden records written by Kafka 4.4.0 decode and re-encode byte for byte.

### [To Do] SHARE-0054 Map a share partition key to its `__share_group_state` partition
P0 · M8 · Task · deps: GRP-F02 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorService.java partitionFor @4.4.0
Done when: the key `groupId:topicId:partition` maps with the Java-compatible string hash modulo `share.coordinator.state.topic.num.partitions`; malformed keys are rejected; golden tests against Kafka.

### [To Do] SHARE-0055 `__share_group_state` topic configuration and auto-creation
P0 · M8 · Task · deps: CFG-F09 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorService.java shareGroupStateTopicConfigs @4.4.0
Done when: creation uses `share.coordinator.state.topic.num.partitions`, `share.coordinator.state.topic.replication.factor`, `min.insync.replicas` from `share.coordinator.state.topic.min.isr`, `segment.bytes` from `share.coordinator.state.topic.segment.bytes` and `cleanup.policy=delete` as in Kafka; tests assert the config map.

### [To Do] SHARE-0056 FindCoordinator SHARE key type
P0 · M8 · Task · deps: SHARE-0054, SHARE-0055, GRP-F03 · ref: core/src/main/scala/kafka/server/KafkaApis.scala getCoordinator @4.4.0
Done when: SHARE keys (v6+) resolve to the leader of their `__share_group_state` partition, auto-create the topic, return COORDINATOR_NOT_AVAILABLE until a leader exists and INVALID_REQUEST for malformed keys; tests.

### [To Do] SHARE-0057 Share coordinator shard state and replay
P0 · M8 · Task · deps: SHARE-0052, GRP-F01 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorShard.java replay @4.4.0
Done when: the shard keeps per share partition the snapshot epoch, state epoch, leader epoch, start offset, delivery-complete count and state batches; ShareSnapshot replaces and ShareUpdate merges state; tombstones delete; replay tests per record.

### [To Do] SHARE-0058 Merge persisted state batches
P0 · M8 · Task · deps: SHARE-0057 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/PersisterStateBatchCombiner.java @4.4.0
Done when: new batches are combined with existing ones by splitting overlaps (newer wins), merging adjacent batches with equal state and delivery count, and dropping batches below the start offset; ported `PersisterStateBatchCombinerTest` cases pass.

### [To Do] SHARE-0059 WriteShareGroupState handling
P0 · M8 · Task · deps: SHARE-0058 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorShard.java writeState @4.4.0
Done when: a null topic id, negative partition, negative leader epoch or negative state epoch returns INVALID_REQUEST; writing an uninitialized partition INVALID_REQUEST; a lower leader epoch FENCED_LEADER_EPOCH; a lower state epoch FENCED_STATE_EPOCH; an unknown topic or partition UNKNOWN_TOPIC_OR_PARTITION; otherwise a ShareUpdate (or ShareSnapshot every `share.coordinator.snapshot.update.records.per.snapshot` updates) is written; v1 stores the delivery-complete count; tests per error.

### [To Do] SHARE-0060 ReadShareGroupState handling
P0 · M8 · Task · deps: SHARE-0057 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorShard.java readStateAndMaybeUpdateLeaderEpoch @4.4.0
Done when: invalid ids return INVALID_REQUEST, an uninitialized partition INVALID_REQUEST, a lower leader epoch FENCED_LEADER_EPOCH, an unknown topic or partition UNKNOWN_TOPIC_OR_PARTITION; a higher leader epoch is persisted before returning state epoch, start offset and batches; tests.

### [To Do] SHARE-0061 ReadShareGroupStateSummary handling
P0 · M8 · Task · deps: SHARE-0057 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorShard.java readStateSummary @4.4.0
Done when: the summary returns state epoch, leader epoch, start offset and (v1) delivery-complete count with the same validation errors as reads, without updating the leader epoch; tests.

### [To Do] SHARE-0062 InitializeShareGroupState handling
P0 · M8 · Task · deps: SHARE-0057 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorShard.java initializeState @4.4.0
Done when: invalid ids or a negative state epoch return INVALID_REQUEST, a state epoch not newer than the stored one FENCED_STATE_EPOCH, an unknown topic or partition UNKNOWN_TOPIC_OR_PARTITION; otherwise a snapshot with the start offset and no batches is written; tests.

### [To Do] SHARE-0063 DeleteShareGroupState handling
P0 · M8 · Task · deps: SHARE-0057 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorShard.java deleteState @4.4.0
Done when: invalid ids return INVALID_REQUEST, unknown topics or partitions UNKNOWN_TOPIC_OR_PARTITION, and existing state is tombstoned; deleting absent state succeeds; tests.

### [To Do] SHARE-0064 Snapshot cold share partitions
P0 · M8 · Task · deps: SHARE-0059 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorShard.java snapshotColdPartitions @4.4.0
Done when: every `share.coordinator.cold.partition.snapshot.interval.ms` partitions whose latest record is an update are snapshotted so old records become redundant; test with the simulated clock.

### [To Do] SHARE-0065 Prune `__share_group_state`
P0 · M8 · Task · deps: SHARE-0064, STOR-F13 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorShard.java lastRedundantOffset @4.4.0
Done when: every `share.coordinator.state.topic.prune.interval.ms` the coordinator computes the last redundant offset (below every live snapshot) and advances the log start offset of its partition with a DeleteRecords-equivalent call, reporting `last-pruned-offset`; tests.

### [To Do] SHARE-0066 Remove share state of deleted topics
P0 · M8 · Task · deps: SHARE-0057 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorShard.java maybeCleanupShareState @4.4.0
Done when: state of share partitions whose topics were deleted is tombstoned when the metadata delta arrives; test.

### [To Do] SHARE-0067 Share coordinator service on the coordinator runtime
P0 · M8 · Task · deps: SHARE-0059, SHARE-0060, SHARE-0061, SHARE-0062, SHARE-0063 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorService.java @4.4.0
Done when: the service routes each partition's request to its shard with `share.coordinator.threads` workers, `share.coordinator.write.timeout.ms`, `share.coordinator.load.buffer.size`, `share.coordinator.append.linger.ms`, `share.coordinator.state.topic.compression.codec` and `share.coordinator.cached.buffer.max.bytes`; inactive returns COORDINATOR_NOT_AVAILABLE; runtime errors use the common mapping; tests.

### [To Do] SHARE-0068 Default state persister client
P0 · M8 · Task · deps: SHARE-0056, NET-F10 · ref: server-common/src/main/java/org/apache/kafka/server/share/persister/PersisterStateManager.java @4.4.0
Done when: brokers send Initialize/Read/Write/Delete/ReadSummary share state requests to the share coordinator found with FindCoordinator SHARE keys, batched per coordinator node, retrying COORDINATOR_NOT_AVAILABLE, NOT_COORDINATOR and COORDINATOR_LOAD_IN_PROGRESS with backoff and re-lookup, with bounded attempts and timeouts; tests with a simulated network.

### [To Do] SHARE-0069 Simulation of share state persistence under coordinator failover
P0 · M8 · Task · deps: SHARE-0067, SHARE-0068, TEST-F06 · labels: test
Done when: seeded simulations fail over the share coordinator during writes and assert every acknowledged write is readable afterwards and fenced writers are rejected.

### [To Do] SHARE-0070 Document the `__share_group_state` record format
P1 · M8 · Task · deps: SHARE-0052 · labels: docs
Done when: docs/formats/share-group-state.md lists ShareSnapshot and ShareUpdate keys and values with hex examples checked by a doc test, and the dump decoder for CLI-F04 is exposed.

## [To Do] SHARE-F06 Share group administration
P1 · M8 · Feature · deps: SHARE-F01
Scope: Describe, offsets describe/alter/delete, group deletion.

### [To Do] SHARE-0071 ShareGroupDescribe
P1 · M8 · Task · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java shareGroupDescribe @4.4.0
Done when: each group returns state, epochs, assignor and members with rack, client id/host, subscriptions and assignment with topic names; a missing or non-share group returns GROUP_ID_NOT_FOUND; tests.

### [To Do] SHARE-0072 DescribeShareGroupOffsets
P1 · M8 · Task · deps: SHARE-F05 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java describeShareGroupOffsets @4.4.0
Done when: for listed or all initialized partitions the start offset and leader epoch come from ReadShareGroupStateSummary; v1 adds lag computed from the delivery-complete count as in KIP-1226; uninitialized partitions report the uninitialized offset; a missing group returns GROUP_ID_NOT_FOUND; tests.

### [To Do] SHARE-0073 AlterShareGroupOffsets
P1 · M8 · Task · deps: SHARE-F05 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java alterShareGroupOffsets @4.4.0
Done when: the group is created if absent; a non-empty group returns NON_EMPTY_GROUP; unknown topics or partitions return UNKNOWN_TOPIC_OR_PARTITION per partition; valid partitions are initialized with the given start offsets through the persister and recorded in ShareGroupStatePartitionMetadata; tests.

### [To Do] SHARE-0074 DeleteShareGroupOffsets
P1 · M8 · Task · deps: SHARE-F05 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java completeDeleteShareGroupOffsets @4.4.0
Done when: a non-empty group returns NON_EMPTY_GROUP; topics are marked deleting, their share state is deleted through the persister, and per-topic errors are returned; tests.

### [To Do] SHARE-0075 DeleteGroups for share groups
P1 · M8 · Task · deps: SHARE-F05 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java shareGroupBuildPartitionDeleteRequest @4.4.0
Done when: a non-empty share group returns NON_EMPTY_GROUP; otherwise its share state is deleted through the persister first and then the group records are tombstoned; a persister failure returns its error and keeps the group; tests.

### [To Do] SHARE-0076 List share groups
P1 · M8 · Task · deps: GRP-F08
Done when: ListGroups with the `share` type filter returns share groups with their states; tests.

### [To Do] SHARE-0077 Clean up share state for topics leaving subscriptions
P1 · M8 · Task · deps: SHARE-0075 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java maybeCleanupShareGroupState @4.4.0
Done when: partitions of topics no longer subscribed by the group are moved to deleting and their share state is deleted; tests.

## [To Do] SHARE-F07 Share configuration
P1 · M8 · Feature · deps: CFG-F09
Scope: `group.share.*` keys and validation.

### [To Do] SHARE-0078 Cross-key validation of share group broker configs
P1 · M8 · Task · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java validate @4.4.0
Done when: `group.share.record.lock.duration.ms`, `group.share.delivery.count.limit` and `group.share.partition.max.record.locks` lie within their min/max keys, session and heartbeat values within theirs, and violations fail startup with the key names; tests per rule.

### [To Do] SHARE-0079 Share group config keys for the GROUP resource
P1 · M8 · Task · deps: GRP-F08 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java @4.4.0
Done when: `share.session.timeout.ms`, `share.heartbeat.interval.ms`, `share.record.lock.duration.ms`, `share.delivery.count.limit`, `share.partition.max.record.locks`, `share.auto.offset.reset`, `share.isolation.level`, `share.renew.acknowledge.enable`, `share.assignment.interval.ms`, `share.assignor.offload.enable`, `errors.deadletterqueue.topic.name` and `errors.deadletterqueue.copy.record.enable` are validated against broker bounds (INVALID_CONFIG otherwise); tests per key.

### [To Do] SHARE-0080 Parse `share.auto.offset.reset`
P1 · M8 · Task · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/ShareGroupAutoOffsetResetStrategy.java @4.4.0
Done when: `earliest`, `latest` and `by_duration:<ISO-8601 duration>` parse case-insensitively and anything else (including negative durations) is rejected; tests.

### [To Do] SHARE-0081 Apply dynamic share configs
P1 · M8 · Task · deps: SHARE-0079, SHARE-F02 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfigProvider.java @4.4.0
Done when: group config changes reach share partitions and the coordinator without restart: lock duration and delivery limit apply to new acquisitions, record lock limits to new fetches, isolation level and reset strategy to new initializations; tests per key.

### [To Do] SHARE-0082 Share coordinator config consumption
P1 · M8 · Task · deps: none · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java @4.4.0
Done when: all `share.coordinator.*` keys map into the share coordinator config struct with Kafka's validation, and `share.fetch.purgatory.purge.interval.requests` and `group.share.max.share.sessions` reach the broker share components; tests.

### [To Do] SHARE-0083 Document share group configuration
P2 · M8 · Task · deps: SHARE-0081 · labels: docs
Done when: a docs page explains broker and group-level share keys, their bounds and when changes take effect.

## [To Do] SHARE-F08 Per-record delivery state store
P0 · M8 · Feature · deps: STOR-F08 · labels: foundation-for:X-QUEUE
Scope: Generic store of per-offset delivery state with acquisition and redelivery.

### [To Do] SHARE-0084 Decide the design of the per-record delivery state store
P0 · M8 · Spike · deps: none · labels: foundation-for:X-QUEUE · ref: server/src/main/java/org/apache/kafka/server/share/fetch/InFlightState.java @4.4.0
Done when: ADR `delivery-state-store` accepted in docs/adr/ covering batch-level versus per-offset state, two-phase updates pending persistence, lock deadline handling with the timer wheel, memory bounds, and the extension points X-QUEUE needs (visibility timeouts, delays) without implementing them.

### [To Do] SHARE-0085 Record delivery states and transition rules
P0 · M8 · Task · deps: SHARE-0084 · ref: server/src/main/java/org/apache/kafka/server/share/fetch/RecordState.java validateTransition @4.4.0
Done when: AVAILABLE(0), ACQUIRED(1), ACKNOWLEDGED(2), ARCHIVING(3) and ARCHIVED(4) have Kafka's ids; self-transitions are rejected, ACKNOWLEDGED and ARCHIVED are terminal and AVAILABLE may only become ACQUIRED; unit test per pair.

### [To Do] SHARE-0086 In-flight state with delivery counts and member ownership
P0 · M8 · Task · deps: SHARE-0085 · ref: server/src/main/java/org/apache/kafka/server/share/fetch/InFlightState.java tryUpdateState @4.4.0
Done when: an in-flight state holds state, delivery count, owning member and lock deadline; INCREASE, DECREASE and NO_OP delivery-count operations apply; moving to AVAILABLE at or beyond the maximum delivery count archives (ARCHIVING when a DLQ is enabled); invalid transitions return no update and are logged; unit tests.

### [To Do] SHARE-0087 Two-phase state transitions pending persistence
P0 · M8 · Task · deps: SHARE-0086 · ref: server/src/main/java/org/apache/kafka/server/share/fetch/InFlightState.java completeStateTransition @4.4.0
Done when: a transition can be started (keeping a rollback state) and later committed or rolled back; a pending transition blocks other updates; rollback after an expired lock yields AVAILABLE or ARCHIVED at the delivery limit; delivery counts are never reverted; tests per path.

### [To Do] SHARE-0088 Batch-level in-flight state with lazy per-offset split
P0 · M8 · Task · deps: SHARE-0087 · ref: server/src/main/java/org/apache/kafka/server/share/fetch/InFlightBatch.java @4.4.0
Done when: a batch keeps one state for its whole range until a partial acquire, acknowledgement or timeout splits it into per-offset states; whole-batch archive works; splitting preserves states and deadlines; tests.

### [To Do] SHARE-0089 Ordered in-flight cache with start and end offsets
P0 · M8 · Task · deps: SHARE-0088
Done when: batches are kept in a `BTreeMap` by first offset with floor and range lookups, the start offset and end offset are tracked, the number of in-flight records is bounded by a configured maximum, and approximate memory is reported; unit tests.

### [To Do] SHARE-0090 Advance the start offset over completed records
P0 · M8 · Task · deps: SHARE-0089 · ref: core/src/main/java/kafka/server/share/SharePartition.java maybeUpdateCachedStateAndOffsets @4.4.0
Done when: when the leading records are ACKNOWLEDGED or ARCHIVED the start offset moves past them, fully completed batches are evicted, and the delivery-complete count (KIP-1226) is maintained; tests.

### [To Do] SHARE-0091 Gaps in the in-flight cache
P0 · M8 · Task · deps: SHARE-0089 · ref: core/src/main/java/kafka/server/share/SharePartition.java maybeUpdatePersisterGapWindowStartOffset @4.4.0
Done when: offsets absent from the cache (gaps from persisted state or compaction) are tracked as a gap window, acknowledgements of type GAP(0) are accepted for them, and the window shrinks as acquisition passes it; tests.

### [To Do] SHARE-0092 Acquisition lock deadlines
P0 · M8 · Task · deps: SHARE-0087, SRV-F07 · ref: server/src/main/java/org/apache/kafka/server/share/fetch/AcquisitionLockTimerTask.java @4.4.0
Done when: each acquired batch or offset has a lock deadline scheduled on the timer wheel as an action; expiry returns the records to AVAILABLE keeping their delivery count (or archives them at the limit); renewal replaces the deadline; tests with the simulated clock.

### [To Do] SHARE-0093 Conversion between in-flight state and persisted state batches
P0 · M8 · Task · deps: SHARE-0089 · ref: server-common/src/main/java/org/apache/kafka/server/share/persister/PersisterStateBatch.java @4.4.0
Done when: changed records become persisted state batches (first offset, last offset, delivery state, delivery count) and persisted batches rebuild an equivalent cache; round-trip property test.

### [To Do] SHARE-0094 Property test of the delivery state store
P0 · M8 · Task · deps: SHARE-0090, SHARE-0092, SHARE-0091 · labels: test
Done when: random acquire, acknowledge, release, reject, timeout and rollback sequences keep: no record acquired by two members, delivery count <= limit, terminal states never change, start offset never decreases; regressions persisted.

### [To Do] SHARE-0095 Benchmark the delivery state store
P2 · M8 · Task · deps: SHARE-0089, TEST-F11 · labels: perf
Done when: criterion benchmarks report acquire and acknowledge throughput with 1 million in-flight records and bytes per in-flight record, with budgets recorded in docs/nfr.md.

### [To Do] SHARE-0096 Document the delivery state store for share groups and future queues
P1 · M8 · Task · deps: SHARE-0090 · labels: docs, foundation-for:X-QUEUE
Done when: module docs describe states, transitions, two-phase persistence and memory bounds, and list the extension points reserved for X-QUEUE.

## [To Do] SHARE-F09 Share group testing
P0 · M8 · Feature · deps: SHARE-F03, SHARE-F04, TEST-F06
Scope: Simulation (at-least-once, bounded redelivery) and client compatibility.

### [To Do] SHARE-0097 Simulated share consumers
P0 · M8 · Task · deps: none · labels: test
Done when: testkit share consumers heartbeat, fetch, acknowledge (accept, release, reject, renew) and crash as deterministic state machines.

### [To Do] SHARE-0098 Share invariant checkers
P0 · M8 · Task · deps: SHARE-0097, TEST-F05 · labels: test
Done when: after each step the checkers assert no record is acquired by two members at once, delivery counts never exceed the limit, in-flight records stay within the limit, and every record below the share start offset was acknowledged or archived.

### [To Do] SHARE-0099 At-least-once simulation under faults
P0 · M8 · Task · deps: SHARE-0098 · labels: test
Done when: fault plans crash consumers, partition leaders and share coordinators and partition the network; every produced record is eventually acknowledged or archived, and redeliveries stay bounded by the delivery limit, over the CI seed set.

### [To Do] SHARE-0100 Share consumer compatibility tests
P0 · M8 · Task · deps: TEST-F08 · labels: test
Done when: the Java 4.4.0 `KafkaShareConsumer` in implicit and explicit acknowledgement modes, with renew and both acquire modes, consumes correctly against Alluvium.

### [To Do] SHARE-0101 Differential tests of share group APIs against Kafka
P0 · M8 · Task · deps: TEST-F09 · labels: test
Done when: scripted ShareGroupHeartbeat, ShareFetch, ShareAcknowledge and share admin sequences produce identical error codes and record ranges on Kafka 4.4.0 and Alluvium.

### [To Do] SHARE-0102 Seed management for share simulations
P1 · M8 · Task · deps: SHARE-0099 · labels: test
Done when: share simulations run a fixed seed set plus fresh seeds in CI and thousands nightly, and failing seeds are added to the fixed set.

## [To Do] SHARE-F10 Share metrics and events
P1 · M8 · Feature · deps: SHARE-F02, OBS-F04
Scope: Acquisition, redelivery and lock-expiry metrics and events.

### [To Do] SHARE-0103 Share partition metrics
P1 · M8 · Task · deps: SHARE-F08 · ref: server/src/main/java/org/apache/kafka/server/share/metrics/SharePartitionMetrics.java @4.4.0
Done when: equivalents of InFlightMessageCount, InFlightBatchCount, InFlightBatchMessageCount, FetchLockTimeMs and FetchLockRatio are exported per topic (partition labels only with `alluvium.metrics.per.partition.enabled`) with docs/metrics.md rows.

### [To Do] SHARE-0104 Share group broker metrics
P1 · M8 · Task · deps: none · ref: server/src/main/java/org/apache/kafka/server/share/metrics/ShareGroupMetrics.java @4.4.0
Done when: equivalents of RecordAcknowledgementsPerSec by ack type, AcquisitionLockTimeoutPerSec, PartitionLoadTimeMs, TopicPartitionsAcquireTimeMs, RequestTopicPartitionsFetchRatio and the DeadLetterQueue record and produce request metrics are exported with docs/metrics.md rows.

### [To Do] SHARE-0105 Share coordinator metrics
P1 · M8 · Task · deps: none · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/metrics/ShareCoordinatorMetrics.java @4.4.0
Done when: equivalents of write-rate, write-total, write-latency-avg/max and last-pruned-offset, plus share-group-count and share-group-rebalance-rate/count from the group coordinator, are exported with docs/metrics.md rows.

### [To Do] SHARE-0106 Share events with reasons
P1 · M8 · Task · deps: OBS-F07
Done when: `share_partition_state_change` (reason: initialized, fenced, leader_lost, failed), `share_lock_expired`, `share_records_archived` (reason: delivery_limit, client_reject, log_start_moved, aborted_txn) and `share_member_*` events are emitted by drivers, never per record at info level, and listed in `events.rs`.

### [To Do] SHARE-0107 Share partition flight recorders
P1 · M8 · Task · deps: SHARE-0106, OBS-F06
Done when: each share partition keeps a bounded flight recorder of state transitions and lock expiries, dumped on errors and shown by the diagnostics endpoint for the group.

## [To Do] SHARE-F11 Share group dead-letter queues (KIP-1191)
P1 · M8 · Feature · deps: SHARE-F02 · labels: foundation-for:X-QUEUE
Scope: Dead-letter queue handling for share groups as in Kafka 4.4.0 (KIP-1191): ARCHIVING state, DLQ record writes and `errors.deadletterqueue.*` group configs.

### [To Do] SHARE-0030 Share dead-letter queue enablement (KIP-1191)
P0 · M8 · Task · deps: SHARE-0017 · ref: core/src/main/java/kafka/server/share/SharePartition.java isDLQEnabledForGroup @4.4.0
Done when: DLQ handling is active only when the finalized `share.version` is at least 2 and the group config `errors.deadletterqueue.topic.name` is set; the topic name must start with `errors.deadletterqueue.topic.name.prefix` and the topic must have `errors.deadletterqueue.group.enable=true`, auto-created when `errors.deadletterqueue.auto.create.topics.enable` is true; tests per rule.

### [To Do] SHARE-0031 ARCHIVING records through the dead-letter queue
P0 · M8 · Task · deps: SHARE-0030, SHARE-0029 · ref: core/src/main/java/kafka/server/share/SharePartition.java initiateDLQAndArchive @4.4.0
Done when: a REJECT or reaching the delivery limit with DLQ enabled moves records to ARCHIVING, enqueues a DLQ write, then moves them to ARCHIVED and persists whether or not the DLQ write succeeded (failure logged at error); ARCHIVING batches found at initialization resume with cause `delivery_count_exceeded` or `client_reject`; tests.

### [To Do] SHARE-0032 Write dead-letter records
P0 · M8 · Task · deps: SHARE-0031, NET-F10 · ref: server/src/main/java/org/apache/kafka/server/share/dlq/ShareGroupDLQStateManager.java @4.4.0
Done when: DLQ records copy the source record when `errors.deadletterqueue.copy.record.enable` is true (else carry headers only) and carry `__dlq.errors.topic`, `__dlq.errors.partition`, `__dlq.errors.offset`, `__dlq.errors.group`, `__dlq.errors.delivery.count` and `__dlq.errors.message` headers; large ranges are sent in rounds of at most 1 MiB; produce retries up to 5 attempts with exponential backoff capped at 30 s; tests with a fake produce path.

### [To Do] SHARE-0108 Share DLQ metrics and events
P1 · M8 · Task · deps: SHARE-0032, OBS-F04
Done when: DLQ write count, failures and latency are exported per group with bounded labels; each DLQ write emits a `share_record_dead_lettered` event with group, topic, partition, offset and reason; tests assert both.

### [To Do] SHARE-0109 Share DLQ compatibility test with the Java client
P1 · M8 · Task · deps: SHARE-0032, TEST-F08 · labels: test
Done when: a Java 4.4.0 share consumer that rejects records past the delivery limit sees them in the configured DLQ topic with the KIP-1191 headers, matching Kafka.

### [To Do] SHARE-0110 Share DLQ differential test against Kafka 4.4.0
P1 · M8 · Task · deps: SHARE-0032, TEST-F09 · labels: test
Done when: the same reject and delivery-limit scenario produces identical DLQ topic contents (ignoring timestamps) and group state on Kafka and Alluvium.
