# X-OBSX: Observability extras

> Crates: `alluvium-observability`, `alluvium-server` · Kafka counterpart: none (Alluvium enhancement) · Milestones: M17

End-to-end tracing through records, lag in time, built-in lag metrics, skew and hot-key detection, client inventory, lineage and cluster-wide diagnostics.

## [To Do] X-OBSX-F01 End-to-end tracing
P0 · M17 · Feature · deps: REC-F13
Scope: W3C trace context in record headers, linked to broker spans.

### [To Do] X-OBSX-0001 Observability extras ADR: record trace context, sampling and cost, lag in time, built-in lag metrics, detectors, client inventory, lineage, cluster diagnostics, inspection and opt-in keys
P0 · M17 · Spike · deps: none · ref: ADR-0005
Done when: ADR accepted deciding: `traceparent`/`tracestate` handling under the REC-F13 header conventions, the broker span model (span links, not parents), the offset-to-trace map for fetch links, the overhead budget, the lag-in-time definition and precision, cardinality caps, what is never recorded (keys, values, non-trace headers), and one `alluvium.*` enable key per feature, all defaulting off.

### [To Do] X-OBSX-0002 `TraceContext` parser for the `traceparent` header (version, 32-hex trace id, 16-hex parent id, flags) in the `alluvium-records` extension-header module
P0 · M17 · Task · deps: X-OBSX-0001 · labels: decoder · ref: W3C Trace Context Level 1
Done when: pure, panic-free parser; rejects all-zero ids, uppercase hex, wrong lengths and version `ff`; accepts future versions per the spec's forward-compatibility rule; W3C test vectors and a property round-trip test pass.

### [To Do] X-OBSX-0003 `tracestate` parser bounded to 32 list members and 512 bytes, kept opaque unless export is enabled
P0 · M17 · Task · deps: X-OBSX-0001 · labels: decoder · ref: W3C Trace Context Level 1
Done when: over-limit or malformed values are dropped with a typed reason, never truncated silently; property tests for list-member parsing; no allocation beyond the bound.

### [To Do] X-OBSX-0004 Fuzz targets `fuzz/fuzz_targets/traceparent.rs` and `tracestate.rs`, plus trace-header extraction over arbitrary record batches
P0 · M17 · Task · deps: X-OBSX-0002, X-OBSX-0003, TEST-F07 · labels: decoder, test, security
Done when: targets run in the per-PR fuzz job with W3C vectors as seed corpus; one-hour local runs find no panic, hang or unbounded allocation.

### [To Do] X-OBSX-0005 `alluvium.tracing.records.enabled` (default false) and companion keys registered through the CFG-F11 registry, dynamic per broker
P0 · M17 · Task · deps: X-OBSX-0001, CFG-F11 · labels: config
Done when: keys documented in docs/config.md with the ADR link; dynamic changes take effect for new requests without restart (test).

### [To Do] X-OBSX-0006 Trace header extraction on the produce path: find `traceparent` in record headers without copying payloads, only when enabled and the batch is sampled
P0 · M17 · Task · deps: X-OBSX-0002, X-OBSX-0005, REC-F12 · labels: perf
Done when: uncompressed batches scanned header-only; compressed batches reuse the validator's decompressed iteration instead of decompressing twice; per-request cap `alluvium.tracing.records.max.contexts.per.request`; with the feature disabled, a counting test shows zero header parsing.

### [To Do] X-OBSX-0007 Broker sampling policy: honour the sampled flag, then apply `alluvium.tracing.records.sample.ratio` and `alluvium.tracing.records.max.traced.per.sec` with a per-node seeded sampler
P0 · M17 · Task · deps: X-OBSX-0006
Done when: property test: traced rate never exceeds the cap; unsampled contexts never produce spans; sampler deterministic for a given seed.

### [To Do] X-OBSX-0008 Propagation point, produce: the Produce `request` span carries span links to sampled record contexts with topic, partition and base_offset attributes
P0 · M17 · Task · deps: X-OBSX-0007, OBS-F03
Done when: integration test with an in-memory span exporter sees one link per sampled record context, capped per request with `links_dropped` counted.

### [To Do] X-OBSX-0009 Propagation point, append: the leader `partition` span links each traced record with its assigned offset
P0 · M17 · Task · deps: X-OBSX-0008
Done when: assigned offsets match the produce response base offset plus record index (test with multi-record batches).

### [To Do] X-OBSX-0010 Offset-to-trace map: bounded per-partition ring (`alluvium.tracing.records.offset.map.entries`) of (offset, trace id, span id) filled at append and used by later hops without reading payloads
P0 · M17 · Task · deps: X-OBSX-0009
Done when: memory bounded per partition and in total (test); FIFO eviction counted; truncation and log deletion remove stale entries; property test: lookups return only contexts whose offsets are in range.

### [To Do] X-OBSX-0011 Propagation point, commit: when the high watermark passes a traced offset, emit a produce-to-commit span (covering the delayed produce wait) linked to the record context
P0 · M17 · Task · deps: X-OBSX-0010, REPL-F08
Done when: span duration equals append-to-HW time under the simulated clock; acks=1 produces no commit span; test with acks=all and RF=3.

### [To Do] X-OBSX-0012 Propagation point, replication (internal hop): the follower append span links traced records it fetched, filling the follower's own offset map
P0 · M17 · Task · deps: X-OBSX-0010, REPL-F06
Done when: follower spans share the producer's trace id in the exported data; follower map stays within its bound; test with RF=3.

### [To Do] X-OBSX-0013 Propagation point, fetch: the consumer Fetch `request` span links to traced offsets in the returned ranges using the offset map, including sendfile responses
P0 · M17 · Task · deps: X-OBSX-0010
Done when: no payload bytes are read to build links (counting fs test); follower fetches excluded; links capped per request.

### [To Do] X-OBSX-0014 Propagation point, transactions (internal hop): transaction marker writes and LSO advance linked to traced records of the transaction
P1 · M17 · Task · deps: X-OBSX-0010, TXN-F08, TXN-F11
Done when: read_committed visibility span (append to LSO passing the offset) emitted for traced transactional records; test with a committed and an aborted transaction.

### [To Do] X-OBSX-0015 Propagation point, consume: the native Rust SDK injects `traceparent` on produce and extracts it on consume, creating producer and consumer spans per the OpenTelemetry messaging conventions
P1 · M17 · Task · deps: X-OBSX-0008, X-NATIVE-F12
Done when: SDK test shows producer, broker and consumer spans in one trace; injection disabled by default in the SDK.

### [To Do] X-OBSX-0016 Propagation point, cluster-link mirroring (internal hop): mirrored batches keep headers and the destination append span links to the source context
P2 · M17 · Task · deps: X-OBSX-0010, X-GEO-F03
Done when: test with a mirror topic: the destination span carries the source trace id; no header rewriting.

### [To Do] X-OBSX-0017 Propagation point, WASM transform (internal hop): the transform invocation span links to the input record context and the context is propagated to output records
P2 · M17 · Task · deps: X-OBSX-0010, X-WASM-F05
Done when: output records carry a `traceparent` with the same trace id and a new parent id; test with a produce-time transform.

### [To Do] X-OBSX-0018 OTLP export of record-linked spans through the OBS-F12 layer, with node_id and cluster id resource attributes and span link limits
P0 · M17 · Task · deps: X-OBSX-0008, OBS-F12
Done when: collector test receives broker spans with links; export queue bounded with drops counted.

### [To Do] X-OBSX-0019 Security: only `traceparent` and `tracestate` header keys are read; other headers, keys and values are never touched or logged; tracestate exported only with `alluvium.tracing.records.export.tracestate=true`
P0 · M17 · Task · deps: X-OBSX-0006, OBS-F11 · labels: security
Done when: OBS-F11 redaction tests extended to record-linked spans; a test with secret-looking header values finds none in logs, spans or metrics.

### [To Do] X-OBSX-0020 Metrics: `alluvium_tracing_records_traced_total`, `alluvium_tracing_records_links_dropped_total{reason}`, `alluvium_tracing_records_parse_errors_total`, `alluvium_tracing_records_offset_map_evictions_total`
P0 · M17 · Task · deps: X-OBSX-0007, X-OBSX-0010 · labels: observability
Done when: values asserted in tests; rows in docs/metrics.md; no topic or partition labels unless `alluvium.metrics.per.partition.enabled`.

### [To Do] X-OBSX-0021 End-to-end test: Java producer and consumer with the OpenTelemetry Java agent export to an OTLP collector that receives one trace across produce, append, replication, commit and fetch
P0 · M17 · Task · deps: X-OBSX-0013, X-OBSX-0018, TEST-F08 · labels: test
Done when: runs in `cargo xtask compat`; asserts span names and links; uses a pinned collector image.

### [To Do] X-OBSX-0022 Overhead benchmarks: produce and fetch throughput and p99 with tracing disabled (equal to baseline within noise), enabled at 1% and at 100% sampling
P0 · M17 · Task · deps: X-OBSX-0013, TEST-F11 · labels: perf
Done when: criterion and macro results published; disabled-path overhead within noise and enabled overhead within the ADR budget, enforced in the benchmark job.

### [To Do] X-OBSX-0023 End-to-end tracing guide: enabling, sampling, client instrumentation (Java agent, librdkafka header injection, native SDK) and reading the traces
P1 · M17 · Task · deps: X-OBSX-0021 · labels: docs
Done when: guide includes a worked trace with every propagation point labeled.

## [To Do] X-OBSX-F02 Lag in time
P0 · M17 · Feature · deps: STOR-F05
Scope: Consumer lag in records and in time.

### [To Do] X-OBSX-0024 Record lag: high watermark minus committed offset, matching kafka-consumer-groups.sh
P0 · M17 · Task · deps: X-OBSX-0001 · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ConsumerGroupCommand.java getLag @4.4.0
Done when: pure function agrees with ConsumerGroupCommand on ported cases; a missing committed offset gives `undefined`, not zero; committed offset beyond the end gives 0 with reason `committed_beyond_end`.

### [To Do] X-OBSX-0025 Offset-to-timestamp lookup: timestamp of the record at an offset from its batch header (base timestamp plus record delta) through the offset index, reading headers only
P0 · M17 · Task · deps: X-OBSX-0001, STOR-F04
Done when: counting fs test shows no payload reads beyond the batch header and record prefix; works for compressed batches via the batch's own timestamps per the ADR's precision rules; typed errors for corrupt headers.

### [To Do] X-OBSX-0026 Bounded timestamp cache per partition (`alluvium.lag.timestamp.cache.entries`) for unchanged committed offsets
P0 · M17 · Task · deps: X-OBSX-0025
Done when: cache hit avoids any fs call (test); memory bounded in total; entries invalidated on truncation and log start offset advance.

### [To Do] X-OBSX-0027 Time lag: now minus the timestamp of the record at the committed offset when record lag is positive, zero when caught up, using the injected clock
P0 · M17 · Task · deps: X-OBSX-0024, X-OBSX-0026
Done when: unit tests with the manual clock; property test: zero if and only if record lag is zero; never negative.

### [To Do] X-OBSX-0028 Timestamp type handling: LogAppendTime topics give exact time lag; CreateTime topics clamp future timestamps to now and flag skew beyond `alluvium.lag.max.client.clock.skew.ms`
P0 · M17 · Task · deps: X-OBSX-0027, CFG-F05
Done when: tests per `message.timestamp.type`; flagged partitions carry reason `client_clock_skew`.

### [To Do] X-OBSX-0029 Edge cases: committed offset below log start offset (lag from log start, flag `data_lost`), empty partitions, recreated topics (topic id mismatch), control records at the committed offset skipped
P0 · M17 · Task · deps: X-OBSX-0027
Done when: one test per case; recreated topics report `unknown` with reason `topic_id_mismatch`.

### [To Do] X-OBSX-0030 Committed offsets in tiered segments: timestamp from remote segment metadata when available, otherwise `unknown` with reason; never a remote read on this path
P1 · M17 · Task · deps: X-OBSX-0027, TIER-F02
Done when: test with an offset in a remote-only segment; no RemoteStorageManager fetch issued (counting mock of the external store).

### [To Do] X-OBSX-0031 Time-to-drain estimate: record lag divided by the group's consumption rate over `alluvium.lag.rate.window.ms`; infinite when consumption has stalled
P0 · M17 · Task · deps: X-OBSX-0024
Done when: unit tests with scripted commits; stalled groups reported as `stalled`, not as a huge number.

### [To Do] X-OBSX-0032 Native admin `GetGroupLag`: records, time lag and drain estimate per partition, requiring DESCRIBE on the group and on each topic
P0 · M17 · Task · deps: X-OBSX-0027, X-OBSX-0031, X-NATIVE-F08, SEC-F08 · labels: security
Done when: partitions of topics the caller cannot describe are omitted, as Kafka's OffsetFetch filtering does; contract tests.

### [To Do] X-OBSX-0033 `alluvium groups lag <group>` CLI printing records and time lag per partition, with `--json`
P1 · M17 · Task · deps: X-OBSX-0032, X-NATIVE-F14
Done when: CLI tests against a dev node; output columns documented.

### [To Do] X-OBSX-0034 Property tests: time lag never increases as the committed offset advances with a fixed clock; record lag and time lag agree on zero
P0 · M17 · Task · deps: X-OBSX-0029, X-OBSX-0028 · labels: test
Done when: proptest suite with persisted regressions over generated logs and commit sequences.

### [To Do] X-OBSX-0035 Benchmark: lag for 10k partitions per computation within the ADR budget, with zero payload reads
P1 · M17 · Task · deps: X-OBSX-0026, TEST-F11 · labels: perf
Done when: criterion bench with a counting fs; budget enforced in the benchmark job.

### [To Do] X-OBSX-0036 Lag-in-time documentation: definitions, precision, CreateTime skew, compaction gaps and tiered caveats
P1 · M17 · Task · deps: X-OBSX-0029 · labels: docs
Done when: page explains each reason value returned with lag.

## [To Do] X-OBSX-F03 Built-in group lag metrics
P0 · M17 · Feature · deps: GRP-F07
Scope: Broker-exported lag per group and partition with bounded cardinality.

### [To Do] X-OBSX-0037 `alluvium.metrics.group.lag.enabled` (default false) and `alluvium.metrics.group.lag.interval.ms` registered via CFG-F11
P0 · M17 · Task · deps: X-OBSX-0001, CFG-F11 · labels: config
Done when: keys documented; dynamic enable and disable create and remove series without restart (test).

### [To Do] X-OBSX-0038 End-offset source for lag metrics: local high watermarks from published offsets, remote ones via batched inter-broker ListOffsets
P0 · M17 · Task · deps: X-OBSX-0037, NET-F10
Done when: one ListOffsets per leader per interval bounded by `alluvium.metrics.group.lag.list.offsets.timeout.ms` (test); timeouts mark partitions `stale` rather than reusing old values silently.

### [To Do] X-OBSX-0039 Periodic computation on each coordinator for its loaded groups, with bounded work per tick (`alluvium.metrics.group.lag.max.partitions.per.tick`)
P0 · M17 · Task · deps: X-OBSX-0038, GRP-F10
Done when: work spread across ticks for large groups; a full pass completes within the interval in the benchmark scenario.

### [To Do] X-OBSX-0040 Metric `alluvium_group_lag_records{group,topic}`: record lag summed over the group's partitions of a topic
P0 · M17 · Task · deps: X-OBSX-0039
Done when: values match `kafka-consumer-groups.sh --describe` for a scripted workload.

### [To Do] X-OBSX-0041 Metrics `alluvium_group_lag_max_records{group,topic}` and, only with `alluvium.metrics.per.partition.enabled`, `alluvium_group_partition_lag_records{group,topic,partition}`
P0 · M17 · Task · deps: X-OBSX-0039
Done when: per-partition series absent by default (test); max matches the largest partition lag.

### [To Do] X-OBSX-0042 Metric `alluvium_group_lag_seconds{group,topic}`: maximum time lag over the group's partitions of a topic
P0 · M17 · Task · deps: X-OBSX-0039, X-OBSX-F02
Done when: uses the X-OBSX-F02 computation; `unknown` partitions excluded and counted in `alluvium_group_lag_unknown_partitions{group,topic}`.

### [To Do] X-OBSX-0043 Metrics `alluvium_group_lag_drain_seconds{group,topic}` and `alluvium_group_consume_rate_records_per_second{group,topic}`
P1 · M17 · Task · deps: X-OBSX-0039, X-OBSX-F02
Done when: stalled groups exported as `+Inf` per Prometheus conventions; values asserted in tests.

### [To Do] X-OBSX-0044 Cardinality bounds: cap `alluvium.metrics.group.lag.max.groups`, include and exclude regexes, overflow counted in `alluvium_group_lag_series_dropped_total`
P0 · M17 · Task · deps: X-OBSX-0040 · labels: security
Done when: test with 100k groups keeps series under the cap; group ids from clients never create series beyond the cap (denial-of-service guard).

### [To Do] X-OBSX-0045 Coordinator migration: series created on group load and removed on unload, never reported by two coordinators at once
P0 · M17 · Task · deps: X-OBSX-0039
Done when: simulation with coordinator failover shows no duplicate series at any scrape point.

### [To Do] X-OBSX-0046 Share group lag: records between the share-partition start offset and the high watermark per share group and topic
P2 · M17 · Task · deps: X-OBSX-0039, SHARE-F02
Done when: metric `alluvium_share_group_lag_records{group,topic}` matches share-partition state in a test.

### [To Do] X-OBSX-0047 Mapping from Burrow and kafka-lag-exporter metric names to Alluvium lag metrics for dashboard migration
P1 · M17 · Task · deps: X-OBSX-0040, X-OBSX-0042 · labels: docs
Done when: table in docs/metrics.md; example PromQL translations.

### [To Do] X-OBSX-0048 Lag metric tests: scripted workload values, empty group, group without commits, deleted topic, disabled feature exports nothing
P0 · M17 · Task · deps: X-OBSX-0040, X-OBSX-0041, X-OBSX-0042, X-OBSX-0044 · labels: test
Done when: each case asserted on the explicit metrics registry.

### [To Do] X-OBSX-0049 Benchmark: 10k groups x 100 partitions computed within one interval, with CPU and memory inside the ADR budget
P1 · M17 · Task · deps: X-OBSX-0039, TEST-F11 · labels: perf
Done when: macro benchmark result published; budget enforced in the benchmark job.

## [To Do] X-OBSX-F04 Skew and hot-key detection
P1 · M17 · Feature · deps: OBS-F04
Scope: Partition skew and hot-key sampling.

### [To Do] X-OBSX-0050 `alluvium.detect.skew.enabled` and `alluvium.detect.hot.keys.enabled` (default false) with detector keys registered via CFG-F11
P1 · M17 · Task · deps: X-OBSX-0001, CFG-F11 · labels: config
Done when: keys documented; disabled detectors allocate nothing (test).

### [To Do] X-OBSX-0051 Detector: partition bytes-in skew per topic (max over mean and coefficient of variation over windows), metric `alluvium_topic_partition_skew_ratio{topic}`
P1 · M17 · Task · deps: X-OBSX-0050
Done when: unit tests on scripted rates; topics under `alluvium.detect.skew.min.bytes.per.sec` ignored.

### [To Do] X-OBSX-0052 Detector: partition produce request-rate skew per topic, metric `alluvium_topic_partition_request_skew_ratio{topic}`
P1 · M17 · Task · deps: X-OBSX-0050
Done when: unit tests on scripted rates; same windowing as the bytes detector.

### [To Do] X-OBSX-0053 Hot-key sampler: 1-in-N records sampled at append (`alluvium.detect.hot.keys.sample.rate`), keys reduced to a keyed SipHash with a per-cluster secret; raw keys never stored
P1 · M17 · Task · deps: X-OBSX-0050 · labels: security
Done when: sampler reuses the validator's record iteration; test asserts no raw key bytes remain in memory structures; secret is a password-type key.

### [To Do] X-OBSX-0054 Heavy-hitter sketch per partition: space-saving top-K (`alluvium.detect.hot.keys.top.k`) with bounded memory per partition and a total cap
P1 · M17 · Task · deps: X-OBSX-0053
Done when: memory per partition is O(K); total sketches capped with eviction of the coldest partitions; unit tests.

### [To Do] X-OBSX-0055 Detector: hot key when one key's share of a partition's records or bytes exceeds `alluvium.detect.hot.keys.threshold.ratio` for `alluvium.detect.hot.keys.windows` windows
P1 · M17 · Task · deps: X-OBSX-0054
Done when: event `hot_key_detected` with key hash, topic, partition and share, rate-limited; test with a scripted skewed workload.

### [To Do] X-OBSX-0056 Authorized key reveal: map a hot-key hash to raw keys only through message inspection, with READ on the topic and redaction rules applied
P2 · M17 · Task · deps: X-OBSX-0055, X-OBSX-F09 · labels: security
Done when: without READ the reveal is denied with `authz_decision`; with redaction the key is masked; test.

### [To Do] X-OBSX-0057 Exposure: `/debug/topics/{topic}/skew` with partition shares and hot-key hashes; metrics with topic labels only
P1 · M17 · Task · deps: X-OBSX-0051, X-OBSX-0055, OBS-F08 · labels: observability
Done when: endpoint and metrics asserted in an integration test; no partition labels unless `alluvium.metrics.per.partition.enabled`.

### [To Do] X-OBSX-0058 Property tests for space-saving guarantees: any key with true frequency above N/K is reported and overestimation is at most N/K
P1 · M17 · Task · deps: X-OBSX-0054 · labels: test
Done when: proptest suite over generated streams with persisted regressions.

### [To Do] X-OBSX-0059 Overhead benchmark: produce throughput with sampling at 1/100 and 1/1, and zero cost when disabled
P2 · M17 · Task · deps: X-OBSX-0053, TEST-F11 · labels: perf
Done when: results within the ADR budget; enforced in the benchmark job.

### [To Do] X-OBSX-0060 Skew and hot-key documentation: detectors, thresholds, privacy model of key hashes and remediation (keying, elastic partitions)
P2 · M17 · Task · deps: X-OBSX-0055 · labels: docs
Done when: page documents every key, metric and event.

## [To Do] X-OBSX-F05 Slow client detection
P1 · M17 · Feature · deps: OBS-F04
Scope: Slow consumer and producer detection.

### [To Do] X-OBSX-0061 Per-client accounting keyed by (principal, client_id), bounded LRU `alluvium.detect.clients.max.tracked` with eviction counter
P1 · M17 · Task · deps: X-OBSX-0001, SRV-F06
Done when: memory bounded (test with 1M distinct client ids); eviction counted in `alluvium_detect_clients_evicted_total`.

### [To Do] X-OBSX-0062 `alluvium.detect.slow.clients.enabled` (default false) and detector threshold keys registered via CFG-F11
P1 · M17 · Task · deps: X-OBSX-0001, CFG-F11 · labels: config
Done when: keys documented; disabled detection allocates nothing.

### [To Do] X-OBSX-0063 Detector: slow consumer - a group's time lag grows for `alluvium.detect.slow.consumer.windows` windows while data is available
P1 · M17 · Task · deps: X-OBSX-0061, X-OBSX-0062, X-OBSX-F02
Done when: scripted consumer falling behind is detected; an idle topic with no new data is not.

### [To Do] X-OBSX-0064 Detector: slow network consumer - fetch response send time above `alluvium.detect.slow.send.threshold.ms` repeatedly per connection (socket backpressure)
P1 · M17 · Task · deps: X-OBSX-0061, X-OBSX-0062, NET-F06
Done when: test with a simulated slow socket; per-connection state bounded.

### [To Do] X-OBSX-0065 Detector: inefficient producer - average records per batch below `alluvium.detect.producer.min.records.per.batch` at a request rate above `alluvium.detect.producer.min.requests.per.sec`
P1 · M17 · Task · deps: X-OBSX-0061, X-OBSX-0062
Done when: scripted `linger.ms=0` producer detected; low-rate producers ignored.

### [To Do] X-OBSX-0066 Detector: retrying producer - elevated DUPLICATE_SEQUENCE_NUMBER, OUT_OF_ORDER_SEQUENCE_NUMBER and NOT_LEADER_OR_FOLLOWER rates per client
P1 · M17 · Task · deps: X-OBSX-0061, X-OBSX-0062, TXN-F01
Done when: thresholds per error code; test with scripted error sequences.

### [To Do] X-OBSX-0067 Detector: rebalance-causing member - members repeatedly exceeding session or rebalance timeouts, from group events
P2 · M17 · Task · deps: X-OBSX-0061, X-OBSX-0062, GRP-F11
Done when: test with scripted group events; detection names group and client id.

### [To Do] X-OBSX-0068 Detector: throttled client - quota throttle time above `alluvium.detect.throttle.ratio` of wall time for N windows
P2 · M17 · Task · deps: X-OBSX-0061, X-OBSX-0062, QUOTA-F07
Done when: test with scripted throttle times.

### [To Do] X-OBSX-0069 Exposure: `/debug/clients/slow` with detector, evidence and since; metric `alluvium_slow_clients{detector}`; rate-limited `slow_client_detected` event
P1 · M17 · Task · deps: X-OBSX-0063, X-OBSX-0064, X-OBSX-0065, X-OBSX-0066, OBS-F08 · labels: observability
Done when: endpoint, metric and event asserted in tests; events at most once per client and detector per `alluvium.detect.event.interval.ms`.

### [To Do] X-OBSX-0070 Detector tests: each detector on scripted simulated workloads, no false positives for idle or healthy clients
P1 · M17 · Task · deps: X-OBSX-0069 · labels: test
Done when: suite runs under the simulated clock with no sleeps.

### [To Do] X-OBSX-0071 Slow client documentation: detectors, thresholds and remediation hints per detector
P2 · M17 · Task · deps: X-OBSX-0069 · labels: docs
Done when: each detector has a symptom, evidence and fix section.

## [To Do] X-OBSX-F06 Client inventory
P1 · M17 · Feature · deps: SRV-F06
Scope: Client software names and versions per principal.

### [To Do] X-OBSX-0072 Client inventory registry: entries keyed by (principal, client_id, software name, software version, listener) with connection count and first and last seen, bounded by `alluvium.clients.inventory.max.entries`
P1 · M17 · Task · deps: X-OBSX-0001
Done when: memory bounded with LRU eviction counted; unit tests for connect, disconnect and eviction.

### [To Do] X-OBSX-0073 `alluvium.clients.inventory.enabled` (default false) and size keys registered via CFG-F11
P1 · M17 · Task · deps: X-OBSX-0001, CFG-F11 · labels: config
Done when: keys documented; disabled inventory records nothing.

### [To Do] X-OBSX-0074 Fill from ApiVersions v3+ `ClientSoftwareName` and `ClientSoftwareVersion` (KIP-511), using `unknown` for older clients
P1 · M17 · Task · deps: X-OBSX-0072, X-OBSX-0073, API-F18 · labels: security · ref: clients/src/main/resources/common/message/ApiVersionsRequest.json @4.4.0
Done when: only values already validated by the ApiVersions handler are recorded; length bounded; v0-v2 clients appear as `unknown`; tests per version.

### [To Do] X-OBSX-0075 API usage per entry: bitset of (api key, version) used, to find clients on old protocol versions before upgrades
P1 · M17 · Task · deps: X-OBSX-0072
Done when: fixed-size bitset per entry; test shows a client using Fetch v4 flagged by a query for versions below a threshold.

### [To Do] X-OBSX-0076 Link KIP-714 client instance ids to inventory entries
P2 · M17 · Task · deps: X-OBSX-0072, TELEM-F01
Done when: test with a telemetry-enabled Java client shows its instance id on the entry.

### [To Do] X-OBSX-0077 Metric `alluvium_client_connections{software_name,software_version}` with a cap on distinct label pairs and an `other` bucket
P1 · M17 · Task · deps: X-OBSX-0074 · labels: observability
Done when: cap enforced under 10k distinct versions (test); rows in docs/metrics.md.

### [To Do] X-OBSX-0078 Exposure: `/debug/clients` and native admin `ListClients`, requiring DESCRIBE on CLUSTER
P1 · M17 · Task · deps: X-OBSX-0075, X-OBSX-0077, X-NATIVE-F08, OBS-F08 · labels: security
Done when: contract tests; denials emit `authz_decision`.

### [To Do] X-OBSX-0079 Cluster-wide inventory through the cluster diagnostics fan-out, merged by key across nodes
P2 · M17 · Task · deps: X-OBSX-0078, X-OBSX-F08
Done when: test with three nodes and clients spread across them.

### [To Do] X-OBSX-0080 Compatibility test: Java client, librdkafka and franz-go appear with correct software names and versions
P1 · M17 · Task · deps: X-OBSX-0074, TEST-F08 · labels: test
Done when: runs in `cargo xtask compat`.

### [To Do] X-OBSX-0081 Client inventory documentation: using it for upgrade planning and finding old clients
P2 · M17 · Task · deps: X-OBSX-0078 · labels: docs
Done when: worked example of finding clients below a protocol version.

## [To Do] X-OBSX-F07 Data lineage
P2 · M17 · Feature · deps: X-OBSX-F06
Scope: Producer-topic-consumer graph.

### [To Do] X-OBSX-0082 Lineage graph model: nodes (client identity, transactional id, group, topic) and edges (produces, consumes, commits) with byte rates and last seen, bounded nodes and edges
P2 · M17 · Task · deps: none
Done when: bounded by `alluvium.lineage.max.nodes` and `alluvium.lineage.max.edges` with eviction counted; unit tests.

### [To Do] X-OBSX-0083 `alluvium.lineage.enabled` (default false) and lineage keys registered via CFG-F11
P2 · M17 · Task · deps: X-OBSX-0082, CFG-F11 · labels: config
Done when: keys documented; disabled lineage records nothing.

### [To Do] X-OBSX-0084 Edge recording for produce: per (client identity, topic), from the produce handler
P2 · M17 · Task · deps: X-OBSX-0083
Done when: byte rates match produced bytes in a test; transactional ids attached when present.

### [To Do] X-OBSX-0085 Edge recording for consume: per (client identity, topic), from consumer fetches, follower fetches excluded
P2 · M17 · Task · deps: X-OBSX-0083
Done when: test shows consumer edges only; rates match fetched bytes.

### [To Do] X-OBSX-0086 Group association: link consume edges to groups through coordinator membership and offset commits
P2 · M17 · Task · deps: X-OBSX-0085, GRP-F08
Done when: test with two groups sharing a client id distinguishes them by member and commit data.

### [To Do] X-OBSX-0087 Internal edges: cluster-link mirrors and derived topics recorded as topic-to-topic edges
P3 · M17 · Task · deps: X-OBSX-0082, X-GEO-F03, X-WASM-F06
Done when: test with a mirror topic and a derived topic shows both edges.

### [To Do] X-OBSX-0088 Edge expiry after `alluvium.lineage.edge.ttl.ms` without traffic, using the injected clock
P2 · M17 · Task · deps: X-OBSX-0084, X-OBSX-0085
Done when: expiry test with the manual clock.

### [To Do] X-OBSX-0089 Cluster-wide graph merged through the cluster diagnostics fan-out
P2 · M17 · Task · deps: X-OBSX-0086, X-OBSX-F08
Done when: test with clients spread over three nodes produces one merged graph.

### [To Do] X-OBSX-0090 Authorization: the graph is filtered to topics and groups the caller can DESCRIBE
P2 · M17 · Task · deps: X-OBSX-0089, SEC-F08 · labels: security
Done when: test with a restricted principal sees only permitted nodes and edges; no edge leaks a hidden topic name.

### [To Do] X-OBSX-0091 Export: JSON graph via native admin and OpenLineage events to `alluvium.lineage.export.url`
P3 · M17 · Task · deps: X-OBSX-0090
Done when: JSON schema published; export queue bounded; OpenLineage payload validated against its schema.

### [To Do] X-OBSX-0092 Lineage tests: scripted produce and consume topology yields the expected graph
P2 · M17 · Task · deps: X-OBSX-0090, X-OBSX-0088 · labels: test
Done when: golden-file graph comparison for a three-service pipeline.

### [To Do] X-OBSX-0093 Lineage documentation
P3 · M17 · Task · deps: X-OBSX-0090 · labels: docs
Done when: page documents the model, limits and export format.

## [To Do] X-OBSX-F08 Cluster-wide diagnostics
P1 · M17 · Feature · deps: OBS-F08
Scope: Fan-out queries across nodes for state and flight recorders.

### [To Do] X-OBSX-0094 `alluvium.diagnostics.cluster.enabled` (default false) and fan-out keys registered via CFG-F11
P1 · M17 · Task · deps: X-OBSX-0001, CFG-F11 · labels: config
Done when: keys documented; disabled nodes return 404 for `/debug/cluster/*`.

### [To Do] X-OBSX-0095 Diagnostics address discovery: nodes publish their diagnostics endpoint in an Alluvium registration record gated by a feature level
P1 · M17 · Task · deps: X-OBSX-0094, META-F06
Done when: record replayed into the image; absent for nodes without the feature; round-trip test.

### [To Do] X-OBSX-0096 Inter-node authentication: mTLS between diagnostics listeners with node certificates; the caller is authenticated at the entry node; no user credentials forwarded
P1 · M17 · Task · deps: X-OBSX-0095, OBS-F08 · labels: security
Done when: requests without a valid node certificate are rejected; test that a bearer token is never forwarded.

### [To Do] X-OBSX-0097 Fan-out framework: the entry node queries every node with bounded concurrency and per-node timeouts, returning partial results with per-node errors
P1 · M17 · Task · deps: X-OBSX-0096
Done when: bounds from `alluvium.diagnostics.cluster.max.concurrency` and `alluvium.diagnostics.cluster.node.timeout.ms`; test with one node down returns other nodes' results plus a typed error for the missing node; no unbounded waits.

### [To Do] X-OBSX-0098 Response limits: maximum bytes per node response and in total, with truncation markers
P1 · M17 · Task · deps: X-OBSX-0097 · labels: security
Done when: oversized node responses truncated with a marker; memory bounded in a test with a large flight recorder.

### [To Do] X-OBSX-0099 Query: one partition across all replicas: LEO, HW, leader epoch and ISR view per replica, with divergence highlighted
P1 · M17 · Task · deps: X-OBSX-0097
Done when: test with a lagging follower highlights its HW and LEO difference.

### [To Do] X-OBSX-0100 Query: merged flight recorder timeline for a partition, group or quorum across nodes, ordered by timestamp with per-node clock offset estimates
P1 · M17 · Task · deps: X-OBSX-0097
Done when: clock offset estimated from request round trips; test with skewed simulated clocks orders events correctly within the estimate.

### [To Do] X-OBSX-0101 Query: quorum state from all voters and observers (leader, epoch, high watermark, lag)
P1 · M17 · Task · deps: X-OBSX-0097
Done when: test with a three-voter quorum and one lagging observer.

### [To Do] X-OBSX-0102 Query: effective config diff across brokers for drift detection, secrets redacted
P1 · M17 · Task · deps: X-OBSX-0097 · labels: security
Done when: test with one broker overriding a key shows the drift; password-type values appear as `[redacted]`.

### [To Do] X-OBSX-0103 Query: supervised tasks and runtime health per node
P2 · M17 · Task · deps: X-OBSX-0097
Done when: merges `/debug/tasks` from all nodes with node ids.

### [To Do] X-OBSX-0104 Query: group state from its coordinator plus member connection details from the brokers they are connected to
P2 · M17 · Task · deps: X-OBSX-0097
Done when: test with members connected to different brokers.

### [To Do] X-OBSX-0105 Cluster debug bundle: `alluvium debug-bundle --cluster` collects every node's bundle through the fan-out
P1 · M17 · Task · deps: X-OBSX-0097, CLI-F05
Done when: bundle contains one directory per node plus a manifest of failures; redaction test passes.

### [To Do] X-OBSX-0106 Metrics: `alluvium_diagnostics_cluster_requests_total{query}`, `alluvium_diagnostics_cluster_node_failures_total{reason}`, `alluvium_diagnostics_cluster_seconds{query}`
P1 · M17 · Task · deps: X-OBSX-0097 · labels: observability
Done when: values asserted in tests; rows in docs/metrics.md.

### [To Do] X-OBSX-0107 Multi-node integration tests for every query, including one node down and one node slow
P1 · M17 · Task · deps: X-OBSX-0099, X-OBSX-0100, X-OBSX-0101, X-OBSX-0102 · labels: test
Done when: partial results and timeouts asserted without real sleeps (simulated transport where possible).

### [To Do] X-OBSX-0108 Cluster diagnostics documentation and debugging walkthroughs using the merged timelines
P1 · M17 · Task · deps: X-OBSX-0107 · labels: docs
Done when: debugging guide gains a cluster-wide investigation example.

## [To Do] X-OBSX-F09 Message inspection
P1 · M17 · Feature · deps: SEC-F07, X-SECX-F09
Scope: Browse records with authorization and redaction.

### [To Do] X-OBSX-0109 `alluvium.inspection.enabled` (default false) and per-topic `alluvium.inspection.allowed` topic config
P1 · M17 · Task · deps: X-OBSX-0001, CFG-F11, CFG-F05 · labels: config, security
Done when: topic config defaults to allowed when the cluster switch is on; both documented; inspection of a disallowed topic returns a permission error.

### [To Do] X-OBSX-0110 Browse API (native gRPC and HTTP): topic, partition, start by offset or timestamp, limit; reads up to the HW, or the LSO with `read_committed` (default)
P1 · M17 · Task · deps: X-OBSX-0109, X-NATIVE-F06
Done when: contract tests; offsets beyond the end return an empty page with the current end offset.

### [To Do] X-OBSX-0111 Authorization: READ on the topic is required; denials return the native permission error and emit `authz_decision`
P1 · M17 · Task · deps: X-OBSX-0110, SEC-F08 · labels: security
Done when: test matrix over literal, prefixed and denied ACLs.

### [To Do] X-OBSX-0112 Redaction: per-topic masking policies from X-SECX-F09 applied before serialization; header values redacted by default
P1 · M17 · Task · deps: X-OBSX-0110, X-SECX-F09 · labels: security
Done when: property test: masked fields never appear in any output encoding; tests for keys, values and headers.

### [To Do] X-OBSX-0113 Rendering: hex, base64, UTF-8 (with invalid sequences escaped) and JSON pretty-printing with size limits
P1 · M17 · Task · deps: X-OBSX-0110
Done when: golden tests per format; invalid JSON falls back to escaped UTF-8.

### [To Do] X-OBSX-0114 Schema-aware rendering: Avro, Protobuf and JSON Schema payloads decoded with the registry schema identified by the schema id
P2 · M17 · Task · deps: X-OBSX-0113, X-SCHEMA-F04, X-SCHEMA-F05, X-SCHEMA-F06
Done when: golden tests per format; unknown schema ids fall back to raw with a reason.

### [To Do] X-OBSX-0115 Fuzz targets for the rendering paths with arbitrary payload bytes and schemas
P1 · M17 · Task · deps: X-OBSX-0113, X-OBSX-0114, TEST-F07 · labels: decoder, test, security
Done when: targets in the per-PR fuzz job; no panic, hang or unbounded allocation.

### [To Do] X-OBSX-0116 Server-side filters: key equality, header presence and timestamp range, with scan bound `alluvium.inspection.max.scan.bytes`
P1 · M17 · Task · deps: X-OBSX-0110
Done when: scans stop at the bound and return a continuation token; tests.

### [To Do] X-OBSX-0117 Limits: maximum records and bytes per request, request rate per principal, timeout; bytes counted against fetch quotas
P1 · M17 · Task · deps: X-OBSX-0110, QUOTA-F03 · labels: security
Done when: over-limit requests rejected with typed errors; throttling observed in a quota test.

### [To Do] X-OBSX-0118 Audit: every inspection request (principal, topic, partition, offset range, filter) written to the audit log
P1 · M17 · Task · deps: X-OBSX-0111, X-SECX-F07 · labels: security
Done when: audit entries asserted in tests; no payload bytes in audit records.

### [To Do] X-OBSX-0119 `alluvium inspect <topic>` CLI with offset, timestamp, format and filter options
P1 · M17 · Task · deps: X-OBSX-0110, X-NATIVE-F14
Done when: CLI tests; payload printing requires the explicit `--show-values` flag.

### [To Do] X-OBSX-0120 Inspection integration tests: authorization, redaction, isolation level, limits and audit together
P1 · M17 · Task · deps: X-OBSX-0112, X-OBSX-0117, X-OBSX-0118 · labels: test
Done when: end-to-end suite against a dev node.

### [To Do] X-OBSX-0121 Message inspection documentation: enabling, permissions, redaction and audit
P1 · M17 · Task · deps: X-OBSX-0120 · labels: docs
Done when: page includes a hardening checklist.

## [To Do] X-OBSX-F10 Web console
P3 · M17 · Feature · deps: X-OBSX-F08
Scope: Built-in, read-mostly web UI.

### [To Do] X-OBSX-0122 Web console technology: server-rendered HTML from the diagnostics listener vs an embedded single-page app, dependency and build-chain impact, CSP
P3 · M17 · Spike · deps: none
Done when: ADR addendum chooses the approach with dependency justification and a no-external-resources rule.

### [To Do] X-OBSX-0123 Serving: console assets embedded in the binary, served on the diagnostics listener under `/console` when `alluvium.console.enabled=true`, strict CSP, no external resources
P3 · M17 · Task · deps: X-OBSX-0122, OBS-F08 · labels: security
Done when: disabled by default (404); response headers include CSP, `X-Content-Type-Options` and `frame-ancestors 'none'`; test.

### [To Do] X-OBSX-0124 Console authentication and sessions: diagnostics authentication (bearer token or mTLS) plus optional OIDC login; CSRF protection on write actions; idle timeout
P3 · M17 · Task · deps: X-OBSX-0123 · labels: security
Done when: session cookies `Secure`, `HttpOnly`, `SameSite=Strict`; CSRF test; expired sessions rejected.

### [To Do] X-OBSX-0125 Console page: cluster overview (nodes, roles, controller, health, under-replicated and offline partitions)
P3 · M17 · Task · deps: X-OBSX-0124
Done when: browser test renders the page against a dev node.

### [To Do] X-OBSX-0126 Console page: topics list and topic detail (partitions, configs, sizes, skew)
P3 · M17 · Task · deps: X-OBSX-0124
Done when: browser test; lists only topics the user can DESCRIBE.

### [To Do] X-OBSX-0127 Console page: partition detail with replica states and the merged flight recorder
P3 · M17 · Task · deps: X-OBSX-0124
Done when: browser test with a lagging follower.

### [To Do] X-OBSX-0128 Console page: consumer groups with lag in records and time
P3 · M17 · Task · deps: X-OBSX-0124, X-OBSX-F03
Done when: browser test with a lagging group.

### [To Do] X-OBSX-0129 Console page: client inventory and slow clients
P3 · M17 · Task · deps: X-OBSX-0124, X-OBSX-F06, X-OBSX-F05
Done when: browser test.

### [To Do] X-OBSX-0130 Console page: lineage graph
P3 · M17 · Task · deps: X-OBSX-0124, X-OBSX-F07
Done when: browser test renders a three-node pipeline.

### [To Do] X-OBSX-0131 Console page: message browser through the inspection API with its authorization and redaction
P3 · M17 · Task · deps: X-OBSX-0124, X-OBSX-F09 · labels: security
Done when: browser test shows masked fields; values hidden until the user requests them.

### [To Do] X-OBSX-0132 Console page: autopilot status, current proposal and blocked reasons (read-only)
P3 · M17 · Task · deps: X-OBSX-0124, X-AUTO-F11
Done when: browser test against a cluster in `recommend` mode.

### [To Do] X-OBSX-0133 Accessibility: WCAG 2.1 AA checks and keyboard navigation in CI
P3 · M17 · Task · deps: X-OBSX-0125 · labels: test
Done when: automated accessibility checks pass on every page.

### [To Do] X-OBSX-0134 Browser end-to-end suite against a dev cluster in CI
P3 · M17 · Task · deps: X-OBSX-0131, X-OBSX-0128 · labels: test
Done when: playwright suite runs headless in CI with a pinned browser.

### [To Do] X-OBSX-0135 Web console documentation: enabling, authentication and exposure guidance
P3 · M17 · Task · deps: X-OBSX-0134 · labels: docs
Done when: page warns against exposing the console without TLS and authentication.

## [To Do] X-OBSX-F11 Observability extras testing and documentation
P0 · M17 · Feature · deps: X-OBSX-F01
Scope: Tests and guide.

### [To Do] X-OBSX-0136 Off by default: with default config no record headers parsed, no lag, detector, inventory or lineage series, no cluster fan-out and no inspection endpoint
P0 · M17 · Task · deps: X-OBSX-F02, X-OBSX-F03, X-OBSX-F04, X-OBSX-F05, X-OBSX-F06, X-OBSX-F08, X-OBSX-F09 · labels: test
Done when: default-config integration test asserts the metrics exposition contains no X-OBSX series, a counting test shows zero header parses, and the new endpoints return 404; every `alluvium.*` enable key reports `false` with source `default`.

### [To Do] X-OBSX-0137 Compatibility and differential suites pass with every X-OBSX feature enabled but no traced records, inspection requests or cluster queries
P0 · M17 · Task · deps: X-OBSX-F02, X-OBSX-F03, X-OBSX-F04, X-OBSX-F05, X-OBSX-F06, X-OBSX-F09, TEST-F08, TEST-F09 · labels: test
Done when: `cargo xtask compat` passes and differential diffs are unchanged versus defaults.

### [To Do] X-OBSX-0138 Overhead gate: all X-OBSX features enabled vs disabled, throughput and p99 within the ADR budget
P0 · M17 · Task · deps: X-OBSX-F03, X-OBSX-F04, TEST-F11 · labels: perf
Done when: macro benchmark comparison enforced in the benchmark job.

### [To Do] X-OBSX-0139 Cardinality soak: 100k groups, 50k client ids and 10k topics keep total series under the documented caps
P0 · M17 · Task · deps: X-OBSX-F03, X-OBSX-F06 · labels: test
Done when: series count asserted at the end of the soak; memory stays within budget.

### [To Do] X-OBSX-0140 Redaction: OBS-F11 tests extended to every new output (record-linked spans, debug endpoints, events, inventory, lineage, inspection)
P0 · M17 · Task · deps: X-OBSX-F06, X-OBSX-F07, X-OBSX-F08, X-OBSX-F09, OBS-F11 · labels: security, test
Done when: tests seed secrets and payload markers and assert none appear in any output.

### [To Do] X-OBSX-0141 Observability extras guide `docs/observability/`: tracing, lag in time, lag metrics, detectors, inventory, lineage, cluster diagnostics, inspection and console
P1 · M17 · Task · deps: X-OBSX-0137 · labels: docs
Done when: guide links each feature page and lists every enable key.

### [To Do] X-OBSX-0142 Migration and opt-in guide: replacing Burrow, kafka-lag-exporter and external UIs, enabling features one at a time, rollback
P1 · M17 · Task · deps: X-OBSX-0141 · labels: docs
Done when: step-by-step enablement with expected metrics and how to verify overhead.

### [To Do] X-OBSX-0143 Alert rules and dashboards for time lag, stalled groups, hot keys and slow clients, with runbooks
P1 · M17 · Task · deps: X-OBSX-F03, OPS-F11 · labels: observability
Done when: rules validated with promtool tests; dashboards checked in.
