# STOR: Log storage

> Crates: `alluvium-storage` (layer 1) · Kafka counterpart: `storage/src/main/java/org/apache/kafka/storage/internals/log` · Milestones: M2-M7

Partition logs: segments, indexes, checkpoints, recovery, truncation, retention, compaction, producer state, log management, flush policy and crash consistency.

## [To Do] STOR-F01 Filesystem abstraction
P0 · M2 · Feature · deps: TYPE-F06 · labels: foundation-for:X-LOWLAT, foundation-for:X-EMBED
Scope: Narrow `fs` trait for all storage I/O, the real implementation, and an in-memory implementation with fault hooks. The fault-injecting simulated disk itself is TEST-F03.

### [To Do] STOR-0001 Define the `Fs` trait and `FsError` for all storage I/O: open/create, positional read and write, append, set_len, sync_data, rename, remove, list_dir, create_dir, sync_dir, try_lock, mtime
P0 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/record/internal/FileRecords.java openChannel, append, readInto, truncateTo, flush @4.4.0
Done when: The trait and error enum compile with rustdoc stating, per method, what is durable after which sync call; `FsError` carries `op`, `path` and an `FsErrorKind`; no method takes an unbounded length without a caller-supplied buffer; module `//!` doc states the crash-durability contract.

### [To Do] STOR-0002 Classify OS errors into `FsErrorKind` (NotFound, AlreadyExists, NoSpace, PermissionDenied, ReadOnlyFs, Interrupted, Io) so callers can distinguish disk-full and log-dir failure
P0 · M2 · Task · deps: STOR-0001 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogDirFailureChannel.java maybeAddOfflineLogDir @4.4.0
Done when: Unit tests map ENOENT, EEXIST, ENOSPC, EDQUOT, EACCES, EROFS and EIO to the documented kinds on Linux and macOS; the mapping is one exhaustive `match` with no `_ =>` arm over the kinds Alluvium handles.

### [To Do] STOR-0003 `StdFs`: the production `Fs` over `std::fs` with positional I/O (`FileExt::read_at`/`write_at`), explicit append position tracking and parent-directory fsync helpers
P0 · M2 · Task · deps: STOR-0001, STOR-0002 · ref: clients/src/main/java/org/apache/kafka/common/utils/Utils.java atomicMoveWithFallback, flushDir @4.4.0
Done when: Integration tests in a temp dir exercise every trait method; rename followed by `sync_dir` is covered; `std::fs` appears only in this module; tests pass on Linux and macOS in CI.

### [To Do] STOR-0004 Atomic file-replace helper: write `<name>.tmp`, `sync_data`, rename over the target, `sync_dir` on the parent
P0 · M2 · Task · deps: STOR-0001 · ref: server-common/src/main/java/org/apache/kafka/server/common/CheckpointFile.java write @4.4.0
Done when: The helper is the only way storage replaces small files; a test with `MemFs` crashes after each step and observes either the complete old or the complete new content, never a mix; a leftover `.tmp` is overwritten on the next write.

### [To Do] STOR-0005 `MemFs`: in-memory `Fs` that tracks durable and unsynced state separately for file contents and directory entries
P0 · M2 · Task · deps: STOR-0001, STOR-0002 · labels: test
Done when: `crash()` deterministically discards unsynced data and unsynced directory entries; property test: data followed by `sync_data` (and `sync_dir` for new names) always survives `crash()`; `MemFs` is usable from unit tests without Tokio.

### [To Do] STOR-0006 Fault-injection hooks on `MemFs`: per-operation error injection, torn-write truncation at a byte offset, capacity limit returning `NoSpace`, used by TEST-F03's simulated disk
P0 · M2 · Task · deps: STOR-0005 · labels: test, foundation-for:TEST
Done when: Each hook is covered by a unit test; hooks are addressed by operation index so a crash point is reproducible; the API is documented for TEST-F03 to drive from a seed.

### [To Do] STOR-0007 Ban direct file-system access in storage code: clippy `disallowed-methods`/`disallowed-types` for `std::fs` and `std::fs::File` outside the `fs` implementations
P0 · M2 · Task · deps: STOR-0003
Done when: `clippy.toml` entries land with a smoke test proving a stray `std::fs::read` in `alluvium-storage` fails `cargo xtask ci`; exceptions use `#[expect(..., reason)]` on the `StdFs` module only.

### [To Do] STOR-0008 Instrumented `Fs` wrapper in the server layer recording per-operation latency and errors
P1 · M2 · Task · deps: STOR-0003, OBS-F04 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java LOG_FLUSH_TIMER @4.4.0
Done when: Wrapper exports `alluvium_storage_fs_op_seconds{op}` histograms and `alluvium_storage_fs_errors_total{op,kind}` counters; label values are a closed enum; metrics are asserted in a test against an explicitly passed registry.

## [To Do] STOR-F02 Segment store abstraction
P0 · M2 · Feature · deps: STOR-F01 · labels: foundation-for:X-DISKLESS, foundation-for:TIER, foundation-for:X-SECX
Scope: Segments addressed through a store interface so local disk, tiered and object-storage backends plug in.

### [To Do] STOR-0009 Spike: segment store interface ADR covering segment identity, the immutable-after-seal contract, positional reads, deletion, and how tiered (TIER) and diskless (X-DISKLESS) backends plug in
P0 · M2 · Spike · deps: none · labels: foundation-for:TIER, foundation-for:X-DISKLESS · ref: ADR-0007
Done when: ADR "Segment store abstraction" is accepted, lists the rejected alternatives, and states which operations must be durable before returning; docs/architecture.md links it.

### [To Do] STOR-0010 Segment descriptor types: `SegmentId` (base offset), `SegmentState` (Active, Sealed), `SegmentDescriptor` (base offset, next offset, size in bytes, max timestamp, leader-epoch range)
P0 · M2 · Task · deps: STOR-0009 · labels: foundation-for:TIER · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java baseOffset, readNextOffset, largestTimestamp @4.4.0
Done when: Types implement `Debug` and ordering by base offset; property test that ordering matches offset ranges; no type exposes file paths outside the local store.

### [To Do] STOR-0011 `SegmentStore` trait: create active segment, open sealed segment, list, read a byte range, seal, delete, with typed `StoreError`
P0 · M2 · Task · deps: STOR-0010 · labels: foundation-for:TIER, foundation-for:X-DISKLESS
Done when: Trait rustdoc states each method's durability and idempotency contract (delete of a missing segment is `Ok`); the trait is the only path the partition log uses to reach segment bytes.

### [To Do] STOR-0012 `SegmentRead` result type: a file region (handle, position, length) for zero-copy sends, or owned bytes for remote and compressed paths
P0 · M2 · Task · deps: STOR-0010 · labels: perf · ref: clients/src/main/java/org/apache/kafka/common/record/internal/FileRecords.java slice @4.4.0
Done when: The local store returns file regions; the enum exposes length without reading; a test proves a file region never extends past the segment size at read time.

### [To Do] STOR-0013 `LocalSegmentStore`: the `SegmentStore` implementation over `Fs` and the partition directory
P0 · M2 · Task · deps: STOR-0011, STOR-0012 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java open @4.4.0
Done when: All trait methods are implemented over `MemFs` and `StdFs`; list returns segments sorted by base offset; errors carry topic, partition and segment base offset.

### [To Do] STOR-0014 Reusable `SegmentStore` conformance suite: create, append, seal, read ranges, delete, idempotent delete, listing after crash
P0 · M2 · Task · deps: STOR-0013 · labels: test, foundation-for:TIER, foundation-for:X-DISKLESS
Done when: The suite is a public test helper; `LocalSegmentStore` passes it; its rustdoc explains how TIER and X-DISKLESS backends run it.

## [To Do] STOR-F03 Segment files
P0 · M2 · Feature · deps: STOR-F02, REC-F02
Scope: Append, positional read, size tracking, 20-digit base-offset naming, optional preallocation, close and flush.

### [To Do] STOR-0015 Spike: segment and index file format ADR: Kafka-identical `.log`/`.index`/`.timeindex`/`.txnindex`/`.snapshot` layouts, 20-digit names, suffix protocol, no mmap (indexes via `Fs` plus an in-memory cache), versioning policy
P0 · M2 · Spike · deps: none · labels: docs · ref: ADR-0007
Done when: ADR accepted; it closes the "Segment and index file formats" row in docs/architecture.md §10; it states that `alluvium dump` tools and docs/formats/ follow it.

### [To Do] STOR-0016 `storage::LogConfig` plain struct with the segment fields (`segment_bytes`, `segment_ms`, `segment_jitter_ms`, `segment_index_bytes`, `index_interval_bytes`, `preallocate`, `max_message_bytes`, `file_delete_delay_ms`) and Kafka defaults
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java @4.4.0
Done when: Struct has rustdoc per field naming the Kafka key and default; constructor validates ranges (for example `segment.bytes` >= 14); later features extend it rather than adding parallel structs.

### [To Do] STOR-0017 Segment file naming: 20-digit zero-padded base offset with `.log`, `.index`, `.timeindex`, `.txnindex`, `.snapshot` and the `.deleted`, `.cleaned`, `.swap` suffixes; parse and format
P0 · M2 · Task · deps: none · labels: decoder · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogFileUtils.java filenamePrefixFromOffset, offsetFromFileName, isLogFile, isIndexFile @4.4.0
Done when: Round-trip property test for all non-negative offsets; non-numeric, negative or overlong names return `FileNameError`; fuzz target `segment_file_name` runs clean.

### [To Do] STOR-0018 `SegmentFile`: open a segment file, append record-batch bytes at the tracked end, track size as a bounded `u32` <= `i32::MAX`
P0 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/record/internal/FileRecords.java open, append, sizeInBytes @4.4.0
Done when: Append returns bytes written; an append that would exceed `i32::MAX` bytes returns `SegmentFull` without writing; operations after close return `Closed`; unit tests over `MemFs`.

### [To Do] STOR-0019 Positional batch iteration over a segment file reading only the batch header (offset, length, epoch, magic) and bounding every length before allocating
P0 · M2 · Task · deps: STOR-0018 · labels: decoder, security · ref: clients/src/main/java/org/apache/kafka/common/record/internal/FileRecords.java batchesFrom @4.4.0
Done when: Iteration from any position yields batches or a typed `CorruptBatch { position }` error; lengths above `max.message.bytes` plus overhead are rejected before allocation; a property test matches REC-F02 parsing of the same bytes.

### [To Do] STOR-0020 Scan from a starting position for the first batch whose last offset is >= a target offset, returning offset, position and batch size
P0 · M2 · Task · deps: STOR-0019 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/FileRecords.java searchForOffsetFromPosition @4.4.0
Done when: Unit tests cover target inside a batch, at a batch boundary, past the end (none) and in the first batch; scan reads headers only.

### [To Do] STOR-0021 Bounded slice reads: `slice(position, size)` clipped to the size observed at call time
P0 · M2 · Task · deps: STOR-0018 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/FileRecords.java slice @4.4.0
Done when: A slice starting past the end returns empty; size is clipped, never extended; concurrent appends after the call do not change an issued slice.

### [To Do] STOR-0022 Timestamp scans on a segment file: first batch with max timestamp >= target at or after a starting offset, and largest timestamp after a position
P0 · M2 · Task · deps: STOR-0019 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/FileRecords.java searchForTimestamp, largestTimestampAfter @4.4.0
Done when: Unit tests cover equal timestamps, decreasing timestamps across batches, and no match; results match a linear-scan model in a property test.

### [To Do] STOR-0023 Truncate a segment file to a byte size and report bytes removed
P0 · M2 · Task · deps: STOR-0018 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/FileRecords.java truncateTo @4.4.0
Done when: Truncating to a size larger than the current size returns `InvalidTruncation`; the size field and file length agree after truncation; covered by unit tests.

### [To Do] STOR-0024 Segment preallocation for `preallocate=true`: create the file at `segment.bytes`, track the logical size separately, trim on seal and clean close
P0 · M2 · Task · deps: STOR-0018 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/FileRecords.java trim @4.4.0
Done when: Reopening a cleanly closed preallocated segment sees the trimmed size; an uncleanly closed one is recovered by scanning (zero-length batch header ends the log); tests cover both.

### [To Do] STOR-0025 `LogSegment` core: base offset, segment file, created time from the injected clock, `append(largest_offset, batches)` with the relative-offset range check
P0 · M2 · Task · deps: STOR-0018, STOR-0016 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java append, ensureOffsetInRange, canConvertToRelativeOffset @4.4.0
Done when: Appending an offset with `offset - base_offset` outside `0..=i32::MAX` returns `LogSegmentOffsetOverflow { segment_base_offset, offset }` before writing; appends of empty input are no-ops.

### [To Do] STOR-0026 `LogSegment` lifecycle operations: flush, close, delete-if-exists, change file suffixes, has-suffix, last-modified get and set
P0 · M2 · Task · deps: STOR-0025 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java flush, close, deleteIfExists, changeFileSuffixes, hasSuffix, lastModified, setLastModified @4.4.0
Done when: Renaming suffixes renames every existing file of the segment together and a missing optional file (txn index) is skipped; `delete_if_exists` reports which files were removed; tests over `MemFs`.

### [To Do] STOR-0027 Fuzz target `segment_scan`: arbitrary bytes as a segment file through iteration, offset search and timestamp search
P0 · M2 · Task · deps: STOR-0019, STOR-0020 · labels: test
Done when: The target runs in CI's short fuzz job with no panics, hangs or allocations above the configured bound; seeded corpus includes valid segments from REC golden batches.

### [To Do] STOR-0028 Port the FileRecordsTest behaviors relevant to segment files (append, slice, truncate, search, preallocation)
P0 · M2 · Task · deps: STOR-0021, STOR-0022, STOR-0023, STOR-0024 · labels: test · ref: clients/src/test/java/org/apache/kafka/common/record/internal/FileRecordsTest.java @4.4.0
Done when: Each ported test carries a `// Ported from:` comment; all pass against `MemFs` and `StdFs`.

## [To Do] STOR-F04 Offset index
P0 · M2 · Feature · deps: STOR-F03
Scope: Sparse offset index (relative offset, position): lookup, append, truncation, sanity checks, resize; no mmap (unsafe is forbidden).

### [To Do] STOR-0029 Offset index entry codec: 8 bytes, big-endian relative offset (i32) and physical position (i32)
P0 · M2 · Task · deps: none · labels: decoder · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/OffsetIndex.java parseEntry, relativeOffset, physical @4.4.0
Done when: Round-trip property test; decoding rejects negative positions and relative offsets with a typed error; fuzz target `offset_index` decodes arbitrary bytes without panic.

### [To Do] STOR-0030 Open an offset index through `Fs` into an in-memory entry cache: new files preallocated to `segment.index.bytes`, existing files loaded with entries = length / 8
P0 · M2 · Task · deps: STOR-0029 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/AbstractIndex.java AbstractIndex, createMappedBuffer, roundDownToExactMultiple @4.4.0
Done when: Loading reads at most `segment.index.bytes`; `max_entries` = size / 8; a file whose length is not a multiple of 8 is rounded down and flagged for sanity checking; no mmap or `unsafe` is used.

### [To Do] STOR-0031 Offset index lookup: greatest entry <= target offset, or (base offset, 0) when none
P0 · M2 · Task · deps: STOR-0030 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/OffsetIndex.java lookup @4.4.0
Done when: Property test against a linear scan over random indexes; empty and full indexes covered; lookup does not touch the file.

### [To Do] STOR-0032 Offset index append with strictly increasing offsets, full-index check and positional write of the entry
P0 · M2 · Task · deps: STOR-0030 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/OffsetIndex.java append @4.4.0
Done when: Non-increasing offsets return `InvalidOffset { offset, last_offset }`; appending to a full index returns `IndexFull`; the file and the cache agree after every append; flush syncs the file.

### [To Do] STOR-0033 Fetch upper bound: smallest entry whose position is >= fetch position + fetch size
P0 · M2 · Task · deps: STOR-0031 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/OffsetIndex.java fetchUpperBoundOffset @4.4.0
Done when: Unit tests cover no such entry (none), exact match and in-between; used by aborted-transaction collection.

### [To Do] STOR-0034 Offset index truncation to an offset (no entry <= offset, exact entry, between entries) and full truncation
P0 · M2 · Task · deps: STOR-0032 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/OffsetIndex.java truncateTo, truncate @4.4.0
Done when: Each of Kafka's three cases has a test; the file length is updated through `Fs::set_len`; the cache matches the file.

### [To Do] STOR-0035 Index resize and trim: round to a multiple of the entry size, trim to valid entries on seal, re-expand to `segment.index.bytes` when a segment becomes active again
P0 · M2 · Task · deps: STOR-0032 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/AbstractIndex.java resize, trimToValidSize, canAppendOffset, relativeOffset @4.4.0
Done when: Resize to the same size is a no-op that returns `false`; trimmed file length equals entries * 8; relative-offset overflow is reported by `can_append_offset`.

### [To Do] STOR-0036 Offset index sanity check: corrupt when entries exist and the last offset is below the base offset, or the length is not a multiple of 8
P0 · M2 · Task · deps: STOR-0030 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/OffsetIndex.java sanityCheck @4.4.0
Done when: Both conditions return `CorruptIndex { path, reason }`; tests build each corrupt file with `MemFs`.

### [To Do] STOR-0037 Lazy index loading: segments open their offset and time indexes on first access
P0 · M2 · Task · deps: STOR-0030 · labels: perf · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LazyIndex.java get, loadIndex @4.4.0
Done when: Loading a log with 10,000 segments reads no index file until a lookup; a test counts `Fs` reads; renames and deletes work on unopened indexes.

### [To Do] STOR-0038 Global bounded cache for sealed-segment index entries with LRU eviction under a configured byte budget (`alluvium.*` key defined by CFG-F11)
P0 · M2 · Task · deps: STOR-0037 · labels: perf, nfr
Done when: Memory used by cached index entries never exceeds the budget in a test with many segments; evicted indexes reload through `Fs`; hit, miss and eviction counts are returned for the driver to export.

### [To Do] STOR-0039 LogSegment appends an offset index entry (batch last offset, physical position) when bytes since the last entry exceed `index.interval.bytes`
P0 · M2 · Task · deps: STOR-0032 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java append @4.4.0
Done when: Test appends batches of varying sizes and checks entries match Kafka's rule exactly (strictly greater than the interval); `bytes_since_last_index_entry` resets on each entry.

### [To Do] STOR-0040 LogSegment read: translate offset via index lookup plus scan, then read up to max size, max position and min-one-message
P0 · M2 · Task · deps: STOR-0031, STOR-0039 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java read, translateOffset @4.4.0
Done when: Returns none when the offset is past the end; empty records when max size is 0 or max position is unknown; `first_entry_incomplete` set when min-one-message enlarged the read; tests ported from LogSegmentTest read cases.

### [To Do] STOR-0041 LogSegment next offset: last index entry then scan to the end
P0 · M2 · Task · deps: STOR-0040 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java readNextOffset @4.4.0
Done when: Empty segment returns its base offset; result equals the last batch's last offset + 1 in a property test.

### [To Do] STOR-0042 Port OffsetIndexTest
P0 · M2 · Task · deps: STOR-0034, STOR-0035, STOR-0036, STOR-0033 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/OffsetIndexTest.java @4.4.0
Done when: Every applicable test is ported with `// Ported from:`; tests that rely on mmap are replaced by equivalent `Fs`-based checks and listed in the PR.

## [To Do] STOR-F05 Time index
P0 · M2 · Feature · deps: STOR-F03
Scope: Sparse timestamp index, lookup by timestamp, monotonic append rules.

### [To Do] STOR-0043 Time index entry codec: 12 bytes, big-endian timestamp (i64) and relative offset (i32)
P0 · M2 · Task · deps: none · labels: decoder · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TimeIndex.java parseEntry, timestamp, relativeOffset @4.4.0
Done when: Round-trip property test; fuzz target `time_index` decodes arbitrary bytes without panic.

### [To Do] STOR-0044 Open a time index through `Fs` into the in-memory cache and recover the last entry from the file
P0 · M2 · Task · deps: STOR-0043 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TimeIndex.java TimeIndex, lastEntryFromIndexFile @4.4.0
Done when: Empty index reports last entry (-1, base offset); a test reopens an index and gets the same last entry; maximum entries = size / 12.

### [To Do] STOR-0045 Time index `maybe_append` monotonic rules: reject offsets below the last entry, reject timestamps below the last entry, append only when the timestamp is strictly greater, honor `skip_full_check`
P0 · M2 · Task · deps: STOR-0044 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TimeIndex.java maybeAppend @4.4.0
Done when: Offset regression returns `InvalidOffset`; timestamp regression is an invariant violation (`expect("invariant: ...")`) because inputs come from the segment's own max-timestamp tracking; property test: both columns stay monotonic.

### [To Do] STOR-0046 Time index lookup: greatest entry with timestamp <= target, or (NO_TIMESTAMP, base offset) when none
P0 · M2 · Task · deps: STOR-0044 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TimeIndex.java lookup @4.4.0
Done when: Property test against a linear scan; empty index returns the sentinel.

### [To Do] STOR-0047 Time index truncation by offset and `is_full` reserving one slot (entries >= max entries - 1) for the seal entry
P0 · M2 · Task · deps: STOR-0045 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TimeIndex.java truncateTo, isFull, truncate @4.4.0
Done when: Truncation cases (none <=, exact, between) tested; `is_full` true one entry early; last entry recomputed after truncation.

### [To Do] STOR-0048 Time index sanity check (last timestamp below first, last offset below base, bad length) and the "newly created time index is resized to 0" load rule
P0 · M2 · Task · deps: STOR-0044 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TimeIndex.java sanityCheck @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java sanityCheck @4.4.0
Done when: Each corrupt case returns `CorruptIndex`; a segment loaded without a `.timeindex` gets an empty one; tests cover all cases.

### [To Do] STOR-0049 LogSegment max-timestamp tracking: max timestamp and its offset updated per batch, lazily loaded from the time index last entry, and a time index entry written with each offset index entry
P0 · M2 · Task · deps: STOR-0045, STOR-F04 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java readMaxTimestampAndOffsetSoFar, maxTimestampSoFar, append @4.4.0
Done when: After reopening a segment, max timestamp equals the value before close; time index entries match Kafka's rule in a test appending out-of-order batch timestamps.

### [To Do] STOR-0050 LogSegment find offset by timestamp: time index lookup, offset index lookup, then file scan honoring the starting offset
P0 · M2 · Task · deps: STOR-0046, STOR-0049 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java findOffsetByTimestamp @4.4.0
Done when: Returns the first record with timestamp >= target and offset >= starting offset; tests ported from LogSegmentTest timestamp cases.

### [To Do] STOR-0051 LogSegment largest timestamp for retention: max record timestamp, or last-modified time when the segment has no record timestamps
P0 · M2 · Task · deps: STOR-0049 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java largestTimestamp, largestRecordTimestamp, lastModified @4.4.0
Done when: Tests cover segments with and without timestamps; `largest_record_timestamp` returns none for an empty segment.

### [To Do] STOR-0052 Port TimeIndexTest
P0 · M2 · Task · deps: STOR-0047, STOR-0048, STOR-0050 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/TimeIndexTest.java @4.4.0
Done when: Every applicable test is ported with `// Ported from:`; mmap-specific tests replaced by `Fs`-based equivalents.

## [To Do] STOR-F06 Transaction index
P0 · M6 · Feature · deps: STOR-F03
Scope: Aborted transaction index per segment for read_committed fetches.

### [To Do] STOR-0053 Aborted transaction entry codec: version-prefixed `AbortedTxn` (version 0: producer id, first offset, last offset, last stable offset)
P0 · M6 · Task · deps: none · labels: decoder · ref: clients/src/main/resources/common/message/AbortedTxn.json @4.4.0
Done when: Golden bytes match Kafka's encoding for a sample entry; decoding rejects unknown versions; fuzz target `txn_index` runs clean.

### [To Do] STOR-0054 Transaction index file: created lazily on first append, iterated entry by entry with bounded reads
P0 · M6 · Task · deps: STOR-0053 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TransactionIndex.java openChannel, iterable @4.4.0
Done when: A segment without aborted transactions has no `.txnindex` file; iteration over a truncated trailing entry stops with `CorruptIndex`; tests over `MemFs`.

### [To Do] STOR-0055 Transaction index append requiring strictly increasing last offsets
P0 · M6 · Task · deps: STOR-0054 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TransactionIndex.java append @4.4.0
Done when: Non-increasing last offset returns `InvalidTxnIndexAppend`; flush syncs the file; test covers first append creating the file.

### [To Do] STOR-0056 Collect aborted transactions overlapping [fetch offset, upper bound) and report completeness when an entry's last stable offset >= upper bound
P0 · M6 · Task · deps: STOR-0054 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TransactionIndex.java collectAbortedTxns @4.4.0
Done when: Unit tests cover overlap rules and early completion; results match a model in a property test.

### [To Do] STOR-0057 Transaction index truncation (keep entries with last offset < offset), reset, delete and rename
P0 · M6 · Task · deps: STOR-0055 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TransactionIndex.java truncateTo, reset, deleteIfExists, renameTo @4.4.0
Done when: Tests cover truncation inside and past the entries; rename keeps the lazily-unopened state.

### [To Do] STOR-0058 Transaction index sanity check: an entry whose last offset is below the segment base offset is corrupt
P0 · M6 · Task · deps: STOR-0054 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TransactionIndex.java sanityCheck @4.4.0
Done when: Returns `CorruptIndex`; test builds such a file.

### [To Do] STOR-0059 LogSegment transaction index integration: append an entry for each completed aborted transaction with its last stable offset; collect per segment
P0 · M6 · Task · deps: STOR-0055, STOR-0056 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java updateTxnIndex, collectAbortedTxns @4.4.0
Done when: Committed transactions write nothing; tests cover aborted transactions spanning segments.

### [To Do] STOR-0060 Partition-log aborted transaction collection across segments for read_committed fetches, using the offset index upper bound or the next segment base
P0 · M6 · Task · deps: STOR-0059, STOR-F08 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java collectAbortedTransactions, read @4.4.0
Done when: `read` with `include_aborted_txns` returns exactly the aborted transactions overlapping the returned records; tests port LocalLogTest and UnifiedLogTest read_committed cases.

### [To Do] STOR-0061 Rebuild the transaction index during segment recovery from completed transactions in the producer state
P0 · M6 · Task · deps: STOR-0059 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java recover, updateProducerState @4.4.0
Done when: After deleting a `.txnindex` and recovering, the rebuilt file equals the original byte for byte in a test.

### [To Do] STOR-0062 Port TransactionIndexTest
P0 · M6 · Task · deps: STOR-0057, STOR-0058, STOR-0060 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/TransactionIndexTest.java @4.4.0
Done when: Every test is ported with `// Ported from:` and passes.

## [To Do] STOR-F07 Segment rolling
P0 · M2 · Feature · deps: STOR-F04, STOR-F05, CFG-F05
Scope: Roll on `segment.bytes`, `segment.ms` with jitter, full index, relative-offset overflow.

### [To Do] STOR-0063 `LogSegments`: ordered map of segments by base offset with add, remove, get, floor, lower, higher, first, last, active, values in range, non-active from, size in bytes
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegments.java @4.4.0
Done when: Backed by a `BTreeMap`; property test against a model for every operation; `values(from, to)` includes the floor segment of `from` and excludes segments starting at or after `to`.

### [To Do] STOR-0064 `should_roll` decision with reasons: size would exceed `segment.bytes`, `segment.ms` minus jitter elapsed (non-empty segment), offset index full, time index full, relative offset not representable
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java shouldRoll @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/RollParams.java @4.4.0
Done when: Returns `Option<RollReason>`; one unit test per reason; size check uses `size > segment_bytes - messages_size` exactly.

### [To Do] STOR-0065 Time waited for roll: based on the first batch's max timestamp when present, otherwise on segment creation time from the injected clock
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java timeWaitedForRoll, getFirstBatchTimestamp, loadFirstBatchTimestamp @4.4.0
Done when: Tests cover record timestamps in the future and past and an empty segment; no direct clock reads.

### [To Do] STOR-0066 Per-segment roll jitter in [0, min(`segment.jitter.ms`, `segment.ms`)) drawn from the injected seeded RNG
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java randomSegmentJitter @4.4.0
Done when: Jitter is 0 when `segment.jitter.ms` is 0; same seed gives the same jitter; property test checks the bound.

### [To Do] STOR-0067 Seal a segment when it stops being active: final time index entry (skip full check), trim both indexes and the preallocated log
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java onBecomeInactiveSegment @4.4.0
Done when: After sealing, index file lengths equal entries times entry size and the log file length equals its logical size; reopening yields identical state.

### [To Do] STOR-0068 Roll: new base offset = max(expected next offset, log end offset); replace an empty active segment with the same base (KAFKA-6388); reject a base below the active segment or an existing non-empty one; delete stale files with the new name; open the new segment with preallocation
P0 · M2 · Task · deps: STOR-0063, STOR-0064, STOR-0067 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java roll @4.4.0
Done when: Each branch has a test; the new active segment's indexes are sized to `segment.index.bytes`; errors carry topic, partition and offsets.

### [To Do] STOR-0069 Segment roll observability: `segment_rolled` event with `reason`, old and new `segment_base_offset` and sizes; flight-recorder entry; roll counter by reason
P0 · M2 · Task · deps: STOR-0068, OBS-F07 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java maybeRoll @4.4.0
Done when: Event is listed in the storage `events.rs`; the driver records it; `alluvium_log_segment_rolls_total{reason}` is asserted in a test.

### [To Do] STOR-0070 Map topic and broker config onto `storage::LogConfig` segment fields: `segment.bytes`/`log.segment.bytes`, `segment.ms`/`log.roll.ms`/`log.roll.hours`, `segment.jitter.ms`/`log.roll.jitter.*`, `segment.index.bytes`/`log.index.size.max.bytes`, `index.interval.bytes`, `preallocate`
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java @4.4.0
Done when: Mapping lives in `alluvium-config`; precedence (topic override, then broker ms key, then hours key) matches Kafka in table-driven tests.

### [To Do] STOR-0071 Port Kafka's roll tests (size-based, time-based, jitter, index-full, KAFKA-6388 empty-segment roll)
P0 · M2 · Task · deps: STOR-0068, STOR-0065, STOR-0066 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/UnifiedLogTest.java @4.4.0
Done when: Tests carry `// Ported from:` comments and use the simulated clock; all pass.

## [To Do] STOR-F08 Partition log
P0 · M2 · Feature · deps: STOR-F07, REC-F12
Scope: Append as leader and follower, bounded reads, log end offset, log start offset, high watermark persistence hooks.

### [To Do] STOR-0072 `LogOffsetMetadata`: message offset, segment base offset, relative position, with unknown sentinels and `on_older_segment`, `on_same_segment`, `position_diff`, `message_offset_only`
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogOffsetMetadata.java @4.4.0
Done when: Methods match Kafka for all sentinel combinations in table-driven tests; comparing positions across segments is an invariant violation.

### [To Do] STOR-0073 `AppendOrigin` (Client, Coordinator, Replication, RaftLeader) and `LogAppendInfo` (first and last offset, last leader epoch, max timestamp, log append time, log start offset, validation stats, source compression, valid bytes, last offset of first batch)
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/AppendOrigin.java @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogAppendInfo.java @4.4.0
Done when: Types documented with when each field is set; `Debug` output never includes payload bytes.

### [To Do] STOR-0074 `LocalLog`: partition directory, config, segments, recovery point, log end offset metadata, last flush time, closed state; append to the active segment and advance the log end offset
P0 · M2 · Task · deps: STOR-0072, STOR-F07 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java append, updateLogEndOffset, checkIfClosed @4.4.0
Done when: Operations on a closed log return `LogClosed`; log end offset metadata points at the active segment and its size after each append; unit tests over `MemFs`.

### [To Do] STOR-0075 Analyze and validate incoming batches: Raft-leader epoch check, client base offset must be 0, skip batches whose epoch exceeds the replica's (KAFKA-18723), monotonic offsets for followers, `max.message.bytes`, CRC, max timestamp, source compression, valid bytes
P0 · M2 · Task · deps: STOR-0073 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java analyzeAndValidateRecords, hasHigherPartitionLeaderEpoch @4.4.0
Done when: Each rule has a test producing its error (`RecordTooLarge`, `CorruptRecord`, `InvalidRecord`, `OffsetsOutOfOrder`); skipped batches are counted and logged at info once per append.

### [To Do] STOR-0076 Trim trailing bytes that do not form complete valid batches before appending
P0 · M2 · Task · deps: STOR-0075 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java trimInvalidBytes @4.4.0
Done when: Test with a partial trailing batch appends only the valid prefix.

### [To Do] STOR-0077 Leader append: validate and assign offsets from the log end offset via REC-F12, set log append time, re-check `max.message.bytes` after recompression, reject sets larger than `segment.bytes`, roll if needed, append
P0 · M2 · Task · deps: STOR-0074, STOR-0075, STOR-0076 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java appendAsLeader, append @4.4.0
Done when: Errors `RecordTooLarge` and `RecordBatchTooLarge` raised as Kafka does; offsets assigned contiguously (property test); Raft-leader and coordinator origins covered.

### [To Do] STOR-0078 Follower append with offsets taken from the leader; first offset below the log end offset returns `UnexpectedAppendOffset` with the first ten offsets in the message
P0 · M2 · Task · deps: STOR-0074, STOR-0075, STOR-0076 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java appendAsFollower, append @4.4.0
Done when: Tests cover in-order append, overlap error, and batches skipped for higher leader epochs; no offset reassignment happens.

### [To Do] STOR-0079 Assign leader-epoch start offsets from each appended v2 batch's partition leader epoch
P0 · M2 · Task · deps: STOR-0077, STOR-0078, STOR-F11 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java assignEpochStartOffset @4.4.0
Done when: After appends spanning epochs, the epoch cache holds one entry per new epoch at its first offset; test ports UnifiedLogTest epoch cases.

### [To Do] STOR-0080 High watermark held by the log: clamp updates to [log start offset, log end offset], monotonic increment (or same offset on a newer segment), refresh message-only metadata, clamp on append and truncation
P0 · M2 · Task · deps: STOR-0072, STOR-0074 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java updateHighWatermark, maybeIncrementHighWatermark, maybeUpdateHighWatermark, fetchHighWatermarkMetadata, updateHighWatermarkWithLogEndOffset @4.4.0
Done when: Increment above the log end offset is an invariant violation; property test keeps log start <= high watermark <= log end; each change returns a transition with a reason for the driver.

### [To Do] STOR-0081 Log start offset tracking: initialized from load, readable without locks, reads below it return `OffsetOutOfRange`
P0 · M2 · Task · deps: STOR-0074 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java logStartOffset, checkLogStartOffset, updateLogStartOffset @4.4.0
Done when: Test reads below and at the log start offset; the value is published for fetchers through the offsets snapshot.

### [To Do] STOR-0082 Convert an offset to full offset metadata by locating its segment and position; return message-only metadata when out of range
P0 · M2 · Task · deps: STOR-0074 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java convertToOffsetMetadataOrThrow @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java maybeConvertToOffsetMetadata @4.4.0
Done when: Tests cover offsets in sealed and active segments and past the end.

### [To Do] STOR-0083 LocalLog read: `OffsetOutOfRange` above the log end or below the first segment; empty at the max offset; per-segment max position from the max offset metadata; continue to later segments when a segment has no data
P0 · M2 · Task · deps: STOR-0074, STOR-0082, STOR-F04 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java read @4.4.0
Done when: Tests port LocalLogTest read cases including reads across segment boundaries and from a fully deleted range.

### [To Do] STOR-0084 UnifiedLog read with isolation `LogEnd` and `HighWatermark`: choose the max offset metadata, check the log start offset, apply max bytes and min-one-message
P0 · M2 · Task · deps: STOR-0083, STOR-0080, STOR-0081 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java read @4.4.0
Done when: Consumers never read past the high watermark (property test); `IsolationLevel` is an enum, never a bool.

### [To Do] STOR-0085 Consistent offsets snapshot (log start, log end metadata, high watermark metadata, last stable offset metadata) published for lock-free readers
P0 · M2 · Task · deps: STOR-0080, STOR-0081 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java fetchOffsetSnapshot, lastStableOffset, lastStableOffsetLag @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogOffsetSnapshot.java @4.4.0
Done when: All four values come from one state version; without transactions the last stable offset equals the high watermark; test reads snapshots during concurrent appends in a model checker.

### [To Do] STOR-0086 High-watermark-updated notification returned as an action so drivers can complete delayed fetches and produces
P0 · M2 · Task · deps: STOR-0080 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogOffsetsListener.java onHighWatermarkUpdated @4.4.0
Done when: Each high watermark increase yields exactly one action; tests assert no action on no-op updates.

### [To Do] STOR-0087 Topic id binding: write `partition.metadata` before the first log data, assign on creation, reject a different id with `InconsistentTopicId`
P0 · M2 · Task · deps: STOR-0074, STOR-F10 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java assignTopicId, initializeTopicId, maybeFlushMetadataFile @4.4.0
Done when: Test crashes between metadata write and first append and finds the topic id on restart; mismatch returns the error with both ids.

### [To Do] STOR-0088 Apply an updated `storage::LogConfig` to a live log (topic config change)
P0 · M2 · Task · deps: STOR-0074 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java updateConfig @4.4.0
Done when: New segment and retention settings take effect on the next append or retention run; a test changes `segment.bytes` and observes the next roll.

### [To Do] STOR-0089 Partition-log transitions as returned events (`high_watermark_updated`, `log_start_offset_incremented`, `log_end_offset_advanced` at debug level) with reason enums, recorded by the driver in the partition flight recorder
P0 · M2 · Task · deps: STOR-0080, STOR-0081, OBS-F06, OBS-F07
Done when: Events are listed in the storage `events.rs`; a driver test shows each transition reaches the flight recorder; no logging inside the state machine above debug.

### [To Do] STOR-0090 Partition log gauges: `NumLogSegments`, `LogStartOffset`, `LogEndOffset`, `Size` mapped to `alluvium_log_*` with partition labels only when `alluvium.metrics.per.partition.enabled` is set
P1 · M2 · Task · deps: STOR-0074, OBS-F04 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogMetricNames.java @4.4.0
Done when: Gauges asserted in a test; docs/metrics.md rows added with the Kafka JMX names.

### [To Do] STOR-0091 `LogError` enum with topic, partition and offsets in each variant, and its single exhaustive mapping to Kafka error codes
P0 · M2 · Task · deps: STOR-0077, STOR-0078, ERR-F02 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java @4.4.0
Done when: Mapping covers `KAFKA_STORAGE_ERROR`, `OFFSET_OUT_OF_RANGE`, `RECORD_TOO_LARGE`, `RECORD_BATCH_TOO_LARGE`, `CORRUPT_MESSAGE`, `INVALID_RECORD`, `INVALID_TIMESTAMP`, `INCONSISTENT_TOPIC_ID` with no `_ =>` arm; table test per variant.

### [To Do] STOR-0092 I/O failure handling on the log: any `FsError` during append, read, roll or flush returns `LogError::Storage` and a `LogDirFailed` action; the driver halts the node when its only log dir fails
P0 · M2 · Task · deps: STOR-0074 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java maybeHandleIOException @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogDirFailureChannel.java @4.4.0
Done when: Test injects an append failure with `MemFs` and observes the action and that later operations fail fast; multi-directory handling is STOR-F21.

### [To Do] STOR-0093 Port UnifiedLogTest append and read cases and add a property test of random append, read and roll sequences against a model log
P0 · M2 · Task · deps: STOR-0077, STOR-0078, STOR-0084, TEST-F16 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/UnifiedLogTest.java @4.4.0
Done when: Offsets are contiguous, reads return exactly what was appended up to the bound, and the property test prints its seed on failure.

## [To Do] STOR-F09 Log recovery
P0 · M2 · Feature · deps: STOR-F08, STOR-F10, STOR-F11
Scope: Clean-shutdown marker, recovery point, segment validation and truncation, index rebuild, producer state rebuild.

### [To Do] STOR-0094 Loader pass 1: delete `.deleted` files (except KRaft snapshot deletions), collect `.cleaned` files and their minimum offset, delete `.swap` files at or above it, delete all `.cleaned` files, return valid swap files
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogLoader.java removeTempFilesAndCollectSwapFiles @4.4.0
Done when: One test per file mix from Kafka's LogLoaderTest; unreadable files return `FsError` with the path.

### [To Do] STOR-0095 Loader passes 2-3: for valid `.swap` log files compute [min base, max next offset), delete non-swap segment files in that range, rename `.swap` files to their final names
P0 · M2 · Task · deps: STOR-0094 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogLoader.java load @4.4.0
Done when: Tests reproduce an interrupted compaction swap and an interrupted split and recover the post-swap segments.

### [To Do] STOR-0096 Loader pass 4: open segments in offset order, delete orphan index files, sanity-check each segment (missing offset index or corrupt index triggers recovery; a newly created time index is resized to 0; transaction index sanity)
P0 · M2 · Task · deps: STOR-0095 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogLoader.java loadSegmentFiles @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java sanityCheck @4.4.0
Done when: Tests cover orphan index deletion, missing `.index`, corrupt `.timeindex`; corrupt-index recovery is logged at warn with the path.

### [To Do] STOR-0097 LogSegment truncate to offset: translate offset, truncate offset, time and transaction indexes, re-expand indexes, truncate the file, reset created and rolling timestamps when empty, recompute the max timestamp
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java truncateTo, readLargestTimestamp @4.4.0
Done when: Returns bytes truncated; tests port LogSegmentTest truncate cases including truncation to the base offset and past the end.

### [To Do] STOR-0098 LogSegment recover: reset indexes, validate each batch (CRC and offset range), rebuild offset and time indexes every `index.interval.bytes`, assign leader epochs, truncate invalid tail bytes, trim indexes, write the final time index entry
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java recover @4.4.0
Done when: Returns truncated byte count; tests corrupt a byte, a length and a trailing partial batch and check the valid prefix and rebuilt indexes match a fresh write.

### [To Do] STOR-0099 Recover unflushed segments from the recovery point when there was no clean shutdown; on truncation delete all later unflushed segments; on invalid offset truncate the segment to its base
P0 · M2 · Task · deps: STOR-0096, STOR-0098, STOR-0097 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogLoader.java recoverLog @4.4.0
Done when: Tests port LogLoaderTest recovery cases; recovery never touches segments below the recovery point.

### [To Do] STOR-0100 Recovery tail rules: delete all segments when the log end is below the log start checkpoint, create an empty segment at the log start offset when none exist, compute the new recovery point (log end after clean shutdown, else min(checkpoint, log end))
P0 · M2 · Task · deps: STOR-0099 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogLoader.java deleteSegmentsIfLogStartGreaterThanLogEnd, recoverLog @4.4.0
Done when: Each rule has a test; the recovery point never moves past flushed data.

### [To Do] STOR-0101 Load completion: resize the active segment's indexes, truncate the epoch cache from the end (next offset) and from the start (log start checkpoint), choose the log start offset as max(checkpoint, first segment base), return loaded offsets
P0 · M2 · Task · deps: STOR-0100 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogLoader.java load @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LoadedLogOffsets.java @4.4.0
Done when: Producer state rebuild is invoked through a hook implemented in STOR-F15; tests check epoch cache truncation and returned offsets.

### [To Do] STOR-0102 Loading a partition directory marked for deletion (`-delete` suffix) skips recovery and creates an empty segment if none exist
P0 · M2 · Task · deps: STOR-0101 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogLoader.java load @4.4.0
Done when: Test loads a `-delete` dir with corrupt data without recovering it.

### [To Do] STOR-0103 Record the KAFKA-6264 deviation: segments with relative-offset overflow (only written by message format v0/v1) are not split; the loader reports `LogError::OffsetOverflow` and fails the log dir
P1 · M2 · Task · deps: STOR-0096 · labels: docs · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogLoader.java retryOnOffsetOverflow @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java splitOverflowedSegment @4.4.0
Done when: docs/parity.md has the deviation row with the reason (v0/v1 formats are a non-goal); a test with a crafted overflowed segment gets the error and no panic.

### [To Do] STOR-0104 Recovery progress reporting: remaining segments per recovery thread returned by the loader and exported by the driver
P1 · M2 · Task · deps: STOR-0099, OBS-F04 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java addLogRecoveryMetrics @4.4.0
Done when: `alluvium_log_recovery_remaining_segments{dir}` gauge (Kafka `remainingSegmentsToRecover`) asserted in a test; docs/metrics.md row added.

### [To Do] STOR-0105 Recovery events with reasons: `segment_recovered` (truncated bytes, warn when > 0), `index_rebuilt` (missing or corrupt), `orphan_index_deleted`, `swap_completed`, `segments_deleted_log_start_beyond_end`
P0 · M2 · Task · deps: STOR-0099, OBS-F07
Done when: Events listed in `events.rs`; a recovery test asserts each event is produced once per occurrence.

### [To Do] STOR-0106 Port LogLoaderTest cases (clean shutdown skip, corrupt index rebuild, swap and cleaned files, recovery with corrupt message, log start beyond log end)
P0 · M2 · Task · deps: STOR-0101, STOR-0102 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/LogLoaderTest.java @4.4.0
Done when: Tests carry `// Ported from:` and pass over `MemFs`.

### [To Do] STOR-0107 Fuzz target `segment_recover`: recover arbitrary segment bytes and check the result is a valid prefix
P0 · M2 · Task · deps: STOR-0098, TEST-F07 · labels: test
Done when: Runs clean in CI's short fuzz job; asserts every remaining batch passes CRC and offsets are increasing.

## [To Do] STOR-F10 Checkpoint files
P0 · M2 · Feature · deps: STOR-F01
Scope: Recovery point, log start offset, replication offset (HW) and leader-epoch checkpoints with atomic write (tmp, fsync, rename, dir fsync).

### [To Do] STOR-0108 Generic text checkpoint codec: version line, entry-count line, one entry per line; reading validates the version, the count and every line
P0 · M2 · Task · deps: none · labels: decoder, security · ref: server-common/src/main/java/org/apache/kafka/server/common/CheckpointFile.java CheckpointReadBuffer, CheckpointWriteBuffer @4.4.0
Done when: Unknown version, count mismatch and malformed lines return `CheckpointError { path, line_number, reason }`; line length and entry count are bounded before allocation; fuzz target `checkpoint_text` runs clean.

### [To Do] STOR-0109 Atomic checkpoint write through the `Fs` atomic-replace helper, including the "write only if the directory still exists" variant used after renames
P0 · M2 · Task · deps: STOR-0108 · ref: server-common/src/main/java/org/apache/kafka/server/common/CheckpointFile.java write @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/CheckpointFileWithFailureHandler.java write, writeIfDirExists @4.4.0
Done when: A write failure returns `CheckpointError` plus a `LogDirFailed` action; `write_if_dir_exists` is a no-op for a deleted directory; tests over `MemFs`.

### [To Do] STOR-0110 Offset checkpoint format (version 0, `<topic> <partition> <offset>` lines) used by `recovery-point-offset-checkpoint`, `log-start-offset-checkpoint`, `replication-offset-checkpoint` and `cleaner-offset-checkpoint`
P0 · M2 · Task · deps: STOR-0108 · ref: storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/OffsetCheckpointFile.java @4.4.0
Done when: Golden file written by Kafka 4.4.0 decodes to the same map; round-trip property test; duplicate partitions are rejected.

### [To Do] STOR-0111 Lazy per-directory offset checkpoint reader that loads each file once on first lookup
P0 · M2 · Task · deps: STOR-0110 · ref: storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/LazyOffsetCheckpoints.java @4.4.0
Done when: A test with three log dirs reads each file at most once; missing files read as empty.

### [To Do] STOR-0112 Leader epoch checkpoint format (version 0, `<epoch> <start offset>` lines) in `leader-epoch-checkpoint`
P0 · M2 · Task · deps: STOR-0108 · ref: storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/LeaderEpochCheckpointFile.java @4.4.0
Done when: Golden file from Kafka decodes identically; negative epochs or offsets are rejected; round-trip property test.

### [To Do] STOR-0113 `partition.metadata` file (`version: 0` and `topic_id: <base64url uuid>` lines) with record-then-flush semantics
P0 · M2 · Task · deps: none · labels: decoder · ref: storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/PartitionMetadataFile.java @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/PartitionMetadataReadBuffer.java @4.4.0
Done when: Golden file decodes; wrong version or malformed topic id returns `CheckpointError`; `maybe_flush` writes atomically only when dirty; fuzz target `partition_metadata` runs clean.

### [To Do] STOR-0114 Clean shutdown marker `.kafka_cleanshutdown` (JSON `{"version":0,"brokerEpoch":N}`): write, read, delete, exists
P0 · M2 · Task · deps: none · labels: decoder · ref: storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/CleanShutdownFileHandler.java @4.4.0
Done when: Golden file decodes to the broker epoch; unparsable content reads as "no broker epoch" with a warn event; write uses the atomic helper; fuzz target `clean_shutdown_marker` runs clean.

## [To Do] STOR-F11 Leader epoch cache
P0 · M2 · Feature · deps: STOR-F10
Scope: Epoch-to-start-offset entries, assign, truncate from start and end, end offset for epoch.

### [To Do] STOR-0115 Leader epoch cache model: ordered epoch to start-offset entries loaded from the checkpoint, with the read accessors
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/epoch/LeaderEpochFileCache.java latestEntry, latestEpoch, previousEpoch, earliestEntry, epochEntry, nextEpoch, previousEntry @4.4.0
Done when: Accessors match Kafka in table tests; loading a checkpoint restores entries in order; a non-monotonic checkpoint returns a typed error.

### [To Do] STOR-0116 Assign an epoch start offset: reject negative values, skip when not needed, remove conflicting later entries (epoch >= new or start >= new) and warn when that removes more than the replaced entry
P0 · M2 · Task · deps: STOR-0115 · ref: storage/src/main/java/org/apache/kafka/storage/internals/epoch/LeaderEpochFileCache.java assign, isUpdateNeeded, maybeTruncateNonMonotonicEntries @4.4.0
Done when: Property test: entries stay strictly increasing in epoch and start offset; batch `assign` of a list behaves like repeated single assigns.

### [To Do] STOR-0117 End offset for epoch (OffsetForLeaderEpoch semantics): undefined epoch, latest epoch returns log end offset, no higher entry returns undefined, no floor returns (requested, higher start), else (floor epoch, higher start)
P0 · M2 · Task · deps: STOR-0115 · ref: storage/src/main/java/org/apache/kafka/storage/internals/epoch/LeaderEpochFileCache.java endOffsetFor @4.4.0
Done when: Each branch tested; property test against a reference model ported from LeaderEpochFileCacheTest.

### [To Do] STOR-0118 Truncate from end: remove entries with start offset >= end offset and request a deferred checkpoint flush
P0 · M2 · Task · deps: STOR-0115 · ref: storage/src/main/java/org/apache/kafka/storage/internals/epoch/LeaderEpochFileCache.java truncateFromEndAsyncFlush @4.4.0
Done when: No-op when nothing is removed (no flush requested); tests cover removing all entries.

### [To Do] STOR-0119 Truncate from start: remove entries with start offset <= new start and re-add the last removed epoch at the new start offset; request a deferred flush
P0 · M2 · Task · deps: STOR-0115 · ref: storage/src/main/java/org/apache/kafka/storage/internals/epoch/LeaderEpochFileCache.java truncateFromStartAsyncFlush @4.4.0
Done when: Tests port Kafka cases including truncation exactly at an entry and before the first entry.

### [To Do] STOR-0120 Epoch for offset, entries in an offset range, clear and clear-and-flush
P0 · M2 · Task · deps: STOR-0115 · ref: storage/src/main/java/org/apache/kafka/storage/internals/epoch/LeaderEpochFileCache.java epochForOffset, epochEntriesInRange, clearAndFlush, clear @4.4.0
Done when: Tests cover offsets before the first entry (none) and range boundaries; used by remote copy and reads.

### [To Do] STOR-0121 Epoch checkpoint persistence as actions: assign returns a synchronous flush, truncations return a deferred flush that the driver runs with write-if-dir-exists semantics
P0 · M2 · Task · deps: STOR-0116, STOR-0118, STOR-0119 · ref: storage/src/main/java/org/apache/kafka/storage/internals/epoch/LeaderEpochFileCache.java writeToFile, writeIfDirExists @4.4.0
Done when: A driver test shows deferred flushes coalesce and never write into a renamed or deleted directory; loss of a deferred flush is repaired at load (tested in STOR-F20).

### [To Do] STOR-0122 Epoch cache events: `leader_epoch_assigned`, `leader_epoch_cache_truncated` with reason (from_start, from_end, conflicting_entry)
P0 · M2 · Task · deps: STOR-0116, STOR-0118, STOR-0119, OBS-F07
Done when: Events listed in `events.rs` and recorded in the partition flight recorder by the driver.

### [To Do] STOR-0123 Port LeaderEpochFileCacheTest
P0 · M2 · Task · deps: STOR-0117, STOR-0120, STOR-0121 · labels: test · ref: core/src/test/scala/unit/kafka/server/epoch/LeaderEpochFileCacheTest.scala @4.4.0
Done when: All applicable tests ported with `// Ported from:` and passing.

## [To Do] STOR-F12 Truncation
P0 · M3 · Feature · deps: STOR-F08, STOR-F11
Scope: Truncate to offset, truncate fully and start at offset, with index and epoch cache updates.

### [To Do] STOR-0124 LocalLog truncate to offset: remove segments with base offset > target (asynchronous deletion), truncate the floor segment, update the log end offset
P0 · M3 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java truncateTo @4.4.0
Done when: Returns the removed segments; tests truncate within a segment, at a boundary and to the first segment's base.

### [To Do] STOR-0125 UnifiedLog truncate to offset: no-op at or above the log end; full reset when below the first segment; otherwise truncate data, epoch cache from end, recovery point and high watermark clamps, producer state reload hook, first unstable offset
P0 · M3 · Task · deps: STOR-0124 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java truncateTo @4.4.0
Done when: Negative target is an invariant violation; returns whether anything was truncated; tests port UnifiedLogTest truncation cases.

### [To Do] STOR-0126 Truncate fully and start at an offset: delete all segments, create a segment at the new offset, clear the epoch cache, reset log end, high watermark, recovery point and optionally the log start offset
P0 · M3 · Task · deps: STOR-0124 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java truncateFullyAndStartAt @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java truncateFullyAndStartAt @4.4.0
Done when: Test resets a log with data in several segments and checks every offset and checkpointable value.

### [To Do] STOR-0127 `log_truncated` event with old and new log end offset and the caller's reason (diverging_epoch, high_watermark, fully_reset), recorded in the partition flight recorder
P0 · M3 · Task · deps: STOR-0125, STOR-0126, OBS-F07
Done when: Event listed in `events.rs`; replication can pass its reason through; a test asserts the event fields.

### [To Do] STOR-0128 LogManager truncation entry points for sets of partitions, refreshing the recovery-point checkpoint when truncation removes data at or below it
P0 · M3 · Task · deps: STOR-0125, STOR-0126, STOR-F16 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java truncateTo, truncateFullyAndStartAt @4.4.0
Done when: After truncation and crash, recovery does not resurrect truncated data (test with `MemFs`); future logs are handled by STOR-F22.

### [To Do] STOR-0129 Property test: after `truncate_to(x)` the log end is x, reads at or above x fail, the epoch cache has no entry at or above x, and the high watermark <= x
P0 · M3 · Task · deps: STOR-0125, STOR-0126 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/UnifiedLogTest.java @4.4.0
Done when: Runs with random append and truncate sequences and prints its seed on failure.

## [To Do] STOR-F13 Retention
P0 · M2 · Feature · deps: STOR-F08
Scope: Time and size retention, log start offset advance (DeleteRecords), asynchronous segment deletion.

### [To Do] STOR-0130 Retention fields in `storage::LogConfig` (`retention.ms`, `retention.bytes`, `cleanup.policy` delete/compact, `file.delete.delay.ms`) and their mapping from topic overrides and `log.retention.{ms,minutes,hours}`, `log.retention.bytes`, `log.segment.delete.delay.ms`
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java @4.4.0
Done when: Precedence of ms over minutes over hours matches Kafka in table tests; -1 means unlimited.

### [To Do] STOR-0131 Deletable segment selection: walk from the oldest segment while the predicate holds, only segments whose upper bound <= high watermark, never the last empty segment; remote-copy gating hook for TIER
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java deletableSegments, isSegmentEligibleForDeletion @4.4.0
Done when: Tests show retention never deletes data above the high watermark and stops at the first non-matching segment.

### [To Do] STOR-0132 Delete segments: roll first when every segment is deletable (handle the empty-active recreate case), advance the log start offset to the next segment's base with reason `SegmentDeletion`, remove from the map, rename to `.deleted` for asynchronous removal
P0 · M2 · Task · deps: STOR-0131 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java deleteSegments @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java removeAndDeleteSegments, deleteSegmentFiles @4.4.0
Done when: Test deletes all segments of a log and keeps one empty active segment at the right base; the log start offset moves before files are renamed.

### [To Do] STOR-0133 Time retention: delete while now - segment largest timestamp > `retention.ms`; segments with future timestamps are ineligible and warned about at most once per interval
P0 · M2 · Task · deps: STOR-0131 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java deleteRetentionMsBreachedSegments @4.4.0
Done when: Tests with the simulated clock cover a segment without record timestamps (last-modified fallback) and a future timestamp.

### [To Do] STOR-0134 Size retention: delete oldest segments while (log size - `retention.bytes`) - segment size >= 0
P0 · M2 · Task · deps: STOR-0131 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java deleteRetentionSizeBreachedSegments @4.4.0
Done when: Tests port Kafka's size retention cases; the active segment is never counted as deletable unless rolled.

### [To Do] STOR-0135 Log-start-offset retention: delete a segment when the next segment's base <= log start offset
P0 · M2 · Task · deps: STOR-0131 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java deleteLogStartOffsetBreachedSegments @4.4.0
Done when: Test after DeleteRecords deletes exactly the segments fully below the new log start offset.

### [To Do] STOR-0136 Retention orchestration by `cleanup.policy` (delete: start offset, size, time; compact only: start offset) and the retention-size-in-percent value
P0 · M2 · Task · deps: STOR-0133, STOR-0134, STOR-0135, STOR-0132 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java deleteOldSegments, calculateRetentionSizeInPercent @4.4.0
Done when: Table test per policy; `RetentionSizeInPercent` value computed exactly as Kafka for the driver to export.

### [To Do] STOR-0137 Increment the log start offset with a reason (LeaderOffsetIncremented, SegmentDeletion, ClientRecordDeletion, SnapshotGenerated): reject values above the high watermark, truncate the epoch cache from the start, notify producer state, update the first unstable offset
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java maybeIncrementLogStartOffset @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogStartOffsetIncrementReason.java @4.4.0
Done when: Above-high-watermark returns `OffsetOutOfRange`; lower values are no-ops returning false; used by DeleteRecords (API-F21).

### [To Do] STOR-0138 Asynchronous deletion of `.deleted` segment files after `file.delete.delay.ms` as a timer action executed by the driver on the storage executor
P0 · M2 · Task · deps: STOR-0132 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java deleteSegmentFiles @4.4.0
Done when: A failure while deleting yields `LogDirFailed`; a test with the simulated clock checks files are removed only after the delay.

### [To Do] STOR-0139 Retention observability: `segment_deleted` event with reason (retention_ms, retention_bytes, log_start_offset, log_truncation, log_deletion) and sizes; `log_start_offset_incremented` with reason; counter `alluvium_log_segments_deleted_total{reason}`; `RetentionSizeInPercent` gauge
P0 · M2 · Task · deps: STOR-0136, STOR-0137, OBS-F07, OBS-F04 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/SegmentDeletionReason.java @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogMetricNames.java @4.4.0
Done when: Events listed in `events.rs`; metrics asserted in tests; docs/metrics.md rows added.

### [To Do] STOR-0140 Port UnifiedLogTest retention and DeleteRecords cases and add a property test that retention never moves the log start offset above the high watermark
P0 · M2 · Task · deps: STOR-0136, STOR-0137 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/UnifiedLogTest.java @4.4.0
Done when: Tests carry `// Ported from:`; the property test prints its seed on failure.

## [To Do] STOR-F14 Log compaction
P0 · M5 · Feature · deps: STOR-F08, STOR-F13, REC-F04
Scope: Cleaner: dirty ratio, compaction lags, tombstone retention, offset map, cleaner checkpoints, transaction marker handling, compact+delete.

### [To Do] STOR-0141 Cleaner configuration struct: `log.cleaner.enable`, `log.cleaner.threads`, `log.cleaner.dedupe.buffer.size`, `log.cleaner.io.buffer.size`, `log.cleaner.io.buffer.load.factor`, `log.cleaner.io.max.bytes.per.second`, `log.cleaner.backoff.ms`, plus topic `min.cleanable.dirty.ratio`, `min.compaction.lag.ms`, `max.compaction.lag.ms`, `delete.retention.ms`
P0 · M5 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerConfig.java @4.4.0
Done when: Defaults and validation match Kafka (dedupe buffer per thread = size / threads); mapping tested in `alluvium-config`.

### [To Do] STOR-0142 Bounded offset map for deduplication: fixed slots of (16-byte key hash, offset) sized from the dedupe buffer and load factor, probing on collision, latest offset and utilization
P0 · M5 · Task · deps: none · labels: perf · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/SkimpyOffsetMap.java @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/OffsetMap.java @4.4.0
Done when: Memory never exceeds the configured buffer (test); property test against a `BTreeMap` model for put and get; hash function and its collision trade-off documented in the module doc with the dependency justification.

### [To Do] STOR-0143 Cleaner manager state machine per partition: none, in progress, aborted, paused(n); abort, abort-and-pause, resume, done-cleaning and done-deleting transitions
P0 · M5 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleaningState.java @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleanerManager.java abortCleaning, abortAndPauseCleaning, resumeCleaning, checkCleaningAborted, doneCleaning, doneDeleting @4.4.0
Done when: Invalid transitions are invariant violations; property test over random transition sequences; every transition returns a `cleaning_state_changed` event with a reason.

### [To Do] STOR-0144 Cleanable range: first dirty offset from the cleaner checkpoint (reset to the log start with a forced checkpoint update when below it or above the log end); first uncleanable = min(last stable offset, active segment base, first segment within `min.compaction.lag.ms`)
P0 · M5 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleanerManager.java cleanableOffsets, findFirstUncleanableSegment @4.4.0
Done when: Each reset case and each bound has a test with the simulated clock.

### [To Do] STOR-0145 `max.compaction.lag.ms`: compute how long the earliest dirty record has waited and flag logs that must be compacted now
P0 · M5 · Task · deps: STOR-0144 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleanerManager.java maxCompactionDelay @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java getFirstBatchTimestampForSegments @4.4.0
Done when: Tests show a log below the dirty ratio is still selected once its delay exceeds `max.compaction.lag.ms`.

### [To Do] STOR-0146 Select the filthiest log: compacted, not in progress, not uncleanable; compute cleanable and total bytes and ratio; pick logs that must be compacted now or exceed `min.cleanable.dirty.ratio`; mark the winner in progress
P0 · M5 · Task · deps: STOR-0143, STOR-0144, STOR-0145 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleanerManager.java grabFilthiestCompactedLog, calculateCleanableBytes @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogToClean.java @4.4.0
Done when: Deterministic tie-breaking (by topic partition) documented and tested; empty logs skipped; dirtiest ratio returned for the `max-dirty-percent` metric.

### [To Do] STOR-0147 Retention on `compact,delete` logs: retention runs only while the cleaner does not own the log, using the in-progress state as a lock
P0 · M5 · Task · deps: STOR-0143 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleanerManager.java deletableLogs, pauseCleaningForNonCompactedPartitions @4.4.0
Done when: Test shows retention and cleaning never run on the same log at the same time.

### [To Do] STOR-0148 `cleaner-offset-checkpoint` per log dir: update after cleaning, truncate on log truncation, move between dirs, drop on dir failure
P0 · M5 · Task · deps: STOR-0143, STOR-F10 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleanerManager.java updateCheckpoints, maybeTruncateCheckpoint, alterCheckpointDir, handleLogDirFailure, allCleanerCheckpoints @4.4.0
Done when: Tests cover each operation; writes use the atomic checkpoint path.

### [To Do] STOR-0149 Build the offset map over dirty segments up to the first uncleanable offset, skipping control batches and aborted-transaction batches, stopping when the map is full
P0 · M5 · Task · deps: STOR-0142, STOR-0146 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/Cleaner.java buildOffsetMap, buildOffsetMapForSegment @4.4.0
Done when: The map's latest offset defines the cleaning end; a test with a small buffer cleans in several passes and converges.

### [To Do] STOR-0150 Group consecutive segments for cleaning so each group fits `segment.bytes`, `segment.index.bytes` and a representable relative offset range
P0 · M5 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/Cleaner.java groupSegmentsBySize, lastOffsetForFirstSegment @4.4.0
Done when: Tests port LogCleanerTest grouping cases including an offset range near `i32::MAX`.

### [To Do] STOR-0151 Record retention rule: keep records past the map's latest offset; drop keyless records (counted as invalid); keep the latest offset per key; keep tombstones until the batch delete horizon passes
P0 · M5 · Task · deps: STOR-0142 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/Cleaner.java shouldRetainRecord @4.4.0
Done when: Table tests for every branch; keyless records increment the cleaner's invalid-message stat.

### [To Do] STOR-0152 Batch retention rule: retain an empty batch when it is the last batch of an active producer or ends at the high watermark; delete or delete-if-empty otherwise; discard control batches once their transaction is resolved and their delete horizon passed
P0 · M5 · Task · deps: STOR-0151 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/Cleaner.java cleanInto, shouldDiscardBatch @4.4.0
Done when: Tests cover each `BatchRetention` outcome, including the producer last-sequence preservation case.

### [To Do] STOR-0153 Batch filtering for uncompressed batches: rebuild retained records into new batches preserving base offset, producer id, epoch, base sequence and timestamps
P0 · M5 · Task · deps: STOR-0151, STOR-0152, REC-F04 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecords.java filterTo @4.4.0
Done when: Retained offsets are unchanged; output batches pass CRC; empty retained batches keep their header when the rule says so; golden test against Kafka's filterTo output.

### [To Do] STOR-0154 Batch filtering for compressed batches: decompress within `max.message.bytes` bounds, recompress with the source codec, grow read and write buffers up to the max message size or fail
P0 · M5 · Task · deps: STOR-0153 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecords.java filterTo @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/Cleaner.java growBuffersOrFail @4.4.0
Done when: A batch larger than the I/O buffer is cleaned after buffer growth; one above `max.message.bytes` aborts with a typed error; decompression is bounded (REC-F10).

### [To Do] STOR-0155 Delete horizon (KIP-534): the first pass that retains tombstones or transaction markers sets the batch delete-horizon flag and timestamp to now + `delete.retention.ms`; later passes remove them after the horizon
P0 · M5 · Task · deps: STOR-0153 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecords.java filterTo @4.4.0; clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java @4.4.0
Done when: Test with the simulated clock shows a tombstone kept for exactly `delete.retention.ms` after its first cleaning, then removed.

### [To Do] STOR-0156 Clean a segment group into a `.cleaned` segment: clean each source segment, seal, flush, copy the last source last-modified time, then replace; delete the cleaned file on abort
P0 · M5 · Task · deps: STOR-0149, STOR-0150, STOR-0154, STOR-0155 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/Cleaner.java cleanSegments @4.4.0
Done when: Abort mid-group leaves the log unchanged; a test compares the result with a model compaction.

### [To Do] STOR-0157 Crash-safe segment replacement: rename `.cleaned` to `.swap` (newest first), add to the map, rename old segments to `.deleted` for async deletion, rename `.swap` to final, fsync the directory
P0 · M5 · Task · deps: STOR-0156 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java replaceSegments @4.4.0
Done when: Each step's crash is recovered by the loader's swap rules (STOR-F09; crash tests in STOR-F20); a test checks readers see either old or new segments, never a gap.

### [To Do] STOR-0158 Cleaning pass orchestration: legacy delete horizon, offset map, segment groups, end offset = map latest offset + 1, checkpoint update, stats (bytes and messages read and written, buffer utilization, elapsed)
P0 · M5 · Task · deps: STOR-0156, STOR-0157, STOR-0148 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/Cleaner.java doClean, clean @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerStats.java @4.4.0
Done when: Stats are returned to the driver; a test cleans a log twice and the second pass is a no-op.

### [To Do] STOR-0159 Cleaner tasks: `log.cleaner.threads` supervised tasks on the storage executor, backoff `log.cleaner.backoff.ms` when idle, cleaning errors mark the partition uncleanable and continue
P0 · M5 · Task · deps: STOR-0158, STOR-0146, STOR-F18 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleaner.java CleanerThread, cleanFilthiestLog @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleanerManager.java markPartitionUncleanable, maintainUncleanablePartitions @4.4.0
Done when: A corrupt segment makes its partition uncleanable without stopping other partitions; shutdown aborts in-progress cleaning; tasks are named `log-cleaner-<n>`.

### [To Do] STOR-0160 Shared I/O throttle for all cleaner tasks at `log.cleaner.io.max.bytes.per.second`, using the injected clock and timer
P0 · M5 · Task · deps: STOR-0159 · ref: storage/src/main/java/org/apache/kafka/storage/internals/utils/Throttler.java maybeThrottle @4.4.0
Done when: Under the simulated clock, measured cleaner I/O stays within 5% of the limit over a 60 s window; no thread sleeps.

### [To Do] STOR-0161 Transaction-aware cleaning: track aborted transactions per producer from the transaction index, drop aborted data, drop markers once their transaction's data is gone and the horizon passed, write retained aborted transactions to the cleaned segment's transaction index
P0 · M6 · Task · deps: STOR-0156, STOR-F06 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanedTransactionMetadata.java @4.4.0
Done when: Tests port LogCleanerTest transaction cases; read_committed consumers see the same data before and after cleaning.

### [To Do] STOR-0162 Cleaner integration with the log manager: abort and pause around truncation, deletion and directory moves; truncate the cleaner checkpoint when a topic becomes compacted
P0 · M5 · Task · deps: STOR-0143, STOR-F16 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java abortAndPauseCleaning, resumeCleaning, maybeTruncateCleanerCheckpointToActiveSegmentBaseOffset @4.4.0
Done when: Tests show truncation of a log being cleaned aborts the pass and resumes cleaning afterwards.

### [To Do] STOR-0163 Dynamic reconfiguration of `log.cleaner.threads`, dedupe buffer, I/O buffer and I/O throttle
P0 · M7 · Task · deps: STOR-0159, CFG-F04 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleaner.java reconfigure, validateReconfiguration @4.4.0
Done when: Invalid values (threads < 1, more than doubling or halving thread count) are rejected as Kafka does; cleaner restarts with new settings without losing checkpoint progress.

### [To Do] STOR-0164 Cleaner metrics: `max-buffer-utilization-percent`, `cleaner-recopy-percent`, `max-clean-time-secs`, `max-compaction-delay-secs`, `DeadThreadCount`, `uncleanable-partitions-count`, `uncleanable-bytes`, `max-dirty-percent`, `time-since-last-run-ms`
P1 · M5 · Task · deps: STOR-0159, OBS-F04 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleaner.java registerMetrics @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleanerManager.java registerMetrics @4.4.0
Done when: Each metric asserted in a test; docs/metrics.md rows added with Kafka names.

### [To Do] STOR-0165 Cleaner events: `log_cleaned` (bytes in and out, duration, end offset), `cleaning_aborted` with reason, `partition_marked_uncleanable` with cause
P0 · M5 · Task · deps: STOR-0159, OBS-F07
Done when: Events listed in `events.rs`; uncleanable marking logs at warn once.

### [To Do] STOR-0166 Compaction correctness: property test that after cleaning every key's latest value is retained, offsets are unchanged and increasing, nothing above the first uncleanable offset changes, and tombstones disappear only after the horizon; port key LogCleanerTest cases
P0 · M5 · Task · deps: STOR-0158 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/LogCleanerTest.java @4.4.0
Done when: Property test prints its seed on failure; ported tests carry `// Ported from:`.

### [To Do] STOR-0167 Port LogCleanerManagerTest (selection, lags, checkpoint resets, state transitions)
P0 · M5 · Task · deps: STOR-0146, STOR-0145 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/LogCleanerManagerTest.java @4.4.0
Done when: All applicable tests pass with the simulated clock.

### [To Do] STOR-0168 Cleaner benchmark: throughput (MB/s) and dedupe memory for logs with 10%, 50% and 90% duplicate keys
P2 · M5 · Task · deps: STOR-0158, TEST-F11 · labels: perf
Done when: Criterion benchmark in CI with a regression threshold; results recorded in the PR.

## [To Do] STOR-F15 Producer state
P0 · M6 · Feature · deps: STOR-F08, STOR-F10
Scope: Per-producer epoch, last five batch sequences, transaction start offsets, snapshots and expiration.

### [To Do] STOR-0169 Producer state entry: producer id, epoch, coordinator epoch, last timestamp, current transaction first offset, and the last five batches' metadata (last sequence, last offset, offset delta, timestamp)
P0 · M6 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateEntry.java @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/BatchMetadata.java @4.4.0
Done when: Adding a sixth batch evicts the oldest; `batch_with_sequence_range` lookup is exposed for TXN-F01's duplicate detection; an epoch change clears batch metadata.

### [To Do] STOR-0170 Per-append producer update: data batches extend the entry and open transactions; end markers close them as `CompletedTxn`; an older coordinator epoch on a marker returns `TransactionCoordinatorFenced` for non-replication origins
P0 · M6 · Task · deps: STOR-0169 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java append, appendDataBatch, appendEndTxnMarker, checkCoordinatorEpoch @4.4.0
Done when: Sequence and epoch validation are delegated to TXN-F01's rules through one call; tests cover begin, commit and abort and coordinator fencing.

### [To Do] STOR-0171 Producer state manager core: producers map, ongoing transactions by first offset, unreplicated transactions, map end offset
P0 · M6 · Task · deps: STOR-0170 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java prepareUpdate, update, updateMapEndOffset, lastEntry, activeProducers, producerIdCount @4.4.0
Done when: Deterministic iteration; property test that applying updates in order reproduces the same state as replaying the log.

### [To Do] STOR-0172 First unstable offset and last stable offset: earliest unreplicated or ongoing transaction; completed transactions move out on high-watermark advance; last stable offset for a completed transaction
P0 · M6 · Task · deps: STOR-0171 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java firstUnstableOffset, firstUndecidedOffset, onHighWatermarkUpdated, lastStableOffset, completeTxn @4.4.0
Done when: Property test: the last stable offset never exceeds the first offset of an ongoing transaction and never decreases.

### [To Do] STOR-0173 Producer state in the append path: prepare updates per producer, detect duplicates, append, apply updates, write aborted transactions to the transaction index with their last stable offset, update the map end offset and the first unstable offset
P0 · M6 · Task · deps: STOR-0171, STOR-0172, STOR-F06 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java analyzeAndValidateProducerState, maybeIncrementFirstUnstableOffset, append @4.4.0
Done when: A duplicate batch returns the original offsets without appending; a failed transaction-index write leaves the last stable offset unchanged (test).

### [To Do] STOR-0174 UnifiedLog read with isolation `TxnCommitted`: bound reads by last stable offset metadata and include aborted transactions
P0 · M6 · Task · deps: STOR-0173, STOR-0172, STOR-0060 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java read, fetchLastStableOffsetMetadata @4.4.0
Done when: read_committed reads never pass the last stable offset (property test); aborted transaction lists match LocalLog collection.

### [To Do] STOR-0175 Producer snapshot file `<20-digit offset>.snapshot`: version (int16, 1), CRC32C over the entries, entries (producer id, epoch, last sequence, last offset, offset delta, timestamp, coordinator epoch, current txn first offset)
P0 · M6 · Task · deps: STOR-0169 · labels: decoder, security · ref: storage/src/main/resources/message/ProducerSnapshot.json @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java readSnapshot, writeSnapshot @4.4.0
Done when: Golden file from Kafka decodes identically; bad version or CRC returns `CorruptSnapshot { path }`; fuzz target `producer_snapshot` runs clean.

### [To Do] STOR-0176 Take producer snapshots at the map end offset, including on every segment roll at the new base offset, with the fsync deferred to the roll's asynchronous flush
P0 · M6 · Task · deps: STOR-0175, STOR-0171 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java takeSnapshot @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java roll, flushProducerStateSnapshot @4.4.0
Done when: A snapshot exists for every segment base after rolls; a test crashes before the deferred flush and recovery rebuilds state correctly.

### [To Do] STOR-0177 Snapshot file management: list, remove strays, delete before an offset, oldest and latest, fetch by offset, mark for deletion (`.deleted`) with segment deletion
P0 · M6 · Task · deps: STOR-0176 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java loadSnapshots, removeStraySnapshots, deleteSnapshotsBefore, latestSnapshotOffset, oldestSnapshotOffset, snapshotFileForOffset, fetchSnapshot, removeAndMarkSnapshotForDeletion @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/SnapshotFile.java @4.4.0
Done when: Tests port ProducerStateManagerTest snapshot management cases.

### [To Do] STOR-0178 Load from the latest valid snapshot at or below the log end, deleting corrupt snapshots and trying older ones, dropping expired producers and entries below the log start; truncate-and-reload removes snapshots outside (log start, log end]
P0 · M6 · Task · deps: STOR-0177 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java loadFromSnapshot, truncateAndReload @4.4.0
Done when: Tests cover a corrupt latest snapshot and truncation below the latest snapshot.

### [To Do] STOR-0179 Rebuild producer state on load and truncation: clean-shutdown-without-snapshot shortcut (empty snapshots at the last two segment bases and the log end), otherwise reload and replay batches from the map end offset, taking snapshots at the planned offsets
P0 · M6 · Task · deps: STOR-0178, STOR-F09 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java rebuildProducerState, loadProducersFromRecords, updateProducers @4.4.0
Done when: The loader hook from STOR-F09 calls it; tests port UnifiedLogTest producer-state rebuild cases.

### [To Do] STOR-0180 Segment recovery with producer state: replay each recovered batch into a fresh manager built from earlier segments, update the transaction index for completed transactions, snapshot after each recovered segment
P0 · M6 · Task · deps: STOR-0171, STOR-F09 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java updateProducerState @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogLoader.java recoverSegment @4.4.0
Done when: Recovering a segment with transactions yields the same producer state and `.txnindex` as before the crash.

### [To Do] STOR-0181 Producer expiration: every `producer.id.expiration.check.interval.ms`, remove producers with no ongoing transaction whose last timestamp is at least `producer.id.expiration.ms` old
P0 · M6 · Task · deps: STOR-0171 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java removeExpiredProducers, isProducerExpired @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java removeExpiredProducers @4.4.0
Done when: Simulated-clock test expires an idle producer and keeps one with an open transaction; verification state entries expire with the same check.

### [To Do] STOR-0182 Producer state on log start offset changes: drop producers and unreplicated transactions entirely below the new start, delete older snapshots; full reset on truncate-fully
P0 · M6 · Task · deps: STOR-0171 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java onLogStartOffsetIncremented, truncateFullyAndStartAt @4.4.0
Done when: Tests cover DeleteRecords past a producer's last offset and full truncation.

### [To Do] STOR-0183 Late transaction detection: the oldest ongoing transaction's last timestamp is older than `transaction.max.timeout.ms` plus 5 minutes
P0 · M6 · Task · deps: STOR-0171 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java hasLateTransaction, LATE_TRANSACTION_BUFFER_MS @4.4.0
Done when: Simulated-clock test; the flag is exposed for TXN-F13's hanging-transaction detection and a per-broker gauge.

### [To Do] STOR-0184 Transaction verification state (KIP-890): verification entries with lowest sequence and epoch, guards created before AddPartitionsToTxn verification, appends without a matching guard rejected with `INVALID_TXN_STATE`
P0 · M6 · Task · deps: STOR-0171 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/VerificationStateEntry.java @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/VerificationGuard.java @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java maybeStartTransactionVerification, batchMissingRequiredVerification @4.4.0
Done when: Tests cover guard creation, mismatch rejection and clearing after the transaction starts; TXN-F06 drives the flow.

### [To Do] STOR-0185 Producer state queries for DescribeProducers, the cleaner and metrics
P0 · M6 · Task · deps: STOR-0171 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java activeProducers, activeProducersWithLastSequence, lastRecordsOfActiveProducers, producerIdCount @4.4.0
Done when: Output fields match DescribeProducers v0 semantics in tests; producer id count exported as the `ProducerIdCount` gauge by the driver.

### [To Do] STOR-0186 Producer state events at debug level (`producer_expired`, `producer_snapshot_taken`, `producer_snapshot_corrupt` at warn) recorded in the partition flight recorder
P1 · M6 · Task · deps: STOR-0181, STOR-0176, OBS-F07
Done when: Events listed in `events.rs`; the corrupt-snapshot event includes the path.

### [To Do] STOR-0187 Port ProducerStateManagerTest and add a property test of snapshot round-trip and last-stable-offset bounds
P0 · M6 · Task · deps: STOR-0178, STOR-0181, STOR-0172 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/ProducerStateManagerTest.java @4.4.0
Done when: Tests carry `// Ported from:`; the property test prints its seed on failure.

## [To Do] STOR-F16 Log manager
P0 · M2 · Feature · deps: STOR-F08, STOR-F09, SRV-F11
Scope: Parallel log loading, create and delete logs, schedulers for flush, retention and checkpoints.

### [To Do] STOR-0188 Partition directory names: `<topic>-<partition>`, `-delete`, `-future` and `-stray` forms with a 32-hex unique id from the injected RNG, capped at 255 bytes; parse with typed errors
P0 · M2 · Task · deps: none · labels: decoder · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java logDirName, logDeleteDirName, logFutureDirName, logStrayDirName, parseTopicPartitionName, logDirNameWithSuffixCappedLength @4.4.0
Done when: Round-trip property test; names over 255 bytes are capped exactly as Kafka; fuzz target `partition_dir_name` runs clean.

### [To Do] STOR-0189 Startup scan of each live log dir: classify partition directories (normal, delete, future, stray) and ignore non-partition entries
P0 · M2 · Task · deps: STOR-0188 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java loadLogs @4.4.0
Done when: Directories marked `-delete` are queued for deletion; unparsable names are logged and skipped; test with a mixed directory tree.

### [To Do] STOR-0190 Clean shutdown marker lifecycle: read the marker and broker epoch per dir, delete it before loading, write it at shutdown only when loading had completed or the dir had a clean shutdown
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java loadLogs, shutdown, readBrokerEpochFromCleanShutdownFiles @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/CleanShutdownFileHandler.java @4.4.0
Done when: The broker epoch read is exposed for broker registration (KIP-966); a crash during loading forces recovery on the next start (test).

### [To Do] STOR-0191 Parallel log loading with `num.recovery.threads.per.data.dir` loaders per dir, recovery point and log start offset from checkpoints, per-dir remaining-log counts
P0 · M2 · Task · deps: STOR-0189, STOR-0190, STOR-F18 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java loadLogs, loadLog, decNumRemainingLogs @4.4.0
Done when: Logs in one dir load concurrently up to the configured parallelism; an I/O error in one log fails that dir only; a simulation test loads 1,000 partitions deterministically.

### [To Do] STOR-0192 Stray replica detection at load: topic id absent from the metadata image or this broker not in the replica set; rename to `-stray` and schedule deletion
P0 · M2 · Task · deps: STOR-0191, META-F03 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java isStrayReplica, startup @4.4.0
Done when: Tests cover both conditions and a topic re-created with a new id.

### [To Do] STOR-0193 Get or create a log: reuse an existing log (checking topic id), else choose a dir with the fewest partitions (excluding cordoned dirs), create the directory and log, write `partition.metadata`
P0 · M2 · Task · deps: STOR-0188, STOR-0191 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java getOrCreateLog, createLog, nextLogDirs, createLogDirectory, initializingLog, finishedInitializingLog @4.4.0
Done when: Directory choice is deterministic for ties; creation failure on a dir returns `KafkaStorageError` and marks it failed; tests cover existing, new and initializing partitions.

### [To Do] STOR-0194 Asynchronous log deletion: rename the partition dir to `-delete`, remove it from the maps, refresh checkpoints, delete after `file.delete.delay.ms`
P0 · M2 · Task · deps: STOR-0193 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java asyncDelete, deleteLogs, nextDeleteDelayMs @4.4.0
Done when: A crash after the rename deletes the dir on the next start; tests use the simulated clock.

### [To Do] STOR-0195 Log manager scheduled tasks as supervised named tasks: retention (`log.retention.check.interval.ms`), flusher (`log.flush.scheduler.interval.ms`), recovery-point checkpoint (`log.flush.offset.checkpoint.interval.ms`), log-start-offset checkpoint (`log.flush.start.offset.checkpoint.interval.ms`), log deletion; first run after `log.initial.task.delay.ms`
P0 · M2 · Task · deps: STOR-0191, SRV-F02 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java startupWithConfigOverrides @4.4.0
Done when: Tasks appear in `/debug/tasks`; simulated-clock test checks each period; shutdown stops them in order.

### [To Do] STOR-0196 Retention pass across logs: run segment deletion on delete-policy logs and leave compact-only logs to the cleaner
P0 · M2 · Task · deps: STOR-0195, STOR-F13 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java cleanupLogs @4.4.0
Done when: A test with mixed policies deletes only from eligible logs; per-pass duration is reported to the driver.

### [To Do] STOR-0197 Write `recovery-point-offset-checkpoint` and `log-start-offset-checkpoint` per live dir from the current logs
P0 · M2 · Task · deps: STOR-0195, STOR-F10 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java checkpointLogRecoveryOffsets, checkpointLogStartOffsets, checkpointRecoveryOffsetsInDir @4.4.0
Done when: The log start offset checkpoint includes only logs whose start differs from their first segment base or that are remote-enabled, as Kafka does; test reloads values after restart.

### [To Do] STOR-0198 Log manager shutdown: in parallel per dir seal active segment indexes, flush and close every log; write checkpoints; write the clean shutdown marker with the broker epoch; release directory locks
P0 · M2 · Task · deps: STOR-0197, STOR-0190 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java shutdown @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java prepareActiveSegmentForClose @4.4.0
Done when: Each step logs start, end and `elapsed_ms`; a restart after clean shutdown performs no segment recovery (test counts recovered segments = 0).

### [To Do] STOR-0199 Apply topic and broker log config changes to existing and initializing logs
P0 · M2 · Task · deps: STOR-0193 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java updateTopicConfig, topicConfigUpdated, brokerConfigUpdated, reconfigureDefaultLogConfig @4.4.0
Done when: A config change racing with log creation is applied once the log finishes initializing (test).

### [To Do] STOR-0200 Log manager metrics: `OfflineLogDirectoryCount`, `LogDirectoryOffline{logDirectory}`, `CordonedLogDirectoryCount`, `remainingLogsToRecover{dir}`
P1 · M2 · Task · deps: STOR-0191, OBS-F04 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java addLogRecoveryMetrics @4.4.0
Done when: Metrics asserted in tests; docs/metrics.md rows added.

### [To Do] STOR-0201 Log manager events: `log_created`, `log_deleted`, `log_dir_loaded` (logs, recovered segments, `elapsed_ms`), `stray_log_detected` with reason
P0 · M2 · Task · deps: STOR-0193, STOR-0194, OBS-F07
Done when: Events listed in `events.rs`; info level only for lifecycle events.

### [To Do] STOR-0202 "Logs loaded" readiness signal consumed by `/health/ready`
P1 · M2 · Task · deps: STOR-0191, OBS-F09
Done when: Readiness is false until every live dir finishes loading; test toggles it.

### [To Do] STOR-0203 Port LogManagerTest cases (create, get, delete, cleanup, checkpoints, dir selection, clean shutdown)
P0 · M2 · Task · deps: STOR-0193, STOR-0198, STOR-0197 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/LogManagerTest.java @4.4.0
Done when: Tests carry `// Ported from:` and use `MemFs` and the simulated clock.

## [To Do] STOR-F17 Flush policy
P0 · M2 · Feature · deps: STOR-F08, STOR-F18
Scope: `flush.messages`, `flush.ms`, `log.flush.*`; ADR on default fsync behavior.

### [To Do] STOR-0204 Spike: default fsync policy ADR: Kafka parity (`flush.messages` and `flush.ms` unset, durability from replication) versus stricter defaults, with the guarantee per `acks` setting
P0 · M2 · Spike · deps: none · labels: docs · ref: ADR-0001
Done when: ADR accepted; docs/architecture.md §10 row closed; NFR-F03 references the ADR's guarantees.

### [To Do] STOR-0205 Flush to an offset: sync segments in [recovery point, offset), sync the directory when new segments are included, then advance the recovery point and last flush time
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java flush, markFlushed @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java flush, flushUptoOffsetExclusive @4.4.0
Done when: The recovery point never advances without a successful sync (test with `MemFs` failing `sync_data`); flush below the recovery point is a no-op.

### [To Do] STOR-0206 `flush.messages` / `log.flush.interval.messages`: flush after an append when unflushed messages (log end - recovery point) reach the threshold
P0 · M2 · Task · deps: STOR-0205 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java append @4.4.0
Done when: Test with a threshold of 1 flushes after each append; default leaves flushing to the scheduler.

### [To Do] STOR-0207 `flush.ms` / `log.flush.interval.ms`: the flusher task flushes logs whose last flush is older than the threshold
P0 · M2 · Task · deps: STOR-0205, STOR-F16 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java flushDirtyLogs @4.4.0
Done when: Simulated-clock test flushes exactly the stale logs.

### [To Do] STOR-0208 Asynchronous flush after roll: flush up to the new segment's base offset (and the producer snapshot) on the storage executor, outside the append path
P0 · M2 · Task · deps: STOR-0205 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java roll @4.4.0
Done when: The append that triggers a roll does not wait for fsync (test); the recovery point reaches the old segment's end after the deferred flush.

### [To Do] STOR-0209 Flush metric equivalent to `kafka.log:type=LogFlushStats,name=LogFlushRateAndTimeMs`
P1 · M2 · Task · deps: STOR-0205, OBS-F04 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java LOG_FLUSH_TIMER @4.4.0
Done when: `alluvium_log_flush_seconds` histogram asserted in a test; docs/metrics.md row added.

## [To Do] STOR-F18 Storage I/O execution model
P0 · M2 · Feature · deps: STOR-F01
Scope: Spike and ADR: dedicated I/O pool vs io_uring; implementation of the chosen model.

### [To Do] STOR-0210 Spike: storage I/O execution model ADR: dedicated blocking I/O pool versus io_uring with a fallback, measured with prototype benchmarks (append plus fsync, tail reads, random reads) on Linux and macOS; queue bounds and per-partition ordering
P0 · M2 · Spike · deps: none · labels: perf · ref: ADR-0002
Done when: ADR accepted with benchmark data attached; docs/architecture.md §6 and §10 updated; X-LOWLAT-F01 referenced as the place to revisit io_uring.

### [To Do] STOR-0211 Storage executor per the ADR: named worker threads (`alluvium-storage-io-<n>`) consuming a bounded job queue; submission returns a completion handle; a full queue returns `StorageBusy` to the caller
P0 · M2 · Task · deps: STOR-0210
Done when: Jobs for one partition run in submission order (test); queue capacity comes from config; no `std::fs` call runs on a Tokio worker (asserted by a debug thread marker).

### [To Do] STOR-0212 Per-log-dir executor partitioning so a slow or failed disk cannot stall I/O for other dirs
P0 · M2 · Task · deps: STOR-0211
Done when: Simulation test with one artificially slow dir shows appends to the other dir keep their latency; each dir has its own queue and workers.

### [To Do] STOR-0213 Executor lifecycle under the task supervisor: start with the log manager, drain or reject queued jobs at shutdown in the documented order, surface worker panics as process-fatal
P0 · M2 · Task · deps: STOR-0211, SRV-F02
Done when: Shutdown test shows in-flight jobs complete and new submissions fail with `ShuttingDown`; workers appear in `/debug/tasks`.

### [To Do] STOR-0214 Executor metrics: queue depth, capacity, rejections, job latency by operation, worker busy ratio, per dir
P1 · M2 · Task · deps: STOR-0211, OBS-F04
Done when: `alluvium_storage_io_queue_depth{dir}`, `alluvium_storage_io_rejections_total{dir}` and `alluvium_storage_io_job_seconds{op}` asserted in tests.

### [To Do] STOR-0215 Deterministic simulation executor: runs storage jobs inline under the simulated clock with seeded completion ordering for alluvium-testkit
P0 · M2 · Task · deps: STOR-0211, TEST-F01 · labels: test
Done when: The same seed yields the same completion order; storage tests and simulations use it instead of threads.

### [To Do] STOR-0216 Benchmark executor overhead versus direct `Fs` calls for 4 KiB, 64 KiB and 1 MiB appends and reads
P2 · M2 · Task · deps: STOR-0211, TEST-F11 · labels: perf
Done when: Criterion benchmark in CI; overhead per job documented in the PR.

## [To Do] STOR-F19 On-disk format documentation
P1 · M2 · Feature · deps: STOR-F03, STOR-F04, STOR-F05, STOR-F10
Scope: docs/formats/ for every file type, with format versions and hex examples.

### [To Do] STOR-0217 docs/formats/README.md: index of on-disk formats, notation (byte order, field tables, hex examples), and the format versioning policy
P1 · M2 · Task · deps: none · labels: docs · ref: ADR-0007
Done when: Page exists, links every format page, and is linked from docs/standards/debuggability.md.

### [To Do] STOR-0218 docs/formats/log-directory.md: log dir and partition dir layout, `-delete`/`-future`/`-stray` names, per-dir files (`.lock`, checkpoints, `.kafka_cleanshutdown`), and the `.cleaned`/`.swap`/`.deleted` suffix protocol with its crash states
P1 · M2 · Task · deps: STOR-0217 · labels: docs · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java logDirName @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LogFileUtils.java @4.4.0
Done when: Page includes a directory tree example and a table of every suffix transition and what recovery does with it.

### [To Do] STOR-0219 docs/formats/log-segment.md: `.log` file as concatenated record batch v2 bytes, naming, preallocation and trailing zeros, with an annotated hex dump of one batch
P1 · M2 · Task · deps: STOR-0217 · labels: docs · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java @4.4.0
Done when: The hex example is decoded by a golden test so the page cannot drift.

### [To Do] STOR-0220 docs/formats/offset-index.md: entry layout, sparsity rule (`index.interval.bytes`), preallocation and trimming, sanity rules, hex example
P1 · M2 · Task · deps: STOR-0217 · labels: docs · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/OffsetIndex.java @4.4.0
Done when: Hex example decoded by a golden test.

### [To Do] STOR-0221 docs/formats/time-index.md: entry layout, monotonic rules, seal entry, sanity rules, hex example
P1 · M2 · Task · deps: STOR-0217 · labels: docs · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TimeIndex.java @4.4.0
Done when: Hex example decoded by a golden test.

### [To Do] STOR-0222 docs/formats/transaction-index.md: version-prefixed `AbortedTxn` entries, ordering rule, hex example
P1 · M6 · Task · deps: STOR-0217, STOR-F06 · labels: docs · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TransactionIndex.java @4.4.0
Done when: Hex example decoded by a golden test.

### [To Do] STOR-0223 docs/formats/producer-snapshot.md: version, CRC32C coverage, entry fields, file naming, hex example
P1 · M6 · Task · deps: STOR-0217, STOR-F15 · labels: docs · ref: storage/src/main/resources/message/ProducerSnapshot.json @4.4.0
Done when: Hex example decoded by a golden test including CRC verification.

### [To Do] STOR-0224 docs/formats/offset-checkpoint.md: text layout shared by recovery-point, log-start-offset, replication-offset and cleaner-offset checkpoints, atomic write sequence
P1 · M2 · Task · deps: STOR-0217 · labels: docs · ref: storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/OffsetCheckpointFile.java @4.4.0
Done when: Example file is parsed by a golden test.

### [To Do] STOR-0225 docs/formats/leader-epoch-checkpoint.md: text layout and when it is flushed
P1 · M2 · Task · deps: STOR-0217 · labels: docs · ref: storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/LeaderEpochCheckpointFile.java @4.4.0
Done when: Example file is parsed by a golden test.

### [To Do] STOR-0226 docs/formats/partition-metadata.md: `partition.metadata` layout and write-before-data rule
P1 · M2 · Task · deps: STOR-0217 · labels: docs · ref: storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/PartitionMetadataFile.java @4.4.0
Done when: Example file is parsed by a golden test.

### [To Do] STOR-0227 docs/formats/clean-shutdown.md: `.kafka_cleanshutdown` JSON layout and lifecycle
P1 · M2 · Task · deps: STOR-0217 · labels: docs · ref: storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/CleanShutdownFileHandler.java @4.4.0
Done when: Example file is parsed by a golden test.

### [To Do] STOR-0228 Golden test harness that extracts fenced hex and text examples from docs/formats/*.md and decodes them with the storage codecs
P1 · M2 · Task · deps: STOR-0217 · labels: test, docs
Done when: A deliberately wrong example fails `cargo xtask ci`; every format page is covered.

## [To Do] STOR-F20 Crash consistency testing
P0 · M2 · Feature · deps: STOR-F09, TEST-F03
Scope: Crash at every write point, torn writes, lost unsynced data; recovery invariants.

### [To Do] STOR-0229 Storage crash harness: run an operation script against the simulated disk, crash at every `Fs` operation index in turn, recover, and run the checkers
P0 · M2 · Task · deps: none · labels: test
Done when: Harness prints the script, the crash point and the seed; exhaustive mode covers scripts of up to 200 operations in CI time.

### [To Do] STOR-0230 Recovery invariant checkers: offsets contiguous from the log start, every acknowledged-and-flushed offset present, all batches pass CRC, indexes agree with the log, epoch cache entries below the log end, checkpoints parse
P0 · M2 · Task · deps: STOR-0229, TEST-F05 · labels: test
Done when: Each checker has a negative test proving it fails on a crafted violation.

### [To Do] STOR-0231 Crash class: torn or lost writes during append (partial batch at the tail, unsynced data beyond the recovery point)
P0 · M2 · Task · deps: STOR-0230 · labels: test
Done when: Every crash point recovers to a batch boundary at or after the recovery point; no partial batch is ever readable.

### [To Do] STOR-0232 Crash class: crash during offset or time index append, resize and trim (index ahead of or behind the log, zero-filled tail)
P0 · M2 · Task · deps: STOR-0230 · labels: test
Done when: Recovery rebuilds or truncates indexes so lookups agree with a full scan at every crash point.

### [To Do] STOR-0233 Crash class: crash during segment roll (new files created but directory entry unsynced, previous segment not yet trimmed)
P0 · M2 · Task · deps: STOR-0230 · labels: test
Done when: The log recovers with either the old active segment or both segments, never a missing range.

### [To Do] STOR-0234 Crash class: crash during checkpoint writes (temporary written, rename not synced) for every checkpoint file
P0 · M2 · Task · deps: STOR-0230 · labels: test
Done when: Each checkpoint reads as fully old or fully new; a stale recovery point only causes extra recovery work.

### [To Do] STOR-0235 Crash class: crash during retention (log start offset advanced in memory, `.deleted` renames partial, log-start checkpoint stale)
P0 · M2 · Task · deps: STOR-0230, STOR-F13 · labels: test
Done when: After recovery the log start offset is max(checkpoint, first segment base) and no deleted data reappears above it.

### [To Do] STOR-0236 Crash class: lost deferred leader-epoch checkpoint flush after truncation
P0 · M2 · Task · deps: STOR-0230, STOR-F11 · labels: test
Done when: The loader's truncate-from-end at the recovered log end removes stale epochs at every crash point.

### [To Do] STOR-0237 Crash class: crash during shutdown before or while writing the clean shutdown marker, and crash during loading after the marker is deleted
P0 · M2 · Task · deps: STOR-0230, STOR-F16 · labels: test
Done when: Every case leads to full recovery on the next start, never a skipped recovery with unflushed data.

### [To Do] STOR-0238 Crash class: crash during partition directory renames for deletion and stray handling
P0 · M2 · Task · deps: STOR-0230, STOR-F16 · labels: test
Done when: After restart the directory is either live or fully deleted on schedule; no partially deleted live log is loaded.

### [To Do] STOR-0239 Failed fsync semantics: a `sync_data` error is never retried; the log dir is failed and data after the recovery point is treated as lost
P0 · M2 · Task · deps: STOR-0230 · labels: test
Done when: Test injects an fsync error and checks the dir fails, the recovery point does not advance and the node halts when it is the only dir.

### [To Do] STOR-0240 Crash class: crash during log truncation (segments removed, floor segment truncated, checkpoints stale)
P0 · M3 · Task · deps: STOR-0230, STOR-F12 · labels: test
Done when: No truncated offset reappears after recovery at any crash point.

### [To Do] STOR-0241 Crash class: crash at every step of the `.cleaned` to `.swap` to final replacement and old-segment `.deleted` renames
P0 · M5 · Task · deps: STOR-0230, STOR-F14 · labels: test
Done when: Recovery always yields either the pre-clean or the post-clean segments for the range, and reads return the same latest value per key.

### [To Do] STOR-0242 Crash class: crash during producer snapshot write, deferred flush and snapshot deletion
P0 · M6 · Task · deps: STOR-0230, STOR-F15 · labels: test
Done when: Recovered producer state equals a replay of the recovered log at every crash point; corrupt snapshots are skipped.

### [To Do] STOR-0243 Randomized crash campaign: seeded random operation sequences (append, roll, retention, flush, restart) with random crash points
P0 · M2 · Task · deps: STOR-0231, STOR-0232, STOR-0233, STOR-0234, TEST-F04 · labels: test
Done when: CI runs a fixed seed set plus new seeds; nightly runs thousands; failures print the replay command with `ALLUVIUM_SIM_SEED`.

## [To Do] STOR-F21 Log directory failure handling
P0 · M7 · Feature · deps: STOR-F16, SRV-F11
Scope: Take a failed directory offline, report to the controller, keep serving other directories.

### [To Do] STOR-0244 Log dir failure signalling across all logs of a dir: the first I/O error raises `LogDirFailed { dir, cause }` once; later operations on that dir fail fast with `KAFKA_STORAGE_ERROR`
P0 · M7 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogDirFailureChannel.java @4.4.0; storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java maybeHandleIOException @4.4.0
Done when: Test with two dirs fails one and checks a single failure action and fast failures only on that dir.

### [To Do] STOR-0245 Log manager handling of a failed dir: remove it from live dirs, close its logs quietly, drop its cleaner checkpoints and stop cleaning there, halt the node when no live dir remains
P0 · M7 · Task · deps: STOR-0244 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java handleLogDirFailure, removeOfflineLogs @4.4.0
Done when: Test fails one of two dirs and keeps serving the other; failing the last dir halts with an error-level event.

### [To Do] STOR-0246 Startup with offline dirs: dirs that fail validation or locking start offline, and an I/O error while loading a log fails its dir while other dirs continue loading
P0 · M7 · Task · deps: STOR-0244 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java loadLogs, handleIOException @4.4.0
Done when: Test injects a load error in one dir and the node starts with the other dir.

### [To Do] STOR-0247 Report offline dirs to the controller through the broker lifecycle path (directory ids in heartbeats and directory events)
P0 · M7 · Task · deps: STOR-0245, CTRL-F20 · ref: server-common/src/main/java/org/apache/kafka/server/common/DirectoryEventHandler.java @4.4.0
Done when: Integration test shows the controller marks replicas on the failed dir offline and elects new leaders.

### [To Do] STOR-0248 Cordoned log dirs (`cordoned.log.dirs`, gated by metadata.version): no new partitions placed on cordoned dirs, existing ones keep serving
P1 · M7 · Task · deps: STOR-0245 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java updateCordonedLogDirs, cordonedLogDirs, nextLogDirs @4.4.0; server-common/src/main/java/org/apache/kafka/server/common/DirectoryEventHandler.java @4.4.0
Done when: Tests show placement skips cordoned dirs and falls back to the first live dir when all are cordoned, as Kafka does; `CordonedLogDirectoryCount` updates.

### [To Do] STOR-0249 Simulation: injected I/O errors on one of several dirs under produce and fetch load
P0 · M7 · Task · deps: STOR-0245, STOR-0246, TEST-F03 · labels: test
Done when: No panic; partitions on healthy dirs keep serving; affected partitions return `KAFKA_STORAGE_ERROR`; the seed is printed on failure.

### [To Do] STOR-0250 `log_dir_failed` error-level event with dir, cause and affected partition count, plus a flight-recorder dump for affected partitions
P0 · M7 · Task · deps: STOR-0245, OBS-F07
Done when: Event listed in `events.rs` and asserted in a test.

## [To Do] STOR-F22 Moving logs between directories
P1 · M7 · Feature · deps: STOR-F21
Scope: Future logs, copy and swap for AlterReplicaLogDirs.

### [To Do] STOR-0251 Future log creation for AlterReplicaLogDirs: a `-future` directory in the target dir, at most one per partition, recorded as the preferred dir
P1 · M7 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java getOrCreateLog, maybeUpdatePreferredLogDir @4.4.0
Done when: Creating a second future for the same partition in another dir replaces the preference as Kafka does; tests over two `MemFs` dirs.

### [To Do] STOR-0252 Copy driver: replicate from the current log into the future log with local reads and follower appends until caught up, throttled by `replica.alter.log.dirs.io.max.bytes.per.second`
P1 · M7 · Task · deps: STOR-0251, REPL-F06 · ref: core/src/main/scala/kafka/server/ReplicaAlterLogDirsThread.scala @4.4.0
Done when: The future log converges to the current log under concurrent produce; throttle verified with the simulated clock.

### [To Do] STOR-0253 Keep the future log aligned when the current log truncates
P1 · M7 · Task · deps: STOR-0251 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java truncateTo, truncateFullyAndStartAt @4.4.0
Done when: Test truncates the current log mid-copy and the future log follows.

### [To Do] STOR-0254 Swap: pause cleaning, rename the current dir to `-delete`, rename the future dir to its final name, swap maps, carry the high watermark, resume cleaning, delete the old log asynchronously
P1 · M7 · Task · deps: STOR-0252 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java replaceCurrentWithFutureLog @4.4.0
Done when: Reads and appends continue across the swap with no offset gap (test).

### [To Do] STOR-0255 Recover abandoned future logs at startup: promote a future log whose dir matches the metadata assignment, otherwise delete it
P1 · M7 · Task · deps: STOR-0251 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java recoverAbandonedFutureLogs, findAbandonedFutureLogs @4.4.0
Done when: Tests cover both outcomes after a crash mid-move.

### [To Do] STOR-0256 Move the cleaner checkpoint entry with the log
P1 · M7 · Task · deps: STOR-0254, STOR-F14 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleanerManager.java alterCheckpointDir @4.4.0
Done when: Compaction resumes from the same dirty offset after the move (test).

### [To Do] STOR-0257 Crash tests for every rename in the swap sequence
P1 · M7 · Task · deps: STOR-0254, STOR-F20 · labels: test
Done when: After restart exactly one authoritative log exists for the partition with no data loss.

### [To Do] STOR-0258 Directory move events (`log_dir_move_started`, `log_dir_move_completed` with bytes and `elapsed_ms`) and future-log lag reporting
P2 · M7 · Task · deps: STOR-0254, OBS-F07
Done when: Events listed in `events.rs`; lag exposed for DescribeLogDirs `isFuture` replicas.

## [To Do] STOR-F23 Storage performance
P2 · M2 · Feature · deps: STOR-F08, TEST-F11
Scope: Read-path caching, file-region reads, benchmarks for append and fetch.

### [To Do] STOR-0259 Append benchmark: 1 KiB to 1 MiB batches, with and without flush, over `StdFs`
P2 · M2 · Task · deps: none · labels: perf
Done when: Criterion benchmark in CI with a regression threshold; results compared with NFR-F01 targets in the PR.

### [To Do] STOR-0260 Fetch benchmark: tail reads at the log end and cold reads from old segments (index lookup plus slice)
P2 · M2 · Task · deps: none · labels: perf
Done when: Criterion benchmark with regression threshold; reports lookups per second and bytes per second.

### [To Do] STOR-0261 Recovery and load benchmark: segments recovered per second and time to load 10,000 partitions after clean and unclean shutdown
P2 · M2 · Task · deps: none · labels: perf
Done when: Results feed NFR-F06; regression threshold in the nightly job.

### [To Do] STOR-0262 Zero-copy reads: fetch results carry file regions that the network layer sends with `sendfile` on plaintext connections
P2 · M2 · Task · deps: none · labels: perf · ref: clients/src/main/java/org/apache/kafka/common/record/internal/FileRecords.java writeTo @4.4.0
Done when: A test proves no user-space copy of record bytes on the plaintext path (counting buffer allocations); TLS falls back to buffered reads.

### [To Do] STOR-0263 Tail-read fast path: reads at or near the high watermark use cached offset metadata (segment and position) and skip index lookups
P2 · M2 · Task · deps: STOR-0260 · labels: perf · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LocalLog.java read @4.4.0
Done when: Benchmark shows a measurable gain for tail reads; results identical to the slow path in a property test.

### [To Do] STOR-0264 Allocation audit of the append path for uncompressed v2 batches
P2 · M2 · Task · deps: STOR-0259 · labels: perf, nfr
Done when: A heap-profiling test shows zero per-batch heap allocations after warm-up; regressions fail CI.

### [To Do] STOR-0265 Publish the storage performance budget (append, fetch, recovery) against the NFR targets
P1 · M2 · Task · deps: STOR-0259, STOR-0260, STOR-0261, NFR-F01 · labels: docs, nfr
Done when: docs/nfr.md storage section lists measured values, hardware and commands to reproduce.

## [To Do] STOR-F24 Append observer hook
P2 · M2 · Feature · deps: STOR-F08 · labels: foundation-for:X-KV
Scope: Hook invoked per committed batch so secondary indexes can be maintained.

### [To Do] STOR-0266 Append observer contract: observers receive committed offset ranges (below the high watermark) in order, plus truncation and log-start notifications; delivery resumes from a persisted observer position after restart
P2 · M2 · Task · deps: none · labels: foundation-for:X-KV
Done when: Contract documented in the module doc and the X-KV design spike can reference it; a type-level API with no observer registered by default.

### [To Do] STOR-0267 Emit committed-range notifications from high-watermark advancement as sans-I/O actions
P2 · M2 · Task · deps: STOR-0266 · labels: foundation-for:X-KV · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogOffsetsListener.java onHighWatermarkUpdated @4.4.0
Done when: Each high watermark increase yields one range [old, new); no notification for records that are appended but not yet committed (test).

### [To Do] STOR-0268 Notify observers of truncation, log start offset increases and full resets so derived indexes can drop entries
P2 · M2 · Task · deps: STOR-0266 · labels: foundation-for:X-KV
Done when: Property test: an observer-maintained set of offsets always equals the readable committed range.

### [To Do] STOR-0269 Notify observers when compaction replaces segments, with the affected offset range
P2 · M5 · Task · deps: STOR-0266, STOR-F14 · labels: foundation-for:X-KV
Done when: Test shows an observer is told about every replaced range exactly once.

### [To Do] STOR-0270 Bounded observer delivery: a full observer queue never blocks appends; the observer is marked lagging and later resumes by re-reading from its position
P2 · M2 · Task · deps: STOR-0267 · labels: foundation-for:X-KV, nfr
Done when: Test with a stalled observer shows append latency unchanged and the observer catching up afterwards.

### [To Do] STOR-0271 Prove zero cost when no observer is registered
P2 · M2 · Task · deps: STOR-0267 · labels: foundation-for:X-KV, perf
Done when: Append benchmark with and without the hook compiled in shows no regression beyond noise.

### [To Do] STOR-0272 Property test: under random append, high watermark, truncate and retention sequences an observer sees every committed offset exactly once and in order
P2 · M2 · Task · deps: STOR-0268, STOR-0270 · labels: test, foundation-for:X-KV
Done when: Test prints its seed on failure.

## [To Do] STOR-F25 Disk-full protection
P1 · M2 · Feature · deps: STOR-F16
Scope: Reject writes with a clear error before a directory fills; reserve space for recovery and metadata.

### [To Do] STOR-0273 Spike: disk-full protection ADR: thresholds, reserved space, which writes are rejected, the Kafka error code returned to clients (deviation from Kafka, which fails the log dir), and whether it is on by default
P1 · M2 · Spike · deps: none
Done when: ADR accepted; docs/parity.md deviation row added; config keys requested from CFG-F11.

### [To Do] STOR-0274 Free-space query on the `Fs` trait (`available_bytes(dir)`): `StdFs` via a safe statvfs wrapper with dependency justification, `MemFs` via its capacity
P1 · M2 · Task · deps: none
Done when: Works on Linux and macOS; dependency passes `cargo deny`; test with `MemFs` capacity.

### [To Do] STOR-0275 Per-log-dir free-space monitor: periodic sampling plus accounting of in-flight appends, states Ok, Low and Critical with hysteresis
P1 · M2 · Task · deps: STOR-0273, STOR-0274
Done when: Simulated-clock test walks through each state; transitions emit `log_dir_space_state_changed` with a reason.

### [To Do] STOR-0276 Write admission: client and replication appends are rejected below the threshold with the error chosen in the ADR; internal writes (checkpoints, recovery, clean shutdown marker, cleaner swaps) may use the reserve
P1 · M2 · Task · deps: STOR-0275
Done when: Test fills `MemFs` and shows appends rejected before `NoSpace` occurs and checkpoints still written.

### [To Do] STOR-0277 Reserved space for recovery and metadata so a node that hit the threshold can always restart and recover
P1 · M2 · Task · deps: STOR-0276
Done when: Restart test at Critical state completes recovery and writes the clean shutdown marker.

### [To Do] STOR-0278 Disk space observability: `alluvium_log_dir_free_bytes{dir}`, `alluvium_log_dir_space_state{dir}`, rejected appends counter
P1 · M2 · Task · deps: STOR-0275, OBS-F04
Done when: Metrics asserted in tests; docs/metrics.md rows added.

### [To Do] STOR-0279 Disk-full simulation and compatibility check: fill the disk under produce load; with ample space the feature changes nothing visible to clients
P1 · M2 · Task · deps: STOR-0276, STOR-0277, TEST-F09 · labels: test
Done when: Simulation passes with printed seeds; differential tests with Kafka 4.4.0 show no difference when the disk is not near full.
