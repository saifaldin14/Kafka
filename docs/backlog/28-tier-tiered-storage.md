# TIER: Tiered storage

> Crates: `alluvium-storage` (remote), drivers in `alluvium-server` · Kafka counterpart: KIP-405 (`storage/…/log/remote`) · Milestones: M9

KIP-405 tiered storage: remote segment copy, remote reads, remote retention, and object-store backends.

## [To Do] TIER-F01 Remote storage interface and backends
P0 · M9 · Feature · deps: STOR-F02
Scope: Remote storage trait and object-store backends (S3, GCS, Azure, local FS).

### [To Do] TIER-0001 Spike: tiered storage plug-in interface ADR: a Rust `RemoteStorageManager` equivalent with built-in backends only (no Java plug-ins), object key layout, upload ordering, how `remote.log.storage.manager.class.name` maps to built-in backend names, and the relationship to X-DISKLESS
P0 · M9 · Spike · deps: none · ref: KIP-405
Done when: ADR accepted; it closes the "Tiered storage plug-in interface" row in docs/architecture.md §10 and records the plug-in deviation in docs/parity.md.

### [To Do] TIER-0002 Spike: object-store client dependency ADR: one multi-cloud crate versus per-cloud SDKs versus a minimal in-house HTTP client on rustls; transitive dependency count, `unsafe` use, maintenance
P0 · M9 · Spike · deps: TIER-0001, FND-F03 · labels: security, foundation-for:X-DISKLESS
Done when: ADR accepted with the dependency review checklist filled in; chosen crates pass `cargo deny`; X-DISKLESS-F02 reuses the decision.

### [To Do] TIER-0003 Remote metadata types: `RemoteLogSegmentId` (topic id partition, uuid), `RemoteLogSegmentMetadata` (start and end offset, max timestamp, broker id, event timestamp, segment leader epochs, size, custom metadata, state, txn index empty), segment updates, partition delete metadata
P0 · M9 · Task · deps: TIER-0001 · ref: storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogSegmentId.java @4.4.0; storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogSegmentMetadata.java @4.4.0; storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogSegmentMetadataUpdate.java @4.4.0; storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemotePartitionDeleteMetadata.java @4.4.0
Done when: `create_with_updates` validates the id and the state transition; leader-epoch map must be non-empty and within [start, end]; property tests for invariants.

### [To Do] TIER-0004 Remote segment state machine (COPY_SEGMENT_STARTED 0, COPY_SEGMENT_FINISHED 1, DELETE_SEGMENT_STARTED 2, DELETE_SEGMENT_FINISHED 3) and partition delete states (MARKED, STARTED, FINISHED) with Kafka's valid transitions
P0 · M9 · Task · deps: TIER-0003 · ref: storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogSegmentState.java isValidTransition @4.4.0; storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemotePartitionDeleteState.java isValidTransition @4.4.0
Done when: Wire ids match Kafka; every invalid transition returns a typed error; exhaustive transition table test.

### [To Do] TIER-0005 `RemoteStorageManager` trait: copy segment data (returning optional custom metadata), fetch segment bytes from a start position (optionally to an end), fetch an index by type (offset, timestamp, producer snapshot, transaction, leader epoch), delete segment data idempotently; `RemoteStorageError` with retriable, not-found and fatal kinds
P0 · M9 · Task · deps: TIER-0003 · ref: storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteStorageManager.java @4.4.0; storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteStorageException.java @4.4.0; storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RetriableRemoteStorageException.java @4.4.0; storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteResourceNotFoundException.java @4.4.0
Done when: Trait rustdoc states idempotency and retry contracts per method; error kinds map to Kafka's exception classes in a documented table.

### [To Do] TIER-0006 `LogSegmentData` bundle: log file, offset index, time index, optional transaction index, producer snapshot, leader-epoch checkpoint bytes
P0 · M9 · Task · deps: TIER-0003 · ref: storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/LogSegmentData.java @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java epochEntriesAsByteBuffer @4.4.0
Done when: Leader-epoch bytes use the STOR checkpoint text format and match Kafka's output for the same entries (golden test); missing mandatory files return a typed error.

### [To Do] TIER-0007 Deterministic object key layout and upload ordering per segment: data and index objects first, a completion marker last, so partial uploads are detectable and deletable
P0 · M9 · Task · deps: TIER-0001, TIER-0006 · ref: KIP-405
Done when: Key derivation is a pure function with a property test for uniqueness across topics, topic ids, partitions and segment ids; layout documented in the module doc.

### [To Do] TIER-0008 `ObjectStore` abstraction: put (multipart above a threshold), ranged get as a bounded stream, delete, delete-many, list by prefix, head; per-operation timeouts; typed `ObjectStoreError`
P0 · M9 · Task · deps: TIER-0001, TIER-0002 · labels: foundation-for:X-DISKLESS
Done when: Every call has a timeout; streaming reads never buffer more than the configured chunk size; the trait is reusable by X-DISKLESS.

### [To Do] TIER-0009 Generic `RemoteStorageManager` over `ObjectStore`: upload all files of a segment, ranged segment reads, index fetches, idempotent deletes tolerating not-found
P0 · M9 · Task · deps: TIER-0005, TIER-0007, TIER-0008 · ref: storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteStorageManager.java copyLogSegmentData, fetchLogSegment, fetchIndex, deleteLogSegmentData @4.4.0
Done when: Passes the conformance suite (task below) with the local backend; a delete of a never-uploaded segment succeeds.

### [To Do] TIER-0010 Local filesystem object-store backend (development and tests) built on the storage `Fs` trait with atomic renames
P0 · M9 · Task · deps: TIER-0008 · ref: storage/src/testFixtures/java/org/apache/kafka/server/log/remote/storage/LocalTieredStorage.java @4.4.0
Done when: Works over `MemFs` for simulation and `StdFs` for compatibility tests; listed keys are sorted deterministically.

### [To Do] TIER-0011 Amazon S3 backend: SigV4 signing, multipart upload with abort on failure, ranged GET, DeleteObjects batches, endpoint override and path-style addressing for S3-compatible stores (MinIO), credentials from config, environment or instance profile
P0 · M9 · Task · deps: TIER-0008 · labels: security
Done when: Passes the conformance suite against MinIO in the compatibility job; credentials are `Secret<T>` and never logged; TLS via rustls with verification on.

### [To Do] TIER-0012 Google Cloud Storage backend: service-account and workload-identity OAuth2, resumable uploads, ranged reads, batch deletes
P0 · M9 · Task · deps: TIER-0008 · labels: security
Done when: Passes the conformance suite against a GCS emulator in the compatibility job; tokens refreshed before expiry; secrets redacted.

### [To Do] TIER-0013 Azure Blob Storage backend: shared key, SAS and managed identity auth, staged block uploads, ranged reads, batch deletes
P0 · M9 · Task · deps: TIER-0008 · labels: security
Done when: Passes the conformance suite against Azurite in the compatibility job; secrets redacted; TLS via rustls.

### [To Do] TIER-0014 Backend configuration: `remote.log.storage.manager.class.name` mapped to a built-in backend name, `rsm.config.`-prefixed (`remote.log.storage.manager.impl.prefix`) keys passed to the backend, `remote.log.storage.manager.class.path` rejected with a clear error
P0 · M9 · Task · deps: TIER-0009, CFG-F11 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java @4.4.0
Done when: Unknown backend names fail startup with the list of supported ones; table test of prefix stripping; secrets marked sensitive.

### [To Do] TIER-0015 Object-store retry policy: classify provider errors (throttling, 5xx, timeouts, connection resets) as retriable, retry idempotent operations with exponential backoff and jitter from the seeded RNG, bounded attempts and total time
P0 · M9 · Task · deps: TIER-0008, TYPE-F07 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java @4.4.0
Done when: Table test of classification per backend; simulated-clock test of backoff bounds; non-idempotent operations are never blindly retried.

### [To Do] TIER-0016 Server-side encryption options: SSE-S3 and SSE-KMS key id, GCS CMEK, Azure encryption scope, passed through backend config
P1 · M9 · Task · deps: TIER-0011, TIER-0012, TIER-0013 · labels: security
Done when: Conformance runs with encryption enabled on emulators that support it; key ids are not treated as secrets but credentials are.

### [To Do] TIER-0017 `RemoteStorageManager` conformance suite run against local, S3 (MinIO), GCS (emulator) and Azure (Azurite) backends: copy, ranged fetch, each index type, idempotent delete, not-found behavior
P0 · M9 · Task · deps: TIER-0009, TIER-0010, TEST-F08 · labels: test · ref: storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteStorageManager.java @4.4.0
Done when: Suite runs in `cargo xtask compat`; each backend passes; failures name the backend and operation.

### [To Do] TIER-0018 Object-store client metrics: requests, errors by kind, latency and bytes by backend and operation
P1 · M9 · Task · deps: TIER-0008, OBS-F04
Done when: `alluvium_tier_objstore_requests_total{backend,op}`, `alluvium_tier_objstore_errors_total{backend,op,kind}` and latency histograms asserted in tests; labels are closed enums.

## [To Do] TIER-F02 Remote log metadata
P0 · M9 · Feature · deps: TIER-F01
Scope: Topic-based remote log metadata manager (`__remote_log_metadata`).

### [To Do] TIER-0019 `RemoteLogMetadataManager` trait: add and update segment metadata (completing once applied locally), segment for (epoch, offset), highest offset for epoch, partition delete metadata, list segments (all or per epoch), leadership changes, stop partitions, remote log size, next segment with a transaction index, readiness
P0 · M9 · Task · deps: none · ref: storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogMetadataManager.java @4.4.0
Done when: Trait rustdoc mirrors Kafka's contracts including ordering guarantees of listings; a mock-free in-memory implementation backs unit tests.

### [To Do] TIER-0020 `__remote_log_metadata` record codec: generated types for the four storage schemas (api keys 0-3) framed as unsigned varints (frame version 1, api key, version) then the message
P0 · M9 · Task · deps: none · labels: decoder, security · ref: storage/src/main/resources/message/RemoteLogSegmentMetadataRecord.json @4.4.0; storage/src/main/resources/message/RemoteLogSegmentMetadataUpdateRecord.json @4.4.0; storage/src/main/resources/message/RemotePartitionDeleteMetadataRecord.json @4.4.0; storage/src/main/resources/message/RemoteLogSegmentMetadataSnapshotRecord.json @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/metadata/storage/serialization/RemoteLogMetadataSerde.java @4.4.0; server-common/src/main/java/org/apache/kafka/server/common/serialization/AbstractApiMessageSerde.java @4.4.0
Done when: Golden bytes produced by Kafka's RemoteLogMetadataSerde decode and re-encode identically; unknown frame versions and api keys return typed errors; fuzz target `remote_log_metadata` runs clean.

### [To Do] TIER-0021 Metadata partition assignment: Java `Objects.hash(topicId lsb, topicId msb, partition)` as 4 big-endian bytes, murmur2, to-positive, modulo the metadata topic partition count
P0 · M9 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/server/log/remote/metadata/storage/RemoteLogMetadataTopicPartitioner.java metadataPartition, toBytes @4.4.0
Done when: Table test of 1,000 partitions matches Kafka's assignments (golden values from the JVM harness).

### [To Do] TIER-0022 Per-partition remote metadata cache: segments by id, per-epoch offset-to-segment maps for COPY_SEGMENT_FINISHED segments, unreferenced in-progress segments, highest offset per epoch, removal on DELETE_SEGMENT_FINISHED
P0 · M9 · Task · deps: TIER-0019 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/metadata/storage/RemoteLogMetadataCache.java @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/metadata/storage/RemoteLogLeaderEpochState.java @4.4.0
Done when: Tests port RemoteLogMetadataCacheTest; property test: lookups by (epoch, offset) equal a linear search over finished segments.

### [To Do] TIER-0023 Partition metadata store: routes records to per-partition caches, applies partition delete states (state removed on DELETE_PARTITION_FINISHED), tracks per-partition readiness
P0 · M9 · Task · deps: TIER-0022 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/metadata/storage/RemotePartitionMetadataStore.java @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/metadata/storage/RemotePartitionMetadataEventHandler.java @4.4.0
Done when: Records for unknown partitions are buffered or ignored as Kafka does; tests cover delete-state handling.

### [To Do] TIER-0024 Publishing side of the topic-based manager: create `__remote_log_metadata` if absent (partitions, replication factor, `retention.ms` -1, `min.insync.replicas`, delete policy), produce with `acks=all`, complete the caller only after the local consumer has applied the record
P0 · M9 · Task · deps: TIER-0020, TIER-0021, NET-F10 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/metadata/storage/ProducerManager.java @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/metadata/storage/TopicBasedRemoteLogMetadataManager.java addRemoteLogSegmentMetadata, updateRemoteLogSegmentMetadata, putRemotePartitionDeleteMetadata @4.4.0
Done when: Completion waits at most `remote.log.metadata.consume.wait.ms` then fails with a retriable error; test shows ordering per partition is preserved.

### [To Do] TIER-0025 Consuming side: assign metadata partitions for user partitions this broker leads or follows, consume from the beginning, apply records idempotently by offset, mark user partitions ready once caught up to end offsets captured at assignment, drop state for unassigned partitions
P0 · M9 · Task · deps: TIER-0023, TIER-0020 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/metadata/storage/ConsumerTask.java @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/metadata/storage/ConsumerManager.java @4.4.0
Done when: Restart test rebuilds identical caches from the topic; readiness flips only after catch-up; consumption runs as a supervised named task.

### [To Do] TIER-0026 Initialization and readiness: retry topic creation and client setup every `remote.log.metadata.initialization.retry.interval.ms` up to `remote.log.metadata.initialization.retry.max.timeout.ms`, then fail the node as Kafka does; expose readiness to copy, expiration and follower tasks
P0 · M9 · Task · deps: TIER-0024, TIER-0025 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/metadata/storage/TopicBasedRemoteLogMetadataManager.java isReady @4.4.0
Done when: Simulated-clock test of retry and timeout; not-ready callers get `RemoteStorageNotReady`.

### [To Do] TIER-0027 Topic-based manager configuration: `rlmm.config.`-prefixed keys (`remote.log.metadata.topic.num.partitions` 50, `.replication.factor` 3, `.retention.ms` -1, `.min.isr`, `consume.wait.ms`, `initialization.retry.*`, client prefixes) and `remote.log.metadata.manager.listener.name`
P0 · M9 · Task · deps: TIER-0026 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/metadata/storage/TopicBasedRemoteLogMetadataManagerConfig.java @4.4.0
Done when: Defaults and validation match Kafka in table tests; client-prefixed overrides reach the internal clients.

### [To Do] TIER-0028 Treat `__remote_log_metadata` as an internal topic: listed as internal in Metadata, protected from client produce and deletion without the required ACLs
P0 · M9 · Task · deps: TIER-0024, SEC-F08 · labels: security
Done when: Authorization tests show a client without `CLUSTER_ACTION`-equivalent rights cannot write or delete it; differential test matches Kafka's responses.

### [To Do] TIER-0029 Port TopicBasedRemoteLogMetadataManagerTest and RemoteLogMetadataManagerTest scenarios
P0 · M9 · Task · deps: TIER-0025, TIER-0026 · labels: test · ref: storage/src/test/java/org/apache/kafka/server/log/remote/metadata/storage/TopicBasedRemoteLogMetadataManagerTest.java @4.4.0
Done when: Tests carry `// Ported from:` and pass under the simulated clock.

## [To Do] TIER-F03 Segment copy
P0 · M9 · Feature · deps: TIER-F02
Scope: The leader copies rolled segments and indexes; copy state machine.

### [To Do] TIER-0030 Remote log manager core: per-partition task registry; leaders get copy and expiration tasks, followers get a follower task; tasks replaced on leader epoch change and cancelled on stop
P0 · M9 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java onLeadershipChange, stopLeaderCopyRLMTasks, stopPartitions, cacheTopicPartitionIds, doHandleLeaderPartition @4.4.0
Done when: Leadership flip tests show exactly one task of each kind per partition; tasks are supervised and named `rlm-<kind>-<topic>-<partition>`.

### [To Do] TIER-0031 Task scheduling: copier, expiration and follower pools (`remote.log.manager.copier.thread.pool.size`, `.expiration.thread.pool.size`, `.follower.thread.pool.size`), run every `remote.log.manager.task.interval.ms`, retriable failures retried with `remote.log.manager.task.retry.backoff.ms`, `.backoff.max.ms` and `.jitter`
P0 · M9 · Task · deps: TIER-0030, SRV-F02 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java RLMScheduledThreadPool, RLMTask @4.4.0
Done when: Simulated-clock tests of period and backoff; pools are bounded and their idle ratio is reported.

### [To Do] TIER-0032 Copied-offset initialization on becoming leader: highest remote offset along the local leader-epoch lineage; apply the remote log start offset to the local log
P0 · M9 · Task · deps: TIER-0030, STOR-F11 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java maybeUpdateCopiedOffset, maybeUpdateLogStartOffsetOnBecomingLeader, findHighestRemoteOffset @4.4.0
Done when: Tests with divergent epoch histories pick the highest offset only from epochs in the local lineage.

### [To Do] TIER-0033 Copy candidates: sealed segments in [max(copied + 1, log start), last stable offset) whose next segment base <= last stable offset, filtered by `log.remote.copy.lag.ms` and `log.remote.copy.lag.bytes` (0 immediate, -1 derived from local retention)
P0 · M9 · Task · deps: TIER-0032 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java candidateLogSegments, isEligibleForUpload, eligibleUploadByTime, eligibleUploadBySize @4.4.0
Done when: Table tests for each lag setting; the active segment is never a candidate.

### [To Do] TIER-0034 Copy one segment: publish COPY_SEGMENT_STARTED, upload log, indexes, transaction index if non-empty, producer snapshot at the next segment base and leader-epoch bytes, publish COPY_SEGMENT_FINISHED with custom metadata, advance the copied offset and the highest offset in remote storage
P0 · M9 · Task · deps: TIER-0033, STOR-F15 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java copyLogSegment, getLeaderEpochEntries @4.4.0
Done when: Test checks metadata states and uploaded objects for a segment with transactions; a missing producer snapshot fails with a typed error.

### [To Do] TIER-0035 Copy failure handling: retriable errors retried on the next run; fatal errors delete the partial upload and count a failed copy; custom metadata above `remote.log.metadata.custom.metadata.max.bytes` deletes the upload and cancels the task
P0 · M9 · Task · deps: TIER-0034 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java copyLogSegmentsToRemote, copyLogSegment @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/storage/CustomMetadataSizeLimitExceededException.java @4.4.0
Done when: Each path has a test with the faulty object store; no COPY_SEGMENT_FINISHED is published for a failed upload.

### [To Do] TIER-0036 Leadership loss mid-copy: the task checks cancellation between steps and leaves COPY_SEGMENT_STARTED for the new leader to clean up
P0 · M9 · Task · deps: TIER-0034 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java RLMTask @4.4.0
Done when: Simulation cancels at every step; the new leader's expiration task removes the dangling segment and readers never see it.

### [To Do] TIER-0037 Copy events and flight-recorder entries: `remote_segment_copy_started`, `remote_segment_copied` (segment id, offsets, bytes, `elapsed_ms`), `remote_segment_copy_failed` with reason
P0 · M9 · Task · deps: TIER-0034, OBS-F07
Done when: Events listed in `events.rs`; asserted in a copy test.

### [To Do] TIER-0038 Port RemoteLogManagerTest copy cases and add a simulation that crashes the leader after each copy step
P0 · M9 · Task · deps: TIER-0035, TIER-0036 · labels: test · ref: storage/src/test/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerTest.java @4.4.0
Done when: No acknowledged record is lost and no segment is visible to readers before COPY_SEGMENT_FINISHED; seeds printed on failure.

## [To Do] TIER-F04 Remote reads
P0 · M9 · Feature · deps: TIER-F02
Scope: Fetch from remote segments, remote fetch delayed operation, remote index cache.

### [To Do] TIER-0039 Fetch routing to remote storage: a consumer fetch below the local log start offset on a remote-enabled log becomes a remote read; a follower fetch gets `OFFSET_MOVED_TO_TIERED_STORAGE`
P0 · M9 · Task · deps: none · ref: core/src/main/scala/kafka/server/ReplicaManager.scala @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/RemoteStorageFetchInfo.java @4.4.0
Done when: Tests cover consumer, follower and remote-disabled cases; the log returns a typed "remote required" outcome rather than an error string.

### [To Do] TIER-0040 Remote read: epoch for offset, segment metadata, start position from the remote index cache, ranged fetch, first batch at or after the offset, next-segment walk when empty, max bytes and min-one-message
P0 · M9 · Task · deps: TIER-0039 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java read, findFirstBatch, findNextSegmentMetadata @4.4.0
Done when: Tests port RemoteLogManagerTest read cases; reads allocate at most the fetch size (or one batch with min-one-message).

### [To Do] TIER-0041 Aborted transactions for read_committed remote reads, collected across remote transaction indexes (skipping segments without one) and continuing into local segments
P0 · M9 · Task · deps: TIER-0040 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java addAbortedTransactions, collectAbortedTransactions, collectAbortedTransactionInLocalSegments @4.4.0
Done when: read_committed consumers get the same aborted list from remote and local copies of the same data (test).

### [To Do] TIER-0042 Remote reader pool: `remote.log.reader.threads` workers and a bounded queue of `remote.log.reader.max.pending.tasks`; rejected reads return an error to the fetch
P0 · M9 · Task · deps: TIER-0040 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogReader.java @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/RemoteStorageThreadPool.java @4.4.0
Done when: Queue-full test returns the Kafka error for the partition without blocking; workers are supervised.

### [To Do] TIER-0043 Delayed remote fetch: complete when all remote reads finish or a partition is no longer led, unknown or offline; expire after `remote.fetch.max.wait.ms` and cancel outstanding reads
P0 · M9 · Task · deps: TIER-0042, SRV-F07, REPL-F09 · ref: storage/src/main/java/org/apache/kafka/server/purgatory/DelayedRemoteFetch.java @4.4.0
Done when: Tests cover each completion case and expiry; expired requests counted; local partitions in the same fetch are returned with the remote results.

### [To Do] TIER-0044 Remote index cache: bounded by `remote.log.index.file.cache.total.size.bytes` with optional TTL `remote.log.index.file.cache.ttl.ms`; entries hold offset, time and transaction indexes stored under `<log dir>/remote-log-index-cache/` as `<offset>_<uuid>.<type>` written via `.tmp` and rename
P0 · M9 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/RemoteIndexCache.java @4.4.0
Done when: Startup reloads valid entries and deletes `.tmp` and corrupt files; eviction renames to `.deleted` and a cleaner task removes them; memory and disk use stay under the bound in a test.

### [To Do] TIER-0045 Lookups through the cache: offset to position and timestamp to offset; corrupt cached files are dropped and refetched
P0 · M9 · Task · deps: TIER-0044 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/RemoteIndexCache.java lookupOffset, lookupTimestamp, getIndexEntry @4.4.0
Done when: Tests port RemoteIndexCacheTest; concurrent lookups of the same segment fetch it once.

### [To Do] TIER-0046 ListOffsets by timestamp over remote data: search segments by leader epoch and max timestamp, time-index lookup, scan; served asynchronously with a delayed operation bounded by `remote.list.offsets.request.timeout.ms`
P0 · M9 · Task · deps: TIER-0045 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java findOffsetByTimestamp, asyncOffsetRead @4.4.0; storage/src/main/java/org/apache/kafka/server/purgatory/DelayedRemoteListOffsets.java @4.4.0
Done when: Tests cover timestamps in remote only, spanning remote and local, and timeout returning `REQUEST_TIMED_OUT`.

### [To Do] TIER-0047 Tier-aware special offsets in the log: earliest local (-4), latest tiered (-5, KIP-1005) from the highest offset in remote storage with its epoch, earliest pending upload (-6)
P0 · M9 · Task · deps: TIER-0039 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java fetchOffsetByTimestamp, fetchEarliestPendingUploadOffset @4.4.0
Done when: Table tests for each timestamp with and without remote data; REPL-F14 consumes them for ListOffsets.

### [To Do] TIER-0048 Remote index cache metrics: hits, misses, evictions, entries and bytes
P1 · M9 · Task · deps: TIER-0044, OBS-F04
Done when: Metrics asserted in tests; docs/metrics.md rows added.

### [To Do] TIER-0049 End-to-end remote read tests: consume from offset 0 after local retention removed early segments, with read_uncommitted and read_committed
P0 · M9 · Task · deps: TIER-0040, TIER-0043, TIER-0046 · labels: test · ref: storage/src/test/java/org/apache/kafka/tiered/storage/integration/OffloadAndConsumeFromLeaderTest.java @4.4.0
Done when: Consumed records equal the produced ones; ported scenarios carry `// Ported from:`.

## [To Do] TIER-F05 Remote retention
P0 · M9 · Feature · deps: TIER-F02
Scope: Retention and deletion of remote segments.

### [To Do] TIER-0050 Expiration task skeleton (leader only): list remote segments, compute counts and sizes (excluding COPY_SEGMENT_STARTED), build retention inputs
P0 · M9 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java RLMExpirationTask, cleanupExpiredRemoteLogSegments @4.4.0
Done when: Runs on the expiration pool at the task interval; returns early when cancelled or when the log is gone.

### [To Do] TIER-0051 Remote size retention: total = remote finished size + local-only size; delete oldest remote segments while the breach remains; seed the highest remote offset before computing local-only size
P0 · M9 · Task · deps: TIER-0050 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java isSegmentBreachedByRetentionSize, maybeSeedHighestOffsetInRemoteStorage @4.4.0
Done when: Test reproduces the double-counting bug the seeding prevents; deletions stop exactly when within `retention.bytes`.

### [To Do] TIER-0052 Remote time retention: delete segments whose max timestamp is older than now - `retention.ms`
P0 · M9 · Task · deps: TIER-0050 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java isSegmentBreachedByRetentionTime @4.4.0
Done when: Simulated-clock test; the candidate log start offset becomes the deleted segment's end + 1.

### [To Do] TIER-0053 Remote log-start-offset retention: delete segments entirely below the log start offset whose epochs are all <= the earliest local epoch
P0 · M9 · Task · deps: TIER-0050 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java isSegmentBreachByLogStartOffset @4.4.0
Done when: Test after DeleteRecords deletes exactly the remote segments below the new start.

### [To Do] TIER-0054 Epoch-lineage cleanup: delete segments whose epochs all precede the earliest cached epoch, and skip or remove segments not within the current leader's epoch lineage
P0 · M9 · Task · deps: TIER-0050 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java isRemoteSegmentWithinLeaderEpochs, buildFilteredLeaderEpochMap, deleteLogSegmentsDueToLeaderEpochCacheTruncation @4.4.0
Done when: Tests port the Kafka cases with unclean leader election histories.

### [To Do] TIER-0055 Dangling segment cleanup: COPY_SEGMENT_STARTED segments not being copied by this broker and DELETE_SEGMENT_STARTED segments are deleted to completion
P0 · M9 · Task · deps: TIER-0050 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java cleanupExpiredRemoteLogSegments @4.4.0
Done when: Simulation leaves dangling segments after failovers and they converge to zero.

### [To Do] TIER-0056 Remote segment deletion: publish DELETE_SEGMENT_STARTED, delete objects, publish DELETE_SEGMENT_FINISHED; failures leave the segment in DELETE_SEGMENT_STARTED for retry
P0 · M9 · Task · deps: TIER-0050 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java deleteRemoteLogSegment @4.4.0
Done when: Faulty-store test retries until finished; deletes are idempotent.

### [To Do] TIER-0057 Propagate the new log start offset after remote deletions to the local log so consumers and followers see it
P0 · M9 · Task · deps: TIER-0051, TIER-0052, TIER-0053 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java handleLogStartOffsetUpdate @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java updateLogStartOffsetFromRemoteTier @4.4.0
Done when: Fetch below the new start returns `OFFSET_OUT_OF_RANGE` after the deletion (test).

### [To Do] TIER-0058 Remote partition deletion on topic deletion: DELETE_PARTITION_MARKED, STARTED, delete every segment, FINISHED; reassignment away keeps remote data
P0 · M9 · Task · deps: TIER-0056 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java stopPartitions, deleteRemoteLogPartition @4.4.0
Done when: Tests port DeleteTopicTest; no objects remain for the topic after convergence.

### [To Do] TIER-0059 Port the remote retention scenarios and add a property test that remote retention never deletes data inside both retention limits
P0 · M9 · Task · deps: TIER-0051, TIER-0054, TIER-0058 · labels: test · ref: storage/src/test/java/org/apache/kafka/tiered/storage/integration/DeleteSegmentsTest.java @4.4.0; storage/src/test/java/org/apache/kafka/tiered/storage/integration/DeleteSegmentsDueToLogStartOffsetBreachTest.java @4.4.0
Done when: Tests carry `// Ported from:`; property test prints its seed on failure.

## [To Do] TIER-F06 Local retention
P0 · M9 · Feature · deps: TIER-F03
Scope: `local.retention.ms` and `local.retention.bytes`.

### [To Do] TIER-0060 `local.retention.ms` and `local.retention.bytes` (and broker `log.local.retention.*`): -2 means use the total retention; local must not exceed total; effective values when copy is disabled
P0 · M9 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java localRetentionMs, localRetentionSize @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java validateRemoteStorageRetentionSize, validateRemoteStorageRetentionTime @4.4.0
Done when: Table tests for every combination including -1 and -2; validation errors match Kafka's messages.

### [To Do] TIER-0061 Local log start offset tracked separately from the log start offset, incremented with reasons; the loader keeps the checkpointed log start offset for remote-enabled logs
P0 · M9 · Task · deps: TIER-0060 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java maybeIncrementLocalLogStartOffset, updateLocalLogStartOffset, localLogStartOffset @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogLoader.java load @4.4.0
Done when: Restart test preserves both offsets; local start never precedes the log start.

### [To Do] TIER-0062 Local deletion gating: a segment is deletable locally only if it is fully in remote storage (end offset <= highest offset in remote storage) or below the log start offset; roll the active segment when it alone breaches retention so it can be uploaded
P0 · M9 · Task · deps: TIER-0061 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java isSegmentEligibleForDeletion, deleteLogStartOffsetBreachedSegments, deletableSegments @4.4.0
Done when: Test shows un-uploaded data is never deleted locally even past `local.retention.ms`.

### [To Do] TIER-0063 Local-only size and count (segments at or above the highest remote offset) for retention and metrics
P0 · M9 · Task · deps: TIER-0061 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java onlyLocalLogSegmentsSize, onlyLocalLogSegmentsCount @4.4.0
Done when: Table tests; values feed `LocalRetentionSizeInPercent`.

### [To Do] TIER-0064 Crash tests for local deletion interleaved with copy and metadata publication
P0 · M9 · Task · deps: TIER-0062, STOR-F20 · labels: test
Done when: After any crash point every offset at or above the log start offset is readable from local or remote storage.

### [To Do] TIER-0065 Port UnifiedLogTest remote-enabled retention cases
P0 · M9 · Task · deps: TIER-0062, TIER-0063 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/UnifiedLogTest.java @4.4.0
Done when: Tests carry `// Ported from:` and pass.

## [To Do] TIER-F07 Follower behavior
P0 · M9 · Feature · deps: TIER-F02
Scope: Earliest local offset; building auxiliary state from remote storage.

### [To Do] TIER-0066 Leader side: follower fetches below the local log start offset of a remote-enabled log get `OFFSET_MOVED_TO_TIERED_STORAGE`
P0 · M9 · Task · deps: none · ref: core/src/main/scala/kafka/server/ReplicaManager.scala @4.4.0
Done when: Test covers follower, consumer and remote-disabled logs; error code per Fetch version matches Kafka.

### [To Do] TIER-0067 Follower tier state machine: on `OFFSET_MOVED_TO_TIERED_STORAGE`, fetch the leader's earliest local offset and epoch (ListOffsets -4) and start building auxiliary state
P0 · M9 · Task · deps: TIER-0066 · ref: core/src/main/java/kafka/server/TierStateMachine.java @4.4.0; core/src/main/scala/kafka/server/ReplicaFetcherThread.scala @4.4.0
Done when: Sans-I/O state machine with transitions returned as events; tests for each transition.

### [To Do] TIER-0068 Build auxiliary state from remote storage: choose the target epoch (OffsetForLeaderEpoch for the earlier epoch), find the remote segment holding leader local start - 1, truncate fully and start at its end + 1, set log start and local log start, restore the epoch cache and producer snapshot from remote
P0 · M9 · Task · deps: TIER-0067 · ref: core/src/main/java/kafka/server/TierStateMachine.java buildRemoteLogAuxState, fetchEarlierEpochEndOffset, readLeaderEpochCheckpoint, buildProducerSnapshotFile @4.4.0
Done when: Tests port TierStateMachineTest; resulting follower state equals the leader's for offsets at or above the local start.

### [To Do] TIER-0069 Treat leader-epoch and producer-snapshot objects fetched from remote storage as untrusted input decoded with bounded STOR codecs
P0 · M9 · Task · deps: TIER-0068 · labels: decoder, security · ref: storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/LeaderEpochCheckpointFile.java @4.4.0
Done when: Fuzz targets cover both decoders on remote input; corrupt objects produce `RemoteStorageError` and a failed partition, not a panic.

### [To Do] TIER-0070 Aux-state error handling: remote metadata not ready retries with backoff; missing remote segment fails the partition with a descriptive error
P0 · M9 · Task · deps: TIER-0068 · ref: core/src/main/java/kafka/server/TierStateMachine.java buildRemoteStorageException @4.4.0
Done when: Tests cover not-ready and missing-segment cases; build attempts and errors are counted.

### [To Do] TIER-0071 Follower remote task: periodically update the highest offset in remote storage and the log start offset from remote metadata so local retention works on followers
P0 · M9 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java RLMFollowerTask @4.4.0
Done when: Test shows a follower deletes local segments only after they are in remote storage.

### [To Do] TIER-0072 Simulation: erase a follower's storage, restart it, and let it rebuild from remote storage while the leader keeps producing; also leader failover with tiered data
P0 · M9 · Task · deps: TIER-0068, TIER-0071, TEST-F06 · labels: test · ref: storage/src/test/java/org/apache/kafka/tiered/storage/actions/EraseBrokerStorageAction.java @4.4.0
Done when: No acknowledged record lost; the follower rejoins the ISR; seeds printed on failure.

## [To Do] TIER-F08 Tiering enable and disable
P1 · M9 · Feature · deps: TIER-F03
Scope: Per-topic enablement and disablement (KIP-950).

### [To Do] TIER-0073 Enablement validation: `remote.storage.enable` only when `remote.log.storage.system.enable` is true, and only with `cleanup.policy=delete`
P1 · M9 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java validateRemoteStorageOnlyIfSystemEnabled, validateRemoteStorageRequiresDeleteCleanupPolicy @4.4.0
Done when: Validation functions used by CTRL-F11; errors and messages match Kafka in differential tests.

### [To Do] TIER-0074 Enabling tiering on an existing topic: tasks start on the config change, existing sealed segments are copied, local retention starts applying
P1 · M9 · Task · deps: TIER-0073 · ref: storage/src/test/java/org/apache/kafka/tiered/storage/integration/EnableRemoteLogOnTopicTest.java @4.4.0
Done when: Ported scenario passes; no data loss during the transition.

### [To Do] TIER-0075 KIP-950 `remote.log.copy.disable=true`: stop copy tasks, keep expiration and remote reads, use total retention locally
P1 · M9 · Task · deps: TIER-0073 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java validateRetentionConfigsWhenRemoteCopyDisabled @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java doHandleLeaderPartition @4.4.0
Done when: Validation rejects local retention different from total retention while copy is disabled, as Kafka does; tests show no new uploads and remote data still readable.

### [To Do] TIER-0076 KIP-950 disabling `remote.storage.enable`: requires `remote.log.delete.on.disable=true`; stop tasks and delete all remote data; the log start offset becomes the local log start offset
P1 · M9 · Task · deps: TIER-0075 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java validateTurningOffRemoteStorageWithDelete @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java stopPartitions @4.4.0
Done when: Ported DisableRemoteLogOnTopicTest passes; re-enabling later starts from a clean remote state.

### [To Do] TIER-0077 Prove tiering is off by default: with `remote.log.storage.system.enable=false` no remote tasks, pools, internal topic or remote metrics exist
P1 · M9 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java @4.4.0
Done when: Startup test inspects `/debug/tasks` and metrics; compatibility suite results are unchanged.

### [To Do] TIER-0078 Differential tests for enabling and disabling tiering on live topics against Kafka 4.4.0
P1 · M9 · Task · deps: TIER-0074, TIER-0076 · labels: test · ref: storage/src/test/java/org/apache/kafka/tiered/storage/integration/DisableRemoteLogOnTopicTest.java @4.4.0
Done when: Config errors and post-transition offsets match Kafka.

## [To Do] TIER-F09 Remote quotas and metrics
P1 · M9 · Feature · deps: TIER-F03, QUOTA-F01
Scope: Copy and fetch quotas (KIP-956) and remote log metrics.

### [To Do] TIER-0079 KIP-956 copy quota: `remote.log.manager.copy.max.bytes.per.second` over `remote.log.manager.copy.quota.window.num` windows of `.window.size.seconds`; copies wait (bounded) while throttled, then record bytes
P1 · M9 · Task · deps: QUOTA-F10 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/quota/RLMQuotaManager.java @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/quota/RLMQuotaManagerConfig.java @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java copyLogSegmentsToRemote @4.4.0
Done when: Simulated-clock test keeps upload rate within the quota; waits are bounded and cancellable.

### [To Do] TIER-0080 KIP-956 fetch quota: `remote.log.manager.fetch.max.bytes.per.second` with its window settings; remote reads are skipped for a fetch while the quota is exceeded
P1 · M9 · Task · deps: QUOTA-F10 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/quota/RLMQuotaManager.java @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java getFetchThrottleTimeMs @4.4.0
Done when: Test shows throttled fetches return no remote data and later succeed; local data is unaffected.

### [To Do] TIER-0081 Quota metrics for remote copy and fetch throttle time
P1 · M9 · Task · deps: TIER-0079, TIER-0080 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/quota/RLMQuotaMetrics.java @4.4.0
Done when: Metrics asserted in tests; docs/metrics.md rows added.

### [To Do] TIER-0082 Dynamic updates of remote quotas, pool sizes and index cache size
P1 · M9 · Task · deps: TIER-0079, TIER-0080, CFG-F04 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java updateCopyQuota, updateFetchQuota, resizeCopierThreadPool, resizeExpirationThreadPool, resizeFollowerThreadPool, resizeReaderThreadPool, resizeCacheSize @4.4.0
Done when: Changes apply without restart in tests; invalid values rejected as Kafka does.

### [To Do] TIER-0083 Remote request metrics per topic and for all topics: `RemoteCopy*`, `RemoteFetch*`, `RemoteDelete*` requests, bytes and errors per second, `BuildRemoteLogAuxState*`
P1 · M9 · Task · deps: OBS-F04 · ref: storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteStorageMetrics.java @4.4.0; storage/src/main/java/org/apache/kafka/storage/log/metrics/BrokerTopicMetrics.java @4.4.0
Done when: Each metric asserted in a test; `topic` label bounded as in the metrics standard.

### [To Do] TIER-0084 Remote lag and size gauges: `RemoteCopyLagBytes`, `RemoteCopyLagSegments`, `RemoteDeleteLagBytes`, `RemoteDeleteLagSegments`, `RemoteLogMetadataCount`, `RemoteLogSizeBytes`, `RemoteLogSizeComputationTime`, `RetentionSizeInPercent`, `LocalRetentionSizeInPercent`
P1 · M9 · Task · deps: OBS-F04 · ref: storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteStorageMetrics.java @4.4.0; storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java recordLagStats, updateRemoteDeleteLagWith @4.4.0
Done when: Gauges reset and are removed when a partition stops being led (no phantom lag, as fixed in Kafka); asserted in tests.

### [To Do] TIER-0085 Thread-pool metrics: `RemoteLogManagerTasksAvgIdlePercent`, `RemoteLogReader` `TaskQueueSize` and `AvgIdlePercent`
P1 · M9 · Task · deps: OBS-F04 · ref: storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteStorageMetrics.java @4.4.0
Done when: Asserted in tests.

### [To Do] TIER-0086 docs/metrics.md rows for every remote metric with its Kafka JMX name
P1 · M9 · Task · deps: TIER-0083, TIER-0084, TIER-0085, TIER-0081, OBS-F05 · labels: docs · ref: storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteStorageMetrics.java @4.4.0
Done when: Every metric in RemoteStorageMetrics has a row or a documented reason for omission.

### [To Do] TIER-0087 Quota behavior under the simulated clock with several partitions copying and fetching concurrently
P1 · M9 · Task · deps: TIER-0079, TIER-0080 · labels: test
Done when: Aggregate rates stay within quotas over 60 s windows; no starvation of any partition.

## [To Do] TIER-F10 Tiered storage testing
P0 · M9 · Feature · deps: TIER-F04, TEST-F06
Scope: Simulation with object-store faults; differential tests.

### [To Do] TIER-0088 Faulty object store for simulation: seeded latency, transient errors, throttling responses, failed multipart completion, lost and duplicated deletes, delayed listing visibility
P0 · M9 · Task · deps: TEST-F04 · labels: test
Done when: Every fault is reproducible from the seed; fault rates come from the TEST-F04 fault plan.

### [To Do] TIER-0089 Simulation of copy, remote read and remote retention under object-store faults with invariant checkers
P0 · M9 · Task · deps: TIER-0088 · labels: test
Done when: No acknowledged record lost; consumers read contiguous offsets across tiers; remote metadata states only move forward; seeds printed on failure.

### [To Do] TIER-0090 Simulation of leader failover and broker crashes during copy, deletion and follower rebuild
P0 · M9 · Task · deps: TIER-0088 · labels: test
Done when: Invariants hold across thousands of nightly seeds; dangling segments converge to zero.

### [To Do] TIER-0091 Orphan-object checker: compare objects in the store with remote metadata after convergence
P0 · M9 · Task · deps: TIER-0089 · labels: test
Done when: Runs in simulation and in the compatibility job; any orphan fails with its key and segment id.

### [To Do] TIER-0092 Differential tests against Kafka 4.4.0 with tiered storage on local and MinIO backends: produce, consume from earliest after local retention, ListOffsets -4/-5/-6, DeleteRecords, topic deletion
P0 · M9 · Task · deps: TEST-F09 · labels: test · ref: storage/src/test/java/org/apache/kafka/tiered/storage/integration/ListOffsetsTest.java @4.4.0
Done when: Responses match Kafka field by field under the documented ignore rules.

### [To Do] TIER-0093 Port Kafka's tiered-storage integration scenarios (offload and consume, alter log dir, reassignment, partition expansion, transactions) as compatibility tests
P0 · M9 · Task · deps: TEST-F08 · labels: test · ref: storage/src/test/java/org/apache/kafka/tiered/storage/integration/ReassignReplicaMoveAndExpandTest.java @4.4.0; storage/src/test/java/org/apache/kafka/tiered/storage/integration/AlterLogDirTest.java @4.4.0; storage/src/test/java/org/apache/kafka/tiered/storage/integration/TransactionsWithTieredStoreTest.java @4.4.0
Done when: Each ported scenario carries `// Ported from:` and passes with real clients.

### [To Do] TIER-0094 Tiered storage benchmarks: copy throughput, remote read latency versus local, index cache hit ratio
P2 · M9 · Task · deps: TEST-F11 · labels: perf
Done when: Benchmarks run nightly against MinIO with regression thresholds.

### [To Do] TIER-0095 Tiered soak test: multi-day run with retention churn on MinIO
P1 · M9 · Task · deps: TEST-F13 · labels: test, nfr
Done when: Memory, file descriptors, task counts and orphan objects stay flat; report attached to the run.

### [To Do] TIER-0096 docs/formats/remote-storage.md: object key layout, `__remote_log_metadata` record framing and fields, and the `remote-log-index-cache` directory layout, with hex examples
P0 · M9 · Task · deps: none · labels: docs · ref: storage/src/main/resources/message/RemoteLogSegmentMetadataRecord.json @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/RemoteIndexCache.java @4.4.0
Done when: Examples decoded by the docs/formats golden harness.

### [To Do] TIER-0097 Tiered storage operator guide: enabling, backends and credentials, local versus total retention, quotas, metrics, failure modes and recovery
P0 · M9 · Task · deps: none · labels: docs · ref: KIP-405
Done when: docs page reviewed against the ported scenarios; every config key named links to the generated config reference.
