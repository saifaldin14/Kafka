# REPL: Replication and partitions

> Crates: `alluvium-replication` (layer 2), drivers in `alluvium-server` · Kafka counterpart: `cluster/Partition.scala`, `server/ReplicaManager.scala`, `ReplicaFetcherThread.scala` · Milestones: M2-M7

Partition leader and follower state, high watermark, ISR management, follower fetching, delayed produce and fetch, fetch sessions, reads, offset resolution and replication throttling.

## [To Do] REPL-F01 Partition ownership model
P0 · M2 · Feature · deps: STOR-F08
Scope: Spike and ADR: actor per partition vs sharded locks with published offsets; implementation.

### [To Do] REPL-0001 Spike: partition ownership model (actor per partition vs sharded locks with published offsets)
P0 · M2 · Spike · deps: none · labels: docs, perf · ref: core/src/main/scala/kafka/cluster/Partition.scala @4.4.0; core/src/main/scala/kafka/server/ReplicaManager.scala @4.4.0
Done when: ADR 'Partition ownership model' is accepted in docs/adr/ comparing an actor task per partition with a bounded mailbox against sharded locks with lock-free published offsets on produce latency, fetch fan-out, memory at 100k partitions, determinism under simulation and deadlock freedom, backed by prototype benchmark numbers; docs/architecture.md section 6 and the section 10 open decision are updated.

### [To Do] REPL-0002 Published partition offsets readable without the writer
P0 · M2 · Task · deps: REPL-0001 · labels: perf · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogOffsetSnapshot.java @4.4.0; core/src/main/scala/kafka/cluster/Partition.scala fetchOffsetSnapshot @4.4.0
Done when: High watermark, last stable offset, log end offset (with segment position) and log start offset are published as a consistent snapshot that fetch and ListOffsets read without blocking the writer; readers never observe a torn snapshot or a regressing high watermark in a multi-threaded stress test with a fixed seed.

### [To Do] REPL-0003 Single-writer partition owner per the ADR
P0 · M2 · Task · deps: REPL-0002 · ref: core/src/main/scala/kafka/cluster/Partition.scala leaderIsrUpdateLock, tryCompleteDelayedRequests @4.4.0
Done when: All mutations of a partition's log and replication state go through one owner (actor mailbox or lock as decided); the mailbox/lock is bounded with explicit backpressure; completion signals (HW advanced, LEO advanced, leadership lost) are emitted after the critical section; tests show no mutation path bypasses the owner.

### [To Do] REPL-0004 Hosted partition registry and lookup errors
P0 · M2 · Task · deps: REPL-0003 · ref: server/src/main/java/org/apache/kafka/server/HostedPartition.java @4.4.0; core/src/main/scala/kafka/server/ReplicaManager.scala getPartitionOrError, getPartitionOrException @4.4.0
Done when: The registry maps topic-partition to None, Online(partition) or Offline; lookups return KAFKA_STORAGE_ERROR for offline partitions, NOT_LEADER_OR_FOLLOWER when absent but known in metadata, UNKNOWN_TOPIC_OR_PARTITION otherwise; iteration is deterministic; tests.

### [To Do] REPL-0005 Contention benchmark for the ownership model
P1 · M2 · Task · deps: REPL-0003 · labels: perf
Done when: Criterion and macro benchmarks for concurrent produce and fetch across 1, 100 and 10000 partitions on the chosen model run in CI with regression thresholds; results are recorded in docs/nfr.md.

## [To Do] REPL-F02 Partition state
P0 · M2 · Feature · deps: REPL-F01
Scope: Leader/follower state, leader and partition epochs, replicas, ISR, maximal ISR.

### [To Do] REPL-0006 Partition data model: epochs, assignment state and leader epoch start offset
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/cluster/Partition.scala @4.4.0; server/src/main/java/org/apache/kafka/server/partition/SimpleAssignmentState.java @4.4.0; server/src/main/java/org/apache/kafka/server/partition/OngoingReassignmentState.java @4.4.0
Done when: The partition holds leader id, leader epoch, partition epoch, leader epoch start offset (leader only), assignment state (simple, or ongoing reassignment with adding and removing replicas) and the local log; module docs describe the invariants; unit tests.

### [To Do] REPL-0007 ISR state: committed, pending expand and pending shrink with the maximal ISR
P0 · M2 · Task · deps: REPL-0006 · ref: server/src/main/java/org/apache/kafka/server/partition/CommittedPartitionState.java @4.4.0; server/src/main/java/org/apache/kafka/server/partition/PendingExpandIsr.java @4.4.0; server/src/main/java/org/apache/kafka/server/partition/PendingShrinkIsr.java @4.4.0
Done when: Committed state carries the ISR and leader recovery state; pending expand adds the new replica to the maximal ISR; pending shrink keeps the current ISR as maximal; is_inflight is true for pending states, which keep the last committed state for reset; unit tests for isr and maximal_isr of each state.

### [To Do] REPL-0008 Remote replica state and follower fetch-state updates
P0 · M2 · Task · deps: REPL-0006, META-F09 · ref: server/src/main/java/org/apache/kafka/server/replica/Replica.java updateFetchStateOrThrow @4.4.0; server/src/main/java/org/apache/kafka/server/replica/ReplicaState.java @4.4.0
Done when: Each remote replica tracks log start offset, log end offset metadata, last fetch leader end offset, last fetch time, last caught-up time and broker epoch; an update with a broker epoch older than the image's fails NOT_LEADER_OR_FOLLOWER; last caught-up time follows Kafka's rule (fetch >= leader end: fetch time; fetch >= previous leader end: previous fetch time); tests per branch.

### [To Do] REPL-0009 Caught-up definition with replica.lag.time.max.ms
P0 · M2 · Task · deps: REPL-0008 · ref: server/src/main/java/org/apache/kafka/server/replica/ReplicaState.java isCaughtUp @4.4.0
Done when: A replica is caught up when its log end offset equals the leader's or it was caught up within replica.lag.time.max.ms; tests with the simulated clock.

### [To Do] REPL-0010 Replica state reset on leader election or re-election
P0 · M2 · Task · deps: REPL-0008 · ref: server/src/main/java/org/apache/kafka/server/replica/Replica.java resetReplicaState @4.4.0
Done when: On a new leader epoch, follower states reset: last caught-up time is now for ISR members else 0; a new leader clears offsets; a re-elected leader keeps offsets and sets last fetch time to now for ISR members else 0; tests.

### [To Do] REPL-0011 Local log creation on state changes with the checkpointed high watermark
P0 · M2 · Task · deps: REPL-0006, STOR-F10 · ref: core/src/main/scala/kafka/cluster/Partition.scala createLogIfNotExists, createLog @4.4.0
Done when: Becoming leader or follower creates the log if missing, initializing the high watermark from the replication offset checkpoint clamped to [log start, log end]; a log whose topic id differs from the partition's fails with INCONSISTENT_TOPIC_ID; tests.

### [To Do] REPL-0012 make_leader: partition epoch check, epoch start offset, replica reset
P0 · M2 · Task · deps: REPL-0007, REPL-0010, REPL-0011, STOR-F11 · ref: core/src/main/scala/kafka/cluster/Partition.scala makeLeader, updateAssignmentAndIsr @4.4.0
Done when: A registration with an older partition epoch is skipped; otherwise assignment and ISR are updated, RECOVERING becomes RECOVERED, and on a new leader epoch the epoch start offset is the log end offset, assigned in the leader epoch cache, and replicas reset; the leader epoch and partition epoch are stored; the transition result says whether this is a new leader and whether the HW moved; tests ported from PartitionTest.

### [To Do] REPL-0013 make_follower: partition epoch check, cleared ISR and fetch restart signal
P0 · M2 · Task · deps: REPL-0007, REPL-0011 · ref: core/src/main/scala/kafka/cluster/Partition.scala makeFollower @4.4.0
Done when: A registration with an older partition epoch is skipped; otherwise leader, leader epoch and partition epoch update first, the ISR is cleared and remote replicas dropped, and the result says whether the leader epoch changed (fetcher restart needed); tests ported from PartitionTest.

### [To Do] REPL-0014 Leader epoch checks for requests: FENCED_LEADER_EPOCH, UNKNOWN_LEADER_EPOCH, NOT_LEADER_OR_FOLLOWER
P0 · M2 · Task · deps: REPL-0006 · ref: core/src/main/scala/kafka/cluster/Partition.scala checkCurrentLeaderEpoch, localLogWithEpochOrThrow, getLocalLog @4.4.0
Done when: A request epoch below the local one fails FENCED_LEADER_EPOCH, above it UNKNOWN_LEADER_EPOCH; leader-only requests to a follower fail NOT_LEADER_OR_FOLLOWER; a missing log fails NOT_LEADER_OR_FOLLOWER; requests without an epoch skip the check; tests.

### [To Do] REPL-0015 Effective min ISR and under-replicated, under-min-ISR, at-min-ISR predicates
P0 · M2 · Task · deps: REPL-0007, CFG-F05 · ref: core/src/main/scala/kafka/cluster/Partition.scala effectiveMinIsr, isUnderMinIsr, isAtMinIsr, isUnderReplicated @4.4.0
Done when: Effective min ISR is min(min.insync.replicas, replica count); under-replicated means leader with ISR smaller than the replica count; under-min-ISR and at-min-ISR apply only to leaders; tests.

### [To Do] REPL-0016 Partition deletion and marking offline
P0 · M2 · Task · deps: REPL-0006 · ref: core/src/main/scala/kafka/cluster/Partition.scala delete, markOffline @4.4.0
Done when: delete clears the replication state and deletes the log through STOR; mark_offline drops the leader and log reference after a directory failure; both complete pending delayed operations as not-leader; tests.

### [To Do] REPL-0017 Property test: arbitrary make_leader/make_follower sequences keep epochs monotonic
P0 · M2 · Task · deps: REPL-0012, REPL-0013 · labels: test
Done when: Random registration sequences (including stale partition epochs) never decrease the leader or partition epoch and never leave a follower with a non-empty ISR; failures print a replayable seed.

## [To Do] REPL-F03 Leader append path
P0 · M2 · Feature · deps: REPL-F02
Scope: Leader check, min ISR check, append, high watermark update.

### [To Do] REPL-0018 Required acks validation
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/ReplicaManager.scala isValidRequiredAcks, sendInvalidRequiredAcksResponse @4.4.0
Done when: acks other than -1, 0 and 1 fail every partition with INVALID_REQUIRED_ACKS without appending; tests.

### [To Do] REPL-0019 Appends to internal topics require internal access
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/ReplicaManager.scala appendToLocalLog @4.4.0
Done when: Producing to __consumer_offsets, __transaction_state or other internal topics fails INVALID_TOPIC_EXCEPTION 'Cannot append to internal topic' unless the request is internal (coordinator writes or the admin client id); tests.

### [To Do] REPL-0020 Partition lookup for produce: leader required, offline and topic id errors
P0 · M2 · Task · deps: REPL-0004 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala getPartitionOrException @4.4.0; core/src/main/scala/kafka/cluster/Partition.scala appendRecordsToLeader @4.4.0
Done when: Non-leaders fail NOT_LEADER_OR_FOLLOWER, offline partitions KAFKA_STORAGE_ERROR, unknown partitions UNKNOWN_TOPIC_OR_PARTITION, and a produce topic id that does not match fails UNKNOWN_TOPIC_ID; tests.

### [To Do] REPL-0021 min.insync.replicas check before an acks=all append
P0 · M2 · Task · deps: REPL-0020 · ref: core/src/main/scala/kafka/cluster/Partition.scala appendRecordsToLeader @4.4.0
Done when: With acks=-1 an ISR smaller than the effective min ISR fails NOT_ENOUGH_REPLICAS before anything is appended; acks 0 and 1 skip the check; tests.

### [To Do] REPL-0022 Leader append in the leader epoch with high-watermark update and completion signals
P0 · M2 · Task · deps: REPL-0021, REC-F12 · ref: core/src/main/scala/kafka/cluster/Partition.scala appendRecordsToLeader @4.4.0; core/src/main/scala/kafka/server/ReplicaManager.scala appendRecordsToLeader @4.4.0
Done when: Validated records append as leader with the current leader epoch through STOR-F08 and REC-F12, the log end offset is published, the high watermark is recomputed, and HW/LEO-advanced signals trigger delayed produce and fetch completion; tests.

### [To Do] REPL-0023 Per-partition append error isolation and mapping
P0 · M2 · Task · deps: REPL-0022 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala appendToLocalLog, processFailedRecord @4.4.0
Done when: One partition's failure never affects others; record validation failures return INVALID_RECORD with per-record errors and message; CORRUPT_MESSAGE, MESSAGE_TOO_LARGE, RECORD_LIST_TOO_LARGE and KAFKA_STORAGE_ERROR map directly; unexpected errors return UNKNOWN_SERVER_ERROR and count as failed produce requests; error responses carry the log start offset; tests per error.

### [To Do] REPL-0024 Produce partition results and immediate completion for acks 0 and 1
P0 · M2 · Task · deps: REPL-0022 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala appendRecords, buildProducePartitionStatus @4.4.0
Done when: Results carry base offset, log append time (when the topic uses LogAppendTime, else -1) and log start offset; acks 0 and 1 respond after the local append; acks=all goes to delayed produce; tests.

### [To Do] REPL-0025 Port Kafka's leader append tests
P0 · M2 · Task · deps: REPL-0023, REPL-0024, REPL-0019, REPL-0018 · labels: test · ref: core/src/test/scala/unit/kafka/cluster/PartitionTest.scala @4.4.0
Done when: Append and min-ISR cases from PartitionTest and ReplicaManagerTest at 4.4.0 pass with attribution.

## [To Do] REPL-F04 High watermark
P0 · M2 · Feature · deps: REPL-F02
Scope: HW over the maximal ISR, caught-up replicas, `replica.lag.time.max.ms`, HW persistence.

### [To Do] REPL-0026 ISR eligibility from the metadata image
P0 · M2 · Task · deps: META-F09 · ref: core/src/main/scala/kafka/cluster/Partition.scala isReplicaIsrEligible, isBrokerEpochIsrEligible @4.4.0
Done when: A replica is ISR-eligible when it is registered, not fenced, not in controlled shutdown, and its last fetch broker epoch equals the image's broker epoch (or is -1); a deleted topic's missing replica is ineligible; tests.

### [To Do] REPL-0027 High watermark over the maximal ISR and caught-up eligible replicas
P0 · M2 · Task · deps: REPL-0026 · ref: core/src/main/scala/kafka/cluster/Partition.scala maybeIncrementLeaderHW @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java maybeIncrementHighWatermark @4.4.0
Done when: The new HW is the minimum log end offset over the leader, maximal-ISR members and caught-up ISR-eligible replicas (with offset metadata); it never decreases and only advances through the log's monotonic update; tests ported from PartitionTest.

### [To Do] REPL-0028 High watermark does not advance while the partition is under min ISR
P0 · M2 · Task · deps: REPL-0027 · ref: core/src/main/scala/kafka/cluster/Partition.scala maybeIncrementLeaderHW @4.4.0; KIP-966
Done when: While the ISR is smaller than the effective min ISR, the leader's HW does not move (Kafka 4.x behavior for KIP-966); it resumes once the ISR recovers; tests.

### [To Do] REPL-0029 HW recomputation triggers and delayed-operation completion outside the critical section
P0 · M2 · Task · deps: REPL-0027, SRV-F07 · ref: core/src/main/scala/kafka/cluster/Partition.scala tryCompleteDelayedRequests, updateFollowerFetchState @4.4.0
Done when: The HW is recomputed after a leader append, a follower log end offset change, an ISR commit and make_leader; when it advances, delayed produce and fetch for the partition are completed after the owner's critical section; tests.

### [To Do] REPL-0030 Periodic high-watermark checkpoint per log directory
P0 · M2 · Task · deps: REPL-0027, STOR-F10 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala checkpointHighWatermarks, startHighWatermarkCheckPointThread @4.4.0
Done when: Every replica.high.watermark.checkpoint.interval.ms (default 5000) and at shutdown, HWs of all online partitions are written to replication-offset-checkpoint per log directory through STOR-F10's atomic writer; crash tests show a restart never loads a HW above the log end; tests.

### [To Do] REPL-0031 Property test: leader HW invariants
P0 · M2 · Task · deps: REPL-0028, REPL-0029 · labels: test
Done when: Random sequences of appends, follower fetches, ISR changes and min ISR changes keep log start <= HW <= LEO, HW never decreasing on a leader, HW <= min LEO of the maximal ISR, and no advance under min ISR.

## [To Do] REPL-F05 ISR expand and shrink
P0 · M3 · Feature · deps: REPL-F04, CTRL-F09
Scope: AlterPartition requests, pending state, retries, rejection handling.

### [To Do] REPL-0032 ISR expansion: caught up past the HW and the leader epoch start
P0 · M3 · Task · deps: REPL-F07 · ref: core/src/main/scala/kafka/cluster/Partition.scala maybeExpandIsr, isFollowerInSync, prepareIsrExpand @4.4.0
Done when: When no ISR change is in flight, a non-ISR, ISR-eligible follower whose log end offset is >= the HW and >= the leader epoch start offset is proposed for the ISR (pending expand, so the HW already accounts for it); tests ported from PartitionTest.

### [To Do] REPL-0033 ISR shrink of stuck or slow followers on a schedule
P0 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/cluster/Partition.scala maybeShrinkIsr, getOutOfSyncReplicas, prepareIsrShrink @4.4.0; core/src/main/scala/kafka/server/ReplicaManager.scala maybeShrinkIsr @4.4.0
Done when: Every replica.lag.time.max.ms / 2 each leader partition with no in-flight change proposes removing ISR followers not caught up within replica.lag.time.max.ms (pending shrink keeps the current ISR as maximal); the shrink is logged with each replica's end offset and last caught-up time; tests.

### [To Do] REPL-0034 Broker epochs in proposed ISRs
P0 · M3 · Task · deps: REPL-0032 · ref: core/src/main/scala/kafka/cluster/Partition.scala addBrokerEpochToIsr @4.4.0
Done when: Proposed ISR entries carry the local broker epoch for the leader, the last fetch broker epoch for followers, and -1 when unknown; tests.

### [To Do] REPL-0035 AlterPartition sender: one pending update per partition, one request in flight, unbounded retry
P0 · M3 · Task · deps: REPL-0034, NET-F10 · ref: server/src/main/java/org/apache/kafka/server/partition/DefaultAlterPartitionManager.java @4.4.0
Done when: Submitting a second update for a partition with one pending fails OPERATION_NOT_ATTEMPTED; pending updates batch into a single in-flight AlterPartition (v3 with broker epochs) through NET-F10's controller channel without a request timeout; top-level errors retry with backoff, authentication failures are treated as NETWORK_EXCEPTION and retried, CLUSTER_AUTHORIZATION_FAILED fails all, partitions missing in a response fail UNKNOWN_SERVER_ERROR; tests.

### [To Do] REPL-0036 AlterPartition success handling
P0 · M3 · Task · deps: REPL-0035 · ref: core/src/main/scala/kafka/cluster/Partition.scala handleAlterPartitionUpdate @4.4.0
Done when: A response with another leader epoch or an older partition epoch is ignored (counted as failed); otherwise the ISR state becomes committed with the returned ISR and recovery state, the partition epoch updates, the expand/shrink signal fires and the HW is recomputed; tests.

### [To Do] REPL-0037 AlterPartition error handling per error code
P0 · M3 · Task · deps: REPL-0035 · ref: core/src/main/scala/kafka/cluster/Partition.scala handleAlterPartitionError @4.4.0
Done when: OPERATION_NOT_ATTEMPTED and INELIGIBLE_REPLICA reset to the last committed state; UNKNOWN_TOPIC_OR_PARTITION, UNKNOWN_TOPIC_ID, FENCED_LEADER_EPOCH, INVALID_UPDATE_VERSION, INVALID_REQUEST and NEW_LEADER_ELECTED stay pending until new metadata; any other error is retried; every failure counts as failed; tests per code.

### [To Do] REPL-0038 Ignore AlterPartition responses after the partition state changed
P0 · M3 · Task · deps: REPL-0036, REPL-0037 · ref: core/src/main/scala/kafka/cluster/Partition.scala submitAlterPartition @4.4.0
Done when: If metadata or another path changed the partition state since the request was sent, the response is ignored; retries are submitted outside the owner's critical section; tests.

### [To Do] REPL-0039 Port Kafka's ISR and AlterPartition tests
P0 · M3 · Task · deps: REPL-0038, REPL-0033 · labels: test · ref: server/src/test/java/org/apache/kafka/server/partition/AlterPartitionManagerTest.java @4.4.0
Done when: ISR expand/shrink cases from PartitionTest and AlterPartitionManagerTest at 4.4.0 pass with attribution.

## [To Do] REPL-F06 Follower fetcher
P0 · M3 · Feature · deps: REPL-F02, NET-F10 · labels: foundation-for:X-GEO
Scope: Fetcher equivalent: fetch sessions as client, backoff, truncation on diverging epoch, OffsetForLeaderEpoch fallback.

### [To Do] REPL-0040 Leader endpoint abstraction for fetchers
P0 · M3 · Task · deps: none · labels: foundation-for:X-GEO · ref: server/src/main/java/org/apache/kafka/server/LeaderEndPoint.java @4.4.0; core/src/main/scala/kafka/server/RemoteLeaderEndPoint.scala @4.4.0
Done when: A leader endpoint trait exposes fetch, earliest/latest offset lookups and epoch end offsets (OffsetForLeaderEpoch) and whether truncation on fetch is supported; the remote implementation uses NET-F10; a fake implementation drives tests.

### [To Do] REPL-0041 Fetcher manager: fetchers per (leader, fetcher id) and partition assignment
P0 · M3 · Task · deps: REPL-0040 · ref: core/src/main/scala/kafka/server/AbstractFetcherManager.scala @4.4.0; core/src/main/scala/kafka/server/ReplicaFetcherManager.scala @4.4.0
Done when: Partitions map to fetcher id hash(topic, partition) mod num.replica.fetchers per source broker; add/remove partitions, shut down idle fetchers, track failed partitions; every fetcher is a named supervised task; tests.

### [To Do] REPL-0042 Per-partition fetch state and initial fetch offset
P0 · M3 · Task · deps: REPL-0041 · ref: server/src/main/java/org/apache/kafka/server/PartitionFetchState.java @4.4.0; core/src/main/scala/kafka/server/ReplicaManager.scala initialFetchOffset @4.4.0
Done when: Fetch state holds topic id, fetch offset, lag, current leader epoch, delay-until, state (truncating or fetching) and last fetched epoch; a new follower starts at its log end offset when the log has epochs, else at its HW in truncating state; tests.

### [To Do] REPL-0043 Client-side incremental fetch session for followers
P0 · M3 · Task · deps: REPL-0042 · ref: clients/src/main/java/org/apache/kafka/clients/FetchSessionHandler.java @4.4.0
Done when: Followers build full or incremental requests with session id and epoch, sending only added, altered and removed partitions; FETCH_SESSION_ID_NOT_FOUND or INVALID_FETCH_SESSION_EPOCH resets to a full request; responses with unexpected partitions are rejected; tests.

### [To Do] REPL-0044 Follower fetch request construction
P0 · M3 · Task · deps: REPL-0043 · ref: core/src/main/scala/kafka/server/ReplicaFetcherThread.scala @4.4.0; core/src/main/scala/kafka/server/RemoteLeaderEndPoint.scala buildFetch @4.4.0
Done when: Requests carry replica id and broker epoch, replica.fetch.wait.max.ms, replica.fetch.min.bytes, replica.fetch.response.max.bytes, per-partition replica.fetch.max.bytes, current leader epoch, last fetched epoch and log start offset, skipping delayed, truncating and throttled partitions; golden test of the encoded request.

### [To Do] REPL-0045 Processing fetched data: append, follower HW, log start offset and lag
P0 · M3 · Task · deps: REPL-0044 · ref: core/src/main/scala/kafka/server/AbstractFetcherThread.scala processFetchRequest @4.4.0; core/src/main/scala/kafka/server/ReplicaFetcherThread.scala processPartitionData @4.4.0
Done when: Records append as follower using the leader epoch of the request; the follower HW becomes min(leader HW, local log end offset); the log start offset advances to the leader's (reason: leader offset incremented); lag and next fetch offset update; responses for partitions whose offset or epoch changed meanwhile are dropped; tests.

### [To Do] REPL-0046 First follower append below the log start offset restarts the log at the batch base
P0 · M3 · Task · deps: REPL-0045 · ref: core/src/main/scala/kafka/cluster/Partition.scala appendRecordsToFollowerOrFutureReplica @4.4.0
Done when: When a follower's log is empty at its log start offset and the first fetched batch starts below it but ends at or after it (leader log start inside a batch after DeleteRecords), the follower truncates fully to the batch's first offset and retries the append; any other unexpected append offset is an error; tests.

### [To Do] REPL-0047 Truncation on a diverging epoch in the fetch response
P0 · M3 · Task · deps: REPL-0045, STOR-F12 · ref: core/src/main/scala/kafka/server/AbstractFetcherThread.scala truncateOnFetchResponse, truncateToEpochEndOffsets @4.4.0
Done when: A diverging epoch truncates the follower to min(diverging end offset, local end offset for that epoch) without processing the data, then fetching resumes; tests ported from AbstractFetcherThreadTest.

### [To Do] REPL-0048 OffsetForLeaderEpoch truncation fallback
P0 · M3 · Task · deps: REPL-0047 · ref: core/src/main/scala/kafka/server/AbstractFetcherThread.scala maybeTruncate, getOffsetTruncationState, fetchTruncatingPartitions @4.4.0
Done when: Partitions in truncating state request epoch end offsets: an undefined end offset truncates to the initial fetch offset; an undefined epoch truncates to min(leader offset, local end); an unknown epoch truncates to the end of the largest smaller local epoch and asks again; otherwise truncates to min(leader end, local end for that epoch); FENCED and UNKNOWN epoch responses back off; tests per branch.

### [To Do] REPL-0049 OFFSET_OUT_OF_RANGE handling for followers
P0 · M3 · Task · deps: REPL-0045, STOR-F12 · ref: core/src/main/scala/kafka/server/AbstractFetcherThread.scala handleOutOfRangeError, fetchOffsetAndTruncate @4.4.0
Done when: If the leader's log end offset is below the follower's, the follower truncates to it (logged at warn as a possible unclean election); if the follower is below the leader's log start offset, it truncates fully and restarts at the leader's log start offset; tests.

### [To Do] REPL-0050 Per-partition fetch errors and backoff
P0 · M3 · Task · deps: REPL-0045 · ref: core/src/main/scala/kafka/server/AbstractFetcherThread.scala processFetchRequest, onPartitionFenced, markPartitionFailed, handlePartitionsWithErrors @4.4.0
Done when: UNKNOWN_LEADER_EPOCH, NOT_LEADER_OR_FOLLOWER, UNKNOWN_TOPIC_OR_PARTITION, UNKNOWN_TOPIC_ID, INCONSISTENT_TOPIC_ID and corrupt records delay the partition by replica.fetch.backoff.ms; FENCED_LEADER_EPOCH with the same epoch marks the partition failed, with a newer epoch retries; storage errors and unexpected errors mark it failed; a failed whole request backs off all partitions; tests per error.

### [To Do] REPL-0051 Topic id updates for fetching partitions
P0 · M3 · Task · deps: REPL-0042 · ref: core/src/main/scala/kafka/server/AbstractFetcherThread.scala maybeUpdateTopicIds @4.4.0
Done when: When a partition's topic id becomes known or changes in metadata, fetch states update without restarting the fetcher; mismatches are handled as INCONSISTENT_TOPIC_ID; tests.

### [To Do] REPL-0052 Port Kafka's fetcher thread tests
P0 · M3 · Task · deps: REPL-0048, REPL-0049, REPL-0050, REPL-0051 · labels: test · ref: core/src/test/scala/unit/kafka/server/AbstractFetcherThreadTest.scala @4.4.0; core/src/test/scala/unit/kafka/server/ReplicaFetcherThreadTest.scala @4.4.0
Done when: Cases from AbstractFetcherThreadTest and ReplicaFetcherThreadTest at 4.4.0 pass with attribution using the fake leader endpoint.

### [To Do] REPL-0053 Follower replication throughput benchmark
P2 · M3 · Task · deps: REPL-0045 · labels: perf
Done when: A benchmark measures follower fetch-and-append throughput and replication latency for 1, 100 and 1000 partitions; results recorded with regression thresholds.

## [To Do] REPL-F07 Leader handling of follower fetches
P0 · M3 · Feature · deps: REPL-F04
Scope: Replica LEO and caught-up time updates, HW advancement trigger.

### [To Do] REPL-0054 Follower fetch validation on the leader
P0 · M3 · Task · deps: REPL-F13 · ref: core/src/main/scala/kafka/cluster/Partition.scala fetchRecords, followerReplicaOrThrow @4.4.0
Done when: A follower fetch uses the leader epoch check; a replica id not in the assignment fails UNKNOWN_LEADER_EPOCH when the request has a leader epoch and NOT_LEADER_OR_FOLLOWER otherwise; tests.

### [To Do] REPL-0055 Diverging epoch detection for fetches with a last fetched epoch
P0 · M3 · Task · deps: REPL-0054 · ref: core/src/main/scala/kafka/cluster/Partition.scala readRecords, lastOffsetForLeaderEpoch @4.4.0
Done when: With a last fetched epoch, the leader looks up its end offset for that epoch: an undefined result or a fetch offset below the log start fails OFFSET_OUT_OF_RANGE; an epoch or end offset smaller than requested returns a diverging epoch and no records; tests ported from PartitionTest.

### [To Do] REPL-0056 Follower fetch state update, HW advance and completion triggers
P0 · M3 · Task · deps: REPL-0055 · ref: core/src/main/scala/kafka/cluster/Partition.scala updateFollowerFetchState, fetchRecords @4.4.0
Done when: A fetch without divergence updates the replica's fetch state with the leader end offset observed before the read, attempts ISR expansion, recomputes the HW when the follower's end offset moved, and triggers delayed operations when HW or low watermark moved; fetches served from purgatory do not update state; tests.

### [To Do] REPL-0057 OffsetForLeaderEpoch on the leader
P0 · M3 · Task · deps: REPL-0054 · ref: core/src/main/scala/kafka/cluster/Partition.scala lastOffsetForLeaderEpoch @4.4.0; core/src/main/scala/kafka/server/ReplicaManager.scala lastOffsetForLeaderEpoch @4.4.0
Done when: end_offset_for_epoch returns the largest epoch <= requested and its end offset (next epoch's start, or log end for the latest), with epoch validation errors and UNDEFINED values when unknown; tests.

### [To Do] REPL-0058 Reassignment fetch flag for adding replicas
P2 · M3 · Task · deps: REPL-0056 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala isAddingReplica, fetchMessages @4.4.0
Done when: Fetches from replicas being added by a reassignment are marked as reassignment fetches so their bytes are reported separately; tests.

### [To Do] REPL-0059 Low watermark on the leader and DeleteRecords on the leader
P0 · M7 · Task · deps: REPL-0056, STOR-F13 · ref: core/src/main/scala/kafka/cluster/Partition.scala lowWatermarkIfLeader, deleteRecordsOnLeader @4.4.0
Done when: The low watermark is the minimum log start offset over the leader and live followers; delete_records_on_leader rejects compact-only topics with POLICY_VIOLATION, maps -1 to the HW, rejects negative offsets with OFFSET_OUT_OF_RANGE, advances the log start offset (reason: client record deletion) and returns the low watermark; non-leaders fail NOT_LEADER_OR_FOLLOWER; tests.

### [To Do] REPL-0060 Delayed DeleteRecords completes when the low watermark reaches the target
P0 · M7 · Task · deps: REPL-0059, SRV-F07 · ref: server/src/main/java/org/apache/kafka/server/purgatory/DelayedDeleteRecords.java @4.4.0; core/src/main/scala/kafka/server/ReplicaManager.scala deleteRecords @4.4.0
Done when: A DeleteRecords request waits until every partition's low watermark reaches its requested offset, completing on follower fetches that raise the low watermark, failing NOT_LEADER_OR_FOLLOWER on leadership loss and REQUEST_TIMED_OUT on expiry; tests with the simulated clock.

### [To Do] REPL-0061 Port Kafka's follower fetch handling tests
P0 · M3 · Task · deps: REPL-0056, REPL-0057 · labels: test · ref: core/src/test/scala/unit/kafka/server/ReplicaManagerTest.scala @4.4.0
Done when: Follower fetch cases from PartitionTest and ReplicaManagerTest at 4.4.0 pass with attribution.

## [To Do] REPL-F08 Delayed produce
P0 · M2 · Feature · deps: SRV-F07, REPL-F03
Scope: acks=all completion when the HW reaches the required offsets, or timeout.

### [To Do] REPL-0062 When produce needs a delayed operation
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/ReplicaManager.scala delayedProduceRequestRequired, buildProducePartitionStatus @4.4.0
Done when: A delayed produce is created only for acks=-1 with at least one partition appended successfully; otherwise the response is sent at once; each partition's required offset is last offset + 1; tests.

### [To Do] REPL-0063 Completion check: HW reached and maximal ISR at or above min ISR
P0 · M2 · Task · deps: REPL-0062 · ref: core/src/main/scala/kafka/cluster/Partition.scala checkEnoughReplicasReachOffset @4.4.0; server/src/main/java/org/apache/kafka/server/purgatory/DelayedProduce.java tryComplete @4.4.0
Done when: A partition is done when the leader HW >= required offset, with NOT_ENOUGH_REPLICAS_AFTER_APPEND if the maximal ISR is then smaller than the effective min ISR; a partition no longer led locally fails NOT_LEADER_OR_FOLLOWER; tests.

### [To Do] REPL-0064 Delayed produce watch keys, completion triggers and expiry
P0 · M2 · Task · deps: REPL-0063 · ref: server/src/main/java/org/apache/kafka/server/purgatory/DelayedProduce.java onExpiration @4.4.0; core/src/main/scala/kafka/server/ReplicaManager.scala completeDelayedOperationsWhenNotPartitionLeader @4.4.0
Done when: Delayed produce watches each partition key, is re-checked on HW advance and leadership loss, and expires after the request timeout with REQUEST_TIMED_OUT for partitions still pending; tests with the simulated clock.

### [To Do] REPL-0065 Delayed produce expiration metrics
P1 · M2 · Task · deps: REPL-0064, OBS-F04 · labels: observability · ref: server/src/main/java/org/apache/kafka/server/purgatory/DelayedProduce.java @4.4.0
Done when: Expirations are counted in total and per topic-partition (DelayedProduceMetrics ExpiresPerSec, partition label only with alluvium.metrics.per.partition.enabled) and per-partition series are removed when leadership moves; tests.

### [To Do] REPL-0066 Port Kafka's delayed produce tests
P0 · M2 · Task · deps: REPL-0064 · labels: test · ref: server/src/test/java/org/apache/kafka/server/purgatory/DelayedProduceTest.java @4.4.0
Done when: Cases from DelayedProduceTest at 4.4.0 pass with attribution.

## [To Do] REPL-F09 Delayed fetch
P0 · M2 · Feature · deps: SRV-F07, REPL-F13
Scope: min bytes, max wait, completion on new data or HW movement.

### [To Do] REPL-0067 Respond immediately or create a delayed fetch
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/ReplicaManager.scala fetchMessages @4.4.0
Done when: A fetch is answered at once when max_wait_ms <= 0, there are no partitions, readable bytes >= min_bytes, any partition errored, a diverging epoch or a preferred read replica was found; otherwise a delayed fetch is created with each partition's fetch offset metadata; tests per condition.

### [To Do] REPL-0068 Delayed fetch completion cases A-H
P0 · M2 · Task · deps: REPL-0067 · ref: core/src/main/scala/kafka/server/DelayedFetch.scala tryComplete @4.4.0
Done when: The delayed fetch completes when a partition is no longer led or hosted, unknown, offline, fenced by epoch, the fetch offset is past the end or on an older segment (unless throttled), a diverging epoch appeared, or accumulated bytes (capped per partition, excluding throttled partitions) reach min_bytes; the end offset is LEO, HW or LSO per isolation; tests per case.

### [To Do] REPL-0069 Delayed fetch completion re-reads without updating follower state
P0 · M2 · Task · deps: REPL-0068 · ref: core/src/main/scala/kafka/server/DelayedFetch.scala onComplete @4.4.0; core/src/main/scala/kafka/server/ReplicaManager.scala readFromLog @4.4.0
Done when: On completion or expiry the fetch reads again from the log (read-from-purgatory mode, no follower state update) and responds; tests.

### [To Do] REPL-0070 Delayed fetch triggers: appends, HW advance and leadership loss
P0 · M2 · Task · deps: REPL-0068, REPL-F04 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala completeDelayedFetchRequests @4.4.0; core/src/main/scala/kafka/cluster/Partition.scala tryCompleteDelayedRequests @4.4.0
Done when: Follower fetches are re-checked on leader appends, consumer fetches on HW advance, and all on leadership changes; watch keys are per partition; tests.

### [To Do] REPL-0071 Delayed fetch expiration metrics for followers and consumers
P1 · M2 · Task · deps: REPL-0069, OBS-F04 · labels: observability · ref: core/src/main/scala/kafka/server/DelayedFetch.scala onExpiration @4.4.0
Done when: Expirations are counted separately for follower and consumer fetches (DelayedFetchMetrics ExpiresPerSec with fetcherType); tests.

## [To Do] REPL-F10 Fetch sessions
P0 · M2 · Feature · deps: REPL-F13
Scope: KIP-227 incremental fetch sessions: cache, eviction, slots, epochs.

### [To Do] REPL-0072 Fetch contexts: full, incremental, sessionless and session error
P0 · M2 · Task · deps: none · ref: server/src/main/java/org/apache/kafka/server/FetchManager.java newContext @4.4.0; server/src/main/java/org/apache/kafka/server/FetchContext.java @4.4.0
Done when: A full request with a session id closes that session; epoch FINAL with a full request is sessionless; a full request otherwise creates a session when a slot is available (else sessionless); an incremental request with an unknown session fails FETCH_SESSION_ID_NOT_FOUND, a wrong epoch INVALID_FETCH_SESSION_EPOCH, a topic-id usage change across versions 13 FETCH_SESSION_TOPIC_ID_ERROR; FINAL on an incremental request removes the session; tests per path.

### [To Do] REPL-0073 Cached partitions: add, alter and forget across incremental requests
P0 · M2 · Task · deps: REPL-0072 · ref: server/src/main/java/org/apache/kafka/server/FetchSession.java update, CachedPartition @4.4.0
Done when: Incremental requests add or update partitions (fetch offset, max bytes, epochs, log start) and remove forgotten topics; the session epoch increments per request; partitions are keyed by topic id (v13+) or name; tests.

### [To Do] REPL-0074 Incremental responses include only changed partitions
P0 · M2 · Task · deps: REPL-0073 · ref: server/src/main/java/org/apache/kafka/server/FetchSession.java maybeUpdateResponseData @4.4.0
Done when: A partition is included when it has records, its HW or log start offset changed, it has a preferred read replica, an error (resetting cached HW to -1) or a diverging epoch; full responses include all; tests.

### [To Do] REPL-0075 Unknown topic ids in sessions
P0 · M2 · Task · deps: REPL-0073 · ref: server/src/main/java/org/apache/kafka/server/FetchContext.java @4.4.0
Done when: Partitions whose topic id is unknown are answered with UNKNOWN_TOPIC_ID per partition (or INCONSISTENT_TOPIC_ID when it conflicts with the log) without breaking the session; tests.

### [To Do] REPL-0076 Sharded session cache with slot limit and eviction
P0 · M2 · Task · deps: REPL-0072 · ref: server/src/main/java/org/apache/kafka/server/FetchSessionCacheShard.java tryEvict, maybeCreateSession @4.4.0
Done when: The cache is split into shards with disjoint session id ranges; total slots are max.incremental.fetch.session.cache.slots (default 1000); a new session may evict a stale session (unused past the eviction time), else an unprivileged one when the newcomer is privileged (follower) or has more partitions and the victim is not recently created; session ids come from the injected RNG; tests.

### [To Do] REPL-0077 Bounded session memory
P1 · M2 · Task · deps: REPL-0076 · labels: nfr
Done when: Cached partitions per session are bounded by the request's partition count and total cached partitions are accounted; a test with the maximum sessions and partitions stays within the documented memory budget.

### [To Do] REPL-0078 Fetch session metrics
P1 · M2 · Task · deps: REPL-0076, OBS-F04 · labels: observability · ref: server/src/main/java/org/apache/kafka/server/FetchSession.java @4.4.0
Done when: NumIncrementalFetchSessions, NumIncrementalFetchPartitionsCached and IncrementalFetchSessionEvictionsPerSec are exported; tests.

### [To Do] REPL-0079 Port Kafka's fetch session tests
P0 · M2 · Task · deps: REPL-0074, REPL-0075, REPL-0076 · labels: test · ref: server/src/test/java/org/apache/kafka/server/FetchSessionTest.java @4.4.0
Done when: Cases from FetchSessionTest at 4.4.0 pass with attribution.

## [To Do] REPL-F11 Fetch from follower
P1 · M3 · Feature · deps: REPL-F06
Scope: KIP-392 rack-aware replica selector and preferred read replica.

### [To Do] REPL-0080 Replica selector: leader-only default and rack-aware selector
P1 · M3 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/replica/RackAwareReplicaSelector.java select @4.4.0; clients/src/main/java/org/apache/kafka/common/replica/ReplicaSelector.java @4.4.0
Done when: replica.selector.class accepts Kafka's LeaderSelector (default) and RackAwareReplicaSelector class names mapped to built-in selectors (other names are refused at startup); the rack-aware selector picks the leader when the client has no rack or the leader shares it, else the most caught-up same-rack replica, else the leader; tests ported from RackAwareReplicaSelectorTest.

### [To Do] REPL-0081 Preferred read replica computation on the leader
P1 · M3 · Task · deps: REPL-0080 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala findPreferredReadReplica, readFromLog @4.4.0
Done when: Only for consumer fetches with client metadata (v11+): candidates are ISR followers whose log covers the fetch offset plus the leader; a selection other than the leader is returned as preferred_read_replica with no records and offsets from the snapshot; follower fetches never get one; tests.

### [To Do] REPL-0082 Followers serve consumer fetches up to their high watermark
P1 · M3 · Task · deps: REPL-0081 · ref: core/src/main/scala/kafka/cluster/Partition.scala fetchRecords @4.4.0; core/src/main/scala/kafka/server/ReplicaManager.scala readFromLog @4.4.0
Done when: Consumer fetches (v11+) to a follower are served up to the follower's HW (LSO for read_committed); offsets between HW and LEO return no records; beyond LEO fail OFFSET_OUT_OF_RANGE; fetches requiring the leader still fail NOT_LEADER_OR_FOLLOWER; tests.

### [To Do] REPL-0083 Followers complete delayed consumer fetches when their HW advances
P1 · M3 · Task · deps: REPL-0082 · ref: core/src/main/scala/kafka/server/ReplicaFetcherThread.scala completeDelayedFetchRequests @4.4.0
Done when: After a follower appends and its HW moves, delayed consumer fetches on that partition are re-checked; tests.

### [To Do] REPL-0084 Fetch-from-follower compatibility test with client.rack
P1 · M3 · Task · deps: REPL-0083, TEST-F08 · labels: test
Done when: A Java consumer with client.rack fetches from a same-rack follower in a 3-broker cluster and receives the same records as from the leader; compatibility test.

## [To Do] REPL-F12 Replica manager
P0 · M2 · Feature · deps: REPL-F02, SRV-F14
Scope: Apply metadata deltas: become leader or follower, stop and delete replicas.

### [To Do] REPL-0085 Local replica changes from a topics delta
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/image/TopicsDelta.java localChanges @4.4.0; core/src/main/scala/kafka/server/ReplicaManager.scala applyDelta @4.4.0
Done when: From a metadata delta compute this broker's deletes, new or changed leaders and followers, and directory assignments, using the delta API of META-F03; tests with deltas covering each change type.

### [To Do] REPL-0086 Get or create partitions for deltas
P0 · M2 · Task · deps: REPL-0085 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala getOrCreatePartition @4.4.0
Done when: An offline partition with the same topic id stays offline (warn); one with another id is replaced by a new partition; an online partition with a different topic id is a fatal invariant violation; a missing one is created (error log if the image already had it); tests.

### [To Do] REPL-0087 Apply deletions first: stop replicas and delete logs
P0 · M2 · Task · deps: REPL-0085, STOR-F16 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala applyDelta, stopPartitions @4.4.0
Done when: Deleted partitions are stopped (fetchers removed, delayed operations completed, metrics removed) and their logs deleted asynchronously through STOR before any creation in the same delta; deletion of a topic led locally also requests remote log deletion when tiering applies; storage errors are logged without stopping others; tests.

### [To Do] REPL-0088 Apply leader changes
P0 · M2 · Task · deps: REPL-0086 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala applyLocalLeadersDelta @4.4.0
Done when: For new leaders: remove fetchers, get or create the partition, run make_leader; storage errors mark the partition offline; tests.

### [To Do] REPL-0089 Apply follower changes and restart fetchers on leader epoch change
P0 · M3 · Task · deps: REPL-0086, REPL-F06 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala applyLocalFollowersDelta @4.4.0
Done when: For followers: run make_follower; during controlled shutdown, partitions with no leader or without this broker in the ISR are stopped; on a new leader epoch fetchers are removed then re-added from the initial fetch offset toward the leader's inter-broker endpoint (skipped when the leader is not alive), delayed operations completed as not-leader; failures mark the partition failed or offline; tests.

### [To Do] REPL-0090 Apply follower changes without fetchers on a single node
P0 · M2 · Task · deps: REPL-0086 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala applyLocalFollowersDelta @4.4.0
Done when: On a single-broker cluster follower transitions (for example leadership moving away during shutdown) run make_follower and complete delayed operations as not-leader; the fetcher restart path is added in M3; tests.

### [To Do] REPL-0091 Controlled shutdown flag affects follower handling
P0 · M3 · Task · deps: REPL-0089 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala beginControlledShutdown @4.4.0
Done when: begin_controlled_shutdown sets a flag consulted by follower delta handling; tests with a delta during shutdown.

### [To Do] REPL-0092 Shut down idle fetchers after each delta
P0 · M3 · Task · deps: REPL-0089 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala applyDelta @4.4.0; core/src/main/scala/kafka/server/AbstractFetcherManager.scala shutdownIdleFetcherThreads @4.4.0
Done when: After applying a delta, fetchers without partitions are shut down; tests.

### [To Do] REPL-0093 Directory assignment updates from deltas
P0 · M7 · Task · deps: REPL-0088, CTRL-F20 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala maybeUpdateTopicAssignment @4.4.0
Done when: When the metadata.version supports directory assignment and a partition's assigned directory differs from where its log lives, the directory event handler reports the actual directory; tests.

### [To Do] REPL-0094 Log directory failure marks partitions offline
P0 · M7 · Task · deps: REPL-0088, STOR-F21 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala handleLogDirFailure, markPartitionOffline @4.4.0
Done when: When a log directory fails, its partitions become offline, fetchers stop, delayed operations complete, and the failure is reported to the controller; tests with the fault-injecting fs.

### [To Do] REPL-0095 Port Kafka's replica manager delta tests
P0 · M3 · Task · deps: REPL-0089, REPL-0087, REPL-0091, REPL-0092 · labels: test · ref: core/src/test/scala/unit/kafka/server/ReplicaManagerTest.scala @4.4.0
Done when: applyDelta cases from ReplicaManagerTest at 4.4.0 pass with attribution.

## [To Do] REPL-F13 Read path
P0 · M2 · Feature · deps: REPL-F04
Scope: Isolation (HW/LSO), max bytes per partition and response, min-one-message, aborted transaction list.

### [To Do] REPL-0096 Fetch byte limits: response max bytes, partition max bytes and min-one-message
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/ReplicaManager.scala readFromLog @4.4.0
Done when: Partitions are read in request order with min(partition max bytes, remaining response bytes); the first non-empty partition may exceed limits (min-one-message), after which limits apply strictly; incomplete first batches return empty records; tests.

### [To Do] REPL-0097 Read isolation: log end for followers, HW for read_uncommitted consumers
P0 · M2 · Task · deps: REPL-0096 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java read @4.4.0; core/src/main/scala/kafka/cluster/Partition.scala readRecords @4.4.0
Done when: Followers read to the log end offset; consumers read to the HW; offsets beyond the readable end return empty; tests.

### [To Do] REPL-0098 Fetch partition checks: topic id consistency, epochs and leader requirement
P0 · M2 · Task · deps: REPL-0096 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala readFromLog, hasConsistentTopicId @4.4.0
Done when: A request topic id that does not match the log fails INCONSISTENT_TOPIC_ID; leader epoch checks apply; consumer fetches that require the leader fail NOT_LEADER_OR_FOLLOWER on followers; tests.

### [To Do] REPL-0099 Fetch error mapping and failed-fetch accounting
P0 · M2 · Task · deps: REPL-0098 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala readFromLog, handleOffsetOutOfRangeError @4.4.0
Done when: UNKNOWN_TOPIC_OR_PARTITION, NOT_LEADER_OR_FOLLOWER, UNKNOWN_LEADER_EPOCH, FENCED_LEADER_EPOCH, REPLICA_NOT_AVAILABLE, KAFKA_STORAGE_ERROR and INCONSISTENT_TOPIC_ID map directly without failure metrics; OFFSET_OUT_OF_RANGE for offsets below the log start or above the end; other errors are logged and counted as failed fetches; tests.

### [To Do] REPL-0100 Fetch partition result fields
P0 · M2 · Task · deps: REPL-0097 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogReadResult.java @4.4.0; core/src/main/scala/kafka/cluster/Partition.scala readRecords @4.4.0
Done when: Each result carries records, HW, LSO, log start offset, diverging epoch, preferred read replica and error, using offsets captured before the read; tests.

### [To Do] REPL-0101 Hand off file-region records for zero-copy sends
P1 · M2 · Task · deps: REPL-0100, STOR-F23 · labels: perf
Done when: Read results reference segment byte ranges that NET-F06 can send with sendfile on plaintext connections, without copying; a test asserts no copy for a plaintext fetch.

### [To Do] REPL-0102 Fetch request rate and failed fetch rate per topic
P1 · M2 · Task · deps: REPL-0099, OBS-F04 · labels: observability · ref: core/src/main/scala/kafka/server/ReplicaManager.scala fetchMessages @4.4.0
Done when: Total and failed fetch request rates per topic and in aggregate (BrokerTopicMetrics TotalFetchRequestsPerSec, FailedFetchRequestsPerSec) are recorded; tests.

## [To Do] REPL-F14 Offset resolution
P0 · M2 · Feature · deps: REPL-F13
Scope: ListOffsets: earliest, latest, timestamp, max timestamp, earliest-local, latest-tiered.

### [To Do] REPL-0103 ListOffsets EARLIEST (-2) and EARLIEST_LOCAL (-4)
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java fetchOffsetByTimestamp @4.4.0; clients/src/main/java/org/apache/kafka/common/requests/ListOffsetsRequest.java @4.4.0
Done when: EARLIEST returns the log start offset; EARLIEST_LOCAL returns the local log start (equal to the log start without tiering); both include the leader epoch for the offset; tests.

### [To Do] REPL-0104 ListOffsets LATEST (-1) by isolation
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/cluster/Partition.scala fetchOffsetForTimestamp @4.4.0
Done when: LATEST returns the HW for read_uncommitted, the LSO for read_committed and the log end offset for replica (debug) requests, with the current leader epoch; tests.

### [To Do] REPL-0105 ListOffsets by timestamp through the time index
P0 · M2 · Task · deps: STOR-F05 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java fetchOffsetByTimestamp @4.4.0; core/src/main/scala/kafka/cluster/Partition.scala fetchOffsetForTimestamp @4.4.0
Done when: A target timestamp returns the first offset whose batch max timestamp >= target and its timestamp, or no offset (-1) when none exists or the found offset is not below the last fetchable offset; tests against a model.

### [To Do] REPL-0106 ListOffsets MAX_TIMESTAMP (-3)
P0 · M2 · Task · deps: REPL-0105 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java fetchOffsetByTimestamp @4.4.0
Done when: MAX_TIMESTAMP returns the offset and timestamp of the record with the largest timestamp (from segment max timestamps); tests with out-of-order timestamps.

### [To Do] REPL-0107 ListOffsets LATEST_TIERED (-5) and EARLIEST_PENDING_UPLOAD (-6) without tiering
P0 · M2 · Task · deps: REPL-0103 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java fetchOffsetByTimestamp @4.4.0
Done when: Without remote storage LATEST_TIERED returns offset -1 with epoch -1 and EARLIEST_PENDING_UPLOAD returns Kafka's non-tiered result; tiered behavior is added by TIER; tests.

### [To Do] REPL-0108 Version gating of special timestamps
P0 · M2 · Task · deps: REPL-0106, REPL-0107 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala isListOffsetsTimestampUnsupported @4.4.0
Done when: Special timestamps not supported by the request version (MAX_TIMESTAMP before v7, EARLIEST_LOCAL before v8, LATEST_TIERED before v9, EARLIEST_PENDING_UPLOAD before its version) fail UNSUPPORTED_VERSION per partition; tests per version.

### [To Do] REPL-0109 OFFSET_NOT_AVAILABLE while the HW lags the epoch start
P0 · M2 · Task · deps: REPL-0104, REPL-0105 · ref: core/src/main/scala/kafka/cluster/Partition.scala fetchOffsetForTimestamp @4.4.0; core/src/main/scala/kafka/server/ReplicaManager.scala fetchOffset @4.4.0
Done when: For client requests on a new leader whose HW is below the leader epoch start offset, LATEST and timestamp lookups beyond the last fetchable offset fail OFFSET_NOT_AVAILABLE (v5+) or LEADER_NOT_AVAILABLE (older); tests.

### [To Do] REPL-0110 ListOffsets epoch checks and error mapping
P0 · M2 · Task · deps: REPL-0104 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala fetchOffset @4.4.0
Done when: Current leader epoch checks fail FENCED_LEADER_EPOCH or UNKNOWN_LEADER_EPOCH; non-leaders fail NOT_LEADER_OR_FOLLOWER except for the debugging replica id; known transient errors are not logged with stack traces; tests.

### [To Do] REPL-0111 Port Kafka's ListOffsets tests
P0 · M2 · Task · deps: REPL-0108, REPL-0109, REPL-0110 · labels: test · ref: core/src/test/scala/unit/kafka/server/ListOffsetsRequestTest.scala @4.4.0
Done when: Cases from ListOffsetsRequestTest at 4.4.0 pass as compatibility tests and PartitionTest timestamp cases pass as unit tests.

## [To Do] REPL-F15 Replication simulation testing
P0 · M3 · Feature · deps: REPL-F06, TEST-F06
Scope: No acknowledged acks=all write lost; HW monotonic; one leader per epoch.

### [To Do] REPL-0112 Replication simulation scaffold: brokers, controller quorum, producers and consumers
P0 · M3 · Task · deps: CTRL-F09, REPL-F05 · labels: test
Done when: A sim test runs 3-5 brokers with simulated storage and network, a simulated controller quorum, acks=all producers and consumers; seeds and replay commands are printed; fixed seeds run in CI.

### [To Do] REPL-0113 Invariant: no acknowledged acks=all write is lost
P0 · M3 · Task · deps: REPL-0112 · labels: test
Done when: Every acknowledged acks=all record is present at its offset in every later leader's log and readable by consumers, across crashes, partitions and leader changes, unless an unclean election was enabled.

### [To Do] REPL-0114 Invariants: HW monotonic on each leader epoch and consumers never read past it
P0 · M3 · Task · deps: REPL-0112 · labels: test
Done when: A checker asserts the HW never decreases within a leader epoch and consumer reads never return offsets at or above the HW.

### [To Do] REPL-0115 Invariant: one leader per partition epoch, and stale-epoch requests are rejected
P0 · M3 · Task · deps: REPL-0112 · labels: test
Done when: No two brokers act as leader for the same partition and leader epoch; fetches and produces with stale epochs are always rejected.

### [To Do] REPL-0116 Invariant: replicas agree on the log prefix below the HW
P0 · M3 · Task · deps: REPL-0112 · labels: test
Done when: Batches below the HW are identical (offset, epoch, CRC) on every ISR member.

### [To Do] REPL-0117 Invariant: ISR membership is well formed and committed members were caught up
P0 · M3 · Task · deps: REPL-0112 · labels: test
Done when: After every step the committed ISR is a non-empty subset of the replicas containing the leader, and every replica added to the ISR had a log end offset at or above the leader's HW and epoch start offset when proposed.

### [To Do] REPL-0118 Scenario: min ISR enforcement under follower failures
P0 · M3 · Task · deps: REPL-0113 · labels: test
Done when: With min.insync.replicas=2 and followers failing, acks=all produces fail NOT_ENOUGH_REPLICAS or NOT_ENOUGH_REPLICAS_AFTER_APPEND and are never acknowledged while under min ISR.

### [To Do] REPL-0119 Scenario: divergent followers truncate and converge after leader changes
P0 · M3 · Task · deps: REPL-0116 · labels: test
Done when: Leaders changing rapidly create divergent logs; followers truncate via diverging epochs and converge to the leader's log.

### [To Do] REPL-0120 Nightly replication simulation with thousands of seeds
P1 · M3 · Task · deps: REPL-0118, REPL-0119, REPL-0114, REPL-0115 · labels: test
Done when: A nightly job runs every replication sim test with thousands of seeds; failing seeds are reported with replay commands and added to the fixed set.

## [To Do] REPL-F16 Replication throttling
P1 · M4 · Feature · deps: REPL-F06, QUOTA-F01
Scope: Throttled replicas and leader/follower replication rates.

### [To Do] REPL-0121 Throttled replica lists and rates
P1 · M4 · Task · deps: CFG-F08, CFG-F07 · ref: server/src/main/java/org/apache/kafka/server/quota/ReplicationQuotaManager.java markThrottled @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/ThrottledReplicaListValidator.java @4.4.0
Done when: leader.replication.throttled.replicas and follower.replication.throttled.replicas topic configs ('partition:broker' lists or '*') and the static leader/follower.replication.throttled.rate broker configs mark throttled partitions and set the quota bounds; throttled replica lists are re-read when topic configs change; invalid lists fail INVALID_CONFIG; tests.

### [To Do] REPL-0122 Replication quota manager on the windowed rate
P1 · M4 · Task · deps: REPL-0121 · ref: server/src/main/java/org/apache/kafka/server/quota/ReplicationQuotaManager.java isQuotaExceeded, record @4.4.0
Done when: Leader and follower quota managers record replicated bytes on QUOTA-F10's windowed rate (replication.quota.window.num, replication.quota.window.size.seconds) and report is_quota_exceeded and is_throttled per partition; tests with the simulated clock.

### [To Do] REPL-0123 Leader-side throttling of out-of-sync followers
P1 · M4 · Task · deps: REPL-0122 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala shouldLeaderThrottle, readFromLog @4.4.0
Done when: A follower fetch for a throttled partition returns empty records when the follower is not in the ISR and the leader quota is exceeded; bytes sent to throttled followers are recorded; delayed fetch accounting excludes throttled partitions; tests.

### [To Do] REPL-0124 Follower-side throttling of fetched partitions
P1 · M4 · Task · deps: REPL-0122 · ref: core/src/main/scala/kafka/server/ReplicaFetcherThread.scala @4.4.0
Done when: When the follower quota is exceeded, throttled partitions are left out of fetch requests and fetched bytes are recorded; tests.

### [To Do] REPL-0125 Dynamic updates of replication throttle rates
P1 · M7 · Task · deps: REPL-0122, CFG-F04 · ref: server/src/main/java/org/apache/kafka/server/quota/ReplicationQuotaManager.java updateQuota @4.4.0
Done when: Changing leader.replication.throttled.rate or follower.replication.throttled.rate through dynamic broker config (per broker or cluster default) updates the quota bound without restart; tests with kafka-configs.sh.

### [To Do] REPL-0126 Replication throttle metrics
P2 · M4 · Task · deps: REPL-0123, REPL-0124 · labels: observability · ref: server/src/main/java/org/apache/kafka/server/quota/ReplicationQuotaManager.java @4.4.0
Done when: LeaderReplication and FollowerReplication byte-rate metrics are exported with bounded labels; tests.

### [To Do] REPL-0127 Port Kafka's replication quota tests
P1 · M4 · Task · deps: REPL-0123, REPL-0124 · labels: test · ref: core/src/test/scala/unit/kafka/server/ReplicaManagerQuotasTest.scala @4.4.0
Done when: Cases from ReplicationQuotasTest and ReplicaManagerQuotasTest at 4.4.0 pass with attribution.

## [To Do] REPL-F17 Replication metrics and events
P1 · M2 · Feature · deps: REPL-F02, OBS-F04
Scope: Under-replicated, under-min-ISR, ISR changes, lag, events with reasons.

### [To Do] REPL-0128 Replication event catalog with reasons
P1 · M2 · Task · deps: OBS-F07 · labels: observability
Done when: alluvium-replication's events.rs lists partition_became_leader, partition_became_follower, isr_shrink (reason: follower_lagged{broker_id, lag_ms}), isr_expand, isr_update_failed (error_code), high_watermark_reset, follower_truncated (reason: diverging_epoch, offset_out_of_range, leader_epoch_fallback), partition_under_min_isr (warn) and partition_offline; the driver logs them with the standard fields; tests assert emission.

### [To Do] REPL-0129 Per-partition flight recorder and /debug/partitions provider
P1 · M2 · Task · deps: OBS-F06 · labels: observability
Done when: Each partition keeps a 256-entry recorder of transitions (leader/follower, ISR changes, HW resets, truncations, AlterPartition outcomes) as small enums; the diagnostics endpoint shows it with replication state; tests.

### [To Do] REPL-0130 Metrics PartitionCount, LeaderCount and OfflineReplicaCount
P1 · M2 · Task · deps: none · labels: observability · ref: core/src/main/scala/kafka/server/ReplicaManager.scala @4.4.0
Done when: Gauges for hosted partitions, partitions led and offline replicas are exported; tests.

### [To Do] REPL-0131 Metrics UnderReplicatedPartitions, UnderMinIsrPartitionCount and AtMinIsrPartitionCount
P1 · M2 · Task · deps: none · labels: observability · ref: core/src/main/scala/kafka/server/ReplicaManager.scala underReplicatedPartitionCount @4.4.0; core/src/main/scala/kafka/cluster/Partition.scala isUnderMinIsr @4.4.0
Done when: Gauges computed over leader partitions; per-partition UnderReplicated, UnderMinIsr and AtMinIsr only with alluvium.metrics.per.partition.enabled; tests.

### [To Do] REPL-0132 Metrics IsrExpandsPerSec, IsrShrinksPerSec and FailedIsrUpdatesPerSec
P1 · M3 · Task · deps: REPL-F05 · labels: observability · ref: server/src/main/java/org/apache/kafka/server/partition/AlterPartitionListener.java @4.4.0
Done when: Counters increment on committed expansions, shrinks and failed AlterPartition updates; tests.

### [To Do] REPL-0133 Per-partition metrics InSyncReplicasCount, ReplicasCount and LastStableOffsetLag
P2 · M2 · Task · deps: none · labels: observability · ref: core/src/main/scala/kafka/cluster/Partition.scala @4.4.0
Done when: Per-partition gauges exist only with alluvium.metrics.per.partition.enabled and are removed with the partition; tests.

### [To Do] REPL-0134 Metric ReassigningPartitions
P2 · M3 · Task · deps: none · labels: observability · ref: core/src/main/scala/kafka/server/ReplicaManager.scala reassigningPartitionsCount @4.4.0
Done when: The gauge counts led partitions with an ongoing reassignment; tests.

### [To Do] REPL-0135 Fetcher metrics: MaxLag, MinFetchRate, ConsumerLag, BytesPerSec, RequestsPerSec, FailedPartitionsCount, DeadThreadCount
P1 · M3 · Task · deps: REPL-F06 · labels: observability · ref: core/src/main/scala/kafka/server/AbstractFetcherManager.scala @4.4.0; core/src/main/scala/kafka/server/AbstractFetcherThread.scala @4.4.0
Done when: Fetcher-manager and per-fetcher metrics are exported with bounded labels (per-partition lag only with alluvium.metrics.per.partition.enabled); tests.

### [To Do] REPL-0136 Per-partition load counters for an internal load report
P1 · M2 · Task · deps: none · labels: foundation-for:X-AUTO
Done when: each leader partition keeps bytes-in, bytes-out, messages-in and request-rate counters in fixed memory, readable through an internal snapshot interface without touching the metrics registry; unit tests cover rate windows and counter reset on leadership change.
