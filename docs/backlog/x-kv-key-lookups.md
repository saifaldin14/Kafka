# X-KV: Key lookups and queryable topics

> Crates: `alluvium-kv` (new, layer 1) · Kafka counterpart: none (Alluvium enhancement) · Milestones: M12-M13

Point lookups and range scans by key on compacted topics, maintained incrementally from the log.

## [To Do] X-KV-F01 Key index design
P0 · M12 · Feature · deps: STOR-F24
Scope: Spike and ADR: index engine, consistency model, storage layout.

### [To Do] X-KV-0001 Key index ADR: index engine, entry format, consistency model, storage layout and opt-in
P0 · M12 · Spike · deps: none
Done when: docs/adr/NNNN-key-index.md is accepted and decides: in-house LSM (memtable plus sorted runs) versus an embedded crate (with the security-standard dependency review), index entries as pointers into the log (offset plus position hint) rather than copied values, visibility at the high watermark and LSO, transactional record handling, behavior under compaction, retention and truncation, per-topic opt-in via `alluvium.kv.index.enable`, memory and disk bounds, partition routing for lookups, and rejected alternatives (external database, value copies, client-side state stores).

### [To Do] X-KV-0002 Key index threat model: existence oracles, key-length and scan DoS, index-size exhaustion, corrupt index files
P0 · M12 · Task · deps: X-KV-0001 · labels: security, docs
Done when: The threat model in docs/standards/security.md gains rows for unauthorized key-existence probing, oversized keys, unbounded scans, disk exhaustion by indexes and tampered index files on disk, each naming its enforcing task.

### [To Do] X-KV-0003 Glossary entries for key index, memtable, sorted run, index watermark and point lookup
P1 · M12 · Task · deps: X-KV-0001 · labels: docs
Done when: docs/glossary.md defines the terms with links to the ADR; link check passes.

## [To Do] X-KV-F02 Incremental key index
P0 · M12 · Feature · deps: X-KV-F01
Scope: Maintain a per-key index from the append observer hook.

### [To Do] X-KV-0004 Create `alluvium-kv` (layer 1) with the `KeyIndex` state machine and register its layer
P0 · M12 · Task · deps: none
Done when: The crate's `lib.rs` `//!` doc states responsibility, layer, invariants and that it has no Kafka counterpart; `xtask check-layers` and docs/architecture.md list it at layer 1 with no Tokio dependency; `KeyIndex::apply(BatchSummary) -> IndexActions` exists with unit tests for a single batch.

### [To Do] X-KV-0005 Index entry model: key bytes, offset, position hint, record size, timestamp, tombstone flag
P0 · M12 · Task · deps: X-KV-0004
Done when: `IndexEntry` with checked construction; keys longer than `alluvium.kv.max.key.bytes` are skipped with a counted reason instead of indexed; property test for ordering and encoding round-trip.

### [To Do] X-KV-0006 Register the key index as a STOR-F24 append observer for topics with `alluvium.kv.index.enable=true`
P0 · M12 · Task · deps: X-KV-0004, STOR-F24
Done when: Indexed partitions receive one `BatchSummary` per committed batch (keys, positions, timestamps, control and transactional flags) and non-indexed partitions pay no cost (benchmark shows no change on the append path); test enabling and disabling at runtime.

### [To Do] X-KV-0007 Bounded memtable with `alluvium.kv.memtable.bytes` and a flush trigger action
P0 · M12 · Task · deps: X-KV-0005
Done when: Memtable keeps the latest entry per key; reaching the bound emits `FlushRequested`; memory accounting is exact to within 1% in a test; overwrites do not grow memory.

### [To Do] X-KV-0008 Index watermark: `indexed_offset` exported per partition and compared with the HW
P0 · M12 · Task · deps: X-KV-0006
Done when: The machine reports the last offset whose batch is fully indexed; lookups use it to decide whether to wait; tests show it never exceeds the HW and never moves backwards except on truncation.

### [To Do] X-KV-0009 Transactional records: pending per producer id until the commit marker, dropped on abort
P0 · M12 · Task · deps: X-KV-0007 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TransactionIndex.java @4.4.0
Done when: Entries from transactional batches stay pending until the COMMIT marker (then visible) or the ABORT marker (discarded); pending entries are bounded by `alluvium.kv.max.pending.txn.bytes` with a typed error and event on overflow; tests including interleaved transactions and a marker for an unknown producer.

### [To Do] X-KV-0010 Tombstones: null values recorded as tombstone entries and reported as not found
P0 · M12 · Task · deps: X-KV-0007
Done when: A null-value record replaces the key's entry with a tombstone; lookups return NotFound with the tombstone offset available for diagnostics; tests.

### [To Do] X-KV-0011 Index rollback on log truncation and log-start advance
P0 · M12 · Task · deps: X-KV-0008 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java @4.4.0
Done when: On truncation to offset T, entries with offset >= T are removed and `indexed_offset` reset; entries below the log start offset are treated as absent at lookup and purged lazily; unit tests for follower truncation, unclean election truncation and DeleteRecords.

### [To Do] X-KV-0012 Indexing lag bound: `alluvium.kv.max.lag.bytes` applies backpressure on the observer queue
P0 · M12 · Task · deps: X-KV-0008
Done when: When indexing falls behind by more than the bound the observer queue is full and the driver signals backpressure to the partition owner as defined in the ADR; metric and event `kv_index_lagging` with reason; test with a slow simulated index.

### [To Do] X-KV-0013 Register core keys: `alluvium.kv.enable`, `alluvium.kv.index.enable`, `alluvium.kv.max.key.bytes`, `alluvium.kv.memtable.bytes`, `alluvium.kv.max.lag.bytes`, `alluvium.kv.max.pending.txn.bytes`
P0 · M12 · Task · deps: X-KV-0004, CFG-F11 · labels: config
Done when: Keys defined in the CFG-F11 registry (cluster or topic scope as per the ADR) with validators, defaults and docs; topic key rejected with INVALID_CONFIG while the cluster switch is off; generated config docs list them.

### [To Do] X-KV-0014 Index metrics: indexed-offset lag, entries, memtable bytes, flush and merge durations
P0 · M12 · Task · deps: X-KV-0008 · labels: observability
Done when: `alluvium_kv_index_lag_offsets{topic}`, `alluvium_kv_index_entries{topic}`, `alluvium_kv_memtable_bytes`, `alluvium_kv_flush_seconds` and `alluvium_kv_merge_seconds` registered in the driver with bounded labels and asserted in tests.

### [To Do] X-KV-0015 Property test: the in-memory index equals a model map for random batches, transactions, tombstones and truncations
P0 · M12 · Task · deps: X-KV-0009, X-KV-0010, X-KV-0011 · labels: test
Done when: A proptest drives arbitrary sequences and compares every key's entry with the model; regressions persisted.

### [To Do] X-KV-0016 Index events and flight-recorder entries
P0 · M12 · Task · deps: X-KV-0008 · labels: observability
Done when: `kv_index_enabled`, `kv_index_disabled`, `kv_index_flushed`, `kv_index_truncated` and `kv_index_lagging` carry topic, partition, offsets and reason; the partition flight recorder includes them; tests.

## [To Do] X-KV-F03 Index persistence and rebuild
P0 · M12 · Feature · deps: X-KV-F02
Scope: Durable index, crash recovery, rebuild from the log.

### [To Do] X-KV-0017 Sorted run file format: data blocks, block index, bloom filter, footer with format version and CRC32C
P0 · M12 · Task · deps: none · labels: decoder
Done when: docs/formats/kv-index.md documents the format with a hex example; encoder and decoder round-trip; the decoder bounds every length before allocating and returns typed errors.

### [To Do] X-KV-0018 Memtable flush to a sorted run through the fs trait with tmp, fsync, rename and directory fsync
P0 · M12 · Task · deps: X-KV-0017
Done when: A flushed run is either fully present and referenced by the manifest or absent; written under the partition directory's `kv/` subdirectory; test with the in-memory fs.

### [To Do] X-KV-0019 Index manifest: list of runs and the checkpointed `indexed_offset`, atomic replacement
P0 · M12 · Task · deps: X-KV-0017
Done when: Manifest format versioned with CRC32C; updated atomically after each flush and merge; recovery ignores orphan run files and deletes them; tests.

### [To Do] X-KV-0020 Fuzz targets for the sorted run and manifest decoders
P0 · M12 · Task · deps: X-KV-0017, X-KV-0019, TEST-F07 · labels: decoder, security
Done when: Two cargo-fuzz targets run in CI per PR and nightly without panics, hangs or unbounded allocation.

### [To Do] X-KV-0021 Run merging (LSM compaction) with bounded I/O and tombstone handling
P0 · M12 · Task · deps: X-KV-0018, X-KV-0019
Done when: Runs are merged per the ADR policy; tombstones are kept until no older run contains the key; merge I/O is bounded by `alluvium.kv.merge.bytes.per.sec`; property test: lookups are identical before and after any merge.

### [To Do] X-KV-0022 Crash recovery: load the manifest, validate run CRCs, replay the log from the checkpoint to rebuild the memtable
P0 · M12 · Task · deps: X-KV-0019, STOR-F09
Done when: After restart the index serves lookups equal to the pre-crash committed state; a corrupt run triggers a full rebuild with event `kv_index_rebuild{reason=corrupt_run}`; tests.

### [To Do] X-KV-0023 Full rebuild from the log at `alluvium.kv.rebuild.bytes.per.sec`; lookups return INDEX_NOT_READY meanwhile
P0 · M12 · Task · deps: X-KV-0022
Done when: Rebuild scans from the log start offset with bounded rate and memory; lookups during rebuild return the retriable `IndexNotReady` error with progress; tests with the simulated clock.

### [To Do] X-KV-0024 Register `alluvium.kv.merge.bytes.per.sec` and `alluvium.kv.rebuild.bytes.per.sec`
P0 · M12 · Task · deps: X-KV-0021, CFG-F11 · labels: config
Done when: Keys defined in the CFG-F11 registry with validators, defaults, dynamic update support and docs; the rate limiters pick up dynamic changes; tests.

### [To Do] X-KV-0025 Crash-consistency tests at every index write point
P0 · M12 · Task · deps: X-KV-0021, X-KV-0022, TEST-F03 · labels: test
Done when: Using the TEST-F03 fault-injecting fs, crashes at each write and fsync of flush, merge and manifest replacement recover to a correct index; torn writes are detected by CRC.

### [To Do] X-KV-0026 Enabling and disabling the index on an existing topic
P0 · M12 · Task · deps: X-KV-0023
Done when: Enabling starts a background build per partition (progress visible in DescribeIndex); disabling removes the observer and deletes index files asynchronously; tests for toggling during a build.

### [To Do] X-KV-0027 Index disk usage counted by disk-full protection and reported per partition
P0 · M12 · Task · deps: X-KV-0018, STOR-F25
Done when: Index bytes count toward STOR-F25 reserved-space checks; a full disk stops index flushes with event `kv_index_disk_full` without affecting log appends; metric `alluvium_kv_index_bytes{topic}`; tests.

### [To Do] X-KV-0028 Index decoder and printer for `alluvium dump kv-index` and `verify kv-index`
P2 · M12 · Task · deps: X-KV-0017, CLI-F04 · labels: debuggability
Done when: Printer outputs runs, key counts, offset ranges and key hashes (never key bytes unless `--payload`); verify compares the index with a log scan; golden tests; wired by CLI-F04.

## [To Do] X-KV-F04 Point lookups
P0 · M12 · Feature · deps: X-KV-F02, X-NATIVE-F03
Scope: Lookup API with read-at-HW consistency.

### [To Do] X-KV-0029 Native IDL: KvService Get and BatchGet with isolation level and `min_offset`
P0 · M12 · Task · deps: X-NATIVE-F02
Done when: Messages include topic, key, optional partition, isolation level and min offset; responses carry value, headers, offset, timestamp, partition and the high watermark used; breaking-change check passes.

### [To Do] X-KV-0030 Lookup partition resolution: Kafka default partitioner hash, explicit partition, or the elastic mapping and lineage
P0 · M12 · Task · deps: X-KV-0029, X-ELASTIC-F02 · ref: clients/src/main/java/org/apache/kafka/clients/producer/internals/BuiltInPartitioner.java partitionForKey @4.4.0
Done when: Without an explicit partition the key maps with murmur2 modulo partition count (golden vectors shared with the producer partitioner); elastic topics resolve through the mapping and fall back to sealed parents when the child has no entry; tests.

### [To Do] X-KV-0031 Lookups served by the partition leader with NOT_LEADER handling and a leader hint
P0 · M12 · Task · deps: X-KV-0029
Done when: A lookup on a non-leader returns the native not-leader error with the current leader; the SDK retries once against the hinted leader; tests.

### [To Do] X-KV-0032 Read-at-HW consistency: wait for `indexed_offset` to reach the HW observed at arrival, bounded by `alluvium.kv.lookup.max.wait.ms`
P0 · M12 · Task · deps: X-KV-0031
Done when: Lookups see every record below the HW at arrival; `min_offset` waits until the HW reaches it (read-your-writes); timeouts return a retriable error; tests with a lagging index.

### [To Do] X-KV-0033 Value fetch from the log: read the batch at the pointer, verify CRC and key equality, bounded decompression
P0 · M12 · Task · deps: X-KV-0032
Done when: The record is read by offset using the position hint, the batch CRC is verified, the stored key is compared with the requested key, decompression is bounded by `message.max.bytes`; a stale hint falls back to the offset index; tests per compression type.

### [To Do] X-KV-0034 Isolation levels: read_committed limits visibility to the LSO, read_uncommitted to the HW
P0 · M12 · Task · deps: X-KV-0033
Done when: Tests with open and aborted transactions show read_committed never returns uncommitted or aborted values.

### [To Do] X-KV-0035 BatchGet with `alluvium.kv.max.keys.per.request` and per-response byte limits
P0 · M12 · Task · deps: X-KV-0033
Done when: Requests over the key limit fail validation; responses stop at the byte limit with per-key status; tests.

### [To Do] X-KV-0036 Lookup error mapping in the native error model
P0 · M12 · Task · deps: X-KV-0033, X-NATIVE-F10
Done when: A single exhaustive mapping covers TopicNotIndexed, IndexNotReady, NotLeader, UnknownTopic, Authorization and Timeout with gRPC status and problem+json details; NotFound is a result, not an error; tests.

### [To Do] X-KV-0037 Register `alluvium.kv.lookup.max.wait.ms` and `alluvium.kv.max.keys.per.request`
P0 · M12 · Task · deps: X-KV-0032, CFG-F11 · labels: config
Done when: Keys defined with validators, defaults and docs in the CFG-F11 registry; lookups honor dynamic updates; tests.

### [To Do] X-KV-0038 HTTP/JSON mapping `GET /v1/topics/{topic}/keys/{key}` with base64url keys
P0 · M12 · Task · deps: X-KV-0036, X-NATIVE-F04
Done when: The gateway serves lookups with content negotiation and the same errors; OpenAPI regenerated; tests.

### [To Do] X-KV-0039 SDK `kv().get()` and `kv().batch_get()`
P0 · M12 · Task · deps: X-KV-0035, X-NATIVE-F12
Done when: SDK methods with retries for retriable errors and docs with examples; integration test against an in-process cluster.

### [To Do] X-KV-0040 Lookup spans and per-endpoint metrics
P0 · M12 · Task · deps: X-KV-0033 · labels: observability
Done when: Each lookup runs in a `request` span with topic and partition (never key bytes); latency histograms by phase (index wait, index lookup, log read); tests assert metrics.

## [To Do] X-KV-F05 Range and prefix scans
P1 · M12 · Feature · deps: X-KV-F04
Scope: Ordered scans with pagination.

### [To Do] X-KV-0041 Merged iterator over memtable and sorted runs with tombstone suppression
P1 · M12 · Task · deps: none
Done when: A k-way merge yields keys in byte order, newest entry per key, skipping tombstones; property test against a model sorted map.

### [To Do] X-KV-0042 Native IDL: Scan by range or prefix with limit and page token
P1 · M12 · Task · deps: X-NATIVE-F02
Done when: Request carries start and end keys or prefix, limit and token; response carries entries and next token; breaking-change check passes.

### [To Do] X-KV-0043 Per-page consistency as decided in the ADR, documented on the API
P1 · M12 · Task · deps: X-KV-0041, X-KV-0042 · ref: ADR from X-KV-0001 (key index)
Done when: Each page reflects the index at the page's HW; keys updated between pages appear with their latest value; the API docs state this; tests demonstrate both behaviors.

### [To Do] X-KV-0044 Page tokens: versioned, bounded, validated on decode, with a fuzz target
P1 · M12 · Task · deps: X-KV-0042 · labels: decoder, security
Done when: Tokens encode partition, last key and HW; malformed or oversized tokens return InvalidPageToken; fuzz target in CI.

### [To Do] X-KV-0045 Cross-partition scans: scatter-gather merge bounded by `alluvium.kv.scan.max.partitions`
P1 · M12 · Task · deps: X-KV-0043
Done when: Scans without a partition fan out to leaders with bounded concurrency and merge in key order; over-limit requests are rejected; tests.

### [To Do] X-KV-0046 Scan limits: `alluvium.kv.scan.max.bytes`, max keys per page and a CPU time budget per page
P1 · M12 · Task · deps: X-KV-0043, CFG-F11 · labels: config
Done when: Pages stop at whichever limit comes first and return a token; registered keys with validators; tests.

### [To Do] X-KV-0047 HTTP mapping and SDK streaming iterator for scans
P1 · M12 · Task · deps: X-KV-0045, X-NATIVE-F12
Done when: `GET /v1/topics/{topic}/keys?prefix=` and SDK `kv().scan()` iterator handle tokens transparently; tests.

### [To Do] X-KV-0048 Scan throughput benchmark
P1 · M12 · Task · deps: X-KV-0045, TEST-F11 · labels: perf
Done when: Criterion and macro benchmarks for prefix scans over 10M keys; results tracked in the benchmark job.

## [To Do] X-KV-F06 Compaction integration
P0 · M12 · Feature · deps: X-KV-F02, STOR-F14
Scope: Index stays consistent with the cleaner.

### [To Do] X-KV-0049 Offset-primary pointers with position hints that survive segment rewrites
P0 · M12 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/Cleaner.java @4.4.0
Done when: After the cleaner swaps segments, lookups with stale position hints re-resolve via the offset index and succeed; tests with a cleaned segment.

### [To Do] X-KV-0050 Cleaner swap notifications invalidate position hints for swapped segments
P0 · M12 · Task · deps: X-KV-0049, STOR-F14
Done when: The cleaner's segment swap emits an observer event; the index bumps a segment generation so stale hints are not trusted; unit tests.

### [To Do] X-KV-0051 Tombstones purged from the index when the cleaner removes them after `delete.retention.ms`
P0 · M12 · Task · deps: X-KV-0050
Done when: Index tombstones older than the cleaner's removal point are dropped during run merges; property test with the simulated clock.

### [To Do] X-KV-0052 Invariant: the index never points to a record the cleaner removed
P0 · M12 · Task · deps: X-KV-0050 · labels: test
Done when: A debug assertion and a property test over random append, clean and lookup sequences hold.

### [To Do] X-KV-0053 `compact,delete` policy: retention deletes segments and the index purges entries below the log start offset
P0 · M12 · Task · deps: X-KV-0049
Done when: Entries below the log start are never returned and are removed during the next merge; tests.

### [To Do] X-KV-0054 Indexes on delete-policy topics: latest value per key until retention, behavior documented
P0 · M12 · Task · deps: X-KV-0049 · ref: ADR from X-KV-0001 (key index)
Done when: Enabling the index on a delete-only topic is allowed (or rejected) exactly as the ADR says; test and doc.

### [To Do] X-KV-0055 Property test: lookups equal the latest value per key in the compacted log
P0 · M12 · Task · deps: X-KV-0051, X-KV-0052, X-KV-0053 · labels: test
Done when: Random produce, clean, retention and lookup sequences match a model computed from the log after each step.

## [To Do] X-KV-F07 Tiered and diskless integration
P2 · M12 · Feature · deps: X-KV-F02, TIER-F04
Scope: Index covers remote data.

### [To Do] X-KV-0056 Lookups whose pointer is below the local log start read through the remote fetch path
P2 · M12 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java @4.4.0
Done when: Index entries pointing into tiered segments are served via TIER-F04 remote reads with remote-fetch quotas; latency reported separately; tests with the simulated remote store.

### [To Do] X-KV-0057 Full rebuild reads remote segments when local data has been tiered away
P2 · M12 · Task · deps: X-KV-0056
Done when: Rebuild scans remote segments with bounded rate and the remote read quota; tests.

### [To Do] X-KV-0058 Index snapshot upload with tiered segments to speed up new replicas
P3 · M12 · Task · deps: X-KV-0057
Done when: Sorted runs covering tiered ranges are uploaded alongside segments and downloaded on rebuild; format reused from the local index; tests.

### [To Do] X-KV-0059 Index built from committed diskless batches with lookups through the diskless read path
P2 · M13 · Task · deps: X-DISKLESS-F05
Done when: Diskless partitions feed the observer from the sequencer's commit stream and lookups read values via X-DISKLESS-F05 range reads and cache; tests with the simulated object store.

### [To Do] X-KV-0060 Remote and object-store failures during lookups return retriable errors
P2 · M12 · Task · deps: X-KV-0056
Done when: Timeouts and unavailability map to a retriable native error with no partial values; tests with fault injection.

## [To Do] X-KV-F08 Replica index behavior
P1 · M12 · Feature · deps: X-KV-F02
Scope: Follower-built indexes and follower reads.

### [To Do] X-KV-0061 Followers build the index from replicated batches with `alluvium.kv.follower.index.enable`
P1 · M12 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java @4.4.0
Done when: Follower observers apply batches as their HW advances; disabled followers rebuild on promotion; tests.

### [To Do] X-KV-0062 Register `alluvium.kv.follower.index.enable` (topic scope, default true for indexed topics)
P1 · M12 · Task · deps: X-KV-0061, CFG-F11 · labels: config
Done when: Key defined with validator and doc; toggling it starts or stops follower indexing; test.

### [To Do] X-KV-0063 Leader promotion: index readiness at the new leader
P1 · M12 · Task · deps: X-KV-0061
Done when: A promoted replica serves lookups once `indexed_offset` reaches the HW; earlier lookups return IndexNotReady; tests.

### [To Do] X-KV-0064 Follower lookups with rack-aware replica selection and an explicit consistency option
P1 · M12 · Task · deps: X-KV-0061 · ref: clients/src/main/java/org/apache/kafka/common/replica/RackAwareReplicaSelector.java @4.4.0
Done when: With `consistency=any_replica`, lookups may be served by a follower in the client's rack and report the follower's HW; the default remains leader; tests.

### [To Do] X-KV-0065 Reassignment: new replicas build their index from fetched data
P1 · M12 · Task · deps: X-KV-0061
Done when: A new replica's index reaches the HW before it is considered caught up for lookups; tests.

### [To Do] X-KV-0066 Simulation: leader failover with concurrent lookups never returns uncommitted or stale-past-ack values
P1 · M12 · Task · deps: X-KV-0063, TEST-F06 · labels: test
Done when: Seeded runs with crashes and partitions check that leader lookups reflect every acknowledged acks=all write and never a truncated one.

## [To Do] X-KV-F09 Lookup performance
P1 · M12 · Feature · deps: X-KV-F04
Scope: Caching and benchmarks.

### [To Do] X-KV-0067 Shared block cache for sorted run blocks bounded by `alluvium.kv.block.cache.bytes`
P1 · M12 · Task · deps: none
Done when: Per-broker cache with the eviction policy from the ADR; hit and miss metrics; eviction tests under memory pressure.

### [To Do] X-KV-0068 Decompressed batch cache for hot values bounded by `alluvium.kv.value.cache.bytes`
P1 · M12 · Task · deps: X-KV-0067
Done when: Repeated lookups into the same compressed batch decompress once; bounded; tests.

### [To Do] X-KV-0069 Bloom filters on sorted runs to skip runs for absent keys
P1 · M12 · Task · deps: none
Done when: False-positive rate measured below the ADR target; lookups for absent keys touch at most one run on average in a benchmark.

### [To Do] X-KV-0070 Broker memory budget for all indexes: `alluvium.kv.memory.bytes`
P1 · M12 · Task · deps: X-KV-0067, X-KV-0068
Done when: Memtables and caches share the budget; enabling an index when the budget is exhausted fails with a clear error; tests.

### [To Do] X-KV-0071 Register `alluvium.kv.block.cache.bytes`, `alluvium.kv.value.cache.bytes` and `alluvium.kv.memory.bytes`
P1 · M12 · Task · deps: X-KV-0070, CFG-F11 · labels: config
Done when: Keys defined with validators (caches must fit in the memory budget), defaults and docs; `check-config` rejects inconsistent values; tests.

### [To Do] X-KV-0072 Allocation-free lookup hot path
P1 · M12 · Task · deps: X-KV-0067 · labels: perf
Done when: A counting-allocator test shows zero allocations per cached lookup.

### [To Do] X-KV-0073 Lookup latency benchmark: p50/p99 for 10M keys, hot and cold
P1 · M12 · Task · deps: X-KV-0072, X-KV-0069, TEST-F11 · labels: perf, nfr
Done when: Criterion and macro benchmarks recorded in the benchmark job with regression thresholds.

### [To Do] X-KV-0074 Produce throughput impact and write amplification with indexing enabled
P1 · M12 · Task · deps: TEST-F11 · labels: perf, nfr
Done when: Produce throughput drop with indexing on is within the ADR budget (target under 5%); write amplification measured and documented.

## [To Do] X-KV-F10 Lookup security
P0 · M12 · Feature · deps: X-KV-F04, SEC-F07
Scope: Authorization for lookups.

### [To Do] X-KV-0075 Authorization for lookups and scans: READ on TOPIC; DESCRIBE for index status; ALTER_CONFIGS to enable
P0 · M12 · Task · deps: none · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest @4.4.0
Done when: Same resource and operation mapping as Fetch; denials emit `authz_decision`; tests per operation.

### [To Do] X-KV-0076 No existence oracle: unauthorized requests get the same error whether or not the topic or key exists
P0 · M12 · Task · deps: X-KV-0075 · labels: security
Done when: Tests compare responses for existing and missing keys and topics under denial.

### [To Do] X-KV-0077 Lookups and scans are charged to request-time quotas
P0 · M12 · Task · deps: X-KV-0075, QUOTA-F04 · ref: server/src/main/java/org/apache/kafka/server/quota/ClientRequestQuotaManager.java @4.4.0
Done when: Lookup handler time is recorded in QUOTA-F04 sensors; throttled principals receive throttle time; tests.

### [To Do] X-KV-0078 Key bytes never appear in logs, metrics, events or errors
P0 · M12 · Task · deps: X-KV-0075, OBS-F11 · labels: security
Done when: OBS-F11 redaction tests cover lookup paths; logs show key length and hash only.

### [To Do] X-KV-0079 Input bounds: key length checked before allocation, request size limits on gRPC and HTTP
P0 · M12 · Task · deps: X-KV-0075 · labels: security
Done when: Oversized keys fail validation without allocating; tests and fuzz coverage through the native API fuzz targets.

## [To Do] X-KV-F11 Key lookup testing and documentation
P0 · M12 · Feature · deps: X-KV-F04, TEST-F16
Scope: Property test that lookups equal the latest value per key in the log; docs.

### [To Do] X-KV-0080 Property test: lookups equal the latest committed value per key in the log
P0 · M12 · Task · deps: TEST-F16 · labels: test
Done when: Random produce, transactional, compaction, truncation and lookup sequences compare lookups with a model computed from the log at each step.

### [To Do] X-KV-0081 Checker: every lookup returns the latest committed value at or above the HW it reports
P0 · M12 · Task · deps: TEST-F05 · labels: test
Done when: A TEST-F05 checker compares each lookup response with a model built from the acknowledged log and fails with the seed and flight recorders on any mismatch.

### [To Do] X-KV-0082 Checker: `indexed_offset` never exceeds the HW and never regresses except on truncation
P0 · M12 · Task · deps: TEST-F05 · labels: test
Done when: The checker reads each replica's index watermark after every step.

### [To Do] X-KV-0083 Simulation: lookups under crashes, disk faults and index corruption
P0 · M12 · Task · deps: X-KV-0080, X-KV-0081, X-KV-0082, TEST-F06 · labels: test
Done when: Seeded runs inject crashes and corrupt runs; lookups either return correct values or retriable errors; never a wrong value.

### [To Do] X-KV-0084 Key index is off by default: no observers, no index files, lookups report the topic as not indexed
P0 · M12 · Task · deps: none · labels: test
Done when: With default config no index directory is created, the append path has no observer registered, and kv endpoints return TopicNotIndexed; test.

### [To Do] X-KV-0085 Kafka compatibility and differential suites pass with `alluvium.kv.enable=true` but no indexed topics
P0 · M12 · Task · deps: X-KV-0084, TEST-F09 · labels: test
Done when: CI job runs TEST-F08 and TEST-F09 suites with the switch on; zero new diffs.

### [To Do] X-KV-0086 Differential suite on indexed compacted topics: Kafka clients see identical behavior to Kafka
P0 · M12 · Task · deps: X-KV-0085 · labels: test
Done when: Produce, fetch, compaction and offset scenarios on topics with the index enabled produce no new diffs against Kafka 4.4.0.

### [To Do] X-KV-0087 User guide: key lookups and scans, consistency model, examples with the SDK and HTTP
P0 · M12 · Task · deps: none · labels: docs
Done when: docs/guides/key-lookups.md covers concepts, API examples and limits; doc examples tested.

### [To Do] X-KV-0088 Operations guide: sizing, memory budget, rebuilds and troubleshooting
P0 · M12 · Task · deps: X-KV-0087 · labels: docs
Done when: Covers disk and memory sizing, rebuild time estimates, metrics and events; links check.

### [To Do] X-KV-0089 Migration and opt-in guide: enabling the cluster switch and indexes on existing topics
P0 · M12 · Task · deps: X-KV-0087 · labels: docs
Done when: Explains enabling per topic, background build, disabling and the effect on brokers; links check.

### [To Do] X-KV-0090 Key index metrics and events in the generated references
P0 · M12 · Task · deps: OBS-F07 · labels: docs, observability
Done when: All `alluvium_kv_*` metrics and `kv_*` events appear in docs/metrics.md and docs/events.md via the catalogs.
