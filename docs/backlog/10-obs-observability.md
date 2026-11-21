# OBS: Observability

> Crates: `alluvium-observability` (layer 3) · Kafka counterpart: `kafka.metrics`, log4j configuration, JMX · Milestones: M2-M9

Logs, spans, metrics, flight recorders, event catalogs and the diagnostics endpoint, as specified in docs/standards/debuggability.md.

## [To Do] OBS-F01 Tracing setup
P0 · M2 · Feature · deps: FND-F01
Scope: Subscriber setup: JSON in production, pretty in development, `ALLUVIUM_LOG`/`alluvium.log.filter`, span hierarchy.

### [To Do] OBS-0001 Create `alluvium-observability` (layer 3) with its crate docs and dependency justification
P0 · M2 · Task · deps: none · ref: docs/adr/0005-observability-model.md
Done when: the crate exists with a `//!` doc (responsibility, layer, Kafka counterpart, invariants), is registered as layer 3 in `xtask check-layers` and docs/architecture.md, and adds `tracing` and `tracing-subscriber` to `[workspace.dependencies]` with the security.md dependency questions answered in the PR; `cargo deny check` passes.

### [To Do] OBS-0002 Subscriber builder: one global install from the binary, with a reloadable filter handle
P0 · M2 · Task · deps: OBS-0001
Done when: `ObservabilityConfig { format, filter, ansi }` (plain struct; config keys are mapped by the binary, since this crate must not depend on CFG) builds the registry and installs it once; a second install returns `AlreadyInstalled`; the returned guard exposes the filter reload handle used by OBS-F02 and flushes on drop; unit tests.

### [To Do] OBS-0003 JSON lines formatter: stable schema with span stack and standard fields
P0 · M2 · Task · deps: OBS-0002 · ref: docs/standards/debuggability.md
Done when: each event is one JSON object with `timestamp` (RFC 3339 UTC, microseconds), `level`, `target`, `message`, event fields, and `spans` (outermost first, with their fields); field values never span lines; the schema is documented in docs/standards/debuggability.md and pinned by a snapshot test.

### [To Do] OBS-0004 Human-readable formatter for development
P1 · M2 · Task · deps: OBS-0002
Done when: `format = pretty` prints level, target, span context and fields on one line with ANSI colours only when stdout is a terminal; JSON stays the default when the format is unset; snapshot test with colours off.

### [To Do] OBS-0005 Filter sources: `ALLUVIUM_LOG` over `alluvium.log.filter`, default `info`
P0 · M2 · Task · deps: OBS-0002 · ref: docs/standards/debuggability.md#tracing
Done when: the filter uses `RUST_LOG` syntax; `ALLUVIUM_LOG` overrides the `alluvium.log.filter` value passed in by the binary; an invalid filter is a startup error naming the source and the parse position; tests cover precedence and every error case.

### [To Do] OBS-0006 Non-blocking bounded log writer with a dropped-line counter
P0 · M2 · Task · deps: OBS-0002 · labels: nfr
Done when: log output goes through a bounded queue drained by a dedicated thread, so Tokio workers never block on stdout; on overflow lines are dropped and counted in `alluvium_log_dropped_lines_total`; the queue flushes on shutdown and from the panic path (best effort, bounded wait); a test with a stalled writer shows bounded memory and the counter increasing.

### [To Do] OBS-0007 Test utility: capture events and spans for assertions in other crates' tests
P1 · M2 · Task · deps: OBS-0002 · labels: test
Done when: a `test-util` feature provides a capturing layer usable without installing a global subscriber (scoped dispatcher), with helpers to find events by `event` name and read fields including inherited span fields; used by at least the OBS-F03 and OBS-F07 tests.

### [To Do] OBS-0008 Operator docs: reading Alluvium logs and choosing format and filter
P1 · M2 · Task · deps: OBS-0003, OBS-0005, OBS-0006 · labels: docs
Done when: docs/guides/debugging.md explains the JSON schema, the span hierarchy, filtering by `conn_id` and `correlation_id`, `ALLUVIUM_LOG` versus `alluvium.log.filter`, and the dropped-line counter, with jq examples that are run in a docs test against sample output.

## [To Do] OBS-F02 Runtime log-level control
P1 · M2 · Feature · deps: OBS-F01, OBS-F08
Scope: `PUT /debug/log-level` and Kafka's `BROKER_LOGGER` config resource via DescribeConfigs/IncrementalAlterConfigs.

### [To Do] OBS-0009 `LogLevelController`: validate-then-swap filter changes with a change event
P1 · M2 · Task · deps: none
Done when: `current()` returns the active filter; `set(filter, source, principal)` parses first, swaps atomically through the OBS-F01 reload handle and returns the previous filter; each change emits `log_filter_changed` (old, new, source `diagnostics` or `broker_logger`, principal); invalid filters leave the filter unchanged; unit tests.

### [To Do] OBS-0010 `GET`/`PUT /debug/log-level` on the diagnostics endpoint
P1 · M2 · Task · deps: OBS-0009 · ref: docs/standards/debuggability.md#diagnostics-endpoint
Done when: `GET` returns the current filter and per-target directives as JSON; `PUT` with a filter body applies it and returns old and new; an invalid filter returns 400 with the parse error; the route obeys the endpoint's authentication on non-loopback binds; it is the endpoint's only mutating route; integration tests.

### [To Do] OBS-0011 BROKER_LOGGER model: Kafka logger names mapped to tracing targets, and describe output
P1 · M2 · Task · deps: OBS-0009 · ref: server/src/main/java/org/apache/kafka/server/logger/RuntimeLoggerManager.java, server/src/main/java/org/apache/kafka/server/logger/LoggingController.java @4.4.0
Done when: a documented table maps Kafka logger names (`root`, `kafka`, `kafka.request.logger`, `kafka.authorizer.logger`, `kafka.controller`, `state.change.logger`, `kafka.log.LogCleaner` and the others Alluvium has equivalents for) to tracing targets; levels map TRACE/DEBUG/INFO/WARN/ERROR (FATAL to error, OFF to off); `describe()` returns every known logger with its effective level as config entries for DescribeConfigs; unit tests.

### [To Do] OBS-0012 BROKER_LOGGER alter semantics with Kafka's error codes
P1 · M2 · Task · deps: OBS-0011 · ref: server/src/main/java/org/apache/kafka/server/logger/RuntimeLoggerManager.java @4.4.0
Done when: `apply_changes(node_id, resource_name, ops, validate_only)` accepts SET and DELETE; APPEND and SUBTRACT, a null value, a non-integer or foreign node id, and DELETE of the root logger return INVALID_REQUEST; an unknown logger or invalid level returns INVALID_CONFIG; validate-only changes nothing; one exhaustive error-to-ErrorCode match; tests port RuntimeLoggerManagerTest cases.

### [To Do] OBS-0013 Runtime log-level changes are node-local and reset on restart, as in Kafka
P2 · M2 · Task · deps: OBS-0010, OBS-0012 · labels: docs
Done when: a test changes levels through both paths, restarts the node and sees the configured filter again; docs/guides/debugging.md documents the scope and how to make a change permanent.

## [To Do] OBS-F03 Field and span conventions
P1 · M2 · Feature · deps: OBS-F01
Scope: Field-name constants, span constructors, and a check that only documented field names are used.

### [To Do] OBS-0014 `fields` module: a constant for every standard field name
P1 · M2 · Task · deps: none · ref: docs/standards/debuggability.md#standard-field-names
Done when: constants exist for every name in the debuggability field table (`node_id`, `broker_id`, `conn_id`, `listener`, `peer_addr`, `api`, `api_version`, `correlation_id`, `client_id`, `principal`, `topic`, `topic_id`, `partition`, offsets and watermarks, replication, group, producer, `error_code`, `reason`, `elapsed_ms`); a test parses the table in docs/standards/debuggability.md and fails when the two differ.

### [To Do] OBS-0015 Span constructors for node, connection, request, partition and driver spans
P1 · M2 · Task · deps: OBS-0014, OBS-0007 · ref: docs/standards/debuggability.md#span-hierarchy
Done when: `node_span(node_id, roles)`, `connection_span(conn_id, listener, peer_addr)`, `request_span(api, api_version, correlation_id, client_id)` with `principal` recorded after authentication, `partition_span(topic, partition)` and `driver_span(component, topic, partition, group_id)` (optional fields empty) are plain functions; a capture test shows an event inside a partition span carrying connection, request and partition fields.

### [To Do] OBS-0016 Field value conventions: Kafka error names, principals, topic-partitions and durations
P1 · M2 · Task · deps: OBS-0014, ERR-F01, TYPE-F09
Done when: helpers render `error_code` as the Kafka error name (`NOT_LEADER_OR_FOLLOWER`) from the ERR-F01 registry, principals as `User:alice`, `elapsed_ms` from a Duration, and payload-bearing values only through `PayloadSummary`; unit tests.

### [To Do] OBS-0017 `cargo xtask check-fields`: reject undocumented field names in tracing calls
P1 · M2 · Task · deps: OBS-0014, FND-F04
Done when: the check scans workspace sources for field names in `tracing` event and span macros and fails on names that are neither in the field table nor `old_<field>`/`new_<field>` forms of documented fields; a fixture with an undocumented field fails; it runs in `cargo xtask ci`.

### [To Do] OBS-0018 Document span and field usage in the debuggability standard
P2 · M2 · Task · deps: OBS-0015, OBS-0017 · labels: docs
Done when: docs/standards/debuggability.md shows the constructors, the field-check workflow, and how to add a field (table row plus constant in the same PR); links resolve.

## [To Do] OBS-F04 Metrics registry and exporter
P0 · M2 · Feature · deps: FND-F01
Scope: Explicitly passed `prometheus-client` registry, `/metrics` exposition, naming and cardinality rules.

### [To Do] OBS-0019 `MetricsRegistry`: explicitly passed `prometheus-client` registry with per-subsystem sub-registries
P0 · M2 · Task · deps: none · ref: docs/adr/0005-observability-model.md
Done when: the registry is created by the binary and passed down (no global, no lazy static); `sub_registry(subsystem)` prefixes `alluvium_<subsystem>_`; `prometheus-client` is added with its dependency justification; two registries in one test process do not interfere.

### [To Do] OBS-0020 Metric naming rules enforced at registration and by a whole-registry test
P0 · M2 · Task · deps: OBS-0019 · ref: docs/standards/debuggability.md#metrics
Done when: names must match `alluvium_<subsystem>_<name>_<unit>` in snake_case; counters end in `_total`, durations in `_seconds`, sizes in `_bytes`; violations fail registration in debug builds, and an integration test builds a full node registry and checks every family.

### [To Do] OBS-0021 Label cardinality controls: topic allowed, partition behind a flag, capped client ids, forbidden labels
P0 · M2 · Task · deps: OBS-0019 · labels: nfr
Done when: label sets are typed; `partition` labels exist only when `alluvium.metrics.per.partition.enabled` is true; `client_id` labels are only allowed on quota families and are capped, with overflow folded into `__other__` and counted in `alluvium_metrics_label_overflow_total`; a test rejects `offset`, `member_id` and `conn_id` labels.

### [To Do] OBS-0022 Removing per-topic and per-partition series when the entity goes away
P1 · M2 · Task · deps: OBS-0021 · labels: nfr
Done when: families support removing a label set; series for deleted topics and partitions moved away are removed, as Kafka removes its metrics; a test creating and deleting 10k topics ends with no leftover series.

### [To Do] OBS-0023 Standard histogram bucket layouts for latency and size
P0 · M2 · Task · deps: OBS-0019
Done when: constructors provide latency buckets in seconds (100 µs to 60 s, exponential) and size buckets in bytes (64 B to 128 MiB); documented in docs/standards/debuggability.md with the reasoning; unit tests check the layouts.

### [To Do] OBS-0024 Queue metrics helper: depth, capacity and rejections for every bounded queue
P0 · M2 · Task · deps: OBS-0019 · ref: docs/standards/debuggability.md#metrics
Done when: `QueueMetrics::register(registry, queue)` creates `<queue>_depth`, `<queue>_capacity` and `<queue>_rejections_total` (or `<queue>_mutes_total` for muting queues); a bounded-channel wrapper updates them; unit test.

### [To Do] OBS-0025 Exposition encoding: OpenMetrics and Prometheus text with content negotiation
P0 · M2 · Task · deps: OBS-0019
Done when: `encode(registry, accept)` returns OpenMetrics text when requested and Prometheus 0.0.4 text otherwise, with the right content type; output passes `promtool check metrics` in CI; served by OBS-F08's `/metrics` route.

### [To Do] OBS-0026 Scrape cost bound: encoding 100k series within budget
P1 · M2 · Task · deps: OBS-0025, TEST-F11 · labels: perf, nfr
Done when: a criterion bench encodes a registry with 100k series and records time and peak allocation; the budget is written down with NFR-F05 and checked by the benchmark job.

### [To Do] OBS-0027 Test helper: read a metric value by name and labels
P1 · M2 · Task · deps: OBS-0019 · labels: test
Done when: `metrics::testing::value(&registry, name, labels)` returns counter, gauge or histogram count and sum, so tests can assert on metrics; used by at least one test in this feature.

## [To Do] OBS-F05 Kafka metric mapping
P1 · M2 · Feature · deps: OBS-F04
Scope: One task per Kafka metric family: Prometheus equivalent plus docs/metrics.md mapping row.

### [To Do] OBS-0028 docs/metrics.md generated from Kafka-name annotations on metric definitions
P1 · M2 · Task · deps: FND-F04 · ref: docs/operations/monitoring.md @4.4.0
Done when: metric definitions take `kafka_name` (JMX ObjectName and attribute) or `kafka_equivalent: none(reason)`; `cargo xtask docs metrics` writes docs/metrics.md (Alluvium name, type, labels, Kafka MBean, conversion note); CI fails when the page is stale; DOCS-F05 publishes it.

### [To Do] OBS-0029 Dashboard translation guide: meters, timers and gauges to PromQL
P1 · M2 · Task · deps: OBS-0028 · labels: docs
Done when: docs/metrics.md explains that Kafka meters map to `_total` counters (`OneMinuteRate` becomes `rate(x[1m])`, `Count` the counter), Yammer timers and histograms map to `_seconds` histograms (`Mean`, `99thPercentile` via `histogram_quantile`), ms values become seconds, and MBean keys become labels; each rule has a worked example.

### [To Do] OBS-0030 Completeness check: every Kafka 4.4.0 broker and controller MBean has a docs/metrics.md row
P1 · M2 · Task · deps: OBS-0028 · labels: test · ref: docs/operations/monitoring.md @4.4.0
Done when: a vendored list of the MBeans in docs/operations/monitoring.md (broker, controller, KRaft, group coordinator, tiered storage and share sections) plus those defined only in code is checked against docs/metrics.md; client-side MBeans listed on that page (kafka.consumer, producer, Connect and Streams) are recorded as out of scope; missing rows fail CI except families listed in an allow-list with their milestone; the allow-list is empty at M9.

### [To Do] OBS-0031 Kafka metric mapping: `kafka.network:type=RequestMetrics` request, error and size metrics
P1 · M2 · Task · deps: SRV-F08 · ref: server/src/main/java/org/apache/kafka/network/metrics/RequestMetrics.java @4.4.0
Done when: RequestsPerSec{request,version} maps to `alluvium_request_total{api,api_version}`, DeprecatedRequestsPerSec to `alluvium_request_deprecated_version_total{api,api_version}` (client software labels dropped for cardinality, documented), ErrorsPerSec{request,error} to `alluvium_request_errors_total{api,error_code}`, RequestBytes to `alluvium_request_size_bytes{api}`, TemporaryMemoryBytes to `alluvium_request_temporary_memory_bytes{api}`; FetchConsumer/FetchFollower become `api="Fetch"` with `replica=consumer|follower`; MessageConversionsTimeMs is documented as having no equivalent (no message format conversion, KIP-724). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0032 Kafka metric mapping: `kafka.network:type=RequestMetrics` latency phases
P1 · M2 · Task · deps: SRV-F08 · ref: server/src/main/java/org/apache/kafka/network/metrics/RequestMetrics.java @4.4.0
Done when: TotalTimeMs, RequestQueueTimeMs, LocalTimeMs, RemoteTimeMs, ThrottleTimeMs, ResponseQueueTimeMs and ResponseSendTimeMs{request} map to the histograms `alluvium_request_{total,queue,local,remote,throttle,response_queue,response_send}_seconds{api}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0033 Kafka metric mapping: `RequestChannel` queue sizes and `KafkaRequestHandlerPool` idle ratio
P1 · M2 · Task · deps: SRV-F05 · ref: core/src/main/scala/kafka/network/RequestChannel.scala, core/src/main/scala/kafka/server/KafkaRequestHandler.scala @4.4.0
Done when: RequestQueueSize maps to `alluvium_request_queue_depth` (with `_capacity` and `_mutes_total`), ResponseQueueSize to `alluvium_network_response_queue_depth`, RequestHandlerAvgIdlePercent (and the Broker/Controller variants in combined mode) to `alluvium_request_handler_idle_ratio{role}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0034 Kafka metric mapping: `kafka.network:type=SocketServer` and `Acceptor` metrics
P1 · M2 · Task · deps: NET-F11 · ref: core/src/main/scala/kafka/network/SocketServer.scala @4.4.0
Done when: MemoryPoolAvailable and MemoryPoolUsed map to `alluvium_network_memory_pool_{available,used}_bytes`, MemoryPoolDepletedTimeTotal to `alluvium_network_memory_pool_depleted_seconds_total` (MemoryPoolAvgDepletedPercent is its rate), ExpiredConnectionsKilledCount to `alluvium_network_expired_connections_killed_total`, AcceptorBlockedPercent{listener} to `alluvium_network_accept_blocked_seconds_total{listener}`; NetworkProcessorAvgIdlePercent and per-processor IdlePercent are documented as replaced by the Tokio runtime busy ratio (no processor threads). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0035 Kafka metric mapping: `socket-server-metrics` connection and traffic metrics per listener
P1 · M2 · Task · deps: NET-F11 · ref: clients/src/main/java/org/apache/kafka/common/network/Selector.java @4.4.0
Done when: connection-count, connection-creation-rate/total, connection-close-rate/total, incoming-byte-rate/total, outgoing-byte-rate/total, request-rate/total, request-size-avg/max, response-rate/total and connections{clientSoftwareName,clientSoftwareVersion} map to `alluvium_network_connections{listener}`, `alluvium_network_connections_{created,closed}_total{listener}`, `alluvium_network_{received,sent}_bytes_total{listener}`, `alluvium_network_{requests,responses}_total{listener}`, `alluvium_network_request_size_bytes{listener}` and a capped `alluvium_network_client_software_connections{client_software_name,client_software_version}`; select and io-wait metrics are documented as not applicable. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0036 Kafka metric mapping: `socket-server-metrics` authentication and re-authentication metrics
P1 · M4 · Task · deps: NET-F08, SEC-F12 · ref: clients/src/main/java/org/apache/kafka/common/network/Selector.java @4.4.0
Done when: successful-authentication-rate/total, failed-authentication-rate/total, successful-authentication-no-reauth-total, successful-reauthentication-rate/total, failed-reauthentication-rate/total and reauthentication-latency-avg/max map to `alluvium_network_authentications_total{listener,mechanism,result}`, `alluvium_network_reauthentications_total{listener,result}` and `alluvium_network_reauthentication_seconds{listener}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0037 Kafka metric mapping: inter-node client channel metrics (`raft-channel-metrics`, `txn-marker-channel-metrics`, node-to-controller channels)
P2 · M2 · Task · deps: NET-F10 · ref: core/src/main/scala/kafka/raft/KafkaRaftManager.scala, server/src/main/java/org/apache/kafka/server/NodeToControllerChannelManagerImpl.java @4.4.0
Done when: the Selector client metrics of Kafka's inter-node clients (connection count, request and response rates, bytes, request-latency-avg/max) map to `alluvium_internode_connections{client}`, `alluvium_internode_requests_total{client,api}`, `alluvium_internode_{sent,received}_bytes_total{client}` and `alluvium_internode_request_seconds{client,api}`, with `client` one of raft, txn_marker, controller_forwarding, controller_heartbeat. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0038 Kafka metric mapping: `kafka.server:type=BrokerTopicMetrics` produce and fetch traffic
P1 · M2 · Task · deps: REPL-F03, REPL-F13 · ref: storage/src/main/java/org/apache/kafka/storage/log/metrics/BrokerTopicMetrics.java @4.4.0
Done when: MessagesInPerSec, BytesInPerSec, BytesOutPerSec, BytesRejectedPerSec, TotalProduceRequestsPerSec, TotalFetchRequestsPerSec, FailedProduceRequestsPerSec and FailedFetchRequestsPerSec{topic} map to `alluvium_topic_{messages_in,bytes_in,bytes_out,bytes_rejected}_total{topic}` and `alluvium_topic_{produce,fetch}_requests_total{topic,result}`; Kafka's all-topics series is `sum()` over topics (documented); Produce/FetchMessageConversionsPerSec have no equivalent (KIP-724). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0039 Kafka metric mapping: `BrokerTopicMetrics` record validation failures
P1 · M2 · Task · deps: REC-F12, REPL-F03 · ref: storage/src/main/java/org/apache/kafka/storage/log/metrics/BrokerTopicMetrics.java @4.4.0
Done when: NoKeyCompactedTopicRecordsPerSec, InvalidMagicNumberRecordsPerSec, InvalidMessageCrcRecordsPerSec and InvalidOffsetOrSequenceRecordsPerSec map to `alluvium_topic_records_rejected_total{reason=no_key_compacted|invalid_magic|invalid_crc|invalid_offset_or_sequence}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0040 Kafka metric mapping: `BrokerTopicMetrics` replication and reassignment bytes
P1 · M3 · Task · deps: REPL-F06, REPL-F07, CTRL-F10 · ref: storage/src/main/java/org/apache/kafka/storage/log/metrics/BrokerTopicMetrics.java @4.4.0
Done when: ReplicationBytesInPerSec, ReplicationBytesOutPerSec, ReassignmentBytesInPerSec and ReassignmentBytesOutPerSec map to `alluvium_replication_bytes_{in,out}_total` and `alluvium_reassignment_bytes_{in,out}_total`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0041 Kafka metric mapping: `BrokerTopicMetrics` share fetch and acknowledge requests
P1 · M8 · Task · deps: SHARE-F03, SHARE-F04 · ref: storage/src/main/java/org/apache/kafka/storage/log/metrics/BrokerTopicMetrics.java @4.4.0
Done when: TotalShareFetchRequestsPerSec, FailedShareFetchRequestsPerSec, TotalShareAcknowledgementRequestsPerSec and FailedShareAcknowledgementRequestsPerSec{topic} map to `alluvium_topic_share_{fetch,acknowledge}_requests_total{topic,result}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0042 Kafka metric mapping: `BrokerTopicMetrics` remote fetch, copy and delete traffic
P1 · M9 · Task · deps: TIER-F03, TIER-F04, TIER-F05 · ref: docs/operations/monitoring.md @4.4.0
Done when: RemoteFetch/RemoteCopy Bytes, Requests and Errors PerSec and RemoteDeleteRequests/ErrorsPerSec{topic} map to `alluvium_remote_log_{fetch,copy}_bytes_total{topic}`, `alluvium_remote_log_{fetch,copy,delete}_requests_total{topic,result}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0043 Kafka metric mapping: `BrokerTopicMetrics` remote lag, size and aux-state metrics
P1 · M9 · Task · deps: TIER-F02, TIER-F03, TIER-F05 · ref: docs/operations/monitoring.md @4.4.0
Done when: RemoteCopyLagBytes, RemoteCopyLagSegments, RemoteDeleteLagBytes, RemoteDeleteLagSegments, BuildRemoteLogAuxStateRequests/ErrorsPerSec, RemoteLogSizeComputationTime, RemoteLogSizeBytes and RemoteLogMetadataCount{topic} map to `alluvium_remote_log_{copy,delete}_lag_{bytes,segments}{topic}`, `alluvium_remote_log_aux_state_builds_total{topic,result}`, `alluvium_remote_log_size_computation_seconds{topic}`, `alluvium_remote_log_size_bytes{topic}` and `alluvium_remote_log_metadata_entries{topic}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0044 Kafka metric mapping: `kafka.server:type=ReplicaManager` partition gauges
P1 · M2 · Task · deps: REPL-F17 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala @4.4.0
Done when: UnderReplicatedPartitions, UnderMinIsrPartitionCount, AtMinIsrPartitionCount, PartitionCount, LeaderCount, OfflineReplicaCount and ReassigningPartitions map to `alluvium_replica_{under_replicated,under_min_isr,at_min_isr,offline,reassigning}_partitions`, `alluvium_replica_partitions` and `alluvium_replica_leader_partitions`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0045 Kafka metric mapping: `ReplicaManager` ISR change meters
P1 · M3 · Task · deps: REPL-F05 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala @4.4.0
Done when: IsrShrinksPerSec, IsrExpandsPerSec and FailedIsrUpdatesPerSec map to `alluvium_replica_isr_shrinks_total`, `alluvium_replica_isr_expands_total` and `alluvium_replica_isr_update_failures_total`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0046 Kafka metric mapping: `ReplicaManager` producer gauges and `Partition` LastStableOffsetLag
P1 · M6 · Task · deps: TXN-F16, TXN-F11 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala, core/src/main/scala/kafka/cluster/Partition.scala @4.4.0
Done when: ProducerIdCount and PartitionsWithLateTransactionsCount map to `alluvium_replica_producer_ids` and `alluvium_replica_partitions_with_late_transactions`; LastStableOffsetLag{topic,partition} maps to `alluvium_partition_last_stable_offset_lag` (per-partition flag). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0047 Kafka metric mapping: `kafka.cluster:type=Partition` per-partition gauges
P1 · M2 · Task · deps: REPL-F17 · ref: core/src/main/scala/kafka/cluster/Partition.scala @4.4.0
Done when: UnderReplicated, InSyncReplicasCount, ReplicasCount, UnderMinIsr and AtMinIsr{topic,partition} map to `alluvium_partition_{under_replicated,in_sync_replicas,replicas,under_min_isr,at_min_isr}{topic,partition}`, registered only when `alluvium.metrics.per.partition.enabled` is set (deviation from Kafka documented with the reason). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0048 Kafka metric mapping: `kafka.log:type=Log` per-partition log gauges
P1 · M2 · Task · deps: STOR-F08, STOR-F13 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/UnifiedLog.java @4.4.0
Done when: Size, NumLogSegments, LogStartOffset, LogEndOffset and RetentionSizeInPercent{topic,partition} map to `alluvium_log_size_bytes`, `alluvium_log_segments`, `alluvium_log_start_offset`, `alluvium_log_end_offset` and `alluvium_log_retention_size_ratio` with per-partition labels behind the flag and a topic-level `alluvium_log_size_bytes{topic}` always. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0049 Kafka metric mapping: `kafka.log:type=LogFlushStats` flush rate and time
P1 · M2 · Task · deps: STOR-F17 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogSegment.java @4.4.0
Done when: LogFlushRateAndTimeMs maps to the histogram `alluvium_log_flush_seconds` (its count gives the rate). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0050 Kafka metric mapping: `kafka.log:type=LogManager` recovery progress
P1 · M2 · Task · deps: STOR-F16, STOR-F09 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java @4.4.0
Done when: remainingLogsToRecover and remainingSegmentsToRecover{dir,threadNum} map to `alluvium_log_recovery_remaining_logs{log_dir}` and `alluvium_log_recovery_remaining_segments{log_dir}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0051 Kafka metric mapping: `LogManager` offline and cordoned log directories
P1 · M7 · Task · deps: STOR-F21, CTRL-F20 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java @4.4.0
Done when: OfflineLogDirectoryCount, LogDirectoryOffline{logDirectory}, CordonedLogDirectoryCount and LogDirectoryCordoned{logDirectory} map to `alluvium_log_dirs_offline`, `alluvium_log_dir_offline{log_dir}`, `alluvium_log_dirs_cordoned` and `alluvium_log_dir_cordoned{log_dir}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0052 Kafka metric mapping: `kafka.log:type=LogCleaner` and `LogCleanerManager`
P1 · M5 · Task · deps: STOR-F14 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleaner.java, storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleanerManager.java @4.4.0
Done when: max-buffer-utilization-percent, cleaner-recopy-percent, max-clean-time-secs, max-compaction-delay-secs, DeadThreadCount, max-dirty-percent, time-since-last-run-ms, uncleanable-partitions-count{logDirectory} and uncleanable-bytes{logDirectory} map to `alluvium_log_cleaner_buffer_utilization_ratio`, `alluvium_log_cleaner_recopy_ratio`, `alluvium_log_cleaner_max_clean_seconds`, `alluvium_log_cleaner_max_compaction_delay_seconds`, `alluvium_log_cleaner_max_dirty_ratio`, `alluvium_log_cleaner_seconds_since_last_run` and `alluvium_log_cleaner_uncleanable_{partitions,bytes}{log_dir}`; DeadThreadCount is documented as not applicable (a task panic aborts the process). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0053 Kafka metric mapping: `kafka.server:type=DelayedOperationPurgatory` sizes
P1 · M2 · Task · deps: SRV-F07 · ref: server-common/src/main/java/org/apache/kafka/server/purgatory/DelayedOperationPurgatory.java @4.4.0
Done when: PurgatorySize and NumDelayedOperations{delayedOperation} map to `alluvium_purgatory_watched_keys{operation}` and `alluvium_purgatory_operations{operation}` for produce, fetch, delete_records, elect_leader, remote_fetch, remote_list_offsets and share_fetch. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0054 Kafka metric mapping: `DelayedProduceMetrics` and `DelayedFetchMetrics` expirations
P1 · M2 · Task · deps: REPL-F08, REPL-F09 · ref: server/src/main/java/org/apache/kafka/server/purgatory/DelayedProduce.java, core/src/main/scala/kafka/server/DelayedFetch.scala @4.4.0
Done when: DelayedProduceMetrics ExpiresPerSec and DelayedFetchMetrics ExpiresPerSec{fetcherType} map to `alluvium_purgatory_expired_total{operation=produce}` and `alluvium_purgatory_expired_total{operation=fetch,replica=consumer|follower}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0055 Kafka metric mapping: `DelayedDeleteRecordsMetrics` expirations
P2 · M7 · Task · deps: API-F21 · ref: server/src/main/java/org/apache/kafka/server/purgatory/DelayedDeleteRecords.java @4.4.0
Done when: ExpiresPerSec maps to `alluvium_purgatory_expired_total{operation=delete_records}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0056 Kafka metric mapping: `DelayedRemoteFetchMetrics` and `DelayedRemoteListOffsetsMetrics` expirations
P2 · M9 · Task · deps: TIER-F04 · ref: storage/src/main/java/org/apache/kafka/server/purgatory/DelayedRemoteFetch.java, storage/src/main/java/org/apache/kafka/server/purgatory/DelayedRemoteListOffsets.java @4.4.0
Done when: both ExpiresPerSec meters map to `alluvium_purgatory_expired_total{operation=remote_fetch|remote_list_offsets,topic}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0057 Kafka metric mapping: `DelayedShareFetchMetrics` expirations
P2 · M8 · Task · deps: SHARE-F03 · ref: docs/operations/monitoring.md @4.4.0
Done when: ExpiresPerSec maps to `alluvium_purgatory_expired_total{operation=share_fetch}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0058 Kafka metric mapping: `kafka.server:type=FetchSessionCache`
P1 · M2 · Task · deps: REPL-F10 · ref: server/src/main/java/org/apache/kafka/server/FetchSession.java @4.4.0
Done when: NumIncrementalFetchSessions, NumIncrementalFetchPartitionsCached and IncrementalFetchSessionEvictionsPerSec map to `alluvium_fetch_session_cache_sessions`, `alluvium_fetch_session_cache_partitions` and `alluvium_fetch_session_cache_evictions_total`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0059 Kafka metric mapping: `ReplicaFetcherManager`, `FetcherStats` and `FetcherLagMetrics`
P1 · M3 · Task · deps: REPL-F06 · ref: core/src/main/scala/kafka/server/AbstractFetcherManager.scala, core/src/main/scala/kafka/server/AbstractFetcherThread.scala @4.4.0
Done when: MaxLag, MinFetchRate and FailedPartitionsCount map to `alluvium_replica_fetcher_max_lag_messages`, `alluvium_replica_fetcher_min_fetch_rate` and `alluvium_replica_fetcher_failed_partitions`; RequestsPerSec and BytesPerSec{brokerHost,brokerPort} to `alluvium_replica_fetcher_{requests,bytes}_total{source_broker_id}`; ConsumerLag{topic,partition} to `alluvium_replica_fetcher_lag_messages` (per-partition flag); DeadThreadCount is documented as not applicable. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0060 Kafka metric mapping: `ReplicaAlterLogDirsManager` fetcher metrics
P2 · M7 · Task · deps: STOR-F22 · ref: core/src/main/scala/kafka/server/ReplicaAlterLogDirsManager.scala @4.4.0
Done when: the same fetcher metrics with clientId=ReplicaAlterLogDirs map to `alluvium_log_dir_mover_{max_lag_messages,failed_partitions,bytes_total}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0061 Kafka metric mapping: `kafka.server:type=Produce|Fetch` client quota metrics
P1 · M4 · Task · deps: QUOTA-F02, QUOTA-F03, QUOTA-F09 · ref: server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java @4.4.0
Done when: throttle-time and byte-rate per (user, client-id) and the throttled-channel queue-size map to `alluvium_quota_throttle_seconds{quota=produce|fetch,user,client_id}`, `alluvium_quota_byte_rate{quota,user,client_id}` and `alluvium_quota_throttled_channels{quota}`, with user and client_id labels capped (overflow to `__other__`). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0062 Kafka metric mapping: `kafka.server:type=Request` request quota metrics
P1 · M4 · Task · deps: QUOTA-F04 · ref: server/src/main/java/org/apache/kafka/server/quota/ClientRequestQuotaManager.java @4.4.0
Done when: request-time, throttle-time and exempt-request-time map to `alluvium_quota_request_time_ratio{user,client_id}`, `alluvium_quota_throttle_seconds{quota=request,...}` and `alluvium_quota_exempt_request_time_ratio`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0063 Kafka metric mapping: `kafka.server:type=ControllerMutation` quota metrics
P1 · M4 · Task · deps: QUOTA-F05 · ref: server/src/main/java/org/apache/kafka/server/quota/ControllerMutationQuotaManager.java @4.4.0
Done when: mutation-rate and throttle-time map to `alluvium_quota_controller_mutation_rate{user,client_id}` and `alluvium_quota_throttle_seconds{quota=controller_mutation,...}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0064 Kafka metric mapping: `LeaderReplication`, `FollowerReplication` and `AlterLogDirs` quota byte rates
P1 · M4 · Task · deps: REPL-F16 · ref: server/src/main/java/org/apache/kafka/server/quota/ReplicationQuotaManager.java @4.4.0
Done when: byte-rate for each replication quota type maps to `alluvium_quota_replication_byte_rate{quota=leader|follower|alter_log_dirs}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0065 Kafka metric mapping: connection creation rate quota metrics
P1 · M4 · Task · deps: QUOTA-F06 · ref: server/src/main/java/org/apache/kafka/network/ConnectionQuotaEntity.java, core/src/main/scala/kafka/network/SocketServer.scala @4.4.0
Done when: connection-accept-rate{listener}, broker-connection-accept-rate and connection-accept-throttle-time{listener} map to `alluvium_quota_connection_accept_rate{listener}`, `alluvium_quota_broker_connection_accept_rate` and `alluvium_quota_connection_accept_throttle_seconds{listener}`; per-IP rates are documented as available on `/debug` only (unbounded label). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0066 Kafka metric mapping: `kafka.controller:type=KafkaController` quorum controller state
P1 · M2 · Task · deps: CTRL-F19 · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/QuorumControllerMetrics.java @4.4.0
Done when: ActiveControllerCount, LastAppliedRecordOffset, LastCommittedRecordOffset, LastAppliedRecordTimestamp, LastAppliedRecordLagMs, NewActiveControllersCount and AvgIdleRatio map to `alluvium_controller_active`, `alluvium_controller_last_{applied,committed}_offset`, `alluvium_controller_last_applied_timestamp_seconds`, `alluvium_controller_last_applied_lag_seconds`, `alluvium_controller_activations_total` and `alluvium_controller_idle_ratio`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0067 Kafka metric mapping: `ControllerEventManager` timings and event-queue operation counts
P1 · M2 · Task · deps: CTRL-F19 · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/QuorumControllerMetrics.java @4.4.0
Done when: EventQueueTimeMs and EventQueueProcessingTimeMs map to `alluvium_controller_event_queue_seconds` and `alluvium_controller_event_processing_seconds`; EventQueueOperationsStartedCount and EventQueueOperationsTimedOutCount map to `alluvium_controller_operations_started_total` and `alluvium_controller_operations_timed_out_total`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0068 Kafka metric mapping: controller broker-heartbeat metrics
P1 · M2 · Task · deps: CTRL-F02 · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/QuorumControllerMetrics.java @4.4.0
Done when: TimedOutBrokerHeartbeatCount and TimeSinceLastHeartbeatReceivedMs{broker} map to `alluvium_controller_broker_heartbeat_timeouts_total` and `alluvium_controller_seconds_since_broker_heartbeat{broker_id}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0069 Kafka metric mapping: `ControllerMetadataMetrics` cluster gauges
P1 · M2 · Task · deps: CTRL-F19, CTRL-F02 · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/ControllerMetadataMetrics.java @4.4.0
Done when: FencedBrokerCount, ActiveBrokerCount, ControlledShutdownBrokerCount, BrokerRegistrationState{broker}, GlobalTopicCount, GlobalPartitionCount, OfflinePartitionsCount, PreferredReplicaImbalanceCount, MetadataErrorCount and IgnoredStaticVoters map to `alluvium_controller_brokers{state=fenced|active|controlled_shutdown}`, `alluvium_controller_broker_registration_state{broker_id}`, `alluvium_controller_topics`, `alluvium_controller_partitions`, `alluvium_controller_offline_partitions`, `alluvium_controller_preferred_replica_imbalance`, `alluvium_controller_metadata_errors_total` and `alluvium_controller_ignored_static_voters`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0070 Kafka metric mapping: `kafka.controller:type=ControllerStats` election meters
P1 · M3 · Task · deps: CTRL-F08 · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/ControllerMetadataMetrics.java @4.4.0
Done when: UncleanLeaderElectionsPerSec and ElectionFromEligibleLeaderReplicasPerSec map to `alluvium_controller_unclean_leader_elections_total` and `alluvium_controller_elr_leader_elections_total`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0071 Kafka metric mapping: ZooKeeper-era controller metrics still listed in the 4.4.0 docs
P2 · M2 · Task · deps: none · ref: docs/operations/monitoring.md @4.4.0
Done when: rows for ControllerChannelManager RequestRateAndQueueTimeMs, ControllerEventManager EventQueueSize, ControllerStats LeaderElectionRateAndTimeMs and KafkaController TopicsToDeleteCount, ReplicasToDeleteCount, TopicsIneligibleToDeleteCount and ReplicasIneligibleToDeleteCount state that Kafka 4.4.0 KRaft does not emit them (no definition in the 4.4.0 sources) and name the closest Alluvium metric where one exists.

### [To Do] OBS-0072 Kafka metric mapping: `kafka.server:type=raft-metrics` quorum state
P1 · M2 · Task · deps: RAFT-F14 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java @4.4.0
Done when: current-state, current-leader, current-vote, current-epoch, high-watermark, log-end-offset, log-end-epoch and number-unknown-voter-connections map to `alluvium_raft_state{state}` (one-hot over leader, candidate, prospective, voted, follower, unattached, observer), `alluvium_raft_leader_id`, `alluvium_raft_voted_id`, `alluvium_raft_epoch`, `alluvium_raft_high_watermark`, `alluvium_raft_log_end_offset`, `alluvium_raft_log_end_epoch` and `alluvium_raft_unknown_voter_connections`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0073 Kafka metric mapping: `raft-metrics` latencies and rates
P1 · M2 · Task · deps: RAFT-F14 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java @4.4.0
Done when: commit-latency-avg/max, election-latency-avg/max, fetch-records-rate, append-records-rate and poll-idle-ratio-avg map to `alluvium_raft_commit_seconds`, `alluvium_raft_election_seconds`, `alluvium_raft_fetched_records_total`, `alluvium_raft_appended_records_total` and `alluvium_raft_idle_ratio`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0074 Kafka metric mapping: `raft-metrics` dynamic quorum gauges
P1 · M3 · Task · deps: RAFT-F07, RAFT-F08 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java @4.4.0
Done when: number-of-voters, number-of-observers and uncommitted-voter-change map to `alluvium_raft_voters`, `alluvium_raft_observers` and `alluvium_raft_uncommitted_voter_change`; current-vote-directory-id is documented as available on `/debug/raft` (not numeric). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0075 Kafka metric mapping: `kafka.server:type=MetadataLoader`
P1 · M2 · Task · deps: META-F04 · ref: metadata/src/main/java/org/apache/kafka/image/loader/metrics/MetadataLoaderMetrics.java @4.4.0
Done when: CurrentMetadataVersion, HandleLoadSnapshotCount, CurrentControllerId, AvgIdleRatio and FinalizedLevel{featureName} map to the META-F04 loader metrics (`alluvium_metadata_loader_*`, `alluvium_metadata_finalized_feature_level{feature}`). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0076 Kafka metric mapping: `kafka.server:type=SnapshotEmitter`
P1 · M2 · Task · deps: META-F05 · ref: metadata/src/main/java/org/apache/kafka/image/publisher/metrics/SnapshotEmitterMetrics.java @4.4.0
Done when: LatestSnapshotGeneratedBytes and LatestSnapshotGeneratedAgeMs map to `alluvium_metadata_snapshot_latest_bytes` and `alluvium_metadata_snapshot_latest_age_seconds`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0077 Kafka metric mapping: `kafka.server:type=broker-metadata-metrics` and `BrokerMetadataListener`
P1 · M2 · Task · deps: SRV-F14, META-F04 · ref: server/src/main/java/org/apache/kafka/server/metrics/BrokerServerMetrics.java @4.4.0
Done when: last-applied-record-offset, last-applied-record-timestamp, last-applied-record-lag-ms, metadata-load-error-count, metadata-apply-error-count and ignored-static-voters map to `alluvium_metadata_last_applied_{offset,timestamp_seconds,lag_seconds}`, `alluvium_metadata_{load,apply}_errors_total` and `alluvium_metadata_ignored_static_voters`; MetadataBatchProcessingTimeUs and MetadataBatchSizes map to `alluvium_metadata_batch_processing_seconds` and `alluvium_metadata_batch_records`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0078 Kafka metric mapping: `app-info`, `KafkaServer` ClusterId and `node-metrics` supported feature levels
P1 · M2 · Task · deps: SRV-F01, META-F06 · ref: clients/src/main/java/org/apache/kafka/common/utils/internals/AppInfoParser.java, server/src/main/java/org/apache/kafka/server/metrics/NodeMetrics.java, core/src/main/scala/kafka/server/KafkaBroker.scala @4.4.0
Done when: app-info version, commit-id and start-time-ms map to `alluvium_build_info{version,commit}` and `alluvium_node_start_time_seconds`; KafkaServer ClusterId maps to `alluvium_node_info{cluster_id,node_id,roles}`; node-metrics maximum/minimum-supported-level{feature-name} map to `alluvium_node_supported_feature_level{feature,bound}`; yammer-metrics-count is documented as not applicable. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0079 Kafka metric mapping: `kafka.server:type=KafkaServer,name=BrokerState`
P1 · M3 · Task · deps: SRV-F13 · ref: core/src/main/scala/kafka/server/KafkaBroker.scala @4.4.0
Done when: BrokerState maps to `alluvium_broker_state{state}` (one-hot over STARTING, RECOVERY, RUNNING, PENDING_CONTROLLED_SHUTDOWN, SHUTTING_DOWN) with Kafka's numeric values documented. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0080 Kafka metric mapping: process and runtime metrics replacing JVM and OS statistics
P2 · M2 · Task · deps: SRV-F01 · ref: server/src/main/java/org/apache/kafka/server/metrics/LinuxIoMetricsCollector.java @4.4.0
Done when: the monitoring page's GC, CPU and I/O guidance and LinuxIoMetricsCollector's linux-disk-read-bytes, linux-disk-write-bytes, rchar, wchar, syscr, syscw and cancelled-write-bytes map to `process_cpu_seconds_total`, `process_resident_memory_bytes`, `process_open_fds`, `alluvium_process_disk_{read,write}_bytes_total` (Linux) and Tokio runtime metrics (`alluvium_runtime_workers_busy_ratio`, `alluvium_runtime_blocking_threads`, `alluvium_runtime_alive_tasks`); GC metrics are documented as not applicable. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0081 Kafka metric mapping: `kafka.server:type=ForwardingManager`
P2 · M2 · Task · deps: CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/metrics/ForwardingManagerMetrics.java @4.4.0
Done when: QueueTimeMs, QueueLength and RemoteTimeMs map to `alluvium_forwarding_queue_seconds`, `alluvium_forwarding_queue_depth` and `alluvium_forwarding_remote_seconds`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0082 Kafka metric mapping: `kafka.server:type=AssignmentsManager`
P2 · M7 · Task · deps: CTRL-F20 · ref: server/src/main/java/org/apache/kafka/server/AssignmentsManager.java @4.4.0
Done when: QueuedReplicaToDirAssignments maps to `alluvium_directory_assignments_queued`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0083 Kafka metric mapping: `group-coordinator-metrics` coordinator runtime
P1 · M5 · Task · deps: GRP-F01 · ref: coordinator-common/src/main/java/org/apache/kafka/coordinator/common/runtime/CoordinatorRuntimeMetricsImpl.java @4.4.0
Done when: num-partitions{state}, partition-load-time-avg/max, thread-idle-ratio-avg, event-queue-size, event-queue-time-ms, event-processing-time-ms, event-purgatory-time-ms, batch-linger-time-ms, batch-flush-time-ms, batch-flush-rate, batch-buffer-cache-size-bytes and batch-buffer-cache-discard-count map to `alluvium_group_coordinator_partitions{state}`, `alluvium_group_coordinator_partition_load_seconds`, `alluvium_group_coordinator_idle_ratio`, `alluvium_group_coordinator_event_queue_depth`, `alluvium_group_coordinator_event_{queue,processing,purgatory}_seconds`, `alluvium_group_coordinator_batch_{linger,flush}_seconds`, `alluvium_group_coordinator_batch_buffer_cache_bytes` and `alluvium_group_coordinator_batch_buffer_discards_total`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0084 Kafka metric mapping: group counts and rebalances (`group-coordinator-metrics`, `GroupMetadataManager`)
P1 · M5 · Task · deps: GRP-F11 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/metrics/GroupCoordinatorMetrics.java @4.4.0
Done when: group-count{protocol}, classic-group-count{state}, consumer-group-count{state}, group-completed-rebalance-rate/count, consumer-group-rebalance-rate/count and GroupMetadataManager NumGroups and NumGroups<State> map to `alluvium_group_groups{protocol,state}` and `alluvium_group_rebalances_total{protocol}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0085 Kafka metric mapping: committed offset metrics
P1 · M5 · Task · deps: GRP-F07, GRP-F11 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/metrics/GroupCoordinatorMetrics.java @4.4.0
Done when: offset-count, offset-commit-rate/count, offset-expiration-rate/count, offset-deletion-rate/count and GroupMetadataManager NumOffsets map to `alluvium_group_offsets` and `alluvium_group_offset_{commits,expirations,deletions}_total`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0086 Kafka metric mapping: streams group metrics
P1 · M8 · Task · deps: STRM-F01, STRM-F05 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/metrics/GroupCoordinatorMetrics.java @4.4.0
Done when: streams-group-count{state}, streams-group-rebalance-rate/count and the streams-group-topology-description set/get/delete success/error and cleanup-cycle/cleanup-eligible rates and counts map to `alluvium_group_groups{protocol=streams,state}`, `alluvium_group_rebalances_total{protocol=streams}`, `alluvium_streams_topology_description_operations_total{operation,result}`, `alluvium_streams_topology_description_cleanup_cycles_total` and `alluvium_streams_topology_description_cleanup_eligible_total`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0087 Kafka metric mapping: share group counts and rebalances
P1 · M8 · Task · deps: SHARE-F01 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/metrics/GroupCoordinatorMetrics.java @4.4.0
Done when: group-count{protocol=share}, share-group-count{state} and share-group-rebalance-rate/count map to `alluvium_group_groups{protocol=share,state}` and `alluvium_group_rebalances_total{protocol=share}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0088 Kafka metric mapping: `share-coordinator-metrics`
P1 · M8 · Task · deps: SHARE-F05 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/metrics/ShareCoordinatorMetrics.java @4.4.0
Done when: partition-load-time-avg/max, thread-idle-ratio-min/avg, write-rate/total, write-latency-avg/max, num-partitions{state}, last-pruned-offset{topic,partition}, batch-buffer-cache-size-bytes and batch-buffer-cache-discard-count map to `alluvium_share_coordinator_*` equivalents, with last-pruned-offset per partition behind the flag. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0089 Kafka metric mapping: `kafka.server:type=ShareGroupMetrics`
P1 · M8 · Task · deps: SHARE-F10 · ref: server/src/main/java/org/apache/kafka/server/share/metrics/ShareGroupMetrics.java @4.4.0
Done when: RecordAcknowledgementsPerSec{ackType}, PartitionLoadTimeMs, RequestTopicPartitionsFetchRatio{group}, TopicPartitionsAcquireTimeMs{group} and the DeadLetterQueue counters map to `alluvium_share_acknowledgements_total{ack_type}`, `alluvium_share_partition_load_seconds`, `alluvium_share_fetch_partition_ratio`, `alluvium_share_acquire_seconds` and `alluvium_share_dead_letter_*`; group-labelled series are aggregated (group ids are unbounded) with per-group values on `/debug/groups/{group_id}`, documented as a deviation. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0090 Kafka metric mapping: `kafka.server:type=SharePartitionMetrics`
P1 · M8 · Task · deps: SHARE-F02 · ref: server/src/main/java/org/apache/kafka/server/share/metrics/SharePartitionMetrics.java @4.4.0
Done when: AcquisitionLockTimeoutPerSec, InFlightMessageCount, InFlightBatchCount, InFlightBatchMessageCount, FetchLockTimeMs and FetchLockRatio{group,topic,partition} map to `alluvium_share_partition_*{topic}` aggregates, with partition labels behind the flag and no group label (deviation documented). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0091 Kafka metric mapping: `kafka.server:type=ShareSessionCache`
P2 · M8 · Task · deps: SHARE-F03 · ref: server/src/main/java/org/apache/kafka/server/share/session/ShareSessionCache.java @4.4.0
Done when: ShareSessionEvictionsPerSec, SharePartitionsCount and ShareSessionsCount map to `alluvium_share_session_cache_evictions_total`, `alluvium_share_session_cache_partitions` and `alluvium_share_session_cache_sessions`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0092 Kafka metric mapping: `transaction-coordinator-metrics`
P1 · M6 · Task · deps: TXN-F03 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionStateManager.scala @4.4.0
Done when: partition-load-time-avg/max map to `alluvium_txn_coordinator_partition_load_seconds`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0093 Kafka metric mapping: `TransactionMarkerChannelManager` queues
P1 · M6 · Task · deps: TXN-F08 · ref: core/src/main/scala/kafka/coordinator/transaction/TransactionMarkerChannelManager.scala @4.4.0
Done when: UnknownDestinationQueueSize and LogAppendRetryQueueSize map to `alluvium_txn_marker_unknown_destination_queue_depth` and `alluvium_txn_marker_append_retry_queue_depth`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0094 Kafka metric mapping: `kafka.server:type=AddPartitionsToTxnManager` verification
P1 · M6 · Task · deps: TXN-F06 · ref: server/src/main/java/org/apache/kafka/server/transaction/AddPartitionsToTxnManager.java @4.4.0
Done when: VerificationFailureRate and VerificationTimeMs map to `alluvium_txn_verification_failures_total` and `alluvium_txn_verification_seconds`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0095 Kafka metric mapping: StandardAuthorizer plugin metrics (KIP-877)
P1 · M4 · Task · deps: SEC-F08 · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizer.java @4.4.0
Done when: acls-total-count and authorization-request/allowed/denied-rate-per-minute map to `alluvium_authorizer_acls` and `alluvium_authorizer_decisions_total{result=allowed|denied}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0096 Kafka metric mapping: `client-metrics` telemetry ingestion metrics (KIP-714)
P2 · M8 · Task · deps: TELEM-F02, TELEM-F04 · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java @4.4.0
Done when: instance-count, unknown-subscription-request rate/count, throttle rate/count, plugin-export rate/count, plugin-error rate/count and plugin-export-time avg/max map to `alluvium_client_telemetry_instances`, `alluvium_client_telemetry_unknown_subscription_requests_total`, `alluvium_client_telemetry_throttled_total`, `alluvium_client_telemetry_exports_total{result}` and `alluvium_client_telemetry_export_seconds`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0097 Kafka metric mapping: `RemoteStorageThreadPool` and `RemoteLogManager` task metrics
P2 · M9 · Task · deps: TIER-F03, TIER-F04 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/RemoteStorageThreadPool.java, storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManager.java @4.4.0
Done when: RemoteLogReaderTaskQueueSize, RemoteLogReaderAvgIdlePercent, RemoteLogManagerTasksAvgIdlePercent and RemoteLogReaderFetchRateAndTimeMs map to `alluvium_remote_log_reader_queue_depth`, `alluvium_remote_log_reader_idle_ratio`, `alluvium_remote_log_tasks_idle_ratio` and `alluvium_remote_log_reader_fetch_seconds`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0098 Kafka metric mapping: `RemoteLogManager` remote copy and fetch throttle times
P2 · M9 · Task · deps: TIER-F03, TIER-F04 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/quota/RLMQuotaMetrics.java @4.4.0
Done when: remote-fetch-throttle-time-avg/max and remote-copy-throttle-time-avg/max map to `alluvium_remote_log_throttle_seconds{direction=fetch|copy}`. Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

### [To Do] OBS-0099 Kafka metric mapping: `kafka.log.remote:type=RemoteLogManager` retention percentages
P2 · M9 · Task · deps: TIER-F05, TIER-F06 · ref: docs/operations/monitoring.md @4.4.0
Done when: RetentionSizeInPercent and LocalRetentionSizeInPercent{topic,partition} map to `alluvium_remote_log_retention_size_ratio` and `alluvium_remote_log_local_retention_size_ratio` (per-partition flag). Each mapped metric carries its Kafka name, the docs/metrics.md rows exist, and a scenario test shows every mapped metric registered and moving.

## [To Do] OBS-F06 Flight recorder
P0 · M2 · Feature · deps: TYPE-F06
Scope: Bounded ring buffer of typed transitions per component; dump on error and panic; read via diagnostics.

### [To Do] OBS-0100 `FlightRecorder<E>`: fixed-capacity ring buffer of typed transitions
P0 · M2 · Task · deps: none · ref: docs/standards/debuggability.md#flight-recorders
Done when: entries hold a Clock timestamp, input kind, old and new state summaries and a reason, all small enums; capacity defaults to 256 and is fixed at construction; recording never allocates; `snapshot()` returns entries oldest first with a count of overwritten entries; a property test checks the bound and order for arbitrary sequences.

### [To Do] OBS-0101 Flight recorder entry formatting on read, as text and JSON
P0 · M2 · Task · deps: OBS-0100, TYPE-F09
Done when: entries format only when read, through `Display` and a JSON serializer shared with the diagnostics endpoint and TEST-F05 failure reports; payload fields are impossible to store (only `PayloadSummary`); a test asserts `size_of` of a standard entry stays within 64 bytes.

### [To Do] OBS-0102 Single writer, non-blocking readers, with a benchmarked record cost
P0 · M2 · Task · deps: OBS-0100, TEST-F11 · labels: perf
Done when: the owning component records without waiting on readers (readers copy under a short critical section); a criterion bench shows recording costs under 100 ns; a stress test with concurrent readers shows no lost or torn entries.

### [To Do] OBS-0103 Recorder registry keyed by component kind and id
P0 · M2 · Task · deps: OBS-0100
Done when: partitions, groups, transaction coordinator partitions, the Raft quorum and connections register and deregister recorders with their lifecycle (connection recorders are removed on close, keeping the registry bounded); lookups serve the `/debug` routes; unit tests.

### [To Do] OBS-0104 Dump a component's recorder to the log when it hits an error
P0 · M2 · Task · deps: OBS-0103, OBS-0101
Done when: `dump_to_log(component, reason)` writes the entries as one structured `flight_recorder_dump` event at error level; dumps are rate-limited per component (at most one per configured interval) with a suppressed-dump counter; tests.

### [To Do] OBS-0105 Best-effort dump of all recorders for the panic hook
P0 · M2 · Task · deps: OBS-0103, OBS-0101
Done when: `dump_all_for_panic(writer)` never blocks (contended recorders are skipped and named), bounds total output size, and writes via the given writer; SRV-F16 calls it from the panic hook; a child-process test panics and finds the dump in its output.

### [To Do] OBS-0106 Flight recorder memory budget and capacity configuration
P1 · M2 · Task · deps: OBS-0100 · labels: nfr
Done when: the capacity is a constructor parameter supplied from the flight-recorder capacity key (CFG-F11); docs give memory as components x capacity x entry size with a worked 100k-partition example recorded in the NFR-F05 budget; a test asserts the per-recorder footprint.

### [To Do] OBS-0107 Render selected recorders for test and simulation failure reports
P1 · M2 · Task · deps: OBS-0103, OBS-0101 · labels: test
Done when: `render(components)` returns the formatted recorders for the given ids in the same layout as `/debug` output, for TEST-F05 reports; unit test.

### [To Do] OBS-0108 Document the flight recorder API and per-component entry types
P1 · M2 · Task · deps: OBS-0104, OBS-0105 · labels: docs
Done when: docs/standards/debuggability.md describes the API, the entry layout, dump triggers and how a component defines its entry enum, with a short example; links resolve.

## [To Do] OBS-F07 Event catalogs
P1 · M2 · Feature · deps: OBS-F01
Scope: `events.rs` per crate, generated docs/events.md, stability rules.

### [To Do] OBS-0109 `events.rs` convention and `EventName` type
P1 · M2 · Task · deps: none · ref: docs/standards/debuggability.md#events-for-state-transitions
Done when: each crate declares `pub const` `EventName`s in `events.rs` with doc lines for trigger, level, fields and reason type; events are emitted as `event = events::NAME.as_str()`; the observability crate's own events (`log_filter_changed`, `diagnostics_listener_started`, `readiness_changed`) follow the template; docs/standards/debuggability.md links the template.

### [To Do] OBS-0110 Check that events are declared, used and never raw strings
P1 · M2 · Task · deps: OBS-0109, FND-F04
Done when: an xtask check fails on `event = "..."` string literals outside `events.rs`, on declared events that are never emitted, and on names that are not snake_case; it runs in `cargo xtask ci`; fixtures prove each failure.

### [To Do] OBS-0111 Reason enum convention with stable `Display` and a snapshot check
P1 · M2 · Task · deps: OBS-0109
Done when: reason enums render as stable snake_case plus fields (`follower_lagged{broker_id=3,lag_ms=30000}`); a test helper renders every variant of a reason enum to a committed snapshot so renames show up in review; used by the observability crate's own reasons.

### [To Do] OBS-0112 Generate docs/events.md from every crate's `events.rs`
P1 · M2 · Task · deps: OBS-0109, FND-F04 · labels: docs
Done when: `cargo xtask docs events` writes name, crate, level, trigger, fields and reason values, sorted by name; CI fails when the page is stale; DOCS-F06 publishes it.

### [To Do] OBS-0113 Event stability rules: stable events cannot disappear silently
P1 · M2 · Task · deps: OBS-0112
Done when: events are marked stable or experimental; the check fails when a stable event present in the committed catalog is renamed or removed unless it is listed in a removed-events section with a changelog entry; docs/standards/debuggability.md states the rule.

### [To Do] OBS-0114 Driver helper: one call logs the event, records the flight recorder entry and counts the transition
P1 · M2 · Task · deps: OBS-0109, OBS-0100, OBS-F04
Done when: `record_transition(event, reason, recorder, metrics)` logs at the declared level with standard fields, records the entry, and increments `alluvium_<subsystem>_transitions_total{event,reason}` (reason as the variant name, a bounded label); a test shows all three effects from one call.

## [To Do] OBS-F08 Diagnostics endpoint
P0 · M2 · Feature · deps: OBS-F04
Scope: Separate HTTP listener: loopback by default, authentication required elsewhere, the routes in the standard.

### [To Do] OBS-0115 Diagnostics HTTP server: separate listener, bounded connections and timeouts
P0 · M2 · Task · deps: none · ref: docs/standards/debuggability.md#diagnostics-endpoint
Done when: a `hyper`-based server (dependency justified) binds the address passed in by the binary (default `127.0.0.1`), runs as a supervised task, caps concurrent connections, request size and header size, applies read and write timeouts, and answers non-GET methods with 405 except on `/debug/log-level`; emits `diagnostics_listener_started` with the bound address; integration tests including a slow-client timeout.

### [To Do] OBS-0116 Non-loopback binds require authentication: startup refusal and bearer tokens
P0 · M2 · Task · deps: OBS-0115, TYPE-F08 · labels: security · ref: docs/standards/security.md
Done when: binding a non-loopback address without a bearer token or mTLS configured fails startup with an error naming the settings; bearer tokens come from a file or environment variable as `Secret`, are compared in constant time, and failures return 401 with a rate-limited `diagnostics_auth_failed` event and `alluvium_diagnostics_auth_failures_total`; tests.

### [To Do] OBS-0117 mTLS for the diagnostics endpoint
P1 · M2 · Task · deps: OBS-0116 · labels: security
Done when: with a CA bundle configured the listener requires client certificates verified by rustls (built in this crate; network is the same layer); handshake failures are counted; certificate and key files reload on change; tests with valid, untrusted and missing client certificates.

### [To Do] OBS-0118 Router, JSON error format and the `DebugSource` registration contract
P0 · M2 · Task · deps: OBS-0115, OBS-F04
Done when: routes dispatch by path; errors use `{ "error": <code>, "message": ... }` with 404 for unknown paths and ids and 503 while a source is not yet registered; components register `DebugSource` implementations at startup; responses are size-bounded; request metrics `alluvium_diagnostics_requests_total{route,status}` and `alluvium_diagnostics_request_seconds{route}`; tests.

### [To Do] OBS-0119 Route `/metrics`: Prometheus and OpenMetrics exposition
P0 · M2 · Task · deps: OBS-0118
Done when: the route serves OBS-F04's encoder with content negotiation on `Accept`; a scrape test parses the output; scraping is read-only and does not reset any metric.

### [To Do] OBS-0120 Route `/debug/config`: effective config with sources from a registered source
P1 · M2 · Task · deps: OBS-0118 · labels: security
Done when: the route serves the CFG-supplied effective config (key, value, source = default, file, env or dynamic); as defence in depth any entry marked sensitive is rendered `[hidden]` even if a source forgets; a test with a fake source proves both; CFG-F03 supplies the real source.

### [To Do] OBS-0121 Route `/debug/tasks`: supervised tasks with name, state and age
P1 · M2 · Task · deps: OBS-0118
Done when: the JSON contract lists every supervised task with name, state, age and parent; SRV-F02's supervisor implements the source; a fake-source test pins the schema.

### [To Do] OBS-0122 Route `/debug/partitions/{topic}/{partition}`: replication and log state plus flight recorder
P1 · M2 · Task · deps: OBS-0118, OBS-0103
Done when: the path is validated (topic name rules, non-negative partition) before lookup; the response holds leader epoch, ISR, high watermark, log start and end offsets, last stable offset, segment count and the partition's flight recorder; unknown partitions return 404; a fake-source test pins the schema; the replication driver (REPL-F17) supplies the data.

### [To Do] OBS-0123 Route `/debug/groups/{group_id}`: group state plus flight recorder
P1 · M2 · Task · deps: OBS-0118, OBS-0103
Done when: percent-encoded group ids decode safely (any UTF-8, length bounded); the response holds protocol type, state, generation or group epoch, members (ids only, no client-supplied metadata bytes) and the group's flight recorder; unknown groups return 404; fake-source test; GRP-F11 supplies the data.

### [To Do] OBS-0124 Route `/debug/raft`: quorum state plus flight recorder
P1 · M2 · Task · deps: OBS-0118, OBS-0103
Done when: the response holds role, epoch, leader id, voted id, high watermark, log end offset, voters and observers with their fetch state, and the quorum's flight recorder; fake-source test; RAFT-F14 supplies the data.

### [To Do] OBS-0125 Diagnostics hardening tests: oversized requests, slow clients, path tricks and method checks
P0 · M2 · Task · deps: OBS-0122, OBS-0123, OBS-0116 · labels: security, test
Done when: tests send oversized headers and bodies, slowloris-style partial requests, `..` and encoded-slash paths, very long ids and disallowed methods, and assert bounded memory, timeouts, 400/404/405/413 responses and no panics.

### [To Do] OBS-0126 Operator docs for the diagnostics endpoint
P1 · M2 · Task · deps: OBS-0119, OBS-0120, OBS-0121, OBS-0124, OBS-0117 · labels: docs
Done when: docs/guides/debugging.md documents every route with a curl example and sample output, the loopback default, and how to configure bearer tokens or mTLS for remote access.

## [To Do] OBS-F09 Health and readiness
P1 · M2 · Feature · deps: OBS-F08
Scope: Liveness and readiness semantics (metadata caught up, logs loaded, listeners up).

### [To Do] OBS-0127 Readiness model: named conditions owned by components
P1 · M2 · Task · deps: OBS-0109
Done when: `Readiness` holds named conditions (`metadata_caught_up`, `logs_loaded`, `listeners_bound`, and for controllers `quorum_joined`) that only their owner can set; ready means every condition required by the node's roles holds; changes emit `readiness_changed` (condition, old, new); unit tests.

### [To Do] OBS-0128 Required readiness conditions per role (broker, controller, combined)
P1 · M2 · Task · deps: OBS-0127 · labels: docs
Done when: a documented table states which conditions each role needs, matching the standard (caught up on metadata, logs loaded) plus listeners bound; the table is code and a test checks it against the docs.

### [To Do] OBS-0129 Route `/health/live`: liveness with a runtime watchdog
P1 · M2 · Task · deps: OBS-0127
Done when: the route returns 200 while a watchdog tick from the runtime is fresher than a threshold and 503 when the runtime is stalled; it never depends on cluster state; a test stalls the watchdog and sees 503.

### [To Do] OBS-0130 Route `/health/ready`: readiness with per-condition detail and drain behaviour
P1 · M2 · Task · deps: OBS-0128
Done when: 200 when ready, else 503; the JSON body lists each condition with its value and when it last changed; shutdown marks the node not ready before draining; integration test over a startup and shutdown sequence with fake owners.

### [To Do] OBS-0131 Readiness metrics
P2 · M2 · Task · deps: OBS-0127, OBS-F04
Done when: `alluvium_node_ready` and `alluvium_node_readiness_condition{condition}` gauges follow the model; test.

### [To Do] OBS-0132 Document liveness and readiness for operators and orchestrators
P2 · M2 · Task · deps: OBS-0129, OBS-0130 · labels: docs
Done when: docs/guides/debugging.md explains each condition, typical causes of a node staying unready, and probe settings that OPS-F04 manifests reuse.

## [To Do] OBS-F10 tokio-console support
P2 · M2 · Feature · deps: OBS-F01
Scope: Development-build feature exposing task internals.

### [To Do] OBS-0133 `tokio-console` Cargo feature for development builds
P2 · M2 · Task · deps: none
Done when: the feature adds the console-subscriber layer and requires `--cfg tokio_unstable`; it is off by default; CI shows release artifacts do not contain `console-subscriber` (`cargo tree` check); dependency justification recorded.

### [To Do] OBS-0134 Console listener bound to loopback only
P2 · M2 · Task · deps: OBS-0133 · labels: security
Done when: the console server binds `127.0.0.1` (port configurable) and refuses non-loopback addresses at startup; test.

### [To Do] OBS-0135 Supervised task names visible in tokio-console
P3 · M2 · Task · deps: OBS-0133, SRV-F02 · labels: docs
Done when: with the feature on, SRV-F02's supervisor spawns through `tokio::task::Builder::name`, so console shows the same names as `/debug/tasks`; a manual check procedure is documented in docs/guides/debugging.md.

## [To Do] OBS-F11 Redaction verification
P0 · M2 · Feature · deps: OBS-F01
Scope: Tests asserting logs, metrics and debug output never contain secrets or record payloads.

### [To Do] OBS-0136 Canary framework: seed unique secrets and payloads, then scan every output channel
P0 · M2 · Task · deps: none · labels: security, test
Done when: a test utility generates unique canary strings, captures log output (JSON writer), `/metrics`, every `/debug` route, flight-recorder dumps and error `Display` chains, and reports every channel where a canary appears; self-test proves detection.

### [To Do] OBS-0137 Logs at trace level never contain record keys, values or headers
P0 · M2 · Task · deps: OBS-0136, API-F00, API-F01 · labels: security, test
Done when: a single-node test produces and consumes records whose keys, values and headers are canaries with the filter at `trace`; no canary appears in any log line; payload-bearing values print as `PayloadSummary`.

### [To Do] OBS-0138 Metrics never contain secrets, payloads or unbounded identifiers
P0 · M2 · Task · deps: OBS-0136 · labels: security, test
Done when: after a canary workload `/metrics` contains no canary and no label value outside the allowed label sets (topic, capped client ids on quota families only); test.

### [To Do] OBS-0139 Diagnostics routes and flight recorders never contain payloads or secrets
P0 · M2 · Task · deps: OBS-0136, OBS-0122, OBS-0120 · labels: security, test
Done when: every `/debug` route and every flight-recorder dump after the canary workload is free of canaries; test.

### [To Do] OBS-0140 Panic reports never contain payloads or secrets
P0 · M2 · Task · deps: OBS-0136, OBS-0105, SRV-F16 · labels: security, test
Done when: a child-process test triggers a panic while canary-bearing state is live; the panic report (backtrace, span stack, flight-recorder dump) contains no canary.

### [To Do] OBS-0141 SASL credentials and tokens never appear in any output
P0 · M4 · Task · deps: OBS-0136, SEC-F03, SEC-F04, SEC-F05, SEC-F10, CLI-F04 · labels: security, test
Done when: the canary workload authenticates with PLAIN, SCRAM-SHA-256/512, OAUTHBEARER (canary JWT) and delegation tokens, including failed attempts; no password, salted hash, JWT or token HMAC appears in logs, metrics, `/debug` routes or `dump metadata` output.

### [To Do] OBS-0142 Run the redaction suite in CI on every PR
P0 · M2 · Task · deps: OBS-0137, OBS-0138, OBS-0139, OBS-0140 · labels: security, docs
Done when: `cargo xtask ci` runs the redaction tests; docs/standards/security.md names the suite as the enforcement of "never log secrets or payloads" and explains how to add a channel.

## [To Do] OBS-F12 OTLP export
P2 · M9 · Feature · deps: OBS-F01
Scope: Optional OpenTelemetry export of spans and metrics via a `tracing` layer.

### [To Do] OBS-0143 Spike: OTLP export design (crates, feature flag, sampling, metrics path)
P2 · M9 · Spike · deps: none · ref: docs/adr/0005-observability-model.md
Done when: an accepted ADR updates ADR-0005's deferral: chosen crates (`opentelemetry`, `tracing-opentelemetry`, OTLP exporter) with their dependency cost, an `otlp` Cargo feature that is off by default, sampling policy, bounded export queues, and whether metrics are pushed over OTLP or left to scraping.

### [To Do] OBS-0144 Export spans over OTLP through a `tracing` layer
P2 · M9 · Task · deps: OBS-0143
Done when: with the feature and config enabled, request, partition and driver spans export with standard fields as attributes and parent-based ratio sampling; the exporter queue is bounded with `alluvium_otlp_dropped_spans_total`; TLS uses rustls; integration test against an in-process collector.

### [To Do] OBS-0145 Export metrics over OTLP from the explicit registry
P3 · M9 · Task · deps: OBS-0143
Done when: a periodic exporter converts the registry's families to OTLP metrics without a global meter provider; interval and endpoint come from config; test against the in-process collector.

### [To Do] OBS-0146 OTLP resource attributes and configuration
P2 · M9 · Task · deps: OBS-0144
Done when: service.name `alluvium`, service.instance.id from node id, cluster id and version are attached; endpoint, protocol (gRPC or HTTP), headers (as `Secret`) and sampling ratio are read from the `alluvium.*` OTLP keys defined in CFG-F11; invalid settings fail startup; tests.

### [To Do] OBS-0147 OTLP output passes the redaction suite
P2 · M9 · Task · deps: OBS-0144, OBS-0145, OBS-0136 · labels: security, test
Done when: the OBS-F11 canary workload with OTLP enabled shows no canary in exported span attributes, events or metric labels.

### [To Do] OBS-0148 Document OTLP export
P3 · M9 · Task · deps: OBS-0146 · labels: docs
Done when: docs/guides/debugging.md explains enabling OTLP, collector setup, sampling and the overhead measured in a benchmark run.

## [To Do] OBS-F13 Profiling endpoint
P3 · M9 · Feature · deps: OBS-F08
Scope: On-demand CPU and heap profiling via the diagnostics endpoint.

### [To Do] OBS-0149 Spike: on-demand CPU and heap profiling approach
P3 · M9 · Spike · deps: none · labels: security
Done when: an accepted ADR chooses the CPU sampler and heap profiler (for example a pprof-compatible sampler and jemalloc profiling), reviews their `unsafe` and signal-handler use under the security standard, and fixes limits (duration, frequency, one profile at a time) and default-off gating.

### [To Do] OBS-0150 `/debug/pprof/profile?seconds=N`: CPU profile in pprof format
P3 · M9 · Task · deps: OBS-0149
Done when: the route samples for N seconds (max 60), returns a pprof protobuf, allows one profile at a time (409 otherwise), requires the endpoint's authentication and emits `profiling_started` and `profiling_finished`; test parses the output.

### [To Do] OBS-0151 `/debug/pprof/heap`: heap profile
P3 · M9 · Task · deps: OBS-0149
Done when: with heap profiling enabled at startup the route returns a pprof heap profile; when disabled it returns 404 with a hint; test.

### [To Do] OBS-0152 Profiling routes disabled unless explicitly enabled
P3 · M9 · Task · deps: OBS-0150, OBS-0151 · labels: security
Done when: the routes exist only when the profiling key in CFG-F11 is true (default false); a test shows 404 by default.

### [To Do] OBS-0153 Profiling overhead benchmark
P3 · M9 · Task · deps: OBS-0150, TEST-F11 · labels: perf
Done when: a produce/consume benchmark run with and without an active CPU profile records the throughput loss (target below 5 percent at the default frequency) in the docs.

### [To Do] OBS-0154 Document capturing and reading profiles
P3 · M9 · Task · deps: OBS-0152 · labels: docs
Done when: docs/guides/debugging.md shows capturing CPU and heap profiles with curl and viewing them with pprof and flame graphs.
