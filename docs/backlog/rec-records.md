# REC: Record batches and compression

> Crates: `alluvium-records` (layer 0) · Kafka counterpart: `common/record`, `storage/…/LogValidator.java` · Milestones: M1-M5

Record batch v2 parsing, building, CRC32C, the four compression codecs with Kafka-compatible framing, bounded decompression, control batches, and append-time validation.

## [To Do] REC-F01 Batch header parsing
P0 · M1 · Feature · deps: TYPE-F02, PROTO-F03
Scope: Parse and validate the record batch v2 header: base offset, length, leader epoch, magic, CRC, attributes, last offset delta, timestamps, producer fields, record count.

### [To Do] REC-0001 Spike: allow `alluvium-records` to depend on `alluvium-protocol` (both layer 0) for wire primitives and generated control-record schemas
P0 · M1 · Spike · deps: none · ref: ADR-0009
Done when: an ADR is accepted that either adds the allow-list entry (with ADR reference) to `xtask check-layers` or chooses another split; docs/architecture.md §5 reflects the decision.

### [To Do] REC-0002 Record batch v2 layout constants and a zero-copy `RecordBatch` view over `Bytes` (field offsets 0–61, RECORD_BATCH_OVERHEAD = 61, LOG_OVERHEAD = 12)
P0 · M1 · Task · deps: REC-0001 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java field offset constants @4.4.0
Done when: accessors read big-endian fields on demand without copying and return TYPE newtypes (`Offset`, `LeaderEpoch`, `ProducerId`, `ProducerEpoch`, `Sequence`); unit tests on a hand-built header; module docs cite DefaultRecordBatch.

### [To Do] REC-0003 Batch framing over a records buffer: size < 14 or > max size → CorruptRecord, magic outside 0–2 → CorruptRecord, a truncated trailing batch returned as a partial tail
P0 · M1 · Task · deps: REC-0002 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/ByteBufferLogInputStream.java nextBatch/nextBatchSize @4.4.0
Done when: messages match ByteBufferLogInputStream; the partial tail is exposed (fetch responses may end mid-batch); property: concatenated valid batches iterate back exactly.

### [To Do] REC-0004 Magic handling: only v2 batches are parsed; v0 and v1 yield `UnsupportedMagic { magic, base_offset }`
P0 · M1 · Task · deps: REC-0003 · labels: decoder · ref: ADR-0007, KIP-724
Done when: a v1 batch is rejected without reading past its header; the error is documented as the ADR-0007/KIP-724 boundary.

### [To Do] REC-0005 Minimum batch size: a v2 batch smaller than 61 bytes is CorruptRecord
P0 · M1 · Task · deps: REC-0003 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java ensureValid @4.4.0
Done when: message equals Kafka's "Record batch is corrupt (the size … is smaller than the minimum allowed overhead 61)"; boundary tests at 60 and 61.

### [To Do] REC-0006 Attribute bits 0–2: compression codec id (0 none, 1 gzip, 2 snappy, 3 lz4, 4 zstd); ids 5–7 are `UnknownCompressionCodec`; bits 7–15 ignored and preserved
P0 · M1 · Task · deps: REC-0002, TYPE-F12 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java COMPRESSION_CODEC_MASK; clients/src/main/java/org/apache/kafka/common/record/internal/CompressionType.java forId @4.4.0
Done when: each id maps to TYPE-F12 `CompressionType`; unknown ids return the typed error and the Kafka error code for it is pinned by a differential test against Kafka 4.4.0 Produce; unused bits survive rewrites byte for byte.

### [To Do] REC-0007 Attribute bit 3: timestamp type (CreateTime or LogAppendTime); for LogAppendTime batches every record's timestamp is the batch max timestamp
P0 · M1 · Task · deps: REC-0002 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java timestampType, RecordIterator @4.4.0
Done when: accessor returns TYPE-F12 `TimestampType`; record iteration applies the LogAppendTime rule (test).

### [To Do] REC-0008 Attribute bit 4: transactional flag
P0 · M1 · Task · deps: REC-0002 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java TRANSACTIONAL_FLAG_MASK @4.4.0
Done when: `is_transactional` reads bit 0x10; tests with the bit alone and combined with control.

### [To Do] REC-0009 Attribute bit 5: control flag
P0 · M1 · Task · deps: REC-0002 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java CONTROL_FLAG_MASK @4.4.0
Done when: `is_control_batch` reads bit 0x20; tests.

### [To Do] REC-0010 Attribute bit 6: delete horizon flag; when set, `delete_horizon_ms` is the base timestamp
P0 · M1 · Task · deps: REC-0002 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java DELETE_HORIZON_FLAG_MASK, deleteHorizonMs @4.4.0
Done when: `delete_horizon_ms` returns None without the flag and the base timestamp with it (test).

### [To Do] REC-0011 Offset fields: base offset, last offset delta, last and next offset with checked arithmetic, record count (negative → InvalidRecord)
P0 · M1 · Task · deps: REC-0002, TYPE-F02 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java lastOffset, RecordIterator @4.4.0
Done when: overflowing `base + delta` is a typed error instead of a panic; a negative count fails with Kafka's "Found invalid record count" message.

### [To Do] REC-0012 Producer fields: producer id (−1 none), producer epoch, base sequence (−1 none), last sequence with Kafka's wraparound at i32::MAX
P0 · M1 · Task · deps: REC-0002 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java incrementSequence/lastSequence @4.4.0
Done when: `last_sequence` and `increment_sequence` match Kafka's formula in a property test; `has_producer_id` is true iff producer id ≥ 0.

### [To Do] REC-0013 Base timestamp, max timestamp and partition leader epoch fields with sentinels NO_TIMESTAMP (−1) and NO_PARTITION_LEADER_EPOCH (−1)
P0 · M1 · Task · deps: REC-0002 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/RecordBatch.java NO_TIMESTAMP, NO_PARTITION_LEADER_EPOCH @4.4.0
Done when: accessors return typed optional values; tests for sentinels.

### [To Do] REC-0014 Batch `Debug`/`Display` summary (offsets, count, codec, timestamp type, producer fields, CRC) without payload bytes
P0 · M1 · Task · deps: REC-0011, REC-0012, REC-0006, TYPE-F09 · labels: security
Done when: output contains no record bytes (test); format used by dump tools and logs.

### [To Do] REC-0015 Property tests: arbitrary header values encode → parse identically; corrupted length or magic produces the documented errors
P0 · M1 · Task · deps: REC-0004, REC-0005, REC-0006, REC-0008, REC-0009, REC-0010, REC-0012, REC-0011, REC-0013, TEST-F16 · labels: test
Done when: proptest strategies from TEST-F16 cover every attribute combination and sentinel; regressions persisted.

### [To Do] REC-0016 `docs/formats/record-batch-v2.md`: byte layout, attribute bits, sentinel values and an annotated hex example
P1 · M1 · Task · deps: REC-0015 · labels: docs
Done when: a reader can decode the example header by hand from the page.

## [To Do] REC-F02 Record parsing
P0 · M1 · Feature · deps: REC-F01
Scope: Lazy iteration over records: varint lengths, attributes, timestamp and offset deltas, key, value, headers.

### [To Do] REC-0017 `Record` view: offset, timestamp, sequence, key and value as `Option<Bytes>` slices, lazily parsed headers, size in bytes
P0 · M1 · Task · deps: none · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java @4.4.0
Done when: fields are slices of the batch buffer (pointer-range test); module docs cite DefaultRecord.

### [To Do] REC-0018 Record length varint: negative → InvalidRecord, larger than remaining → InvalidRecord, body consumed exactly
P0 · M1 · Task · deps: REC-0017, PROTO-F03 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java readFrom @4.4.0
Done when: messages match Kafka's ("Invalid record size: … is negative", "expected … bytes in record payload, but instead the buffer has only …", "expected to read … but instead read …").

### [To Do] REC-0019 Record attributes byte read and preserved (unused in v2)
P0 · M1 · Task · deps: REC-0018 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java readFrom @4.4.0
Done when: any attribute value parses and is kept for re-encoding; test with non-zero attributes.

### [To Do] REC-0020 Timestamp delta varlong: timestamp = base timestamp + delta with Java wrapping semantics; LogAppendTime batches use the max timestamp
P0 · M1 · Task · deps: REC-0019, REC-0007 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java readFrom @4.4.0
Done when: results match Kafka for extreme base timestamps and deltas (wrapping test); LogAppendTime override test.

### [To Do] REC-0021 Offset delta varint: offset = base offset + delta; sequence = increment_sequence(base sequence, delta) when base sequence ≥ 0
P0 · M1 · Task · deps: REC-0020, REC-0012 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java readFrom; clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java incrementSequence @4.4.0
Done when: sequence wraparound case and NO_SEQUENCE case tested.

### [To Do] REC-0022 Key: varint length, any negative length is null, slice without copy
P0 · M1 · Task · deps: REC-0021 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java readFrom; clients/src/main/java/org/apache/kafka/common/utils/Utils.java readBytes @4.4.0
Done when: −1 and −5 both decode to None as in Kafka's Utils.readBytes; length beyond the body is InvalidRecord ("Found invalid record structure").

### [To Do] REC-0023 Value: varint length, any negative length is null (tombstone), slice without copy
P0 · M1 · Task · deps: REC-0022 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java readFrom @4.4.0
Done when: null and empty values are distinct; tombstone test.

### [To Do] REC-0024 Header count varint: negative → InvalidRecord, larger than remaining bytes → InvalidRecord
P0 · M1 · Task · deps: REC-0023 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java readFrom @4.4.0
Done when: messages match Kafka's "Found invalid number of record headers" variants.

### [To Do] REC-0025 Headers: key length varint (negative → InvalidRecord), key bytes, value length (negative → null); key decoded lazily as UTF-8 with Java replacement semantics
P0 · M1 · Task · deps: REC-0024 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java readHeaders @4.4.0
Done when: "Invalid negative header key size" message; null header values preserved; invalid UTF-8 keys decode with the PROTO-F03 string decoder, matching Java.

### [To Do] REC-0026 Lazy iterator over an uncompressed batch yielding `Result<Record>`: stops after `count` records, errors on leftover bytes or premature end
P0 · M1 · Task · deps: REC-0025, REC-0011 · labels: decoder, perf · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java RecordIterator/uncompressedIterator @4.4.0
Done when: messages "Incorrect declared batch size, records still remaining in file" and "premature EOF reached"; no heap allocation per record (counting-allocator test).

### [To Do] REC-0027 Skip-key-value iteration reading only offset, timestamp and size, for validation hot paths
P0 · M1 · Task · deps: REC-0026 · labels: decoder, perf · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java readPartiallyFrom; clients/src/main/java/org/apache/kafka/common/record/internal/PartialDefaultRecord.java @4.4.0
Done when: skipped key, value and header bytes are still bounds-checked; results equal the full iterator's offsets and timestamps (property).

### [To Do] REC-0028 Record `Debug` redaction: key and value as `PayloadSummary`, headers as a count only
P0 · M1 · Task · deps: REC-0025, TYPE-F09 · labels: security
Done when: no key, value or header bytes appear in Debug output (test).

### [To Do] REC-0029 Property tests: arbitrary records (null, empty and large keys and values, 0–N headers, extreme deltas) encode → parse identically
P0 · M1 · Task · deps: REC-0026, TEST-F16 · labels: test
Done when: uses TEST-F16 strategies and a test-only encoder; regressions persisted.

### [To Do] REC-0030 Record and header layout section in `docs/formats/record-batch-v2.md`
P1 · M1 · Task · deps: REC-0029, REC-0016 · labels: docs
Done when: the page shows an annotated record with two headers.

## [To Do] REC-F03 CRC32C
P0 · M1 · Feature · deps: REC-F01
Scope: Compute and validate CRC32C over the batch body, hardware-accelerated where available.

### [To Do] REC-0031 Adopt the `crc32c` crate (SSE4.2 and ARMv8 CRC instructions with software fallback) through the dependency checklist
P0 · M1 · Task · deps: none · labels: security · ref: docs/standards/security.md#dependencies
Done when: workspace dependency with minimal features; `cargo deny check` passes; the PR answers the five dependency questions, including its `unsafe` use.

### [To Do] REC-0032 CRC32C compute and incremental append API in alluvium-records
P0 · M1 · Task · deps: REC-0031 · ref: clients/src/main/java/org/apache/kafka/common/utils/internals/Crc32C.java @4.4.0
Done when: known vectors (RFC 3720 and Kafka's Crc32CTest) pass; incremental equals one-shot (property).

### [To Do] REC-0033 Test-only table-driven CRC32C reference and differential property test against the accelerated path
P0 · M1 · Task · deps: REC-0032 · labels: test
Done when: random lengths 0–64 KiB at unaligned starts agree; CI runs it on x86_64 and aarch64.

### [To Do] REC-0034 Batch CRC validation over bytes from the attributes field (offset 21) to the end: `is_valid` and `ensure_valid` → `CorruptRecord { base_offset, stored, computed }`
P0 · M1 · Task · deps: REC-0032, REC-0002 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java computeChecksum/isValid/ensureValid @4.4.0
Done when: the stored CRC is read as u32; changing base offset, length or partition leader epoch keeps the batch valid; flipping any bit from offset 21 on is detected (property).

### [To Do] REC-0035 CRC write helper used by builders and header rewrites
P0 · M1 · Task · deps: REC-0034 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java writeHeader @4.4.0
Done when: writing then validating is always valid (property).

## [To Do] REC-F04 Batch builder
P0 · M1 · Feature · deps: REC-F02, REC-F03
Scope: Build batches (like `MemoryRecordsBuilder`) for tests, control records and re-compression.

### [To Do] REC-0036 `BatchBuilder` configuration (base offset, timestamp type, compression, producer id/epoch/base sequence, transactional, control, partition leader epoch, delete horizon) with Kafka's argument checks
P0 · M1 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecordsBuilder.java MemoryRecordsBuilder, appendDefaultHeader checks @4.4.0
Done when: transactional without a producer id, or a producer id with negative epoch or sequence, is a typed error; tests per rule.

### [To Do] REC-0037 Append with offset: offsets strictly increasing, timestamp ≥ 0 or −1, control records only in control batches, offset delta and count within i32
P0 · M1 · Task · deps: REC-0036 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecordsBuilder.java appendWithOffset @4.4.0
Done when: errors mirror Kafka's ("Illegal offset … following previous offset …", "Invalid negative timestamp", "Maximum offset delta exceeded"); tests per rule.

### [To Do] REC-0038 Record encoding: body size varint, attributes 0, timestamp and offset deltas, key, value, headers with non-null keys
P0 · M1 · Task · deps: REC-0037, REC-0026 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java writeTo/sizeInBytes @4.4.0
Done when: `size_of` equals bytes written (property); output parses with REC-F02.

### [To Do] REC-0039 Build: write the batch header (last offset delta, base timestamp or delete horizon, max timestamp or log-append time, producer fields, count) and CRC
P0 · M1 · Task · deps: REC-0038, REC-0035 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecordsBuilder.java build/writeDefaultBatchHeader @4.4.0
Done when: an empty builder produces no batch; LogAppendTime mode sets max timestamp to the append time; built batches pass `ensure_valid` and parse back.

### [To Do] REC-0040 In-place header rewrite for offset assignment: set last offset (rewrites base offset), set partition leader epoch (not CRC-covered), set max timestamp and timestamp type with CRC recompute only when they change
P0 · M1 · Task · deps: REC-0035 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java setLastOffset/setPartitionLeaderEpoch/setMaxTimestamp @4.4.0
Done when: operates on `BytesMut` (copy-on-write from `Bytes` documented); the CRC is valid after every rewrite; an unchanged timestamp leaves the CRC bytes untouched (test).

### [To Do] REC-0041 Compression stage: records encoded into an uncompressed buffer and compressed at build by the configured codec
P0 · M1 · Task · deps: REC-0039
Done when: codec dispatch is an enum (no trait objects); with `none` the payload is written directly; codec features plug in without changing callers.

### [To Do] REC-0042 Write limit with `has_room_for` and `is_full` based on estimated size, for Raft batch accumulation
P1 · M1 · Task · deps: REC-0039 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecordsBuilder.java hasRoomFor/isFull; raft/src/main/java/org/apache/kafka/raft/internals/BatchAccumulator.java @4.4.0
Done when: a builder never exceeds its write limit except for a single oversized first record (Kafka behavior); tests.

### [To Do] REC-0043 Test-support constructors (`with_records`, `with_idempotent_records`, `with_transactional_records`) behind a `test-util` feature
P1 · M1 · Task · deps: REC-0039 · labels: test · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecords.java withRecords @4.4.0
Done when: helpers are used by REC tests and documented for other crates' tests.

### [To Do] REC-0044 Property tests: builder output parses back to the appended records for arbitrary inputs and configurations
P0 · M1 · Task · deps: REC-0039, TEST-F16 · labels: test
Done when: uses TEST-F16 strategies; regressions persisted.

### [To Do] REC-0045 Empty batch writer (record count 0, producer id/epoch and sequence retained) for compaction
P1 · M5 · Task · deps: REC-0039 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java writeEmptyHeader @4.4.0
Done when: output matches Kafka's writeEmptyHeader bytes for a golden case; parses as an empty batch.

### [To Do] REC-0046 Batch filter-and-rewrite primitive for compaction: keep or drop records, keep empty batches when required, set the delete horizon on batches with tombstones or empty-transaction markers
P1 · M5 · Task · deps: REC-0045 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecords.java filterTo @4.4.0
Done when: behaves like MemoryRecords.filterTo for its BatchRetention outcomes (DELETE, RETAIN_EMPTY, keep original bytes when nothing changes); tests ported from MemoryRecordsTest.

## [To Do] REC-F05 gzip codec
P0 · M1 · Feature · deps: REC-F04
Scope: gzip compress/decompress compatible with Kafka's GZIP framing.

### [To Do] REC-0047 Spike: compression library per codec (flate2 backend, snap, lz4_flex vs an lz4 binding with HC levels, zstd binding vs ruzstd) weighing unsafe/FFI, KIP-390 level coverage, performance and licenses
P0 · M1 · Spike · deps: none · labels: security · ref: KIP-390; docs/standards/security.md#dependencies
Done when: an ADR is accepted naming the crate and features per codec and any level deviation; deny.toml updated.

### [To Do] REC-0048 gzip decompression of RFC 1952 members as written by Java's GZIPOutputStream and librdkafka, verifying the CRC32 and ISIZE trailer
P0 · M1 · Task · deps: REC-0047 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/compress/GzipCompression.java wrapForInput @4.4.0
Done when: output is produced in chunks (no whole-batch buffer); FEXTRA, FNAME, FCOMMENT and FHCRC headers are handled; a bad trailer is `DecompressionError::Gzip`.

### [To Do] REC-0049 gzip concatenated members and trailing bytes handled like java.util.zip.GZIPInputStream
P0 · M1 · Task · deps: REC-0048 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/compress/GzipCompression.java wrapForInput @4.4.0
Done when: two concatenated members decode fully; trailing non-gzip bytes are ignored; a truncated second member header is ignored as Java does (tests).

### [To Do] REC-0050 gzip compression with a level parameter (−1 → zlib default)
P0 · M1 · Task · deps: REC-0047, REC-0041 · ref: clients/src/main/java/org/apache/kafka/common/compress/GzipCompression.java wrapForOutput; clients/src/main/java/org/apache/kafka/common/compress/GzipOutputStream.java @4.4.0
Done when: round-trip property test; output is a single valid gzip member.

### [To Do] REC-0051 Interop: decode gzip batches produced by the Java client 4.4.0 and librdkafka
P0 · M1 · Task · deps: REC-0049, REC-0132, REC-0133 · labels: test
Done when: every gzip batch in the golden corpus decodes to the expected records.

### [To Do] REC-0052 Interop: Kafka 4.4.0 classes decode gzip batches built by Alluvium
P0 · M1 · Task · deps: REC-0050, REC-0136 · labels: test
Done when: the harness verify mode reads Alluvium gzip batches at default and boundary levels with identical records.

## [To Do] REC-F06 snappy codec
P0 · M1 · Feature · deps: REC-F04
Scope: Snappy with Kafka's xerial-framed format.

### [To Do] REC-0053 Xerial snappy stream decoding: magic `82 53 4E 41 50 50 59 00`, version and compatible-version ints, then (int32 BE length, raw snappy block) chunks
P0 · M1 · Task · deps: REC-0047 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/compress/SnappyCompression.java wrapForInput (snappy-java SnappyInputStream) @4.4.0
Done when: chunk lengths are bounded by remaining bytes; an incompatible version is an error; output in chunks.

### [To Do] REC-0054 Raw (unframed) snappy fallback when the xerial magic is absent, as snappy-java's SnappyInputStream does
P0 · M1 · Task · deps: REC-0053 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/compress/SnappyCompression.java wrapForInput @4.4.0
Done when: a raw-block batch decodes; which form librdkafka emits is recorded from the golden corpus.

### [To Do] REC-0055 Xerial snappy encoding with snappy-java defaults (header version 1, compatible version 1, 32 KiB blocks)
P0 · M1 · Task · deps: REC-0047, REC-0041 · ref: clients/src/main/java/org/apache/kafka/common/compress/SnappyCompression.java wrapForOutput @4.4.0
Done when: round-trip property test; header bytes equal snappy-java's.

### [To Do] REC-0056 Interop: decode snappy batches produced by the Java client 4.4.0 and librdkafka
P0 · M1 · Task · deps: REC-0054, REC-0132, REC-0133 · labels: test
Done when: every snappy batch in the golden corpus decodes to the expected records.

### [To Do] REC-0057 Interop: Kafka 4.4.0 classes decode snappy batches built by Alluvium
P0 · M1 · Task · deps: REC-0055, REC-0136 · labels: test
Done when: harness verify mode reads Alluvium snappy batches with identical records.

## [To Do] REC-F07 lz4 codec
P0 · M1 · Feature · deps: REC-F04
Scope: LZ4 with Kafka's frame format and its historical checksum quirks.

### [To Do] REC-0058 Kafka LZ4 frame header: magic 0x184D2204, FLG (version 01, reserved bits 0, independent blocks required), BD (block size id 4–7), optional 8-byte content size skipped, HC = (xxhash32(descriptor) >> 8) & 0xFF
P0 · M1 · Task · deps: REC-0047 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/compress/Lz4BlockInputStream.java readHeader; clients/src/main/java/org/apache/kafka/common/compress/Lz4BlockOutputStream.java FLG/BD @4.4.0
Done when: each violation returns `DecompressionError::Lz4` with Kafka's message ("Stream unsupported (invalid magic bytes)", "Stream frame descriptor corrupted", "Dependent block stream is unsupported", "Block size value must be between 4 and 7", reserved-bit errors); the magic-v0 broken-checksum mode is not implemented (ADR-0007) and documented.

### [To Do] REC-0059 LZ4 block decoding: little-endian size with the high-bit uncompressed flag, size ≤ the BD maximum, optional xxhash32 block checksum
P0 · M1 · Task · deps: REC-0058 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/compress/Lz4BlockInputStream.java readBlock @4.4.0
Done when: "Block size … exceeded max", "Block checksum mismatch" and premature-end errors tested; uncompressed blocks are slices without copy.

### [To Do] REC-0060 LZ4 end mark: content checksum read but not verified and bytes after the end mark ignored, as in Kafka
P0 · M1 · Task · deps: REC-0059 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/compress/Lz4BlockInputStream.java readBlock @4.4.0
Done when: tests with a wrong content checksum and trailing garbage both decode; the parity note is in docs/parity.md.

### [To Do] REC-0061 Kafka LZ4 frame encoding: FLG version 1 with independent blocks and no checksums, BD 64 KiB, correct HC, incompressible blocks stored raw, zero end mark
P0 · M1 · Task · deps: REC-0047, REC-0041 · ref: clients/src/main/java/org/apache/kafka/common/compress/Lz4BlockOutputStream.java @4.4.0
Done when: header bytes equal Kafka's Lz4BlockOutputStream output; round-trip property test.

### [To Do] REC-0062 Interop: decode lz4 batches produced by the Java client 4.4.0 and librdkafka
P0 · M1 · Task · deps: REC-0060, REC-0132, REC-0133 · labels: test
Done when: every lz4 batch in the golden corpus decodes, including any content-size or block-checksum flags librdkafka sets.

### [To Do] REC-0063 Interop: Kafka 4.4.0 classes decode lz4 batches built by Alluvium
P0 · M1 · Task · deps: REC-0061, REC-0136 · labels: test
Done when: harness verify mode reads Alluvium lz4 batches with identical records.

## [To Do] REC-F08 zstd codec
P0 · M1 · Feature · deps: REC-F04
Scope: Zstandard compress/decompress.

### [To Do] REC-0064 zstd streaming decompression with window-log limit 27 (libzstd's default as used by zstd-jni) and chunked output
P0 · M1 · Task · deps: REC-0047 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/compress/ZstdCompression.java wrapForInput/wrapForZstdInput @4.4.0
Done when: frames needing a larger window are rejected with `DecompressionError::Zstd`; output in 16 KiB chunks.

### [To Do] REC-0065 Concatenated and skippable zstd frames handled like zstd-jni's ZstdInputStream
P0 · M1 · Task · deps: REC-0064 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/compress/ZstdCompression.java wrapForInput @4.4.0
Done when: two concatenated frames decode fully; skippable frames are skipped; tests.

### [To Do] REC-0066 zstd compression with a level parameter (default 3)
P0 · M1 · Task · deps: REC-0047, REC-0041 · ref: clients/src/main/java/org/apache/kafka/common/compress/ZstdCompression.java wrapForOutput @4.4.0
Done when: round-trip property test; streaming output without content size like zstd-jni.

### [To Do] REC-0067 Interop: decode zstd batches produced by the Java client 4.4.0 and librdkafka
P0 · M1 · Task · deps: REC-0065, REC-0132, REC-0133 · labels: test
Done when: every zstd batch in the golden corpus decodes to the expected records.

### [To Do] REC-0068 Interop: Kafka 4.4.0 classes decode zstd batches built by Alluvium
P0 · M1 · Task · deps: REC-0066, REC-0136 · labels: test
Done when: harness verify mode reads Alluvium zstd batches with identical records.

## [To Do] REC-F09 Compression levels
P1 · M2 · Feature · deps: REC-F05, REC-F06, REC-F07, REC-F08, CFG-F05
Scope: `compression.gzip.level`, `compression.lz4.level`, `compression.zstd.level` (KIP-390) per topic.

### [To Do] REC-0069 `CompressionLevel` per codec with Kafka's ranges: gzip 1–9 or −1, lz4 1–17 (default 9), zstd −131072–22 (default 3)
P1 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/record/internal/CompressionType.java levelValidator; KIP-390
Done when: validation messages match CompressionType's level validators (for example "Value must be between 1 and 9 or equal to -1") so CFG can surface them; tests at every bound.

### [To Do] REC-0070 Apply `compression.gzip.level` when encoding gzip
P1 · M2 · Task · deps: REC-0069 · ref: clients/src/main/java/org/apache/kafka/common/compress/GzipCompression.java Builder.level @4.4.0
Done when: levels 1, 9 and −1 produce valid output; higher levels compress the standard corpus at least as well as lower ones (test).

### [To Do] REC-0071 Apply `compression.lz4.level`: level 9 uses the fast compressor and other levels the high-compression compressor, or the deviation chosen in the codec ADR
P1 · M2 · Task · deps: REC-0069 · ref: clients/src/main/java/org/apache/kafka/common/compress/Lz4BlockOutputStream.java Lz4BlockOutputStream @4.4.0
Done when: behavior per level matches Lz4BlockOutputStream's rule or the documented ADR deviation; tests at 1, 9 and 17.

### [To Do] REC-0072 Apply `compression.zstd.level`, including negative fast levels
P1 · M2 · Task · deps: REC-0069 · ref: clients/src/main/java/org/apache/kafka/common/compress/ZstdCompression.java Builder.level @4.4.0
Done when: levels −131072, −1, 3 and 22 produce valid output (tests).

### [To Do] REC-0073 Target compression resolution from topic `compression.type` (producer, uncompressed, gzip, snappy, lz4, zstd) and the level keys
P1 · M2 · Task · deps: REC-0070, REC-0071, REC-0072 · ref: server-common/src/main/java/org/apache/kafka/server/record/BrokerCompressionType.java targetCompression @4.4.0
Done when: `producer` keeps the source codec at its default level; a level difference alone never triggers recompression because LogValidator compares codecs only (test).

### [To Do] REC-0074 `docs/compression.md`: codecs, levels, defaults, and when the broker recompresses
P2 · M2 · Task · deps: REC-0073 · labels: docs
Done when: the page lists every level key with range and default and links the codec ADR.

## [To Do] REC-F10 Bounded decompression
P0 · M1 · Feature · deps: REC-F05, REC-F06, REC-F07, REC-F08
Scope: Cap decompressed size by limits and declared sizes; decompression-bomb tests per codec.

### [To Do] REC-0075 Spike: decompressed-size policy — Kafka streams batches and caps each record with `max.decompressed.message.bytes`; decide whether to add an opt-in total-per-batch cap and reconcile docs/standards/security.md rule 5
P0 · M1 · Spike · deps: none · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java MAX_DECOMPRESSED_MESSAGE_BYTES_CONFIG @4.4.0
Done when: an ADR is accepted; security.md matches it; any new `alluvium.*` key is handed to CFG-F11.

### [To Do] REC-0076 Streaming decompression reader over the four codecs with fixed chunk buffers (gzip 16 KiB, snappy 2 KiB, lz4 2 KiB, zstd 16 KiB) and `DecompressionError { codec, kind, base_offset }`
P0 · M1 · Task · deps: REC-0075, REC-0049, REC-0054, REC-0060, REC-0065 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/utils/internals/ChunkedBytesStream.java; GzipCompression.java, SnappyCompression.java, Lz4Compression.java, ZstdCompression.java decompressionOutputSize @4.4.0
Done when: memory per reader is bounded by its chunk size regardless of output size (counting-allocator test).

### [To Do] REC-0077 Record iteration over compressed batches: per-record body cap (`max.decompressed.message.bytes`) checked before allocation, premature-end and leftover-record errors
P0 · M1 · Task · deps: REC-0076, REC-0026 · labels: security, decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java readFrom(InputStream); clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java compressedIterator/StreamRecordIterator @4.4.0
Done when: messages match Kafka's ("exceeds the configured maximum record size", "Incorrect declared batch size"); the Kafka error code for decompression failures is pinned by a differential test against ReplicaManager's handling.

### [To Do] REC-0078 Skip-key-value iteration over compressed batches with the same caps
P0 · M1 · Task · deps: REC-0077, REC-0027 · labels: decoder, perf · ref: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecord.java readPartiallyFrom @4.4.0
Done when: results equal the full iterator's offsets and timestamps (property).

### [To Do] REC-0079 gzip bomb limits: output bounded by the per-record cap and the chunk budget
P0 · M1 · Task · deps: REC-0077 · labels: security, test
Done when: a 10 KiB gzip of 1 GiB of zeros wrapped as one record is rejected with peak memory under 1 MiB (test).

### [To Do] REC-0080 snappy bomb limits: each raw block's declared uncompressed length checked against the remaining budget before allocation
P0 · M1 · Task · deps: REC-0077 · labels: security, test
Done when: a block declaring 4 GiB fails before allocating; xerial chunks are limited the same way (tests).

### [To Do] REC-0081 lz4 bomb limits: block output bounded by the BD maximum; a declared content size never drives allocation
P0 · M1 · Task · deps: REC-0077 · labels: security, test
Done when: a frame declaring a huge content size and BD 4 MiB never allocates more than one block buffer (test).

### [To Do] REC-0082 zstd bomb limits: window-log cap and frame content size checked against the budget before decoding
P0 · M1 · Task · deps: REC-0077 · labels: security, test
Done when: a frame declaring 1 TiB content or windowLog 31 fails without large allocation (tests).

## [To Do] REC-F11 Control batches
P0 · M1 · Feature · deps: REC-F04
Scope: Transaction markers (COMMIT/ABORT), leader change messages, snapshot header/footer, KRaft version and voters records.

### [To Do] REC-0083 Control record key codec (ControlRecordTypeSchema version and type): fewer than 4 bytes → InvalidRecord, version < 0 → InvalidRecord, newer versions parsed as the highest known
P0 · M1 · Task · deps: PROTO-F05 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/ControlRecordType.java parseTypeId @4.4.0
Done when: messages match ControlRecordType.parseTypeId; uses the generated data type from PROTO-F05.

### [To Do] REC-0084 `ControlRecordType`: ABORT 0, COMMIT 1, LEADER_CHANGE 2, SNAPSHOT_HEADER 3, SNAPSHOT_FOOTER 4, KRAFT_VERSION 5, KRAFT_VOTERS 6, `Unknown(id)`
P0 · M1 · Task · deps: REC-0083 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/ControlRecordType.java fromTypeId @4.4.0
Done when: unknown ids are surfaced as `Unknown` rather than errors, as in Kafka; ControlRecordTypeTest ported.

### [To Do] REC-0085 EndTxnMarker value codec (version prefix, coordinator epoch): version < 0 → InvalidRecord, newer versions parsed as the highest, only COMMIT and ABORT keys accepted
P0 · M1 · Task · deps: REC-0084 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/record/internal/EndTransactionMarker.java deserializeValue @4.4.0
Done when: messages match EndTransactionMarker.deserializeValue; EndTransactionMarkerTest ported.

### [To Do] REC-0086 ABORT marker batches: build (control and transactional flags, producer id and epoch, base sequence −1, uncompressed) and recognize
P0 · M1 · Task · deps: REC-0085 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecords.java withEndTransactionMarker; clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecordsBuilder.java appendEndTxnMarker @4.4.0
Done when: a built ABORT batch equals Kafka's withEndTransactionMarker bytes for a golden case and parses back.

### [To Do] REC-0087 COMMIT marker batches: build and recognize
P0 · M1 · Task · deps: REC-0086 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecords.java withEndTransactionMarker @4.4.0
Done when: a built COMMIT batch equals Kafka's bytes for a golden case and parses back.

### [To Do] REC-0088 LEADER_CHANGE control records (LeaderChangeMessage: leader id, voters, granting voters); the builder requires a partition leader epoch ≥ 0
P0 · M1 · Task · deps: REC-0084 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecordsBuilder.java appendLeaderChangeMessage; clients/src/main/java/org/apache/kafka/common/record/internal/ControlRecordUtils.java deserializeLeaderChangeMessage @4.4.0
Done when: value decodes at ControlRecordUtils' current version; build-and-parse test; a negative epoch is rejected.

### [To Do] REC-0089 SNAPSHOT_HEADER control records (SnapshotHeaderRecord: version, last contained log timestamp)
P0 · M1 · Task · deps: REC-0084 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/ControlRecordUtils.java deserializeSnapshotHeaderRecord @4.4.0
Done when: build-and-parse test; golden bytes match Kafka's withSnapshotHeaderRecord.

### [To Do] REC-0090 SNAPSHOT_FOOTER control records (SnapshotFooterRecord: version)
P0 · M1 · Task · deps: REC-0084 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/ControlRecordUtils.java deserializeSnapshotFooterRecord @4.4.0
Done when: build-and-parse test; golden bytes match Kafka's withSnapshotFooterRecord.

### [To Do] REC-0091 KRAFT_VERSION control records (KRaftVersionRecord: version, kraft version)
P0 · M1 · Task · deps: REC-0084 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/ControlRecordUtils.java deserializeKRaftVersionRecord @4.4.0
Done when: build-and-parse test; golden bytes match Kafka's withKRaftVersionRecord.

### [To Do] REC-0092 KRAFT_VOTERS control records (VotersRecord: voter id, directory id, endpoints, kraft.version feature range)
P0 · M1 · Task · deps: REC-0084 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/ControlRecordUtils.java deserializeVotersRecord @4.4.0
Done when: build-and-parse test; golden bytes match Kafka's withVotersRecord.

### [To Do] REC-0093 Typed control record iteration: a `ControlRecord` enum parsed from each record's key and value in a control batch
P0 · M1 · Task · deps: REC-0087, REC-0088, REC-0089, REC-0090, REC-0091, REC-0092
Done when: every type, including Unknown, is yielded with its offset; malformed values return typed errors with the offset.

## [To Do] REC-F12 Append validation
P0 · M2 · Feature · deps: REC-F02, REC-F03, REC-F10, CFG-F05
Scope: Everything `LogValidator` does: magic v2 only, offset assignment, timestamp type and bounds, recompression, compacted-topic key requirement, size limits, producer-field consistency.

### [To Do] REC-0094 Validator inputs and outputs: plain `ValidationConfig` (timestamp type, before/after max ms, compacted, max message bytes, max decompressed message bytes, target compression, partition leader epoch, origin) and `ValidationOutcome`
P0 · M2 · Task · deps: none · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java LogValidator/ValidationResult; storage/src/main/java/org/apache/kafka/storage/internals/log/AppendOrigin.java @4.4.0
Done when: the config struct documents its source keys (`message.timestamp.type`, `message.timestamp.before.max.ms`, `message.timestamp.after.max.ms`, `cleanup.policy`, `max.message.bytes`, `max.decompressed.message.bytes`, `compression.type`, level keys) mapped by alluvium-config; `now_ms` is an input (no clock read); origin is Client, Replication, Coordinator or RaftLeader.

### [To Do] REC-0095 Produce records shape: at least one batch, magic 2, exactly one batch per partition, zstd only with Produce v7+ (UNSUPPORTED_COMPRESSION_TYPE)
P0 · M2 · Task · deps: REC-0094 · ref: clients/src/main/java/org/apache/kafka/common/requests/ProduceRequest.java validateRecords @4.4.0
Done when: messages match ProduceRequest.validateRecords; tests per rule and per Produce version boundary.

### [To Do] REC-0096 Pre-append: client v2 batches must have base offset 0; Raft-leader appends must carry the leader epoch
P0 · M2 · Task · deps: REC-0094 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java analyzeAndValidateRecords @4.4.0
Done when: messages match analyzeAndValidateRecords; INVALID_RECORD for clients; tests per origin.

### [To Do] REC-0097 Pre-append: batch size above `max.message.bytes` → MESSAGE_TOO_LARGE with a bytes-rejected signal
P0 · M2 · Task · deps: REC-0094 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java analyzeAndValidateRecords @4.4.0
Done when: boundary tests at max and max+1; the signal carries the full request size like Kafka's BytesRejectedPerSec.

### [To Do] REC-0098 Pre-append: invalid batch CRC → CORRUPT_MESSAGE with an invalid-CRC signal
P0 · M2 · Task · deps: REC-0094, REC-0034 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java analyzeAndValidateRecords @4.4.0
Done when: a single flipped payload bit is rejected; message matches Kafka's "Record is corrupt (stored crc = …)".

### [To Do] REC-0099 Pre-append summary: first and last offset, monotonic offsets (OffsetsOutOfOrder when required), max timestamp, valid bytes, source codec
P0 · M2 · Task · deps: REC-0094 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java analyzeAndValidateRecords; storage/src/main/java/org/apache/kafka/storage/internals/log/LogAppendInfo.java @4.4.0
Done when: follower appends with out-of-order offsets fail; the summary matches Kafka's LogAppendInfo fields for golden inputs.

### [To Do] REC-0100 Replication appends skip batches whose partition leader epoch is above the replica's current epoch (KAFKA-18723)
P0 · M2 · Task · deps: REC-0099 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java hasHigherPartitionLeaderEpoch @4.4.0
Done when: batches after the first higher-epoch batch are excluded from valid bytes; tests.

### [To Do] REC-0101 Rule: input with no batches is InvalidRecord ("Record batch has no batches at all")
P0 · M2 · Task · deps: REC-0094 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java getFirstBatchAndMaybeValidateNoMoreBatches @4.4.0
Done when: an empty records field fails with that message and INVALID_RECORD (test).

### [To Do] REC-0102 Rule: v2 or compressed input must hold exactly one batch ("Compressed outer record has more than one batch")
P0 · M2 · Task · deps: REC-0101 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java getFirstBatchAndMaybeValidateNoMoreBatches @4.4.0
Done when: two batches in one partition's records fail with that message and INVALID_RECORD; a single batch passes (tests).

### [To Do] REC-0103 Rule: every batch has the first batch's magic, with an invalid-magic signal
P0 · M2 · Task · deps: REC-0102 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java validateBatch @4.4.0
Done when: message matches "Batch magic … is not the same as the first batch's magic byte"; test.

### [To Do] REC-0104 Rule (client origin): last offset − base offset + 1 > 0, record count > 0, and the two agree, with an invalid-offset signal
P0 · M2 · Task · deps: REC-0103, REC-0011 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java validateBatch @4.4.0
Done when: the three messages ("invalid offset range", "Invalid reported count", "Inconsistent batch offset range") are tested; other origins skip the rule.

### [To Do] REC-0105 Rule (client origin): clients cannot write control batches (invalid-offset signal)
P0 · M2 · Task · deps: REC-0103 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java validateBatch @4.4.0
Done when: message "Clients are not allowed to write control records in topic partition …"; Coordinator origin may write markers (test).

### [To Do] REC-0106 Rule (client origin): a batch with a producer id must have base sequence ≥ 0, with an invalid-sequence signal
P0 · M2 · Task · deps: REC-0103, REC-0012 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java validateBatch @4.4.0
Done when: message "Invalid sequence number … in record batch with producerId …"; test.

### [To Do] REC-0107 Rule: a CreateTime record timestamp older than now − `message.timestamp.before.max.ms` is an INVALID_TIMESTAMP record error
P0 · M2 · Task · deps: REC-0094, REC-0026 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java validateTimestamp/recordHasInvalidTimestamp @4.4.0
Done when: the difference uses Java long wrapping like recordHasInvalidTimestamp (default Long.MAX_VALUE); NO_TIMESTAMP is skipped; the message names the allowed window.

### [To Do] REC-0108 Rule: a CreateTime record timestamp newer than now + `message.timestamp.after.max.ms` (default 1 hour) is an INVALID_TIMESTAMP record error
P0 · M2 · Task · deps: REC-0107 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java validateTimestamp @4.4.0
Done when: boundary tests at exactly after.max.ms and one millisecond beyond.

### [To Do] REC-0109 Rule: batches flagged LogAppendTime by the producer are INVALID_TIMESTAMP ("Producer should not set timestamp type to LogAppendTime")
P0 · M2 · Task · deps: REC-0107 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java validateTimestamp @4.4.0
Done when: applies when the topic uses LogAppendTime or the record timestamp is −1, as in Kafka's else-branch (tests for both).

### [To Do] REC-0110 Rule: compacted topics require keys, with a no-key signal
P0 · M2 · Task · deps: REC-0094, REC-0026 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java validateKey @4.4.0
Done when: message "Compacted topic cannot accept message without key in topic partition …"; non-compacted topics accept null keys.

### [To Do] REC-0111 Record error reporting: `RecordError { batch_index, message }` list; INVALID_TIMESTAMP wins over INVALID_RECORD; summary message with at most three errors; data for Produce v8+ `RecordErrors` and `ErrorMessage`
P0 · M2 · Task · deps: REC-0108, REC-0109, REC-0110 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java processRecordErrors; storage/src/main/java/org/apache/kafka/storage/internals/log/RecordValidationException.java @4.4.0
Done when: summary text matches processRecordErrors; every record error is collected before failing (test with mixed errors).

### [To Do] REC-0112 Uncompressed in-place offset assignment: offset counter per record, set last offset, set partition leader epoch
P0 · M2 · Task · deps: REC-0111, REC-0040, REC-0104, REC-0105, REC-0106 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java assignOffsetsNonCompressed @4.4.0
Done when: offsets are contiguous from the counter; no record bytes are copied (counting-allocator test).

### [To Do] REC-0113 CreateTime max timestamp: each batch's max record timestamp written with set-max-timestamp
P0 · M2 · Task · deps: REC-0112 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java assignOffsetsNonCompressed @4.4.0
Done when: the CRC changes only when the max timestamp changes; outcome max timestamp is the maximum over batches.

### [To Do] REC-0114 LogAppendTime topics: batch max timestamp and type set to now, CRC recomputed, outcome max timestamp = now
P0 · M2 · Task · deps: REC-0112 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java assignOffsetsNonCompressed/validateMessagesAndAssignOffsetsCompressed @4.4.0
Done when: records then read back with timestamp = now; tests for uncompressed and compressed in-place paths.

### [To Do] REC-0115 Compressed path decision: in place only when source codec equals target codec and inner offsets are 0..n−1; uncompressed control batches stay in place
P0 · M2 · Task · deps: REC-0111, REC-0073 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java validateMessagesAndAssignOffsetsCompressed @4.4.0
Done when: decision table tested, including inner offsets not starting at 0 (rebuild) and compressed control batches.

### [To Do] REC-0116 Compressed in-place assignment using skip-key-value iteration: last offset, max timestamp, leader epoch
P0 · M2 · Task · deps: REC-0115, REC-0078, REC-0112 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java validateMessagesAndAssignOffsetsCompressed @4.4.0
Done when: the compressed payload bytes are unchanged; uncompressed size is reported in stats.

### [To Do] REC-0117 Rebuild path: re-encode validated records with the target compression, keeping producer id, epoch, base sequence and transactional flag; outcome flags size-may-have-changed
P0 · M2 · Task · deps: REC-0115, REC-0077, REC-0041 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java buildRecordsAndAssignOffsets @4.4.0
Done when: covers source none → target codec, codec A → codec B and non-sequential inner offsets; output re-validates cleanly.

### [To Do] REC-0118 Post-rebuild size check against `max.message.bytes` → MESSAGE_TOO_LARGE
P0 · M2 · Task · deps: REC-0117, REC-0097 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java append @4.4.0
Done when: a batch that grows past the limit after recompression is rejected (test).

### [To Do] REC-0119 Validation stats and metric signals: temporary memory bytes, conversion count, and invalid magic, offset/sequence, CRC, no-key and bytes-rejected counts
P0 · M2 · Task · deps: REC-0117, REC-0116, REC-0098, REC-0110 · labels: observability · ref: storage/src/main/java/org/apache/kafka/storage/log/metrics/BrokerTopicMetrics.java; storage/src/main/java/org/apache/kafka/storage/internals/log/RecordValidationStats.java @4.4.0
Done when: the driver can export InvalidMagicNumberRecordsPerSec, InvalidOffsetOrSequenceRecordsPerSec, InvalidMessageCrcRecordsPerSec, NoKeyCompactedTopicRecordsPerSec and BytesRejectedPerSec equivalents and the TemporaryMemoryBytes request metric from the outcome; layer 0 touches no registry.

### [To Do] REC-0120 Error-code mapping: one exhaustive `From<&RecordsError> for ErrorCode` covering INVALID_RECORD, CORRUPT_MESSAGE, MESSAGE_TOO_LARGE, INVALID_TIMESTAMP, UNSUPPORTED_COMPRESSION_TYPE and the decompression errors
P0 · M2 · Task · deps: REC-0118, REC-0095, REC-0096, REC-0100, ERR-F02
Done when: the match has no wildcard arm; each variant has a test asserting its code.

### [To Do] REC-0121 Origin matrix tests: Client, Replication, Coordinator and RaftLeader appends apply exactly Kafka's checks
P0 · M2 · Task · deps: REC-0120 · labels: test
Done when: a table-driven test runs every rule under every origin.

### [To Do] REC-0122 Differential test: identical inputs through Kafka's LogValidator in the JVM harness and through Alluvium; validated bytes, max timestamp and errors match
P1 · M2 · Task · deps: REC-0120, PROTO-F10 · labels: test · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java @4.4.0
Done when: at least 10,000 generated cases per run agree; mismatches print a replayable seed.

### [To Do] REC-0123 Port LogValidatorTest cases that encode tricky semantics
P1 · M2 · Task · deps: REC-0120 · labels: test · ref: storage/src/test/java/org/apache/kafka/storage/internals/log/LogValidatorTest.java @4.4.0
Done when: ported tests carry `// Ported from:` attribution and pass.

### [To Do] REC-0124 Property tests: validated output has contiguous offsets from the counter, a valid CRC and unchanged record payloads; the validator never panics
P0 · M2 · Task · deps: REC-0120, TEST-F16 · labels: test
Done when: uses TEST-F16 strategies; regressions persisted.

### [To Do] REC-0125 `docs/records/validation.md`: every rule with its error code, record-error message, metric signal and Kafka source
P1 · M2 · Task · deps: REC-0120 · labels: docs
Done when: the page lists all rules in this feature.

## [To Do] REC-F13 Extension header conventions
P2 · M1 · Feature · deps: REC-F04 · labels: foundation-for:X-LARGE, foundation-for:X-SCHEMA, foundation-for:X-OBSX, foundation-for:X-WASM
Scope: Spike: reserve record-header conventions for chunking, schema ids and trace context so enhancements stay compatible.

### [To Do] REC-0126 Spike: record header conventions for Alluvium extensions — reserved name prefix, collisions with ecosystem headers, and a guarantee that Kafka-compatible paths never interpret or strip headers
P2 · M1 · Spike · deps: none · labels: foundation-for:X-LARGE, foundation-for:X-SCHEMA, foundation-for:X-OBSX
Done when: an ADR is accepted defining the prefix and the compatibility guarantee.

### [To Do] REC-0127 Chunking header convention for X-LARGE: names and binary encodings for chunk group id, index, count, total size and checksum
P2 · M1 · Task · deps: REC-0126 · labels: foundation-for:X-LARGE, docs
Done when: `docs/formats/record-headers.md` defines the headers with byte layouts and examples.

### [To Do] REC-0128 Schema-id header convention for X-SCHEMA and its relation to the Confluent wire-format magic byte
P2 · M1 · Task · deps: REC-0126 · labels: foundation-for:X-SCHEMA, docs
Done when: `docs/formats/record-headers.md` defines the header, its encoding and precedence rules versus an in-payload schema id.

### [To Do] REC-0129 Trace-context header convention for X-OBSX: W3C `traceparent` and `tracestate` as used by OpenTelemetry Kafka instrumentation
P2 · M1 · Task · deps: REC-0126 · labels: foundation-for:X-OBSX, docs
Done when: `docs/formats/record-headers.md` states that Alluvium reads but never requires or rewrites them.

### [To Do] REC-0130 `reserved_headers` module: constants and pure parse/encode helpers for the three conventions
P3 · M1 · Task · deps: REC-0127, REC-0128, REC-0129
Done when: property tests round-trip each header; malformed values return typed errors.

### [To Do] REC-0131 Test: records carrying the reserved headers pass validation and offset assignment byte for byte unchanged
P2 · M2 · Task · deps: REC-0130, REC-0112 · labels: test
Done when: the validator and builder leave header bytes untouched (test).

## [To Do] REC-F14 Record testing
P0 · M1 · Feature · deps: REC-F04, TEST-F07, TEST-F16
Scope: Property tests, fuzz targets per codec and for batch parsing, golden batches captured from Kafka clients.

### [To Do] REC-0132 Golden batches from Kafka 4.4.0 classes via the PROTO-F10 harness: every codec at default and boundary levels, CreateTime and LogAppendTime, idempotent, transactional, each control type, headers, null keys and values, empty batch, delete horizon
P0 · M1 · Task · deps: PROTO-F10 · labels: test · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecordsBuilder.java @4.4.0
Done when: files and a manifest describing each batch's expected header and records are committed under `crates/alluvium-records/tests/golden/java/`.

### [To Do] REC-0133 Golden batches from librdkafka (pinned image) for gzip, snappy, lz4, zstd and idempotent production, captured from a pinned Kafka 4.4.0 broker's segments with compression.type=producer
P1 · M1 · Task · deps: none · labels: test
Done when: batches with provenance (librdkafka version, image digests) are committed under `tests/golden/librdkafka/`; the capture script is `cargo xtask golden records`.

### [To Do] REC-0134 Golden batches from franz-go for each codec, captured the same way
P2 · M1 · Task · deps: REC-0133 · labels: test
Done when: batches with provenance are committed under `tests/golden/franz-go/`.

### [To Do] REC-0135 Golden batch tests: every corpus batch frames, validates its CRC and yields the manifest's header fields and records
P0 · M1 · Task · deps: REC-0132, REC-0133 · labels: test
Done when: all golden batches pass; failures name the file and field.

### [To Do] REC-0136 Harness verify mode: Kafka 4.4.0 MemoryRecords decodes Alluvium-built batches and compares records
P0 · M1 · Task · deps: PROTO-F10 · labels: test
Done when: the mode accepts a directory of batches plus expected records and reports per-batch results.

### [To Do] REC-0137 Fuzz target `record_batch`: framing, header, CRC and uncompressed record iteration
P0 · M1 · Task · deps: REC-0026, REC-0034, TEST-F07 · labels: security, test, decoder
Done when: runs clean in the CI fuzz budget; allocation bounded by input size.

### [To Do] REC-0138 Fuzz target `decompress_gzip`: bounded decompression of arbitrary bytes with a compress → decompress oracle
P0 · M1 · Task · deps: REC-0079, TEST-F07 · labels: security, test, decoder
Done when: runs clean in the CI fuzz budget with the decompression caps enforced.

### [To Do] REC-0139 Fuzz target `decompress_snappy` (xerial and raw) with the same oracle
P0 · M1 · Task · deps: REC-0080, TEST-F07 · labels: security, test, decoder
Done when: runs clean in the CI fuzz budget with the decompression caps enforced.

### [To Do] REC-0140 Fuzz target `decompress_lz4` (Kafka frame) with the same oracle
P0 · M1 · Task · deps: REC-0081, TEST-F07 · labels: security, test, decoder
Done when: runs clean in the CI fuzz budget with the decompression caps enforced.

### [To Do] REC-0141 Fuzz target `decompress_zstd` with the same oracle
P0 · M1 · Task · deps: REC-0082, TEST-F07 · labels: security, test, decoder
Done when: runs clean in the CI fuzz budget with the decompression caps enforced.

### [To Do] REC-0142 Fuzz target `record_batch_compressed`: compressed batch iteration with the per-record cap for every codec
P0 · M1 · Task · deps: REC-0078, TEST-F07 · labels: security, test, decoder
Done when: runs clean in the CI fuzz budget; every codec is reached (coverage report) and peak memory stays under the per-record cap plus chunk buffers.

### [To Do] REC-0143 Fuzz target `control_records`: key and value decoders for every control type
P0 · M1 · Task · deps: REC-0093, TEST-F07 · labels: security, test, decoder
Done when: runs clean in the CI fuzz budget; all seven control types and Unknown are reached from the seed corpus.

### [To Do] REC-0144 Fuzz target `log_validator`: arbitrary batches and configurations never panic and accepted output re-validates
P0 · M2 · Task · deps: REC-0120, TEST-F07 · labels: security, test
Done when: runs clean in the CI fuzz budget; any accepted output validates again with identical offsets and timestamps.

### [To Do] REC-0145 Seed record fuzz corpora from the golden batch corpus
P1 · M1 · Task · deps: REC-0135, REC-0137, REC-0142, REC-0143 · labels: test
Done when: each record target's corpus includes the relevant golden batches.

### [To Do] REC-0146 Property tests: builder → parser round trip for every codec, attribute combination and control type
P0 · M1 · Task · deps: REC-0050, REC-0055, REC-0061, REC-0066, REC-0077, REC-0093, TEST-F16 · labels: test
Done when: uses TEST-F16 strategies; regressions persisted.

### [To Do] REC-0147 Port DefaultRecordBatchTest, DefaultRecordTest and MemoryRecordsBuilderTest cases
P1 · M1 · Task · deps: REC-0039, REC-0077 · labels: test · ref: clients/src/test/java/org/apache/kafka/common/record/internal/DefaultRecordBatchTest.java @4.4.0
Done when: ported tests carry `// Ported from:` attribution and pass.

### [To Do] REC-0148 Port Lz4CompressionTest and SnappyCompressionTest cases
P1 · M1 · Task · deps: REC-0061, REC-0055, REC-0060, REC-0054 · labels: test · ref: clients/src/test/java/org/apache/kafka/common/compress/Lz4CompressionTest.java @4.4.0
Done when: ported tests carry attribution and pass.

### [To Do] REC-0149 Differential malformed-input corpus: crafted invalid batches decoded by Kafka and Alluvium yield the same error class
P2 · M1 · Task · deps: REC-0136, REC-0077 · labels: test
Done when: a committed corpus of at least 50 malformed batches agrees on accept/reject and error class.

## [To Do] REC-F15 Record performance
P2 · M2 · Feature · deps: REC-F04, TEST-F11
Scope: Benchmarks for parse/validate/CRC/compress; allocation-free iteration on the hot path.

### [To Do] REC-0150 Criterion benchmarks: batch framing, header parse and record iteration for 1, 100 and 10,000 records with 100 B and 10 KiB values
P2 · M2 · Task · deps: none · labels: perf
Done when: benchmarks run in the benchmark CI job with recorded baselines.

### [To Do] REC-0151 CRC32C throughput benchmarks: accelerated vs software path at 1 KiB, 64 KiB and 1 MiB
P2 · M2 · Task · deps: none · labels: perf
Done when: results recorded; the accelerated path is the default on CI hardware.

### [To Do] REC-0152 Codec benchmarks: compress and decompress throughput and ratio per codec and level on the NFR-F01 payload corpus
P2 · M2 · Task · deps: NFR-F01, REC-0073 · labels: perf
Done when: results recorded in docs/compression.md.

### [To Do] REC-0153 Validator benchmarks: uncompressed in place, compressed in place and recompression paths
P2 · M2 · Task · deps: REC-0120 · labels: perf
Done when: benchmarks run in CI with baselines.

### [To Do] REC-0154 Builder benchmarks: build 1 MiB batches per codec
P2 · M2 · Task · deps: none · labels: perf
Done when: benchmarks run in CI with baselines.

### [To Do] REC-0155 Allocation-free hot path: counting-allocator tests assert zero per-record allocations for uncompressed iteration and in-place validation
P1 · M2 · Task · deps: REC-0112 · labels: perf, nfr
Done when: tests fail on any per-record allocation.

### [To Do] REC-0156 Decompression buffer reuse (BufferSupplier equivalent), adopted only if the benchmarks show a gain
P3 · M2 · Task · deps: REC-0152, REC-0153 · labels: perf · ref: clients/src/main/java/org/apache/kafka/common/utils/internals/BufferSupplier.java @4.4.0
Done when: a benchmark comparison is in the PR; the pool is bounded, or the task closes as Won't Do with the numbers.

### [To Do] REC-0157 Performance budgets for record handling in docs/nfr.md with benchmark CI regression thresholds
P2 · M2 · Task · deps: REC-0150, REC-0151, REC-0152, REC-0153, REC-0154 · labels: perf, nfr
Done when: thresholds are set for parse, CRC, codec and validation benchmarks.
