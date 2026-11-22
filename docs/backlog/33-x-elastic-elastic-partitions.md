# X-ELASTIC: Elastic partitions

> Crates: `alluvium-controller`, `alluvium-metadata`, `alluvium-replication` · Kafka counterpart: none (Alluvium enhancement) · Milestones: M12

Partitions that split and merge without breaking per-key ordering, so partition counts can grow and shrink with load.

## [To Do] X-ELASTIC-F01 Elastic partition design
P0 · M12 · Feature · deps: META-F08
Scope: Spike and ADR: key-range model, lineage, Kafka-client compatibility.

### [To Do] X-ELASTIC-0001 Elastic partitions ADR: key-range model, mapping epochs, lineage, seal protocol and Kafka-client compatibility modes
P0 · M12 · Spike · deps: none
Done when: docs/adr/NNNN-elastic-partitions.md is accepted and decides: the hash function and hash space (murmur2-compatible KeyHash from TYPE-F11), the range-to-partition mapping with mapping epochs, the lineage DAG, the seal protocol (seal request, leader drain, seal report, atomic commit), the offset-continuity rule (children start at the parent's seal offset; a merged child starts at the max parent seal offset), the produce redirect errors (native and Kafka), lineage-aware consumption, the virtual partition view, the off-by-default gating, and the rejected alternatives (Kafka CreatePartitions, copy-based resharding, per-key forwarding only).

### [To Do] X-ELASTIC-0002 Elastic threat model: split/merge abuse, lineage-depth exhaustion and redirect loops
P0 · M12 · Task · deps: X-ELASTIC-0001 · labels: security, docs
Done when: The threat model in docs/standards/security.md gains rows for split/merge storms (bounded by controller mutation quotas and lineage-depth and partition caps), unauthorized resize (ALTER on TOPIC required), redirect loops between parent and children (bounded retries in the SDK), and metadata growth from lineage records; each row names the task that enforces the defense.

### [To Do] X-ELASTIC-0003 Glossary entries for key range, key hash, mapping epoch, lineage, sealed partition and seal offset
P1 · M12 · Task · deps: X-ELASTIC-0001 · labels: docs
Done when: docs/glossary.md defines key hash, key range, mapping epoch, lineage, parent and child partition, sealed partition, seal offset, split, merge and virtual partition, each linking to the ADR; no other doc defines these terms differently (link check passes).

## [To Do] X-ELASTIC-F02 Key-range mapping
P0 · M12 · Feature · deps: X-ELASTIC-F01
Scope: Ranges over the hash space mapped to physical partitions, with epochs.

### [To Do] X-ELASTIC-0004 `KeyRangeMap` type: ordered, gap-free, non-overlapping ranges over the key-hash space mapped to partition indexes
P0 · M12 · Task · deps: none
Done when: `alluvium-metadata::elastic::KeyRangeMap` built from `KeyRange`s (TYPE-F11) rejects gaps and overlaps with a typed `ElasticMappingError::{Gap, Overlap, EmptyRange, UnknownPartition}` carrying the offending ranges; lookup by `KeyHash` is O(log n); property test: lookup equals a linear scan for arbitrary maps of up to 4096 ranges.

### [To Do] X-ELASTIC-0005 `MappingEpoch` newtype and epoch rules: strictly increasing per topic, bumped by every committed split or merge
P0 · M12 · Task · deps: X-ELASTIC-0004
Done when: `MappingEpoch` is a checked newtype; the map exposes its epoch; applying a delta whose epoch is not exactly current+1 returns `ElasticMappingError::StaleEpoch{current, received}`; unit tests cover equal, lower and skipped epochs.

### [To Do] X-ELASTIC-0006 Key routing function: record key to active partition through the mapping, with the null-key policy from the ADR
P0 · M12 · Task · deps: X-ELASTIC-0004 · ref: clients/src/main/java/org/apache/kafka/common/utils/Utils.java murmur2 @4.4.0
Done when: `route(key: Option<&[u8]>, map)` returns the owning partition for keyed records using the Kafka-compatible murmur2 hash, and for null keys returns the ADR's policy (sticky choice among active partitions, passed in by the caller); test vectors match Kafka's `Utils.murmur2` outputs for 100 golden keys.

### [To Do] X-ELASTIC-0007 Lineage model: DAG of partitions with key range, state (Active, Sealing, Sealed, Deleted), parents, children, start offset and seal offset
P0 · M12 · Task · deps: X-ELASTIC-0004
Done when: `Lineage` enforces: acyclic; split children exactly partition the parent's range; a merge child's range is the union of adjacent parents' ranges; child start offset >= every parent's seal offset; depth <= the configured cap; property tests over random split/merge sequences keep every invariant and reject each violation with a typed error.

### [To Do] X-ELASTIC-0008 Metadata record `PartitionSealRecord`: topic id, partition, seal state, seal offset, leader epoch at seal and operation id
P0 · M12 · Task · deps: X-ELASTIC-0007 · labels: decoder · ref: metadata/src/main/resources/common/metadata/PartitionRecord.json @4.4.0
Done when: Schema added to the Alluvium metadata schema set with an Alluvium record type id from the META-F08 range; generated codec round-trips; replay sets the partition's seal state in the delta; replay of a seal for an unknown partition or a non-monotonic state change is rejected with a typed error; the generated metadata fuzz target covers it.

### [To Do] X-ELASTIC-0009 Metadata record `PartitionLineageRecord`: child partition, parent partitions, key range and child start offset
P0 · M12 · Task · deps: X-ELASTIC-0007 · labels: decoder
Done when: Schema, codec round-trip and replay into the elastic part of the topics delta; replay validates lineage invariants and rejects violations with `ElasticMappingError`; covered by the generated metadata fuzz target.

### [To Do] X-ELASTIC-0010 Metadata record `ElasticOperationRecord`: operation id, kind (split or merge), state (Sealing, Committed, Aborted), participants, deadline and abort reason
P0 · M12 · Task · deps: X-ELASTIC-0007 · labels: decoder
Done when: Schema, codec round-trip and replay; at most one non-terminal operation per topic is accepted on replay (second one rejected with a typed error); covered by the generated metadata fuzz target; abort reasons are a closed enum shared with events.

### [To Do] X-ELASTIC-0011 `ElasticTopicImage` in `MetadataImage`: current mapping, lineage, seal states and in-flight operation, built from deltas
P0 · M12 · Task · deps: X-ELASTIC-0005, X-ELASTIC-0008, X-ELASTIC-0009, X-ELASTIC-0010 · ref: metadata/src/main/java/org/apache/kafka/image/TopicsImage.java @4.4.0
Done when: Applying the META-F08 mapping records plus the three elastic records yields an immutable per-topic image; replaying any prefix of a valid record stream yields a valid image; image equality after delta application matches a from-scratch rebuild (property test).

### [To Do] X-ELASTIC-0012 Elastic image in metadata snapshots: write and reload mapping, lineage, seal states and in-flight operations
P0 · M12 · Task · deps: X-ELASTIC-0011
Done when: Snapshot writer emits the records needed to rebuild `ElasticTopicImage`; round-trip test: image -> snapshot records -> image is equal for 1000 generated lineages, including an in-flight operation.

### [To Do] X-ELASTIC-0013 Feature level `alluvium.elastic.version`: gate elastic records and behaviors, block downgrade while elastic topics exist
P0 · M12 · Task · deps: X-ELASTIC-0011 · ref: server-common/src/main/java/org/apache/kafka/server/common/MetadataVersion.java @4.4.0
Done when: With the level at 0 the controller never emits elastic records and rejects elastic topic configs with INVALID_CONFIG; UpdateFeatures downgrading below 1 while any elastic topic exists fails with INVALID_UPDATE_VERSION and a message naming the topics; tests cover upgrade, downgrade and replay gating.

### [To Do] X-ELASTIC-0014 Broker-side mapping publication: per-topic `KeyRangeMap` from each image version, read lock-free by the produce path
P0 · M12 · Task · deps: X-ELASTIC-0011, X-ELASTIC-0006 · labels: perf
Done when: Brokers publish the mapping with the image (one version per request, per architecture invariant); a criterion benchmark shows lookup under 100 ns for 1024 ranges; a test asserts a request never observes two mapping epochs.

## [To Do] X-ELASTIC-F03 Split
P0 · M12 · Feature · deps: X-ELASTIC-F02
Scope: Controller workflow: seal parent, create children, update mapping atomically.

### [To Do] X-ELASTIC-0015 Split operation state machine in `alluvium-controller`: Validating, Sealing, Committing, Committed, Aborted
P0 · M12 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/PartitionChangeBuilder.java @4.4.0
Done when: A sans-I/O `ElasticOperation` machine takes (request, seal reports, timer ticks, image changes) and returns record batches and timer actions; one non-terminal operation per topic; unit tests cover every transition and every illegal input returning a typed error.

### [To Do] X-ELASTIC-0016 Split request validation with typed errors
P0 · M12 · Task · deps: X-ELASTIC-0015
Done when: Rejects with a typed `SplitError` (mapped once to native and Kafka error codes): topic not elastic, partition not Active, unknown partition, range too narrow to split, lineage depth at `alluvium.elastic.max.lineage.depth`, partition count at `alluvium.elastic.max.partitions`, operation already in flight, reassignment in progress, partition offline or under min ISR; one test per error.

### [To Do] X-ELASTIC-0017 Split point selection: explicit key hash or range midpoint, both halves non-empty
P0 · M12 · Task · deps: X-ELASTIC-0015
Done when: An explicit split hash outside the parent range or on its boundary is rejected; no split hash picks the midpoint; property test: the two child ranges always partition the parent exactly.

### [To Do] X-ELASTIC-0018 Seal request: controller writes `ElasticOperationRecord(Sealing)` and `PartitionSealRecord(Requested)` with a partition-epoch bump
P0 · M12 · Task · deps: X-ELASTIC-0016, X-ELASTIC-0017 · ref: metadata/src/main/java/org/apache/kafka/controller/PartitionChangeBuilder.java @4.4.0
Done when: On a valid split the controller emits one atomic batch with the operation record, the seal request and a partition epoch bump that fences stale AlterPartition requests; replay test shows the leader observes Sealing in the same image version as the epoch bump.

### [To Do] X-ELASTIC-0019 Leader sealing in `alluvium-replication`: stop new appends, drain in-flight appends, wait for HW == LEO and LSO == HW
P0 · M12 · Task · deps: X-ELASTIC-0018, REPL-F03 · ref: core/src/main/scala/kafka/cluster/Partition.scala appendRecordsToLeader @4.4.0
Done when: A partition in Sealing rejects new appends with the redirect error, lets already-appended batches complete or time out, and emits `SealReady{seal_offset, leader_epoch}` once HW == LEO and LSO == HW; unit tests cover in-flight acks=all appends, an open transaction holding LSO, and ISR shrink during sealing.

### [To Do] X-ELASTIC-0020 Seal report RPC: Alluvium-internal controller API carrying topic id, partition, leader epoch, partition epoch and seal offset
P0 · M12 · Task · deps: X-ELASTIC-0018 · labels: decoder, security
Done when: Request/response schemas in the Alluvium API key range, generated codecs with round-trip tests and a fuzz target, served only on the controller listener with CLUSTER_ACTION authorization, never listed in ApiVersions on client listeners (test asserts absence).

### [To Do] X-ELASTIC-0021 Broker driver sends the seal report and retries it with backoff until acknowledged or fenced
P0 · M12 · Task · deps: X-ELASTIC-0019, X-ELASTIC-0020
Done when: The replica-manager driver sends the report on `SealReady`, retries on timeouts and NOT_CONTROLLER with bounded backoff, and stops on FENCED_LEADER_EPOCH; a test with a simulated controller verifies exactly one accepted report per seal.

### [To Do] X-ELASTIC-0022 Split commit: one atomic batch with the sealed parent, child partition records, lineage records, new mapping epoch and Committed state
P0 · M12 · Task · deps: X-ELASTIC-0021 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createPartitions @4.4.0
Done when: On a report whose leader and partition epochs are current, the controller writes `PartitionSealRecord(Sealed, seal_offset)`, two `PartitionRecord`s with start offset = seal offset, two `PartitionLineageRecord`s, the mapping records for epoch E+1 and `ElasticOperationRecord(Committed)` in one batch; replay test: no image version ever shows a partial split.

### [To Do] X-ELASTIC-0023 Seal report fencing: reports with a stale leader epoch, partition epoch or offset below the known HW are rejected
P0 · M12 · Task · deps: X-ELASTIC-0022
Done when: Stale reports return FENCED_LEADER_EPOCH or INVALID_REQUEST with a message, never commit; tests cover a report from a deposed leader and a duplicate report after commit (idempotent success).

### [To Do] X-ELASTIC-0024 Child replica placement: reuse the parent's replica set when it satisfies rack constraints, else the striped placer
P0 · M12 · Task · deps: X-ELASTIC-0022, CTRL-F07 · ref: metadata/src/main/java/org/apache/kafka/metadata/placement/StripedReplicaPlacer.java @4.4.0
Done when: Children get replicas through CTRL-F07's placer with a preference for the parent's replicas (no data movement is needed because children start empty); tests cover rack-aware placement, fenced parent replicas and the replication factor of the topic.

### [To Do] X-ELASTIC-0025 Child log creation on brokers at a non-zero log start offset equal to the seal offset
P0 · M12 · Task · deps: X-ELASTIC-0022, REPL-F12 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java @4.4.0
Done when: The replica manager creates child logs with log start offset, LEO, HW and recovery point all equal to the child start offset and leader epoch 0, checkpointed before serving; a restart test reloads them unchanged; fetches below the start offset return OFFSET_OUT_OF_RANGE.

### [To Do] X-ELASTIC-0026 Seal timeout and abort: `alluvium.elastic.seal.timeout.ms` expiry writes Aborted and unseals the parent
P0 · M12 · Task · deps: X-ELASTIC-0022
Done when: If no valid seal report arrives before the deadline, the controller writes `ElasticOperationRecord(Aborted, reason=seal_timeout)` and `PartitionSealRecord(Active)`; the leader resumes appends; produce availability returns within one metadata propagation; unit and replay tests.

### [To Do] X-ELASTIC-0027 Controller failover mid-split: the new active controller resumes or aborts the in-flight operation from the image
P0 · M12 · Task · deps: X-ELASTIC-0026 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java @4.4.0
Done when: On activation the controller rebuilds operation timers from `ElasticOperationRecord` deadlines (injected clock) and resumes waiting for the seal report or aborts on expiry; replay tests cut the record stream after each split record and assert every run ends Committed or Aborted with a consistent image.

### [To Do] X-ELASTIC-0028 Leader failover mid-seal: a new parent leader inherits Sealing and reports with its own epoch
P0 · M12 · Task · deps: X-ELASTIC-0023
Done when: A new leader elected while Sealing never accepts appends, truncates per normal epoch reconciliation, then reports its seal offset; the old leader's in-flight report is fenced; unit tests on the replication machine plus a controller replay test.

### [To Do] X-ELASTIC-0029 Transactions and idempotent producers at seal: wait for open transactions, reject AddPartitionsToTxn on a sealing partition
P0 · M12 · Task · deps: X-ELASTIC-0019, TXN-F11 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java @4.4.0
Done when: AddPartitionsToTxn and transactional produce to a Sealing or Sealed partition return the redirect error from the ADR; a split whose parent still has an open transaction at the seal deadline aborts with reason `open_transaction` (the transaction is not aborted); producer ids start fresh on children (sequence 0) as for any new partition; tests per case.

### [To Do] X-ELASTIC-0030 Split conflicts: reassignment, topic deletion and CreatePartitions on elastic topics
P0 · M12 · Task · deps: X-ELASTIC-0015 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createPartitions @4.4.0
Done when: Split during reassignment is rejected; topic deletion during a split aborts it with reason `topic_deleted`; CreatePartitions on an elastic topic returns INVALID_PARTITIONS with a message pointing to split; tests per case.

### [To Do] X-ELASTIC-0031 Split events and flight-recorder entries with reasons
P0 · M12 · Task · deps: X-ELASTIC-0022, X-ELASTIC-0026 · labels: observability
Done when: Controller emits `elastic_split_started`, `elastic_split_committed`, `elastic_split_aborted` and the partition driver emits `partition_seal_requested` and `partition_sealed`, each with `reason` enums and mapping epoch, seal offset and operation id fields; the controller and partition flight recorders hold the transitions; tests assert the events from a recorded run.

### [To Do] X-ELASTIC-0032 Register split keys `alluvium.elastic.seal.timeout.ms`, `alluvium.elastic.max.lineage.depth`, `alluvium.elastic.max.partitions`
P0 · M12 · Task · deps: X-ELASTIC-0016, CFG-F11 · labels: config
Done when: The keys are defined in the CFG-F11 `alluvium.*` registry with type, default, validator, dynamic mode and doc linking the ADR, are mapped onto the controller's plain config struct, and appear in generated docs/config.md; invalid values fail `check-config`.

## [To Do] X-ELASTIC-F04 Merge
P0 · M12 · Feature · deps: X-ELASTIC-F02
Scope: Merge adjacent ranges to reduce partition count.

### [To Do] X-ELASTIC-0033 Merge eligibility: two Active partitions with adjacent ranges in the current mapping
P0 · M12 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createPartitions @4.4.0
Done when: Validation returns a typed `MergeError` for non-adjacent ranges, a non-Active participant, an in-flight operation, lineage depth or `alluvium.elastic.min.partitions` floor reached, and reassignment in progress; one test per error.

### [To Do] X-ELASTIC-0034 Merge in the operation state machine: seal both parents in parallel and wait for both seal reports
P0 · M12 · Task · deps: X-ELASTIC-0033, X-ELASTIC-0015, X-ELASTIC-0022, X-ELASTIC-0026
Done when: The operation machine handles two participants: both get `PartitionSealRecord(Requested)` in one batch, both reports are required, a report from either alone never commits; unit tests cover report order and duplicates.

### [To Do] X-ELASTIC-0035 Merge commit: child start offset = max of the parents' seal offsets, one atomic batch
P0 · M12 · Task · deps: X-ELASTIC-0034
Done when: The commit batch seals both parents, creates one child at max(seal offsets), writes its lineage record with both parents and the mapping for epoch E+1; property test: the child start offset is >= every offset in either parent.

### [To Do] X-ELASTIC-0036 Partial seal failure in a merge: one parent sealed, the other timed out or fenced, aborts and unseals both
P0 · M12 · Task · deps: X-ELASTIC-0035
Done when: A merge whose second report misses `alluvium.elastic.seal.timeout.ms` writes Aborted with reason `participant_seal_timeout` and returns both parents to Active; no image shows a sealed parent without a child; replay test.

### [To Do] X-ELASTIC-0037 Merged child placement: prefer the replica set of the parent with more bytes, rack constraints preserved
P0 · M12 · Task · deps: X-ELASTIC-0035 · ref: metadata/src/main/java/org/apache/kafka/metadata/placement/StripedReplicaPlacer.java @4.4.0
Done when: Placement picks the larger parent's replica set when valid, otherwise the placer; tests for rack-aware and fenced-replica cases.

### [To Do] X-ELASTIC-0038 Controller and leader failover mid-merge
P0 · M12 · Task · deps: X-ELASTIC-0036
Done when: Replay tests cut the stream after each merge record and after each single seal report; every run ends Committed or Aborted with consistent lineage and mapping.

### [To Do] X-ELASTIC-0039 Transactions at merge: open transactions on either parent block the merge until the deadline
P0 · M12 · Task · deps: X-ELASTIC-0034
Done when: A merge with an open transaction on one parent aborts with `open_transaction` at the deadline; transactional produce to either sealing parent returns the redirect error; tests.

### [To Do] X-ELASTIC-0040 Property test: arbitrary split/merge sequences keep full coverage, offset continuity and depth bounds
P0 · M12 · Task · deps: X-ELASTIC-0035, X-ELASTIC-0036 · labels: test
Done when: A proptest drives random split and merge requests with random seal reports and aborts through the controller machine; after every step the image has gap-free coverage, children start at or above parents' seal offsets and depth <= cap; failing cases persist to proptest-regressions.

### [To Do] X-ELASTIC-0041 Merge events and flight-recorder entries
P0 · M12 · Task · deps: X-ELASTIC-0035 · labels: observability
Done when: `elastic_merge_started`, `elastic_merge_committed` and `elastic_merge_aborted` carry operation id, parents, child, mapping epoch and reason; recorder entries present; tests on a recorded run.

### [To Do] X-ELASTIC-0042 Register `alluvium.elastic.min.partitions` topic key used as the merge floor
P0 · M12 · Task · deps: X-ELASTIC-0033, CFG-F11 · labels: config
Done when: Topic-level key in the CFG-F11 registry with validator (1 <= min <= max), default 1, doc linking the ADR; merge below the floor is rejected; config docs generated.

## [To Do] X-ELASTIC-F05 Sealed partitions
P0 · M12 · Feature · deps: X-ELASTIC-F03
Scope: Read-only semantics, retention and deletion of sealed partitions.

### [To Do] X-ELASTIC-0043 Sealed partitions are read-only: produce rejected with the redirect error, fetch served up to the seal offset
P0 · M12 · Task · deps: none · ref: core/src/main/scala/kafka/cluster/Partition.scala appendRecordsToLeader @4.4.0
Done when: On a Sealed partition produce returns the ADR redirect error with the current mapping epoch, fetch returns data up to the seal offset, HW = LEO = seal offset never moves; unit tests on the replication machine.

### [To Do] X-ELASTIC-0044 Offset queries on sealed partitions: ListOffsets latest and max-timestamp at the seal, OffsetForLeaderEpoch end at the seal
P0 · M12 · Task · deps: X-ELASTIC-0043 · ref: clients/src/main/java/org/apache/kafka/clients/admin/internals/ListOffsetsHandler.java @4.4.0
Done when: ListOffsets LATEST returns the seal offset and MAX_TIMESTAMP the last batch before the seal; OffsetForLeaderEpoch for the final epoch returns the seal offset; tests per API version using the API-F02 and API-F23 handlers.

### [To Do] X-ELASTIC-0045 Seal finality invariant: no replica of a sealed partition ever truncates below or appends beyond the seal offset
P0 · M12 · Task · deps: X-ELASTIC-0043
Done when: A `debug_assert!` in the replication machine and storage append path enforces it; unclean leader election on a sealed partition is refused unless the candidate's LEO equals the seal offset (reason `sealed_partition_unclean_refused` event); tests.

### [To Do] X-ELASTIC-0046 Leader election and reassignment continue for sealed partitions to keep them readable
P0 · M12 · Task · deps: X-ELASTIC-0045
Done when: Sealed partitions take part in preferred, ELR-based and reassignment flows like other partitions; followers that join must reach the seal offset to enter the ISR; controller tests.

### [To Do] X-ELASTIC-0047 Retention of sealed partitions: time and size retention apply as for any partition
P0 · M12 · Task · deps: X-ELASTIC-0043, STOR-F13
Done when: Retention advances the log start offset of sealed partitions under the topic's `retention.ms` and `retention.bytes`; when the log start offset reaches the seal offset the partition is marked drained in the broker's report; tests with the simulated clock.

### [To Do] X-ELASTIC-0048 Deletion of drained sealed partitions: `RemoveElasticPartitionRecord` and broker-side log deletion
P0 · M12 · Task · deps: X-ELASTIC-0047 · labels: decoder
Done when: When a sealed partition is empty (log start offset == seal offset on the leader) and all its children are past their own start, the controller writes `RemoveElasticPartitionRecord`; brokers delete the logs asynchronously; lineage keeps the node as Deleted so offsets stay resolvable; the partition index is never reused; record has codec, replay and fuzz coverage.

### [To Do] X-ELASTIC-0049 Compacted elastic topics: sealed partitions are never auto-deleted
P0 · M12 · Task · deps: X-ELASTIC-0048
Done when: With `cleanup.policy` containing `compact`, sealed partitions are retained and keep being cleaned; a metric `alluvium_elastic_sealed_retained_bytes{topic}` reports their size; the limitation is documented in the elastic guide; test asserts no deletion record is written.

### [To Do] X-ELASTIC-0050 Broker restart with sealed partitions: sealed state applied from the image before the partition serves
P0 · M12 · Task · deps: X-ELASTIC-0043
Done when: After restart a sealed partition rejects produce from the first request (readiness waits for the image as per OBS-F09); a crash-restart test proves no append lands after the seal.

### [To Do] X-ELASTIC-0051 Sealed partition events: `sealed_partition_drained`, `sealed_partition_deleted` with reasons
P0 · M12 · Task · deps: X-ELASTIC-0048 · labels: observability
Done when: Events carry topic, partition, seal offset and reason (`retention_time`, `retention_size`, `delete_records`); flight-recorder entries present; tests.

## [To Do] X-ELASTIC-F06 SDK producer routing
P0 · M12 · Feature · deps: X-ELASTIC-F02, X-NATIVE-F12
Scope: Key-range routing in the native SDK from metadata.

### [To Do] X-ELASTIC-0052 Native IDL: mapping epoch in produce requests, `PartitionSealed` and `StaleMappingEpoch` error details carrying the new mapping
P0 · M12 · Task · deps: X-NATIVE-F02
Done when: X-NATIVE-F02 IDL gains the fields and error details; the IDL breaking-change check passes; generated docs list them.

### [To Do] X-ELASTIC-0053 Broker validation of native produce mapping epoch and key ranges
P0 · M12 · Task · deps: X-ELASTIC-0052
Done when: A native produce with an older mapping epoch returns `StaleMappingEpoch` with the current mapping; records whose key hash is outside the partition's range return `KeyOutOfRange`; nothing is appended on either error (test).

### [To Do] X-ELASTIC-0054 SDK mapping metadata: fetch, cache and refresh per-topic mapping and lineage
P0 · M12 · Task · deps: X-ELASTIC-0052
Done when: `alluvium-client` caches mapping epoch, ranges and leaders per elastic topic and refreshes on `PartitionSealed`, `StaleMappingEpoch` and on a bounded max age; unit tests with a fake transport.

### [To Do] X-ELASTIC-0055 SDK key-range partitioner with the ADR null-key policy
P0 · M12 · Task · deps: X-ELASTIC-0054 · ref: clients/src/main/java/org/apache/kafka/clients/producer/internals/BuiltInPartitioner.java partitionForKey @4.4.0
Done when: The SDK routes keyed records with the same hash as the broker (shared golden vectors) and null-key records per the ADR; property test agrees with the broker-side `route` on 100k random keys.

### [To Do] X-ELASTIC-0056 Order-preserving re-route of rejected batches after a split or merge
P0 · M12 · Task · deps: X-ELASTIC-0055
Done when: Batches rejected with `PartitionSealed` are re-partitioned by key under the new mapping and sent before any later batch for the same keys; property test with random splits during production: per-key order of acknowledged records equals send order.

### [To Do] X-ELASTIC-0057 Resolve unknown-outcome batches before re-routing: query the sealed parent's producer state
P0 · M12 · Task · deps: X-ELASTIC-0056 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java @4.4.0
Done when: For batches that timed out against a parent which then sealed, the SDK asks the parent for its last sequence for the producer id and drops already-appended batches before re-routing; test: no duplicate after a timeout plus split.

### [To Do] X-ELASTIC-0058 SDK metrics for mapping refreshes, re-routed batches and redirect errors
P1 · M12 · Task · deps: X-ELASTIC-0056 · labels: observability
Done when: SDK exposes counters for refreshes, re-routes and redirects by reason through its metrics interface; tests assert values after a scripted split.

### [To Do] X-ELASTIC-0059 SDK integration test: produce through continuous splits and merges with per-key order verification
P0 · M12 · Task · deps: X-ELASTIC-0057 · labels: test
Done when: An in-process cluster test produces 1M keyed records while splitting and merging; a lineage-aware reader confirms per-key order and no loss; runs in CI.

### [To Do] X-ELASTIC-0060 SDK documentation for elastic topics: routing, retries, ordering guarantees
P1 · M12 · Task · deps: X-ELASTIC-0056 · labels: docs
Done when: Rustdoc and the SDK guide describe mapping refresh, redirect handling and the per-key ordering guarantee with an example; doc tests compile.

## [To Do] X-ELASTIC-F07 Kafka-client key routing
P1 · M12 · Feature · deps: X-ELASTIC-F02
Scope: Opt-in broker-side re-routing of keyed records from Kafka clients.

### [To Do] X-ELASTIC-0061 Topic key `alluvium.elastic.kafka.routing` with values `reject` (default) and `reroute`
P1 · M12 · Task · deps: CFG-F11 · labels: config
Done when: Registered in CFG-F11 with validator and doc; `reroute` is rejected unless the cluster has elastic enabled; DescribeConfigs shows it with the correct source.

### [To Do] X-ELASTIC-0062 Default `reject` mode: Kafka produce to a sealed partition or with keys outside the partition's range
P1 · M12 · Task · deps: X-ELASTIC-0061 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest @4.4.0
Done when: Kafka produce to a sealed partition returns the ADR-chosen error code for every Produce version 3-13; keyed records outside the target partition's range are accepted unchanged in `reject` mode (Kafka partitioning semantics preserved) and counted in a metric; tests per version.

### [To Do] X-ELASTIC-0063 Verify Java 4.4.0, librdkafka and franz-go producer behavior for the chosen sealed-partition error
P1 · M12 · Task · deps: X-ELASTIC-0062, TEST-F08 · labels: test · ref: clients/src/main/java/org/apache/kafka/clients/producer/internals/Sender.java completeBatch @4.4.0
Done when: Compatibility tests show each client surfaces the error (no infinite retry, no silent loss) and the observed behavior is recorded in the client compatibility doc.

### [To Do] X-ELASTIC-0064 Reroute mode: split an incoming batch by key into per-owning-partition batches
P1 · M12 · Task · deps: X-ELASTIC-0061 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest @4.4.0
Done when: For `reroute` topics the handler hashes each record key, groups records by owning partition preserving order, and re-encodes batches (recompressing with the original codec); null-key records stay on the target partition if Active; unit tests with every compression type.

### [To Do] X-ELASTIC-0065 Reroute forwarding to remote leaders over the inter-node client with a bounded queue and timeouts
P1 · M12 · Task · deps: X-ELASTIC-0064, NET-F10
Done when: Sub-batches for partitions led elsewhere are forwarded with the original acks and the remaining request timeout; the forward queue is bounded by `alluvium.elastic.reroute.max.inflight.bytes` and full queues mute the client connection; tests with a simulated peer.

### [To Do] X-ELASTIC-0066 Reroute response semantics: complete only when every sub-batch completes, aggregate errors, offsets per the ADR
P1 · M12 · Task · deps: X-ELASTIC-0065
Done when: The produce response for the original partition is sent after all sub-batches finish; any sub-batch error yields the most severe error for the partition; base offset follows the ADR rule (verified acceptable to the three clients in compatibility tests).

### [To Do] X-ELASTIC-0067 Reroute with idempotent producers: deduplicate on the original target partition before forwarding
P1 · M12 · Task · deps: X-ELASTIC-0066 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java @4.4.0
Done when: The receiving leader keeps the producer id, epoch and sequence window for rerouted batches and returns the cached result for retries; tests show no duplicates when a client retries after a lost response.

### [To Do] X-ELASTIC-0068 Transactional produce to `reroute` topics is rejected with the ADR error
P1 · M12 · Task · deps: X-ELASTIC-0064
Done when: A transactional produce to a reroute topic returns the chosen error with a message; documented; test per Produce version.

### [To Do] X-ELASTIC-0069 Rerouted bytes charged to the original client's produce quota
P1 · M12 · Task · deps: X-ELASTIC-0065, QUOTA-F02
Done when: Quota accounting charges the client once for the original batch, not for forwarded sub-batches; inter-broker forwards are exempt from client quotas; test with QUOTA-F02 throttling.

### [To Do] X-ELASTIC-0070 Reroute metrics: `alluvium_elastic_rerouted_records_total{topic}`, forward latency histogram, queue depth and capacity
P1 · M12 · Task · deps: X-ELASTIC-0065 · labels: observability
Done when: Metrics registered with bounded labels and asserted in tests.

### [To Do] X-ELASTIC-0071 Compatibility test: Kafka clients produce keyed data through splits in reroute mode with per-key order
P1 · M12 · Task · deps: X-ELASTIC-0067, TEST-F08 · labels: test
Done when: Java, librdkafka and franz-go producers write keyed records while splits run; a lineage-aware reader confirms per-key order and no loss.

### [To Do] X-ELASTIC-0072 Benchmark reroute overhead versus native SDK routing
P2 · M12 · Task · deps: X-ELASTIC-0066, TEST-F11 · labels: perf
Done when: Throughput and p99 produce latency of reroute mode against direct routing are measured and recorded in the elastic guide; benchmark job tracks regressions.

## [To Do] X-ELASTIC-F08 Ordered consumption across splits
P0 · M12 · Feature · deps: X-ELASTIC-F05
Scope: Parent drained before children; lineage-aware group assignment.

### [To Do] X-ELASTIC-0073 Lineage readiness rule: a child is consumable by a group only when every parent's position reached its seal offset
P0 · M12 · Task · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/TargetAssignmentBuilder.java @4.4.0
Done when: Pure function `child_ready(lineage, group_positions)` handles split and merge parents, deleted parents (treated as drained) and parents emptied by retention; property tests over random lineages.

### [To Do] X-ELASTIC-0074 Consumer-protocol (KIP-848) assignment for elastic topics: hold children until parents are drained, keep parent and children on the same member
P0 · M12 · Task · deps: X-ELASTIC-0073, GRP-F06 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/assignor/UniformAssignor.java @4.4.0
Done when: The target assignment for elastic topics excludes not-ready children and prefers assigning a child to the member that owns its parent; tests with the uniform and range assignors and with heterogeneous subscriptions.

### [To Do] X-ELASTIC-0075 Drain detection in the group coordinator: a parent commit at the seal offset triggers a new target assignment
P0 · M12 · Task · deps: X-ELASTIC-0074 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java @4.4.0
Done when: An OffsetCommit that reaches a parent's seal offset (or a parent becoming empty by retention) bumps the group epoch and recomputes the assignment; test: children are assigned within one heartbeat after the drain commit.

### [To Do] X-ELASTIC-0076 No committed parent position: `auto.offset.reset` semantics for lineage
P0 · M12 · Task · deps: X-ELASTIC-0073
Done when: With no committed offset on the parent, `latest` makes children ready immediately at their start offsets and `earliest` requires draining the parent first; tests per reset policy.

### [To Do] X-ELASTIC-0077 Native SDK consumer: lineage-aware cursors continue from the parent's seal into its children
P0 · M12 · Task · deps: X-ELASTIC-0073, X-NATIVE-F12
Done when: On reaching a sealed parent's end the SDK consumer reads the children from their start offsets without gaps or duplicates; per-key order test across nested splits and merges.

### [To Do] X-ELASTIC-0078 Classic group protocol on elastic topics: detect out-of-order consumption and emit an event
P0 · M12 · Task · deps: X-ELASTIC-0073, GRP-F04 · ref: clients/src/main/java/org/apache/kafka/clients/consumer/internals/ConsumerCoordinator.java @4.4.0
Done when: When a classic group commits on a child while a parent's committed offset is below its seal, the coordinator emits `elastic_unordered_consumption` (rate-limited) and increments a metric; the limitation is documented; test.

### [To Do] X-ELASTIC-0079 Share groups on elastic topics: allowed, no ordering guarantee, sealed partitions drained normally
P2 · M12 · Task · deps: X-ELASTIC-0073, SHARE-F03
Done when: Share-group fetches on sealed partitions stop at the seal offset and children are acquirable immediately; documented; test.

### [To Do] X-ELASTIC-0080 Handoff events: `elastic_child_assignment_released` with reason parent_drained, parent_empty or reset_latest
P0 · M12 · Task · deps: X-ELASTIC-0075 · labels: observability
Done when: Events carry group id, topic, child partition and reason; flight-recorder entries in the group recorder; tests.

### [To Do] X-ELASTIC-0081 Compatibility test: Java consumer with the consumer protocol reads keyed data in order across a split
P0 · M12 · Task · deps: X-ELASTIC-0075, TEST-F08 · labels: test
Done when: A Java 4.4.0 consumer group using `group.protocol=consumer` consumes while splits happen; per-key order and completeness are verified.

## [To Do] X-ELASTIC-F09 Offsets across lineage
P0 · M12 · Feature · deps: X-ELASTIC-F03
Scope: Committed offsets and positions carried across split and merge.

### [To Do] X-ELASTIC-0082 Offset translation across lineage: positions on a drained parent map to children's start offsets
P0 · M12 · Task · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java @4.4.0
Done when: Pure function maps a parent position to child positions for split and merge; property test: translated positions never skip records and never re-read a record the parent already delivered.

### [To Do] X-ELASTIC-0083 Group coordinator writes children's committed offsets when a parent is drained
P0 · M12 · Task · deps: X-ELASTIC-0082, GRP-F07 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java @4.4.0
Done when: When a group's committed offset on a parent reaches the seal, the coordinator appends offset commit records for each child at its start offset (skipping children with an existing commit); event `elastic_offsets_carried`; tests with OffsetFetch before and after.

### [To Do] X-ELASTIC-0084 Merged child offsets: commit written only when all parents are drained by the group
P0 · M12 · Task · deps: X-ELASTIC-0083
Done when: A merge child gets its carried offset at max(seal offsets) only after every parent is drained; tests with one parent drained.

### [To Do] X-ELASTIC-0085 Transactional offset commits on sealed partitions and carried offsets interplay
P0 · M12 · Task · deps: X-ELASTIC-0083, TXN-F07
Done when: TxnOffsetCommit to a sealed parent is accepted; carried commits happen only after the transactional commit is completed; test with an aborted transaction leaving the parent undrained.

### [To Do] X-ELASTIC-0086 Offsets of deleted sealed partitions are removed from groups
P0 · M12 · Task · deps: X-ELASTIC-0083, X-ELASTIC-F05
Done when: When `RemoveElasticPartitionRecord` is applied, group offsets for the partition are deleted with tombstones like OffsetDelete; event emitted; test.

### [To Do] X-ELASTIC-0087 Native cursor tokens encode mapping epoch and per-partition positions across lineage
P0 · M12 · Task · deps: X-ELASTIC-0082, X-NATIVE-F06 · labels: decoder
Done when: Cursor tokens are versioned, bounded in size and validated on decode (fuzz target); a cursor from before a split resumes correctly after it; tests.

### [To Do] X-ELASTIC-0088 Offset-for-timestamp across lineage for native consumers
P1 · M12 · Task · deps: X-ELASTIC-0082
Done when: A native time-based seek returns per-active-partition positions resolved through the lineage (parent if the timestamp precedes the seal); tests with nested splits.

## [To Do] X-ELASTIC-F10 Virtual partition view
P1 · M12 · Feature · deps: X-ELASTIC-F02
Scope: Fixed logical partition count for Kafka clients over an elastic physical layout.

### [To Do] X-ELASTIC-0089 Virtual view model: fixed virtual partition count N, contiguous virtual partition ranges hosted by physical partitions
P1 · M12 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/clients/producer/internals/BuiltInPartitioner.java partitionForKey @4.4.0
Done when: Topic key `alluvium.elastic.virtual.partitions` (immutable, 1..=max) defines N; the mapping for a virtual-view topic ranges over virtual partition indexes; physical splits move contiguous virtual ranges; property test: every virtual partition has exactly one host per mapping epoch.

### [To Do] X-ELASTIC-0090 Virtual leader epochs: monotonic per virtual partition across host changes and leader changes
P1 · M12 · Task · deps: X-ELASTIC-0089
Done when: The virtual leader epoch is a metadata-maintained counter bumped on any host or leader change of the virtual partition; property test: never decreases across split, merge and elections.

### [To Do] X-ELASTIC-0091 Virtual partition sidecar index: per segment, batch position to virtual partition, rebuildable from the log
P1 · M12 · Task · deps: X-ELASTIC-0089, STOR-F09 · labels: decoder
Done when: Format documented in docs/formats/elastic-virtual-index.md with version and CRC; written through the fs trait; rebuilt on recovery if missing or corrupt; decoder fuzz target; crash-consistency test with the fault-injecting fs.

### [To Do] X-ELASTIC-0092 Metadata and DescribeTopicPartitions report N virtual partitions with the host's leader and the virtual epoch
P1 · M12 · Task · deps: X-ELASTIC-0090 · ref: clients/src/main/resources/common/message/MetadataResponse.json @4.4.0
Done when: For virtual-view topics, Kafka responses list partitions 0..N-1 with the hosting physical partition's leader, replicas, ISR and the virtual leader epoch; tests per Metadata version 0-13.

### [To Do] X-ELASTIC-0093 Produce to virtual partition v appends to the host with the virtual tag
P1 · M12 · Task · deps: X-ELASTIC-0091, X-ELASTIC-0092
Done when: Appends land in the hosting physical log with the sidecar entry; offsets returned are physical offsets, monotonic for v across moves because hosts start at or above the source's end offset; tests.

### [To Do] X-ELASTIC-0094 Fetch of virtual partition v: route by offset to the right host in the lineage and filter batches of v
P1 · M12 · Task · deps: X-ELASTIC-0093
Done when: Fetch resolves the host covering the fetch offset, reads only v's batches using the sidecar, reports HW and LSO of the host, and continues into the next host after a move; fetch sessions are keyed by virtual partition; tests including an offset in a moved range.

### [To Do] X-ELASTIC-0095 ListOffsets and OffsetForLeaderEpoch for virtual partitions
P1 · M12 · Task · deps: X-ELASTIC-0094
Done when: Earliest, latest, timestamp and max-timestamp are computed over v's batches across hosts; OffsetForLeaderEpoch uses the virtual epoch history; tests per version.

### [To Do] X-ELASTIC-0096 Producer state keyed by virtual partition on the host
P1 · M12 · Task · deps: X-ELASTIC-0093, STOR-F15 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java @4.4.0
Done when: Idempotent sequence validation and transaction tracking on a host are per (producer id, virtual partition); duplicates across moves are detected using state carried in the move; tests.

### [To Do] X-ELASTIC-0097 Transaction markers and aborted-transaction lists per virtual partition
P1 · M12 · Task · deps: X-ELASTIC-0096, TXN-F08
Done when: WriteTxnMarkers to virtual partition v writes a marker tagged for v on its host; read_committed fetches of v use v's aborted list only; tests.

### [To Do] X-ELASTIC-0098 Group offsets for virtual partitions are unchanged by physical moves
P1 · M12 · Task · deps: X-ELASTIC-0094
Done when: Committed offsets for virtual partitions remain valid across splits and merges (no carry needed); test with a Java consumer group across moves.

### [To Do] X-ELASTIC-0099 Compatibility suite on virtual-view topics with Java, librdkafka and franz-go through splits and merges
P1 · M12 · Task · deps: X-ELASTIC-0097, X-ELASTIC-0098, TEST-F08 · labels: test
Done when: Produce and consume, transactions and consumer groups pass with physical layout changes during the run; per-key order verified.

### [To Do] X-ELASTIC-0100 Benchmark virtual-view fetch filtering overhead versus a plain topic
P2 · M12 · Task · deps: X-ELASTIC-0094, TEST-F11 · labels: perf
Done when: Throughput and p99 fetch latency recorded; the cost of losing zero-copy is documented in the guide.

## [To Do] X-ELASTIC-F11 Elastic administration
P1 · M12 · Feature · deps: X-ELASTIC-F03
Scope: APIs and CLI for split, merge and lineage description.

### [To Do] X-ELASTIC-0101 Native admin IDL: SplitPartition, MergePartitions, DescribeLineage and DescribeKeyRanges with validate-only
P1 · M12 · Task · deps: X-NATIVE-F08
Done when: IDL methods with request and response messages, error details and pagination for lineage; breaking-change check passes.

### [To Do] X-ELASTIC-0102 Native admin handlers with authorization and controller forwarding
P1 · M12 · Task · deps: X-ELASTIC-0101, SEC-F08 · labels: security
Done when: Split and merge require ALTER on TOPIC, describe requires DESCRIBE; denials emit `authz_decision`; requests are forwarded to the active controller; validate-only runs full validation without records; tests per operation.

### [To Do] X-ELASTIC-0103 Creating elastic topics through Kafka CreateTopics with `alluvium.elastic.enabled=true` and through the native API
P1 · M12 · Task · deps: X-ELASTIC-0102, CTRL-F04
Done when: Both paths create the initial mapping with even ranges over the requested partition count; the config is immutable after creation (IncrementalAlterConfigs returns INVALID_CONFIG); tests.

### [To Do] X-ELASTIC-0104 Converting an existing topic to elastic via the virtual view, preserving Kafka key placement
P2 · M12 · Task · deps: X-ELASTIC-0103, X-ELASTIC-F10
Done when: Native ConvertToElastic sets N to the current partition count with one virtual partition per physical partition; existing producers and consumers see no change; test with a Java client before and after conversion.

### [To Do] X-ELASTIC-0105 Controller mutation quotas apply to split and merge
P1 · M12 · Task · deps: X-ELASTIC-0102, QUOTA-F05 · ref: server/src/main/java/org/apache/kafka/server/quota/ControllerMutationQuotaManager.java @4.4.0
Done when: Each split or merge consumes partition-mutation quota like CreatePartitions; exceeding returns THROTTLING_QUOTA_EXCEEDED with throttle time; test.

### [To Do] X-ELASTIC-0106 `alluvium admin elastic` commands: split, merge, describe-lineage, describe-ranges with `--json`
P1 · M12 · Task · deps: X-ELASTIC-0102, X-NATIVE-F14
Done when: Commands built on X-NATIVE-F14 print lineage as a tree and ranges as a table; golden-output tests.

### [To Do] X-ELASTIC-0107 Kafka admin tools against elastic topics: kafka-topics describe shows sealed partitions; CreatePartitions explains the alternative
P1 · M12 · Task · deps: X-ELASTIC-0103, TEST-F08 · labels: test
Done when: Compatibility tests for kafka-topics.sh `--describe` and `--alter --partitions` on elastic topics match the documented behavior.

## [To Do] X-ELASTIC-F12 Automatic resize hooks
P2 · M12 · Feature · deps: X-ELASTIC-F03
Scope: Signals and hooks for autopilot-driven split and merge.

### [To Do] X-ELASTIC-0108 Per-partition load signals for resize: bytes in, records in and a bounded key-hash histogram
P2 · M12 · Task · deps: none · ref: ADR from X-ELASTIC-0001 (elastic partitions)
Done when: Leaders maintain a fixed-size sampled histogram of key hashes and rate windows; memory per partition bounded and tested; signals exposed to the controller through the metrics pipeline chosen in the ADR.

### [To Do] X-ELASTIC-0109 Resize recommendation function: split and merge recommendations from signals and topic bounds
P2 · M12 · Task · deps: X-ELASTIC-0108
Done when: Pure function with inputs signals, `alluvium.elastic.auto.split.bytes.per.sec`, `alluvium.elastic.auto.merge.bytes.per.sec`, min and max partitions; split points come from the histogram median; property tests: never recommends beyond bounds.

### [To Do] X-ELASTIC-0110 Hysteresis and cooldown with `alluvium.elastic.auto.resize.cooldown.ms`
P2 · M12 · Task · deps: X-ELASTIC-0109
Done when: Oscillating load never produces a split followed by a merge of the same range within the cooldown; property test with an oscillating signal generator.

### [To Do] X-ELASTIC-0111 Recommendation exposure: native ListResizeRecommendations and `/debug/elastic/recommendations`
P2 · M12 · Task · deps: X-ELASTIC-0109, X-NATIVE-F08
Done when: Recommendations with reasons are readable through both; tests; X-AUTO-F08 consumes this interface.

### [To Do] X-ELASTIC-0112 Optional automatic execution with `alluvium.elastic.auto.resize.enable` (default false) and a rate limit
P2 · M12 · Task · deps: X-ELASTIC-0110
Done when: When enabled, the controller executes recommendations through the normal split and merge paths at most one operation per topic per cooldown; disabled by default (test); events `elastic_auto_resize_triggered` with reason.

### [To Do] X-ELASTIC-0113 Register the automatic resize keys in the alluvium registry
P2 · M12 · Task · deps: X-ELASTIC-0109, CFG-F11 · labels: config
Done when: `alluvium.elastic.auto.resize.enable`, `alluvium.elastic.auto.resize.cooldown.ms`, `alluvium.elastic.auto.split.bytes.per.sec` and `alluvium.elastic.auto.merge.bytes.per.sec` defined with validators, defaults and docs.

### [To Do] X-ELASTIC-0114 Simulation: automatic resize under ramping and oscillating load stays within bounds and does not flap
P2 · M12 · Task · deps: X-ELASTIC-0112, TEST-F06 · labels: test
Done when: Seeded simulation with load profiles asserts partition count within bounds, no flapping within the cooldown, and per-key order preserved.

## [To Do] X-ELASTIC-F13 Elastic metrics and events
P1 · M12 · Feature · deps: X-ELASTIC-F03, OBS-F04
Scope: Split and merge events, lineage depth, sealed partition counts.

### [To Do] X-ELASTIC-0115 Elastic event catalog entries in `events.rs` and generated docs/events.md
P1 · M12 · Task · deps: OBS-F07 · labels: observability, docs
Done when: Every elastic event (split, merge, seal, drain, deletion, reroute, handoff, auto-resize) is listed with trigger, fields and reason enum; the catalog drift check passes.

### [To Do] X-ELASTIC-0116 Elastic metrics: operations, seal duration, sealed partitions, lineage depth and mapping epoch
P1 · M12 · Task · deps: none · labels: observability
Done when: `alluvium_elastic_operations_total{topic,kind,result}`, `alluvium_elastic_seal_duration_seconds`, `alluvium_elastic_sealed_partitions{topic}`, `alluvium_elastic_lineage_depth{topic}`, `alluvium_elastic_mapping_epoch{topic}` and `alluvium_elastic_produce_redirects_total{topic,reason}` are registered and asserted in tests; no partition or key labels.

### [To Do] X-ELASTIC-0117 Standard log fields `mapping_epoch`, `seal_offset`, `key_range` and `operation_id`
P1 · M12 · Task · deps: OBS-F03 · labels: docs
Done when: Fields are added to the debuggability standard's field table and to the field-name constants; the field check passes.

### [To Do] X-ELASTIC-0118 Diagnostics route `/debug/elastic/{topic}`: mapping, lineage DAG, seal states and in-flight operation with flight recorder
P1 · M12 · Task · deps: OBS-F08
Done when: Route returns JSON with the current image view and the controller's elastic flight recorder for the topic; requires the diagnostics endpoint's authentication when not on loopback; test.

### [To Do] X-ELASTIC-0119 Human and JSON rendering of elastic records and images for `alluvium dump metadata`
P2 · M12 · Task · deps: CLI-F04 · labels: observability
Done when: Each elastic record type has a stable text and `--json` rendering consumed by the CLI-F04 dump command; golden-output tests cover every record and state; keys never appear (only ranges and hashes).

### [To Do] X-ELASTIC-0120 Alert rules and dashboard panels for stuck seals and redirect storms
P2 · M12 · Task · deps: X-ELASTIC-0116, OPS-F11
Done when: OPS-F11 rules fire when seal duration exceeds the timeout or redirects exceed a rate; dashboards show splits, sealed partitions and lineage depth; promtool tests.

## [To Do] X-ELASTIC-F14 Elastic simulation testing
P0 · M12 · Feature · deps: X-ELASTIC-F08, TEST-F06
Scope: Per-key ordering invariant across split and merge with failures.

### [To Do] X-ELASTIC-0121 Benchmark split latency and produce unavailability window
P1 · M12 · Task · deps: TEST-F11 · labels: perf, nfr
Done when: A benchmark measures request-to-commit time and the produce-unavailable window for a parent under 50 MB/s load, reported as p50/p99 in the benchmark CI job with a regression threshold; results are recorded in the ADR.

### [To Do] X-ELASTIC-0122 Checker: per-key order of acknowledged records holds across split and merge for lineage-aware readers
P0 · M12 · Task · deps: TEST-F05 · labels: test
Done when: A TEST-F05 checker compares each key's acknowledged sequence per producer with the sequence read by a lineage-aware reader and fails with the seed and flight recorders on any inversion.

### [To Do] X-ELASTIC-0123 Checker: no acknowledged write is lost across lineage
P0 · M12 · Task · deps: TEST-F05 · labels: test
Done when: Every acknowledged record is readable in exactly one partition of the lineage at its acknowledged offset until retention removes it.

### [To Do] X-ELASTIC-0124 Checker: every committed mapping epoch covers the hash space exactly once
P0 · M12 · Task · deps: TEST-F05 · labels: test
Done when: After every step the image on every broker passes the coverage check; violations report the epoch and ranges.

### [To Do] X-ELASTIC-0125 Checker: at most one partition accepts appends for any key hash at any time
P0 · M12 · Task · deps: TEST-F05 · labels: test
Done when: No two partitions append records whose key hashes fall in overlapping ranges within the same simulated instant, across mapping epochs and broker lag.

### [To Do] X-ELASTIC-0126 Checker: seal finality and offset continuity on every replica
P0 · M12 · Task · deps: TEST-F05 · labels: test
Done when: No replica of a sealed partition changes its LEO after the seal; every child start offset is >= each parent's seal offset; mapping epochs are monotonic on every broker.

### [To Do] X-ELASTIC-0127 Checker: consumer-protocol members never fetch a child before its parents are drained by their group
P0 · M12 · Task · deps: TEST-F05 · labels: test
Done when: Assignments and fetches are compared with group positions after every step.

### [To Do] X-ELASTIC-0128 Simulation: controller failover at every step of split and merge
P0 · M12 · Task · deps: X-ELASTIC-0122, X-ELASTIC-0123, X-ELASTIC-0124, X-ELASTIC-0125, X-ELASTIC-0126 · labels: test
Done when: Seeded runs crash the active controller after each elastic record and after each seal report; all checkers pass; failing seeds join the fixed set.

### [To Do] X-ELASTIC-0129 Simulation: parent leader crash and network partition during sealing
P0 · M12 · Task · deps: X-ELASTIC-0128 · labels: test
Done when: Runs isolate or crash the parent leader during Sealing and Committing; operations end Committed or Aborted and all checkers pass.

### [To Do] X-ELASTIC-0130 Simulation: disk faults during child log creation and sealed-partition deletion
P0 · M12 · Task · deps: X-ELASTIC-0128, TEST-F03 · labels: test
Done when: Write failures and torn writes during child log creation and deletion leave recoverable state; all checkers pass.

### [To Do] X-ELASTIC-0131 Simulation: split/merge storms with producers, consumer groups and transactions, nightly thousands of seeds
P0 · M12 · Task · deps: X-ELASTIC-0129, X-ELASTIC-0127 · labels: test
Done when: Nightly job runs 5000 seeds of mixed workloads with random splits and merges; CI runs the fixed seed set plus 20 new seeds per PR.

### [To Do] X-ELASTIC-0132 Elastic is off by default: no elastic records, configs rejected, wire responses unchanged
P0 · M12 · Task · deps: none · labels: test
Done when: With default config the feature level is 0, `alluvium.elastic.*` topic configs are rejected with INVALID_CONFIG, no elastic record is ever written, and Metadata/DescribeTopicPartitions responses are byte-identical to a build without the elastic code paths (golden test).

### [To Do] X-ELASTIC-0133 Kafka compatibility and differential suites pass with elastic enabled but unused
P0 · M12 · Task · deps: X-ELASTIC-0132, TEST-F09 · labels: test
Done when: A CI job runs the full TEST-F08 compatibility and TEST-F09 differential suites with `alluvium.elastic.version` finalized and the cluster switch on but no elastic topics; zero new diffs.

### [To Do] X-ELASTIC-0134 Benchmarks: elastic topic produce and fetch throughput versus classic topics
P0 · M12 · Task · deps: TEST-F11 · labels: perf, nfr
Done when: Steady-state throughput and p99 latency on an elastic topic without splits stay within 2% of a classic topic; results in the benchmark CI job.

## [To Do] X-ELASTIC-F15 Elastic documentation
P1 · M12 · Feature · deps: X-ELASTIC-F03
Scope: Concepts and operations guide.

### [To Do] X-ELASTIC-0135 Concepts guide: key ranges, lineage, sealing and ordering guarantees per client type
P1 · M12 · Task · deps: none · labels: docs
Done when: docs/guides/elastic-partitions.md explains the model with diagrams and a guarantee table (native SDK, Kafka consumer protocol, classic groups, share groups, reroute and virtual view); reviewed against the ADR.

### [To Do] X-ELASTIC-0136 Operations guide: running splits and merges, stuck-seal troubleshooting, metrics and events to watch
P1 · M12 · Task · deps: X-ELASTIC-0135 · labels: docs
Done when: The guide includes a runbook per alert from the elastic alert rules and the diagnostic routes; links check.

### [To Do] X-ELASTIC-0137 Migration and opt-in guide: enable the feature level and cluster switch, create or convert topics, roll back
P1 · M12 · Task · deps: X-ELASTIC-0135 · labels: docs
Done when: Covers enabling `alluvium.elastic.version`, creating elastic topics, converting via the virtual view, client requirements, and why downgrade is blocked while elastic topics exist.

### [To Do] X-ELASTIC-0138 Kafka compatibility statement for elastic topics in the compatibility guide
P1 · M12 · Task · deps: X-ELASTIC-0135, DOCS-F10 · labels: docs
Done when: DOCS-F10's guide lists each client-visible difference on elastic topics (sealed partitions, redirect errors, reroute offsets) with links to the ADR; classic topics are unaffected.

## [To Do] X-ELASTIC-F16 Lineage compaction
P2 · M12 · Feature · deps: X-ELASTIC-F05, STOR-F14
Scope: Compacted topics with split or merged partitions: carry live keys forward so sealed partitions can eventually be deleted without losing the latest value per key.

### [To Do] X-ELASTIC-0139 Spike: retiring sealed partitions of compacted topics
P2 · M12 · Spike · deps: none
Done when: an accepted ADR defines when a sealed partition of a compacted topic may be deleted (every live key rewritten into a child or tombstoned past the delete horizon), how keys are carried forward, and how consumers reading from the start observe it.

### [To Do] X-ELASTIC-0140 Carry live keys from sealed parents into children
P2 · M12 · Task · deps: X-ELASTIC-0139
Done when: a background job copies each key's latest value (and unexpired tombstones) from a sealed compacted parent into the child that owns the key's range, preserving the value bytes and headers; idempotent under restart.

### [To Do] X-ELASTIC-0141 Delete fully carried-forward sealed partitions
P2 · M12 · Task · deps: X-ELASTIC-0140
Done when: a sealed compacted partition is deleted only after carry-forward completes and every consumer group has committed past it or the configured grace period expires; an event records the deletion with its reason.

### [To Do] X-ELASTIC-0142 Simulation: latest value per key survives lineage compaction
P1 · M12 · Task · deps: X-ELASTIC-0141, TEST-F06 · labels: test
Done when: a simulation with splits, merges, crashes and compaction checks that reading a compacted topic from the start always yields the latest value for every live key.

### [To Do] X-ELASTIC-0143 Document lineage compaction
P2 · M12 · Task · deps: X-ELASTIC-0141 · labels: docs
Done when: the elastic partitions guide explains retention of sealed compacted partitions, carry-forward cost and the related metrics.
