# QUOTA: Quotas and throttling

> Crates: `alluvium-server` (quota manager), `alluvium-controller` · Kafka counterpart: `server/quota`, `ClientQuotaManager` · Milestones: M4

Client quotas by user and client id, produce, fetch and request quotas, controller mutation quotas, connection rate quotas, throttling semantics and quota administration.

## [To Do] QUOTA-F01 Quota model
P0 · M4 · Feature · deps: META-F02
Scope: Entities (user, client-id, user+client-id, ip), defaults, precedence resolution.

### [To Do] QUOTA-0001 Quota entities: user, client-id, user+client-id and ip, with defaults and sanitized user names
P0 · M4 · Task · deps: none · ref: server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java KafkaQuotaEntity, clients/src/main/java/org/apache/kafka/common/utils/internals/Sanitizer.java @4.4.0
Done when: Entity types and the `<default>` entity are modelled with Kafka's semantics; user principals are sanitized like Kafka's `Sanitizer` for tags; IP entities are separate from client entities; unit tests.

### [To Do] QUOTA-0002 Quota types and keys: producer_byte_rate, consumer_byte_rate, request_percentage, controller_mutation_rate, connection_creation_rate
P0 · M4 · Task · deps: QUOTA-0001 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java @4.4.0
Done when: Each key maps to its quota type with Kafka's value types and unlimited defaults; `connection_creation_rate` applies only to IP entities; tests.

### [To Do] QUOTA-0003 Quota limit lookup with Kafka's precedence order
P0 · M4 · Task · deps: QUOTA-0002 · ref: server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java DefaultQuotaCallback.findQuota @4.4.0
Done when: For (user, client-id) the limit is found in order user+client, user+default client, default user+client, default user+default client, user, default user, client, default client, restricted to the components present, as `DefaultQuotaCallback.findQuota`; table tests cover every level.

### [To Do] QUOTA-0004 Metric tags select the sharing group: which clients share one quota sensor
P0 · M4 · Task · deps: QUOTA-0003 · ref: server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java DefaultQuotaCallback.quotaMetricTags @4.4.0
Done when: Tags (user, client-id) are derived as `quotaMetricTags` so all clients matched by a user-level quota share one sensor while user+client quotas are per pair; tests show sharing per level.

### [To Do] QUOTA-0005 Enabled-quota-type fast path that gives the same answer as the full lookup
P0 · M4 · Task · deps: QUOTA-0004 · ref: server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java updateQuotaTypes, quotaTypesEnabled @4.4.0
Done when: When only one entity type is configured, tags come from the fast path like Kafka's `quotaTypesEnabled`; a property test compares it with the full lookup over random configurations.

### [To Do] QUOTA-0006 Apply client quota deltas from the metadata image
P0 · M4 · Task · deps: QUOTA-0004, SRV-F14 · ref: core/src/main/scala/kafka/server/metadata/ClientQuotaMetadataManager.scala, metadata/src/main/java/org/apache/kafka/metadata/publisher/DynamicClientQuotaPublisher.java @4.4.0
Done when: Quota publisher updates add, change and remove entity quotas for each quota type and the IP connection quotas; removal reverts to defaults; existing sensors get the new bound; tests with crafted deltas.

### [To Do] QUOTA-0007 Reject a custom `client.quota.callback.class`
P0 · M4 · Task · deps: QUOTA-0002, CFG-F07 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java CLIENT_QUOTA_CALLBACK_CLASS_CONFIG @4.4.0
Done when: A configured callback class fails startup with a message explaining the deviation; recorded in docs/parity.md; test.

### [To Do] QUOTA-0008 Property test of quota resolution against a brute-force model
P0 · M4 · Task · deps: QUOTA-0005 · labels: test
Done when: Random quota configurations and (user, client-id) pairs give the same limit and tags as a brute-force model of Kafka's rules; regressions committed.

### [To Do] QUOTA-0009 Document quota entities and precedence
P1 · M4 · Task · deps: QUOTA-0004 · labels: docs
Done when: Docs show the precedence table with examples of shared and per-client quotas; doc build passes.

## [To Do] QUOTA-F02 Produce byte-rate quotas
P0 · M4 · Feature · deps: QUOTA-F01, QUOTA-F10
Scope: Measure, compute throttle time, delay the response.

### [To Do] QUOTA-0010 Record produce request bytes and compute throttle time per entity
P0 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest (quotas.produce.maybeRecordAndGetThrottleTimeMs), server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java maybeRecordAndGetThrottleTimeMs, quotasEnabled @4.4.0
Done when: Each produce request's size is recorded on the entity's byte-rate sensor; a violation yields a throttle time from QUOTA-F10; with no quotas configured no sensor is created; tests on the simulated clock.

### [To Do] QUOTA-0011 Combine produce bandwidth and request-time throttles
P0 · M4 · Task · deps: QUOTA-0010, QUOTA-F04 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest @4.4.0
Done when: The response throttle is the maximum of the bandwidth and request throttles and the channel is throttled by the larger one; the request quota is not enforced for acks=0; tests per combination.

### [To Do] QUOTA-0012 acks=0 produce: no response, but the channel is still throttled on bandwidth violation
P0 · M4 · Task · deps: QUOTA-0011 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest (sendNoOpResponseExemptThrottle) @4.4.0
Done when: An acks=0 produce over quota sends no response yet the connection is muted for the throttle time; tests.

### [To Do] QUOTA-0013 Set `throttle_time_ms` in Produce responses for every version
P0 · M4 · Task · deps: QUOTA-0011
Done when: Produce responses carry the computed throttle time in all supported versions; tests per version.

### [To Do] QUOTA-0014 Port Kafka's client quota manager tests for produce
P0 · M4 · Task · deps: QUOTA-0010 · labels: test · ref: server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java @4.4.0
Done when: Cases from ClientQuotaManagerTest (quota violation, recovery, default and override quotas, sensor expiry) pass with identical throttle times on the simulated clock.

### [To Do] QUOTA-0015 Produce quota compatibility test with the Java client and librdkafka
P0 · M4 · Task · deps: QUOTA-0012, QUOTA-0013, TEST-F08 · labels: test
Done when: Producers with a 1 MiB/s quota converge to within 10% of the quota and report client-side throttle metrics; matches Kafka 4.4.0 behavior.

## [To Do] QUOTA-F03 Fetch byte-rate quotas
P0 · M4 · Feature · deps: QUOTA-F01, QUOTA-F10
Scope: Measure, compute throttle time, delay the response.

### [To Do] QUOTA-0016 Record consumer fetch response bytes and compute throttle time
P0 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest (quotas.fetch.maybeRecordAndGetThrottleTimeMs) @4.4.0
Done when: After building a consumer fetch response its size is recorded on the entity's sensor; follower fetches are not recorded here (replication quotas are REPL-F16); tests.

### [To Do] QUOTA-0017 Return an empty throttled fetch response and un-record its bytes when throttled
P0 · M4 · Task · deps: QUOTA-0016, QUOTA-F04 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest (unrecordQuotaSensor, getThrottledResponse) @4.4.0
Done when: When the fetch or request quota is violated the recorded bytes are un-recorded and an empty response with the throttle time is returned, keeping the fetch session consistent; tests.

### [To Do] QUOTA-0018 Cap consumer fetch max bytes to the quota window's allowance
P0 · M4 · Task · deps: QUOTA-0016 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest (maxValueInQuotaWindow), server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java maxValueInQuotaWindow @4.4.0
Done when: A consumer fetch's max bytes is limited to the bytes the entity may fetch in one quota window, so a single fetch cannot exceed the quota; tests.

### [To Do] QUOTA-0019 Follower fetches are exempt from client fetch and request quotas
P0 · M4 · Task · deps: QUOTA-0016 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest (sendResponseExemptThrottle) @4.4.0
Done when: Fetches from replicas are recorded as exempt request time and never throttled by client quotas; tests.

### [To Do] QUOTA-0020 Set `throttle_time_ms` in Fetch responses for every version
P0 · M4 · Task · deps: QUOTA-0017
Done when: Fetch responses carry the throttle time in all supported versions; tests per version.

### [To Do] QUOTA-0021 Fetch quota compatibility test with the Java client and librdkafka
P0 · M4 · Task · deps: QUOTA-0018, QUOTA-0020, TEST-F08 · labels: test
Done when: Consumers with a 1 MiB/s quota converge to within 10% and report throttle metrics as with Kafka 4.4.0.

## [To Do] QUOTA-F04 Request-time quotas
P0 · M4 · Feature · deps: QUOTA-F01, QUOTA-F10
Scope: Request handler time percentage quotas.

### [To Do] QUOTA-0022 Spike: measuring request-handler time for `request_percentage` in an async runtime
P0 · M4 · Spike · deps: none · ref: KIP-124
Done when: ADR defines what time counts (handler poll time on the request path plus connection-task time for reading and writing the request) and the capacity basis ((`num.io.threads` + `num.network.threads`) x 100% as in Kafka) and validates it with a benchmark; accepted ADR.

### [To Do] QUOTA-0023 Record handler time as a percentage of one thread-second per entity
P0 · M4 · Task · deps: QUOTA-0022 · ref: server/src/main/java/org/apache/kafka/server/quota/ClientRequestQuotaManager.java maybeRecordAndGetThrottleTimeMs, nanosToPercentage @4.4.0
Done when: Handler time per request (local time including callbacks) is recorded as nanos x 100 / 1e9 on the entity's request sensor and a violation yields a throttle time; tests on the simulated clock.

### [To Do] QUOTA-0024 Include connection-task time for receiving and sending in request quota usage
P0 · M4 · Task · deps: QUOTA-0023, NET-F06 · ref: server/src/main/java/org/apache/kafka/network/Request.java updateRequestMetrics (recordNetworkThreadTimeCallback) @4.4.0
Done when: Time spent reading, decoding, encoding and writing a request on the connection task is recorded at response completion without throttling (it applies to later requests), including authentication time for a connection's first request; tests.

### [To Do] QUOTA-0025 Exempt cluster-action requests and record them on the exempt sensor
P0 · M4 · Task · deps: QUOTA-0023 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala handleError, server/src/main/java/org/apache/kafka/server/quota/ClientRequestQuotaManager.java maybeRecordExempt @4.4.0
Done when: Inter-broker (cluster action) APIs record `exempt-request-time` and are never throttled; their error responses are exempt except ClusterAuthorizationException; tests.

### [To Do] QUOTA-0026 Response helpers: send with or without throttle, error responses, no-op responses
P0 · M4 · Task · deps: QUOTA-0025, NET-F05 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala @4.4.0
Done when: Helpers equivalent to `sendMaybeThrottle`, `sendResponseMaybeThrottle`, `sendErrorResponseMaybeThrottle`, `sendResponseExemptThrottle` and `sendNoOpResponseExemptThrottle` set `throttle_time_ms`, start throttling only for non-forwarded requests, and are what handlers call; tests per helper.

### [To Do] QUOTA-0027 Forwarded requests: broker applies the max of its own and the controller's throttle
P0 · M4 · Task · deps: QUOTA-0026 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendForwardedResponse @4.4.0
Done when: For responses to forwarded requests the broker records its request quota, takes the maximum with the controller's throttle time, throttles the client channel and sets the field; the controller never mutes the broker's channel for forwarded requests; tests.

### [To Do] QUOTA-0028 Request quota tests: expensive requests get throttled per the formula
P0 · M4 · Task · deps: QUOTA-0024, QUOTA-0027 · labels: test
Done when: A client issuing many-partition Metadata requests under a 10% quota receives throttle times matching Kafka's formula within rounding on the simulated clock; integration test with a real client.

## [To Do] QUOTA-F05 Controller mutation quotas
P1 · M4 · Feature · deps: QUOTA-F01, QUOTA-F10
Scope: KIP-599 for topic and partition creation and deletion.

### [To Do] QUOTA-0029 Controller mutation quota manager with token-bucket sensors
P1 · M4 · Task · deps: none · ref: server/src/main/java/org/apache/kafka/server/quota/ControllerMutationQuotaManager.java throttleTimeMs, clients/src/main/java/org/apache/kafka/common/metrics/stats/TokenBucket.java @4.4.0
Done when: Per-entity token buckets use `controller.quota.window.num` and `controller.quota.window.size.seconds`; throttle time is the token deficit divided by the rate as in Kafka; tests.

### [To Do] QUOTA-0030 Strict quota: reject partition mutations with THROTTLING_QUOTA_EXCEEDED once exhausted
P1 · M4 · Task · deps: QUOTA-0029 · ref: server/src/main/java/org/apache/kafka/server/quota/StrictControllerMutationQuota.java record @4.4.0
Done when: Each topic's partition count is recorded as permits; when the bucket is exhausted further topics in the request fail with THROTTLING_QUOTA_EXCEEDED and the throttle time; the operation that crosses zero is accepted, as Kafka's strict quota; tests.

### [To Do] QUOTA-0031 Permissive quota for older versions and automatic topic creation
P1 · M4 · Task · deps: QUOTA-0029 · ref: server/src/main/java/org/apache/kafka/server/quota/PermissiveControllerMutationQuota.java, core/src/main/scala/kafka/server/KafkaApis.scala (newPermissiveQuotaFor) @4.4.0
Done when: Permissive quotas record mutations but never reject; they are used below the strict version and for topic auto-creation from Metadata and FindCoordinator; tests.

### [To Do] QUOTA-0032 Version thresholds for strict quotas: CreateTopics v6+, DeleteTopics v5+, CreatePartitions v3+
P1 · M4 · Task · deps: QUOTA-0030, QUOTA-0031 · ref: server/src/main/java/org/apache/kafka/server/quota/ControllerMutationQuotaManager.java newQuotaFor, core/src/main/scala/kafka/server/ControllerApis.scala @4.4.0
Done when: Quota selection by API version follows `newQuotaFor(header, strictSinceVersion)` with Kafka's thresholds; tests per API and version.

### [To Do] QUOTA-0033 Throttle responses by the larger of the mutation and request throttles
P1 · M4 · Task · deps: QUOTA-0032, QUOTA-0026 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottleWithControllerQuota @4.4.0
Done when: Responses set `throttle_time_ms` to the maximum of both and the channel is throttled by the larger manager (non-forwarded requests only); tests.

### [To Do] QUOTA-0034 End-to-end mutation quota test through broker forwarding
P1 · M4 · Task · deps: QUOTA-0033, QUOTA-0027, TEST-F08 · labels: test
Done when: Creating many partitions through a broker under a low `controller_mutation_rate` yields THROTTLING_QUOTA_EXCEEDED and throttle times; the Java admin client retries and eventually succeeds as against Kafka 4.4.0.

## [To Do] QUOTA-F06 Connection creation rate quotas
P1 · M4 · Feature · deps: QUOTA-F01, NET-F02
Scope: KIP-612 per-IP and per-listener rate limits.

### [To Do] QUOTA-0035 Broker-wide and per-listener connection creation rate limits
P1 · M4 · Task · deps: NET-F09 · ref: core/src/main/scala/kafka/network/SocketServer.scala ConnectionQuotas.recordConnectionAndGetThrottleTimeMs @4.4.0
Done when: Each accepted connection is recorded on the broker sensor (`max.connection.creation.rate`) and the listener sensor (`listener.name.<l>.max.connection.creation.rate`); the acceptor delays further accepts by the larger throttle, capped at the quota window; the protected inter-broker listener only applies its listener limit; tests on the simulated clock.

### [To Do] QUOTA-0036 Per-IP `connection_creation_rate` quotas with delayed close and un-recording
P1 · M4 · Task · deps: QUOTA-0035 · ref: core/src/main/scala/kafka/network/SocketServer.scala ConnectionQuotas.recordIpConnectionMaybeThrottle, updateListenerMetrics @4.4.0
Done when: A connection that violates its IP's rate is held and closed after the throttle time; its sample is un-recorded from the IP, listener and broker sensors so throttled attempts do not inflate those rates; IP throttle time is recorded per listener; tests.

### [To Do] QUOTA-0037 Apply IP quota updates (specific IP and default) to existing sensors
P1 · M4 · Task · deps: QUOTA-0036, QUOTA-0006 · ref: core/src/main/scala/kafka/network/SocketServer.scala ConnectionQuotas.updateIpConnectionRateQuota @4.4.0
Done when: Updates for one IP or for the default re-bound existing IP sensors; removing an override reverts to the default; tests.

### [To Do] QUOTA-0038 Expire idle per-IP sensors to bound memory
P1 · M4 · Task · deps: QUOTA-0036 · labels: nfr · ref: server/src/main/java/org/apache/kafka/network/ConnectionQuotaEntity.java @4.4.0
Done when: Per-IP sensors expire after inactivity like Kafka's `ConnectionQuotaEntity` expiration; a test with 100k source addresses keeps memory bounded.

### [To Do] QUOTA-0039 Connection rate metrics: accept rate and throttle time per listener and IP
P1 · M4 · Task · deps: QUOTA-0036, OBS-F04 · labels: nfr · ref: core/src/main/scala/kafka/network/SocketServer.scala ListenerConnectionQuota.createConnectionRateThrottleSensor @4.4.0
Done when: `alluvium_network_connection_accept_rate{listener}`, IP accept rates with bounded labels, and average non-zero `connection-accept-throttle-time` and `ip-connection-accept-throttle-time` per listener exist; tests.

### [To Do] QUOTA-0040 Connection rate tests: flood from one IP and from many IPs
P1 · M4 · Task · deps: QUOTA-0037, QUOTA-0038 · labels: test
Done when: Under a flood from one IP only that IP is throttled; under a flood from many IPs the listener limit holds; other clients keep connecting within limits.

## [To Do] QUOTA-F07 Throttling semantics
P0 · M4 · Feature · deps: QUOTA-F02
Scope: throttle_time_ms per API version, client-side throttling (KIP-219), channel muting.

### [To Do] QUOTA-0041 Throttled-channel delay queue with a reaper that ends throttling at expiry
P0 · M4 · Task · deps: none · ref: server/src/main/java/org/apache/kafka/server/quota/ThrottledChannel.java, server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java throttle, ThrottledChannelReaper @4.4.0
Done when: Throttling starts immediately and ends when the entry's time passes; one supervised reaper per quota type uses the injected clock; size is bounded by open connections; tests on the simulated clock.

### [To Do] QUOTA-0042 Drive connection muting from start and end of throttling
P0 · M4 · Task · deps: QUOTA-0041, NET-F05
Done when: Start/end throttling events feed the NET-F05 mute machine so the connection stays muted after the response until throttling ends; closing a throttled connection drops its entry harmlessly; tests.

### [To Do] QUOTA-0043 `throttle_time_ms` in every response that has the field, including error responses
P0 · M4 · Task · deps: QUOTA-0041 · ref: clients/src/main/java/org/apache/kafka/common/requests/AbstractResponse.java maybeSetThrottleTimeMs @4.4.0
Done when: All responses and error responses whose version has `throttle_time_ms` carry the computed value (`maybeSetThrottleTimeMs`); a generated test iterates every API version with the field.

### [To Do] QUOTA-0044 Throttle after the response for all versions (KIP-219)
P0 · M4 · Task · deps: QUOTA-0042 · ref: KIP-219
Done when: The server always sends the response immediately and then mutes the channel, for clients that throttle themselves and older versions that do not; a raw client on Produce v5 (no client-side throttling) is still held off for the throttle time; tests.

### [To Do] QUOTA-0045 Inter-node clients honor `throttle_time_ms` per KIP-219
P0 · M4 · Task · deps: QUOTA-0043, NET-F10 · ref: clients/src/main/java/org/apache/kafka/common/requests/AbstractResponse.java shouldClientThrottle @4.4.0
Done when: Outbound clients delay further requests to a node by the throttle time when the response version says clients throttle (`shouldClientThrottle` table per API and version); tests with a throttling stub peer.

### [To Do] QUOTA-0046 Throttling compatibility tests with the Java client and librdkafka
P0 · M4 · Task · deps: QUOTA-0044, TEST-F08 · labels: test
Done when: Under produce and fetch quotas both clients observe the same throttle behavior and client throttle metrics as with Kafka 4.4.0.

### [To Do] QUOTA-0047 Document throttling semantics
P1 · M4 · Task · deps: QUOTA-0044 · labels: docs
Done when: Docs explain throttle-after-response, which responses carry the field, muting and exemptions; doc build passes.

## [To Do] QUOTA-F08 Quota administration
P0 · M4 · Feature · deps: QUOTA-F01
Scope: Describe/AlterClientQuotas handling and validation.

### [To Do] QUOTA-0048 DescribeClientQuotas filter validation
P0 · M4 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/image/ClientQuotasImage.java describe @4.4.0
Done when: Empty entity types, duplicate types, unsupported types, EXACT without a match string, DEFAULT or SPECIFIED with one, and unknown match types fail with Kafka's errors and messages; tests per rule.

### [To Do] QUOTA-0049 Filter matching: exact, default and specified components, strict and non-strict
P0 · M4 · Task · deps: QUOTA-0048 · ref: metadata/src/main/java/org/apache/kafka/image/ClientQuotasImage.java matches @4.4.0
Done when: Entities match exact names, default entities or any specified name per component; strict filters require exactly the filtered components while non-strict ones allow extra components; tests ported from Kafka.

### [To Do] QUOTA-0050 DescribeClientQuotas results: entities with default markers and quota values
P0 · M4 · Task · deps: QUOTA-0049
Done when: Results list each matching entity (default as null name) with its key/value quotas from one image version; tests.

### [To Do] QUOTA-0051 Differential test of quota administration against Kafka 4.4.0
P0 · M4 · Task · deps: QUOTA-0050, TEST-F09 · labels: test
Done when: A corpus of alterations (including validate-only and IP entities) followed by describe filters gives identical results on both systems.

### [To Do] QUOTA-0052 Altered quotas take effect on every broker after metadata propagation
P0 · M4 · Task · deps: QUOTA-0050, QUOTA-0006 · labels: test
Done when: An alteration through any broker changes throttling on all brokers once they apply the image; the test measures the delay against the documented bound.

### [To Do] QUOTA-0053 Document quota administration
P2 · M4 · Task · deps: QUOTA-0050 · labels: docs
Done when: Docs cover `kafka-configs.sh` and admin client usage for each entity type and quota key with examples; doc build passes.

## [To Do] QUOTA-F09 Quota metrics
P1 · M4 · Feature · deps: QUOTA-F02, OBS-F04
Scope: Per-entity throttle metrics with bounded cardinality.

### [To Do] QUOTA-0054 Per-entity quota metrics: usage and throttle time with user and client_id labels
P1 · M4 · Task · deps: none · ref: server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java clientQuotaMetricName, throttleMetricName @4.4.0
Done when: Byte-rate and request-time usage and average and maximum throttle time per entity exist with sanitized `user` and `client_id` labels, mapping `kafka.server:type=Produce|Fetch|Request,user=,client-id=`; tests.

### [To Do] QUOTA-0055 Bounded label cardinality for quota metrics
P1 · M4 · Task · deps: QUOTA-0054 · labels: nfr
Done when: Series are removed when their sensors expire (1 h inactivity) and a cap on distinct entities folds the excess into `other` with a counter of folded series; test exceeds the cap.

### [To Do] QUOTA-0056 Exempt request time metric
P1 · M4 · Task · deps: QUOTA-0054 · ref: server/src/main/java/org/apache/kafka/server/quota/ClientRequestQuotaManager.java exemptSensor @4.4.0
Done when: `exempt-request-time` is exported as `alluvium_quota_exempt_request_time_ratio`; test.

### [To Do] QUOTA-0057 Throttled-channel queue size per quota type
P1 · M4 · Task · deps: QUOTA-0054 · ref: server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java delayQueueSensor @4.4.0
Done when: `alluvium_quota_throttled_channels{type}` mirrors Kafka's per-type `queue-size`; test.

### [To Do] QUOTA-0058 Controller mutation rate and throttle metrics
P1 · M4 · Task · deps: QUOTA-0054, QUOTA-F05
Done when: Mutation rate and throttle time per entity map `kafka.server:type=ControllerMutation`; test.

## [To Do] QUOTA-F10 Rate measurement
P0 · M4 · Feature · deps: TYPE-F06
Scope: Sampled windowed rates (`quota.window.num`, `quota.window.size.seconds`).

### [To Do] QUOTA-0059 Sampled statistics with `quota.window.num` samples of `quota.window.size.seconds`
P0 · M4 · Task · deps: CFG-F07 · ref: clients/src/main/java/org/apache/kafka/common/metrics/stats/SampledStat.java, server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java @4.4.0
Done when: Samples roll on time, obsolete samples (older than the full window) are purged, and the defaults are 11 samples of 1 s; unit tests ported from SampledStat tests on the simulated clock.

### [To Do] QUOTA-0060 Rate measurement with Kafka's window-size rule
P0 · M4 · Task · deps: QUOTA-0059 · ref: clients/src/main/java/org/apache/kafka/common/metrics/stats/Rate.java windowSize @4.4.0
Done when: Rate is total over elapsed time since the oldest sample, padded to at least N-1 full windows and at least 1 ms; golden tests match values from Kafka's `Rate` for fixed sequences.

### [To Do] QUOTA-0061 Quota bound check on record with value and bound reported
P0 · M4 · Task · deps: QUOTA-0060 · ref: clients/src/main/java/org/apache/kafka/common/metrics/Sensor.java checkQuotas, clients/src/main/java/org/apache/kafka/common/metrics/QuotaViolationException.java @4.4.0
Done when: Recording a value updates the stat and reports a violation when the measured rate exceeds the upper bound; the recorded value is kept, as Kafka does; tests.

### [To Do] QUOTA-0062 Throttle time from a violation, with a bounded variant
P0 · M4 · Task · deps: QUOTA-0061 · ref: server-common/src/main/java/org/apache/kafka/server/quota/QuotaUtils.java throttleTime, boundedThrottleTime @4.4.0
Done when: Throttle = (value - bound) / bound x window size, rounded, and the bounded variant caps it (used by connection quotas); golden tests against QuotaUtils.

### [To Do] QUOTA-0063 Token bucket stat for controller mutations (KIP-599)
P0 · M4 · Task · deps: QUOTA-0059 · ref: clients/src/main/java/org/apache/kafka/common/metrics/stats/TokenBucket.java @4.4.0
Done when: Tokens accrue at the quota rate up to quota x samples x window, recording consumes tokens, and the deficit gives the throttle time; golden tests against Kafka's TokenBucket.

### [To Do] QUOTA-0064 Sensor registry with inactivity expiry and bounded size
P0 · M4 · Task · deps: QUOTA-0061 · labels: nfr · ref: server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java getOrCreateSensor, INACTIVE_SENSOR_EXPIRATION_TIME_SECONDS @4.4.0
Done when: Quota and throttle-time sensors are created per entity on demand and expire after inactivity (3600 s for client quotas); a cap bounds the registry; tests on the simulated clock.

### [To Do] QUOTA-0065 Record without quota checks and un-record
P0 · M4 · Task · deps: QUOTA-0061 · ref: server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java recordNoThrottle, unrecordQuotaSensor @4.4.0
Done when: `record_no_throttle` and negative records (un-record) behave like Kafka's `recordNoThrottle` and `unrecordQuotaSensor`; tests.

### [To Do] QUOTA-0066 Benchmark the record path and memory per sensor
P2 · M4 · Task · deps: QUOTA-0064 · labels: perf
Done when: Criterion bench of recording on a hot sensor and of creating sensors; results recorded with a regression threshold.
