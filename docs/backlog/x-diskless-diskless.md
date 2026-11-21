# X-DISKLESS: Diskless object-storage-native topics

> Crates: `alluvium-storage` object backend, `alluvium-diskless` (new) · Kafka counterpart: none (Alluvium enhancement) · Milestones: M13

Topics whose data lives directly in object storage: stateless brokers, no inter-broker replication traffic, zone-aware reads, and per-topic mixing with classic topics.

## [To Do] X-DISKLESS-F01 Diskless architecture
P0 · M13 · Feature · deps: STOR-F02
Scope: Spike and ADR: shared object log, sequencing, consistency, failure model.

### [To Do] X-DISKLESS-0001 Diskless topics ADR: shared WAL objects, leaderless writes, batch sequencer, read path, compaction and failure model
P0 · M13 · Spike · deps: none · ref: KIP-1150
Done when: docs/adr/NNNN-diskless-topics.md is accepted and decides: brokers coalesce batches from many partitions into shared WAL objects and acknowledge only after the object is durable and its batches are committed by the batch sequencer; offsets are assigned at commit; the sequencer runs on the coordinator runtime over an internal replicated topic (alternatives: KRaft-hosted, external database); per-zone serving brokers instead of replica leaders; read path with range GETs and zone-local caches; object compaction into per-partition segment objects; retention and garbage collection; acks semantics; idempotence and transactions; layering of the new `alluvium-diskless` crate; off-by-default gating; and how the design relates to KIP-1150, KIP-1163, KIP-1164 and KIP-1165 and other object-storage-native systems.

### [To Do] X-DISKLESS-0002 Diskless cost and latency model: PUT, GET and byte costs per GB produced, expected produce latency
P0 · M13 · Task · deps: X-DISKLESS-0001 · labels: nfr, docs
Done when: A model in the ADR appendix computes object-store requests and bytes per GB for given WAL object size and commit interval, and predicted p50/p99 produce latency; the benchmark tasks validate it.

### [To Do] X-DISKLESS-0003 Diskless threat model: bucket credentials, object tampering, cross-tenant object access and request-cost amplification
P0 · M13 · Task · deps: X-DISKLESS-0001 · labels: security, docs
Done when: docs/standards/security.md gains rows for credential leakage, direct bucket access bypassing ACLs, tampered or substituted objects (CRC and object id checks), orphan-object growth, and tiny-produce request amplification, each naming its enforcing task.

### [To Do] X-DISKLESS-0004 Glossary entries for diskless topic, WAL object, segment object, batch sequencer, commit, serving broker and zone
P1 · M13 · Task · deps: X-DISKLESS-0001 · labels: docs
Done when: docs/glossary.md defines the terms with links to the ADR; link check passes.

## [To Do] X-DISKLESS-F02 Object store backends
P0 · M13 · Feature · deps: X-DISKLESS-F01, TIER-F01
Scope: S3, GCS, Azure, MinIO and local FS with retries and timeouts.

### [To Do] X-DISKLESS-0005 Object store operations needed by diskless beyond TIER-F01: range GET, conditional create, multipart, batch delete, paginated list
P0 · M13 · Task · deps: TIER-F01 · ref: storage/api/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteStorageManager.java @4.4.0
Done when: The TIER-F01 object store interface gains these operations with typed errors (`ObjectStoreError::{NotFound, AlreadyExists, Throttled, Timeout, Unavailable, AccessDenied, Corrupt}` carrying bucket, key and request id); the local-FS backend implements them; unit tests.

### [To Do] X-DISKLESS-0006 S3 and MinIO backend support for the diskless operations
P0 · M13 · Task · deps: X-DISKLESS-0005
Done when: Range GET, `If-None-Match: *` conditional PUT, multipart upload with abort, DeleteObjects and ListObjectsV2 pagination implemented; conformance suite passes against MinIO in the compatibility harness.

### [To Do] X-DISKLESS-0007 GCS backend support for the diskless operations
P0 · M13 · Task · deps: X-DISKLESS-0005
Done when: Range reads, `ifGenerationMatch=0` conditional create, resumable or compose uploads, batch delete and paginated list implemented; conformance suite passes against a GCS emulator.

### [To Do] X-DISKLESS-0008 Azure Blob backend support for the diskless operations
P0 · M13 · Task · deps: X-DISKLESS-0005
Done when: Range reads, `If-None-Match: *` conditional create, block blob staging and commit, batch delete and paginated list implemented; conformance suite passes against Azurite.

### [To Do] X-DISKLESS-0009 Retry classification and backoff with seeded jitter per backend
P0 · M13 · Task · deps: X-DISKLESS-0005
Done when: Throttling (S3 503 SlowDown, GCS 429, Azure 503), 5xx and timeouts retry with exponential backoff and jitter from the injected RNG; 4xx except throttling never retry; table-driven tests per backend.

### [To Do] X-DISKLESS-0010 Per-request and total deadlines with cancellation: `alluvium.diskless.object.request.timeout.ms`, `alluvium.diskless.object.upload.timeout.ms`
P0 · M13 · Task · deps: X-DISKLESS-0009, CFG-F11 · labels: config
Done when: Every object request has a deadline; cancelled requests release buffers; keys registered with validators and docs; tests with a stalled fake store.

### [To Do] X-DISKLESS-0011 Bounded request concurrency and connection pools: `alluvium.diskless.object.max.concurrent.requests`
P0 · M13 · Task · deps: X-DISKLESS-0005
Done when: A semaphore per operation class (PUT, GET, LIST, DELETE) enforces the bound; queue depth and rejections exported; tests.

### [To Do] X-DISKLESS-0012 Hedged range GETs after `alluvium.diskless.read.hedge.after.ms`
P1 · M13 · Task · deps: X-DISKLESS-0011
Done when: A second GET is issued when the first exceeds the hedge delay, the first response wins and the loser is cancelled; hedges count against the concurrency bound; tests with a slow fake store.

### [To Do] X-DISKLESS-0013 Credentials: static keys, instance metadata and workload identity per backend, held as `Secret<T>` and rotated without restart
P0 · M13 · Task · deps: X-DISKLESS-0005, SEC-F11 · labels: security
Done when: Credentials come from files or environment, never logs or arguments; rotation picks up new files; OBS-F11 redaction tests cover them.

### [To Do] X-DISKLESS-0014 Fault-injecting object store for diskless simulation: lost-ack PUT, partial multipart, throttling, slow GETs and delayed list visibility
P0 · M13 · Task · deps: X-DISKLESS-0005, TIER-F10 · labels: test
Done when: The TIER-F10 simulated store gains these faults driven by the seeded fault plan; each fault has a unit test.

## [To Do] X-DISKLESS-F03 Diskless write path
P0 · M13 · Feature · deps: X-DISKLESS-F02
Scope: Cross-partition batch coalescing into objects; acknowledge after durable upload.

### [To Do] X-DISKLESS-0015 WAL object format: header, concatenated batches, footer index of batch locations, footer CRC32C
P0 · M13 · Task · deps: X-DISKLESS-0030 · labels: decoder
Done when: docs/formats/diskless-wal-object.md documents magic, version, object id, writer broker id, creation time, batch entries (topic id, partition, position, length, record count, base and max timestamp, producer id, epoch and base sequence, batch CRC) and the footer CRC with a hex example; encoder and bounded decoder round-trip.

### [To Do] X-DISKLESS-0016 Fuzz target for the WAL object decoder
P0 · M13 · Task · deps: X-DISKLESS-0015, TEST-F07 · labels: decoder, security
Done when: A cargo-fuzz target runs in CI without panics, hangs or unbounded allocation; seed corpus includes truncated and oversized footers.

### [To Do] X-DISKLESS-0017 Produce routing: diskless partitions go to the diskless writer, classic partitions are untouched
P0 · M13 · Task · deps: none · ref: core/src/main/scala/kafka/server/ReplicaManager.scala appendRecords @4.4.0
Done when: The produce handler splits a request by topic config; classic partitions follow the existing path with no extra cost (benchmark); tests with mixed requests.

### [To Do] X-DISKLESS-0018 Diskless produce validation identical to classic append validation before buffering
P0 · M13 · Task · deps: X-DISKLESS-0017, REC-F12
Done when: Batches go through REC-F12 validation (CRC, magic, timestamps, size limits, compacted key rule) with the same error codes; LogAppendTime topics get the time stamped and CRC recomputed at buffering; tests per error.

### [To Do] X-DISKLESS-0019 Cross-partition WAL buffer: seal on `alluvium.diskless.wal.object.target.bytes` or `alluvium.diskless.wal.commit.interval.ms`
P0 · M13 · Task · deps: X-DISKLESS-0015, X-DISKLESS-0018
Done when: The sans-I/O accumulator appends validated batches, keeps per (producer, partition) order, and emits `SealObject` on size or age; memory bounded by `alluvium.diskless.produce.buffer.max.bytes`; unit tests with the simulated clock.

### [To Do] X-DISKLESS-0020 Object key layout: prefix, hash-sharded path segment and object UUID from the injected RNG
P0 · M13 · Task · deps: X-DISKLESS-0015
Done when: Keys follow `<prefix>/wal/<shard>/<uuid>`, documented in the format doc; property test: distinct objects never share a key.

### [To Do] X-DISKLESS-0021 Upload driver: PUT or multipart above `alluvium.diskless.multipart.threshold.bytes`, conditional create, durable on success
P0 · M13 · Task · deps: X-DISKLESS-0019, X-DISKLESS-0020
Done when: The driver uploads sealed objects with at most `alluvium.diskless.wal.max.inflight.uploads` in flight; success means durable per the ADR; failures are reported to the writer machine; tests with the fake store.

### [To Do] X-DISKLESS-0022 Commit request to the batch sequencer after upload, per sequencer shard, retried with the same object id
P0 · M13 · Task · deps: X-DISKLESS-0021, X-DISKLESS-F04
Done when: After upload the writer sends one commit per shard covering that shard's batches; retries reuse the object id so the sequencer deduplicates; tests with a simulated sequencer that drops responses.

### [To Do] X-DISKLESS-0023 Produce completion after commit with assigned base offsets and per-partition errors
P0 · M13 · Task · deps: X-DISKLESS-0022 · ref: server/src/main/java/org/apache/kafka/server/purgatory/DelayedProduce.java @4.4.0
Done when: Delayed produce completes when every partition of the request has its commit result; base offsets and log append times come from the commit; partition-level rejections (unknown partition, fenced producer) map to Kafka errors; tests.

### [To Do] X-DISKLESS-0024 Acks semantics: acks=0 immediate, acks=1 and acks=all after commit; `min.insync.replicas` not applicable
P0 · M13 · Task · deps: X-DISKLESS-0023
Done when: Tests per acks value; the difference from classic topics is documented in the compatibility statement.

### [To Do] X-DISKLESS-0025 Produce timeouts while an upload or commit is in flight return REQUEST_TIMED_OUT with unknown outcome
P0 · M13 · Task · deps: X-DISKLESS-0023
Done when: A request whose deadline passes returns REQUEST_TIMED_OUT; the object may still commit later; idempotent producers deduplicate on retry (tested with X-DISKLESS-F10).

### [To Do] X-DISKLESS-0026 Per-producer, per-partition order preserved across consecutive WAL objects
P0 · M13 · Task · deps: X-DISKLESS-0022
Done when: Batches from the same producer and partition commit in send order even when they span objects (the writer commits object N+1's batches for that producer only after N); property test with concurrent uploads finishing out of order.

### [To Do] X-DISKLESS-0027 Buffer-full backpressure: mute connections and account memory per connection
P0 · M13 · Task · deps: X-DISKLESS-0019, NET-F05
Done when: When the WAL buffer is full, connections with pending diskless produce are muted via NET-F05 until space frees; metrics for buffer bytes, capacity and mute events; tests.

### [To Do] X-DISKLESS-0028 Register write-path keys
P0 · M13 · Task · deps: X-DISKLESS-0019, CFG-F11 · labels: config
Done when: `alluvium.diskless.wal.object.target.bytes`, `alluvium.diskless.wal.commit.interval.ms`, `alluvium.diskless.produce.buffer.max.bytes`, `alluvium.diskless.wal.max.inflight.uploads` and `alluvium.diskless.multipart.threshold.bytes` defined in the CFG-F11 registry with validators, defaults and docs.

### [To Do] X-DISKLESS-0029 Writer events and flight recorder: object sealed, uploaded, committed, failed with reasons
P0 · M13 · Task · deps: X-DISKLESS-0022 · labels: observability
Done when: Events `diskless_object_sealed`, `diskless_object_uploaded`, `diskless_object_committed` and `diskless_object_failed` carry object id, bytes, batch count and reason; the writer's flight recorder holds object lifecycle transitions; tests.

## [To Do] X-DISKLESS-F04 Batch sequencer
P0 · M13 · Feature · deps: X-DISKLESS-F01
Scope: Offset assignment and ordering for diskless partitions; object-to-batch metadata.

### [To Do] X-DISKLESS-0030 Sequencer state model: per partition next offset, log start offset and batch index; per shard object registry with reference counts
P0 · M13 · Task · deps: none · ref: KIP-1164
Done when: Creates `alluvium-diskless` (its `//!` doc states responsibility, layer and invariants; xtask layer table and docs/architecture.md list it; no Tokio dependency) with the sans-I/O `Sequencer` state and invariants (offsets contiguous, batch index sorted and non-overlapping, reference counts match the index) checked by `debug_assert!` and property tests.

### [To Do] X-DISKLESS-0031 Commit operation: assign contiguous offsets per partition in object order, atomic per shard
P0 · M13 · Task · deps: X-DISKLESS-0030
Done when: A commit assigns base offsets to every batch, rejects batches for unknown or deleted partitions individually, and is all-or-nothing for the rest; property test: offsets never reused or skipped.

### [To Do] X-DISKLESS-0032 Commit idempotence by object id
P0 · M13 · Task · deps: X-DISKLESS-0031
Done when: A repeated commit of the same object id returns the original result without state change; test with retries after lost responses.

### [To Do] X-DISKLESS-0033 Sequencer records: `DisklessBatchCommitRecord`, `DisklessLogStartRecord`, `DisklessObjectRecord`, `DisklessObjectDeleteRecord`
P0 · M13 · Task · deps: X-DISKLESS-0031 · labels: decoder
Done when: Record schemas with versioned codecs, replay into the sequencer state with validation and typed errors, and a fuzz target per record decoder.

### [To Do] X-DISKLESS-0034 Sequencer shards on the coordinator runtime over `__alluvium_diskless_sequencer`
P0 · M13 · Task · deps: X-DISKLESS-0033, GRP-F01 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntime.java @4.4.0
Done when: Each shard is a coordinator state machine whose writes complete at the internal topic's HW; load on leadership gain, unload on loss; snapshot-at-offset structures for reads; tests with the GRP-F01 runtime test harness.

### [To Do] X-DISKLESS-0035 Internal topic `__alluvium_diskless_sequencer`: creation, partition count, replication and configs
P0 · M13 · Task · deps: X-DISKLESS-0034, CFG-F11 · labels: config
Done when: Auto-created with `alluvium.diskless.sequencer.partitions` partitions, the cluster's default replication factor (at least 3 in production profiles), `cleanup.policy=compact` and min ISR 2; always a classic topic; hidden from tenants; tests.

### [To Do] X-DISKLESS-0036 Diskless partition to sequencer shard mapping and shard discovery
P0 · M13 · Task · deps: X-DISKLESS-0035
Done when: Shard = hash(topic id, partition) modulo shard count, stable for the cluster's lifetime; brokers find the shard leader from the metadata image; tests.

### [To Do] X-DISKLESS-0037 Alluvium-internal inter-broker RPCs: CommitWalObject, FindBatches, GetPartitionState
P0 · M13 · Task · deps: X-DISKLESS-0036 · labels: decoder, security
Done when: Schemas in the Alluvium API key range, generated codecs with round-trip tests and fuzz targets, served on the inter-broker listener with CLUSTER_ACTION authorization, never listed in ApiVersions on client listeners.

### [To Do] X-DISKLESS-0038 Broker batch-location cache for hot partitions with HW-driven invalidation
P0 · M13 · Task · deps: X-DISKLESS-0037
Done when: Brokers cache batch index ranges for partitions they serve, bounded by `alluvium.diskless.location.cache.bytes`; cache entries are immutable below the HW so only the tail is refreshed; tests.

### [To Do] X-DISKLESS-0039 HW propagation: brokers subscribe to HW changes of partitions they serve
P0 · M13 · Task · deps: X-DISKLESS-0037
Done when: The shard pushes HW updates to subscribed brokers with bounded fan-out and coalescing; delayed fetches complete on updates; tests with lagging subscribers.

### [To Do] X-DISKLESS-0040 Sequencer shard failover: in-flight commits retried and deduplicated on the new leader
P0 · M13 · Task · deps: X-DISKLESS-0034, X-DISKLESS-0032
Done when: A shard leader change during commits leads to retries that yield exactly-once commits; tests cut the internal topic at each record.

### [To Do] X-DISKLESS-0041 Sequencer limits: batches per commit, partitions per shard, state memory bound
P0 · M13 · Task · deps: X-DISKLESS-0031
Done when: Over-limit commits fail with typed errors; state memory per shard is bounded and exported; tests.

### [To Do] X-DISKLESS-0042 ListOffsets on diskless partitions: earliest, latest, timestamp and max timestamp from the batch index
P0 · M13 · Task · deps: X-DISKLESS-0030 · ref: core/src/main/scala/kafka/cluster/Partition.scala fetchOffsetForTimestamp @4.4.0
Done when: Results match a classic partition holding the same batches (differential test between a classic and a diskless topic).

### [To Do] X-DISKLESS-0043 Leader epochs on diskless partitions: monotonic, bumped on serving-broker changes, never used for truncation
P0 · M13 · Task · deps: X-DISKLESS-0030
Done when: OffsetForLeaderEpoch returns the HW for the current epoch; clients never see truncation; tests with the Java consumer's epoch validation.

### [To Do] X-DISKLESS-0044 Sequencer events and flight recorder per shard
P0 · M13 · Task · deps: X-DISKLESS-0034 · labels: observability
Done when: Events `diskless_commit_rejected`, `diskless_shard_loaded` and `diskless_shard_unloaded` with reasons; shard flight recorder holds commits and log-start changes; tests.

## [To Do] X-DISKLESS-F05 Diskless read path
P0 · M13 · Feature · deps: X-DISKLESS-F04
Scope: Range reads, local caching, zone-aware cache.

### [To Do] X-DISKLESS-0045 Fetch routing: diskless partitions go to the diskless reader
P0 · M13 · Task · deps: none · ref: core/src/main/scala/kafka/server/ReplicaManager.scala fetchMessages @4.4.0
Done when: The fetch handler splits partitions by topic config; classic partitions unaffected; tests with mixed fetches.

### [To Do] X-DISKLESS-0046 Read planning: batch locations from the fetch offset, coalesced into range GETs
P0 · M13 · Task · deps: X-DISKLESS-0045
Done when: The planner returns range reads covering up to max bytes, merging ranges whose gap is under `alluvium.diskless.read.coalesce.gap.bytes`; property test: planned ranges cover exactly the requested batches.

### [To Do] X-DISKLESS-0047 Batch header patching on read: base offset and partition leader epoch rewritten, body CRC verified
P0 · M13 · Task · deps: X-DISKLESS-0046 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java @4.4.0
Done when: Patched batches are byte-identical to what a classic partition would return (differential test); a CRC mismatch returns KAFKA_STORAGE_ERROR and emits `diskless_batch_corrupt`.

### [To Do] X-DISKLESS-0048 In-memory range cache bounded by `alluvium.diskless.cache.memory.bytes`
P0 · M13 · Task · deps: X-DISKLESS-0046
Done when: Cache keyed by object id and range with the eviction policy from the ADR; correctness never depends on it; eviction tests under pressure.

### [To Do] X-DISKLESS-0049 Optional local disk cache bounded by `alluvium.diskless.cache.disk.bytes` through the fs trait
P1 · M13 · Task · deps: X-DISKLESS-0048
Done when: Cached ranges survive restarts and are CRC-checked on load; corrupt entries are dropped; crash-consistency test.

### [To Do] X-DISKLESS-0050 Write-through: uploaded WAL objects stay in the writer's cache for tail reads
P0 · M13 · Task · deps: X-DISKLESS-0048, X-DISKLESS-F03
Done when: Tail fetches in the writer's zone hit the cache after commit; tests.

### [To Do] X-DISKLESS-0051 Delayed fetch for diskless partitions: min bytes, max wait, completion on HW notifications
P0 · M13 · Task · deps: X-DISKLESS-0046 · ref: core/src/main/scala/kafka/server/DelayedFetch.scala @4.4.0
Done when: Fetches wait for HW updates with SRV-F07 delayed operations; tests per min bytes and max wait.

### [To Do] X-DISKLESS-0052 Fetch sessions and read_committed on diskless partitions
P0 · M13 · Task · deps: X-DISKLESS-0051
Done when: KIP-227 incremental sessions work unchanged; read_committed stops at the LSO and returns aborted transactions from the diskless aborted index; tests.

### [To Do] X-DISKLESS-0053 Single-flight range reads for concurrent fetches of the same range
P0 · M13 · Task · deps: X-DISKLESS-0048
Done when: Concurrent misses on the same range issue one GET; test counts GETs.

### [To Do] X-DISKLESS-0054 Read failures: timeouts and errors yield partial responses with retriable partition errors
P0 · M13 · Task · deps: X-DISKLESS-0046
Done when: Partitions whose reads fail return a retriable error while others return data; tests with the fault store.

### [To Do] X-DISKLESS-0055 Per-fetch and per-broker read memory bounds
P0 · M13 · Task · deps: X-DISKLESS-0046
Done when: Fetch response memory is reserved before GETs and bounded by `alluvium.diskless.read.max.inflight.bytes`; over-budget fetches wait or return partial data; tests.

### [To Do] X-DISKLESS-0056 Register read-path keys
P0 · M13 · Task · deps: X-DISKLESS-0046, CFG-F11 · labels: config
Done when: `alluvium.diskless.read.coalesce.gap.bytes`, `alluvium.diskless.cache.memory.bytes`, `alluvium.diskless.cache.disk.bytes`, `alluvium.diskless.cache.dir`, `alluvium.diskless.read.max.inflight.bytes`, `alluvium.diskless.read.hedge.after.ms` and `alluvium.diskless.location.cache.bytes` defined with validators, defaults and docs.

## [To Do] X-DISKLESS-F06 Stateless broker mode
P0 · M13 · Feature · deps: X-DISKLESS-F03, X-DISKLESS-F05
Scope: Brokers with no local data for diskless topics; fast scaling.

### [To Do] X-DISKLESS-0057 Serving-broker assignment for diskless partitions by the controller, no data movement
P0 · M13 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/metadata/placement/StripedReplicaPlacer.java @4.4.0
Done when: Controller assigns serving brokers per zone, records them in partition records, and reassigns immediately when a broker is fenced with a leader epoch bump; tests.

### [To Do] X-DISKLESS-0058 Stateless broker profile: `alluvium.diskless.broker.stateless=true` excludes the broker from classic replica placement
P0 · M13 · Task · deps: X-DISKLESS-0057, CTRL-F07, CFG-F11 · labels: config
Done when: Stateless brokers never receive classic replicas or internal topic replicas; placement tests; key registered with doc.

### [To Do] X-DISKLESS-0059 Fast scale-out: a new broker serves diskless partitions as soon as it is unfenced
P0 · M13 · Task · deps: X-DISKLESS-0057
Done when: Serving assignment is rebalanced with bounded churn per round; metric `alluvium_diskless_time_to_serve_seconds`; simulation test.

### [To Do] X-DISKLESS-0060 Controlled shutdown flushes the WAL buffer and waits for commits before exit
P0 · M13 · Task · deps: X-DISKLESS-0057, SRV-F03
Done when: On controlled shutdown the broker stops accepting diskless produce, uploads and commits buffered data within the shutdown bound, then exits; test: no acknowledged write lost and no buffered data dropped silently.

### [To Do] X-DISKLESS-0061 Startup and readiness without local state for diskless partitions
P0 · M13 · Task · deps: X-DISKLESS-0058
Done when: A stateless broker becomes ready after metadata catch-up without log loading; cache starts empty; test.

### [To Do] X-DISKLESS-0062 Stateless broker Deployment variant for the Helm chart
P2 · M13 · Task · deps: X-DISKLESS-0058, OPS-F04
Done when: OPS-F04 chart gains a Deployment for stateless brokers with HPA example; chart lint and render tests.

## [To Do] X-DISKLESS-F07 Object compaction
P0 · M13 · Feature · deps: X-DISKLESS-F03
Scope: Merge small objects into partition-ordered segments.

### [To Do] X-DISKLESS-0063 Compaction planner: select WAL objects by age, size and fragmentation into bounded jobs
P0 · M13 · Task · deps: none · ref: KIP-1165
Done when: Sans-I/O planner bounded by `alluvium.diskless.compaction.max.input.bytes`; property test: every batch older than the target age is planned exactly once.

### [To Do] X-DISKLESS-0064 Segment object format: per-partition contiguous batches with offset and time index and CRC
P0 · M13 · Task · deps: none · labels: decoder
Done when: docs/formats/diskless-segment-object.md with a hex example; encoder and bounded decoder; fuzz target.

### [To Do] X-DISKLESS-0065 Compaction job leases assigned by the sequencer shard to brokers with expiry
P0 · M13 · Task · deps: X-DISKLESS-0063
Done when: Jobs carry a lease epoch; an expired lease makes the job reassignable and the old worker's swap is rejected; tests.

### [To Do] X-DISKLESS-0066 Compaction executor: read input ranges, write segment objects, upload
P0 · M13 · Task · deps: X-DISKLESS-0064, X-DISKLESS-0065
Done when: Executor streams inputs with bounded memory and uploads outputs with conditional create; throttled by `alluvium.diskless.compaction.bytes.per.sec`; tests.

### [To Do] X-DISKLESS-0067 Atomic swap in the sequencer: `DisklessObjectSwapRecord` replaces batch locations for exact offset ranges
P0 · M13 · Task · deps: X-DISKLESS-0066 · labels: decoder
Done when: The swap succeeds only if the replaced ranges still reference the input objects; otherwise it is rejected and outputs become orphans; replay and fuzz coverage for the record.

### [To Do] X-DISKLESS-0068 Deferred deletion of replaced objects after `alluvium.diskless.object.delete.delay.ms`
P0 · M13 · Task · deps: X-DISKLESS-0067
Done when: Replaced objects are deleted only after the grace period and when their reference count is zero; tests with readers holding stale locations.

### [To Do] X-DISKLESS-0069 `cleanup.policy=compact` on diskless topics is rejected at creation and alteration with INVALID_CONFIG
P0 · M13 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java @4.4.0
Done when: Rejection with a message pointing to the documentation; test per API.

### [To Do] X-DISKLESS-0070 Crash and failover tests for compaction mid-job and mid-swap
P0 · M13 · Task · deps: X-DISKLESS-0068 · labels: test
Done when: Worker crash, lease expiry and shard failover at each step leave readable data and no lost batches.

### [To Do] X-DISKLESS-0071 Register compaction keys
P0 · M13 · Task · deps: X-DISKLESS-0063, CFG-F11 · labels: config
Done when: `alluvium.diskless.compaction.max.input.bytes`, `alluvium.diskless.compaction.bytes.per.sec`, `alluvium.diskless.compaction.min.age.ms` and `alluvium.diskless.object.delete.delay.ms` defined with validators and docs.

### [To Do] X-DISKLESS-0072 Compaction events and metrics
P0 · M13 · Task · deps: X-DISKLESS-0067 · labels: observability
Done when: Events `diskless_compaction_started`, `diskless_compaction_swapped`, `diskless_compaction_aborted` with reasons; metrics for WAL bytes awaiting compaction and job durations; tests.

## [To Do] X-DISKLESS-F08 Diskless retention
P0 · M13 · Feature · deps: X-DISKLESS-F07
Scope: Retention and deletion in object storage.

### [To Do] X-DISKLESS-0073 Time and size retention evaluated by the sequencer on the batch index
P0 · M13 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java @4.4.0
Done when: `retention.ms` and `retention.bytes` advance the log start offset with a `DisklessLogStartRecord`; tests with the simulated clock.

### [To Do] X-DISKLESS-0074 DeleteRecords on diskless partitions advances the log start offset
P0 · M13 · Task · deps: X-DISKLESS-0073
Done when: API-F21 requests on diskless partitions return the new low watermark; tests.

### [To Do] X-DISKLESS-0075 Object garbage collection: delete objects with zero references after the grace period
P0 · M13 · Task · deps: X-DISKLESS-0073
Done when: Batch deletes respect the concurrency bound; deletion is idempotent; tests.

### [To Do] X-DISKLESS-0076 Orphan collection: objects unknown to the sequencer and older than `alluvium.diskless.orphan.grace.ms`
P0 · M13 · Task · deps: X-DISKLESS-0075
Done when: A paginated LIST scan cross-checks every candidate with the sequencer registry before deleting; never deletes a committed object even with delayed list visibility (simulation); metric of orphans found and deleted.

### [To Do] X-DISKLESS-0077 Incomplete multipart uploads aborted after the orphan grace period
P0 · M13 · Task · deps: X-DISKLESS-0076
Done when: The collector lists and aborts stale multipart uploads per backend; tests with the fault store.

### [To Do] X-DISKLESS-0078 Topic deletion reclaims all diskless data
P0 · M13 · Task · deps: X-DISKLESS-0075
Done when: Deleting a diskless topic removes its partitions from the sequencer, drops references, and objects become collectable; tests including shared WAL objects with other topics.

### [To Do] X-DISKLESS-0079 Register `alluvium.diskless.orphan.grace.ms` and `alluvium.diskless.gc.interval.ms`
P0 · M13 · Task · deps: X-DISKLESS-0076, CFG-F11 · labels: config
Done when: Keys defined with validators (grace must exceed the upload timeout) and docs.

## [To Do] X-DISKLESS-F09 Zone awareness
P1 · M13 · Feature · deps: X-DISKLESS-F05
Scope: Rack-aware routing to avoid cross-zone traffic.

### [To Do] X-DISKLESS-0080 Client zone resolution: listener zone, CIDR map and client id tag, cached per connection
P1 · M13 · Task · deps: none · ref: ADR from X-DISKLESS-0001 (diskless topics)
Done when: Precedence and syntax decided in the ADR: `alluvium.diskless.listener.zone.<listener>`, `alluvium.diskless.client.zone.cidrs` and a client id tag; resolved after authentication; tests per source.

### [To Do] X-DISKLESS-0081 Zone-aware Metadata and DescribeTopicPartitions: diskless partitions report a serving broker in the client's zone
P1 · M13 · Task · deps: X-DISKLESS-0080, X-DISKLESS-F06 · ref: clients/src/main/resources/common/message/MetadataResponse.json @4.4.0
Done when: Responses are tailored per connection zone with a consistent leader epoch; Java, librdkafka and franz-go accept per-client leaders (compatibility test).

### [To Do] X-DISKLESS-0082 Fetch `rack_id` (KIP-392) used to keep reads in the consumer's zone
P1 · M13 · Task · deps: X-DISKLESS-0081 · ref: clients/src/main/java/org/apache/kafka/common/replica/RackAwareReplicaSelector.java @4.4.0
Done when: Preferred read replica for diskless partitions is a serving broker in the consumer's rack; tests per Fetch version that has rack id.

### [To Do] X-DISKLESS-0083 Zone peer cache: rendezvous hashing of object ranges to brokers within a zone
P1 · M13 · Task · deps: X-DISKLESS-0080 · labels: decoder
Done when: Brokers fetch missed ranges from the owning zone peer before the object store when `alluvium.diskless.cache.peer.enable` is on; an internal RPC with fuzz coverage serves them; tests.

### [To Do] X-DISKLESS-0084 Fallback to cross-zone serving when a zone has no live broker, counted by a metric
P1 · M13 · Task · deps: X-DISKLESS-0081
Done when: `alluvium_diskless_cross_zone_bytes_total{direction}` counts cross-zone bytes; tests with a zone outage.

### [To Do] X-DISKLESS-0085 Register zone keys
P1 · M13 · Task · deps: X-DISKLESS-0080, CFG-F11 · labels: config
Done when: `alluvium.diskless.listener.zone.<listener>`, `alluvium.diskless.client.zone.cidrs` and `alluvium.diskless.cache.peer.enable` defined with validators and docs.

### [To Do] X-DISKLESS-0086 Simulation with three zones: no cross-zone inter-broker bytes in steady state
P1 · M13 · Task · deps: X-DISKLESS-0083, X-DISKLESS-0084, TEST-F06 · labels: test
Done when: Seeded simulation asserts zero cross-zone produce and fetch bytes between brokers when every zone has a serving broker.

## [To Do] X-DISKLESS-F10 Diskless idempotence and transactions
P1 · M13 · Feature · deps: X-DISKLESS-F04, TXN-F01
Scope: Producer state and transactions on diskless topics.

### [To Do] X-DISKLESS-0087 Producer state in the sequencer: epoch and last five batch sequences per (partition, producer id)
P1 · M13 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java @4.4.0
Done when: Validation happens at commit: duplicates return the original offsets, gaps return OUT_OF_ORDER_SEQUENCE_NUMBER, old epochs return INVALID_PRODUCER_EPOCH; tests ported from Kafka's producer state tests.

### [To Do] X-DISKLESS-0088 Producer state persistence, snapshots and expiration in sequencer records
P1 · M13 · Task · deps: X-DISKLESS-0087 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java @4.4.0
Done when: Producer state survives shard failover; `producer.id.expiration.ms` expiry matches classic behavior; tests.

### [To Do] X-DISKLESS-0089 AddPartitionsToTxn verification for diskless partitions (KIP-890 part 1)
P1 · M13 · Task · deps: X-DISKLESS-0087
Done when: Transactional batches for partitions not in the transaction are rejected with the same errors as classic topics; tests.

### [To Do] X-DISKLESS-0090 Transaction markers on diskless partitions written by the sequencer as control batches
P1 · M13 · Task · deps: X-DISKLESS-0089, TXN-F08 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala @4.4.0
Done when: WriteTxnMarkers for diskless partitions append marker batches through the sequencer (stored as decided in the ADR) and update the LSO; tests.

### [To Do] X-DISKLESS-0091 Aborted transaction index for diskless partitions
P1 · M13 · Task · deps: X-DISKLESS-0090 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TransactionIndex.java @4.4.0
Done when: read_committed fetches receive the aborted transaction list identical to a classic partition with the same history (differential test).

### [To Do] X-DISKLESS-0092 Transactions v2 (KIP-890 part 2) epoch bumps on diskless partitions
P1 · M13 · Task · deps: X-DISKLESS-0090, TXN-F10
Done when: Per-transaction epoch bumps and implicit partition addition behave as on classic topics; tests.

### [To Do] X-DISKLESS-0093 Exactly-once compatibility test: Kafka Streams EOS application on diskless topics
P1 · M13 · Task · deps: X-DISKLESS-0091, TEST-F08 · labels: test
Done when: An EOS Streams app with diskless input and output topics produces exactly-once results under broker restarts.

## [To Do] X-DISKLESS-F11 Mixed-mode clusters
P0 · M13 · Feature · deps: X-DISKLESS-F03
Scope: Classic and diskless topics side by side; per-topic config.

### [To Do] X-DISKLESS-0094 Topic config `alluvium.diskless.enable`: set at creation, immutable afterwards
P0 · M13 · Task · deps: CFG-F11 · labels: config
Done when: Registered topic key; rejected when the cluster switch is off; IncrementalAlterConfigs changing it returns INVALID_CONFIG; tests.

### [To Do] X-DISKLESS-0095 Feature level `alluvium.diskless.version` gating diskless records and configs, downgrade blocked while diskless topics exist
P0 · M13 · Task · deps: X-DISKLESS-0094 · ref: server-common/src/main/java/org/apache/kafka/server/common/MetadataVersion.java @4.4.0
Done when: With level 0 no diskless record is emitted and the topic config is rejected; downgrade below 1 with diskless topics fails with INVALID_UPDATE_VERSION; tests.

### [To Do] X-DISKLESS-0096 Internal topics are always classic
P0 · M13 · Task · deps: X-DISKLESS-0094
Done when: Creating `__consumer_offsets`, `__transaction_state`, `__alluvium_diskless_sequencer` or other internal topics as diskless is rejected; tests.

### [To Do] X-DISKLESS-0097 Classic topic paths unaffected in mixed clusters
P0 · M13 · Task · deps: X-DISKLESS-0094, TEST-F11 · labels: perf
Done when: Benchmarks show classic produce and fetch throughput and latency within 1% with diskless enabled and active on other topics.

### [To Do] X-DISKLESS-0098 CreatePartitions on diskless topics registers new partitions with the sequencer
P0 · M13 · Task · deps: X-DISKLESS-0094
Done when: New partitions start at offset 0 and are served immediately; tests.

### [To Do] X-DISKLESS-0099 Admin behavior on diskless partitions: DescribeLogDirs omits them; reassignment returns INVALID_REPLICA_ASSIGNMENT
P0 · M13 · Task · deps: X-DISKLESS-0094, TEST-F08
Done when: kafka-topics.sh and kafka-reassign-partitions.sh compatibility tests match the documented behavior.

## [To Do] X-DISKLESS-F12 Object store failure handling
P0 · M13 · Feature · deps: X-DISKLESS-F03
Scope: Backpressure during outages, retries, documented guarantees.

### [To Do] X-DISKLESS-0100 Object store health machine: Healthy, Degraded, Unavailable from error rates and timeouts
P0 · M13 · Task · deps: none · ref: ADR from X-DISKLESS-0001 (diskless topics)
Done when: Transitions emit `diskless_object_store_health` events with reasons and are in the writer flight recorder; tests with the fault store.

### [To Do] X-DISKLESS-0101 Produce during outages: buffer then mute, fail after `alluvium.diskless.produce.max.block.ms` with a retriable error
P0 · M13 · Task · deps: X-DISKLESS-0100, CFG-F11
Done when: No acknowledged write is lost; unacknowledged batches fail with the ADR error (retriable) and producers retry; key registered; tests.

### [To Do] X-DISKLESS-0102 Lost-ack PUT: conditional create on retry detects the existing object and verifies its checksum before commit
P0 · M13 · Task · deps: X-DISKLESS-0100
Done when: A retry receiving AlreadyExists reads the object's footer CRC and proceeds to commit if equal, or fails the batches if different; tests.

### [To Do] X-DISKLESS-0103 Partial multipart upload: abort and fail the object's batches with a retriable error
P0 · M13 · Task · deps: X-DISKLESS-0100
Done when: Failed multipart uploads are aborted; batches fail; orphan collection removes leftovers; tests.

### [To Do] X-DISKLESS-0104 Sequencer unavailable after upload: commit retries until the produce deadline, then orphan
P0 · M13 · Task · deps: X-DISKLESS-0100
Done when: Uncommitted objects are left for orphan collection; producers get REQUEST_TIMED_OUT; tests.

### [To Do] X-DISKLESS-0105 Committed object missing on read raises a data-loss alarm
P0 · M13 · Task · deps: X-DISKLESS-0100
Done when: A 404 for a committed object returns KAFKA_STORAGE_ERROR, emits `diskless_object_missing` at error level and increments a metric; tests.

### [To Do] X-DISKLESS-0106 Adaptive concurrency on throttling: additive increase, multiplicative decrease per operation class
P0 · M13 · Task · deps: X-DISKLESS-0100
Done when: Throttle responses halve the allowed concurrency, successes grow it by one; tests with a throttling fake store.

### [To Do] X-DISKLESS-0107 Documented guarantees and availability matrix per failure
P0 · M13 · Task · deps: X-DISKLESS-0101, X-DISKLESS-0104, X-DISKLESS-0105 · labels: docs
Done when: The diskless guide states durability, availability and latency behavior for object store down, sequencer shard down, zone down and broker down, each linked to its simulation test.

## [To Do] X-DISKLESS-F13 Cost and performance metrics
P1 · M13 · Feature · deps: X-DISKLESS-F03, OBS-F04
Scope: Request counts, bytes, cache hit rates, cost estimates.

### [To Do] X-DISKLESS-0108 Object store metrics: requests by operation, backend and result; latency; bytes by direction
P1 · M13 · Task · deps: none · labels: observability
Done when: `alluvium_diskless_object_requests_total{op,result}`, `alluvium_diskless_object_request_seconds{op}` and `alluvium_diskless_object_bytes_total{direction}` registered and asserted in tests.

### [To Do] X-DISKLESS-0109 Write and read path metrics: object sizes, commit latency, buffer bytes, cache hits and bytes
P1 · M13 · Task · deps: none · labels: observability
Done when: `alluvium_diskless_wal_object_bytes`, `alluvium_diskless_commit_seconds`, `alluvium_diskless_produce_buffer_bytes` with capacity, `alluvium_diskless_cache_hits_total`, `alluvium_diskless_cache_misses_total` and `alluvium_diskless_cache_bytes` registered and asserted.

### [To Do] X-DISKLESS-0110 Sequencer metrics: commits, commit batch counts, rejected commits by reason, state bytes per shard
P1 · M13 · Task · deps: none · labels: observability
Done when: `alluvium_diskless_sequencer_commits_total{result}`, `alluvium_diskless_sequencer_batches_per_commit` and `alluvium_diskless_sequencer_state_bytes` registered in the shard driver and asserted in tests.

### [To Do] X-DISKLESS-0111 Cost estimate metrics from a configurable price table
P2 · M13 · Task · deps: X-DISKLESS-0108, CFG-F11
Done when: `alluvium.diskless.cost.put.per.thousand`, `alluvium.diskless.cost.get.per.thousand`, `alluvium.diskless.cost.storage.gb.month` and `alluvium.diskless.cost.cross.zone.gb` feed `alluvium_diskless_estimated_cost_total{category}`; unit tests against the ADR cost model.

### [To Do] X-DISKLESS-0112 Diskless event catalog and generated docs
P1 · M13 · Task · deps: OBS-F07 · labels: docs
Done when: All `diskless_*` events listed in `events.rs` with triggers and reasons; docs/events.md regenerated.

### [To Do] X-DISKLESS-0113 Standard log fields `object_id`, `sequencer_shard` and `zone`
P1 · M13 · Task · deps: OBS-F03 · labels: docs
Done when: Added to the debuggability standard field table and constants; field check passes.

### [To Do] X-DISKLESS-0114 Diagnostics routes `/debug/diskless/partitions/{topic}/{partition}` and `/debug/diskless/objects/{id}`
P1 · M13 · Task · deps: OBS-F08
Done when: Routes return sequencer state, batch locations and flight recorders as JSON; authenticated when not on loopback; tests.

### [To Do] X-DISKLESS-0115 Dashboards and alert rules for diskless
P2 · M13 · Task · deps: X-DISKLESS-0108, X-DISKLESS-0109, OPS-F11
Done when: Panels for object requests, commit latency, cache hit rate, orphan growth and cross-zone bytes; alerts for missing objects and outage; promtool tests.

## [To Do] X-DISKLESS-F14 Diskless testing, benchmarks and documentation
P0 · M13 · Feature · deps: X-DISKLESS-F03, TEST-F06
Scope: Simulation with a faulty object store, benchmarks, guide.

### [To Do] X-DISKLESS-0116 Checker: no acknowledged diskless write is lost
P0 · M13 · Task · deps: TEST-F05 · labels: test
Done when: Every acknowledged batch is readable at its assigned offset until retention removes it.

### [To Do] X-DISKLESS-0117 Checker: offsets contiguous and unique; every broker returns identical bytes for an offset
P0 · M13 · Task · deps: TEST-F05 · labels: test
Done when: Reads from brokers in all zones agree byte for byte.

### [To Do] X-DISKLESS-0118 Checker: no object referenced by the sequencer is ever deleted
P0 · M13 · Task · deps: TEST-F05 · labels: test
Done when: The fault store records deletes and the checker cross-checks them with the sequencer state after every step.

### [To Do] X-DISKLESS-0119 Checker: idempotent producers never produce duplicates on diskless topics
P0 · M13 · Task · deps: TEST-F05 · labels: test
Done when: Sequence numbers per producer and partition appear at most once in the log.

### [To Do] X-DISKLESS-0120 Checker: fetches never return data beyond the HW or, for read_committed, the LSO
P0 · M13 · Task · deps: TEST-F05 · labels: test
Done when: Every fetch response is checked against the sequencer state at the time of the request.

### [To Do] X-DISKLESS-0121 Simulation with a faulty object store: latency, errors, throttling, lost acks and partial multipart
P0 · M13 · Task · deps: X-DISKLESS-0116, X-DISKLESS-0117, X-DISKLESS-0118, X-DISKLESS-0119, X-DISKLESS-0120 · labels: test
Done when: Seeded runs with each fault type pass all checkers; failing seeds join the fixed set.

### [To Do] X-DISKLESS-0122 Simulation: broker crash after upload before commit, sequencer failover mid-commit, compaction worker crash mid-swap
P0 · M13 · Task · deps: X-DISKLESS-0121 · labels: test
Done when: All checkers pass; orphans are eventually collected.

### [To Do] X-DISKLESS-0123 Simulation: cache eviction pressure and zone outages
P0 · M13 · Task · deps: X-DISKLESS-0121 · labels: test
Done when: Tiny caches and zone failures never affect correctness; all checkers pass.

### [To Do] X-DISKLESS-0124 Nightly diskless simulation with thousands of seeds
P0 · M13 · Task · deps: X-DISKLESS-0122, X-DISKLESS-0123 · labels: test
Done when: 5000 seeds per night across fault plans; CI runs fixed seeds plus 20 new per PR.

### [To Do] X-DISKLESS-0125 Diskless is off by default: no sequencer topic, configs rejected, no object store connections
P0 · M13 · Task · deps: none · labels: test
Done when: With default config the feature level is 0, `alluvium.diskless.enable` topic config is rejected, `__alluvium_diskless_sequencer` is not created and no object store client is constructed; test.

### [To Do] X-DISKLESS-0126 Kafka compatibility and differential suites pass with diskless enabled but unused
P0 · M13 · Task · deps: X-DISKLESS-0125, TEST-F09 · labels: test
Done when: CI job runs TEST-F08 and TEST-F09 suites with the switch on and an object store configured but no diskless topics; zero new diffs.

### [To Do] X-DISKLESS-0127 Compatibility suite with every test topic diskless
P0 · M13 · Task · deps: X-DISKLESS-0126 · labels: test
Done when: The TEST-F08 suite passes with diskless topics except documented deviations (latency, acks=1 durability, per-zone leaders, compacted topics rejected), each listed in the compatibility statement.

### [To Do] X-DISKLESS-0128 Chaos test against MinIO with real processes
P0 · M13 · Task · deps: TEST-F12 · labels: test
Done when: TEST-F12 chaos runs with process kills and MinIO restarts verify no acknowledged loss and ordering.

### [To Do] X-DISKLESS-0129 Benchmarks: throughput, produce and end-to-end latency, object requests per GB versus the cost model
P0 · M13 · Task · deps: TEST-F11 · labels: perf, nfr
Done when: Results for several WAL object sizes and commit intervals against MinIO are recorded in the benchmark job and compared with the ADR model; classic-topic baseline included.

### [To Do] X-DISKLESS-0130 Concepts guide: diskless topics, write and read paths, guarantees and trade-offs
P0 · M13 · Task · deps: none · labels: docs
Done when: docs/guides/diskless-topics.md with diagrams and the guarantees table; reviewed against the ADR.

### [To Do] X-DISKLESS-0131 Operations guide: bucket setup, IAM policies, lifecycle rules, cache sizing and runbooks
P0 · M13 · Task · deps: X-DISKLESS-0130 · labels: docs
Done when: Covers each backend, required permissions, multipart lifecycle rule, sizing of caches and buffers, and runbooks for outage, orphan growth and missing objects.

### [To Do] X-DISKLESS-0132 Migration and opt-in guide: enabling the feature, creating diskless topics, moving data from classic topics
P0 · M13 · Task · deps: X-DISKLESS-0130 · labels: docs
Done when: Explains the feature level, cluster switch, per-topic opt-in, that in-place conversion is not supported, and migration by mirroring; links check.

### [To Do] X-DISKLESS-0133 Kafka compatibility statement for diskless topics in the compatibility guide
P0 · M13 · Task · deps: X-DISKLESS-0130, DOCS-F10 · labels: docs
Done when: DOCS-F10's guide lists every client-visible difference on diskless topics with links to the ADR.

## [To Do] X-DISKLESS-F15 Key compaction for diskless topics
P2 · M13 · Feature · deps: X-DISKLESS-F07, STOR-F14
Scope: `cleanup.policy=compact` on diskless topics: compaction over object-stored segments with Kafka's tombstone and transaction-marker semantics.

### [To Do] X-DISKLESS-0134 Spike: compaction model for diskless topics
P2 · M13 · Spike · deps: none
Done when: an accepted ADR defines how the cleaner reads and rewrites object-stored segments, how the offset map is bounded, how compacted objects replace originals atomically in sequencer metadata, and the cost model.

### [To Do] X-DISKLESS-0135 Cleaner over object-stored segments
P2 · M13 · Task · deps: X-DISKLESS-0134
Done when: the cleaner selects dirty diskless partitions by dirty ratio and compaction lags, builds a bounded offset map from object range reads, and writes compacted segments as new objects; unit tests cover each selection rule.

### [To Do] X-DISKLESS-0136 Atomic replacement of compacted objects
P2 · M13 · Task · deps: X-DISKLESS-0135
Done when: compacted objects replace their sources in a single sequencer metadata update; readers never see a gap or a duplicate; superseded objects are deleted only after a grace period; a simulation test crashes the cleaner at every step.

### [To Do] X-DISKLESS-0137 Tombstones and transaction markers in diskless compaction
P2 · M13 · Task · deps: X-DISKLESS-0135
Done when: tombstones are kept until `delete.retention.ms` past the delete horizon and aborted-transaction data and markers are removed as Kafka's cleaner does; property tests compare the result with the classic cleaner on the same input.

### [To Do] X-DISKLESS-0138 Allow `cleanup.policy=compact` on diskless topics
P2 · M13 · Task · deps: X-DISKLESS-0136
Done when: topic creation and config validation accept compact and compact,delete for diskless topics once this feature is enabled; the earlier rejection is removed; tests cover both policies.

### [To Do] X-DISKLESS-0139 Diskless compaction metrics, benchmarks and docs
P2 · M13 · Task · deps: X-DISKLESS-0136, OBS-F04
Done when: cleaner metrics (bytes read and written, objects replaced, lag) are exported; a benchmark reports object requests per GB compacted; the diskless guide documents compaction and its cost.
