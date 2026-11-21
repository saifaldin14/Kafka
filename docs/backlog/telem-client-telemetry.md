# TELEM: Client telemetry

> Crates: `alluvium-server` · Kafka counterpart: KIP-714 (`server/metrics/ClientMetricsManager`) · Milestones: M8

Broker-side client metrics subscriptions and ingestion (KIP-714).

## [To Do] TELEM-F01 Telemetry subscriptions
P1 · M8 · Feature · deps: CFG-F04
Scope: GetTelemetrySubscriptions, subscription matching, client instance ids.

### [To Do] TELEM-0001 Client instance ids and instance metadata
P1 · M8 · Task · deps: none · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java processGetTelemetrySubscriptionRequest @4.4.0
Done when: GetTelemetrySubscriptions with a zero client instance id gets a newly generated non-reserved UUID from the injected RNG; a known id reuses its instance; instance metadata records client id, software name and version, source address and port from the request context; tests.

### [To Do] TELEM-0002 Bounded client instance cache with expiry
P1 · M8 · Task · deps: TELEM-0001 · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java clientInstance @4.4.0
Done when: instances live in an LRU cache of at most 16 384 entries and expire after max(60 s, 3 × push interval) without requests, using the injected clock; eviction and expiry are counted; the instance-count gauge equivalent is exported; tests.

### [To Do] TELEM-0003 Subscription matching against client selectors
P1 · M8 · Task · deps: TELEM-0001 · ref: server/src/main/java/org/apache/kafka/server/metrics/ClientMetricsInstanceMetadata.java isMatch @4.4.0
Done when: a subscription's `match` patterns over `client_instance_id`, `client_id`, `client_software_name`, `client_software_version`, `client_source_address` and `client_source_port` must all fully match (regex) for the client; an empty match selects all clients; tests per selector.

### [To Do] TELEM-0004 Compute a client's effective subscription
P1 · M8 · Task · deps: TELEM-0003 · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java createClientInstance @4.4.0
Done when: matched subscriptions are combined as the union of metric prefixes (`*` means all metrics) with the minimum push interval; the subscription id is CRC32C over the sorted metrics and interval XOR the Java hash of the instance id, as Kafka computes it; the response lists accepted compression types (zstd, lz4, gzip, snappy), delta temporality and `telemetry.max.bytes`; golden tests against Kafka.

### [To Do] TELEM-0005 Re-evaluate instances when subscriptions change
P1 · M8 · Task · deps: TELEM-0004 · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java updateSubscription @4.4.0
Done when: every subscription change increments a subscription version, and an instance computed for an older version is recomputed on its next request with a new subscription id; tests.

### [To Do] TELEM-0006 Throttle repeated GetTelemetrySubscriptions
P1 · M8 · Task · deps: TELEM-0004 · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java validateGetRequest @4.4.0
Done when: a second request before the push interval elapses returns THROTTLING_QUOTA_EXCEEDED unless the instance's last error was UNKNOWN_SUBSCRIPTION_ID or UNSUPPORTED_COMPRESSION_TYPE; the throttle counter is updated; tests.

### [To Do] TELEM-0007 Advertise telemetry APIs only with an exporter configured
P1 · M8 · Task · deps: TELEM-0004 · ref: server/src/main/java/org/apache/kafka/server/DefaultApiVersionManager.java @4.4.0
Done when: GetTelemetrySubscriptions and PushTelemetry appear in ApiVersions only when a telemetry exporter (TELEM-F04) is enabled, as Kafka does; with none configured they are absent; tests for both.

### [To Do] TELEM-0008 Unexpected telemetry errors map to INVALID_REQUEST
P1 · M8 · Task · deps: TELEM-0004
Done when: any unexpected failure while handling GetTelemetrySubscriptions or PushTelemetry returns INVALID_REQUEST like Kafka's handler, logged once with the client instance id; test with an injected failure.

## [To Do] TELEM-F02 Telemetry ingestion
P1 · M8 · Feature · deps: TELEM-F01
Scope: PushTelemetry decoding (OTLP), compression, validation, limits.

### [To Do] TELEM-0009 PushTelemetry validation
P1 · M8 · Task · deps: none · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java validatePushRequest @4.4.0
Done when: a null or reserved client instance id returns INVALID_REQUEST; a push after the instance sent `terminating=true` INVALID_REQUEST; a push before the interval (unless terminating) THROTTLING_QUOTA_EXCEEDED; a mismatched subscription id UNKNOWN_SUBSCRIPTION_ID; an unsupported compression type UNSUPPORTED_COMPRESSION_TYPE; a payload over `telemetry.max.bytes` TELEMETRY_TOO_LARGE; the terminating flag and last error are recorded either way; one test per error.

### [To Do] TELEM-0010 Decide how telemetry payloads are handled
P1 · M8 · Spike · deps: none · ref: server/src/main/java/org/apache/kafka/server/metrics/DefaultClientTelemetryPayload.java @4.4.0
Done when: ADR `client-telemetry-payloads` accepted in docs/adr/ deciding pass-through versus decoding of OTLP payloads, the protobuf dependency (with the dependency policy justification), decompression limits, and what the built-in exporter receives.

### [To Do] TELEM-0011 Bounded payload decompression
P1 · M8 · Task · deps: TELEM-0009, REC-F10 · ref: server/src/main/java/org/apache/kafka/server/metrics/DefaultClientTelemetryPayload.java @4.4.0
Done when: zstd, lz4, gzip and snappy payloads are decompressed with output capped at `telemetry.max.bytes` using the bounded codecs; exceeding the cap fails the push with the mapped error and no unbounded allocation; tests per codec including a decompression bomb.

### [To Do] TELEM-0012 Decode OTLP metrics payloads per the ADR
P1 · M8 · Task · deps: TELEM-0010, TELEM-0011 · labels: decoder · ref: server/src/main/java/org/apache/kafka/server/metrics/ClientTelemetryExporterPlugin.java @4.4.0
Done when: payloads decode into OTLP `MetricsData` with limits on message size, nesting and repeated fields; malformed payloads are rejected with the error chosen in the ADR without panics; unit tests with payloads captured from the Java client.

### [To Do] TELEM-0013 Fuzz targets for telemetry payloads
P1 · M8 · Task · deps: TELEM-0012, TEST-F07 · labels: decoder, test
Done when: cargo-fuzz targets cover decompression plus OTLP decoding and run without panics, hangs or unbounded allocation.

### [To Do] TELEM-0014 Hand payloads to the exporter
P1 · M8 · Task · deps: TELEM-0009 · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java processPushTelemetryRequest @4.4.0
Done when: non-empty payloads go to the configured exporter with the push interval and request context; exporter failure returns INVALID_RECORD and is counted; empty payloads (for example the final terminating push) are accepted without export; plugin export time is measured; tests.

### [To Do] TELEM-0015 Client telemetry compatibility tests
P1 · M8 · Task · deps: TELEM-0014, TEST-F08 · labels: test
Done when: the Java 4.4.0 client with `enable.metrics.push=true` and librdkafka with telemetry enabled subscribe, push and terminate against Alluvium with a test exporter receiving their metrics.

## [To Do] TELEM-F03 Client metrics resources
P1 · M8 · Feature · deps: CFG-F04
Scope: Client-metrics config resources and ListConfigResources.

### [To Do] TELEM-0016 CLIENT_METRICS config resource validation
P1 · M8 · Task · deps: CTRL-F11 · ref: server/src/main/java/org/apache/kafka/server/metrics/ClientMetricsConfigs.java validate @4.4.0
Done when: resource names must be non-empty (INVALID_REQUEST); only `metrics`, `interval.ms` and `match` are accepted (unknown keys INVALID_REQUEST); `interval.ms` must be within 100–3 600 000 (INVALID_REQUEST); `match` entries must be `<selector>=<regex>` with a known selector and a valid regex (INVALID_CONFIG); validation runs where the controller validates config records; tests per rule.

### [To Do] TELEM-0017 Apply client-metrics subscriptions from metadata
P1 · M8 · Task · deps: TELEM-0016, SRV-F14 · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java updateSubscription @4.4.0
Done when: the metadata publisher applies CLIENT_METRICS config records to the telemetry manager as subscription upserts and removes a subscription when its configs are deleted; defaults are empty metrics, 300 000 ms interval and empty match; tests.

### [To Do] TELEM-0018 ListConfigResources
P1 · M8 · Task · deps: TELEM-0017, GRP-F08 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListConfigResources @4.4.0
Done when: v0 lists client-metrics resources; v1 lists resources of the requested types (GROUP, CLIENT_METRICS, BROKER_LOGGER, BROKER, TOPIC), all of them when none are requested, and returns UNSUPPORTED_VERSION when any requested type is unsupported; brokers come from the request's listener view of metadata; tests per type.

### [To Do] TELEM-0019 Describe and alter client-metrics configs
P1 · M8 · Task · deps: TELEM-0017, CFG-F06 · labels: test
Done when: DescribeConfigs and IncrementalAlterConfigs for CLIENT_METRICS resources return and change `metrics`, `interval.ms` and `match` with sources as Kafka does, and a test shows a change reaching clients on their next GetTelemetrySubscriptions.

### [To Do] TELEM-0020 Document client-metrics subscriptions
P2 · M8 · Task · deps: TELEM-0018 · labels: docs
Done when: a docs page explains subscription keys, matching rules, push intervals and how to manage subscriptions with `kafka-client-metrics.sh`.

## [To Do] TELEM-F04 Telemetry export
P2 · M8 · Feature · deps: TELEM-F02
Scope: Built-in export to OTLP and Prometheus.

### [To Do] TELEM-0021 Design the built-in telemetry exporters
P2 · M8 · Spike · deps: TELEM-0010
Done when: ADR `client-telemetry-export` accepted in docs/adr/ covering OTLP forwarding and Prometheus re-exposition, bounded queues and label cardinality, `alluvium.client.telemetry.*` keys (defined in CFG-F11), and that export is off by default.

### [To Do] TELEM-0022 Consume telemetry exporter configuration
P2 · M8 · Task · deps: TELEM-0021, CFG-F11
Done when: the exporter type, OTLP endpoint, headers (secret type), queue size and series cap are read from `alluvium.client.telemetry.*`; with nothing set no exporter runs; invalid values fail startup with the key name; tests.

### [To Do] TELEM-0023 OTLP forwarding exporter
P2 · M8 · Task · deps: TELEM-0022, TELEM-0012
Done when: payloads are forwarded to an OTLP/HTTP endpoint over rustls with timeouts, a bounded queue, retries with backoff and drop-on-overflow counted in a metric; client instance attributes are attached as resource attributes; tests against a fake OTLP receiver.

### [To Do] TELEM-0024 Prometheus re-exposition exporter
P2 · M8 · Task · deps: TELEM-0022, TELEM-0012
Done when: received client metrics are exposed on the metrics endpoint under a separate prefix with a cap on distinct client instances (oldest evicted), and series limits are documented; tests assert bounded series under churn.

### [To Do] TELEM-0025 Client metrics broker metrics
P2 · M8 · Task · deps: TELEM-0023 · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java @4.4.0
Done when: equivalents of Kafka's client-metrics group (instance-count, unknown-subscription-request, throttle, plugin-export, plugin-error and plugin-export-time) are exported with docs/metrics.md rows.

### [To Do] TELEM-0026 Prove export is off by default and invisible when unused
P2 · M8 · Task · deps: TELEM-0023, TELEM-0024, TEST-F09 · labels: test
Done when: a test shows no exporter and no telemetry APIs in ApiVersions by default, and the compatibility and differential suites pass with an exporter enabled but no client pushing.

### [To Do] TELEM-0027 Document built-in telemetry export
P2 · M8 · Task · deps: TELEM-0023, TELEM-0024 · labels: docs
Done when: a docs page explains enabling each exporter, its keys, limits and dropped-data metrics.

## [To Do] TELEM-F05 Telemetry limits and security
P1 · M8 · Feature · deps: TELEM-F02
Scope: Size limits, rate limits, authorization.

### [To Do] TELEM-0028 Size limits for telemetry payloads
P1 · M8 · Task · deps: TELEM-0011 · labels: security
Done when: compressed payloads above `telemetry.max.bytes` are rejected before decompression and decompressed output is capped at the same limit; tests at the boundary.

### [To Do] TELEM-0029 Protection against client-instance floods
P1 · M8 · Task · deps: TELEM-0002 · labels: security
Done when: a client creating new instance ids on every GetTelemetrySubscriptions cannot grow memory beyond the cache bound; a test with 100 000 new ids shows bounded memory and eviction counts.

### [To Do] TELEM-0030 Bound and sanitize client-provided instance metadata
P1 · M8 · Task · deps: TELEM-0001 · labels: security
Done when: client id, software name and version are length-bounded before storage, used only for matching and for logs with escaping, and never used as metric labels; tests with oversized and control-character inputs.

### [To Do] TELEM-0031 Never log telemetry payloads
P1 · M8 · Task · deps: TELEM-0014, OBS-F11 · labels: security
Done when: payloads appear in logs and debug output only as length and CRC32C; the redaction test suite covers the telemetry paths.

### [To Do] TELEM-0032 Telemetry requests are subject to request quotas
P1 · M8 · Task · deps: TELEM-0009, QUOTA-F04 · labels: test
Done when: GetTelemetrySubscriptions and PushTelemetry count toward request-time quotas and are throttled like other requests; test.

### [To Do] TELEM-0033 Authorization for telemetry configuration
P1 · M8 · Task · deps: TELEM-0018, SEC-F08 · labels: security
Done when: tests confirm Kafka's rules: altering CLIENT_METRICS resources requires ALTER_CONFIGS on the cluster, describing them and ListConfigResources require DESCRIBE_CONFIGS on the cluster, and telemetry push and subscription requests need no ACL; the threat model notes the client-instance-id spoofing limitation shared with Kafka.
