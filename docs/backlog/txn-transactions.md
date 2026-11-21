# TXN: Idempotence and transactions

> Crates: `alluvium-txn-coordinator` (layer 2), producer state in `alluvium-storage` · Kafka counterpart: `transaction-coordinator/`, `ProducerStateManager` · Milestones: M6

Idempotent producers, producer id management, the transaction coordinator and state machine, markers, last stable offset, transactions v2 (KIP-890), admin APIs and exactly-once verification.

## [To Do] TXN-F01 Idempotent producer
P0 · M6 · Feature · deps: STOR-F15
Scope: Sequence validation, duplicate detection over the last five batches, epoch bumps, error semantics.

### [To Do] TXN-0001 Pure producer append validation over producer state entries
P0 · M6 · Task · deps: none · labels: foundation-for:X-DISKLESS · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java @4.4.0
Done when: a sans-I/O `ProducerAppendInfo` equivalent validates a batch against the current `ProducerStateEntry` and returns an updated entry without touching the log, with append origin (client, replication, coordinator) as input; it has no dependency on local segments so another sequencer can reuse it; module docs state the at-most-once sequence invariant.

### [To Do] TXN-0002 Producer epoch check for data batches
P0 · M6 · Task · deps: TXN-0001 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java checkProducerEpoch @4.4.0
Done when: a client data batch with an epoch lower than the producer's current epoch fails with INVALID_PRODUCER_EPOCH carrying topic, partition, offset and both epochs; the same condition from replication is accepted with a warning; tests for both origins.

### [To Do] TXN-0003 Sequence validation within the current epoch
P0 · M6 · Task · deps: TXN-0001 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java checkSequence @4.4.0
Done when: a batch's first sequence must equal the last sequence + 1, wrapping from `i32::MAX` to 0; anything else returns OUT_OF_ORDER_SEQUENCE_NUMBER with incoming and current sequences; when the producer has no epoch on record (state removed by retention or DeleteRecords) any sequence is accepted; tests for gap, repeat, wrap and missing state.

### [To Do] TXN-0004 Sequence reset on a new producer epoch
P0 · M6 · Task · deps: TXN-0003 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java checkSequence @4.4.0
Done when: the first batch of a higher epoch must start at sequence 0 when an earlier epoch exists, otherwise OUT_OF_ORDER_SEQUENCE_NUMBER; a client-side epoch bump after an OUT_OF_ORDER error (KIP-360: new epoch, sequence 0) is accepted and the old epoch is then rejected with INVALID_PRODUCER_EPOCH; tests.

### [To Do] TXN-0005 Duplicate batch detection over the last five batches
P0 · M6 · Task · deps: TXN-0003 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateEntry.java findDuplicateBatch @4.4.0
Done when: a batch whose epoch and sequence range match one of the producer's last five retained batches is not appended and the produce succeeds with the original base offset, last offset and log append time; a retried batch older than the retained five fails with OUT_OF_ORDER_SEQUENCE_NUMBER; tests at the retention boundary.

### [To Do] TXN-0006 Sequence continuity across several batches in one append
P0 · M6 · Task · deps: TXN-0003 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java appendDataBatch @4.4.0
Done when: multiple batches from one producer in a single produce are validated against the progressively updated entry, so a gap between them is rejected and a contiguous run is accepted; test with three batches.

### [To Do] TXN-0007 Validation rules per append origin
P0 · M6 · Task · deps: TXN-0001 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java maybeValidateDataBatch @4.4.0
Done when: client appends run epoch and sequence checks, replication appends skip sequence checks and only warn on stale epochs, coordinator appends skip sequence checks; a table-driven test covers each origin.

### [To Do] TXN-0008 Map idempotence errors to Produce responses in one place
P0 · M6 · Task · deps: TXN-0002, TXN-0003, ERR-F02
Done when: OUT_OF_ORDER_SEQUENCE_NUMBER and INVALID_PRODUCER_EPOCH map exhaustively from the storage error enum with the record error message; no other error is produced for sequence problems; tests check the produce partition response.

### [To Do] TXN-0009 Property test: each producer sequence is appended at most once
P0 · M6 · Task · deps: TXN-0005, TXN-0004 · labels: test
Done when: a proptest replays random retries, duplicates and reorderings of a producer's batches through validation and asserts the resulting log holds every acknowledged sequence exactly once and in order.

### [To Do] TXN-0010 Simulation of idempotent produce under retries and leader failover
P0 · M6 · Task · deps: TXN-0005, TEST-F06 · labels: test
Done when: seeded simulations with message loss, duplication and leader changes show no duplicate and no gap in each partition for idempotent producers, with the seed printed on failure.

### [To Do] TXN-0011 Idempotent producer compatibility tests
P0 · M6 · Task · deps: TXN-0005, TEST-F08 · labels: test
Done when: the Java 4.4.0 producer (idempotence enabled by default) and librdkafka with `enable.idempotence=true` produce through broker restarts and network faults without duplicates, verified by consuming and counting.

### [To Do] TXN-0012 Benchmark producer state validation on the append hot path
P2 · M6 · Task · deps: TXN-0005, TEST-F11 · labels: perf
Done when: a criterion benchmark measures validation cost per batch with 10 000 active producers and the result stays within the append budget in docs/nfr.md.

### [To Do] TXN-0013 Document idempotent producer semantics
P2 · M6 · Task · deps: TXN-0005 · labels: docs
Done when: module docs describe the epoch and sequence rules, duplicate handling and error codes with links to KIP-98 and KIP-360.

## [To Do] TXN-F02 Producer id management
P0 · M6 · Feature · deps: CTRL-F16
Scope: Non-transactional InitProducerId, block allocation and caching.

### [To Do] TXN-0014 Producer id block model
P0 · M6 · Task · deps: none · ref: server-common/src/main/java/org/apache/kafka/server/common/ProducerIdsBlock.java @4.4.0
Done when: a `ProducerIdsBlock` (assigned broker id, first producer id, size) yields ids in order, reports remaining ids and exhaustion, and rejects overflow with checked arithmetic; unit tests.

### [To Do] TXN-0015 Producer id manager state machine with prefetch
P0 · M6 · Task · deps: TXN-0014 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/RPCProducerIdManager.java @4.4.0
Done when: the sans-I/O manager hands out ids from the current block, requests the next block once 90% of the current block is used, keeps at most one request in flight, applies a 50 ms retry backoff after failures, and returns COORDINATOR_LOAD_IN_PROGRESS when the block is exhausted and the next is not ready; tests with the simulated clock.

### [To Do] TXN-0016 AllocateProducerIds client driver
P0 · M6 · Task · deps: TXN-0015, NET-F10 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/RPCProducerIdManager.java @4.4.0
Done when: the driver sends AllocateProducerIds with broker id and epoch to the active controller over the controller channel, retries error responses after the backoff, logs authentication failures once, and rejects a returned block that does not start after the current one; tests with a fake controller channel.

### [To Do] TXN-0017 Non-transactional InitProducerId
P0 · M6 · Task · deps: TXN-0015 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala handleInitProducerId @4.4.0
Done when: a request with a null transactional id returns a fresh producer id with epoch 0, ignoring v3+ producer id and epoch fields; a manager error is returned as its error code (COORDINATOR_LOAD_IN_PROGRESS is retriable); tests.

### [To Do] TXN-0018 Simulation: producer ids are unique across the cluster
P0 · M6 · Task · deps: TXN-0016, TEST-F06 · labels: test
Done when: a seeded simulation with several brokers allocating blocks across controller failovers asserts no producer id is handed out twice.

### [To Do] TXN-0019 Producer id allocation events and metrics
P1 · M6 · Task · deps: TXN-0016, OBS-F04
Done when: `producer_id_block_allocated` events carry the range and elapsed time, and a gauge reports remaining ids in the current block; documented in events.rs and docs/metrics.md.

## [To Do] TXN-F03 Transaction coordinator runtime
P0 · M6 · Feature · deps: GRP-F01
Scope: `__transaction_state` partitions, loading, coordinator epochs.

### [To Do] TXN-0020 Decide how the transaction coordinator runs on the shared coordinator runtime
P0 · M6 · Spike · deps: none · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala @4.4.0
Done when: ADR `transaction-coordinator-runtime` accepted in docs/adr/ deciding whether the transaction coordinator uses the GRP-F01 runtime or a dedicated state manager, and how coordinator epochs, append callbacks, loading and marker sending map onto it while preserving Kafka's TransactionStateManager semantics.

### [To Do] TXN-0021 Vendor and generate the transaction log record schemas
P0 · M6 · Task · deps: PROTO-F05 · labels: decoder · ref: transaction-coordinator/src/main/resources/common/message/TransactionLogValue.json @4.4.0
Done when: TransactionLogKey and TransactionLogValue schemas are vendored unmodified with checksums, types are generated, and a serde encodes key and value with version prefixes and tombstones; round-trip property tests for every version.

### [To Do] TXN-0022 Fuzz target and golden bytes for transaction log records
P0 · M6 · Task · deps: TXN-0021, TEST-F07 · labels: decoder, test
Done when: a cargo-fuzz target decodes arbitrary transaction log keys and values without panics; golden records written by Kafka 4.4.0 decode and re-encode byte for byte.

### [To Do] TXN-0023 Map a transactional id to its `__transaction_state` partition
P0 · M6 · Task · deps: GRP-F02 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala partitionFor @4.4.0
Done when: the partition is `abs(java_string_hash(transactional_id)) % transaction.state.log.num.partitions` using the same Java-compatible hash as group ids; golden tests against Kafka.

### [To Do] TXN-0024 `__transaction_state` topic configuration and auto-creation
P0 · M6 · Task · deps: CFG-F09 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala transactionStateTopicConfigs @4.4.0
Done when: creation uses `transaction.state.log.num.partitions`, `transaction.state.log.replication.factor`, and configs `cleanup.policy=compact`, `min.insync.replicas=transaction.state.log.min.isr`, `segment.bytes=transaction.state.log.segment.bytes` and the remaining Kafka settings; auto-creation happens on the first transaction coordinator lookup; tests assert the config map.

### [To Do] TXN-0025 FindCoordinator TRANSACTION key type
P0 · M6 · Task · deps: TXN-0023, TXN-0024, GRP-F03 · ref: core/src/main/scala/kafka/server/KafkaApis.scala getCoordinator @4.4.0
Done when: TRANSACTION keys resolve to the leader of the transactional id's partition, auto-create the topic when missing and return COORDINATOR_NOT_AVAILABLE until a leader exists; tests mirror the GROUP key cases.

### [To Do] TXN-0026 Transaction metadata cache with coordinator epochs
P0 · M6 · Task · deps: TXN-0020, TXN-0021 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala getAndMaybeAddTransactionState @4.4.0
Done when: lookups return COORDINATOR_LOAD_IN_PROGRESS for a loading partition and NOT_COORDINATOR for a partition not owned; `put_if_not_exists` adds new metadata; each owned partition records its coordinator epoch; unit tests.

### [To Do] TXN-0027 Load transaction state on becoming coordinator
P0 · M6 · Task · deps: TXN-0026 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala loadTransactionsForTxnTopicPartition @4.4.0
Done when: loading replays the partition up to the log end offset (tombstones remove ids), sets the coordinator epoch from the leader epoch, and afterwards resends markers for every transaction in PrepareCommit or PrepareAbort; a newer load for the same partition supersedes an older one; tests.

### [To Do] TXN-0028 Unload transaction state on resignation
P0 · M6 · Task · deps: TXN-0027 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala removeTransactionsForTxnTopicPartition @4.4.0
Done when: resignation with a matching or newer epoch removes cached metadata and loading markers for the partition and drops its queued markers; requests then return NOT_COORDINATOR; tests.

### [To Do] TXN-0029 Append transaction state transitions to the log
P0 · M6 · Task · deps: TXN-0026 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala appendTransactionToLog @4.4.0
Done when: a transition record is appended with `acks=all`; on completion the coordinator epoch and pending state are re-checked (mismatch returns NOT_COORDINATOR) before completing the transition; UNKNOWN_TOPIC_OR_PARTITION, NOT_ENOUGH_REPLICAS, NOT_ENOUGH_REPLICAS_AFTER_APPEND and REQUEST_TIMED_OUT map to COORDINATOR_NOT_AVAILABLE, NOT_LEADER_OR_FOLLOWER and KAFKA_STORAGE_ERROR to NOT_COORDINATOR, MESSAGE_TOO_LARGE and RECORD_LIST_TOO_LARGE to UNKNOWN_SERVER_ERROR; a failed append aborts the pending transition.

### [To Do] TXN-0030 Detect a change of the `__transaction_state` partition count
P0 · M6 · Task · deps: TXN-0027 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala validateTransactionTopicPartitionCountIsStable @4.4.0
Done when: a partition count different from the one seen at startup stops transaction coordination with a clear error event instead of silently remapping transactional ids; test with a changed metadata image.

### [To Do] TXN-0031 Read the finalized `transaction.version` feature level
P0 · M6 · Task · deps: TXN-0026, META-F06 · ref: server-common/src/main/java/org/apache/kafka/server/common/TransactionVersion.java @4.4.0
Done when: the coordinator reads TV_0, TV_1 or TV_2 from the metadata image, uses it for record value versions (flexible records from TV_1) and server-initiated transitions, and reacts to level changes; tests per level.

### [To Do] TXN-0032 Transaction coordinator driver and leadership hooks
P0 · M6 · Task · deps: TXN-0027, TXN-0028, TXN-0029, SRV-F14 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala onElection @4.4.0
Done when: the driver runs as a supervised named task, receives `on_election`/`on_resignation` from `__transaction_state` leadership changes through the metadata publisher, and shuts down in the documented order; tests for election and resignation.

### [To Do] TXN-0033 Simulation of transaction coordinator failover
P0 · M6 · Task · deps: TXN-0032, TEST-F06 · labels: test
Done when: seeded simulations fail over the coordinator at random points and assert the new coordinator's state equals the committed records and that prepared transactions are completed.

### [To Do] TXN-0034 Document the `__transaction_state` record format
P1 · M6 · Task · deps: TXN-0021 · labels: docs
Done when: docs/formats/transaction-state.md lists key and value fields per version with hex examples checked by a doc test.

### [To Do] TXN-0035 Expose the transaction log decoder for dump tooling
P1 · M6 · Task · deps: TXN-0021
Done when: a library function renders transaction log records as text and JSON for `alluvium dump log` (CLI-F04); unit test on golden records.

## [To Do] TXN-F04 Transaction state machine
P0 · M6 · Feature · deps: TXN-F03
Scope: Empty, Ongoing, PrepareCommit/Abort, CompleteCommit/Abort, Dead, PrepareEpochFence transitions.

### [To Do] TXN-0036 Transaction metadata and transit metadata model
P0 · M6 · Task · deps: none · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java @4.4.0
Done when: metadata holds transactional id, producer id, previous and next producer id, producer epoch, last producer epoch, timeout, state, partitions, start and last update timestamps, client transaction version and pending state; transit metadata is an immutable target; `Debug` output contains no payloads.

### [To Do] TXN-0037 Transaction state enum with ids, names and valid previous states
P0 · M6 · Task · deps: TXN-0036 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionState.java @4.4.0
Done when: EMPTY(0), ONGOING(1), PREPARE_COMMIT(2), PREPARE_ABORT(3), COMPLETE_COMMIT(4), COMPLETE_ABORT(5), DEAD(6) and PREPARE_EPOCH_FENCE(7) carry Kafka's ids and names, valid previous states and `is_expiration_allowed` (EMPTY, COMPLETE_COMMIT, COMPLETE_ABORT); unit tests.

### [To Do] TXN-0038 Pending transition protocol
P0 · M6 · Task · deps: TXN-0037 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java completeTransitionTo @4.4.0
Done when: preparing a transition sets the pending state (a second prepare while one is pending is refused so callers return CONCURRENT_TRANSACTIONS), completing validates the transit metadata against the current state and producer id/epoch rules (violations panic as invariant breaches), and aborting clears the pending state; tests.

### [To Do] TXN-0039 Producer epoch bump, fence and producer id rotation
P0 · M6 · Task · deps: TXN-0036 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java prepareIncrementProducerEpoch @4.4.0
Done when: epoch exhaustion is `epoch >= i16::MAX - 1`; incrementing checks the expected epoch (mismatch returns PRODUCER_FENCED); fencing bumps the epoch and records the last epoch; rotation installs a new producer id with epoch 0 and keeps the previous id; ported `TransactionMetadataTest` cases pass.

### [To Do] TXN-0040 Transition to Ongoing when partitions are added
P0 · M6 · Task · deps: TXN-0038 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java prepareAddPartitions @4.4.0
Done when: Empty, CompleteCommit and CompleteAbort move to Ongoing with a new start timestamp when partitions are added, and Ongoing stays Ongoing while adding more; unit test per source state.

### [To Do] TXN-0041 Transition Ongoing to PrepareCommit
P0 · M6 · Task · deps: TXN-0038 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java prepareAbortOrCommit @4.4.0
Done when: a commit request moves Ongoing to PrepareCommit keeping the partition set and updating the last update timestamp; any other source state is refused; unit test.

### [To Do] TXN-0042 Transition to PrepareAbort
P0 · M6 · Task · deps: TXN-0038 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java prepareAbortOrCommit @4.4.0
Done when: Ongoing moves to PrepareAbort on client abort or timeout; PrepareEpochFence moves to PrepareAbort with the fenced epoch; Empty, CompleteCommit and CompleteAbort move to PrepareAbort only for transactions v2 aborts with no partitions; unit test per source state.

### [To Do] TXN-0043 Transition PrepareCommit to CompleteCommit and PrepareAbort to CompleteAbort
P0 · M6 · Task · deps: TXN-0038 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java prepareComplete @4.4.0
Done when: after markers are written the transaction moves to the matching Complete state, clears its partitions and updates timestamps; unit tests.

### [To Do] TXN-0044 Transition Ongoing to PrepareEpochFence
P0 · M6 · Task · deps: TXN-0039, TXN-0038 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java prepareFenceProducerEpoch @4.4.0
Done when: fencing an ongoing transaction (InitProducerId or timeout) records PrepareEpochFence as the pending state with the bumped epoch, never as a persisted state; finding PrepareEpochFence or Dead as a current state panics; unit tests.

### [To Do] TXN-0045 Transition to Empty on producer re-initialization
P0 · M6 · Task · deps: TXN-0039, TXN-0038 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java prepareProducerIdRotation @4.4.0
Done when: Empty, CompleteCommit and CompleteAbort move to Empty with a bumped epoch or a rotated producer id; unit tests for both.

### [To Do] TXN-0046 Transition to Dead on transactional id expiration
P0 · M6 · Task · deps: TXN-0038 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java prepareDead @4.4.0
Done when: Empty, CompleteCommit and CompleteAbort move to Dead and produce a tombstone; other states are refused; unit test.

### [To Do] TXN-0047 Model-based property test of the transaction state machine
P0 · M6 · Task · deps: TXN-0040, TXN-0041, TXN-0042, TXN-0043, TXN-0044, TXN-0045, TXN-0046 · labels: test
Done when: random sequences of coordinator operations never yield a transition outside the valid previous-state table and producer epochs never decrease for a producer id.

## [To Do] TXN-F05 Transactional InitProducerId
P0 · M6 · Feature · deps: TXN-F04
Scope: Fencing, epoch bumps, recovery of in-flight transactions.

### [To Do] TXN-0048 Transactional InitProducerId validation
P0 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala handleInitProducerId @4.4.0
Done when: an empty transactional id returns INVALID_REQUEST; a timeout <= 0 or above `transaction.max.timeout.ms` returns INVALID_TRANSACTION_TIMEOUT; `keep_prepared_txn` returns UNSUPPORTED_VERSION; the 2PC flag is handled by TXN-F14; tests per error.

### [To Do] TXN-0049 InitProducerId for a new transactional id
P0 · M6 · Task · deps: TXN-0048, TXN-F02 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala handleInitProducerId @4.4.0
Done when: an unknown transactional id gets new metadata with a fresh producer id, is moved to Empty with epoch 0 through the log, and the response returns the producer id and epoch after the append commits; tests.

### [To Do] TXN-0050 InitProducerId for an existing transactional id in a completed state
P0 · M6 · Task · deps: TXN-0048 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala prepareInitProducerIdTransit @4.4.0
Done when: Empty, CompleteCommit and CompleteAbort bump the epoch, or rotate to a new producer id when the epoch is exhausted (and the expected epoch matches); the new timeout is stored; tests for bump and rotation.

### [To Do] TXN-0051 InitProducerId while a transition is in progress
P0 · M6 · Task · deps: TXN-0048 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala prepareInitProducerIdTransit @4.4.0
Done when: a pending transition or a PrepareCommit/PrepareAbort state returns CONCURRENT_TRANSACTIONS without changes; tests.

### [To Do] TXN-0052 InitProducerId fencing an ongoing transaction
P0 · M6 · Task · deps: TXN-0050, TXN-F08 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala handleInitProducerId @4.4.0
Done when: an Ongoing transaction is fenced (PrepareEpochFence), aborted by the coordinator using the server transaction version, and the request returns CONCURRENT_TRANSACTIONS so the client retries and receives a second bump; an abort failure returns its error; tests.

### [To Do] TXN-0053 InitProducerId with an expected producer id and epoch (KIP-360)
P0 · M6 · Task · deps: TXN-0050 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala prepareInitProducerIdTransit @4.4.0
Done when: v3+ requests are accepted when the stored epoch is -1, the producer id matches, or the previous producer id matches with an exhausted epoch (retry after rotation); otherwise PRODUCER_FENCED; a retry of a completed bump returns the same id and epoch; tests per rule.

### [To Do] TXN-0054 Zombie fencing after re-initialization
P0 · M6 · Task · deps: TXN-0053 · labels: test
Done when: after a second producer instance initializes, the first instance gets PRODUCER_FENCED from AddPartitionsToTxn and EndTxn and INVALID_PRODUCER_EPOCH from Produce; integration test.

### [To Do] TXN-0055 Port Kafka's InitProducerId coordinator tests
P1 · M6 · Task · deps: TXN-0052, TXN-0053, TEST-F10 · labels: test · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala @4.4.0
Done when: InitProducerId cases from `TransactionCoordinatorTest` in the TEST-F10 inventory are ported with attributions and pass.

## [To Do] TXN-F06 AddPartitionsToTxn
P0 · M6 · Feature · deps: TXN-F04
Scope: Client and broker verification paths (KIP-890 part 1).

### [To Do] TXN-0056 AddPartitionsToTxn from clients (v0-v3)
P0 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala handleAddPartitionsToTransaction @4.4.0
Done when: an empty transactional id returns INVALID_REQUEST; unknown metadata or a producer id mismatch INVALID_PRODUCER_ID_MAPPING; an epoch mismatch PRODUCER_FENCED; a pending transition, PrepareCommit or PrepareAbort CONCURRENT_TRANSACTIONS; partitions already in an Ongoing transaction return NONE without a write; otherwise the partitions are added through the log; every partition in the response carries the same result.

### [To Do] TXN-0057 Verify-only AddPartitionsToTxn from brokers (v4+)
P0 · M6 · Task · deps: TXN-0056 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala handleVerifyPartitionsInTransaction @4.4.0
Done when: batched transactions are checked independently; for each partition the result is NONE if it is in the ongoing transaction and TRANSACTION_ABORTABLE otherwise; INVALID_PRODUCER_ID_MAPPING, PRODUCER_FENCED, CONCURRENT_TRANSACTIONS and INVALID_REQUEST apply per transaction as in Kafka; tests.

### [To Do] TXN-0058 Verification guard in partition producer state
P0 · M6 · Task · deps: STOR-F15 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/VerificationStateEntry.java @4.4.0
Done when: starting verification records a guard (and lowest sequence and epoch-bump support) in producer state; a transactional append must present the matching guard, otherwise INVALID_TXN_STATE ("not part of an ongoing transaction"); a stale epoch returns INVALID_PRODUCER_EPOCH; guards are cleared when the transaction starts or ends; tests.

### [To Do] TXN-0059 Leader-side verification manager
P0 · M6 · Task · deps: TXN-0057, TXN-0058, NET-F10 · ref: server/src/main/java/org/apache/kafka/server/transaction/AddPartitionsToTxnManager.java @4.4.0
Done when: transactional produces (and transactional offset commits) wait for verification batched per coordinator node with one request in flight per node; a newer request for the same producer supersedes an older one; CLUSTER_AUTHORIZATION_FAILED maps to INVALID_TXN_STATE, PRODUCER_FENCED to INVALID_PRODUCER_EPOCH, TRANSACTION_ABORTABLE to INVALID_TXN_STATE for clients that cannot handle it, coordinator lookup failures to retriable errors and disconnects to NETWORK_EXCEPTION; tests per mapping.

### [To Do] TXN-0060 Verification configuration
P0 · M6 · Task · deps: TXN-0059 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/AddPartitionsToTxnConfig.java @4.4.0
Done when: `transaction.partition.verification.enable` (dynamic) turns verification on and off for new requests, and `add.partitions.to.txn.retry.backoff.ms` and `add.partitions.to.txn.retry.backoff.max.ms` drive retries of CONCURRENT_TRANSACTIONS responses; tests per key.

### [To Do] TXN-0061 Test that verification prevents hanging transactions
P0 · M6 · Task · deps: TXN-0059 · labels: test
Done when: a produce from a producer whose transaction was already aborted, without a new AddPartitionsToTxn, is rejected and leaves no open transaction at the partition; a regression test reproduces the KIP-890 hanging scenario.

### [To Do] TXN-0062 Port Kafka's AddPartitionsToTxn tests
P1 · M6 · Task · deps: TXN-0057, TXN-0059, TEST-F10 · labels: test · ref: server/src/main/java/org/apache/kafka/server/transaction/AddPartitionsToTxnManager.java @4.4.0
Done when: coordinator and manager cases in the TEST-F10 inventory are ported with attributions and pass.

## [To Do] TXN-F07 Offsets in transactions
P0 · M6 · Feature · deps: TXN-F04
Scope: AddOffsetsToTxn and TxnOffsetCommit.

### [To Do] TXN-0063 AddOffsetsToTxn
P0 · M6 · Task · deps: GRP-F02 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala handleAddPartitionsToTransaction @4.4.0
Done when: the group id is mapped to its `__consumer_offsets` partition and added to the transaction through the AddPartitionsToTxn path with the same validation and error codes; tests.

### [To Do] TXN-0064 TxnOffsetCommit validation
P0 · M6 · Task · deps: GRP-F07 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java validateTransactionalOffsetCommit @4.4.0
Done when: a missing group with generation < 0 creates a simple group, otherwise GROUP_ID_NOT_FOUND on v6+ and ILLEGAL_GENERATION below; member id, instance id and generation are validated only when provided (UNKNOWN_MEMBER_ID, FENCED_INSTANCE_ID, ILLEGAL_GENERATION); a stale consumer-protocol epoch returns STALE_MEMBER_EPOCH on v6+ and ILLEGAL_GENERATION below (KIP-1319); oversized metadata returns OFFSET_METADATA_TOO_LARGE per partition.

### [To Do] TXN-0065 Topic ids in TxnOffsetCommit (v6, KIP-1319)
P0 · M6 · Task · deps: TXN-0064 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java commitTransactionalOffset @4.4.0
Done when: v6 commits resolve topic ids and an unknown id returns UNKNOWN_TOPIC_ID for its partitions; tests.

### [To Do] TXN-0066 Pending transactional offsets
P0 · M6 · Task · deps: TXN-0064 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java commitTransactionalOffset @4.4.0
Done when: transactional commits are written as transactional records (producer id and epoch) through the runtime's transactional write path, kept in a pending store per producer id invisible to normal fetches, and indexed as open transactions per group and topic; tests.

### [To Do] TXN-0067 Apply COMMIT and ABORT markers to pending offsets
P0 · M6 · Task · deps: TXN-0066 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java replayEndTransactionMarker @4.4.0
Done when: a COMMIT marker moves a producer's pending offsets into committed offsets unless a newer committed record already exists for the partition, an ABORT marker drops them, and both clear the open-transaction indexes; replay during loading gives the same result; tests.

### [To Do] TXN-0068 Complete transactions on `__consumer_offsets` partitions
P0 · M6 · Task · deps: TXN-0067, TXN-F08 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java completeTransaction @4.4.0
Done when: a WriteTxnMarkers request for a `__consumer_offsets` partition writes the control batch through the coordinator runtime's transaction completion with the coordinator epoch, and an inactive coordinator returns COORDINATOR_NOT_AVAILABLE; tests.

### [To Do] TXN-0069 OffsetFetch `require_stable` with pending transactional offsets
P0 · M6 · Task · deps: TXN-0066 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java fetchOffsets @4.4.0
Done when: with `require_stable` a partition with a pending transactional offset returns UNSTABLE_OFFSET_COMMIT, for listed partitions and for all-partition fetches; without the flag the last committed offset is returned; tests.

### [To Do] TXN-0070 Pending transactional offsets block expiry and group removal
P0 · M6 · Task · deps: TXN-0066 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java cleanupExpiredOffsets @4.4.0
Done when: an offset with a pending transactional offset for its partition is not expired, and a group with open transactions is not removed; tests.

### [To Do] TXN-0071 Implicit offsets partition addition for TxnOffsetCommit v5+ under transactions v2
P0 · M6 · Task · deps: TXN-0066, TXN-0059 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java commitTransactionalOffsets @4.4.0
Done when: with transactions v2 a TxnOffsetCommit v5+ verifies and adds the `__consumer_offsets` partition to the transaction (replacing AddOffsetsToTxn), and below v5 the client must have called AddOffsetsToTxn; tests per version.

### [To Do] TXN-0072 Port Kafka's transactional offset tests
P1 · M6 · Task · deps: TXN-0067, TXN-0069, TEST-F10 · labels: test · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/OffsetMetadataManager.java @4.4.0
Done when: transactional cases from `OffsetMetadataManagerTest` in the TEST-F10 inventory are ported with attributions and pass.

## [To Do] TXN-F08 EndTxn and markers
P0 · M6 · Feature · deps: TXN-F04
Scope: Marker writing via WriteTxnMarkers, retries, completion.

### [To Do] TXN-0073 EndTxn for transaction versions 0 and 1
P0 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala endTransactionWithTV1 @4.4.0
Done when: an empty id returns INVALID_REQUEST; unknown metadata or producer id mismatch INVALID_PRODUCER_ID_MAPPING; a client epoch different from the stored one (or a lower epoch) PRODUCER_FENCED; a pending transition other than an epoch fence CONCURRENT_TRANSACTIONS; Ongoing moves to PrepareCommit or PrepareAbort; a retry against CompleteCommit/CompleteAbort with the matching result returns NONE; PrepareCommit/PrepareAbort with the matching result CONCURRENT_TRANSACTIONS; mismatched results and Empty return INVALID_TXN_STATE.

### [To Do] TXN-0074 EndTxn completion after the prepare record commits
P0 · M6 · Task · deps: TXN-0073 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala endTransaction @4.4.0
Done when: after the PrepareCommit/PrepareAbort record commits the coordinator epoch, producer id/epoch and state are re-checked (NOT_COORDINATOR, INVALID_PRODUCER_ID_MAPPING, PRODUCER_FENCED, CONCURRENT_TRANSACTIONS, INVALID_TXN_STATE), the client receives its response, and markers are queued; a failed append returns its mapped error; tests.

### [To Do] TXN-0075 Transaction marker queues per destination broker
P0 · M6 · Task · deps: TXN-0074 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TxnMarkerQueue.java @4.4.0
Done when: markers are grouped by partition leader into per-broker queues keyed by transaction topic partition; partitions without a known leader wait in an unknown-destination queue and are retried on metadata updates; queue depth is bounded by in-flight transactions and exported; tests.

### [To Do] TXN-0076 Send WriteTxnMarkers requests
P0 · M6 · Task · deps: TXN-0075, NET-F10 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionMarkerChannelManager.scala @4.4.0
Done when: the driver drains each broker queue into WriteTxnMarkers (v1-v2) requests carrying producer id, epoch, coordinator epoch, result and partitions, using the inter-node client with timeouts; a disconnect re-enqueues all markers of the request; tests with a simulated network.

### [To Do] TXN-0077 Handle WriteTxnMarkers responses
P0 · M6 · Task · deps: TXN-0076 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionMarkerRequestCompletionHandler.scala @4.4.0
Done when: NONE removes the partition; UNKNOWN_TOPIC_OR_PARTITION, NOT_LEADER_OR_FOLLOWER, NOT_ENOUGH_REPLICAS, NOT_ENOUGH_REPLICAS_AFTER_APPEND, REQUEST_TIMED_OUT and KAFKA_STORAGE_ERROR re-enqueue it; INVALID_PRODUCER_EPOCH and TRANSACTION_COORDINATOR_FENCED abandon the transaction's markers; UNSUPPORTED_FOR_MESSAGE_FORMAT and UNSUPPORTED_VERSION skip the partition with a warning; CORRUPT_MESSAGE, MESSAGE_TOO_LARGE, RECORD_LIST_TOO_LARGE and INVALID_REQUIRED_ACKS are invariant violations; a coordinator that lost the partition or is loading drops the markers.

### [To Do] TXN-0078 Write the Complete record after all markers are acknowledged
P0 · M6 · Task · deps: TXN-0077 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionMarkerChannelManager.scala tryAppendToLog @4.4.0
Done when: when every partition acknowledged, the CompleteCommit/CompleteAbort record is appended; NOT_COORDINATOR drops it, COORDINATOR_NOT_AVAILABLE retries the append, COORDINATOR_LOAD_IN_PROGRESS drops it; the transition completes on success; tests per error.

### [To Do] TXN-0079 Write transaction markers at the partition leader
P0 · M6 · Task · deps: STOR-F15, REC-F11 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java appendEndTxnMarker @4.4.0
Done when: a WriteTxnMarkers entry appends a COMMIT or ABORT control batch; a coordinator epoch lower than the one in producer state returns TRANSACTION_COORDINATOR_FENCED; a stale producer epoch returns INVALID_PRODUCER_EPOCH; an unknown partition returns UNKNOWN_TOPIC_OR_PARTITION; the completed transaction updates producer state and the first unstable offset; tests.

### [To Do] TXN-0080 Drop queued markers when the coordinator resigns
P0 · M6 · Task · deps: TXN-0075, TXN-0028 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionMarkerChannelManager.scala removeMarkersForTxnTopicPartition @4.4.0
Done when: resignation of a transaction topic partition removes its queued markers and pending completions; the new coordinator resends them after loading; tests.

### [To Do] TXN-0081 Simulation of marker delivery under faults
P0 · M6 · Task · deps: TXN-0078, TXN-0079, TEST-F06 · labels: test
Done when: seeded simulations with coordinator and partition leader failovers assert every prepared transaction eventually has markers on all its partitions and reaches a Complete state, and no partition keeps an open transaction once the coordinator completed it.

## [To Do] TXN-F09 Transaction timeouts
P0 · M6 · Feature · deps: TXN-F04
Scope: `transaction.max.timeout.ms`, proactive abort of expired transactions.

### [To Do] TXN-0082 Find timed-out transactions
P0 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala timedOutTransactions @4.4.0
Done when: every `transaction.abort.timed.out.transaction.cleanup.interval.ms` the coordinator lists Ongoing transactions with no pending transition whose start timestamp plus timeout is before now, using the injected clock; tests.

### [To Do] TXN-0083 Abort timed-out transactions
P0 · M6 · Task · deps: TXN-0082, TXN-F08 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala abortTimedOutTransactions @4.4.0
Done when: each timed-out transaction is fenced (epoch bumped unless exhausted) and aborted as a coordinator-initiated EndTxn; a producer id mismatch is logged; INVALID_PRODUCER_ID_MAPPING, PRODUCER_FENCED and CONCURRENT_TRANSACTIONS end the attempt at debug level and other errors at warn; tests.

### [To Do] TXN-0084 Producer behavior after a timeout abort
P0 · M6 · Task · deps: TXN-0083 · labels: test
Done when: tests show that after a timeout abort the original producer gets PRODUCER_FENCED from EndTxn or AddPartitionsToTxn and INVALID_PRODUCER_EPOCH from Produce, for transaction versions 1 and 2.

### [To Do] TXN-0085 Consume `transaction.max.timeout.ms`
P0 · M6 · Task · deps: CFG-F09
Done when: the coordinator reads `transaction.max.timeout.ms` for InitProducerId validation and documents that it caps client `transaction.timeout.ms`; test at the boundary value.

### [To Do] TXN-0086 Timeout aborts unblock read_committed consumers
P0 · M6 · Task · deps: TXN-0083, TXN-F11 · labels: test
Done when: an abandoned transaction is aborted after its timeout, the last stable offset advances past it, and a read_committed consumer progresses; simulated-clock test.

## [To Do] TXN-F10 Transactions v2
P0 · M6 · Feature · deps: TXN-F08
Scope: KIP-890 part 2: per-transaction epoch bump, implicit partition addition.

### [To Do] TXN-0087 Determine the client transaction version
P0 · M6 · Task · deps: none · ref: server-common/src/main/java/org/apache/kafka/server/common/TransactionVersion.java transactionVersionForEndTxn @4.4.0
Done when: EndTxn above v4 and AddPartitionsToTxn above v3 mark the client as TV_2, lower versions TV_0; the server uses TV_2 semantics only if the finalized `transaction.version` is 2; tests per version.

### [To Do] TXN-0088 EndTxn under transactions v2
P0 · M6 · Task · deps: TXN-0087, TXN-0074 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala endTransaction @4.4.0
Done when: epoch validity follows Kafka's TV2 rules (same epoch for Ongoing/Empty, retry-on-bump for Prepare states, retry-on-bump, retry-on-overflow or same epoch for Complete states); every completion bumps the epoch and rotates the producer id when exhausted; the response returns the new producer id and epoch; retries return NONE with the stored id and epoch; tests per state.

### [To Do] TXN-0089 Abort without partitions under transactions v2
P0 · M6 · Task · deps: TXN-0088 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java prepareAbortOrCommit @4.4.0
Done when: an abort for Empty, CompleteCommit or CompleteAbort moves to PrepareAbort with no partitions to bump the epoch, and completes without sending markers; tests.

### [To Do] TXN-0090 Producer epoch overflow under transactions v2
P0 · M6 · Task · deps: TXN-0088 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala endTransaction @4.4.0
Done when: when the epoch reaches `i16::MAX - 1` the completion rotates to a new producer id stored as next producer id; a client retry with the old id and epoch is recognised and succeeds; tests.

### [To Do] TXN-0091 Implicit partition addition on produce
P0 · M6 · Task · deps: TXN-0087, TXN-0059 · ref: server/src/main/java/org/apache/kafka/server/transaction/AddPartitionsToTxnManager.java @4.4.0
Done when: a TV2 transactional produce makes the partition leader add the partition through the coordinator (not verify-only) before appending, so clients no longer send AddPartitionsToTxn; tests with a TV2 client.

### [To Do] TXN-0092 Marker epoch rules for transactions v2
P0 · M6 · Task · deps: TXN-0079 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java checkProducerEpoch @4.4.0
Done when: a TV2 marker must carry an epoch strictly greater than the producer's current epoch, except that a marker with the same epoch and no ongoing transaction is accepted as an idempotent retry (KAFKA-19999); TV0/1 markers need epoch >= current; tests.

### [To Do] TXN-0093 Sequence rules with verification state
P0 · M6 · Task · deps: TXN-0058 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java checkSequence @4.4.0
Done when: a TV2 idempotent producer with no state must start at sequence 0, and a first sequence above the verification state's lowest sequence returns OUT_OF_ORDER_SEQUENCE_NUMBER; tests.

### [To Do] TXN-0094 TRANSACTION_ABORTABLE and its downgrade for older versions
P0 · M6 · Task · deps: TXN-0087, ERR-F02
Done when: errors that are abortable are returned as TRANSACTION_ABORTABLE to clients whose request version supports it (EndTxn v4+, AddPartitionsToTxn v5+, TxnOffsetCommit v4+, InitProducerId v5+) and mapped to the older code otherwise, through the per-version downgrade table; tests per API version.

### [To Do] TXN-0095 Transaction log fields for transactions v2
P0 · M6 · Task · deps: TXN-0031 · ref: transaction-coordinator/src/main/resources/common/message/TransactionLogValue.json @4.4.0
Done when: with TV_1+ records use the flexible value version with previous producer id, next producer id and client transaction version, and loading restores them; tests with golden records.

### [To Do] TXN-0096 Transactions v2 compatibility tests
P0 · M6 · Task · deps: TXN-0088, TXN-0091, TEST-F08 · labels: test
Done when: Java 4.4.0 transactional producers run commit, abort and fencing scenarios with `transaction.version=2` finalized and with level 1, and the results match Kafka.

## [To Do] TXN-F11 Last stable offset
P0 · M6 · Feature · deps: STOR-F06, TXN-F08
Scope: End-to-end read_committed flow: aborted-transaction lists in fetch responses and LSO-driven fetch wake-ups. LSO tracking and the transaction index are owned by STOR-F15 and STOR-F06.

### [To Do] TXN-0097 Track the first unstable offset and last stable offset
P0 · M6 · Task · deps: STOR-F15 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java firstUnstableOffset @4.4.0
Done when: the first unstable offset is the smallest first offset of ongoing transactions in producer state, and the last stable offset is `min(first unstable offset, high watermark)`; it is recomputed on append, marker, high watermark change, truncation and log start offset change; unit tests per trigger.

### [To Do] TXN-0098 Publish the last stable offset for readers
P0 · M6 · Task · deps: TXN-0097, REPL-F13 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java lastStableOffset @4.4.0
Done when: the LSO is published alongside the high watermark so read_committed fetches and ListOffsets with READ_COMMITTED use it as the upper bound without the writer lock; tests.

### [To Do] TXN-0099 Record aborted transactions in the transaction index
P0 · M6 · Task · deps: TXN-0097 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java completeTxn @4.4.0
Done when: an ABORT marker writes an entry (producer id, first offset, last offset, last stable offset) to the active segment's transaction index in completion order, and a COMMIT writes none; tests.

### [To Do] TXN-0100 Aborted transaction lookup for read_committed fetches
P0 · M6 · Task · deps: TXN-0099 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java collectAbortedTransactions @4.4.0
Done when: a query returns the aborted transactions overlapping a fetch range up to the fetch's upper bound, across segments, for REPL-F13 to place in Fetch responses; tests spanning segment boundaries.

### [To Do] TXN-0101 LSO advances complete delayed read_committed fetches
P0 · M6 · Task · deps: TXN-0098, REPL-F09
Done when: an LSO increase triggers completion checks for delayed read_committed fetches on that partition; test with a delayed fetch blocked on an open transaction.

### [To Do] TXN-0102 LSO on followers
P0 · M6 · Task · deps: TXN-0097 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java @4.4.0
Done when: followers rebuild producer state from replicated batches and markers and compute their LSO bounded by their high watermark, so read_committed fetch-from-follower is correct; tests.

### [To Do] TXN-0103 LSO after log recovery
P0 · M6 · Task · deps: TXN-0097, STOR-F09 · labels: test
Done when: after an unclean shutdown the recovered producer state yields the same ongoing transactions and LSO as before the crash; crash-consistency test.

### [To Do] TXN-0104 LSO invariants under random transactional workloads
P0 · M6 · Task · deps: TXN-0100, TXN-0102 · labels: test
Done when: a property test asserts LSO <= high watermark, LSO never decreases on a leader except by truncation, and read_committed reads never return records of open or aborted transactions.

### [To Do] TXN-0105 Last stable offset lag metric
P1 · M6 · Task · deps: TXN-0098, OBS-F04
Done when: a per-topic gauge equivalent to Kafka's last-stable-offset lag (high watermark minus LSO) is exported, with per-partition labels only when `alluvium.metrics.per.partition.enabled` is set; docs/metrics.md row added.

## [To Do] TXN-F12 Transactional id expiration
P1 · M6 · Feature · deps: TXN-F04
Scope: `transactional.id.expiration.ms`.

### [To Do] TXN-0106 Expire idle transactional ids
P1 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala removeExpiredTransactionalIds @4.4.0
Done when: every `transaction.remove.expired.transaction.cleanup.interval.ms` transactional ids in Empty, CompleteCommit or CompleteAbort with no pending transition and last update older than `transactional.id.expiration.ms` move to Dead; tests with the simulated clock.

### [To Do] TXN-0107 Write expiration tombstones in bounded batches
P1 · M6 · Task · deps: TXN-0106 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala writeTombstonesForExpiredTransactionalIds @4.4.0
Done when: tombstones are written per partition in batches bounded by the partition's maximum batch size, the cache entry is removed only after a successful append under the same coordinator epoch, and failures restore the previous state; tests.

### [To Do] TXN-0108 Start expiration only after startup loading
P1 · M6 · Task · deps: TXN-0106 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala enableTransactionalIdExpiration @4.4.0
Done when: the expiration scan is enabled after the coordinator starts and skips partitions still loading; test.

### [To Do] TXN-0109 Reuse of an expired transactional id
P1 · M6 · Task · deps: TXN-0107 · labels: test
Done when: InitProducerId for an expired id creates new metadata with a new producer id, and the old producer gets INVALID_PRODUCER_ID_MAPPING or PRODUCER_FENCED; integration test.

### [To Do] TXN-0110 Expiration events
P1 · M6 · Task · deps: TXN-0107, OBS-F07
Done when: a `txn_id_expired` event per batch reports partition and count, and a counter of expired transactional ids is exported and documented.

## [To Do] TXN-F13 Transaction administration
P1 · M6 · Feature · deps: TXN-F04
Scope: DescribeProducers, DescribeTransactions, ListTransactions filters, hanging-transaction abort.

### [To Do] TXN-0111 ListTransactions filters
P1 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala listTransactionStates @4.4.0
Done when: results cover all owned partitions; state filters with unknown names are echoed in `unknown_state_filters`; producer id filters apply; v1 duration filter returns transactions running longer than the given time; v2 transactional id pattern filters by regex (invalid pattern returns INVALID_REQUEST); a loading partition returns COORDINATOR_LOAD_IN_PROGRESS; tests per filter.

### [To Do] TXN-0112 DescribeTransactions
P1 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala handleDescribeTransactions @4.4.0
Done when: per transactional id: an inactive coordinator returns COORDINATOR_NOT_AVAILABLE, an empty id INVALID_REQUEST, an unowned id NOT_COORDINATOR, a loading partition COORDINATOR_LOAD_IN_PROGRESS, unknown or Dead TRANSACTIONAL_ID_NOT_FOUND; otherwise state, timeout, start time, producer id and epoch and partitions grouped by topic; tests.

### [To Do] TXN-0113 DescribeProducers at partition leaders
P1 · M6 · Task · deps: STOR-F15 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java activeProducers @4.4.0
Done when: the leader returns active producers with producer id, epoch, last sequence, last timestamp, coordinator epoch and current transaction start offset; non-leaders return NOT_LEADER_OR_FOLLOWER and unknown partitions UNKNOWN_TOPIC_OR_PARTITION; tests.

### [To Do] TXN-0114 Abort a hanging transaction with WriteTxnMarkers from an admin client
P1 · M6 · Task · deps: TXN-F08 · labels: test
Done when: an admin-sent ABORT marker with the producer's id, epoch and coordinator epoch (as `kafka-transactions.sh abort` sends) is appended even when the coordinator has no matching transaction, the LSO advances, and a stale coordinator epoch returns TRANSACTION_COORDINATOR_FENCED; integration test.

### [To Do] TXN-0115 Detect hanging transactions
P1 · M6 · Task · deps: TXN-0113, TXN-0111 · labels: test
Done when: a test creates an open transaction at a partition that the coordinator considers complete and shows the data returned by DescribeProducers, ListTransactions and DescribeTransactions identifies it the way `kafka-transactions.sh find-hanging` does.

### [To Do] TXN-0116 Operator guide for hanging transactions
P2 · M6 · Task · deps: TXN-0114 · labels: docs
Done when: a docs page explains detecting and aborting hanging transactions with `kafka-transactions.sh`, the errors involved and when verification makes them impossible.

## [To Do] TXN-F14 Two-phase commit
P3 · M6 · Feature · deps: TXN-F05
Scope: KIP-939 prepared transactions, if stable in 4.4.0.

### [To Do] TXN-0117 Decide the scope of two-phase commit support (KIP-939)
P3 · M6 · Spike · deps: none · ref: clients/src/main/resources/common/message/InitProducerIdRequest.json @4.4.0
Done when: ADR `two-phase-commit` accepted in docs/adr/ recording that InitProducerId v6 is `latestVersionUnstable` in 4.4.0 and `keepPreparedTxn` returns UNSUPPORTED_VERSION, and deciding which 4.4.0 behaviors Alluvium implements behind `unstable.api.versions.enable`.

### [To Do] TXN-0118 2PC enablement check
P3 · M6 · Task · deps: TXN-0117 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala handleInitProducerId @4.4.0
Done when: InitProducerId with `enable_2pc` returns TRANSACTIONAL_ID_AUTHORIZATION_FAILED when `transaction.two.phase.commit.enable` is false; tests.

### [To Do] TXN-0119 2PC transactions never time out
P3 · M6 · Task · deps: TXN-0118 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala validateTransactionTimeoutMs @4.4.0
Done when: with `enable_2pc` the timeout check is bypassed and the stored timeout is `i32::MAX`, so the timeout scan never aborts the transaction; tests.

### [To Do] TXN-0120 `keepPreparedTxn` returns UNSUPPORTED_VERSION
P3 · M6 · Task · deps: TXN-0118 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala handleInitProducerId @4.4.0
Done when: requests with `keep_prepared_txn` return UNSUPPORTED_VERSION as in 4.4.0; test.

### [To Do] TXN-0121 InitProducerId v6 is only reachable with unstable versions enabled
P3 · M6 · Task · deps: TXN-0120 · labels: test
Done when: a test shows v6 is not advertised by default and becomes usable only with `unstable.api.versions.enable=true`.

## [To Do] TXN-F15 Exactly-once verification
P0 · M6 · Feature · deps: TXN-F08, TEST-F06
Scope: Simulation and compatibility tests for EOS invariants, including Kafka Streams EOS.

### [To Do] TXN-0122 Simulated transactional producer and read_committed consumer
P0 · M6 · Task · deps: none · labels: test
Done when: testkit clients implement the transactional producer protocol (both TV1 and TV2) and a read_committed consumer as deterministic state machines.

### [To Do] TXN-0123 Invariant checker: transactions are atomic and aborted data is invisible
P0 · M6 · Task · deps: TXN-0122, TEST-F05 · labels: test
Done when: after each step the checker asserts read_committed consumers see all or none of each transaction's records across partitions, never see aborted records and never see duplicates.

### [To Do] TXN-0124 Invariant checker: zombie producers cannot commit
P0 · M6 · Task · deps: TXN-0122, TEST-F05 · labels: test
Done when: once a new producer instance has initialized, no record written by an older epoch of that transactional id becomes visible to read_committed consumers.

### [To Do] TXN-0125 EOS simulation under coordinator and leader faults
P0 · M6 · Task · deps: TXN-0123, TXN-0124 · labels: test
Done when: fault plans crash the coordinator and partition leaders at every transaction step, partition the network and crash producers; invariants hold over the CI seed set.

### [To Do] TXN-0126 Consume-transform-produce exactly-once simulation
P0 · M6 · Task · deps: TXN-0125, TXN-F07 · labels: test
Done when: a simulated application reads, writes and commits offsets in transactions under faults, and the output equals exactly one transformation of each input record.

### [To Do] TXN-0127 Kafka Streams exactly_once_v2 compatibility test
P0 · M6 · Task · deps: TEST-F08 · labels: test
Done when: a Kafka Streams 4.4.0 application with `processing.guarantee=exactly_once_v2` survives broker restarts and application crashes against Alluvium with verified exactly-once output.

### [To Do] TXN-0128 Transactional client compatibility tests
P0 · M6 · Task · deps: TEST-F08 · labels: test
Done when: the Java 4.4.0 transactional producer and librdkafka transactions pass commit, abort, timeout and fencing scenarios against Alluvium.

### [To Do] TXN-0129 Differential tests of transaction APIs against Kafka
P0 · M6 · Task · deps: TEST-F09 · labels: test
Done when: scripted InitProducerId, AddPartitionsToTxn, AddOffsetsToTxn, TxnOffsetCommit, EndTxn and WriteTxnMarkers sequences produce identical error codes on Kafka 4.4.0 and Alluvium for transaction versions 1 and 2.

### [To Do] TXN-0130 Seed management for EOS simulations
P1 · M6 · Task · deps: TXN-0125 · labels: test
Done when: EOS simulations run a fixed seed set plus fresh seeds in CI and thousands nightly, and failing seeds are added to the fixed set.

### [To Do] TXN-0131 Port Kafka's producer state and transaction manager tests
P1 · M6 · Task · deps: TXN-0125, TEST-F10 · labels: test · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java @4.4.0
Done when: cases from `ProducerStateManagerTest`, `TransactionStateManagerTest` and `TransactionCoordinatorTest` in the TEST-F10 inventory are ported with attributions and pass.

## [To Do] TXN-F16 Transaction metrics and events
P1 · M6 · Feature · deps: TXN-F04, OBS-F04
Scope: Transaction state counts, marker latencies, events with reasons.

### [To Do] TXN-0132 Transaction events with reasons
P1 · M6 · Task · deps: OBS-F07 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala @4.4.0
Done when: the driver emits `txn_state_change` (transactional_id, producer_id, producer_epoch, old and new state, reason: client_add_partitions, client_end_txn, timeout_abort, epoch_fence, init_producer_id, expiration), `txn_markers_completed` (elapsed_ms) and `producer_fenced` events, listed in `events.rs`.

### [To Do] TXN-0133 Transaction coordinator flight recorders
P1 · M6 · Task · deps: TXN-0132, OBS-F06
Done when: each transaction coordinator partition keeps a bounded flight recorder of transitions that is dumped on coordinator errors and shown by the diagnostics endpoint.

### [To Do] TXN-0134 `/debug/transactions/{transactional_id}` diagnostics
P2 · M6 · Task · deps: TXN-0133, OBS-F08
Done when: the endpoint returns the transaction's metadata, pending state and pending markers for locally coordinated ids, the debuggability standard's route table lists it, and unknown ids return 404.

### [To Do] TXN-0135 Transaction coordinator metrics
P1 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionMarkerChannelManager.scala @4.4.0
Done when: equivalents of partition-load-time-avg/max, UnknownDestinationQueueSize and LogAppendRetryQueueSize, plus gauges of transactions per state and a marker round-trip latency histogram, are exported with docs/metrics.md rows.

### [To Do] TXN-0136 Transaction verification metrics
P1 · M6 · Task · deps: none · ref: server/src/main/java/org/apache/kafka/server/transaction/AddPartitionsToTxnManager.java @4.4.0
Done when: equivalents of VerificationTimeMs and VerificationFailureRate are exported with docs/metrics.md rows; tests assert updates on success and failure.

### [To Do] TXN-0137 Standard tracing fields for transaction work
P1 · M6 · Task · deps: OBS-F03
Done when: coordinator and verification spans carry `transactional_id`, `producer_id` and `producer_epoch`; the field check passes.
