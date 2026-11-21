# X-SQL: Built-in SQL and stream processing

> Crates: `alluvium-sql` (new, layer 4) · Kafka counterpart: none (Alluvium enhancement) · Milestones: M16-M17

SQL over topics: batch queries over history and continuous queries producing topics, with windows, joins, state and exactly-once output.

## [To Do] X-SQL-F01 SQL engine design
P0 · M16 · Feature · deps: X-SCHEMA-F02
Scope: Spike and ADR: engine choice and scope.

### [To Do] X-SQL-0001 Spike: SQL engine ADR (engine choice, dialect, batch and streaming scope, state and exactly-once model)
P0 · M16 · Spike · deps: none · ref: new ADR "Built-in SQL engine"
Done when: ADR accepted that compares Apache DataFusion plus Arrow (planning, batch execution) with custom operators for streaming, against embedding a separate engine; decides the SQL dialect and supported statements, event-time and watermark semantics, the state-store model on X-KV indexes with changelog topics, the checkpoint and exactly-once commit protocol on Kafka transactions, where queries run (broker nodes or a dedicated `sql` process role), and crate placement; rejected alternatives recorded.

### [To Do] X-SQL-0002 Dependency review: DataFusion, Arrow and sqlparser with minimal features
P0 · M16 · Task · deps: X-SQL-0001, FND-F03 · labels: security
Done when: crates are added with minimal features (no object-store, no file formats beyond what the ADR needs, no default UDF crates); the review records transitive crate count, `unsafe` use in Arrow kernels, compile-time and binary-size impact, license and maintenance; `cargo deny check` passes; `alluvium-sql` keeps `forbid(unsafe_code)`; `xtask check-layers` restricts these crates to `alluvium-sql`.

### [To Do] X-SQL-0003 SQL threat model
P0 · M16 · Task · deps: X-SQL-0001, SEC-F14 · labels: security, docs
Done when: docs/security/threat-model-sql.md covers resource exhaustion by queries, authorization bypass through catalog access or continuous-query output topics with weaker ACLs, filesystem or network access through engine features, malformed records crashing deserializers, sensitive literals in query logs, and long-running queries outliving their owner's permissions; each threat maps to mitigations and X-SQL test task IDs.

### [To Do] X-SQL-0004 SQL semantics specification
P0 · M16 · Task · deps: X-SQL-0001 · labels: docs
Done when: docs/sql/semantics.md defines NULL handling, type coercion, UTC time handling, event time vs processing time, watermark and lateness semantics, determinism guarantees for batch and continuous queries, and the isolation level each query reads at.

### [To Do] X-SQL-0005 `alluvium.sql.version` finalized feature gating SQL metadata records
P0 · M16 · Task · deps: X-SQL-0001, META-F06, CTRL-F17
Done when: at level 0 (default) the controller rejects X-SQL record types with UNSUPPORTED_VERSION; level 1 enables them; downgrade is refused while continuous queries exist; unit tests per record type.

### [To Do] X-SQL-0006 Register the `alluvium.sql.*` config namespace
P0 · M16 · Task · deps: X-SQL-0001, CFG-F11
Done when: the prefix is in the CFG-F11 registry linked to the SQL ADR; the registry check rejects unregistered keys under it; docs/config.md lists it.

### [To Do] X-SQL-0007 Query execution runtime isolated from the request path
P0 · M16 · Task · deps: X-SQL-0002, SRV-F02
Done when: queries run on a dedicated runtime with `alluvium.sql.worker.threads` threads created and supervised by the server; no query work runs on request-handling workers; startup and shutdown order is documented and tested; a test shows a CPU-bound query does not delay request handling.

## [To Do] X-SQL-F02 Topic catalog
P0 · M16 · Feature · deps: X-SQL-F01
Scope: Topics as tables with schemas from the registry and metadata columns.

### [To Do] X-SQL-0008 Catalog model: catalog, schemas per namespace, tables per topic
P0 · M16 · Task · deps: X-SQL-0002
Done when: catalog `alluvium` exposes schema `public` (and one schema per namespace when X-TENANT is enabled) and one table per topic through DataFusion catalog, schema and table provider implementations; table lookup by name is case-sensitive as documented; unit tests cover listing and lookup.

### [To Do] X-SQL-0009 Metadata columns
P0 · M16 · Task · deps: X-SQL-0008
Done when: every table has `_topic`, `_partition`, `_offset`, `_timestamp`, `_timestamp_type`, `_key` (binary), `_headers` (map of string to binary) and `_leader_epoch`; column types are documented; tests read each column from a fixture topic.

### [To Do] X-SQL-0010 Schema resolution from the registry with evolution
P0 · M16 · Task · deps: X-SQL-0008, X-SCHEMA-F03
Done when: key and value schemas come from `<topic>-key` and `<topic>-value` subjects (latest by default, pinned with a table option); records written with older schema versions are read through the reader schema with Avro resolution rules; tests cover added, removed and defaulted fields.

### [To Do] X-SQL-0011 Avro to Arrow type mapping
P0 · M16 · Task · deps: X-SQL-0010, X-SCHEMA-F04
Done when: null, boolean, int, long, float, double, bytes, string, record (struct), enum (dictionary), array (list), map (map), unions (nullable or dense union), fixed and logical types (decimal, uuid, date, time-millis and -micros, timestamp-millis and -micros, local-timestamp, duration) map as documented in docs/sql/types.md; a property test round-trips random Avro values to Arrow and back.

### [To Do] X-SQL-0012 Protobuf to Arrow type mapping
P0 · M16 · Task · deps: X-SQL-0010, X-SCHEMA-F05
Done when: scalar types, enums, nested messages, repeated fields, maps, oneofs and well-known types (Timestamp, Duration, wrappers) map as documented; tests cover each kind including proto3 presence.

### [To Do] X-SQL-0013 JSON Schema to Arrow type mapping
P0 · M16 · Task · deps: X-SQL-0010, X-SCHEMA-F06
Done when: types, `format: date-time` and `date`, nullable type arrays, nested objects, arrays and `additionalProperties` (map or JSON text) map as documented; unsupported constructs make the column JSON text with a documented rule; tests per construct.

### [To Do] X-SQL-0014 Schemaless topics and declared table formats
P2 · M16 · Task · deps: X-SQL-0008
Done when: topics without registered schemas expose key and value as binary; `CREATE TABLE … WITH (format = 'json', columns …)` declares a schema for schemaless JSON topics stored as catalog metadata; tests cover both.

### [To Do] X-SQL-0015 Record to Arrow deserialization with Confluent wire format and error policy
P0 · M16 · Task · deps: X-SQL-0011, X-SQL-0012, X-SQL-0013 · labels: security, decoder
Done when: values with magic byte 0 and a schema id decode into Arrow builders with bounded allocation; `alluvium.sql.deserialization.error.policy` (`fail`, `skip`, `null`) handles malformed records and counts them; fuzz target `fuzz_sql_record_decode` runs clean.

### [To Do] X-SQL-0016 Catalog authorization
P0 · M16 · Task · deps: X-SQL-0008, SEC-F08 · labels: security
Done when: tables are listed only for topics the principal can DESCRIBE; selecting requires READ; schema details require registry subject read permission; tests show hidden topics are absent from `SHOW TABLES` and `information_schema`.

### [To Do] X-SQL-0017 `SHOW TABLES`, `DESCRIBE` and `information_schema`
P0 · M16 · Task · deps: X-SQL-0016, X-SQL-0009
Done when: the statements return tables, columns and types consistent with the mapping docs; tests compare output with fixtures.

### [To Do] X-SQL-0018 Catalog caching and invalidation
P0 · M16 · Task · deps: X-SQL-0010
Done when: table schemas are cached with a bound (`alluvium.sql.catalog.cache.max.entries`) and invalidated on topic metadata changes and new schema versions; a test registers a new schema version and sees it on the next query.

## [To Do] X-SQL-F03 Batch queries
P0 · M16 · Feature · deps: X-SQL-F02
Scope: Queries over topic history with time and partition pruning.

### [To Do] X-SQL-0019 Topic scan operator with bounded memory
P0 · M16 · Task · deps: SRV-F10
Done when: scans read partitions through the service-layer read path into Arrow batches of `alluvium.sql.batch.rows` rows, with per-partition read-ahead bounded by `alluvium.sql.scan.buffer.bytes`; parallelism is per partition; tests scan multi-partition fixtures.

### [To Do] X-SQL-0020 Snapshot consistency: per-partition end offsets captured at query start
P0 · M16 · Task · deps: X-SQL-0019, REPL-F13
Done when: a query reads each partition up to the HW (or LSO when `alluvium.sql.isolation.level=read_committed`) captured at start, so repeated runs over unchanged data return identical results; tests produce during a scan and show stable results.

### [To Do] X-SQL-0021 Partition pruning
P0 · M16 · Task · deps: X-SQL-0019
Done when: predicates `_partition = n` and `_partition IN (…)` restrict scans; key-equality predicates map to a partition with murmur2 only when the table option `partitioner = 'default'` is set; EXPLAIN shows pruned partitions; tests compare results with and without pruning.

### [To Do] X-SQL-0022 Offset range pruning
P0 · M16 · Task · deps: X-SQL-0019
Done when: `_offset` range predicates set start and end offsets per partition; out-of-range bounds are clamped to log start and end; tests cover open and closed ranges.

### [To Do] X-SQL-0023 Time pruning through the time index
P0 · M16 · Task · deps: X-SQL-0019, REPL-F14
Done when: `_timestamp` range predicates resolve start offsets with ListOffsets-by-timestamp semantics and stop scanning past the upper bound per the semantics doc (CreateTime may be non-monotonic; the rule is documented); tests compare with full scans.

### [To Do] X-SQL-0024 Projection pushdown
P0 · M16 · Task · deps: X-SQL-0019, X-SQL-0015
Done when: only referenced columns are built; metadata-only queries skip value decoding entirely; benchmarks show the saving.

### [To Do] X-SQL-0025 Scans over tiered data
P1 · M16 · Task · deps: X-SQL-0019, TIER-F04
Done when: scans read remote segments through TIER-F04 with a separate throttle `alluvium.sql.remote.scan.max.bytes.per.second`; tests cover a query spanning local and remote data.

### [To Do] X-SQL-0026 Statement and function allow-list
P0 · M16 · Task · deps: none · labels: security
Done when: only SELECT, EXPLAIN, SHOW, DESCRIBE and the continuous-query DDL are accepted; `CREATE EXTERNAL TABLE`, `COPY`, file and object-store access, UDF registration and session settings outside an allow-list are rejected with a clear error; tests attempt each forbidden feature.

### [To Do] X-SQL-0027 Scalar and aggregate function library documentation and tests
P0 · M16 · Task · deps: X-SQL-0026
Done when: the allowed string, math, conditional, date/time, JSON and aggregate functions (count, sum, avg, min, max, approx_distinct, approx_percentile) are listed in docs/sql/functions.md with examples, each covered by a sqllogictest case.

### [To Do] X-SQL-0028 Kafka-specific functions
P0 · M16 · Task · deps: X-SQL-0027
Done when: `header(_headers, name)`, `key_string(_key)`, `kafka_partition(key, partitions)` (murmur2) and `from_avro`/`from_json` for ad-hoc decoding are implemented and documented; tests compare `kafka_partition` with the Java default partitioner on fixture keys.

### [To Do] X-SQL-0029 Masking of SQL results
P0 · M16 · Task · deps: X-SQL-0019, X-SECX-F09 · labels: security
Done when: result columns from topics covered by masking policies are masked through the X-SECX masking engine unless the principal has `ReadUnmasked`; masking applies after all expressions so derived values cannot reveal raw data (aggregates on masked fields operate on masked values, as documented); tests cover direct selects, expressions and aggregates.

### [To Do] X-SQL-0030 Result streaming with backpressure and early termination
P0 · M16 · Task · deps: X-SQL-0019
Done when: results stream to the caller in Arrow batches; a slow client pauses execution within a bounded buffer; LIMIT stops scans early; tests show memory stays bounded with a slow reader.

### [To Do] X-SQL-0031 SQL correctness suite for batch queries
P0 · M16 · Task · deps: X-SQL-0027, X-SQL-0020 · labels: test
Done when: sqllogictest files cover projections, filters, joins over tables, aggregations, subqueries, CTEs, ordering, NULL semantics and every type mapping over fixture topics; the suite runs in CI.

### [To Do] X-SQL-0032 Batch query benchmarks
P0 · M16 · Task · deps: X-SQL-0019, TEST-F11 · labels: perf
Done when: benchmarks report rows/s and bytes/s for full scans, pruned scans and aggregations per format and the impact on concurrent produce latency; results and budgets are in docs/benchmarks/sql.md.

## [To Do] X-SQL-F04 Continuous queries
P0 · M16 · Feature · deps: X-SQL-F02
Scope: Streaming SQL writing to output topics.

### [To Do] X-SQL-0033 Continuous query DDL and management statements
P0 · M16 · Task · deps: X-SQL-0026
Done when: `CREATE CONTINUOUS QUERY <name> AS INSERT INTO <topic> SELECT …`, `DROP`, `PAUSE`, `RESUME` and `SHOW QUERIES` parse with the syntax chosen in the ADR; parse errors carry positions; tests per statement.

### [To Do] X-SQL-0034 Continuous query definition records
P0 · M16 · Task · deps: X-SQL-0033, X-SQL-0005, META-F02
Done when: `SqlQueryRecord` (id, name, SQL text, plan version, owner principal, state, output topic, settings) and its removal record are validated (the query plans successfully, the output topic exists or is created per settings), replayed and snapshotted; the count is bounded by `alluvium.sql.max.continuous.queries`; tests cover validation errors.

### [To Do] X-SQL-0035 Continuous query lifecycle state machine
P0 · M16 · Task · deps: X-SQL-0034
Done when: states Created → Starting → Running ↔ Paused → Failed or Stopped follow transitions with reasons (`started`, `paused_by_user`, `owner_permission_revoked`, `state_limit_exceeded`, `operator_error`, `dropped`); transitions go to the query's flight recorder; a property test checks only legal transitions occur.

### [To Do] X-SQL-0036 Streaming planner: logical plan to operator graph
P0 · M16 · Task · deps: X-SQL-0033
Done when: filters, projections, windowed aggregations, joins and unions become streaming operators; unbounded ORDER BY, LIMIT and non-windowed aggregates without changelog output are rejected with errors explaining why; EXPLAIN shows the operator graph; tests per accepted and rejected shape.

### [To Do] X-SQL-0037 Streaming source operator from committed positions
P0 · M16 · Task · deps: X-SQL-0036, X-SQL-0019
Done when: each input partition is read from the query's committed position with bounded read-ahead; new partitions on inputs are picked up; tests cover partition additions and empty inputs.

### [To Do] X-SQL-0038 Task assignment across SQL-capable nodes
P0 · M16 · Task · deps: X-SQL-0037, GRP-F05
Done when: query tasks (one per input partition group) are assigned to nodes through an internal group per query using the consumer group protocol, preferring nodes that lead the input partitions; rebalances on node join and leave move tasks; a simulation test shows exactly one active owner per task.

### [To Do] X-SQL-0039 Sink operator writing to the output topic with an idempotent producer
P0 · M16 · Task · deps: X-SQL-0036
Done when: outputs are written with an idempotent internal producer authenticated as the query's owner; keys come from the SELECT's designated key columns; sink errors move the query to Failed with reason; tests cover retries and authorization failures.

### [To Do] X-SQL-0040 Output schema derivation and registry registration
P0 · M16 · Task · deps: X-SQL-0039, X-SCHEMA-F07
Done when: the output Avro schema is derived from the plan and registered under `<output-topic>-value` with a compatibility check; incompatible changes fail query creation; a Java consumer with the Confluent Avro deserializer reads the output.

### [To Do] X-SQL-0041 Source offset checkpointing for at-least-once processing
P0 · M16 · Task · deps: X-SQL-0037, X-SQL-0039
Done when: source positions are committed to the query's group after outputs are acknowledged, every `alluvium.sql.commit.interval.ms`; restart resumes from the committed positions; crash tests show no lost input and bounded duplicates.

### [To Do] X-SQL-0042 Bounded channels between operators
P0 · M16 · Task · deps: X-SQL-0036
Done when: operators exchange batches over bounded channels (`alluvium.sql.operator.channel.batches`); a slow sink pauses sources; channel depth and capacity are exported; a test throttles the sink and shows bounded memory.

### [To Do] X-SQL-0043 Continuous query authorization and owner revocation
P0 · M16 · Task · deps: X-SQL-0034, SEC-F08 · labels: security
Done when: creating a query requires READ on inputs, WRITE on the output and CREATE on the cluster; queries run as their owner; revoking any of those permissions moves the query to Failed with reason `owner_permission_revoked` within one metadata propagation; tests cover each permission.

### [To Do] X-SQL-0044 Story: stateless continuous query end to end
P0 · M16 · Story · deps: X-SQL-0038, X-SQL-0040, X-SQL-0041
Done when: a user creates a filter-and-project continuous query with the CLI, produces to the input with the Java producer and reads matching records from the output with the Java consumer; the test runs in the compat harness.

### [To Do] X-SQL-0045 Query plan versioning and redeploy with carried positions
P2 · M16 · Task · deps: X-SQL-0034, X-SQL-0041
Done when: `CREATE OR REPLACE CONTINUOUS QUERY` keeps source positions when the new plan is position-compatible per documented rules, and otherwise requires `RESET POSITIONS`; tests cover both.

### [To Do] X-SQL-0046 Continuous query metrics and events
P0 · M16 · Task · deps: X-SQL-0035, OBS-F04, OBS-F07
Done when: `alluvium_sql_continuous_query_state{state}`, `_records_in_total{query}`, `_records_out_total{query}` and `_input_lag_records{query}` exist with bounded labels; events `continuous_query_state_changed` carry reasons.

## [To Do] X-SQL-F05 Windowing
P0 · M16 · Feature · deps: X-SQL-F04
Scope: Tumbling, hopping and session windows; watermarks.

### [To Do] X-SQL-0047 Event-time column and watermark declaration
P0 · M16 · Task · deps: none
Done when: queries use `_timestamp` or a declared column as event time with `WATERMARK FOR <col> AS <col> - INTERVAL …`; invalid declarations fail planning; tests cover both.

### [To Do] X-SQL-0048 Watermark generation per partition and across partitions
P0 · M16 · Task · deps: X-SQL-0047
Done when: watermarks are tracked per input partition and combined as the minimum; partitions idle for `alluvium.sql.watermark.idle.timeout.ms` are excluded until they receive data; property tests show the combined watermark never decreases.

### [To Do] X-SQL-0049 Tumbling windows: `TUMBLE(ts, size)`
P0 · M16 · Task · deps: X-SQL-0048, X-SQL-F07 · ref: streams/src/main/java/org/apache/kafka/streams/kstream/TimeWindows.java ofSizeAndGrace @4.4.0
Done when: records are assigned to aligned, non-overlapping windows; results include `window_start` and `window_end`; a property test compares results with a batch computation over the same input.

### [To Do] X-SQL-0050 Hopping windows: `HOP(ts, size, advance)`
P0 · M16 · Task · deps: X-SQL-0049 · ref: streams/src/main/java/org/apache/kafka/streams/kstream/TimeWindows.java advanceBy @4.4.0
Done when: each record belongs to every window containing it (size / advance windows); invalid parameters (advance > size or zero) fail planning; property test against batch computation.

### [To Do] X-SQL-0051 Session windows: `SESSION(ts, gap)` with merging
P0 · M16 · Task · deps: X-SQL-0048, X-SQL-F07 · ref: streams/src/main/java/org/apache/kafka/streams/kstream/SessionWindows.java ofInactivityGapAndGrace @4.4.0
Done when: sessions per key merge when records fall within the gap, including out-of-order records that bridge two sessions; merged sessions retract previous results in emit-changes mode; property test against batch computation.

### [To Do] X-SQL-0052 Sliding windows
P2 · M16 · Task · deps: X-SQL-0049 · ref: streams/src/main/java/org/apache/kafka/streams/kstream/SlidingWindows.java @4.4.0
Done when: `SLIDE(ts, size)` windows follow Kafka Streams sliding-window semantics (windows defined by record time differences); property test against batch computation.

### [To Do] X-SQL-0053 Grace period and late records
P0 · M16 · Task · deps: X-SQL-0049, X-SQL-0051
Done when: `GRACE <interval>` keeps windows open after the watermark passes their end; later records are dropped and counted, or routed to a late-records topic when `alluvium.sql.late.records.topic` is set; tests cover on-time, within-grace and late records.

### [To Do] X-SQL-0054 Emit modes: final results on window close and emit-changes
P0 · M16 · Task · deps: X-SQL-0053
Done when: `EMIT FINAL` writes one result per window after grace; `EMIT CHANGES` writes updates keyed by (group key, window) so compacted outputs hold the latest value; tests verify output sequences for both modes.

### [To Do] X-SQL-0055 Window state retention and expiry
P0 · M16 · Task · deps: X-SQL-0053
Done when: window state is deleted after window end plus grace, and session state after the gap plus grace; state size is bounded under steady load in a long-running simulation.

### [To Do] X-SQL-0056 Windowing under out-of-order input and crashes
P0 · M16 · Task · deps: X-SQL-0054, TEST-F06 · labels: test
Done when: a seeded simulation with shuffled timestamps, idle partitions and task crashes asserts final windowed results equal the batch reference computed over records that were not late.

## [To Do] X-SQL-F06 Joins
P1 · M16 · Feature · deps: X-SQL-F04
Scope: Stream-stream and stream-table joins.

### [To Do] X-SQL-0057 Co-partitioning check and automatic repartitioning
P1 · M16 · Task · deps: none
Done when: the planner checks that joined inputs have equal partition counts and keys partitioned by the default partitioner; otherwise it inserts a repartition through internal topic `_alluvium_sql_<query>_repartition_<n>` with matching partition count; EXPLAIN shows repartitions; tests cover both paths.

### [To Do] X-SQL-0058 Repartition operator and internal topic lifecycle
P1 · M16 · Task · deps: X-SQL-0057
Done when: the repartition sink and source preserve per-key order; internal topics are created with the query, protected from client writes and deleted on DROP; tests cover lifecycle and authorization.

### [To Do] X-SQL-0059 Stream-stream inner interval join
P1 · M16 · Task · deps: X-SQL-0057, X-SQL-F07 · ref: streams/src/main/java/org/apache/kafka/streams/kstream/JoinWindows.java ofTimeDifferenceAndGrace @4.4.0
Done when: `a JOIN b ON a.k = b.k AND b.ts BETWEEN a.ts - INTERVAL x AND a.ts + INTERVAL y` buffers both sides for the window plus grace and emits matches; a property test compares with a batch join.

### [To Do] X-SQL-0060 Stream-stream left and full outer joins
P1 · M16 · Task · deps: X-SQL-0059
Done when: unmatched records emit null-padded results after their window and grace close; property tests against batch outer joins.

### [To Do] X-SQL-0061 Table materialization from compacted changelog topics
P1 · M16 · Task · deps: X-SQL-F07
Done when: a topic declared as a table (`CREATE TABLE … WITH (kind = 'changelog')`) materializes the latest value per key; tombstones delete keys; restoration reads from the start; tests cover updates and deletes.

### [To Do] X-SQL-0062 Stream-table inner and left joins
P1 · M16 · Task · deps: X-SQL-0061, X-SQL-0057
Done when: each stream record joins with the table value current at processing time (or at event time when the ADR's temporal option is enabled); left joins emit nulls for missing keys; property tests against reference semantics.

### [To Do] X-SQL-0063 Foreign-key table-table joins
P2 · M16 · Task · deps: X-SQL-0061
Done when: table-table joins on a foreign key extracted from the left value follow Kafka Streams foreign-key join semantics (KIP-213), including updates on either side; property tests against batch joins.

### [To Do] X-SQL-0064 Join state bounds
P1 · M16 · Task · deps: X-SQL-0059, X-SQL-0061
Done when: join buffers and tables count toward `alluvium.sql.state.max.bytes` per query; exceeding it fails the query with reason `state_limit_exceeded`; tests trigger the limit.

### [To Do] X-SQL-0065 Joins under out-of-order input and crashes
P1 · M16 · Task · deps: X-SQL-0060, X-SQL-0062, TEST-F06 · labels: test
Done when: a seeded simulation with reordering and task crashes asserts join outputs equal the batch reference for records within grace.

## [To Do] X-SQL-F07 State stores
P0 · M16 · Feature · deps: X-SQL-F04, X-KV-F02
Scope: Query state backed by key indexes and changelogs.

### [To Do] X-SQL-0066 State store interface: key-value, window and session stores
P0 · M16 · Task · deps: none
Done when: sans-I/O store traits support get, put, delete and ordered range scans, plus window fetch by (key, time range) and session find-and-merge; an in-memory implementation passes a shared conformance test suite.

### [To Do] X-SQL-0067 Persistent store backend on the key index engine
P0 · M16 · Task · deps: X-SQL-0066
Done when: stores persist through the X-KV index engine with keys encoded to preserve ordering for range and window scans; the backend passes the conformance suite; crash-consistency tests with the simulated disk recover to the last flushed state.

### [To Do] X-SQL-0068 Changelog topics for every store
P0 · M16 · Task · deps: X-SQL-0067
Done when: each mutation is written to the compacted topic `_alluvium_sql_<query>_<store>_changelog` (window stores use time-bounded retention); topics are created with the query, protected from clients and deleted on DROP; tests cover writes and deletes.

### [To Do] X-SQL-0069 State restoration from changelogs
P0 · M16 · Task · deps: X-SQL-0068
Done when: on task move or restart the store restores from its last checkpointed changelog offset to the changelog end before processing resumes, throttled by `alluvium.sql.restore.max.bytes.per.second`; restore progress is reported; tests cover full and incremental restores.

### [To Do] X-SQL-0070 Commit protocol for at-least-once processing
P0 · M16 · Task · deps: X-SQL-0068
Done when: on each commit interval, stores flush, changelog writes are acknowledged, a checkpoint file records changelog offsets, then source positions are committed; crash tests at each step show restored state is consistent with committed positions (no lost updates; duplicates only from replayed input).

### [To Do] X-SQL-0071 State size limits per query
P0 · M16 · Task · deps: X-SQL-0067
Done when: store sizes are tracked and enforced against `alluvium.sql.state.max.bytes`; exceeding the limit moves the query to Failed with reason `state_limit_exceeded`; a metric shows usage.

### [To Do] X-SQL-0072 Standby replicas for faster failover
P2 · M16 · Task · deps: X-SQL-0069
Done when: `alluvium.sql.standby.replicas` keeps warm copies on other nodes by tailing changelogs; failover to a standby restores only the remaining tail; a simulation shows reduced restore time.

### [To Do] X-SQL-0073 Point lookups into continuous query state
P2 · M16 · Task · deps: X-SQL-0067, X-KV-F04
Done when: `SELECT … FROM <query>.<store> WHERE key = …` returns the current value from the owning node; reads are authorized like the query's output topic; tests cover lookups during rebalances.

### [To Do] X-SQL-0074 State store metrics
P0 · M16 · Task · deps: X-SQL-0067, OBS-F04
Done when: `alluvium_sql_state_bytes{query}`, `_restore_seconds{query}` and `_changelog_lag_records{query}` exist with bounded labels; tests assert values.

## [To Do] X-SQL-F08 Exactly-once output
P1 · M16 · Feature · deps: X-SQL-F04, TXN-F08
Scope: Transactional output of continuous queries.

### [To Do] X-SQL-0075 Transactional producer per task with fencing
P1 · M16 · Task · deps: X-SQL-0070
Done when: with `processing.guarantee = 'exactly_once'` each task uses transactional id `alluvium-sql-<query-id>-<task>`; task moves fence the previous owner by epoch bump; tests cover fencing during rebalances.

### [To Do] X-SQL-0076 Atomic commit of outputs, changelogs and source positions
P1 · M16 · Task · deps: X-SQL-0075, TXN-F07 · ref: KIP-447
Done when: each commit interval sends outputs, changelog records and source offsets (TxnOffsetCommit to the query's group) in one transaction; failures abort; tests inspect committed and aborted transactions.

### [To Do] X-SQL-0077 State consistency on abort and crash
P1 · M16 · Task · deps: X-SQL-0076
Done when: after an abort or crash a task discards local state newer than the last committed changelog offsets and restores from changelogs read with read_committed; tests crash at every commit step.

### [To Do] X-SQL-0078 Read-committed inputs for exactly-once queries
P1 · M16 · Task · deps: X-SQL-0075
Done when: exactly-once queries read inputs and changelogs with read_committed up to the LSO; tests show aborted input records never affect outputs.

### [To Do] X-SQL-0079 Exactly-once configuration validation
P1 · M16 · Task · deps: X-SQL-0076
Done when: query settings validate that the commit interval is below `transaction.max.timeout.ms` and that output topics are not compacted-only when the plan emits non-keyed records; invalid combinations fail query creation with a clear message.

### [To Do] X-SQL-0080 Exactly-once verification under crashes and rebalances
P1 · M16 · Task · deps: X-SQL-0077, X-SQL-0078, TEST-F06 · labels: test
Done when: a seeded simulation with task crashes, node failures and rebalances asserts that read_committed consumers of the output see exactly the reference results once; a compat test reads the output with a Java read_committed consumer.

### [To Do] X-SQL-0081 Exactly-once overhead benchmark
P2 · M16 · Task · deps: X-SQL-0076, TEST-F11 · labels: perf
Done when: benchmarks compare throughput and end-to-end latency of exactly-once and at-least-once queries at several commit intervals; results are in docs/benchmarks/sql.md.

### [To Do] X-SQL-0082 Exactly-once failure handling and events
P1 · M16 · Task · deps: X-SQL-0075, OBS-F07
Done when: producer fencing and transaction aborts restart the task with events `continuous_query_task_fenced` and `continuous_query_transaction_aborted` carrying reasons; tests assert the events.

## [To Do] X-SQL-F09 Query API
P1 · M16 · Feature · deps: X-SQL-F03
Scope: Native query endpoint; PostgreSQL wire protocol later.

### [To Do] X-SQL-0083 Native query endpoint: gRPC streaming and HTTP
P1 · M16 · Task · deps: X-NATIVE-F03, X-NATIVE-F04
Done when: `Sql.Execute` streams results as Arrow IPC over gRPC; HTTP returns NDJSON or Arrow IPC by content negotiation; both carry query id, schema and completion status; tests cover each encoding.

### [To Do] X-SQL-0084 JSON result encoding rules
P1 · M16 · Task · deps: X-SQL-0083
Done when: 64-bit integers and decimals are encoded as strings, timestamps as RFC 3339 UTC, binary as base64, maps and structs as objects, as documented in docs/sql/api.md; tests per type.

### [To Do] X-SQL-0085 Cancellation endpoint and cancellation on client disconnect
P1 · M16 · Task · deps: X-SQL-0083
Done when: `Sql.Cancel` and HTTP DELETE on the query stop execution within `alluvium.sql.cancel.max.delay.ms` and free memory; a client disconnect cancels its query; tests verify memory is released.

### [To Do] X-SQL-0086 Continuous query management endpoints and `alluvium sql` CLI
P1 · M16 · Task · deps: X-SQL-0083, X-SQL-0035, CLI-F01
Done when: native endpoints and `alluvium sql create|list|describe|pause|resume|drop` manage continuous queries; describe shows state, lag, task placement and last error; tests per operation.

### [To Do] X-SQL-0087 Authentication and caller identity for queries
P1 · M16 · Task · deps: X-SQL-0083, X-NATIVE-F09 · labels: security
Done when: queries run as the authenticated caller with native authentication; unauthenticated requests fail; tests cover mTLS, bearer tokens and API keys.

### [To Do] X-SQL-0088 SQL errors mapped to the native error model
P1 · M16 · Task · deps: X-SQL-0083, X-NATIVE-F10
Done when: parse errors (with line and column), planning errors, authorization failures, resource-limit errors and execution errors map to gRPC status details and problem+json types through one exhaustive mapping; tests per error class.

### [To Do] X-SQL-0089 Prepared statements with parameters
P2 · M16 · Task · deps: X-SQL-0083
Done when: parameterized queries (`$1`, `$2`) are planned once and executed with typed parameters; type mismatches return errors; tests cover reuse and mismatch.

### [To Do] X-SQL-0090 Interactive shell: `alluvium sql shell`
P2 · M16 · Task · deps: X-SQL-0083, CLI-F01
Done when: the shell supports multi-line input, history, tabular and JSON output and cancellation with Ctrl-C; a test drives it through a pseudo-terminal.

### [To Do] X-SQL-0091 Query API reference
P1 · M16 · Task · deps: X-SQL-0083, X-SQL-0088 · labels: docs
Done when: docs/sql/api.md documents the gRPC service and HTTP endpoints with examples, encodings, errors and limits; the reference is generated from the IDL where possible.

### [To Do] X-SQL-0092 Dependency review: PostgreSQL wire protocol crate
P3 · M17 · Task · deps: X-SQL-0083, FND-F03 · labels: security
Done when: the review covers the candidate crate (for example `pgwire`) or a minimal own implementation: transitive count, `unsafe`, rustls-only TLS, license and maintenance; `cargo deny check` passes.

### [To Do] X-SQL-0093 PostgreSQL wire protocol: startup, TLS and authentication
P3 · M17 · Task · deps: X-SQL-0092 · labels: security
Done when: a pgwire listener (off unless `alluvium.sql.pgwire.listeners` is set) handles SSLRequest with rustls, startup parameters and SCRAM-SHA-256 or TLS client-certificate authentication mapped to Alluvium principals; malformed startup messages fail cleanly; fuzz target `fuzz_pgwire_frontend` runs clean.

### [To Do] X-SQL-0094 PostgreSQL simple and extended query protocols with type OID mapping
P3 · M17 · Task · deps: X-SQL-0093
Done when: Query, Parse, Bind, Describe, Execute and Sync work; Arrow types map to PostgreSQL type OIDs as documented; cancellation via CancelRequest works; tests use the PostgreSQL JDBC driver and `psql` in the compat harness.

## [To Do] X-SQL-F10 Query resource isolation
P0 · M16 · Feature · deps: X-SQL-F03
Scope: Quotas, cancellation, memory limits.

### [To Do] X-SQL-0095 Memory limits per query and a global memory pool
P0 · M16 · Task · deps: none
Done when: each query gets a memory reservation from a global pool sized `alluvium.sql.total.memory.bytes`, capped at `alluvium.sql.query.max.memory.bytes`; exceeding it fails the query with a resources-exhausted error (spilling is disabled unless the ADR enables it); tests trigger the limit with a large aggregation.

### [To Do] X-SQL-0096 CPU-time and wall-clock limits
P0 · M16 · Task · deps: X-SQL-0095
Done when: queries stop with typed errors after `alluvium.sql.query.timeout.ms` wall time or `alluvium.sql.query.max.cpu.ms` CPU time; tests use the simulated clock where possible and a CPU-bound fixture otherwise.

### [To Do] X-SQL-0097 Scan limits and fetch-quota accounting
P0 · M16 · Task · deps: X-SQL-0095, QUOTA-F03 · labels: security
Done when: scanned bytes are capped by `alluvium.sql.query.max.scan.bytes` and charged to the principal's fetch quota, so SQL cannot bypass quotas; tests show throttling and the scan cap.

### [To Do] X-SQL-0098 Concurrency limits and admission control
P0 · M16 · Task · deps: X-SQL-0095
Done when: running queries are capped globally (`alluvium.sql.max.concurrent.queries`) and per principal (`.max.concurrent.queries.per.principal`); excess queries wait in a bounded queue up to `alluvium.sql.admission.timeout.ms` and then fail with a typed error; queue metrics are exported; tests cover both limits.

### [To Do] X-SQL-0099 Cooperative cancellation reaching every operator
P0 · M16 · Task · deps: X-SQL-0095
Done when: cancellation reaches every running operator within `alluvium.sql.cancel.max.delay.ms`; a test cancels queries at random points and checks memory reservations return to zero.

### [To Do] X-SQL-0100 Continuous query budgets and tenant limits
P0 · M16 · Task · deps: X-SQL-0095, X-TENANT-F04
Done when: continuous queries have CPU-share and state budgets; namespaces have `max.continuous.queries` and `max.sql.memory.bytes`; violations fail creation or move queries to Failed with reasons; tests per limit.

### [To Do] X-SQL-0101 SQL noisy-neighbor test against broker hot paths
P0 · M16 · Task · deps: X-SQL-0096, X-SQL-0097, TEST-F11 · labels: test, nfr
Done when: a benchmark runs heavy scans and aggregations while producing and fetching on the same nodes and asserts produce and fetch p99 stay within the budget from the ADR.

### [To Do] X-SQL-0102 Resource isolation metrics and events
P0 · M16 · Task · deps: X-SQL-0098, OBS-F04
Done when: `alluvium_sql_memory_reserved_bytes`, `_queries_rejected_total{reason}`, `_queries_cancelled_total{reason}` and `_admission_queue_depth` exist; event `sql_query_limit_exceeded` carries the limit kind.

## [To Do] X-SQL-F11 SQL observability, testing and documentation
P0 · M16 · Feature · deps: X-SQL-F03
Scope: Metrics, correctness tests, guide.

### [To Do] X-SQL-0103 SQL is off by default
P0 · M16 · Task · deps: X-SQL-F04, X-SQL-F09, X-SQL-F10 · labels: test
Done when: test `sql_off_by_default` starts a node with default config and asserts `alluvium.sql.version` is 0, no SQL runtime threads exist, no SQL endpoints or pgwire listeners are bound and no SQL internal topics exist.

### [To Do] X-SQL-0104 Kafka compatibility and differential suites pass with SQL enabled but unused
P0 · M16 · Task · deps: X-SQL-0103, TEST-F08, TEST-F09 · labels: test
Done when: the full compat and differential suites run in CI with feature level 1, the SQL runtime started and endpoints bound but no queries; all pass with no new ignore rules.

### [To Do] X-SQL-0105 `EXPLAIN` and `EXPLAIN ANALYZE` with operator metrics
P0 · M16 · Task · deps: X-SQL-F04
Done when: EXPLAIN shows the logical and physical (or streaming) plan with pruning decisions; EXPLAIN ANALYZE adds per-operator rows, bytes, time and memory; tests compare against fixtures.

### [To Do] X-SQL-0106 Structured query log with literal redaction
P0 · M16 · Task · deps: X-SQL-F04, OBS-F07 · labels: security
Done when: each query emits start and end events with query id, principal, duration, rows and bytes; SQL text is logged only when `alluvium.sql.log.query.text=true` and then with literals replaced by placeholders; a redaction test asserts no literal values appear by default.

### [To Do] X-SQL-0107 `/debug/sql` diagnostics and flight recorder
P0 · M16 · Task · deps: X-SQL-F04, OBS-F08
Done when: `/debug/sql/queries` lists running batch and continuous queries with state, memory and the continuous query flight recorders; tests assert the output.

### [To Do] X-SQL-0108 SQL metrics catalog
P0 · M16 · Task · deps: X-SQL-F04, OBS-F04
Done when: `alluvium_sql_queries_total{kind,outcome}`, `_query_seconds{kind}`, `_rows_scanned_total`, `_bytes_scanned_total` and watermark lag gauges exist and are documented in docs/metrics.md with cardinality bounds.

### [To Do] X-SQL-0109 Differential correctness against a reference engine
P0 · M16 · Task · deps: X-SQL-0031 · labels: test
Done when: a test-only harness runs the batch sqllogictest corpus against Alluvium and a reference engine (DuckDB, test-only dependency with its own review) over identical data and reports differences beyond documented dialect differences.

### [To Do] X-SQL-0110 Property: continuous results equal batch results
P0 · M16 · Task · deps: X-SQL-F05, X-SQL-F06, X-SQL-F08 · labels: test
Done when: for random inputs and a corpus of windowed and joined queries, the final output of each continuous query equals the batch query over the same data, under simulation with crashes and rebalances.

### [To Do] X-SQL-0111 Fuzzing the SQL parser and planner
P0 · M16 · Task · deps: X-SQL-F04, TEST-F07 · labels: security, test, decoder
Done when: `fuzz_sql_plan` feeds arbitrary SQL text through parsing and planning (including continuous-query DDL) and runs clean with no panic and bounded memory; it runs nightly with a committed corpus.

### [To Do] X-SQL-0112 Nexmark streaming benchmark
P1 · M16 · Task · deps: X-SQL-F05, X-SQL-F06, TEST-F11 · labels: perf
Done when: a Nexmark generator writes person, auction and bid topics; queries q0–q8 run as continuous queries; throughput and latency results with hardware and versions are in docs/benchmarks/sql.md with regression tracking.

### [To Do] X-SQL-0113 SQL reference documentation
P0 · M16 · Task · deps: X-SQL-F04, X-SQL-F05, X-SQL-F06 · labels: docs
Done when: docs/sql/reference.md covers every statement, clause, window function, join type, type mapping and function, with an example per item, linking the semantics specification.

### [To Do] X-SQL-0114 SQL guide, opt-in and migration from ksqlDB and Kafka Streams
P0 · M16 · Task · deps: X-SQL-0113, X-SQL-F08 · labels: docs
Done when: docs/sql/guide.md covers enabling the feature level, batch exploration, continuous queries, windows, joins, state, exactly-once, limits and monitoring; a migration section maps common ksqlDB statements and Kafka Streams DSL operations to Alluvium SQL with worked examples.
