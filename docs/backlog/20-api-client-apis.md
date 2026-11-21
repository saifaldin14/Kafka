# API: Kafka API handlers

> Crates: `alluvium-server/src/handlers/` · Kafka counterpart: `KafkaApis.scala`, `ControllerApis.scala` · Milestones: M2-M9

One feature per Kafka 4.4.0 API key; the feature number is the API key (`API-F00` is Produce). Each covers routing, authorization, version-specific behavior, error mapping, metrics, tests, compatibility and differential tests, and finally advertising in ApiVersions. Domain logic lives in the owning epic named in each feature's dependencies. Keys 4-7 were removed in Kafka 4.0 and are not planned.

## [To Do] API-F00 Produce (v3-13)
P0 · M2 · Feature · deps: REPL-F03, REPL-F08, REC-F12, SRV-F10
Scope: Kafka 4.4.0 versions 3-13; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0001 Produce handler module and router registration (broker listener)
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest @4.4.0
Done when: `handlers/produce.rs` (module doc names the Kafka counterpart) decodes v3-13 with the generated codec and calls the produce service (SRV-F10); it is registered for the broker listener in the router (SRV-F04). A Produce request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0002 Produce request checks: acks, transactional batches and internal topics
P0 · M2 · Task · deps: API-0001 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest @4.4.0
Done when: acks outside {-1, 0, 1} fails every partition with INVALID_REQUIRED_ACKS; a request carrying transactional batches with a null transactional_id fails as a whole with TRANSACTIONAL_ID_AUTHORIZATION_FAILED even without an authorizer; writes to `__consumer_offsets`, `__transaction_state` or `__share_group_state` get INVALID_TOPIC_EXCEPTION unless the client id is `__admin_client`; unit test per rule.

### [To Do] API-0003 Produce per-partition record validation (`ProduceRequest.validateRecords`)
P0 · M2 · Task · deps: API-0001 · labels: security, decoder · ref: clients/src/main/java/org/apache/kafka/common/requests/ProduceRequest.java validateRecords @4.4.0
Done when: Each partition must carry exactly one magic-v2 record batch: a null or empty records field, a v0/v1 batch or a second batch fails only that partition with INVALID_RECORD and an error message naming the version; deeper checks are delegated to REC-F12; a malformed partition never fails its siblings; fuzz-style test feeds truncated and multi-batch payloads.

### [To Do] API-0004 Produce acks=0: no response, close the connection on any partition error
P0 · M2 · Task · deps: API-0001 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest sendResponseCallback @4.4.0
Done when: With acks=0 the handler sends no response on success; if any partition fails it logs the per-partition error summary once at info and closes the connection so the producer refreshes metadata, as Kafka does; request-quota accounting is skipped for acks=0; tests cover success, one failed partition and the error-count metric.

### [To Do] API-0005 Produce v3-v6: log_append_time_ms and log_start_offset in partition responses, ZSTD rejected
P0 · M2 · Task · deps: API-0003 · ref: clients/src/main/resources/common/message/ProduceResponse.json @4.4.0
Done when: Partition responses carry log_append_time_ms (the broker time when the topic uses LogAppendTime, else -1) and, from v5, log_start_offset; a ZSTD batch in v3-v6 fails its partition with UNSUPPORTED_COMPRESSION_TYPE; v4 adds no wire change (clients must accept KAFKA_STORAGE_ERROR); per-version tests.

### [To Do] API-0006 Produce v7: accept ZSTD-compressed batches (KIP-110)
P0 · M2 · Task · deps: API-0005 · ref: clients/src/main/resources/common/message/ProduceRequest.json @4.4.0 (KIP-110)
Done when: From v7 a ZSTD batch is appended like any other codec; the same batch at v6 is still rejected; test at both versions with REC-F08 output.

### [To Do] API-0007 Produce v8: record_errors and error_message for dropped batches (KIP-467)
P0 · M2 · Task · deps: API-0001 · ref: clients/src/main/resources/common/message/ProduceResponse.json @4.4.0 (KIP-467)
Done when: When REC-F12 rejects a batch, v8+ responses list each offending record's batch_index with its message plus a partition error_message; v3-v7 return only the error code; message text matches Kafka for compacted-topic null keys and out-of-range timestamps; tests per version.

### [To Do] API-0008 Produce v9-v10: flexible encoding and KIP-951 leader hints
P0 · M2 · Task · deps: API-0001 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest getCurrentLeader @4.4.0 (KIP-951)
Done when: v9+ requests with unknown tagged fields are accepted; for NOT_LEADER_OR_FOLLOWER partitions v10+ responses set the current_leader tagged field (leader id and epoch) and add the leader to node_endpoints using the request's listener, from META-F09; below v10 neither is written; tests with a stale leader.

### [To Do] API-0009 Produce v13: address topics by topic id (KIP-516)
P0 · M2 · Task · deps: API-0001 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest @4.4.0 (KIP-516)
Done when: v3-v12 resolve names to ids through the metadata image and v13 resolves ids to names; an unknown topic id in v13 fails its partitions with UNKNOWN_TOPIC_ID while an unknown name or partition yields UNKNOWN_TOPIC_OR_PARTITION; responses echo the request's addressing (name below v13, id from v13); tests per version.

### [To Do] API-0010 Produce error mapping per version
P0 · M2 · Task · deps: API-0001 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest @4.4.0
Done when: Produce has no top-level error: every failure is a partition error_code from one exhaustive mapping (ERR-F02) covering NOT_LEADER_OR_FOLLOWER, NOT_ENOUGH_REPLICAS, NOT_ENOUGH_REPLICAS_AFTER_APPEND, MESSAGE_TOO_LARGE, RECORD_LIST_TOO_LARGE, CORRUPT_MESSAGE, INVALID_TIMESTAMP, INVALID_RECORD, KAFKA_STORAGE_ERROR (returned as-is to v3 because 4.4.0 no longer downgrades it) and REQUEST_TIMED_OUT from delayed produce; mapping table test.

### [To Do] API-0011 Produce throttle_time_ms through the shared response path
P0 · M2 · Task · deps: API-0001 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0012 Produce throttling under the produce byte-rate and request quotas (KIP-219)
P0 · M4 · Task · deps: API-0011, QUOTA-F02, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: Produce records usage against the produce byte-rate and request quotas and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v6+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Bandwidth and request quotas are both recorded, the larger throttle wins, and acks=0 requests skip the request quota. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0013 Produce request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0001 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for Produce; the request-log summary (SRV-F09) shows acks, timeout_ms, transactional_id presence, topic and partition counts and total batch bytes. Closing a connection after a failed acks=0 request emits an event with reason `acks0_partition_error`. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0014 Produce handler unit tests across v3-13
P0 · M2 · Task · deps: API-0002, API-0004, API-0006, API-0007, API-0008, API-0009, API-0010, API-0011 · labels: test
Done when: Table-driven tests send every version v3-v13 through the handler, with the produce service (SRV-F10) replaced by a fake, and check the service call and the encoded response; cases cover acks 0/1/all, transactional and non-transactional batches, unknown names and ids, internal topics, INVALID_RECORD with record_errors, delayed-produce timeout and v10 leader hints.

### [To Do] API-0015 Produce handler allocation and latency benchmark
P1 · M2 · Task · deps: API-0014, TEST-F11 · labels: perf, nfr
Done when: A TEST-F11 benchmark drives 1 KiB and 1 MiB batches through decode, validation and the service call; record bytes are not copied between the codec and the service (zero-copy records field, PROTO-F08); results recorded in the NFR-F01 budget.

### [To Do] API-0016 Produce compatibility test with Java client 4.4.0, librdkafka, franz-go and kafka-console-producer.sh
P0 · M2 · Task · deps: API-0014, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that produces with acks 0, 1 and all, every compression codec, headers and a stale-leader retry (Java producer with enable.idempotence=false until M6), then reads the records back with kafka-console-consumer.sh, passes with Java client 4.4.0, librdkafka, franz-go and kafka-console-producer.sh.

### [To Do] API-0017 Produce differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0014, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical Produce requests to Kafka 4.4.0 and Alluvium for acks 1 and all appends and for each client-triggerable error (UNKNOWN_TOPIC_OR_PARTITION, UNKNOWN_TOPIC_ID at v13, INVALID_REQUIRED_ACKS, INVALID_RECORD with record_errors, UNSUPPORTED_COMPRESSION_TYPE for ZSTD below v7, MESSAGE_TOO_LARGE, NOT_ENOUGH_REPLICAS, INVALID_TOPIC_EXCEPTION on an internal topic and TRANSACTIONAL_ID_AUTHORIZATION_FAILED for transactional batches without an id); normalized responses match for every supported version.

### [To Do] API-0018 Advertise Produce v3-13 in ApiVersions
P0 · M2 · Task · deps: API-0013, API-0015, API-0016, API-0017, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists Produce with min 0 and max 13 through the generated list owned by API-F18; docs/parity.md marks Produce supported; a test asserts the range. Produce is advertised from v0 although v0-v2 are rejected, as Kafka does for librdkafka (KAFKA-18659, `ApiKeys.PRODUCE_API_VERSIONS_RESPONSE_MIN_VERSION`). Later tasks (M4, M6) add behavior without changing the advertised range.

### [To Do] API-0019 Produce authorization: WRITE on TRANSACTIONAL_ID and per-topic WRITE
P0 · M4 · Task · deps: API-0001, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest @4.4.0
Done when: Transactional requests need WRITE on the TRANSACTIONAL_ID or the whole request fails with TRANSACTIONAL_ID_AUTHORIZATION_FAILED; topics without WRITE on TOPIC fail per partition with TOPIC_AUTHORIZATION_FAILED while authorized partitions are appended; authorization is evaluated once per topic (SEC-F08); tests and a differential case for each denial.

### [To Do] API-0020 Produce v11: TRANSACTION_ABORTABLE from transaction verification (KIP-890)
P0 · M6 · Task · deps: API-0001, TXN-F06 · ref: server/src/main/java/org/apache/kafka/server/transaction/AddPartitionsToTxnManager.java produceRequestVersionToTransactionSupportedOperation @4.4.0
Done when: Transactional appends are verified through TXN-F06; a TRANSACTION_ABORTABLE verification result is returned to v11+ and mapped to INVALID_TXN_STATE for v3-v10, and PRODUCER_FENCED becomes INVALID_PRODUCER_EPOCH, as `AddPartitionsToTxnManager` does; tests per version.

### [To Do] API-0021 Produce v12: implicit AddPartitionsToTxn for transactions v2 (KIP-890 part 2)
P0 · M6 · Task · deps: API-0020, TXN-F10 · ref: clients/src/main/resources/common/message/ProduceRequest.json @4.4.0 (KIP-890)
Done when: v12+ transactional produce uses the ADD_PARTITION operation so the partition joins the transaction with epoch-bump support (TXN-F10); v11 and below only verify; test shows a v12 transactional produce succeeds without a prior AddPartitionsToTxn.

### [To Do] API-0022 Produce idempotence and sequence errors per version
P0 · M6 · Task · deps: API-0010, TXN-F01 · ref: clients/src/main/resources/common/message/ProduceResponse.json @4.4.0
Done when: Errors from producer-state validation (STOR-F15, TXN-F01) map through the produce mapping: OUT_OF_ORDER_SEQUENCE_NUMBER, DUPLICATE_SEQUENCE_NUMBER (returned as success with the original offset), INVALID_PRODUCER_EPOCH, UNKNOWN_PRODUCER_ID and INVALID_PRODUCER_ID_MAPPING; log_start_offset (v5+) is set so clients can ignore spurious sequence errors; unit tests.

### [To Do] API-0023 Produce compatibility test with idempotent and transactional producers
P0 · M6 · Task · deps: API-0021, API-0022, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that runs the default idempotent producer and a transactional producer (transactions v1 and v2) through broker restarts with no duplicates passes with Java client 4.4.0 and librdkafka idempotent and transactional producers.

### [To Do] API-0024 Produce differential test for idempotent and transactional errors
P0 · M6 · Task · deps: API-0021, API-0022, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical Produce requests to Kafka 4.4.0 and Alluvium for idempotent and transactional appends and for each client-triggerable error (OUT_OF_ORDER_SEQUENCE_NUMBER, INVALID_PRODUCER_EPOCH, INVALID_TXN_STATE at v10 versus TRANSACTION_ABORTABLE at v11, and TRANSACTIONAL_ID_AUTHORIZATION_FAILED); normalized responses match for every supported version.

## [To Do] API-F01 Fetch (v4-18)
P0 · M2 · Feature · deps: REPL-F13, REPL-F09, REPL-F10, SRV-F10
Scope: Kafka 4.4.0 versions 4-18; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0025 Fetch handler module and broker-listener registration
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest @4.4.0
Done when: `handlers/fetch.rs` (module doc names `KafkaApis.handleFetchRequest`) decodes v4-v18 and calls the fetch service; it is registered for the broker listener now and for the controller listener by the KRaft fetch task; replica_id comes from replica_id (v4-v14) or replica_state (v15+) and distinguishes consumers (-1) from followers (>= 0); unit test per listener.

### [To Do] API-0026 Fetch byte limits: max_bytes, min_bytes and partition_max_bytes
P0 · M2 · Task · deps: API-0025 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest @4.4.0
Done when: Response size is capped at min(max_bytes, `fetch.max.bytes`, the client's quota-window bytes) and min_bytes at that cap; partition_max_bytes is honored per partition with the first-batch exception from REPL-F13; tests for each clamp and for followers ignoring the quota cap.

### [To Do] API-0027 Fetch topic resolution: names in v4-v12, topic ids from v13 (KIP-516)
P0 · M2 · Task · deps: API-0025 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest @4.4.0 (KIP-516)
Done when: v13+ topic ids (including forgotten_topics_data) are resolved through the metadata image; an unknown id yields UNKNOWN_TOPIC_ID for its partitions and an unknown name or partition UNKNOWN_TOPIC_OR_PARTITION, without failing the rest of the request; responses echo names below v13 and ids from v13; tests per version.

### [To Do] API-0028 Fetch v7: incremental fetch sessions in the handler (KIP-227)
P0 · M2 · Task · deps: API-0027 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest fetchManager.newContext @4.4.0 (KIP-227)
Done when: session_id and session_epoch select a sessionless, full or incremental context from REPL-F10; FETCH_SESSION_ID_NOT_FOUND, INVALID_FETCH_SESSION_EPOCH and FETCH_SESSION_TOPIC_ID_ERROR are returned as the top-level error_code with an empty response; the response carries the session id; v4-v6 are always sessionless; tests for create, update, forget and close.

### [To Do] API-0029 Fetch v4-v6: isolation_level, last_stable_offset and log_start_offset
P0 · M2 · Task · deps: API-0025 · ref: clients/src/main/resources/common/message/FetchResponse.json @4.4.0
Done when: isolation_level (v4+) selects the high watermark or the last stable offset as the read bound through REPL-F13 and last_stable_offset is returned; log_start_offset (v5+) is returned per partition; v6+ may return KAFKA_STORAGE_ERROR; ZSTD batches are served unchanged to v4-v9 because 4.4.0 never down-converts; per-version tests.

### [To Do] API-0030 Fetch v9: current_leader_epoch fencing (KIP-320)
P0 · M2 · Task · deps: API-0025 · ref: clients/src/main/resources/common/message/FetchRequest.json @4.4.0 (KIP-320)
Done when: A partition's current_leader_epoch older than the leader's returns FENCED_LEADER_EPOCH and a newer one UNKNOWN_LEADER_EPOCH, -1 skips the check; other partitions are unaffected; tests for each case.

### [To Do] API-0031 Fetch v12: last_fetched_epoch, diverging_epoch and flexible encoding
P0 · M2 · Task · deps: API-0025 · ref: clients/src/main/resources/common/message/FetchRequest.json @4.4.0 (KIP-595)
Done when: From v12 the handler passes last_fetched_epoch to REPL-F13 and returns diverging_epoch when the fetcher's log diverges; tagged fields are accepted; below v12 neither field exists; tests with a divergent epoch.

### [To Do] API-0032 Fetch v16: current_leader and node_endpoints on leader errors (KIP-951)
P0 · M2 · Task · deps: API-0025 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest @4.4.0 (KIP-951)
Done when: For NOT_LEADER_OR_FOLLOWER or FENCED_LEADER_EPOCH partitions, v16+ responses fill current_leader (id and epoch) and add the leader's endpoint for the request's listener to node_endpoints, also in throttled responses; below v16 they stay unset; tests with a moved leader.

### [To Do] API-0033 Fetch error mapping per version
P0 · M2 · Task · deps: API-0025 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest maybeDownConvertStorageError @4.4.0
Done when: Partition errors map through one exhaustive table (ERR-F02); KAFKA_STORAGE_ERROR is down-converted to NOT_LEADER_OR_FOLLOWER for v4-v5 (`maybeDownConvertStorageError`); OFFSET_OUT_OF_RANGE, UNKNOWN_TOPIC_OR_PARTITION, UNKNOWN_TOPIC_ID and the session errors land at the right level; mapping test per version.

### [To Do] API-0034 Fetch throttle_time_ms through the shared response path
P0 · M2 · Task · deps: API-0025 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0035 Fetch throttling under the fetch byte-rate and request quotas (KIP-219)
P0 · M4 · Task · deps: API-0034, QUOTA-F03, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: Fetch records usage against the fetch byte-rate and request quotas and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v8+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. The recorded bandwidth is un-recorded when the response is throttled, a throttled response carries no records, and follower fetches are exempt. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0036 Fetch request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0025 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for Fetch; the request-log summary (SRV-F09) shows replica id, isolation level, session id and epoch, partition count, min/max bytes and max wait. BytesOut per topic is recorded on the response path, split by follower and reassignment traffic. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0037 Fetch handler unit tests across v4-18
P0 · M2 · Task · deps: API-0026, API-0028, API-0029, API-0030, API-0031, API-0032, API-0033, API-0034 · labels: test
Done when: Table-driven tests send every version v4-v18 through the handler, with the fetch service (SRV-F10) replaced by a fake, and check the service call and the encoded response; cases cover sessionless and session fetches, long-poll completion through REPL-F09, unknown ids, fencing, diverging epoch, storage-error down-conversion and v16 leader hints.

### [To Do] API-0038 Fetch response path benchmark: zero-copy record bytes
P1 · M2 · Task · deps: API-0037, TEST-F11 · labels: perf, nfr
Done when: A TEST-F11 benchmark serves 100 partitions of 1 MiB through the handler; record bytes go from the segment to the socket without an intermediate copy (NET-F06, PROTO-F08) and per-request allocations stay within the NFR-F01 budget.

### [To Do] API-0039 Fetch compatibility test with Java client 4.4.0, librdkafka, franz-go and kafka-console-consumer.sh
P0 · M2 · Task · deps: API-0037, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that consumes from many partitions with long polling, max.partition.fetch.bytes limits, incremental sessions and a leader change mid-stream, passes with Java client 4.4.0, librdkafka, franz-go and kafka-console-consumer.sh.

### [To Do] API-0040 Fetch differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0037, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical Fetch requests to Kafka 4.4.0 and Alluvium for sessionless and incremental fetches at both isolation levels and for each client-triggerable error (OFFSET_OUT_OF_RANGE, UNKNOWN_TOPIC_OR_PARTITION, UNKNOWN_TOPIC_ID, FENCED_LEADER_EPOCH, UNKNOWN_LEADER_EPOCH, FETCH_SESSION_ID_NOT_FOUND and INVALID_FETCH_SESSION_EPOCH); normalized responses match for every supported version.

### [To Do] API-0041 Advertise Fetch v4-18 in ApiVersions
P0 · M2 · Task · deps: API-0036, API-0038, API-0039, API-0040, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists Fetch with min 4 and max 18 through the generated list owned by API-F18; docs/parity.md marks Fetch supported; a test asserts the range. Later tasks (M3, M4, M6, M9) add behavior without changing the advertised range.

### [To Do] API-0042 Fetch from followers: CLUSTER_ACTION path and replica_state (v15, KIP-903)
P0 · M3 · Task · deps: API-0025, REPL-F07 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest isFromFollower @4.4.0 (KIP-903)
Done when: Requests with replica_id >= 0 (v4-v14) or replica_state.replica_id >= 0 (v15+, with replica_epoch) are follower fetches: partitions skip per-topic READ checks, the leader records follower state through REPL-F07, responses are exempt from quotas and recorded against the leader replication quota; tests per version.

### [To Do] API-0043 Fetch v11: rack_id and preferred_read_replica (KIP-392)
P0 · M3 · Task · deps: API-0025, REPL-F11 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest clientMetadata @4.4.0 (KIP-392)
Done when: From v11 the handler passes rack_id, client id, address and principal as client metadata so REPL-F11 can pick a preferred read replica, returned in preferred_read_replica (-1 otherwise); v4-v10 never get one; tests with a rack-aware selector.

### [To Do] API-0044 Fetch on the controller listener: KRaft replication (v17 directory id, v18 high watermark)
P0 · M3 · Task · deps: API-0025, RAFT-F04 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleFetch @4.4.0 (KIP-853, KIP-1166)
Done when: The controller-listener Fetch path hands requests for `__cluster_metadata` to the Raft driver (RAFT-F04): it requires CLUSTER_ACTION, validates the tagged cluster_id, passes replica_directory_id (v17, KIP-853) and the fetcher's high_watermark (v18, KIP-1166) so the leader can answer early when its high watermark is ahead; tests per version.

### [To Do] API-0045 Fetch multi-node simulation test
P0 · M3 · Task · deps: API-0042, API-0044, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) runs a 3-broker cluster with followers fetching through the handler and 3 controllers replicating metadata over controller-listener Fetch, with leader changes and dropped messages, and checks no acknowledged record is lost; a failing seed is printed and replays exactly.

### [To Do] API-0046 Fetch authorization: READ on topics, CLUSTER_ACTION for followers
P0 · M4 · Task · deps: API-0042, API-0044, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest @4.4.0
Done when: Consumer fetches need READ on each TOPIC (unauthorized partitions get TOPIC_AUTHORIZATION_FAILED, others are served); follower fetches need CLUSTER_ACTION on CLUSTER or every partition fails with TOPIC_AUTHORIZATION_FAILED; controller-listener fetches without CLUSTER_ACTION get CLUSTER_AUTHORIZATION_FAILED; tests and differential cases.

### [To Do] API-0047 Fetch read_committed: aborted_transactions and last stable offset
P0 · M6 · Task · deps: API-0029, TXN-F11 · ref: clients/src/main/resources/common/message/FetchResponse.json @4.4.0
Done when: With isolation_level READ_COMMITTED the handler returns the aborted_transactions list and last_stable_offset from TXN-F11 (null list for READ_UNCOMMITTED); tests with committed, aborted and open transactions.

### [To Do] API-0048 Fetch v14: OFFSET_MOVED_TO_TIERED_STORAGE (KIP-405)
P0 · M9 · Task · deps: API-0025, TIER-F04 · ref: clients/src/main/resources/common/message/FetchRequest.json @4.4.0 (KIP-405)
Done when: When a follower fetch targets an offset only in remote storage, v14+ returns OFFSET_MOVED_TO_TIERED_STORAGE via TIER-F07, and consumer reads below the local log start are served from TIER-F04; tests per version.

## [To Do] API-F02 ListOffsets (v1-11)
P0 · M2 · Feature · deps: REPL-F14
Scope: Kafka 4.4.0 versions 1-11; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0049 ListOffsets handler module and router registration (broker listener)
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListOffsetRequest @4.4.0
Done when: `handlers/list_offsets.rs` (module doc names the Kafka counterpart) decodes v1-11 with the generated codec and calls the offsets service (SRV-F10); it is registered for the broker listener in the router (SRV-F04). A ListOffsets request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0050 ListOffsets validation: duplicate partitions and unsupported special timestamps
P0 · M2 · Task · deps: API-0049 · labels: security · ref: core/src/main/scala/kafka/server/ReplicaManager.scala fetchOffset isListOffsetsTimestampUnsupported @4.4.0
Done when: A partition listed twice gets INVALID_REQUEST for every occurrence; a negative timestamp that is not a special value, or one newer than the request version allows (-3 needs v7, -4 v8, -5 v9, -6 v11), gets UNSUPPORTED_VERSION for that partition only; replica_id -1 marks consumers and -2 debug readers that may read from any replica; unit tests.

### [To Do] API-0051 ListOffsets v1-v3: single offset with timestamp, isolation_level from v2
P0 · M2 · Task · deps: API-0049 · ref: clients/src/main/resources/common/message/ListOffsetsRequest.json @4.4.0
Done when: Each partition returns one offset with its timestamp (-1 for -1/-2 lookups); isolation_level (v2+) bounds latest lookups at the high watermark or the last stable offset for consumers only; per-version tests.

### [To Do] API-0052 ListOffsets v4-v6: leader-epoch fencing, OFFSET_NOT_AVAILABLE and flexible encoding
P0 · M2 · Task · deps: API-0049 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala fetchOffset @4.4.0
Done when: current_leader_epoch (v4+) yields FENCED_LEADER_EPOCH or UNKNOWN_LEADER_EPOCH; leader_epoch is returned from v4; when the new leader's high watermark is not yet known, v5+ gets OFFSET_NOT_AVAILABLE and v1-v4 LEADER_NOT_AVAILABLE; v6 accepts tagged fields; tests per version.

### [To Do] API-0053 ListOffsets v7: max-timestamp lookup (KIP-734)
P0 · M2 · Task · deps: API-0050 · ref: clients/src/main/resources/common/message/ListOffsetsRequest.json @4.4.0 (KIP-734)
Done when: Timestamp -3 returns the offset and timestamp of the record with the largest timestamp through REPL-F14; at v6 the same request gets UNSUPPORTED_VERSION; tests at both versions.

### [To Do] API-0054 ListOffsets v8-v9: earliest-local (-4) and latest-tiered (-5) lookups
P0 · M2 · Task · deps: API-0050 · ref: clients/src/main/resources/common/message/ListOffsetsRequest.json @4.4.0 (KIP-405, KIP-1005)
Done when: Without tiered storage, -4 returns the local log start offset and -5 returns -1 with the epoch rules of REPL-F14; both are rejected with UNSUPPORTED_VERSION below v8 and v9 respectively; tests per version.

### [To Do] API-0055 ListOffsets error mapping per version
P0 · M2 · Task · deps: API-0049 · ref: core/src/main/scala/kafka/server/ReplicaManager.scala fetchOffset @4.4.0
Done when: Partition errors UNKNOWN_TOPIC_OR_PARTITION, NOT_LEADER_OR_FOLLOWER, UNKNOWN_LEADER_EPOCH, FENCED_LEADER_EPOCH, KAFKA_STORAGE_ERROR and UNSUPPORTED_FOR_MESSAGE_FORMAT come from one exhaustive mapping with timestamp and offset -1; the OFFSET_NOT_AVAILABLE downgrade is applied below v5; table test.

### [To Do] API-0056 ListOffsets throttle_time_ms through the shared response path
P0 · M2 · Task · deps: API-0049 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is written in v2+ and absent in v1; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0057 ListOffsets throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0056, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: ListOffsets records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v3+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0058 ListOffsets request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0049 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ListOffsets; the request-log summary (SRV-F09) shows replica id, isolation level, partition count and the special timestamps requested. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0059 ListOffsets handler unit tests across v1-11
P0 · M2 · Task · deps: API-0051, API-0052, API-0053, API-0054, API-0055, API-0056 · labels: test
Done when: Table-driven tests send every version v1-v11 through the handler, with the offsets service (SRV-F10) replaced by a fake, and check the service call and the encoded response; cases cover earliest, latest and timestamp lookups, duplicates, fencing and every special timestamp below its minimum version.

### [To Do] API-0060 ListOffsets compatibility test with Java client 4.4.0 (consumer seekToEnd and Admin.listOffsets), librdkafka, franz-go and kafka-get-offsets.sh
P0 · M2 · Task · deps: API-0059, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that lists earliest, latest, max-timestamp and by-time offsets for a multi-partition topic passes with Java client 4.4.0 (consumer seekToEnd and Admin.listOffsets), librdkafka, franz-go and kafka-get-offsets.sh.

### [To Do] API-0061 ListOffsets differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0059, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ListOffsets requests to Kafka 4.4.0 and Alluvium for every special timestamp and a by-time lookup and for each client-triggerable error (UNKNOWN_TOPIC_OR_PARTITION, duplicate-partition INVALID_REQUEST, UNSUPPORTED_VERSION for a too-new special timestamp, FENCED_LEADER_EPOCH and OFFSET_NOT_AVAILABLE versus LEADER_NOT_AVAILABLE); normalized responses match for every supported version.

### [To Do] API-0062 Advertise ListOffsets v1-11 in ApiVersions
P0 · M2 · Task · deps: API-0058, API-0060, API-0061, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists ListOffsets with min 1 and max 11 through the generated list owned by API-F18; docs/parity.md marks ListOffsets supported; a test asserts the range. Later tasks (M4, M6, M9) add behavior without changing the advertised range.

### [To Do] API-0063 ListOffsets authorization: DESCRIBE on each topic
P0 · M4 · Task · deps: API-0049, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListOffsetRequest @4.4.0
Done when: Topics without DESCRIBE on TOPIC get TOPIC_AUTHORIZATION_FAILED for all their partitions with offset and timestamp -1 while authorized topics are answered; tests and a differential case.

### [To Do] API-0064 ListOffsets read_committed latest lookups bounded by the last stable offset
P0 · M6 · Task · deps: API-0051, TXN-F11 · ref: clients/src/main/resources/common/message/ListOffsetsRequest.json @4.4.0
Done when: For READ_COMMITTED consumers, -1 returns the last stable offset from TXN-F11 while transactions are open; test with an open and a committed transaction.

### [To Do] API-0065 ListOffsets v10: asynchronous remote lookups with timeout_ms (KIP-1075)
P0 · M9 · Task · deps: API-0054, TIER-F04 · ref: clients/src/main/resources/common/message/ListOffsetsRequest.json @4.4.0 (KIP-1075)
Done when: Lookups that need remote storage complete asynchronously through a delayed operation bounded by the request's timeout_ms (TIER-F04, SRV-F07), returning REQUEST_TIMED_OUT on expiry; below v10 the lookup is synchronous; tests with a slow remote store.

### [To Do] API-0066 ListOffsets v11: earliest pending upload offset (-6, KIP-1023)
P0 · M9 · Task · deps: API-0065, TIER-F03 · ref: clients/src/main/resources/common/message/ListOffsetsRequest.json @4.4.0 (KIP-1023)
Done when: Timestamp -6 returns the earliest offset not yet uploaded to remote storage from TIER-F03; below v11 it gets UNSUPPORTED_VERSION; tests at v10 and v11.

### [To Do] API-0067 ListOffsets -4 and -5 with tiered storage enabled
P0 · M9 · Task · deps: API-0054, TIER-F02 · ref: clients/src/main/resources/common/message/ListOffsetsResponse.json @4.4.0 (KIP-405, KIP-1005)
Done when: With remote storage, -4 returns the local log start offset and -5 the highest tiered offset from TIER-F02; differential test against Kafka 4.4.0 with tiering enabled.

## [To Do] API-F03 Metadata (v0-13)
P0 · M2 · Feature · deps: META-F09, CTRL-F04
Scope: Kafka 4.4.0 versions 0-13; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0068 Metadata handler module and router registration (broker listener)
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTopicMetadataRequest @4.4.0
Done when: `handlers/metadata.rs` (module doc names the Kafka counterpart) decodes v0-13 with the generated codec and calls the broker metadata lookups (META-F09); it is registered for the broker listener in the router (SRV-F04). A Metadata request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0069 Metadata v0-v1: all-topics semantics, rack, controller id and is_internal
P0 · M2 · Task · deps: API-0068 · ref: clients/src/main/resources/common/message/MetadataRequest.json @4.4.0
Done when: In v0 an empty topic list means all topics; from v1 a null list means all and an empty list none; v1+ responses carry broker rack, controller_id (a random alive broker id, as KRaft brokers report) and is_internal; per-version tests.

### [To Do] API-0070 Metadata v0 unavailable replicas and v5 offline_replicas
P0 · M2 · Task · deps: API-0068 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTopicMetadataRequest errorUnavailableEndpoints @4.4.0
Done when: v0 responses report REPLICA_NOT_AVAILABLE for partitions whose replicas are on unavailable brokers while v1+ silently omit those brokers; offline_replicas is filled from v5; tests with a fenced broker.

### [To Do] API-0071 Metadata v2-v7: cluster_id, listener errors and leader_epoch
P0 · M2 · Task · deps: API-0068 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTopicMetadataRequest errorUnavailableListeners @4.4.0
Done when: cluster_id is returned from v2; a leader without the request's listener yields LEADER_NOT_AVAILABLE for v0-v5 and LISTENER_NOT_FOUND from v6; leader_epoch is returned from v7; per-version tests.

### [To Do] API-0072 Metadata v9-v12: flexible encoding, topic ids in responses and lookup by topic id
P0 · M2 · Task · deps: API-0068 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTopicMetadataRequest @4.4.0 (KIP-516)
Done when: v9 accepts tagged fields; topic_id is returned from v10; v10-v11 requests with a null name or a non-zero topic id fail with INVALID_REQUEST; v12 lookups by id return the topic or UNKNOWN_TOPIC_ID with a null name; tests per version.

### [To Do] API-0073 Metadata v13: top-level error_code
P0 · M2 · Task · deps: API-0068 · ref: clients/src/main/resources/common/message/MetadataResponse.json @4.4.0
Done when: v13 responses carry the top-level error_code, which the 4.4.0 broker leaves at NONE; earlier versions omit it; test asserts both encodings.

### [To Do] API-0074 Metadata responses for unknown and invalid topic names
P0 · M2 · Task · deps: API-0068 · ref: core/src/main/scala/kafka/server/KafkaApis.scala getTopicMetadata @4.4.0
Done when: Requested topics that do not exist and are not auto-created get UNKNOWN_TOPIC_OR_PARTITION, or INVALID_TOPIC_EXCEPTION when the name fails topic-name validation, with is_internal set from the name; all-topic requests never auto-create; tests.

### [To Do] API-0075 Auto-topic creation manager: CreateTopics to the controller
P0 · M2 · Task · deps: API-0068, API-F19 · ref: server/src/main/java/org/apache/kafka/server/AutoTopicCreationManager.java @4.4.0
Done when: A bounded broker component sends CreateTopics for missing topics through the controller channel (NET-F10) using `num.partitions` and `default.replication.factor`, de-duplicates in-flight names, and returns LEADER_NOT_AVAILABLE (or the controller's error such as INVALID_REPLICATION_FACTOR) to the caller; used later by FindCoordinator for internal topics; unit tests with a fake controller.

### [To Do] API-0076 Metadata v4: allow_auto_topic_creation
P0 · M2 · Task · deps: API-0075, API-0074 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTopicMetadataRequest allowAutoCreation @4.4.0
Done when: Missing topics are auto-created only when `auto.create.topics.enable` is true, the request is not for all topics and allow_auto_topic_creation is true (v0-v3 behave as true); tests per version and config.

### [To Do] API-0077 Metadata error mapping per version
P0 · M2 · Task · deps: API-0068 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTopicMetadataRequest @4.4.0
Done when: Topic and partition errors (UNKNOWN_TOPIC_OR_PARTITION, INVALID_TOPIC_EXCEPTION, LEADER_NOT_AVAILABLE, LISTENER_NOT_FOUND, REPLICA_NOT_AVAILABLE, UNKNOWN_TOPIC_ID, TOPIC_AUTHORIZATION_FAILED) come from one exhaustive mapping with the version rules above; table test.

### [To Do] API-0078 Metadata throttle_time_ms through the shared response path
P0 · M2 · Task · deps: API-0068 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is written in v3+ and absent in v0-v2; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0079 Metadata throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0078, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: Metadata records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v6+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0080 Metadata request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0068 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for Metadata; the request-log summary (SRV-F09) shows topic count or all-topics flag, allow_auto_topic_creation and the number of topics auto-created. Each auto-creation emits an event with reason `metadata_request`. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0081 Metadata handler unit tests across v0-13
P0 · M2 · Task · deps: API-0069, API-0070, API-0071, API-0072, API-0073, API-0076, API-0077, API-0078 · labels: test
Done when: Table-driven tests send every version v0-v13 through the handler, with the broker metadata lookups (META-F09) replaced by a fake, and check the service call and the encoded response; cases cover all-topics and named requests at v0 and v1+, unknown and invalid names, auto-creation on and off, listener errors and lookups by id.

### [To Do] API-0082 Metadata handler benchmark for large clusters
P1 · M2 · Task · deps: API-0081, TEST-F11 · labels: perf, nfr
Done when: A TEST-F11 benchmark answers all-topics requests for 10k topics and 200k partitions from the metadata image with allocation bounded by the response size; result recorded against NFR-F02.

### [To Do] API-0083 Metadata compatibility test with Java client 4.4.0 (producer, consumer and Admin.describeTopics), librdkafka, franz-go and kafka-topics.sh --describe
P0 · M2 · Task · deps: API-0081, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that bootstraps, auto-creates a topic, describes topics by name and id and follows a leader change passes with Java client 4.4.0 (producer, consumer and Admin.describeTopics), librdkafka, franz-go and kafka-topics.sh --describe.

### [To Do] API-0084 Metadata differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0081, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical Metadata requests to Kafka 4.4.0 and Alluvium for all-topics, named and by-id requests and for each client-triggerable error (UNKNOWN_TOPIC_OR_PARTITION, INVALID_TOPIC_EXCEPTION, UNKNOWN_TOPIC_ID at v12, INVALID_REQUEST for ids at v10-v11 and LEADER_NOT_AVAILABLE during auto-creation); normalized responses match for every supported version.

### [To Do] API-0085 Advertise Metadata v0-13 in ApiVersions
P0 · M2 · Task · deps: API-0080, API-0082, API-0083, API-0084, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists Metadata with min 0 and max 13 through the generated list owned by API-F18; docs/parity.md marks Metadata supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0086 Metadata authorization: DESCRIBE and CREATE filtering without disclosing topics
P0 · M4 · Task · deps: API-0076, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTopicMetadataRequest @4.4.0
Done when: Topics without DESCRIBE are omitted from all-topics responses and return TOPIC_AUTHORIZATION_FAILED with a zero topic id when named; auto-creation needs CREATE on CLUSTER or on the TOPIC, otherwise TOPIC_AUTHORIZATION_FAILED; by-id requests may return the id with the error; tests and differential cases.

### [To Do] API-0087 Metadata v8-v11: authorized operations
P0 · M4 · Task · deps: API-0086 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTopicMetadataRequest @4.4.0 (KIP-430, KIP-700)
Done when: When requested, v8+ topics carry topic_authorized_operations and v8-v10 carry cluster_authorized_operations (0 without DESCRIBE on CLUSTER); v11 drops the cluster field in favor of DescribeCluster (KIP-700); the default -2147483648 is kept when not requested; tests per version.

### [To Do] API-0088 Metadata auto-creation under the controller mutation quota
P0 · M4 · Task · deps: API-0075, QUOTA-F05 · ref: core/src/main/scala/kafka/server/KafkaApis.scala getTopicMetadata newPermissiveQuotaFor @4.4.0
Done when: Auto-creation uses a permissive controller mutation quota (KIP-599) so Metadata is never rejected with THROTTLING_QUOTA_EXCEEDED while the mutations are still recorded; test with a limited principal.

## [To Do] API-F08 OffsetCommit (v2-10)
P0 · M5 · Feature · deps: GRP-F07
Scope: Kafka 4.4.0 versions 2-10; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0089 OffsetCommit handler module and router registration (broker listener)
P0 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleOffsetCommitRequest @4.4.0
Done when: `handlers/offset_commit.rs` (module doc names the Kafka counterpart) decodes v2-10 with the generated codec and calls the group coordinator (GRP-F07); it is registered for the broker listener in the router (SRV-F04). A OffsetCommit request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0090 OffsetCommit topic and partition resolution
P0 · M5 · Task · deps: API-0089 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleOffsetCommitRequest @4.4.0
Done when: Topics are resolved by name (v2-v9) or id (v10); unknown names or partitions get UNKNOWN_TOPIC_OR_PARTITION and unknown ids UNKNOWN_TOPIC_ID for all their partitions without reaching the coordinator; responses echo names or ids by version; unit tests.

### [To Do] API-0091 OffsetCommit v2-v5: generation and member checks, retention_time_ms
P0 · M5 · Task · deps: API-0090 · ref: clients/src/main/resources/common/message/OffsetCommitRequest.json @4.4.0
Done when: generation_id and member_id are checked by GRP-F07 (ILLEGAL_GENERATION, UNKNOWN_MEMBER_ID; generation -1 with an empty member is a standalone commit); v2-v4 retention_time_ms is accepted and ignored in favor of `offsets.retention.minutes`, and v5 drops the field; per-version tests.

### [To Do] API-0092 OffsetCommit v6-v8: committed_leader_epoch, group_instance_id and flexible encoding
P0 · M5 · Task · deps: API-0091 · ref: clients/src/main/resources/common/message/OffsetCommitRequest.json @4.4.0 (KIP-320, KIP-345)
Done when: v6+ commits store committed_leader_epoch; v7+ group_instance_id is checked for static members (FENCED_INSTANCE_ID); v8 accepts tagged fields; per-version tests.

### [To Do] API-0093 OffsetCommit v9: consumer-protocol member epochs (KIP-848)
P0 · M5 · Task · deps: API-0092 · ref: clients/src/main/resources/common/message/OffsetCommitResponse.json @4.4.0 (KIP-848)
Done when: From v9 generation_id_or_member_epoch carries the member epoch for consumer groups, so STALE_MEMBER_EPOCH and GROUP_ID_NOT_FOUND can be returned; v2-v8 commits to a consumer-protocol group get the older-client errors GRP-F07 maps; tests per version.

### [To Do] API-0094 OffsetCommit v10: commit by topic id
P0 · M5 · Task · deps: API-0093 · ref: clients/src/main/resources/common/message/OffsetCommitRequest.json @4.4.0 (KIP-848)
Done when: v10 requests carry topic ids instead of names and unknown ids get UNKNOWN_TOPIC_ID; tests at v9 and v10 for the same commit.

### [To Do] API-0095 OffsetCommit error mapping per version
P0 · M5 · Task · deps: API-0089 · ref: clients/src/main/resources/common/message/OffsetCommitResponse.json @4.4.0
Done when: Partition errors (NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, ILLEGAL_GENERATION, UNKNOWN_MEMBER_ID, REBALANCE_IN_PROGRESS, INVALID_COMMIT_OFFSET_SIZE, OFFSET_METADATA_TOO_LARGE, FENCED_INSTANCE_ID from v7, GROUP_ID_NOT_FOUND and STALE_MEMBER_EPOCH from v9, UNKNOWN_TOPIC_ID at v10) map exhaustively, never returning a code newer than the request version; table test.

### [To Do] API-0096 OffsetCommit throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M5 · Task · deps: API-0089, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is written in v3+ and absent in v2; OffsetCommit records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v4+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0097 OffsetCommit request metrics, events and request-log summary
P0 · M5 · Task · deps: API-0089 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for OffsetCommit; the request-log summary (SRV-F09) shows group id, generation or member epoch, partition count and whether the commit is standalone. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0098 OffsetCommit authorization: READ on GROUP and READ on topics
P0 · M5 · Task · deps: API-0090, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleOffsetCommitRequest @4.4.0
Done when: Without READ on the GROUP the whole request fails with GROUP_AUTHORIZATION_FAILED; topics without READ on TOPIC get TOPIC_AUTHORIZATION_FAILED for every partition while authorized ones are committed; tests and differential cases.

### [To Do] API-0099 OffsetCommit handler unit tests across v2-10
P0 · M5 · Task · deps: API-0094, API-0095, API-0096, API-0098 · labels: test
Done when: Table-driven tests send every version v2-v10 through the handler, with the group coordinator (GRP-F07) replaced by a fake, and check the service call and the encoded response; cases cover classic and consumer groups, standalone commits, static members, unknown topics and ids, and each error by version.

### [To Do] API-0100 OffsetCommit compatibility test with Java client 4.4.0 consumers (classic and consumer protocols), librdkafka, franz-go and kafka-consumer-groups.sh --reset-offsets --execute
P0 · M5 · Task · deps: API-0099, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that commits synchronously and asynchronously, resets offsets for an inactive group and resumes from them passes with Java client 4.4.0 consumers (classic and consumer protocols), librdkafka, franz-go and kafka-consumer-groups.sh --reset-offsets --execute.

### [To Do] API-0101 OffsetCommit differential test against Kafka 4.4.0
P0 · M5 · Task · deps: API-0099, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical OffsetCommit requests to Kafka 4.4.0 and Alluvium for commits at v2-v10 and for each client-triggerable error (UNKNOWN_TOPIC_OR_PARTITION, UNKNOWN_TOPIC_ID, ILLEGAL_GENERATION, UNKNOWN_MEMBER_ID, FENCED_INSTANCE_ID, STALE_MEMBER_EPOCH and OFFSET_METADATA_TOO_LARGE); normalized responses match for every supported version.

### [To Do] API-0102 Advertise OffsetCommit v2-10 in ApiVersions
P0 · M5 · Task · deps: API-0097, API-0100, API-0101, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists OffsetCommit with min 2 and max 10 through the generated list owned by API-F18; docs/parity.md marks OffsetCommit supported; a test asserts the range.

## [To Do] API-F09 OffsetFetch (v1-10)
P0 · M5 · Feature · deps: GRP-F07
Scope: Kafka 4.4.0 versions 1-10; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0103 OffsetFetch handler module and router registration (broker listener)
P0 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleOffsetFetchRequest @4.4.0
Done when: `handlers/offset_fetch.rs` (module doc names the Kafka counterpart) decodes v1-10 with the generated codec and calls the group coordinator (GRP-F07); it is registered for the broker listener in the router (SRV-F04). A OffsetFetch request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0104 OffsetFetch v1-v4: offsets per partition, all-topics fetch and top-level error
P0 · M5 · Task · deps: API-0103 · ref: clients/src/main/resources/common/message/OffsetFetchRequest.json @4.4.0
Done when: v1 returns committed offset, metadata and a partition error, with group errors copied to every partition; v2+ allows a null topics list for all committed offsets and reports group errors in the top-level error_code; v3 adds throttle_time_ms and v4 changes only client-side throttling; per-version tests.

### [To Do] API-0105 OffsetFetch v5-v7: committed_leader_epoch, flexible encoding and require_stable
P0 · M5 · Task · deps: API-0104 · ref: clients/src/main/resources/common/message/OffsetFetchRequest.json @4.4.0 (KIP-447)
Done when: v5+ returns committed_leader_epoch; v6 accepts tagged fields; with require_stable (v7+) partitions with pending transactional commits return UNSTABLE_OFFSET_COMMIT once TXN-F07 supplies pending state in M6; per-version tests.

### [To Do] API-0106 OffsetFetch v8: several groups per request (KIP-709)
P0 · M5 · Task · deps: API-0105 · ref: clients/src/main/resources/common/message/OffsetFetchRequest.json @4.4.0 (KIP-709)
Done when: v8+ requests carry a groups array; each group is authorized and answered independently with its own error_code, and responses below v8 are built from the single group; tests mixing successful and failing groups.

### [To Do] API-0107 OffsetFetch v9: member id and epoch for consumer groups (KIP-848)
P0 · M5 · Task · deps: API-0106 · ref: clients/src/main/resources/common/message/OffsetFetchRequest.json @4.4.0 (KIP-848)
Done when: v9 member_id and member_epoch are validated for consumer-protocol groups (STALE_MEMBER_EPOCH, UNKNOWN_MEMBER_ID) and ignored when absent; tests.

### [To Do] API-0108 OffsetFetch v10: topic ids
P0 · M5 · Task · deps: API-0107 · ref: core/src/main/scala/kafka/server/KafkaApis.scala fetchOffsetsForGroup @4.4.0
Done when: v10 requests and responses use topic ids; ids missing from metadata get UNKNOWN_TOPIC_ID and all-topics responses carry ids; tests at v9 and v10.

### [To Do] API-0109 OffsetFetch error mapping per version
P0 · M5 · Task · deps: API-0103 · ref: clients/src/main/java/org/apache/kafka/common/requests/OffsetFetchResponse.java @4.4.0
Done when: Group errors (NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, GROUP_ID_NOT_FOUND, GROUP_AUTHORIZATION_FAILED) are placed on partitions for v1, at the top level for v2-v7 and per group from v8; UNSTABLE_OFFSET_COMMIT only from v7, UNKNOWN_MEMBER_ID and STALE_MEMBER_EPOCH from v9, UNKNOWN_TOPIC_ID at v10; table test.

### [To Do] API-0110 OffsetFetch throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M5 · Task · deps: API-0103, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is written in v3+ and absent in v1-v2; OffsetFetch records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v4+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0111 OffsetFetch request metrics, events and request-log summary
P0 · M5 · Task · deps: API-0103 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for OffsetFetch; the request-log summary (SRV-F09) shows group count, topic count or all-topics flag and require_stable. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0112 OffsetFetch authorization: DESCRIBE on GROUP and DESCRIBE on topics
P0 · M5 · Task · deps: API-0106, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleOffsetFetchRequest @4.4.0
Done when: Each group needs DESCRIBE on GROUP or gets GROUP_AUTHORIZATION_FAILED; requested topics without DESCRIBE get TOPIC_AUTHORIZATION_FAILED and all-topics fetches silently omit them; tests and differential cases.

### [To Do] API-0113 OffsetFetch handler unit tests across v1-10
P0 · M5 · Task · deps: API-0108, API-0109, API-0110, API-0112 · labels: test
Done when: Table-driven tests send every version v1-v10 through the handler, with the group coordinator (GRP-F07) replaced by a fake, and check the service call and the encoded response; cases cover single and several groups, all-topics fetches, require_stable, consumer-protocol members and each version's error placement.

### [To Do] API-0114 OffsetFetch compatibility test with Java client 4.4.0 consumers and Admin.listConsumerGroupOffsets, librdkafka, franz-go and kafka-consumer-groups.sh --describe
P0 · M5 · Task · deps: API-0113, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that reads committed offsets for one and several groups passes with Java client 4.4.0 consumers and Admin.listConsumerGroupOffsets, librdkafka, franz-go and kafka-consumer-groups.sh --describe.

### [To Do] API-0115 OffsetFetch differential test against Kafka 4.4.0
P0 · M5 · Task · deps: API-0113, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical OffsetFetch requests to Kafka 4.4.0 and Alluvium for fetches at v1-v10 and for each client-triggerable error (GROUP_ID_NOT_FOUND, UNKNOWN_TOPIC_ID, STALE_MEMBER_EPOCH and coordinator errors at each placement); normalized responses match for every supported version.

### [To Do] API-0116 Advertise OffsetFetch v1-10 in ApiVersions
P0 · M5 · Task · deps: API-0111, API-0114, API-0115, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists OffsetFetch with min 1 and max 10 through the generated list owned by API-F18; docs/parity.md marks OffsetFetch supported; a test asserts the range. Later tasks (M6) add behavior without changing the advertised range.

### [To Do] API-0117 OffsetFetch require_stable with pending transactional offsets
P0 · M6 · Task · deps: API-0105, TXN-F07 · ref: clients/src/main/resources/common/message/OffsetFetchResponse.json @4.4.0 (KIP-447)
Done when: With TXN-F07 pending offsets, v7+ require_stable fetches return UNSTABLE_OFFSET_COMMIT for affected partitions until the transaction completes, and older versions or require_stable false return the last stable commit; differential test with an open transaction.

## [To Do] API-F10 FindCoordinator (v0-6)
P0 · M5 · Feature · deps: GRP-F03
Scope: Kafka 4.4.0 versions 0-6; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0118 FindCoordinator handler module and router registration (broker listener)
P0 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFindCoordinatorRequest @4.4.0
Done when: `handlers/find_coordinator.rs` (module doc names the Kafka counterpart) decodes v0-6 with the generated codec and calls the coordinator lookup (GRP-F03); it is registered for the broker listener in the router (SRV-F04). A FindCoordinator request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0119 FindCoordinator lookup and internal-topic creation
P0 · M5 · Task · deps: API-0118, API-F03 · ref: core/src/main/scala/kafka/server/KafkaApis.scala getCoordinator @4.4.0
Done when: The key is hashed to a partition of `__consumer_offsets` or `__transaction_state` and that partition leader's endpoint for the request's listener is returned; a missing internal topic is created through the auto-topic creation manager while the caller gets COORDINATOR_NOT_AVAILABLE; a missing leader or endpoint also yields COORDINATOR_NOT_AVAILABLE; tests.

### [To Do] API-0120 FindCoordinator v0-v3: single key, key_type and error_message
P0 · M5 · Task · deps: API-0119 · ref: clients/src/main/resources/common/message/FindCoordinatorRequest.json @4.4.0
Done when: v0 looks up a group key; v1+ adds key_type (0 group, 1 transaction; other values INVALID_REQUEST), throttle time and error_message; v3 accepts tagged fields; per-version tests.

### [To Do] API-0121 FindCoordinator v4: batched coordinator_keys (KIP-699)
P0 · M5 · Task · deps: API-0120 · ref: clients/src/main/resources/common/message/FindCoordinatorRequest.json @4.4.0 (KIP-699)
Done when: v4+ requests carry coordinator_keys and get one coordinators entry per key with its own error_code and error_message; tests with mixed successes and failures.

### [To Do] API-0122 FindCoordinator v5: clients that understand TRANSACTION_ABORTABLE (KIP-890)
P0 · M5 · Task · deps: API-0121 · ref: clients/src/main/resources/common/message/FindCoordinatorRequest.json @4.4.0 (KIP-890)
Done when: v5 has the v4 layout and only signals that the client handles TRANSACTION_ABORTABLE; the handler treats it like v4; test that v4 and v5 responses carry identical bodies.

### [To Do] API-0123 FindCoordinator error mapping per version
P0 · M5 · Task · deps: API-0118 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFindCoordinatorRequest @4.4.0
Done when: COORDINATOR_NOT_AVAILABLE, INVALID_REQUEST, GROUP_AUTHORIZATION_FAILED and TRANSACTIONAL_ID_AUTHORIZATION_FAILED are top-level below v4 and per key from v4, with node id -1 and empty host; table test.

### [To Do] API-0124 FindCoordinator throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M5 · Task · deps: API-0118, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is written in v1+ and absent in v0; FindCoordinator records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v2+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0125 FindCoordinator request metrics, events and request-log summary
P0 · M5 · Task · deps: API-0118 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for FindCoordinator; the request-log summary (SRV-F09) shows key type, key count and the coordinator chosen. Internal-topic auto-creation triggered by a lookup emits an event with reason `find_coordinator`. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0126 FindCoordinator authorization: DESCRIBE on GROUP or TRANSACTIONAL_ID
P0 · M5 · Task · deps: API-0121, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala getCoordinator @4.4.0
Done when: Group keys need DESCRIBE on GROUP (else GROUP_AUTHORIZATION_FAILED) and transaction keys DESCRIBE on TRANSACTIONAL_ID (else TRANSACTIONAL_ID_AUTHORIZATION_FAILED), checked per key; tests and differential cases.

### [To Do] API-0127 FindCoordinator handler unit tests across v0-6
P0 · M5 · Task · deps: API-0122, API-0123, API-0124, API-0126 · labels: test
Done when: Table-driven tests send every version v0-v6 through the handler, with the coordinator lookup (GRP-F03) replaced by a fake, and check the service call and the encoded response; cases cover group and transaction keys, batched lookups, missing internal topics, missing leaders and invalid key types.

### [To Do] API-0128 FindCoordinator compatibility test with Java client 4.4.0 consumer, producer and Admin, librdkafka and franz-go
P0 · M5 · Task · deps: API-0127, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that finds group and transaction coordinators, including the first lookup that creates `__consumer_offsets`, passes with Java client 4.4.0 consumer, producer and Admin, librdkafka and franz-go.

### [To Do] API-0129 FindCoordinator differential test against Kafka 4.4.0
P0 · M5 · Task · deps: API-0127, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical FindCoordinator requests to Kafka 4.4.0 and Alluvium for single and batched lookups at v0-v5 and for each client-triggerable error (COORDINATOR_NOT_AVAILABLE, INVALID_REQUEST for unknown key types and authorization failures); normalized responses match for every supported version.

### [To Do] API-0130 Advertise FindCoordinator v0-6 in ApiVersions
P0 · M5 · Task · deps: API-0125, API-0128, API-0129, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists FindCoordinator with min 0 and max 6 through the generated list owned by API-F18; docs/parity.md marks FindCoordinator supported; a test asserts the range. Later tasks (M8) add behavior without changing the advertised range.

### [To Do] API-0131 FindCoordinator v6: share-group coordinator keys (KIP-932)
P0 · M8 · Task · deps: API-0121, SHARE-F05 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala getCoordinator @4.4.0 (KIP-932)
Done when: Key type 2 (share) is accepted only from v6, otherwise INVALID_REQUEST; keys must have the form `groupId:topicId:partition` (else INVALID_REQUEST), need CLUSTER_ACTION, and map to `__share_group_state` through SHARE-F05; tests and a differential case.

## [To Do] API-F11 JoinGroup (v0-9)
P0 · M5 · Feature · deps: GRP-F04
Scope: Kafka 4.4.0 versions 0-9; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0132 JoinGroup handler module and router registration (broker listener)
P0 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleJoinGroupRequest @4.4.0
Done when: `handlers/join_group.rs` (module doc names the Kafka counterpart) decodes v0-9 with the generated codec and calls the classic group coordinator (GRP-F04); it is registered for the broker listener in the router (SRV-F04). A JoinGroup request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0133 JoinGroup v0-v3: session and rebalance timeouts
P0 · M5 · Task · deps: API-0132 · ref: clients/src/main/resources/common/message/JoinGroupRequest.json @4.4.0
Done when: v0 uses session_timeout_ms as the rebalance timeout; v1+ passes rebalance_timeout_ms; v2 adds throttle_time_ms; out-of-range session timeouts get INVALID_SESSION_TIMEOUT from GRP-F04; per-version tests.

### [To Do] API-0134 JoinGroup v4: MEMBER_ID_REQUIRED two-step join (KIP-394)
P0 · M5 · Task · deps: API-0133 · ref: clients/src/main/resources/common/message/JoinGroupRequest.json @4.4.0 (KIP-394)
Done when: A v4+ dynamic member joining with an empty member_id gets MEMBER_ID_REQUIRED with its assigned member_id and must rejoin; v0-v3 members are admitted directly; tests per version.

### [To Do] API-0135 JoinGroup v5: group_instance_id for static membership (KIP-345)
P0 · M5 · Task · deps: API-0134 · ref: clients/src/main/resources/common/message/JoinGroupRequest.json @4.4.0 (KIP-345)
Done when: v5+ group_instance_id is passed to GRP-F04 (FENCED_INSTANCE_ID, static member replacement) and returned per member to the leader; tests.

### [To Do] API-0136 JoinGroup v6-v9: protocol_type in responses, reason and skip_assignment
P0 · M5 · Task · deps: API-0135 · ref: clients/src/main/java/org/apache/kafka/common/requests/JoinGroupResponse.java @4.4.0 (KIP-559, KIP-800, KIP-814)
Done when: v6 accepts tagged fields; v7+ responses carry protocol_type and a nullable protocol_name (v0-v6 get an empty string instead of null, as `JoinGroupResponse` does); v8+ reason is passed to the rebalance event; v9 returns skip_assignment when the leader must not assign; per-version tests.

### [To Do] API-0137 JoinGroup error mapping per version
P0 · M5 · Task · deps: API-0132 · ref: clients/src/main/java/org/apache/kafka/common/requests/JoinGroupRequest.java getErrorResponse @4.4.0
Done when: NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, INVALID_GROUP_ID, INCONSISTENT_GROUP_PROTOCOL, INVALID_SESSION_TIMEOUT, UNKNOWN_MEMBER_ID, MEMBER_ID_REQUIRED (v4+), FENCED_INSTANCE_ID (v5+), GROUP_MAX_SIZE_REACHED and REBALANCE_IN_PROGRESS map exhaustively in the `JoinGroupRequest.getErrorResponse` shape; table test.

### [To Do] API-0138 JoinGroup throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M5 · Task · deps: API-0132, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is written in v2+ and absent in v0-v1; JoinGroup records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v3+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0139 JoinGroup request metrics, events and request-log summary
P0 · M5 · Task · deps: API-0132 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for JoinGroup; the request-log summary (SRV-F09) shows group id, member id, instance id, protocol type and names, and the reason from v8. The client reason is carried into the GRP-F04 rebalance event. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0140 JoinGroup authorization: READ on GROUP
P0 · M5 · Task · deps: API-0132, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleJoinGroupRequest @4.4.0
Done when: Requests without READ on the GROUP get GROUP_AUTHORIZATION_FAILED; tests and a differential case.

### [To Do] API-0141 JoinGroup handler unit tests across v0-9
P0 · M5 · Task · deps: API-0136, API-0137, API-0138, API-0140 · labels: test
Done when: Table-driven tests send every version v0-v9 through the handler, with the classic group coordinator (GRP-F04) replaced by a fake, and check the service call and the encoded response; cases cover leader and follower joins, member id required, static members, protocol mismatch and each version's shape.

### [To Do] API-0142 JoinGroup compatibility test with Java client 4.4.0 classic-protocol consumers, librdkafka, franz-go and a Kafka Connect-style custom protocol
P0 · M5 · Task · deps: API-0141, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that forms groups of 1, 3 and 10 members with eager and cooperative assignors and static membership passes with Java client 4.4.0 classic-protocol consumers, librdkafka, franz-go and a Kafka Connect-style custom protocol.

### [To Do] API-0143 JoinGroup differential test against Kafka 4.4.0
P0 · M5 · Task · deps: API-0141, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical JoinGroup requests to Kafka 4.4.0 and Alluvium for joins at v0-v9 and for each client-triggerable error (MEMBER_ID_REQUIRED, INCONSISTENT_GROUP_PROTOCOL, INVALID_SESSION_TIMEOUT and FENCED_INSTANCE_ID); normalized responses match for every supported version.

### [To Do] API-0144 Advertise JoinGroup v0-9 in ApiVersions
P0 · M5 · Task · deps: API-0139, API-0142, API-0143, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists JoinGroup with min 0 and max 9 through the generated list owned by API-F18; docs/parity.md marks JoinGroup supported; a test asserts the range.

## [To Do] API-F12 Heartbeat (v0-4)
P0 · M5 · Feature · deps: GRP-F04
Scope: Kafka 4.4.0 versions 0-4; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0145 Heartbeat handler module and router registration (broker listener)
P0 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleHeartbeatRequest @4.4.0
Done when: `handlers/heartbeat.rs` (module doc names the Kafka counterpart) decodes v0-4 with the generated codec and calls the classic group coordinator (GRP-F04); it is registered for the broker listener in the router (SRV-F04). A Heartbeat request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0146 Heartbeat v0-v2: generation and member liveness
P0 · M5 · Task · deps: API-0145 · ref: clients/src/main/resources/common/message/HeartbeatRequest.json @4.4.0
Done when: generation_id and member_id are passed to GRP-F04, which returns NONE, REBALANCE_IN_PROGRESS, ILLEGAL_GENERATION or UNKNOWN_MEMBER_ID; v1 adds throttle_time_ms; per-version tests.

### [To Do] API-0147 Heartbeat v3-v4: group_instance_id and flexible encoding
P0 · M5 · Task · deps: API-0146 · ref: clients/src/main/resources/common/message/HeartbeatRequest.json @4.4.0 (KIP-345)
Done when: v3+ group_instance_id lets the coordinator fence stale static members with FENCED_INSTANCE_ID; v4 accepts tagged fields; per-version tests.

### [To Do] API-0148 Heartbeat error mapping per version
P0 · M5 · Task · deps: API-0145 · ref: clients/src/main/resources/common/message/HeartbeatResponse.json @4.4.0
Done when: NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, ILLEGAL_GENERATION, UNKNOWN_MEMBER_ID, REBALANCE_IN_PROGRESS, FENCED_INSTANCE_ID (v3+) and GROUP_AUTHORIZATION_FAILED map exhaustively; table test.

### [To Do] API-0149 Heartbeat throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M5 · Task · deps: API-0145, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is written in v1+ and absent in v0; Heartbeat records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v2+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0150 Heartbeat request metrics, events and request-log summary
P0 · M5 · Task · deps: API-0145 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for Heartbeat; the request-log summary (SRV-F09) shows group id, member id, generation and instance id. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0151 Heartbeat authorization: READ on GROUP
P0 · M5 · Task · deps: API-0145, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleHeartbeatRequest @4.4.0
Done when: Requests without READ on the GROUP get GROUP_AUTHORIZATION_FAILED; tests and a differential case.

### [To Do] API-0152 Heartbeat handler unit tests across v0-4
P0 · M5 · Task · deps: API-0147, API-0148, API-0149, API-0151 · labels: test
Done when: Table-driven tests send every version v0-v4 through the handler, with the classic group coordinator (GRP-F04) replaced by a fake, and check the service call and the encoded response; cases cover stable, rebalancing, stale generation, unknown and fenced members.

### [To Do] API-0153 Heartbeat compatibility test with Java client 4.4.0 classic consumers, librdkafka and franz-go
P0 · M5 · Task · deps: API-0152, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that keeps a group stable through long polls and starts a rebalance when a member joins passes with Java client 4.4.0 classic consumers, librdkafka and franz-go.

### [To Do] API-0154 Heartbeat differential test against Kafka 4.4.0
P0 · M5 · Task · deps: API-0152, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical Heartbeat requests to Kafka 4.4.0 and Alluvium for heartbeats at v0-v4 and for each client-triggerable error (REBALANCE_IN_PROGRESS, ILLEGAL_GENERATION, UNKNOWN_MEMBER_ID and FENCED_INSTANCE_ID); normalized responses match for every supported version.

### [To Do] API-0155 Advertise Heartbeat v0-4 in ApiVersions
P0 · M5 · Task · deps: API-0150, API-0153, API-0154, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists Heartbeat with min 0 and max 4 through the generated list owned by API-F18; docs/parity.md marks Heartbeat supported; a test asserts the range.

## [To Do] API-F13 LeaveGroup (v0-5)
P0 · M5 · Feature · deps: GRP-F04
Scope: Kafka 4.4.0 versions 0-5; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0156 LeaveGroup handler module and router registration (broker listener)
P0 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleLeaveGroupRequest @4.4.0
Done when: `handlers/leave_group.rs` (module doc names the Kafka counterpart) decodes v0-5 with the generated codec and calls the classic group coordinator (GRP-F04); it is registered for the broker listener in the router (SRV-F04). A LeaveGroup request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0157 LeaveGroup v0-v2: single member leave
P0 · M5 · Task · deps: API-0156 · ref: clients/src/main/java/org/apache/kafka/common/requests/LeaveGroupRequest.java normalizedData @4.4.0
Done when: v0-v2 member_id is normalized into a one-member batch and the single response error_code comes from the group error or that member's error (`LeaveGroupResponse` v0-v2 shape); v1 adds throttle_time_ms; tests.

### [To Do] API-0158 LeaveGroup v3-v4: batched leave by member id and group_instance_id (KIP-345)
P0 · M5 · Task · deps: API-0157 · ref: clients/src/main/resources/common/message/LeaveGroupRequest.json @4.4.0 (KIP-345)
Done when: v3+ requests list members with member_id and group_instance_id and get per-member error codes plus a top-level error; v4 accepts tagged fields; tests with mixed outcomes.

### [To Do] API-0159 LeaveGroup v5: per-member reason (KIP-800)
P0 · M5 · Task · deps: API-0158 · ref: clients/src/main/resources/common/message/LeaveGroupRequest.json @4.4.0 (KIP-800)
Done when: v5 member reasons are carried into the rebalance event; below v5 the default reason is used; tests.

### [To Do] API-0160 LeaveGroup error mapping per version
P0 · M5 · Task · deps: API-0156 · ref: clients/src/main/java/org/apache/kafka/common/requests/LeaveGroupResponse.java @4.4.0
Done when: NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, UNKNOWN_MEMBER_ID, FENCED_INSTANCE_ID and GROUP_AUTHORIZATION_FAILED map exhaustively and are folded into the single error for v0-v2; table test.

### [To Do] API-0161 LeaveGroup throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M5 · Task · deps: API-0156, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is written in v1+ and absent in v0; LeaveGroup records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v2+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0162 LeaveGroup request metrics, events and request-log summary
P0 · M5 · Task · deps: API-0156 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for LeaveGroup; the request-log summary (SRV-F09) shows group id, member count and reasons. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0163 LeaveGroup authorization: READ on GROUP
P0 · M5 · Task · deps: API-0156, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleLeaveGroupRequest @4.4.0
Done when: Requests without READ on the GROUP get GROUP_AUTHORIZATION_FAILED; tests and a differential case.

### [To Do] API-0164 LeaveGroup handler unit tests across v0-5
P0 · M5 · Task · deps: API-0159, API-0160, API-0161, API-0163 · labels: test
Done when: Table-driven tests send every version v0-v5 through the handler, with the classic group coordinator (GRP-F04) replaced by a fake, and check the service call and the encoded response; cases cover dynamic and static leaves, batches, unknown members and the v0-v2 error folding.

### [To Do] API-0165 LeaveGroup compatibility test with Java client 4.4.0 classic consumers and Admin.removeMembersFromConsumerGroup, librdkafka and kafka-consumer-groups.sh
P0 · M5 · Task · deps: API-0164, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that closes consumers cleanly and removes static members administratively passes with Java client 4.4.0 classic consumers and Admin.removeMembersFromConsumerGroup, librdkafka and kafka-consumer-groups.sh.

### [To Do] API-0166 LeaveGroup differential test against Kafka 4.4.0
P0 · M5 · Task · deps: API-0164, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical LeaveGroup requests to Kafka 4.4.0 and Alluvium for leaves at v0-v5 and for each client-triggerable error (UNKNOWN_MEMBER_ID and FENCED_INSTANCE_ID); normalized responses match for every supported version.

### [To Do] API-0167 Advertise LeaveGroup v0-5 in ApiVersions
P0 · M5 · Task · deps: API-0162, API-0165, API-0166, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists LeaveGroup with min 0 and max 5 through the generated list owned by API-F18; docs/parity.md marks LeaveGroup supported; a test asserts the range.

## [To Do] API-F14 SyncGroup (v0-5)
P0 · M5 · Feature · deps: GRP-F04
Scope: Kafka 4.4.0 versions 0-5; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0168 SyncGroup handler module and router registration (broker listener)
P0 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleSyncGroupRequest @4.4.0
Done when: `handlers/sync_group.rs` (module doc names the Kafka counterpart) decodes v0-5 with the generated codec and calls the classic group coordinator (GRP-F04); it is registered for the broker listener in the router (SRV-F04). A SyncGroup request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0169 SyncGroup v0-v4: assignment distribution and group_instance_id
P0 · M5 · Task · deps: API-0168 · ref: clients/src/main/resources/common/message/SyncGroupRequest.json @4.4.0
Done when: The leader's assignments are passed to GRP-F04 and each member receives its own; v1 adds throttle_time_ms, v3+ group_instance_id (FENCED_INSTANCE_ID) and v4 tagged fields; per-version tests.

### [To Do] API-0170 SyncGroup v5: mandatory protocol_type and protocol_name (KIP-559)
P0 · M5 · Task · deps: API-0169 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleSyncGroupRequest @4.4.0 (KIP-559)
Done when: A v5 request missing protocol_type or protocol_name gets INCONSISTENT_GROUP_PROTOCOL before authorization; values that differ from the group's also get INCONSISTENT_GROUP_PROTOCOL; v5 responses echo both; tests.

### [To Do] API-0171 SyncGroup error mapping per version
P0 · M5 · Task · deps: API-0168 · ref: clients/src/main/resources/common/message/SyncGroupResponse.json @4.4.0
Done when: NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, ILLEGAL_GENERATION, UNKNOWN_MEMBER_ID, REBALANCE_IN_PROGRESS, FENCED_INSTANCE_ID (v3+) and INCONSISTENT_GROUP_PROTOCOL map exhaustively; table test.

### [To Do] API-0172 SyncGroup throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M5 · Task · deps: API-0168, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is written in v1+ and absent in v0; SyncGroup records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v2+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0173 SyncGroup request metrics, events and request-log summary
P0 · M5 · Task · deps: API-0168 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for SyncGroup; the request-log summary (SRV-F09) shows group id, member id, generation and the leader's assignment count. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0174 SyncGroup authorization: READ on GROUP
P0 · M5 · Task · deps: API-0170, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleSyncGroupRequest @4.4.0
Done when: Requests without READ on the GROUP get GROUP_AUTHORIZATION_FAILED, checked after the v5 protocol check; tests and a differential case.

### [To Do] API-0175 SyncGroup handler unit tests across v0-5
P0 · M5 · Task · deps: API-0171, API-0172, API-0174 · labels: test
Done when: Table-driven tests send every version v0-v5 through the handler, with the classic group coordinator (GRP-F04) replaced by a fake, and check the service call and the encoded response; cases cover leader and follower syncs, missing v5 protocol fields, stale generations and fenced static members.

### [To Do] API-0176 SyncGroup compatibility test with Java client 4.4.0 classic consumers, librdkafka and franz-go
P0 · M5 · Task · deps: API-0175, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that completes rebalances with range, round-robin, sticky and cooperative-sticky assignors passes with Java client 4.4.0 classic consumers, librdkafka and franz-go.

### [To Do] API-0177 SyncGroup differential test against Kafka 4.4.0
P0 · M5 · Task · deps: API-0175, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical SyncGroup requests to Kafka 4.4.0 and Alluvium for syncs at v0-v5 and for each client-triggerable error (INCONSISTENT_GROUP_PROTOCOL, ILLEGAL_GENERATION and REBALANCE_IN_PROGRESS); normalized responses match for every supported version.

### [To Do] API-0178 Advertise SyncGroup v0-5 in ApiVersions
P0 · M5 · Task · deps: API-0173, API-0176, API-0177, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists SyncGroup with min 0 and max 5 through the generated list owned by API-F18; docs/parity.md marks SyncGroup supported; a test asserts the range.

## [To Do] API-F15 DescribeGroups (v0-6)
P1 · M5 · Feature · deps: GRP-F08
Scope: Kafka 4.4.0 versions 0-6; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0179 DescribeGroups handler module and router registration (broker listener)
P1 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeGroupsRequest @4.4.0
Done when: `handlers/describe_groups.rs` (module doc names the Kafka counterpart) decodes v0-6 with the generated codec and calls the group administration service (GRP-F08); it is registered for the broker listener in the router (SRV-F04). A DescribeGroups request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0180 DescribeGroups v0-v4: state, protocol, members and group_instance_id
P1 · M5 · Task · deps: API-0179 · ref: clients/src/main/resources/common/message/DescribeGroupsResponse.json @4.4.0
Done when: Each group returns state, protocol type, protocol data and members, with protocol data and member assignments only when Stable and no members when Dead; v4+ members carry group_instance_id; per-version tests.

### [To Do] API-0181 DescribeGroups v5-v6: flexible encoding and GROUP_ID_NOT_FOUND (KIP-1043)
P1 · M5 · Task · deps: API-0180 · ref: clients/src/main/resources/common/message/DescribeGroupsRequest.json @4.4.0 (KIP-1043)
Done when: v5 accepts tagged fields; an unknown group gets GROUP_ID_NOT_FOUND with error_message at v6 and NONE with state Dead at v0-v5; per-version tests.

### [To Do] API-0182 DescribeGroups error mapping per version
P1 · M5 · Task · deps: API-0179 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeGroupsRequest @4.4.0
Done when: Per-group NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, GROUP_AUTHORIZATION_FAILED and GROUP_ID_NOT_FOUND (v6 only) map exhaustively, with error_message from v6; table test.

### [To Do] API-0183 DescribeGroups throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M5 · Task · deps: API-0179, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is written in v1+ and absent in v0; DescribeGroups records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v2+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0184 DescribeGroups request metrics, events and request-log summary
P1 · M5 · Task · deps: API-0179 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeGroups; the request-log summary (SRV-F09) shows group count and include_authorized_operations. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0185 DescribeGroups authorization: DESCRIBE on each group and authorized operations (v3+)
P1 · M5 · Task · deps: API-0179, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeGroupsRequest @4.4.0
Done when: Groups without DESCRIBE on GROUP get GROUP_AUTHORIZATION_FAILED; with include_authorized_operations (v3+) successful groups carry authorized_operations; tests and differential cases.

### [To Do] API-0186 DescribeGroups handler unit tests across v0-6
P1 · M5 · Task · deps: API-0181, API-0182, API-0183, API-0185 · labels: test
Done when: Table-driven tests send every version v0-v6 through the handler, with the group administration service (GRP-F08) replaced by a fake, and check the service call and the encoded response; cases cover stable, rebalancing, empty, dead and unknown groups, consumer-protocol groups and authorized operations.

### [To Do] API-0187 DescribeGroups compatibility test with Java client 4.4.0 Admin.describeConsumerGroups, librdkafka and kafka-consumer-groups.sh --describe --members --state
P1 · M5 · Task · deps: API-0186, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that describes classic groups in every state passes with Java client 4.4.0 Admin.describeConsumerGroups, librdkafka and kafka-consumer-groups.sh --describe --members --state.

### [To Do] API-0188 DescribeGroups differential test against Kafka 4.4.0
P1 · M5 · Task · deps: API-0186, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeGroups requests to Kafka 4.4.0 and Alluvium for descriptions at v0-v6 and for each client-triggerable error (GROUP_ID_NOT_FOUND at v6 versus Dead at v5 and coordinator errors); normalized responses match for every supported version.

### [To Do] API-0189 Advertise DescribeGroups v0-6 in ApiVersions
P1 · M5 · Task · deps: API-0184, API-0187, API-0188, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists DescribeGroups with min 0 and max 6 through the generated list owned by API-F18; docs/parity.md marks DescribeGroups supported; a test asserts the range.

## [To Do] API-F16 ListGroups (v0-5)
P1 · M5 · Feature · deps: GRP-F08
Scope: Kafka 4.4.0 versions 0-5; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0190 ListGroups handler module and router registration (broker listener)
P1 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListGroupsRequest @4.4.0
Done when: `handlers/list_groups.rs` (module doc names the Kafka counterpart) decodes v0-5 with the generated codec and calls the group administration service (GRP-F08); it is registered for the broker listener in the router (SRV-F04). A ListGroups request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0191 ListGroups v0-v3: all groups with protocol type
P1 · M5 · Task · deps: API-0190 · ref: clients/src/main/resources/common/message/ListGroupsRequest.json @4.4.0
Done when: Every group on this broker's coordinator partitions is returned with protocol_type; v1 adds throttle_time_ms and v3 tagged fields; COORDINATOR_LOAD_IN_PROGRESS is returned while partitions load; per-version tests.

### [To Do] API-0192 ListGroups v4: states_filter and group_state (KIP-518)
P1 · M5 · Task · deps: API-0191 · ref: clients/src/main/resources/common/message/ListGroupsRequest.json @4.4.0 (KIP-518)
Done when: v4+ states_filter (case-insensitive) restricts results and each group carries group_state; tests.

### [To Do] API-0193 ListGroups v5: types_filter and group_type (KIP-848)
P1 · M5 · Task · deps: API-0192 · ref: clients/src/main/resources/common/message/ListGroupsRequest.json @4.4.0 (KIP-848)
Done when: v5 types_filter selects classic, consumer, share or streams groups and each group carries group_type; tests.

### [To Do] API-0194 ListGroups error mapping
P1 · M5 · Task · deps: API-0190 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListGroupsRequest @4.4.0
Done when: Top-level NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS and NONE map exhaustively; table test.

### [To Do] API-0195 ListGroups throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M5 · Task · deps: API-0190, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is written in v1+ and absent in v0; ListGroups records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v2+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0196 ListGroups request metrics, events and request-log summary
P1 · M5 · Task · deps: API-0190 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ListGroups; the request-log summary (SRV-F09) shows state and type filters and the group count returned. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0197 ListGroups authorization: DESCRIBE on CLUSTER or per-group filtering
P1 · M5 · Task · deps: API-0190, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListGroupsRequest @4.4.0
Done when: With DESCRIBE on CLUSTER every group is returned; otherwise only groups with DESCRIBE on GROUP, and these checks are not logged as denials; tests and differential cases.

### [To Do] API-0198 ListGroups handler unit tests across v0-5
P1 · M5 · Task · deps: API-0193, API-0194, API-0195, API-0197 · labels: test
Done when: Table-driven tests send every version v0-v5 through the handler, with the group administration service (GRP-F08) replaced by a fake, and check the service call and the encoded response; cases cover empty brokers, state and type filters and loading partitions.

### [To Do] API-0199 ListGroups compatibility test with Java client 4.4.0 Admin.listGroups, librdkafka, kafka-consumer-groups.sh --list and kafka-groups.sh --list
P1 · M5 · Task · deps: API-0198, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that lists groups by state and type passes with Java client 4.4.0 Admin.listGroups, librdkafka, kafka-consumer-groups.sh --list and kafka-groups.sh --list.

### [To Do] API-0200 ListGroups differential test against Kafka 4.4.0
P1 · M5 · Task · deps: API-0198, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ListGroups requests to Kafka 4.4.0 and Alluvium for listings with every filter at v0-v5 and for each client-triggerable error (COORDINATOR_LOAD_IN_PROGRESS); normalized responses match for every supported version.

### [To Do] API-0201 Advertise ListGroups v0-5 in ApiVersions
P1 · M5 · Task · deps: API-0196, API-0199, API-0200, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists ListGroups with min 0 and max 5 through the generated list owned by API-F18; docs/parity.md marks ListGroups supported; a test asserts the range.

## [To Do] API-F17 SaslHandshake (v0-1)
P0 · M4 · Feature · deps: NET-F08, SEC-F02
Scope: Kafka 4.4.0 versions 0-1; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0202 SaslHandshake handler module and router registration (broker and controller listeners)
P0 · M4 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java handleHandshakeRequest @4.4.0
Done when: `handlers/sasl_handshake.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the SASL driver (NET-F08); it is registered for the broker and controller listeners in the router (SRV-F04). Before authentication the request is answered by the connection's SASL state machine; after authentication the handler returns ILLEGAL_SASL_STATE as `KafkaApis.handleSaslHandshakeRequest` does. Unit test per listener.

### [To Do] API-0203 SaslHandshake mechanism selection and UNSUPPORTED_SASL_MECHANISM
P0 · M4 · Task · deps: API-0202 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java handleHandshakeRequest @4.4.0
Done when: The requested mechanism is matched against the listener's enabled mechanisms (`sasl.enabled.mechanisms`, overridable per listener); an unknown or disabled one gets UNSUPPORTED_SASL_MECHANISM with the enabled list in mechanisms and the connection is closed after the response; unit tests per listener.

### [To Do] API-0204 SaslHandshake v0: raw SASL tokens after the handshake
P0 · M4 · Task · deps: API-0203 · ref: clients/src/main/resources/common/message/SaslHandshakeRequest.json @4.4.0
Done when: After a v0 handshake the client's tokens are raw length-prefixed SASL bytes outside Kafka request framing, as pre-KIP-152 clients send them; the SASL driver switches framing accordingly; test with a raw-token PLAIN exchange.

### [To Do] API-0205 SaslHandshake v1: tokens carried in SaslAuthenticate (KIP-152)
P0 · M4 · Task · deps: API-0203 · ref: clients/src/main/resources/common/message/SaslHandshakeRequest.json @4.4.0 (KIP-152)
Done when: After a v1 handshake tokens must arrive in SaslAuthenticate requests; any other request before authentication completes closes the connection; tests.

### [To Do] API-0206 SaslHandshake error mapping
P0 · M4 · Task · deps: API-0202 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleSaslHandshakeRequest @4.4.0
Done when: UNSUPPORTED_SASL_MECHANISM, ILLEGAL_SASL_STATE (a handshake after authentication or a second handshake) and UNSUPPORTED_VERSION map exhaustively; table test.

### [To Do] API-0207 SaslHandshake request metrics, events and request-log summary
P0 · M4 · Task · deps: API-0202 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for SaslHandshake; the request-log summary (SRV-F09) shows listener, requested mechanism and outcome, never any credential bytes. Handshake failures emit an authentication event with reason `unsupported_mechanism` (SEC-F02). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0208 SaslHandshake handler unit tests across v0-1
P0 · M4 · Task · deps: API-0204, API-0205, API-0206 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the SASL driver (NET-F08) replaced by a fake, and check the service call and the encoded response; cases cover every enabled mechanism, disabled mechanisms, v0 and v1 framing and handshakes after authentication.

### [To Do] API-0209 SaslHandshake compatibility test with Java client 4.4.0, librdkafka, franz-go and kafka-topics.sh with SASL PLAIN, SCRAM-SHA-256/512 and OAUTHBEARER
P0 · M4 · Task · deps: API-0208, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that authenticates over SASL_PLAINTEXT and SASL_SSL listeners and runs a produce and consume passes with Java client 4.4.0, librdkafka, franz-go and kafka-topics.sh with SASL PLAIN, SCRAM-SHA-256/512 and OAUTHBEARER.

### [To Do] API-0210 SaslHandshake differential test against Kafka 4.4.0
P0 · M4 · Task · deps: API-0208, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical SaslHandshake requests to Kafka 4.4.0 and Alluvium for handshakes for each mechanism at v0 and v1 and for each client-triggerable error (UNSUPPORTED_SASL_MECHANISM and ILLEGAL_SASL_STATE); normalized responses match for every supported version.

### [To Do] API-0211 Advertise SaslHandshake v0-1 in ApiVersions
P0 · M4 · Task · deps: API-0207, API-0209, API-0210, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists SaslHandshake with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks SaslHandshake supported; a test asserts the range. SaslHandshake is listed on every listener, including non-SASL ones, as Kafka does.

## [To Do] API-F18 ApiVersions (v0-5)
P0 · M2 · Feature · deps: PROTO-F07, SRV-F04
Scope: Kafka 4.4.0 versions 0-5; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0212 ApiVersions handler module and router registration (broker and controller listeners)
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleApiVersionsRequest @4.4.0
Done when: `handlers/api_versions.rs` (module doc names the Kafka counterpart) decodes v0-5 with the generated codec and calls the API version registry (PROTO-F07); it is registered for the broker and controller listeners in the router (SRV-F04). ApiVersions is accepted before SASL authentication completes, as Kafka allows. Unit test per listener.

### [To Do] API-0213 Advertised API version list generated per listener
P0 · M2 · Task · deps: API-0212 · ref: clients/src/main/java/org/apache/kafka/common/requests/ApiVersionsResponse.java controllerApiVersions @4.4.0
Done when: The broker and controller lists are generated from PROTO-F07 and contain only APIs whose feature has its advertise task done, each with the listener-specific range; the broker list intersects forwardable APIs with the active controller's ApiVersions (`controllerApiVersions`); a test fails if an API is listed without its feature being complete.

### [To Do] API-0214 Unstable latest versions behind `unstable.api.versions.enable`
P0 · M2 · Task · deps: API-0213 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: APIs whose latest version is marked unstable (InitProducerId v6 in 4.4.0) are advertised without that version unless `unstable.api.versions.enable` is true, and requests at that version are then rejected; tests with the key on and off.

### [To Do] API-0215 Hide GetTelemetrySubscriptions and PushTelemetry without a telemetry exporter
P1 · M8 · Task · deps: API-0213, TELEM-F04 · ref: clients/src/main/java/org/apache/kafka/common/requests/ApiVersionsResponse.java brokerApiVersions @4.4.0
Done when: Keys 71 and 72 are listed only when a client telemetry exporter is configured, as `brokerApiVersions` does; test both cases.

### [To Do] API-0216 ApiVersions v0-v2: UNSUPPORTED_VERSION fallback with the supported range
P0 · M2 · Task · deps: API-0212 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleApiVersionsRequest hasUnsupportedRequestVersion @4.4.0
Done when: A request with a version above 5 is answered with UNSUPPORTED_VERSION and an ApiVersions entry for key 18 using the v0 response layout and v0 response header, so old clients can retry; v1 adds throttle_time_ms; tests with versions 6 and 127.

### [To Do] API-0217 ApiVersions v3: client software name and version validation
P0 · M2 · Task · deps: API-0212 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/requests/ApiVersionsRequest.java isValid @4.4.0
Done when: client_software_name and client_software_version must match `[a-zA-Z0-9](?:[a-zA-Z0-9\-.]*[a-zA-Z0-9])?` or the response is INVALID_REQUEST; valid values are stored in the request context (SRV-F06) for later requests on the connection; tests with valid, empty and illegal strings.

### [To Do] API-0218 ApiVersions v3+: supported and finalized features (tagged fields)
P0 · M2 · Task · deps: API-0212 · ref: server/src/main/java/org/apache/kafka/server/DefaultApiVersionManager.java apiVersionResponse @4.4.0
Done when: v3+ responses carry supported_features from the node's supported feature ranges and finalized_features with finalized_features_epoch from the metadata image (META-F06); zk_migration_ready is always false; test against a metadata image with metadata.version, kraft.version and group.version.

### [To Do] API-0219 ApiVersions v4: omit features with minimum version 0 below v4 (KAFKA-17011)
P0 · M2 · Task · deps: API-0218 · ref: clients/src/main/java/org/apache/kafka/common/requests/ApiVersionsResponse.java maybeFilterSupportedFeatureKeys @4.4.0 (KAFKA-17011)
Done when: For v0-v3 requests, supported features whose minimum version is 0 are omitted because older clients cannot parse them; v4+ lists them; tests per version.

### [To Do] API-0220 ApiVersions v5: cluster id and node id check with REBOOTSTRAP_REQUIRED (KIP-1242)
P0 · M2 · Task · deps: API-0212 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleApiVersionsRequest clusterIdOrNodeIdIsInvalid @4.4.0 (KIP-1242)
Done when: A v5 request with a non-null cluster_id that differs from the cluster's id, or a node_id that differs from this node's id, gets REBOOTSTRAP_REQUIRED; null cluster_id skips the check; tests for each mismatch.

### [To Do] API-0221 ApiVersions error mapping and response-header exception
P0 · M2 · Task · deps: API-0216, API-0217, API-0220 · ref: clients/src/main/resources/common/message/ApiVersionsResponse.json @4.4.0
Done when: UNSUPPORTED_VERSION, INVALID_REQUEST and REBOOTSTRAP_REQUIRED are the only top-level errors; the response header is always v0 (no tagged fields) even for flexible versions, as PROTO-F06 requires; encoding tests.

### [To Do] API-0222 ApiVersions throttle_time_ms through the shared response path
P0 · M2 · Task · deps: API-0212 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is written in v1+ and absent in v0; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0223 ApiVersions throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0222, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: ApiVersions records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v2+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0224 ApiVersions request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0212 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ApiVersions; the request-log summary (SRV-F09) shows client software name and version, request version and the error returned. Connections are counted by client software name and version, as Kafka's `ConnectionCount` metric with clientSoftwareName and clientSoftwareVersion tags does. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0225 ApiVersions handler unit tests across v0-5
P0 · M2 · Task · deps: API-0214, API-0219, API-0221, API-0222 · labels: test
Done when: Table-driven tests send every version v0-v5 through the handler, with the API version registry (PROTO-F07) replaced by a fake, and check the service call and the encoded response; cases cover every listener, unstable gating, telemetry gating, feature filtering, version fallback and REBOOTSTRAP_REQUIRED.

### [To Do] API-0226 ApiVersions compatibility test with Java client 4.4.0, librdkafka, franz-go and kafka-broker-api-versions.sh
P0 · M2 · Task · deps: API-0225, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that negotiates versions on broker and controller listeners and prints the same API table as Kafka for the completed APIs passes with Java client 4.4.0, librdkafka, franz-go and kafka-broker-api-versions.sh.

### [To Do] API-0227 ApiVersions differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0225, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ApiVersions requests to Kafka 4.4.0 and Alluvium for requests at v0-v5 on both listeners (ignoring APIs not yet complete) and for each client-triggerable error (UNSUPPORTED_VERSION fallback, INVALID_REQUEST for bad software names and REBOOTSTRAP_REQUIRED); normalized responses match for every supported version.

### [To Do] API-0228 Advertise ApiVersions v0-5 in its own response
P0 · M2 · Task · deps: API-0224, API-0226, API-0227 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists ApiVersions with min 0 and max 5 through the generated list owned by API-F18; docs/parity.md marks ApiVersions supported; a test asserts the range. Later tasks (M4, M8) add behavior without changing the advertised range.

### [To Do] API-0229 ApiVersions during SASL negotiation is answered without quota throttling
P0 · M4 · Task · deps: API-0223, NET-F08 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java @4.4.0
Done when: ApiVersions requests that arrive before SASL authentication completes are answered by the authenticator path without request-quota recording or channel muting, as `SaslServerAuthenticator` does, so the handshake can proceed; test with a quota-limited SASL user.

## [To Do] API-F19 CreateTopics (v2-7)
P0 · M2 · Feature · deps: CTRL-F04, CTRL-F21
Scope: Kafka 4.4.0 versions 2-7; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0230 CreateTopics handler module and router registration (broker and controller listeners)
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleCreateTopics @4.4.0
Done when: `handlers/create_topics.rs` (module doc names the Kafka counterpart) decodes v2-7 with the generated codec and calls the controller topic-creation path (CTRL-F04); it is registered for the broker and controller listeners in the router (SRV-F04). The controller handler builds the controller request context with the principal and deadline from timeout_ms. Unit test per listener.

### [To Do] API-0231 Forward CreateTopics from brokers to the active controller in an Envelope
P0 · M2 · Task · deps: API-0230, API-F58 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener CreateTopics is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0232 CreateTopics request validation: duplicates and reserved names
P0 · M2 · Task · deps: API-0230 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala createTopics @4.4.0
Done when: Names listed more than once get INVALID_REQUEST 'Duplicate topic name.' and are not created; `__cluster_metadata` gets INVALID_REQUEST with Kafka's message; other per-topic validation (names, partitions, replication factor, assignments, configs) is delegated to CTRL-F04 and returned per topic; unit tests.

### [To Do] API-0233 CreateTopics v2-v4: validate_only, error messages and defaulted partitions (KIP-464)
P0 · M2 · Task · deps: API-0232 · ref: clients/src/main/resources/common/message/CreateTopicsRequest.json @4.4.0 (KIP-464)
Done when: validate_only runs all checks without writing records; per-topic error_message is returned; num_partitions or replication_factor of -1 takes `num.partitions` or `default.replication.factor` at every version, because Kafka restricts -1 below v4 only in the client builder; per-version tests.

### [To Do] API-0234 CreateTopics v5: return configs, partitions and replication factor (KIP-525)
P0 · M2 · Task · deps: API-0230 · ref: clients/src/main/resources/common/message/CreateTopicsResponse.json @4.4.0 (KIP-525)
Done when: v5+ results include num_partitions, replication_factor and the topic's effective configs (name, value, read_only, config_source, is_sensitive with sensitive values null) and topic_config_error_code when the configs cannot be described; below v5 these fields are absent; tests per version.

### [To Do] API-0235 CreateTopics v7: return the new topic id
P0 · M2 · Task · deps: API-0230 · ref: clients/src/main/resources/common/message/CreateTopicsResponse.json @4.4.0
Done when: v7 results carry the topic id of a successfully created topic (zero for failures and validate_only); test.

### [To Do] API-0236 CreateTopics error mapping per version
P0 · M2 · Task · deps: API-0230 · ref: core/src/main/scala/kafka/server/ControllerApis.scala createTopics @4.4.0
Done when: Per-topic errors TOPIC_ALREADY_EXISTS, INVALID_TOPIC_EXCEPTION, INVALID_PARTITIONS, INVALID_REPLICATION_FACTOR, INVALID_REPLICA_ASSIGNMENT, INVALID_CONFIG, POLICY_VIOLATION, REQUEST_TIMED_OUT and NOT_CONTROLLER come from one exhaustive mapping with messages; THROTTLING_QUOTA_EXCEEDED is only produced for v6+; table test.

### [To Do] API-0237 CreateTopics throttle_time_ms through the shared response path
P0 · M2 · Task · deps: API-0230 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0238 CreateTopics throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0237, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: CreateTopics records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v3+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0239 CreateTopics request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0230 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for CreateTopics; the request-log summary (SRV-F09) shows topic count, validate_only, timeout_ms and per-topic outcome. Each created topic emits a controller event with reason `create_topics_request` (CTRL-F04). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0240 CreateTopics handler unit tests across v2-7
P0 · M2 · Task · deps: API-0231, API-0233, API-0234, API-0235, API-0236, API-0237 · labels: test
Done when: Table-driven tests send every version v2-v7 through the handler, with the controller topic-creation path (CTRL-F04) replaced by a fake, and check the service call and the encoded response; cases cover duplicates, reserved names, validate_only, defaults at v4+, configs at v5+, topic id at v7 and forwarded requests.

### [To Do] API-0241 CreateTopics compatibility test with Java client 4.4.0 Admin.createTopics, librdkafka, franz-go and kafka-topics.sh --create
P0 · M2 · Task · deps: API-0240, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that creates topics with defaults, explicit assignments, configs and --if-not-exists through a broker and directly against the controller (bootstrap-controller) passes with Java client 4.4.0 Admin.createTopics, librdkafka, franz-go and kafka-topics.sh --create.

### [To Do] API-0242 CreateTopics differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0240, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical CreateTopics requests to Kafka 4.4.0 and Alluvium for creation and validate_only and for each client-triggerable error (duplicate names, `__cluster_metadata`, TOPIC_ALREADY_EXISTS, INVALID_PARTITIONS, INVALID_REPLICATION_FACTOR, INVALID_REPLICA_ASSIGNMENT, INVALID_CONFIG and -1 defaults below v4); normalized responses match for every supported version.

### [To Do] API-0243 Advertise CreateTopics v2-7 in ApiVersions
P0 · M2 · Task · deps: API-0239, API-0241, API-0242, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists CreateTopics with min 2 and max 7 through the generated list owned by API-F18; docs/parity.md marks CreateTopics supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0244 CreateTopics authorization: CREATE and DESCRIBE_CONFIGS
P0 · M4 · Task · deps: API-0231, API-0234, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleCreateTopics @4.4.0
Done when: Without CREATE on CLUSTER, topics need CREATE on TOPIC or get TOPIC_AUTHORIZATION_FAILED 'Authorization failed.'; v5+ configs are returned only with DESCRIBE_CONFIGS on the topic, otherwise topic_config_error_code is TOPIC_AUTHORIZATION_FAILED; forwarded requests are authorized with the envelope principal; tests and differential cases.

### [To Do] API-0245 CreateTopics v6: controller mutation quota and THROTTLING_QUOTA_EXCEEDED (KIP-599)
P0 · M4 · Task · deps: API-0238, QUOTA-F05 · ref: clients/src/main/resources/common/message/CreateTopicsRequest.json @4.4.0 (KIP-599)
Done when: Partition creations are charged to the controller mutation quota; when exceeded, v6+ topics fail with THROTTLING_QUOTA_EXCEEDED and throttle_time_ms set, while v2-v5 requests are accepted and only throttled (permissive quota); tests per version.

## [To Do] API-F20 DeleteTopics (v1-6)
P0 · M2 · Feature · deps: CTRL-F05, CTRL-F21
Scope: Kafka 4.4.0 versions 1-6; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0246 DeleteTopics handler module and router registration (broker and controller listeners)
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleDeleteTopics @4.4.0
Done when: `handlers/delete_topics.rs` (module doc names the Kafka counterpart) decodes v1-6 with the generated codec and calls the controller topic-deletion path (CTRL-F05); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0247 Forward DeleteTopics from brokers to the active controller in an Envelope
P0 · M2 · Task · deps: API-0246, API-F58 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener DeleteTopics is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0248 DeleteTopics validation: duplicates, disabled deletion and unknown topics
P0 · M2 · Task · deps: API-0246 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala deleteTopics @4.4.0
Done when: Duplicate names or ids get INVALID_REQUEST 'Duplicate topic name.' or 'Duplicate topic id.'; with `delete.topic.enable=false` v1-v2 fail with INVALID_REQUEST and v3+ with TOPIC_DELETION_DISABLED; unknown names get UNKNOWN_TOPIC_OR_PARTITION; unit tests.

### [To Do] API-0249 DeleteTopics v1-v5: topic names and v5 error_message
P0 · M2 · Task · deps: API-0248 · ref: clients/src/main/resources/common/message/DeleteTopicsRequest.json @4.4.0
Done when: v1-v5 list topic_names; v5+ results carry error_message; per-version tests.

### [To Do] API-0250 DeleteTopics v6: delete by topic id with null names
P0 · M2 · Task · deps: API-0248 · ref: clients/src/main/resources/common/message/DeleteTopicsRequest.json @4.4.0 (KIP-516)
Done when: v6 topics carry a name or an id; an unknown id gets UNKNOWN_TOPIC_ID with a null name, an id and name pair is resolved by id; results return both name and id; tests.

### [To Do] API-0251 DeleteTopics error mapping per version
P0 · M2 · Task · deps: API-0246 · ref: core/src/main/scala/kafka/server/ControllerApis.scala deleteTopics @4.4.0
Done when: UNKNOWN_TOPIC_OR_PARTITION, UNKNOWN_TOPIC_ID, TOPIC_DELETION_DISABLED, INVALID_REQUEST, REQUEST_TIMED_OUT and NOT_CONTROLLER map exhaustively; THROTTLING_QUOTA_EXCEEDED only for v5+; table test.

### [To Do] API-0252 DeleteTopics throttle_time_ms through the shared response path
P0 · M2 · Task · deps: API-0246 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0253 DeleteTopics throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0252, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: DeleteTopics records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v2+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0254 DeleteTopics request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0246 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DeleteTopics; the request-log summary (SRV-F09) shows topic count, by-name or by-id, timeout_ms and per-topic outcome. Each deletion emits a controller event with reason `delete_topics_request` (CTRL-F05). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0255 DeleteTopics handler unit tests across v1-6
P0 · M2 · Task · deps: API-0247, API-0249, API-0250, API-0251, API-0252 · labels: test
Done when: Table-driven tests send every version v1-v6 through the handler, with the controller topic-deletion path (CTRL-F05) replaced by a fake, and check the service call and the encoded response; cases cover duplicates, disabled deletion per version, names and ids, unknown topics and forwarding.

### [To Do] API-0256 DeleteTopics compatibility test with Java client 4.4.0 Admin.deleteTopics (by name and id), librdkafka, franz-go and kafka-topics.sh --delete
P0 · M2 · Task · deps: API-0255, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that deletes topics and verifies they disappear from Metadata passes with Java client 4.4.0 Admin.deleteTopics (by name and id), librdkafka, franz-go and kafka-topics.sh --delete.

### [To Do] API-0257 DeleteTopics differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0255, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DeleteTopics requests to Kafka 4.4.0 and Alluvium for deletion by name and by id and for each client-triggerable error (UNKNOWN_TOPIC_OR_PARTITION, UNKNOWN_TOPIC_ID, duplicate names, TOPIC_DELETION_DISABLED and INVALID_REQUEST at v1-v2 with deletion disabled); normalized responses match for every supported version.

### [To Do] API-0258 Advertise DeleteTopics v1-6 in ApiVersions
P0 · M2 · Task · deps: API-0254, API-0256, API-0257, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists DeleteTopics with min 1 and max 6 through the generated list owned by API-F18; docs/parity.md marks DeleteTopics supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0259 DeleteTopics authorization: DELETE with DESCRIBE-based disclosure
P0 · M4 · Task · deps: API-0247, API-0250, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala deleteTopics @4.4.0
Done when: Without DELETE on CLUSTER each topic needs DELETE on TOPIC: undescribable topics are reported as TOPIC_AUTHORIZATION_FAILED, and by-id requests for topics the principal cannot describe return UNKNOWN_TOPIC_ID so existence is not leaked; tests and differential cases.

### [To Do] API-0260 DeleteTopics v5: controller mutation quota and THROTTLING_QUOTA_EXCEEDED (KIP-599)
P0 · M4 · Task · deps: API-0253, QUOTA-F05 · ref: clients/src/main/resources/common/message/DeleteTopicsRequest.json @4.4.0 (KIP-599)
Done when: Deleted partitions are charged to the controller mutation quota; v5+ topics over quota fail with THROTTLING_QUOTA_EXCEEDED while v1-v4 are only throttled; tests per version.

## [To Do] API-F21 DeleteRecords (v0-2)
P1 · M7 · Feature · deps: STOR-F13, REPL-F07
Scope: Kafka 4.4.0 versions 0-2; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0261 DeleteRecords handler module and router registration (broker listener)
P1 · M7 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteRecordsRequest @4.4.0
Done when: `handlers/delete_records.rs` (module doc names the Kafka counterpart) decodes v0-2 with the generated codec and calls the log-start-offset service (STOR-F13); it is registered for the broker listener in the router (SRV-F04). A DeleteRecords request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0262 DeleteRecords v0: advance the log start offset
P1 · M7 · Task · deps: API-0261 · ref: clients/src/main/resources/common/message/DeleteRecordsRequest.json @4.4.0
Done when: Each partition's offset (or -1 for the high watermark) advances the log start offset on the leader and the response waits in a delayed operation (SRV-F07) until every replica's low watermark reaches it or timeout_ms expires (REQUEST_TIMED_OUT); an offset above the high watermark gets OFFSET_OUT_OF_RANGE; tests.

### [To Do] API-0263 DeleteRecords v1-v2: client-side throttling and flexible encoding
P1 · M7 · Task · deps: API-0262 · ref: clients/src/main/resources/common/message/DeleteRecordsResponse.json @4.4.0
Done when: v1 clients throttle themselves and v2 accepts tagged fields with otherwise identical semantics; per-version round-trip tests.

### [To Do] API-0264 DeleteRecords error mapping
P1 · M7 · Task · deps: API-0261 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteRecordsRequest @4.4.0
Done when: UNKNOWN_TOPIC_OR_PARTITION, NOT_LEADER_OR_FOLLOWER, OFFSET_OUT_OF_RANGE, POLICY_VIOLATION (compacted-only topics), REQUEST_TIMED_OUT, KAFKA_STORAGE_ERROR and TOPIC_AUTHORIZATION_FAILED map exhaustively with low_watermark -1; table test.

### [To Do] API-0265 DeleteRecords throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M7 · Task · deps: API-0261, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; DeleteRecords records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v1+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0266 DeleteRecords request metrics, events and request-log summary
P1 · M7 · Task · deps: API-0261 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DeleteRecords; the request-log summary (SRV-F09) shows topic and partition counts and timeout_ms. Log-start advances carry reason `delete_records_request` (STOR-F13). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0267 DeleteRecords authorization: DELETE on each topic
P1 · M7 · Task · deps: API-0261, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteRecordsRequest @4.4.0
Done when: Topics without DELETE on TOPIC get TOPIC_AUTHORIZATION_FAILED for their partitions while the rest proceed; tests and a differential case.

### [To Do] API-0268 DeleteRecords handler unit tests across v0-2
P1 · M7 · Task · deps: API-0263, API-0264, API-0265, API-0267 · labels: test
Done when: Table-driven tests send every version v0-v2 through the handler, with the log-start-offset service (STOR-F13) replaced by a fake, and check the service call and the encoded response; cases cover specific offsets, -1, offsets past the high watermark, unknown partitions and timeouts.

### [To Do] API-0269 DeleteRecords compatibility test with Java client 4.4.0 Admin.deleteRecords and kafka-delete-records.sh
P1 · M7 · Task · deps: API-0268, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that truncates the head of a topic and consumers start at the new log start offset passes with Java client 4.4.0 Admin.deleteRecords and kafka-delete-records.sh.

### [To Do] API-0270 DeleteRecords differential test against Kafka 4.4.0
P1 · M7 · Task · deps: API-0268, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DeleteRecords requests to Kafka 4.4.0 and Alluvium for deletions at v0-v2 and for each client-triggerable error (OFFSET_OUT_OF_RANGE, UNKNOWN_TOPIC_OR_PARTITION and POLICY_VIOLATION); normalized responses match for every supported version.

### [To Do] API-0271 Advertise DeleteRecords v0-2 in ApiVersions
P1 · M7 · Task · deps: API-0266, API-0269, API-0270, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists DeleteRecords with min 0 and max 2 through the generated list owned by API-F18; docs/parity.md marks DeleteRecords supported; a test asserts the range.

## [To Do] API-F22 InitProducerId (v0-6 (latest unstable))
P0 · M6 · Feature · deps: TXN-F02, TXN-F05
Scope: Kafka 4.4.0 versions 0-6 (latest unstable); listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0272 InitProducerId handler module and router registration (broker listener)
P0 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleInitProducerIdRequest @4.4.0
Done when: `handlers/init_producer_id.rs` (module doc names the Kafka counterpart) decodes v0-6 with the generated codec and calls the transaction coordinator (TXN-F02, TXN-F05); it is registered for the broker listener in the router (SRV-F04). A InitProducerId request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0273 InitProducerId v0-v2: idempotent and transactional producer ids
P0 · M6 · Task · deps: API-0272 · ref: clients/src/main/resources/common/message/InitProducerIdRequest.json @4.4.0
Done when: A null transactional_id gets a fresh producer id and epoch from TXN-F02; a transactional id is initialized, fenced or recovered by TXN-F05 with transaction_timeout_ms validated (INVALID_TRANSACTION_TIMEOUT); v2 accepts tagged fields; per-version tests.

### [To Do] API-0274 InitProducerId v3: resume with producer_id and producer_epoch (KIP-360)
P0 · M6 · Task · deps: API-0273 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleInitProducerIdRequest @4.4.0 (KIP-360)
Done when: v3+ requests may carry the current producer id and epoch to bump the epoch after INVALID_PRODUCER_EPOCH; exactly one of them set to -1 gets INVALID_REQUEST before the coordinator is called; tests for each combination.

### [To Do] API-0275 InitProducerId v4-v5: PRODUCER_FENCED and TRANSACTION_ABORTABLE
P0 · M6 · Task · deps: API-0274 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleInitProducerIdRequest @4.4.0 (KIP-588, KIP-890)
Done when: PRODUCER_FENCED is returned from v4 and mapped to INVALID_PRODUCER_EPOCH for v0-v3; TRANSACTION_ABORTABLE is never returned below v5 (KIP-890); tests per version.

### [To Do] API-0276 InitProducerId v6: two-phase commit fields (KIP-939), behind the unstable flag
P0 · M6 · Task · deps: API-0275, TXN-F14 · ref: clients/src/main/resources/common/message/InitProducerIdRequest.json @4.4.0 (KIP-939)
Done when: v6 enable_2pc and keep_prepared_txn are passed to TXN-F14 and the response returns ongoing_txn_producer_id and ongoing_txn_producer_epoch; because v6 is the unstable latest version in 4.4.0, requests at v6 are rejected unless `unstable.api.versions.enable` is true; tests with the flag on and off.

### [To Do] API-0277 InitProducerId error mapping per version
P0 · M6 · Task · deps: API-0272 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleInitProducerIdRequest @4.4.0
Done when: NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, CONCURRENT_TRANSACTIONS, INVALID_TRANSACTION_TIMEOUT, INVALID_REQUEST, PRODUCER_FENCED (v4+), INVALID_PRODUCER_EPOCH, TRANSACTION_ABORTABLE (v5+), TRANSACTIONAL_ID_AUTHORIZATION_FAILED and CLUSTER_AUTHORIZATION_FAILED map exhaustively; table test.

### [To Do] API-0278 InitProducerId throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M6 · Task · deps: API-0272, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; InitProducerId records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v1+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0279 InitProducerId request metrics, events and request-log summary
P0 · M6 · Task · deps: API-0272 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for InitProducerId; the request-log summary (SRV-F09) shows transactional id presence, transaction timeout, resume fields and 2PC flags. Producer id allocation and fencing are emitted by TXN-F05 with this request as the reason source. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0280 InitProducerId authorization: WRITE or TWO_PHASE_COMMIT on TRANSACTIONAL_ID, IDEMPOTENT_WRITE otherwise
P0 · M6 · Task · deps: API-0276, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleInitProducerIdRequest @4.4.0
Done when: Transactional requests need WRITE on TRANSACTIONAL_ID, plus TWO_PHASE_COMMIT when enable_2pc is set, else TRANSACTIONAL_ID_AUTHORIZATION_FAILED; idempotent requests need IDEMPOTENT_WRITE on CLUSTER or WRITE on any TOPIC (`authorizeByResourceType`), else CLUSTER_AUTHORIZATION_FAILED; tests and differential cases.

### [To Do] API-0281 InitProducerId handler unit tests across v0-6
P0 · M6 · Task · deps: API-0277, API-0278, API-0280 · labels: test
Done when: Table-driven tests send every version v0-v6 through the handler, with the transaction coordinator (TXN-F02, TXN-F05) replaced by a fake, and check the service call and the encoded response; cases cover idempotent and transactional producers, epoch bumps, fencing, timeouts, v3 resume combinations and v6 gating.

### [To Do] API-0282 InitProducerId compatibility test with Java client 4.4.0 (default idempotent producer and transactional producer), librdkafka and franz-go
P0 · M6 · Task · deps: API-0281, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that initializes producers, recovers from a fenced epoch and restarts a transactional producer passes with Java client 4.4.0 (default idempotent producer and transactional producer), librdkafka and franz-go.

### [To Do] API-0283 InitProducerId differential test against Kafka 4.4.0
P0 · M6 · Task · deps: API-0281, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical InitProducerId requests to Kafka 4.4.0 and Alluvium for idempotent and transactional initialization at v0-v5 and for each client-triggerable error (INVALID_REQUEST for half-set resume fields, INVALID_TRANSACTION_TIMEOUT, PRODUCER_FENCED versus INVALID_PRODUCER_EPOCH by version and CONCURRENT_TRANSACTIONS); normalized responses match for every supported version.

### [To Do] API-0284 Advertise InitProducerId v0-5, and v6 only with unstable API versions enabled
P0 · M6 · Task · deps: API-0279, API-0282, API-0283, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists InitProducerId with min 0 and max 5 through the generated list owned by API-F18; docs/parity.md marks InitProducerId supported; a test asserts the range. v6 is the unstable latest version in 4.4.0, so max 6 is advertised only when `unstable.api.versions.enable` is true (the API-F18 unstable-version task); a test covers both settings.

## [To Do] API-F23 OffsetForLeaderEpoch (v2-4)
P0 · M3 · Feature · deps: STOR-F11
Scope: Kafka 4.4.0 versions 2-4; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0285 OffsetForLeaderEpoch handler module and router registration (broker listener)
P0 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleOffsetForLeaderEpochRequest @4.4.0
Done when: `handlers/offset_for_leader_epoch.rs` (module doc names the Kafka counterpart) decodes v2-4 with the generated codec and calls the leader-epoch lookup service (STOR-F11); it is registered for the broker listener in the router (SRV-F04). A OffsetForLeaderEpoch request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0286 OffsetForLeaderEpoch v2: end offset per requested epoch with current_leader_epoch fencing
P0 · M3 · Task · deps: API-0285 · ref: clients/src/main/resources/common/message/OffsetForLeaderEpochRequest.json @4.4.0 (KIP-320)
Done when: For each partition the handler returns the end offset and leader epoch for the requested leader_epoch from STOR-F11; a stale current_leader_epoch yields FENCED_LEADER_EPOCH and a newer one UNKNOWN_LEADER_EPOCH, -1 skips the check; non-leaders return NOT_LEADER_OR_FOLLOWER; tests per case.

### [To Do] API-0287 OffsetForLeaderEpoch v3-v4: replica_id and flexible encoding
P0 · M3 · Task · deps: API-0286 · ref: clients/src/main/resources/common/message/OffsetForLeaderEpochRequest.json @4.4.0
Done when: v3+ replica_id (default -2, the debug consumer) is decoded and carried into the lookup and the request log so followers and consumers are distinguished; v4 accepts tagged fields; per-version tests.

### [To Do] API-0288 OffsetForLeaderEpoch error mapping
P0 · M3 · Task · deps: API-0285 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleOffsetForLeaderEpochRequest @4.4.0
Done when: Partition errors UNKNOWN_TOPIC_OR_PARTITION, NOT_LEADER_OR_FOLLOWER, FENCED_LEADER_EPOCH, UNKNOWN_LEADER_EPOCH and KAFKA_STORAGE_ERROR map exhaustively with end_offset and leader_epoch -1; table test.

### [To Do] API-0289 OffsetForLeaderEpoch throttle_time_ms through the shared response path
P0 · M3 · Task · deps: API-0285 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0290 OffsetForLeaderEpoch throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0289, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: OffsetForLeaderEpoch records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0291 OffsetForLeaderEpoch request metrics, events and request-log summary
P0 · M3 · Task · deps: API-0285 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for OffsetForLeaderEpoch; the request-log summary (SRV-F09) shows replica id, partition count and the range of requested epochs. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0292 OffsetForLeaderEpoch handler unit tests across v2-4
P0 · M3 · Task · deps: API-0287, API-0288, API-0289 · labels: test
Done when: Table-driven tests send every version v2-v4 through the handler, with the leader-epoch lookup service (STOR-F11) replaced by a fake, and check the service call and the encoded response; cases cover known and unknown epochs, epochs past the latest, fencing and non-leader partitions.

### [To Do] API-0293 OffsetForLeaderEpoch compatibility test with Java client 4.4.0 and librdkafka consumers
P0 · M3 · Task · deps: API-0292, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that detects log truncation after an unclean leader election (KIP-320 position validation) and resets as configured passes with Java client 4.4.0 and librdkafka consumers.

### [To Do] API-0294 OffsetForLeaderEpoch multi-node simulation test
P0 · M3 · Task · deps: API-0292, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) forces leader changes with truncated followers and checks consumers validating positions with OffsetForLeaderEpoch never skip acknowledged records; a failing seed is printed and replays exactly.

### [To Do] API-0295 OffsetForLeaderEpoch differential test against Kafka 4.4.0
P0 · M3 · Task · deps: API-0292, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical OffsetForLeaderEpoch requests to Kafka 4.4.0 and Alluvium for lookups for current, older and future epochs and for each client-triggerable error (FENCED_LEADER_EPOCH, UNKNOWN_LEADER_EPOCH, NOT_LEADER_OR_FOLLOWER and UNKNOWN_TOPIC_OR_PARTITION); normalized responses match for every supported version.

### [To Do] API-0296 Advertise OffsetForLeaderEpoch v2-4 in ApiVersions
P0 · M3 · Task · deps: API-0291, API-0293, API-0294, API-0295, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists OffsetForLeaderEpoch with min 2 and max 4 through the generated list owned by API-F18; docs/parity.md marks OffsetForLeaderEpoch supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0297 OffsetForLeaderEpoch authorization: CLUSTER_ACTION or DESCRIBE per topic
P0 · M4 · Task · deps: API-0285, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleOffsetForLeaderEpochRequest @4.4.0
Done when: With CLUSTER_ACTION on CLUSTER every topic is allowed; otherwise topics need DESCRIBE on TOPIC or get TOPIC_AUTHORIZATION_FAILED per partition; tests and differential cases.

## [To Do] API-F24 AddPartitionsToTxn (v0-5)
P0 · M6 · Feature · deps: TXN-F06
Scope: Kafka 4.4.0 versions 0-5; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0298 AddPartitionsToTxn handler module and router registration (broker listener)
P0 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAddPartitionsToTxnRequest @4.4.0
Done when: `handlers/add_partitions_to_txn.rs` (module doc names the Kafka counterpart) decodes v0-5 with the generated codec and calls the transaction coordinator (TXN-F06); it is registered for the broker listener in the router (SRV-F04). A AddPartitionsToTxn request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0299 AddPartitionsToTxn v0-v3: single transaction from clients
P0 · M6 · Task · deps: API-0298 · ref: clients/src/main/java/org/apache/kafka/common/requests/AddPartitionsToTxnRequest.java normalizeRequest @4.4.0
Done when: v0-v3 requests are normalized into one transaction; unknown partitions get UNKNOWN_TOPIC_OR_PARTITION and the rest OPERATION_NOT_ATTEMPTED without calling the coordinator; results use results_by_topic_v3_and_below; v3 accepts tagged fields; tests.

### [To Do] API-0300 AddPartitionsToTxn v4: batched broker verification with verify_only (KIP-890)
P0 · M6 · Task · deps: API-0299 · ref: clients/src/main/resources/common/message/AddPartitionsToTxnRequest.json @4.4.0 (KIP-890)
Done when: v4+ requests come from brokers and carry several transactions, each with verify_only; a null transactional_id gets INVALID_REQUEST; results are per transaction with a top-level error_code; tests with mixed transactions.

### [To Do] API-0301 AddPartitionsToTxn v2 and v5: PRODUCER_FENCED and TRANSACTION_ABORTABLE
P0 · M6 · Task · deps: API-0300 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAddPartitionsToTxnRequest @4.4.0 (KIP-588, KIP-890)
Done when: PRODUCER_FENCED is mapped to INVALID_PRODUCER_EPOCH for v0-v1 and TRANSACTION_ABORTABLE is never returned below v5; tests per version.

### [To Do] API-0302 AddPartitionsToTxn error mapping per version
P0 · M6 · Task · deps: API-0298 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAddPartitionsToTxnRequest @4.4.0
Done when: NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, CONCURRENT_TRANSACTIONS, INVALID_PRODUCER_ID_MAPPING, INVALID_PRODUCER_EPOCH, PRODUCER_FENCED, INVALID_TXN_STATE, OPERATION_NOT_ATTEMPTED, UNKNOWN_TOPIC_OR_PARTITION and the authorization errors map exhaustively per partition or transaction; table test.

### [To Do] API-0303 AddPartitionsToTxn throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M6 · Task · deps: API-0298, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; AddPartitionsToTxn records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v1+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0304 AddPartitionsToTxn request metrics, events and request-log summary
P0 · M6 · Task · deps: API-0298 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for AddPartitionsToTxn; the request-log summary (SRV-F09) shows transaction count, partition count, verify_only and request origin (client v0-v3 or broker v4+). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0305 AddPartitionsToTxn authorization: client and broker paths
P0 · M6 · Task · deps: API-0300, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAddPartitionsToTxnRequest @4.4.0
Done when: v0-v3 need WRITE on TRANSACTIONAL_ID (else TRANSACTIONAL_ID_AUTHORIZATION_FAILED for that transaction) and WRITE on each non-internal TOPIC (TOPIC_AUTHORIZATION_FAILED, others OPERATION_NOT_ATTEMPTED); v4+ need CLUSTER_ACTION on CLUSTER for the whole request; tests and differential cases.

### [To Do] API-0306 AddPartitionsToTxn handler unit tests across v0-5
P0 · M6 · Task · deps: API-0301, API-0302, API-0303, API-0305 · labels: test
Done when: Table-driven tests send every version v0-v5 through the handler, with the transaction coordinator (TXN-F06) replaced by a fake, and check the service call and the encoded response; cases cover client and broker shapes, unknown partitions, verify_only, fencing and per-version error downgrades.

### [To Do] API-0307 AddPartitionsToTxn compatibility test with Java client 4.4.0 and librdkafka transactional producers (transactions v1)
P0 · M6 · Task · deps: API-0306, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that add partitions across several topics in one transaction and commit passes with Java client 4.4.0 and librdkafka transactional producers (transactions v1).

### [To Do] API-0308 AddPartitionsToTxn multi-node simulation test
P0 · M6 · Task · deps: API-0306, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) runs transactional producers against 3 brokers with broker-side verification (v4) under coordinator moves and checks no record is appended to a partition outside its transaction; a failing seed is printed and replays exactly.

### [To Do] API-0309 AddPartitionsToTxn differential test against Kafka 4.4.0
P0 · M6 · Task · deps: API-0306, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical AddPartitionsToTxn requests to Kafka 4.4.0 and Alluvium for client v0-v3 and broker v4-v5 requests and for each client-triggerable error (OPERATION_NOT_ATTEMPTED, UNKNOWN_TOPIC_OR_PARTITION, INVALID_PRODUCER_EPOCH and CONCURRENT_TRANSACTIONS); normalized responses match for every supported version.

### [To Do] API-0310 Advertise AddPartitionsToTxn v0-5 in ApiVersions
P0 · M6 · Task · deps: API-0304, API-0307, API-0308, API-0309, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists AddPartitionsToTxn with min 0 and max 5 through the generated list owned by API-F18; docs/parity.md marks AddPartitionsToTxn supported; a test asserts the range.

## [To Do] API-F25 AddOffsetsToTxn (v0-4)
P0 · M6 · Feature · deps: TXN-F07
Scope: Kafka 4.4.0 versions 0-4; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0311 AddOffsetsToTxn handler module and router registration (broker listener)
P0 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAddOffsetsToTxnRequest @4.4.0
Done when: `handlers/add_offsets_to_txn.rs` (module doc names the Kafka counterpart) decodes v0-4 with the generated codec and calls the transaction coordinator (TXN-F07); it is registered for the broker listener in the router (SRV-F04). A AddOffsetsToTxn request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0312 AddOffsetsToTxn v0-v3: add the group's offsets partition to the transaction
P0 · M6 · Task · deps: API-0311 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAddOffsetsToTxnRequest @4.4.0
Done when: The group's `__consumer_offsets` partition is added to the transaction through TXN-F07 (always as transactions v1); v2+ may return PRODUCER_FENCED, mapped to INVALID_PRODUCER_EPOCH for v0-v1; v3 accepts tagged fields; tests.

### [To Do] API-0313 AddOffsetsToTxn v4: TRANSACTION_ABORTABLE (KIP-890)
P0 · M6 · Task · deps: API-0312 · ref: clients/src/main/resources/common/message/AddOffsetsToTxnResponse.json @4.4.0 (KIP-890)
Done when: TRANSACTION_ABORTABLE is returned only to v4+; tests at v3 and v4.

### [To Do] API-0314 AddOffsetsToTxn error mapping per version
P0 · M6 · Task · deps: API-0311 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAddOffsetsToTxnRequest @4.4.0
Done when: NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, CONCURRENT_TRANSACTIONS, INVALID_PRODUCER_ID_MAPPING, INVALID_PRODUCER_EPOCH, PRODUCER_FENCED, INVALID_TXN_STATE and the authorization errors map exhaustively; table test.

### [To Do] API-0315 AddOffsetsToTxn throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M6 · Task · deps: API-0311, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; AddOffsetsToTxn records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v1+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0316 AddOffsetsToTxn request metrics, events and request-log summary
P0 · M6 · Task · deps: API-0311 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for AddOffsetsToTxn; the request-log summary (SRV-F09) shows transactional id, group id and producer id and epoch. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0317 AddOffsetsToTxn authorization: WRITE on TRANSACTIONAL_ID and READ on GROUP
P0 · M6 · Task · deps: API-0311, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAddOffsetsToTxnRequest @4.4.0
Done when: Without WRITE on TRANSACTIONAL_ID the response is TRANSACTIONAL_ID_AUTHORIZATION_FAILED, then without READ on the GROUP it is GROUP_AUTHORIZATION_FAILED; tests and differential cases.

### [To Do] API-0318 AddOffsetsToTxn handler unit tests across v0-4
P0 · M6 · Task · deps: API-0313, API-0314, API-0315, API-0317 · labels: test
Done when: Table-driven tests send every version v0-v4 through the handler, with the transaction coordinator (TXN-F07) replaced by a fake, and check the service call and the encoded response; cases cover first add, repeated add, fenced producers and wrong epochs.

### [To Do] API-0319 AddOffsetsToTxn compatibility test with Java client 4.4.0 transactional producer sendOffsetsToTransaction and librdkafka
P0 · M6 · Task · deps: API-0318, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that consume-transform-produce loop commits offsets inside transactions passes with Java client 4.4.0 transactional producer sendOffsetsToTransaction and librdkafka.

### [To Do] API-0320 AddOffsetsToTxn differential test against Kafka 4.4.0
P0 · M6 · Task · deps: API-0318, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical AddOffsetsToTxn requests to Kafka 4.4.0 and Alluvium for adds at v0-v4 and for each client-triggerable error (INVALID_PRODUCER_EPOCH, PRODUCER_FENCED and CONCURRENT_TRANSACTIONS); normalized responses match for every supported version.

### [To Do] API-0321 Advertise AddOffsetsToTxn v0-4 in ApiVersions
P0 · M6 · Task · deps: API-0316, API-0319, API-0320, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists AddOffsetsToTxn with min 0 and max 4 through the generated list owned by API-F18; docs/parity.md marks AddOffsetsToTxn supported; a test asserts the range.

## [To Do] API-F26 EndTxn (v0-5)
P0 · M6 · Feature · deps: TXN-F08
Scope: Kafka 4.4.0 versions 0-5; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0322 EndTxn handler module and router registration (broker listener)
P0 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleEndTxnRequest @4.4.0
Done when: `handlers/end_txn.rs` (module doc names the Kafka counterpart) decodes v0-5 with the generated codec and calls the transaction coordinator (TXN-F08); it is registered for the broker listener in the router (SRV-F04). A EndTxn request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0323 EndTxn v0-v3: commit or abort
P0 · M6 · Task · deps: API-0322 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleEndTxnRequest @4.4.0
Done when: committed selects COMMIT or ABORT through TXN-F08, which writes markers; v2+ may return PRODUCER_FENCED, mapped to INVALID_PRODUCER_EPOCH for v0-v1; v3 accepts tagged fields; tests.

### [To Do] API-0324 EndTxn v4: TRANSACTION_ABORTABLE (KIP-890)
P0 · M6 · Task · deps: API-0323 · ref: clients/src/main/resources/common/message/EndTxnResponse.json @4.4.0 (KIP-890)
Done when: TRANSACTION_ABORTABLE is returned only to v4+; tests at v3 and v4.

### [To Do] API-0325 EndTxn v5: epoch bump on every transaction (KIP-890 part 2)
P0 · M6 · Task · deps: API-0324, TXN-F10 · ref: clients/src/main/resources/common/message/EndTxnResponse.json @4.4.0 (KIP-890)
Done when: v5 selects transactions v2 (`TransactionVersion.transactionVersionForEndTxn`) and the response returns the new producer_id and producer_epoch; v0-v4 always use transactions v1 and return -1; tests per version.

### [To Do] API-0326 EndTxn error mapping per version
P0 · M6 · Task · deps: API-0322 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleEndTxnRequest @4.4.0
Done when: NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, CONCURRENT_TRANSACTIONS, INVALID_PRODUCER_ID_MAPPING, INVALID_PRODUCER_EPOCH, PRODUCER_FENCED, INVALID_TXN_STATE, TRANSACTION_ABORTABLE (v4+) and TRANSACTIONAL_ID_AUTHORIZATION_FAILED map exhaustively; table test.

### [To Do] API-0327 EndTxn throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M6 · Task · deps: API-0322, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; EndTxn records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v1+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0328 EndTxn request metrics, events and request-log summary
P0 · M6 · Task · deps: API-0322 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for EndTxn; the request-log summary (SRV-F09) shows transactional id, producer id and epoch, commit or abort and transaction version. Transaction completion is emitted by TXN-F08 with reason `end_txn_request`. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0329 EndTxn authorization: WRITE on TRANSACTIONAL_ID
P0 · M6 · Task · deps: API-0322, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleEndTxnRequest @4.4.0
Done when: Requests without WRITE on TRANSACTIONAL_ID get TRANSACTIONAL_ID_AUTHORIZATION_FAILED; tests and a differential case.

### [To Do] API-0330 EndTxn handler unit tests across v0-5
P0 · M6 · Task · deps: API-0325, API-0326, API-0327, API-0329 · labels: test
Done when: Table-driven tests send every version v0-v5 through the handler, with the transaction coordinator (TXN-F08) replaced by a fake, and check the service call and the encoded response; cases cover commit, abort, retries after timeout, fencing, transactions v1 and v2.

### [To Do] API-0331 EndTxn compatibility test with Java client 4.4.0 transactional producer (transactions v1 and v2), librdkafka and franz-go
P0 · M6 · Task · deps: API-0330, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that commits and aborts transactions and read_committed consumers see only committed data passes with Java client 4.4.0 transactional producer (transactions v1 and v2), librdkafka and franz-go.

### [To Do] API-0332 EndTxn differential test against Kafka 4.4.0
P0 · M6 · Task · deps: API-0330, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical EndTxn requests to Kafka 4.4.0 and Alluvium for commit and abort at v0-v5 and for each client-triggerable error (INVALID_TXN_STATE, PRODUCER_FENCED versus INVALID_PRODUCER_EPOCH and CONCURRENT_TRANSACTIONS); normalized responses match for every supported version.

### [To Do] API-0333 Advertise EndTxn v0-5 in ApiVersions
P0 · M6 · Task · deps: API-0328, API-0331, API-0332, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists EndTxn with min 0 and max 5 through the generated list owned by API-F18; docs/parity.md marks EndTxn supported; a test asserts the range.

## [To Do] API-F27 WriteTxnMarkers (v1-2)
P0 · M6 · Feature · deps: TXN-F08
Scope: Kafka 4.4.0 versions 1-2; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0334 WriteTxnMarkers handler module and router registration (broker listener)
P0 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleWriteTxnMarkersRequest @4.4.0
Done when: `handlers/write_txn_markers.rs` (module doc names the Kafka counterpart) decodes v1-2 with the generated codec and calls the marker append path (TXN-F08); it is registered for the broker listener in the router (SRV-F04). A WriteTxnMarkers request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0335 WriteTxnMarkers v1: append COMMIT or ABORT markers per producer
P0 · M6 · Task · deps: API-0334 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleWriteTxnMarkersRequest @4.4.0
Done when: Each marker appends a control batch with the coordinator epoch to its partitions through the replica manager; offline or unknown partitions get UNKNOWN_TOPIC_OR_PARTITION; coordinator-side errors (NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS) from the group coordinator become NOT_LEADER_OR_FOLLOWER; the response waits for every append and an empty request is answered at once; tests.

### [To Do] API-0336 WriteTxnMarkers v2: transaction_version per marker (KIP-1228)
P0 · M6 · Task · deps: API-0335, TXN-F10 · ref: clients/src/main/resources/common/message/WriteTxnMarkersRequest.json @4.4.0 (KIP-1228)
Done when: v2 markers carry transaction_version, passed to the append so transactions v2 markers are validated with the bumped epoch; v1 markers default to 0; tests per version.

### [To Do] API-0337 WriteTxnMarkers completion of transactional offsets in `__consumer_offsets`
P0 · M6 · Task · deps: API-0335 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleWriteTxnMarkersRequest @4.4.0
Done when: Markers for `__consumer_offsets` partitions also complete or discard the group's pending transactional offsets through GRP-F07 before the partition result is reported; tests for commit and abort.

### [To Do] API-0338 WriteTxnMarkers error mapping
P0 · M6 · Task · deps: API-0334 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleWriteTxnMarkersRequest @4.4.0
Done when: Partition errors UNKNOWN_TOPIC_OR_PARTITION, NOT_LEADER_OR_FOLLOWER, KAFKA_STORAGE_ERROR, INVALID_PRODUCER_EPOCH and CLUSTER_AUTHORIZATION_FAILED map exhaustively; table test.

### [To Do] API-0339 WriteTxnMarkers responses exempt from throttling
P0 · M6 · Task · deps: API-0334 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleWriteTxnMarkersRequest sendResponseExemptThrottle @4.4.0
Done when: WriteTxnMarkers is a cluster-action API with no throttle_time_ms field; responses are sent through the exempt path (`sendResponseExemptThrottle`) even for quota-limited principals; test.

### [To Do] API-0340 WriteTxnMarkers request metrics, events and request-log summary
P0 · M6 · Task · deps: API-0334 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for WriteTxnMarkers; the request-log summary (SRV-F09) shows marker count, partition count, transaction result and coordinator epoch. Marker appends are recorded in the partition flight recorder with reason `txn_marker`. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0341 WriteTxnMarkers authorization: ALTER or CLUSTER_ACTION on CLUSTER
P0 · M6 · Task · deps: API-0334, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleWriteTxnMarkersRequest @4.4.0
Done when: Requests need ALTER on CLUSTER (checked without logging a denial) or else CLUSTER_ACTION on CLUSTER, otherwise CLUSTER_AUTHORIZATION_FAILED; tests.

### [To Do] API-0342 WriteTxnMarkers handler unit tests across v1-2
P0 · M6 · Task · deps: API-0336, API-0337, API-0338, API-0339, API-0341 · labels: test
Done when: Table-driven tests send every version v1-v2 through the handler, with the marker append path (TXN-F08) replaced by a fake, and check the service call and the encoded response; cases cover commit and abort markers, unknown partitions, offsets partitions and transaction versions.

### [To Do] API-0343 WriteTxnMarkers multi-node simulation test
P0 · M6 · Task · deps: API-0342, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) runs transactions across 3 brokers with coordinator and leader failover and checks every partition of a completed transaction receives exactly one marker; a failing seed is printed and replays exactly.

### [To Do] API-0344 WriteTxnMarkers differential test against Kafka 4.4.0
P0 · M6 · Task · deps: API-0342, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical WriteTxnMarkers requests to Kafka 4.4.0 and Alluvium for marker writes from a Kafka 4.4.0 transaction coordinator and for each client-triggerable error (UNKNOWN_TOPIC_OR_PARTITION and NOT_LEADER_OR_FOLLOWER); normalized responses match for every supported version.

### [To Do] API-0345 Advertise WriteTxnMarkers v1-2 in ApiVersions
P0 · M6 · Task · deps: API-0340, API-0343, API-0344, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists WriteTxnMarkers with min 1 and max 2 through the generated list owned by API-F18; docs/parity.md marks WriteTxnMarkers supported; a test asserts the range.

## [To Do] API-F28 TxnOffsetCommit (v0-6)
P0 · M6 · Feature · deps: TXN-F07, GRP-F07
Scope: Kafka 4.4.0 versions 0-6; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0346 TxnOffsetCommit handler module and router registration (broker listener)
P0 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTxnOffsetCommitRequest @4.4.0
Done when: `handlers/txn_offset_commit.rs` (module doc names the Kafka counterpart) decodes v0-6 with the generated codec and calls the group coordinator (GRP-F07) and transactions (TXN-F07); it is registered for the broker listener in the router (SRV-F04). A TxnOffsetCommit request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0347 TxnOffsetCommit topic and partition resolution
P0 · M6 · Task · deps: API-0346 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTxnOffsetCommitRequest @4.4.0
Done when: Names (v0-v5) or ids (v6) are resolved through metadata; unknown ids get UNKNOWN_TOPIC_ID, unknown names or partitions UNKNOWN_TOPIC_OR_PARTITION, without reaching the coordinator; tests.

### [To Do] API-0348 TxnOffsetCommit v0-v2: pending offsets and committed_leader_epoch
P0 · M6 · Task · deps: API-0347 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTxnOffsetCommitRequest @4.4.0
Done when: Offsets are written as pending transactional offsets; v2 adds committed_leader_epoch; for v0-v1 COORDINATOR_LOAD_IN_PROGRESS is downgraded to COORDINATOR_NOT_AVAILABLE; tests per version.

### [To Do] API-0349 TxnOffsetCommit v3: member id, group instance id and generation (KIP-447)
P0 · M6 · Task · deps: API-0348 · ref: clients/src/main/resources/common/message/TxnOffsetCommitRequest.json @4.4.0 (KIP-447)
Done when: v3+ carries member_id, group_instance_id and generation_id so the coordinator can return ILLEGAL_GENERATION, UNKNOWN_MEMBER_ID and FENCED_INSTANCE_ID; v3 accepts tagged fields; tests.

### [To Do] API-0350 TxnOffsetCommit v4-v5: TRANSACTION_ABORTABLE and implicit AddOffsetsToTxn (KIP-890)
P0 · M6 · Task · deps: API-0349, TXN-F10 · ref: server/src/main/java/org/apache/kafka/server/transaction/AddPartitionsToTxnManager.java txnOffsetCommitRequestVersionToTransactionSupportedOperation @4.4.0
Done when: v4+ may return TRANSACTION_ABORTABLE (INVALID_TXN_STATE below v4 via the transaction-supported operation); v5+ uses the ADD_PARTITION operation so transactions v2 producers need no AddOffsetsToTxn; tests per version.

### [To Do] API-0351 TxnOffsetCommit v6: topic ids and consumer-group member epochs (KIP-1319)
P0 · M6 · Task · deps: API-0350 · ref: clients/src/main/resources/common/message/TxnOffsetCommitResponse.json @4.4.0 (KIP-1319)
Done when: v6 addresses topics by id and may return GROUP_ID_NOT_FOUND, STALE_MEMBER_EPOCH and UNKNOWN_TOPIC_ID; v0-v5 never receive those codes; tests at v5 and v6.

### [To Do] API-0352 TxnOffsetCommit error mapping per version
P0 · M6 · Task · deps: API-0346 · ref: clients/src/main/resources/common/message/TxnOffsetCommitResponse.json @4.4.0
Done when: The supported-error list of `TxnOffsetCommitResponse.json` is enforced per version: ILLEGAL_GENERATION, UNKNOWN_MEMBER_ID and FENCED_INSTANCE_ID from v3, TRANSACTION_ABORTABLE from v4, GROUP_ID_NOT_FOUND, STALE_MEMBER_EPOCH and UNKNOWN_TOPIC_ID from v6; table test.

### [To Do] API-0353 TxnOffsetCommit throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M6 · Task · deps: API-0346, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; TxnOffsetCommit records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v1+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0354 TxnOffsetCommit request metrics, events and request-log summary
P0 · M6 · Task · deps: API-0346 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for TxnOffsetCommit; the request-log summary (SRV-F09) shows transactional id, group id, producer id and epoch, partition count. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0355 TxnOffsetCommit authorization: TRANSACTIONAL_ID, GROUP and topics
P0 · M6 · Task · deps: API-0347, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTxnOffsetCommitRequest @4.4.0
Done when: Without WRITE on TRANSACTIONAL_ID the request fails with TRANSACTIONAL_ID_AUTHORIZATION_FAILED, then without READ on GROUP with GROUP_AUTHORIZATION_FAILED; topics without READ get TOPIC_AUTHORIZATION_FAILED per partition; tests and differential cases.

### [To Do] API-0356 TxnOffsetCommit handler unit tests across v0-6
P0 · M6 · Task · deps: API-0351, API-0352, API-0353, API-0355 · labels: test
Done when: Table-driven tests send every version v0-v6 through the handler, with the group coordinator (GRP-F07) and transactions (TXN-F07) replaced by a fake, and check the service call and the encoded response; cases cover pending commits, generation checks, fencing, topic ids, every version's error set and the v0-v1 downgrade.

### [To Do] API-0357 TxnOffsetCommit compatibility test with Java client 4.4.0 and Kafka Streams exactly-once (transactions v1 and v2), librdkafka
P0 · M6 · Task · deps: API-0356, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that commits offsets inside transactions and recovers after a crash without duplicates passes with Java client 4.4.0 and Kafka Streams exactly-once (transactions v1 and v2), librdkafka.

### [To Do] API-0358 TxnOffsetCommit differential test against Kafka 4.4.0
P0 · M6 · Task · deps: API-0356, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical TxnOffsetCommit requests to Kafka 4.4.0 and Alluvium for transactional commits at v0-v6 and for each client-triggerable error (ILLEGAL_GENERATION, FENCED_INSTANCE_ID, TRANSACTION_ABORTABLE versus INVALID_TXN_STATE and UNKNOWN_TOPIC_ID); normalized responses match for every supported version.

### [To Do] API-0359 Advertise TxnOffsetCommit v0-6 in ApiVersions
P0 · M6 · Task · deps: API-0354, API-0357, API-0358, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists TxnOffsetCommit with min 0 and max 6 through the generated list owned by API-F18; docs/parity.md marks TxnOffsetCommit supported; a test asserts the range.

## [To Do] API-F29 DescribeAcls (v1-3)
P0 · M4 · Feature · deps: SEC-F09
Scope: Kafka 4.4.0 versions 1-3; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0360 DescribeAcls handler module and router registration (broker and controller listeners)
P0 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/AclApis.scala handleDescribeAcls @4.4.0
Done when: `handlers/describe_acls.rs` (module doc names the Kafka counterpart) decodes v1-3 with the generated codec and calls the ACL administration service (SEC-F09); it is registered for the broker and controller listeners in the router (SRV-F04). Brokers answer from their authorizer copy and the controller from its own; nothing is forwarded. Unit test per listener.

### [To Do] API-0361 DescribeAcls without an authorizer: SECURITY_DISABLED
P0 · M4 · Task · deps: API-0360 · ref: core/src/main/scala/kafka/server/AclApis.scala handleDescribeAcls @4.4.0
Done when: With no authorizer configured, DescribeAcls returns SECURITY_DISABLED at the top level with 'No Authorizer is configured on the broker'; test.

### [To Do] API-0362 DescribeAcls validation of resource, pattern, operation and permission values
P0 · M4 · Task · deps: API-0360 · labels: security · ref: core/src/main/scala/kafka/server/AclApis.scala handleDescribeAcls @4.4.0
Done when: Requests containing UNKNOWN (0) resource types, pattern types, operations or permission types fail to parse, which closes the connection as Kafka's request parsing does; filters may use ANY and MATCH; unit tests.

### [To Do] API-0363 DescribeAcls v1: pattern_type_filter (KIP-290)
P0 · M4 · Task · deps: API-0362 · ref: clients/src/main/resources/common/message/DescribeAclsRequest.json @4.4.0 (KIP-290)
Done when: pattern_type_filter selects LITERAL, PREFIXED, ANY or MATCH bindings and each resource in the response carries its pattern_type; v1 is the 4.4.0 baseline; tests per filter.

### [To Do] API-0364 DescribeAcls v2-v3: flexible encoding and the USER resource type
P0 · M4 · Task · deps: API-0363 · ref: clients/src/main/resources/common/message/DescribeAclsRequest.json @4.4.0
Done when: v2 accepts tagged fields; v3 adds the USER resource type used by CREATE_TOKENS and DESCRIBE_TOKENS bindings (KIP-373); 4.4.0 does not gate USER by version on the server, which a test records; per-version tests.

### [To Do] API-0365 DescribeAcls error mapping
P0 · M4 · Task · deps: API-0360 · ref: core/src/main/scala/kafka/server/AclApis.scala handleDescribeAcls @4.4.0
Done when: SECURITY_DISABLED, INVALID_REQUEST, CLUSTER_AUTHORIZATION_FAILED, NOT_CONTROLLER and UNKNOWN_SERVER_ERROR map exhaustively with error messages at the top level; table test.

### [To Do] API-0366 DescribeAcls throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0360, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; DescribeAcls records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0367 DescribeAcls request metrics, events and request-log summary
P0 · M4 · Task · deps: API-0360 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeAcls; the request-log summary (SRV-F09) shows resource, pattern, principal and operation filters and the number of bindings returned. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0368 DescribeAcls authorization: DESCRIBE on CLUSTER
P0 · M4 · Task · deps: API-0360, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/AclApis.scala handleDescribeAcls @4.4.0
Done when: Requests need DESCRIBE on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

### [To Do] API-0369 DescribeAcls handler unit tests across v1-3
P0 · M4 · Task · deps: API-0361, API-0364, API-0365, API-0366, API-0368 · labels: test
Done when: Table-driven tests send every version v1-v3 through the handler, with the ACL administration service (SEC-F09) replaced by a fake, and check the service call and the encoded response; cases cover literal, prefixed, ANY and MATCH patterns, the USER resource type, no authorizer and authorization failures.

### [To Do] API-0370 DescribeAcls compatibility test with Java client 4.4.0 Admin and kafka-acls.sh
P0 · M4 · Task · deps: API-0369, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that lists ACLs with every filter combination (--list) passes with Java client 4.4.0 Admin and kafka-acls.sh.

### [To Do] API-0371 DescribeAcls differential test against Kafka 4.4.0
P0 · M4 · Task · deps: API-0369, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeAcls requests to Kafka 4.4.0 and Alluvium for descriptions with each filter and for each client-triggerable error (SECURITY_DISABLED, INVALID_REQUEST and CLUSTER_AUTHORIZATION_FAILED); normalized responses match for every supported version.

### [To Do] API-0372 Advertise DescribeAcls v1-3 in ApiVersions
P0 · M4 · Task · deps: API-0367, API-0370, API-0371, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists DescribeAcls with min 1 and max 3 through the generated list owned by API-F18; docs/parity.md marks DescribeAcls supported; a test asserts the range.

## [To Do] API-F30 CreateAcls (v1-3)
P0 · M4 · Feature · deps: SEC-F09
Scope: Kafka 4.4.0 versions 1-3; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0373 CreateAcls handler module and router registration (broker and controller listeners)
P0 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/AclApis.scala handleCreateAcls @4.4.0
Done when: `handlers/create_acls.rs` (module doc names the Kafka counterpart) decodes v1-3 with the generated codec and calls the ACL administration service (SEC-F09); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0374 Forward CreateAcls from brokers to the active controller in an Envelope
P0 · M4 · Task · deps: API-0373, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener CreateAcls is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0375 CreateAcls without an authorizer: SECURITY_DISABLED
P0 · M4 · Task · deps: API-0373 · ref: core/src/main/scala/kafka/server/AclApis.scala handleCreateAcls @4.4.0
Done when: With no authorizer configured, CreateAcls returns SECURITY_DISABLED for every entry with 'No Authorizer is configured.'; test.

### [To Do] API-0376 CreateAcls validation of resource, pattern, operation and permission values
P0 · M4 · Task · deps: API-0373 · labels: security · ref: core/src/main/scala/kafka/server/AclApis.scala handleCreateAcls @4.4.0
Done when: Requests containing UNKNOWN (0) resource types, pattern types, operations or permission types fail to parse, which closes the connection as Kafka's request parsing does; a CLUSTER binding not named `kafka-cluster` or an empty resource name fails that creation with INVALID_REQUEST; unit tests.

### [To Do] API-0377 CreateAcls v1: resource_pattern_type (KIP-290)
P0 · M4 · Task · deps: API-0376 · ref: clients/src/main/resources/common/message/CreateAclsRequest.json @4.4.0 (KIP-290)
Done when: Each creation carries resource_pattern_type (LITERAL or PREFIXED; ANY or MATCH fail that creation with INVALID_REQUEST from SEC-F09) and results keep request order; v1 is the 4.4.0 baseline; tests.

### [To Do] API-0378 CreateAcls v2-v3: flexible encoding and the USER resource type
P0 · M4 · Task · deps: API-0377 · ref: clients/src/main/resources/common/message/CreateAclsRequest.json @4.4.0
Done when: v2 accepts tagged fields; v3 adds the USER resource type used by CREATE_TOKENS and DESCRIBE_TOKENS bindings (KIP-373); 4.4.0 does not gate USER by version on the server, which a test records; per-version tests.

### [To Do] API-0379 CreateAcls error mapping
P0 · M4 · Task · deps: API-0373 · ref: core/src/main/scala/kafka/server/AclApis.scala handleCreateAcls @4.4.0
Done when: SECURITY_DISABLED, INVALID_REQUEST, CLUSTER_AUTHORIZATION_FAILED, NOT_CONTROLLER and UNKNOWN_SERVER_ERROR map exhaustively with error messages at the entry level; table test.

### [To Do] API-0380 CreateAcls completion through a bounded delayed operation
P0 · M4 · Task · deps: API-0373 · ref: core/src/main/scala/kafka/server/AclApis.scala handleCreateAcls alterAclsPurgatory @4.4.0
Done when: CreateAcls responses wait for every ACL change future in a delayed operation (SRV-F07) bounded by `connections.max.idle.ms`, as Kafka's AlterAcls purgatory does; test with a slow authorizer write.

### [To Do] API-0381 CreateAcls throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0373, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; CreateAcls records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal.

### [To Do] API-0382 CreateAcls request metrics, events and request-log summary
P0 · M4 · Task · deps: API-0373 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for CreateAcls; the request-log summary (SRV-F09) shows the number of creations and their resource types. Each ACL change emits an audit event with reason `create_acls_request` (SEC-F09). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0383 CreateAcls authorization: ALTER on CLUSTER
P0 · M4 · Task · deps: API-0373, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/AclApis.scala handleCreateAcls @4.4.0
Done when: Requests need ALTER on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED (forwarded requests are checked on the controller with the envelope principal); tests and a differential case.

### [To Do] API-0384 CreateAcls handler unit tests across v1-3
P0 · M4 · Task · deps: API-0374, API-0375, API-0378, API-0379, API-0380, API-0381, API-0383 · labels: test
Done when: Table-driven tests send every version v1-v3 through the handler, with the ACL administration service (SEC-F09) replaced by a fake, and check the service call and the encoded response; cases cover literal, prefixed, ANY and MATCH patterns, the USER resource type, no authorizer and authorization failures.

### [To Do] API-0385 CreateAcls compatibility test with Java client 4.4.0 Admin and kafka-acls.sh
P0 · M4 · Task · deps: API-0384, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that adds literal and prefixed ACLs for users and groups (--add) passes with Java client 4.4.0 Admin and kafka-acls.sh.

### [To Do] API-0386 CreateAcls differential test against Kafka 4.4.0
P0 · M4 · Task · deps: API-0384, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical CreateAcls requests to Kafka 4.4.0 and Alluvium for creations of each pattern type and for each client-triggerable error (SECURITY_DISABLED, INVALID_REQUEST and CLUSTER_AUTHORIZATION_FAILED); normalized responses match for every supported version.

### [To Do] API-0387 Advertise CreateAcls v1-3 in ApiVersions
P0 · M4 · Task · deps: API-0382, API-0385, API-0386, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists CreateAcls with min 1 and max 3 through the generated list owned by API-F18; docs/parity.md marks CreateAcls supported; a test asserts the range.

## [To Do] API-F31 DeleteAcls (v1-3)
P0 · M4 · Feature · deps: SEC-F09
Scope: Kafka 4.4.0 versions 1-3; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0388 DeleteAcls handler module and router registration (broker and controller listeners)
P0 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/AclApis.scala handleDeleteAcls @4.4.0
Done when: `handlers/delete_acls.rs` (module doc names the Kafka counterpart) decodes v1-3 with the generated codec and calls the ACL administration service (SEC-F09); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0389 Forward DeleteAcls from brokers to the active controller in an Envelope
P0 · M4 · Task · deps: API-0388, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener DeleteAcls is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0390 DeleteAcls without an authorizer: SECURITY_DISABLED
P0 · M4 · Task · deps: API-0388 · ref: core/src/main/scala/kafka/server/AclApis.scala handleDeleteAcls @4.4.0
Done when: With no authorizer configured, DeleteAcls returns SECURITY_DISABLED for every entry with 'No Authorizer is configured.'; test.

### [To Do] API-0391 DeleteAcls validation of resource, pattern, operation and permission values
P0 · M4 · Task · deps: API-0388 · labels: security · ref: core/src/main/scala/kafka/server/AclApis.scala handleDeleteAcls @4.4.0
Done when: Requests containing UNKNOWN (0) resource types, pattern types, operations or permission types fail to parse, which closes the connection as Kafka's request parsing does; filters may use ANY and MATCH and each filter's matches are reported separately; unit tests.

### [To Do] API-0392 DeleteAcls v1: pattern_type_filter and per-match results (KIP-290)
P0 · M4 · Task · deps: API-0391 · ref: clients/src/main/resources/common/message/DeleteAclsRequest.json @4.4.0 (KIP-290)
Done when: Filters carry pattern_type_filter and every matching ACL is returned with its pattern_type, error_code and error_message; v1 is the 4.4.0 baseline; tests.

### [To Do] API-0393 DeleteAcls v2-v3: flexible encoding and the USER resource type
P0 · M4 · Task · deps: API-0392 · ref: clients/src/main/resources/common/message/DeleteAclsRequest.json @4.4.0
Done when: v2 accepts tagged fields; v3 adds the USER resource type used by CREATE_TOKENS and DESCRIBE_TOKENS bindings (KIP-373); 4.4.0 does not gate USER by version on the server, which a test records; per-version tests.

### [To Do] API-0394 DeleteAcls error mapping
P0 · M4 · Task · deps: API-0388 · ref: core/src/main/scala/kafka/server/AclApis.scala handleDeleteAcls @4.4.0
Done when: SECURITY_DISABLED, INVALID_REQUEST, CLUSTER_AUTHORIZATION_FAILED, NOT_CONTROLLER and UNKNOWN_SERVER_ERROR map exhaustively with error messages at the entry level; table test.

### [To Do] API-0395 DeleteAcls completion through a bounded delayed operation
P0 · M4 · Task · deps: API-0388 · ref: core/src/main/scala/kafka/server/AclApis.scala handleDeleteAcls alterAclsPurgatory @4.4.0
Done when: DeleteAcls responses wait for every ACL change future in a delayed operation (SRV-F07) bounded by `connections.max.idle.ms`, as Kafka's AlterAcls purgatory does; test with a slow authorizer write.

### [To Do] API-0396 DeleteAcls throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0388, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; DeleteAcls records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal.

### [To Do] API-0397 DeleteAcls request metrics, events and request-log summary
P0 · M4 · Task · deps: API-0388 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DeleteAcls; the request-log summary (SRV-F09) shows the number of filters and of deleted bindings. Each ACL change emits an audit event with reason `delete_acls_request` (SEC-F09). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0398 DeleteAcls authorization: ALTER on CLUSTER
P0 · M4 · Task · deps: API-0388, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/AclApis.scala handleDeleteAcls @4.4.0
Done when: Requests need ALTER on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED (forwarded requests are checked on the controller with the envelope principal); tests and a differential case.

### [To Do] API-0399 DeleteAcls handler unit tests across v1-3
P0 · M4 · Task · deps: API-0389, API-0390, API-0393, API-0394, API-0395, API-0396, API-0398 · labels: test
Done when: Table-driven tests send every version v1-v3 through the handler, with the ACL administration service (SEC-F09) replaced by a fake, and check the service call and the encoded response; cases cover literal, prefixed, ANY and MATCH patterns, the USER resource type, no authorizer and authorization failures.

### [To Do] API-0400 DeleteAcls compatibility test with Java client 4.4.0 Admin and kafka-acls.sh
P0 · M4 · Task · deps: API-0399, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that removes ACLs by filter, including forced wildcard removals (--remove --force) passes with Java client 4.4.0 Admin and kafka-acls.sh.

### [To Do] API-0401 DeleteAcls differential test against Kafka 4.4.0
P0 · M4 · Task · deps: API-0399, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DeleteAcls requests to Kafka 4.4.0 and Alluvium for deletions by filter and for each client-triggerable error (SECURITY_DISABLED, INVALID_REQUEST and CLUSTER_AUTHORIZATION_FAILED); normalized responses match for every supported version.

### [To Do] API-0402 Advertise DeleteAcls v1-3 in ApiVersions
P0 · M4 · Task · deps: API-0397, API-0400, API-0401, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists DeleteAcls with min 1 and max 3 through the generated list owned by API-F18; docs/parity.md marks DeleteAcls supported; a test asserts the range.

## [To Do] API-F32 DescribeConfigs (v1-4)
P0 · M2 · Feature · deps: CFG-F05
Scope: Kafka 4.4.0 versions 1-4; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0403 DescribeConfigs handler module and router registration (broker and controller listeners)
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/ConfigHelper.scala handleDescribeConfigsRequest @4.4.0
Done when: `handlers/describe_configs.rs` (module doc names the Kafka counterpart) decodes v1-4 with the generated codec and calls the config service (CFG-F05); it is registered for the broker and controller listeners in the router (SRV-F04). Brokers answer locally and the controller answers from its own config view; neither forwards. Unit test per listener.

### [To Do] API-0404 DescribeConfigs resource validation
P0 · M2 · Task · deps: API-0403 · labels: security · ref: core/src/main/scala/kafka/server/ConfigHelper.scala describeConfigs @4.4.0
Done when: Resource types other than BROKER, BROKER_LOGGER, CLIENT_METRICS, TOPIC and GROUP get INVALID_REQUEST; a BROKER name must be empty (cluster defaults) or this node's id, a BROKER_LOGGER name must be this node's id, and CLIENT_METRICS and GROUP names must be non-empty, otherwise INVALID_REQUEST for that resource; invalid topic names get INVALID_TOPIC_EXCEPTION and unknown topics UNKNOWN_TOPIC_OR_PARTITION; null configuration_keys returns every key; unit tests.

### [To Do] API-0405 DescribeConfigs v1-v2: config_source and include_synonyms
P0 · M2 · Task · deps: API-0404 · ref: clients/src/main/resources/common/message/DescribeConfigsResponse.json @4.4.0
Done when: v1+ entries carry config_source (DYNAMIC_TOPIC_CONFIG, STATIC_BROKER_CONFIG, DEFAULT_CONFIG and so on) instead of is_default, and synonyms when include_synonyms is set; sensitive values are null; tests per version with topic overrides and static broker defaults.

### [To Do] API-0406 DescribeConfigs v3-v4: config_type and documentation
P0 · M2 · Task · deps: API-0404 · ref: clients/src/main/resources/common/message/DescribeConfigsRequest.json @4.4.0
Done when: v3+ entries carry config_type and, when include_documentation is set, the documentation string generated by CFG-F12; v4 is the first flexible version; tests per version.

### [To Do] API-0407 DescribeConfigs error mapping
P0 · M2 · Task · deps: API-0403 · ref: core/src/main/scala/kafka/server/ConfigHelper.scala handleDescribeConfigsRequest @4.4.0
Done when: Per-resource errors INVALID_REQUEST, INVALID_TOPIC_EXCEPTION, UNKNOWN_TOPIC_OR_PARTITION and TOPIC_AUTHORIZATION_FAILED/CLUSTER_AUTHORIZATION_FAILED/GROUP_AUTHORIZATION_FAILED carry error messages and an empty config list; table test.

### [To Do] API-0408 DescribeConfigs throttle_time_ms through the shared response path
P0 · M2 · Task · deps: API-0403 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0409 DescribeConfigs throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0408, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: DescribeConfigs records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v2+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0410 DescribeConfigs request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0403 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeConfigs; the request-log summary (SRV-F09) shows resource types and names, key count, include_synonyms and include_documentation. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0411 DescribeConfigs handler unit tests across v1-4
P0 · M2 · Task · deps: API-0405, API-0406, API-0407, API-0408 · labels: test
Done when: Table-driven tests send every version v1-v4 through the handler, with the config service (CFG-F05) replaced by a fake, and check the service call and the encoded response; cases cover every resource type, selected keys, synonyms, documentation, sensitive values and unknown resources.

### [To Do] API-0412 DescribeConfigs compatibility test with Java client 4.4.0 Admin.describeConfigs, librdkafka, franz-go and kafka-configs.sh --describe
P0 · M2 · Task · deps: API-0411, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that describes topic, broker and broker-logger configs with and without --all passes with Java client 4.4.0 Admin.describeConfigs, librdkafka, franz-go and kafka-configs.sh --describe.

### [To Do] API-0413 DescribeConfigs differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0411, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeConfigs requests to Kafka 4.4.0 and Alluvium for topic and broker resources with synonyms and documentation and for each client-triggerable error (INVALID_REQUEST for unknown types and foreign broker ids and UNKNOWN_TOPIC_OR_PARTITION); normalized responses match for every supported version.

### [To Do] API-0414 Advertise DescribeConfigs v1-4 in ApiVersions
P0 · M2 · Task · deps: API-0410, API-0412, API-0413, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists DescribeConfigs with min 1 and max 4 through the generated list owned by API-F18; docs/parity.md marks DescribeConfigs supported; a test asserts the range. Later tasks (M4, M7, M8) add behavior without changing the advertised range.

### [To Do] API-0415 DescribeConfigs authorization: DESCRIBE_CONFIGS per resource type
P0 · M4 · Task · deps: API-0403, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ConfigHelper.scala handleDescribeConfigsRequest @4.4.0
Done when: BROKER, BROKER_LOGGER and CLIENT_METRICS need DESCRIBE_CONFIGS on CLUSTER, TOPIC and GROUP need it on the resource; denied resources get the matching *_AUTHORIZATION_FAILED error while others are answered; tests and differential cases.

### [To Do] API-0416 DescribeConfigs for dynamic broker and cluster-default configs
P0 · M7 · Task · deps: API-0405, CFG-F04, CFG-F06 · ref: core/src/main/scala/kafka/server/ConfigHelper.scala describeConfigs @4.4.0
Done when: Broker resources report DYNAMIC_BROKER_CONFIG and DYNAMIC_DEFAULT_BROKER_CONFIG sources and synonyms from CFG-F04 and CFG-F06; an empty broker name describes cluster defaults; differential test.

### [To Do] API-0417 DescribeConfigs for GROUP and CLIENT_METRICS resources
P0 · M8 · Task · deps: API-0405, TELEM-F03, OBS-F02 · ref: core/src/main/scala/kafka/server/ConfigHelper.scala describeConfigs @4.4.0
Done when: GROUP resources return the group config defaults layered with overrides (no existence check, as Kafka) and CLIENT_METRICS resources the subscription configs from TELEM-F03; BROKER_LOGGER returns logger levels with DYNAMIC_BROKER_LOGGER_CONFIG source; differential test.

## [To Do] API-F33 AlterConfigs (v0-2)
P1 · M7 · Feature · deps: CFG-F04, CTRL-F11
Scope: Kafka 4.4.0 versions 0-2; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0418 AlterConfigs handler module and router registration (broker and controller listeners)
P1 · M7 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAlterConfigsRequest @4.4.0
Done when: `handlers/alter_configs.rs` (module doc names the Kafka counterpart) decodes v0-2 with the generated codec and calls the config change path (CTRL-F11, CFG-F04); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0419 AlterConfigs broker-side preprocessing
P1 · M7 · Task · deps: API-0418 · labels: security · ref: core/src/main/scala/kafka/server/ConfigAdminManager.scala preprocess @4.4.0
Done when: Before forwarding, the broker answers resources it handles locally or rejects (duplicate resources get INVALID_REQUEST 'Duplicate resource.', BROKER_LOGGER is not supported by the legacy API) and forwards the rest, then reassembles one response in request order as `ConfigAdminManager.reassembleLegacyResponse` does; unit tests.

### [To Do] API-0420 Forward AlterConfigs from brokers to the active controller in an Envelope
P1 · M7 · Task · deps: API-0419, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener AlterConfigs is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0421 AlterConfigs v0-v2: full replacement and validate_only
P1 · M7 · Task · deps: API-0420 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleLegacyAlterConfigs @4.4.0
Done when: Each resource's configs replace all existing dynamic configs for it (keys not listed are removed); validate_only validates without writing; unknown resource types get UNSUPPORTED_VERSION 'Unknown resource type N.'; v2 accepts tagged fields; tests.

### [To Do] API-0422 AlterConfigs error mapping
P1 · M7 · Task · deps: API-0418 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleLegacyAlterConfigs @4.4.0
Done when: INVALID_REQUEST, INVALID_CONFIG, POLICY_VIOLATION, UNKNOWN_TOPIC_OR_PARTITION, UNSUPPORTED_VERSION and the authorization errors map exhaustively per resource with messages; table test.

### [To Do] API-0423 AlterConfigs throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M7 · Task · deps: API-0418, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; AlterConfigs records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v1+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal.

### [To Do] API-0424 AlterConfigs request metrics, events and request-log summary
P1 · M7 · Task · deps: API-0418 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for AlterConfigs; the request-log summary (SRV-F09) shows resource types and names, config key count and validate_only, never sensitive values. Config changes carry reason `alter_configs_request` (CTRL-F11). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0425 AlterConfigs authorization: ALTER_CONFIGS per resource
P1 · M7 · Task · deps: API-0419, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala authorizeAlterResource @4.4.0
Done when: BROKER resources need ALTER_CONFIGS on CLUSTER, TOPIC and GROUP resources need it on the resource; denied resources get CLUSTER_AUTHORIZATION_FAILED, TOPIC_AUTHORIZATION_FAILED or GROUP_AUTHORIZATION_FAILED while others proceed; tests and differential cases.

### [To Do] API-0426 AlterConfigs handler unit tests across v0-2
P1 · M7 · Task · deps: API-0421, API-0422, API-0423, API-0425 · labels: test
Done when: Table-driven tests send every version v0-v2 through the handler, with the config change path (CTRL-F11, CFG-F04) replaced by a fake, and check the service call and the encoded response; cases cover topic, broker and cluster-default resources, removal of unlisted keys, duplicates, unknown types and validate_only.

### [To Do] API-0427 AlterConfigs compatibility test with Java client 4.4.0 Admin.alterConfigs (deprecated API) and librdkafka AlterConfigs
P1 · M7 · Task · deps: API-0426, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that replaces topic configs and verifies with DescribeConfigs passes with Java client 4.4.0 Admin.alterConfigs (deprecated API) and librdkafka AlterConfigs.

### [To Do] API-0428 AlterConfigs differential test against Kafka 4.4.0
P1 · M7 · Task · deps: API-0426, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical AlterConfigs requests to Kafka 4.4.0 and Alluvium for replacements and validate_only and for each client-triggerable error (duplicate resources, INVALID_CONFIG, UNSUPPORTED_VERSION for unknown types and POLICY_VIOLATION); normalized responses match for every supported version.

### [To Do] API-0429 Advertise AlterConfigs v0-2 in ApiVersions
P1 · M7 · Task · deps: API-0424, API-0427, API-0428, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists AlterConfigs with min 0 and max 2 through the generated list owned by API-F18; docs/parity.md marks AlterConfigs supported; a test asserts the range.

## [To Do] API-F34 AlterReplicaLogDirs (v1-2)
P1 · M7 · Feature · deps: STOR-F22
Scope: Kafka 4.4.0 versions 1-2; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0430 AlterReplicaLogDirs handler module and router registration (broker listener)
P1 · M7 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAlterReplicaLogDirsRequest @4.4.0
Done when: `handlers/alter_replica_log_dirs.rs` (module doc names the Kafka counterpart) decodes v1-2 with the generated codec and calls the log directory mover (STOR-F22); it is registered for the broker listener in the router (SRV-F04). A AlterReplicaLogDirs request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0431 AlterReplicaLogDirs v1-v2: move replicas between log directories
P1 · M7 · Task · deps: API-0430 · ref: clients/src/main/resources/common/message/AlterReplicaLogDirsRequest.json @4.4.0
Done when: Each listed replica is moved to the named path through STOR-F22 (future log, copy and swap); an unknown path gets LOG_DIR_NOT_FOUND, an offline directory KAFKA_STORAGE_ERROR, a replica not on this broker REPLICA_NOT_AVAILABLE; v2 accepts tagged fields; tests.

### [To Do] API-0432 AlterReplicaLogDirs error mapping
P1 · M7 · Task · deps: API-0430 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAlterReplicaLogDirsRequest @4.4.0
Done when: LOG_DIR_NOT_FOUND, KAFKA_STORAGE_ERROR, REPLICA_NOT_AVAILABLE, UNKNOWN_TOPIC_OR_PARTITION and CLUSTER_AUTHORIZATION_FAILED map exhaustively per partition; table test.

### [To Do] API-0433 AlterReplicaLogDirs throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M7 · Task · deps: API-0430, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; AlterReplicaLogDirs records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0434 AlterReplicaLogDirs request metrics, events and request-log summary
P1 · M7 · Task · deps: API-0430 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for AlterReplicaLogDirs; the request-log summary (SRV-F09) shows partition count and target directories. Replica moves carry reason `alter_replica_log_dirs_request` (STOR-F22). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0435 AlterReplicaLogDirs authorization: ALTER on CLUSTER
P1 · M7 · Task · deps: API-0430, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAlterReplicaLogDirsRequest @4.4.0
Done when: Requests need ALTER on CLUSTER or every partition gets CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

### [To Do] API-0436 AlterReplicaLogDirs handler unit tests across v1-2
P1 · M7 · Task · deps: API-0431, API-0432, API-0433, API-0435 · labels: test
Done when: Table-driven tests send every version v1-v2 through the handler, with the log directory mover (STOR-F22) replaced by a fake, and check the service call and the encoded response; cases cover moves to valid, unknown and offline directories and replicas not hosted here.

### [To Do] API-0437 AlterReplicaLogDirs compatibility test with Java client 4.4.0 Admin.alterReplicaLogDirs and kafka-reassign-partitions.sh with log_dirs
P1 · M7 · Task · deps: API-0436, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that moves a replica between directories while producing passes with Java client 4.4.0 Admin.alterReplicaLogDirs and kafka-reassign-partitions.sh with log_dirs.

### [To Do] API-0438 AlterReplicaLogDirs differential test against Kafka 4.4.0
P1 · M7 · Task · deps: API-0436, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical AlterReplicaLogDirs requests to Kafka 4.4.0 and Alluvium for moves between two directories and for each client-triggerable error (LOG_DIR_NOT_FOUND and REPLICA_NOT_AVAILABLE); normalized responses match for every supported version.

### [To Do] API-0439 Advertise AlterReplicaLogDirs v1-2 in ApiVersions
P1 · M7 · Task · deps: API-0434, API-0437, API-0438, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists AlterReplicaLogDirs with min 1 and max 2 through the generated list owned by API-F18; docs/parity.md marks AlterReplicaLogDirs supported; a test asserts the range.

## [To Do] API-F35 DescribeLogDirs (v1-5)
P1 · M7 · Feature · deps: SRV-F11
Scope: Kafka 4.4.0 versions 1-5; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0440 DescribeLogDirs handler module and router registration (broker listener)
P1 · M7 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeLogDirsRequest @4.4.0
Done when: `handlers/describe_log_dirs.rs` (module doc names the Kafka counterpart) decodes v1-5 with the generated codec and calls the log directory manager (SRV-F11); it is registered for the broker listener in the router (SRV-F04). A DescribeLogDirs request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0441 DescribeLogDirs v1-v2: directories, partitions, sizes and lag
P1 · M7 · Task · deps: API-0440 · ref: clients/src/main/resources/common/message/DescribeLogDirsResponse.json @4.4.0
Done when: A null topics list describes every partition; each directory returns its error code, path and partitions with size, offset lag and is_future_key; v2 accepts tagged fields; tests.

### [To Do] API-0442 DescribeLogDirs v3: top-level error_code
P1 · M7 · Task · deps: API-0441 · ref: clients/src/main/resources/common/message/DescribeLogDirsResponse.json @4.4.0
Done when: v3+ responses carry a top-level error_code (CLUSTER_AUTHORIZATION_FAILED with no results); v1-v2 return an empty result list instead; tests per version.

### [To Do] API-0443 DescribeLogDirs v4-v5: total and usable bytes, is_cordoned
P1 · M7 · Task · deps: API-0442 · ref: clients/src/main/resources/common/message/DescribeLogDirsResponse.json @4.4.0 (KIP-827)
Done when: v4+ directories report total_bytes and usable_bytes from the filesystem abstraction and v5 is_cordoned from SRV-F11; older versions omit them; tests per version.

### [To Do] API-0444 DescribeLogDirs error mapping per version
P1 · M7 · Task · deps: API-0440 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeLogDirsRequest @4.4.0
Done when: Per-directory KAFKA_STORAGE_ERROR and the v3+ top-level CLUSTER_AUTHORIZATION_FAILED map exhaustively; table test.

### [To Do] API-0445 DescribeLogDirs throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M7 · Task · deps: API-0440, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; DescribeLogDirs records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0446 DescribeLogDirs request metrics, events and request-log summary
P1 · M7 · Task · deps: API-0440 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeLogDirs; the request-log summary (SRV-F09) shows topic filter size and directory count. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0447 DescribeLogDirs authorization: DESCRIBE on CLUSTER
P1 · M7 · Task · deps: API-0442, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeLogDirsRequest @4.4.0
Done when: Requests need DESCRIBE on CLUSTER, otherwise v1-v2 get an empty result and v3+ CLUSTER_AUTHORIZATION_FAILED; tests and differential cases.

### [To Do] API-0448 DescribeLogDirs handler unit tests across v1-5
P1 · M7 · Task · deps: API-0443, API-0444, API-0445, API-0447 · labels: test
Done when: Table-driven tests send every version v1-v5 through the handler, with the log directory manager (SRV-F11) replaced by a fake, and check the service call and the encoded response; cases cover all and selected partitions, offline and cordoned directories, future replicas and each version's fields.

### [To Do] API-0449 DescribeLogDirs compatibility test with Java client 4.4.0 Admin.describeLogDirs and kafka-log-dirs.sh --describe
P1 · M7 · Task · deps: API-0448, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that describes directories with sizes during a replica move passes with Java client 4.4.0 Admin.describeLogDirs and kafka-log-dirs.sh --describe.

### [To Do] API-0450 DescribeLogDirs differential test against Kafka 4.4.0
P1 · M7 · Task · deps: API-0448, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeLogDirs requests to Kafka 4.4.0 and Alluvium for descriptions at v1-v5 (ignoring sizes) and for each client-triggerable error (offline directories (KAFKA_STORAGE_ERROR)); normalized responses match for every supported version.

### [To Do] API-0451 Advertise DescribeLogDirs v1-5 in ApiVersions
P1 · M7 · Task · deps: API-0446, API-0449, API-0450, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists DescribeLogDirs with min 1 and max 5 through the generated list owned by API-F18; docs/parity.md marks DescribeLogDirs supported; a test asserts the range.

## [To Do] API-F36 SaslAuthenticate (v0-2)
P0 · M4 · Feature · deps: NET-F08, SEC-F02
Scope: Kafka 4.4.0 versions 0-2; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0452 SaslAuthenticate handler module and router registration (broker and controller listeners)
P0 · M4 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java handleSaslToken @4.4.0
Done when: `handlers/sasl_authenticate.rs` (module doc names the Kafka counterpart) decodes v0-2 with the generated codec and calls the SASL driver (NET-F08); it is registered for the broker and controller listeners in the router (SRV-F04). After authentication completes, a SaslAuthenticate outside re-authentication gets ILLEGAL_SASL_STATE 'SaslAuthenticate request received after successful authentication'. Unit test per listener.

### [To Do] API-0453 SaslAuthenticate v0: token exchange and failure reporting
P0 · M4 · Task · deps: API-0452 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java handleSaslToken @4.4.0
Done when: auth_bytes are passed to the mechanism state machine (SEC-F02) and its challenge returned; a failed exchange returns SASL_AUTHENTICATION_FAILED with the mechanism's client-safe error_message and closes the connection after the response; a SaslAuthenticate without a prior v1 handshake gets ILLEGAL_SASL_STATE; tests.

### [To Do] API-0454 SaslAuthenticate v1: session_lifetime_ms (KIP-368)
P0 · M4 · Task · deps: API-0453 · ref: clients/src/main/resources/common/message/SaslAuthenticateResponse.json @4.4.0 (KIP-368)
Done when: v1+ final responses carry session_lifetime_ms derived from `connections.max.reauth.ms` and the credential's expiry (SEC-F12); v0 clients get no lifetime and are disconnected when it ends; tests per version.

### [To Do] API-0455 SaslAuthenticate v2: flexible encoding
P0 · M4 · Task · deps: API-0454 · ref: clients/src/main/resources/common/message/SaslAuthenticateRequest.json @4.4.0
Done when: v2 requests and responses accept tagged fields with the same semantics as v1; round-trip tests at v1 and v2.

### [To Do] API-0456 SaslAuthenticate re-authentication on an authenticated connection
P0 · M4 · Task · deps: API-0454, SEC-F12 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java reauthenticate @4.4.0 (KIP-368)
Done when: A v1+ client may re-run SaslHandshake and SaslAuthenticate before its session expires; in-flight requests keep the old principal, a changed principal fails re-authentication, and an expired session closes the connection on its next request (SEC-F12); tests.

### [To Do] API-0457 SaslAuthenticate error mapping
P0 · M4 · Task · deps: API-0452 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java @4.4.0
Done when: SASL_AUTHENTICATION_FAILED, ILLEGAL_SASL_STATE, UNSUPPORTED_SASL_MECHANISM and UNSUPPORTED_VERSION map exhaustively; error_message never contains credential material; table test.

### [To Do] API-0458 SaslAuthenticate request metrics, events and request-log summary
P0 · M4 · Task · deps: API-0452 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for SaslAuthenticate; the request-log summary (SRV-F09) shows listener, mechanism, round number and outcome without auth_bytes. Successful and failed authentications and re-authentications emit events with reasons (SEC-F02). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0459 SaslAuthenticate handler unit tests across v0-2
P0 · M4 · Task · deps: API-0455, API-0456, API-0457 · labels: test
Done when: Table-driven tests send every version v0-v2 through the handler, with the SASL driver (NET-F08) replaced by a fake, and check the service call and the encoded response; cases cover successful PLAIN and SCRAM exchanges, bad passwords, out-of-order requests, lifetimes and re-authentication.

### [To Do] API-0460 SaslAuthenticate compatibility test with Java client 4.4.0, librdkafka and franz-go with SCRAM, PLAIN, OAUTHBEARER and delegation-token SCRAM
P0 · M4 · Task · deps: API-0459, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that authenticates, re-authenticates with `connections.max.reauth.ms` set and reports a bad password with the right error passes with Java client 4.4.0, librdkafka and franz-go with SCRAM, PLAIN, OAUTHBEARER and delegation-token SCRAM.

### [To Do] API-0461 SaslAuthenticate differential test against Kafka 4.4.0
P0 · M4 · Task · deps: API-0459, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical SaslAuthenticate requests to Kafka 4.4.0 and Alluvium for successful exchanges at v0-v2 and for each client-triggerable error (SASL_AUTHENTICATION_FAILED messages and ILLEGAL_SASL_STATE); normalized responses match for every supported version.

### [To Do] API-0462 Advertise SaslAuthenticate v0-2 in ApiVersions
P0 · M4 · Task · deps: API-0458, API-0460, API-0461, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists SaslAuthenticate with min 0 and max 2 through the generated list owned by API-F18; docs/parity.md marks SaslAuthenticate supported; a test asserts the range. SaslAuthenticate is listed on every listener.

## [To Do] API-F37 CreatePartitions (v0-3)
P0 · M2 · Feature · deps: CTRL-F06, CTRL-F21
Scope: Kafka 4.4.0 versions 0-3; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0463 CreatePartitions handler module and router registration (broker and controller listeners)
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleCreatePartitions @4.4.0
Done when: `handlers/create_partitions.rs` (module doc names the Kafka counterpart) decodes v0-3 with the generated codec and calls the controller partition-creation path (CTRL-F06); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0464 Forward CreatePartitions from brokers to the active controller in an Envelope
P0 · M2 · Task · deps: API-0463, API-F58 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener CreatePartitions is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0465 CreatePartitions validation: duplicates and request shape
P0 · M2 · Task · deps: API-0463 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala createPartitions @4.4.0
Done when: Topics listed twice get INVALID_REQUEST 'Duplicate topic name.'; count, assignments and validate_only are passed to CTRL-F06, whose INVALID_PARTITIONS and INVALID_REPLICA_ASSIGNMENT results are returned per topic with messages; unit tests.

### [To Do] API-0466 CreatePartitions v0-v2: validate_only and per-topic results
P0 · M2 · Task · deps: API-0465 · ref: clients/src/main/resources/common/message/CreatePartitionsRequest.json @4.4.0
Done when: validate_only checks without writing records; each topic returns error_code and error_message; v2 is the first flexible version; per-version tests.

### [To Do] API-0467 CreatePartitions error mapping
P0 · M2 · Task · deps: API-0463 · ref: core/src/main/scala/kafka/server/ControllerApis.scala createPartitions @4.4.0
Done when: UNKNOWN_TOPIC_OR_PARTITION, INVALID_PARTITIONS, INVALID_REPLICA_ASSIGNMENT, REQUEST_TIMED_OUT and NOT_CONTROLLER map exhaustively; THROTTLING_QUOTA_EXCEEDED only for v3; table test.

### [To Do] API-0468 CreatePartitions throttle_time_ms through the shared response path
P0 · M2 · Task · deps: API-0463 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0469 CreatePartitions throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0468, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: CreatePartitions records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v1+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0470 CreatePartitions request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0463 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for CreatePartitions; the request-log summary (SRV-F09) shows topic count, target counts and validate_only. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0471 CreatePartitions handler unit tests across v0-3
P0 · M2 · Task · deps: API-0464, API-0466, API-0467, API-0468 · labels: test
Done when: Table-driven tests send every version v0-v3 through the handler, with the controller partition-creation path (CTRL-F06) replaced by a fake, and check the service call and the encoded response; cases cover increase with and without assignments, decrease, duplicates, unknown topics and validate_only.

### [To Do] API-0472 CreatePartitions compatibility test with Java client 4.4.0 Admin.createPartitions, librdkafka, franz-go and kafka-topics.sh --alter --partitions
P0 · M2 · Task · deps: API-0471, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that grows partition counts and verifies producers start using the new partitions passes with Java client 4.4.0 Admin.createPartitions, librdkafka, franz-go and kafka-topics.sh --alter --partitions.

### [To Do] API-0473 CreatePartitions differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0471, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical CreatePartitions requests to Kafka 4.4.0 and Alluvium for growth with and without assignments and for each client-triggerable error (duplicates, UNKNOWN_TOPIC_OR_PARTITION, INVALID_PARTITIONS for a decrease and INVALID_REPLICA_ASSIGNMENT); normalized responses match for every supported version.

### [To Do] API-0474 Advertise CreatePartitions v0-3 in ApiVersions
P0 · M2 · Task · deps: API-0470, API-0472, API-0473, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists CreatePartitions with min 0 and max 3 through the generated list owned by API-F18; docs/parity.md marks CreatePartitions supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0475 CreatePartitions authorization: ALTER on each topic
P0 · M4 · Task · deps: API-0464, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala createPartitions @4.4.0
Done when: Topics without ALTER on TOPIC get TOPIC_AUTHORIZATION_FAILED while others proceed; tests and differential cases.

### [To Do] API-0476 CreatePartitions v3: controller mutation quota and THROTTLING_QUOTA_EXCEEDED (KIP-599)
P0 · M4 · Task · deps: API-0469, QUOTA-F05 · ref: clients/src/main/resources/common/message/CreatePartitionsRequest.json @4.4.0 (KIP-599)
Done when: New partitions are charged to the controller mutation quota; v3 topics over quota fail with THROTTLING_QUOTA_EXCEEDED while v0-v2 are only throttled; tests per version.

## [To Do] API-F38 CreateDelegationToken (v1-3)
P1 · M4 · Feature · deps: SEC-F10
Scope: Kafka 4.4.0 versions 1-3; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0477 CreateDelegationToken handler module and router registration (broker and controller listeners)
P1 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleCreateTokenRequest @4.4.0
Done when: `handlers/create_delegation_token.rs` (module doc names the Kafka counterpart) decodes v1-3 with the generated codec and calls the delegation token service (SEC-F10); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0478 CreateDelegationToken connection and renewer checks
P1 · M4 · Task · deps: API-0477 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala allowTokenRequests @4.4.0
Done when: Requests on PLAINTEXT listeners, from one-way-TLS (anonymous) connections or from token-authenticated principals get DELEGATION_TOKEN_REQUEST_NOT_ALLOWED before any other check, as `KafkaApis.allowTokenRequests` decides; renewers whose principal type is not User get INVALID_PRINCIPAL_TYPE; responses always fill owner and requester (`CreateDelegationTokenResponse.prepareResponse`); unit tests.

### [To Do] API-0479 Forward CreateDelegationToken from brokers to the active controller in an Envelope
P1 · M4 · Task · deps: API-0478, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener CreateDelegationToken is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Only requests that pass the broker-side checks are forwarded. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0480 CreateDelegationToken v1-v2: renewers and max lifetime
P1 · M4 · Task · deps: API-0479 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleCreateDelegationTokenRequest @4.4.0
Done when: The controller creates the token through SEC-F10 with max_lifetime_ms (non-positive means `delegation.token.max.lifetime.ms`, larger values are capped by it) and returns owner, issue, expiry and max timestamps, token id and HMAC; disabled token auth gives DELEGATION_TOKEN_AUTH_DISABLED; v2 accepts tagged fields; tests.

### [To Do] API-0481 CreateDelegationToken v3: owner principal and token requester (KIP-373)
P1 · M4 · Task · deps: API-0480 · ref: clients/src/main/resources/common/message/CreateDelegationTokenRequest.json @4.4.0 (KIP-373)
Done when: v3 may name an owner other than the requester; responses carry token_requester_principal_type and name; below v3 the owner is always the requester; per-version tests.

### [To Do] API-0482 CreateDelegationToken error mapping
P1 · M4 · Task · deps: API-0477 · ref: clients/src/main/java/org/apache/kafka/common/requests/CreateDelegationTokenResponse.java prepareResponse @4.4.0
Done when: DELEGATION_TOKEN_REQUEST_NOT_ALLOWED, DELEGATION_TOKEN_AUTHORIZATION_FAILED, DELEGATION_TOKEN_AUTH_DISABLED, INVALID_PRINCIPAL_TYPE and NOT_CONTROLLER map exhaustively; table test.

### [To Do] API-0483 CreateDelegationToken throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0477, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; CreateDelegationToken records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal.

### [To Do] API-0484 CreateDelegationToken request metrics, events and request-log summary
P1 · M4 · Task · deps: API-0477 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for CreateDelegationToken; the request-log summary (SRV-F09) shows owner, renewer count and requested max lifetime, never the HMAC. Token creation emits an event with reason `create_delegation_token_request` carrying only the token id. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0485 CreateDelegationToken authorization: CREATE_TOKENS on the owner USER
P1 · M4 · Task · deps: API-0481, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleCreateTokenRequest @4.4.0
Done when: When the owner differs from the requester, the requester needs CREATE_TOKENS on USER:<owner> or gets DELEGATION_TOKEN_AUTHORIZATION_FAILED; the controller repeats the check for forwarded requests; tests and a differential case.

### [To Do] API-0486 CreateDelegationToken handler unit tests across v1-3
P1 · M4 · Task · deps: API-0482, API-0483, API-0485 · labels: test
Done when: Table-driven tests send every version v1-v3 through the handler, with the delegation token service (SEC-F10) replaced by a fake, and check the service call and the encoded response; cases cover self-owned and delegated owners, renewer types, disabled token auth, PLAINTEXT and token-authenticated connections.

### [To Do] API-0487 CreateDelegationToken compatibility test with Java client 4.4.0 Admin.createDelegationToken and kafka-delegation-tokens.sh --create
P1 · M4 · Task · deps: API-0486, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that creates a token, authenticates a client with it over SCRAM and is refused on a PLAINTEXT listener passes with Java client 4.4.0 Admin.createDelegationToken and kafka-delegation-tokens.sh --create.

### [To Do] API-0488 CreateDelegationToken differential test against Kafka 4.4.0
P1 · M4 · Task · deps: API-0486, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical CreateDelegationToken requests to Kafka 4.4.0 and Alluvium for token creation at v1-v3 (ignoring token ids, HMACs and timestamps) and for each client-triggerable error (DELEGATION_TOKEN_REQUEST_NOT_ALLOWED, DELEGATION_TOKEN_AUTHORIZATION_FAILED and INVALID_PRINCIPAL_TYPE); normalized responses match for every supported version.

### [To Do] API-0489 Advertise CreateDelegationToken v1-3 in ApiVersions
P1 · M4 · Task · deps: API-0484, API-0487, API-0488, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists CreateDelegationToken with min 1 and max 3 through the generated list owned by API-F18; docs/parity.md marks CreateDelegationToken supported; a test asserts the range.

## [To Do] API-F39 RenewDelegationToken (v1-2)
P1 · M4 · Feature · deps: SEC-F10
Scope: Kafka 4.4.0 versions 1-2; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0490 RenewDelegationToken handler module and router registration (broker and controller listeners)
P1 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleRenewTokenRequest @4.4.0
Done when: `handlers/renew_delegation_token.rs` (module doc names the Kafka counterpart) decodes v1-2 with the generated codec and calls the delegation token service (SEC-F10); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0491 RenewDelegationToken connection checks
P1 · M4 · Task · deps: API-0490 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala allowTokenRequests @4.4.0
Done when: Requests on PLAINTEXT listeners, from one-way-TLS (anonymous) connections or from token-authenticated principals get DELEGATION_TOKEN_REQUEST_NOT_ALLOWED before any other check, as `KafkaApis.allowTokenRequests` decides; unit tests.

### [To Do] API-0492 Forward RenewDelegationToken from brokers to the active controller in an Envelope
P1 · M4 · Task · deps: API-0491, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener RenewDelegationToken is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Only requests that pass the broker-side checks are forwarded. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0493 RenewDelegationToken v1: extend expiry by HMAC
P1 · M4 · Task · deps: API-0492 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleRenewDelegationTokenRequest @4.4.0
Done when: The controller finds the token by HMAC and extends its expiry by renew_period_ms (negative uses `delegation.token.expiry.time.ms`), capped at the token's max timestamp; only the owner or a renewer may renew; the new expiry_timestamp_ms is returned; tests.

### [To Do] API-0494 RenewDelegationToken v2: flexible encoding
P1 · M4 · Task · deps: API-0493 · ref: clients/src/main/resources/common/message/RenewDelegationTokenRequest.json @4.4.0
Done when: v2 requests and responses accept tagged fields with the same semantics as v1; round-trip tests at v1 and v2.

### [To Do] API-0495 RenewDelegationToken error mapping
P1 · M4 · Task · deps: API-0490 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleRenewDelegationTokenRequest @4.4.0
Done when: DELEGATION_TOKEN_REQUEST_NOT_ALLOWED, DELEGATION_TOKEN_NOT_FOUND, DELEGATION_TOKEN_OWNER_MISMATCH, DELEGATION_TOKEN_EXPIRED, DELEGATION_TOKEN_AUTH_DISABLED and NOT_CONTROLLER map exhaustively; table test.

### [To Do] API-0496 RenewDelegationToken throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0490, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; RenewDelegationToken records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal.

### [To Do] API-0497 RenewDelegationToken request metrics, events and request-log summary
P1 · M4 · Task · deps: API-0490 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for RenewDelegationToken; the request-log summary (SRV-F09) shows the token id resolved from the HMAC and the period, never the HMAC itself. Each renewal emits an event with reason `renew_delegation_token_request`. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0498 RenewDelegationToken handler unit tests across v1-2
P1 · M4 · Task · deps: API-0494, API-0495, API-0496 · labels: test
Done when: Table-driven tests send every version v1-v2 through the handler, with the delegation token service (SEC-F10) replaced by a fake, and check the service call and the encoded response; cases cover owner and renewer callers, unknown HMACs, expired tokens and disallowed connections.

### [To Do] API-0499 RenewDelegationToken compatibility test with Java client 4.4.0 Admin.renewDelegationToken and kafka-delegation-tokens.sh --renew
P1 · M4 · Task · deps: API-0498, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that renews a token and keeps using it past its original expiry passes with Java client 4.4.0 Admin.renewDelegationToken and kafka-delegation-tokens.sh --renew.

### [To Do] API-0500 RenewDelegationToken differential test against Kafka 4.4.0
P1 · M4 · Task · deps: API-0498, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical RenewDelegationToken requests to Kafka 4.4.0 and Alluvium for renewal by owner and renewer (ignoring timestamps) and for each client-triggerable error (DELEGATION_TOKEN_NOT_FOUND, DELEGATION_TOKEN_OWNER_MISMATCH and DELEGATION_TOKEN_EXPIRED); normalized responses match for every supported version.

### [To Do] API-0501 Advertise RenewDelegationToken v1-2 in ApiVersions
P1 · M4 · Task · deps: API-0497, API-0499, API-0500, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists RenewDelegationToken with min 1 and max 2 through the generated list owned by API-F18; docs/parity.md marks RenewDelegationToken supported; a test asserts the range.

## [To Do] API-F40 ExpireDelegationToken (v1-2)
P1 · M4 · Feature · deps: SEC-F10
Scope: Kafka 4.4.0 versions 1-2; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0502 ExpireDelegationToken handler module and router registration (broker and controller listeners)
P1 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleExpireTokenRequest @4.4.0
Done when: `handlers/expire_delegation_token.rs` (module doc names the Kafka counterpart) decodes v1-2 with the generated codec and calls the delegation token service (SEC-F10); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0503 ExpireDelegationToken connection checks
P1 · M4 · Task · deps: API-0502 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala allowTokenRequests @4.4.0
Done when: Requests on PLAINTEXT listeners, from one-way-TLS (anonymous) connections or from token-authenticated principals get DELEGATION_TOKEN_REQUEST_NOT_ALLOWED before any other check, as `KafkaApis.allowTokenRequests` decides; unit tests.

### [To Do] API-0504 Forward ExpireDelegationToken from brokers to the active controller in an Envelope
P1 · M4 · Task · deps: API-0503, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener ExpireDelegationToken is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Only requests that pass the broker-side checks are forwarded. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0505 ExpireDelegationToken v1: expire now or later by HMAC
P1 · M4 · Task · deps: API-0504 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleExpireDelegationTokenRequest @4.4.0
Done when: The controller finds the token by HMAC and sets its expiry to now plus expiry_time_period_ms; a negative period removes it immediately; only the owner or a renewer may expire it; tests.

### [To Do] API-0506 ExpireDelegationToken v2: flexible encoding
P1 · M4 · Task · deps: API-0505 · ref: clients/src/main/resources/common/message/ExpireDelegationTokenRequest.json @4.4.0
Done when: v2 requests and responses accept tagged fields with the same semantics as v1; round-trip tests at v1 and v2.

### [To Do] API-0507 ExpireDelegationToken error mapping
P1 · M4 · Task · deps: API-0502 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleExpireDelegationTokenRequest @4.4.0
Done when: DELEGATION_TOKEN_REQUEST_NOT_ALLOWED, DELEGATION_TOKEN_NOT_FOUND, DELEGATION_TOKEN_OWNER_MISMATCH, DELEGATION_TOKEN_EXPIRED, DELEGATION_TOKEN_AUTH_DISABLED and NOT_CONTROLLER map exhaustively; table test.

### [To Do] API-0508 ExpireDelegationToken throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0502, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; ExpireDelegationToken records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal.

### [To Do] API-0509 ExpireDelegationToken request metrics, events and request-log summary
P1 · M4 · Task · deps: API-0502 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ExpireDelegationToken; the request-log summary (SRV-F09) shows the token id resolved from the HMAC and the period, never the HMAC itself. Each expiry emits an event with reason `expire_delegation_token_request`. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0510 ExpireDelegationToken handler unit tests across v1-2
P1 · M4 · Task · deps: API-0506, API-0507, API-0508 · labels: test
Done when: Table-driven tests send every version v1-v2 through the handler, with the delegation token service (SEC-F10) replaced by a fake, and check the service call and the encoded response; cases cover owner and renewer callers, unknown HMACs, expired tokens and disallowed connections.

### [To Do] API-0511 ExpireDelegationToken compatibility test with Java client 4.4.0 Admin.expireDelegationToken and kafka-delegation-tokens.sh --expire
P1 · M4 · Task · deps: API-0510, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that expires a token and checks clients using it are rejected passes with Java client 4.4.0 Admin.expireDelegationToken and kafka-delegation-tokens.sh --expire.

### [To Do] API-0512 ExpireDelegationToken differential test against Kafka 4.4.0
P1 · M4 · Task · deps: API-0510, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ExpireDelegationToken requests to Kafka 4.4.0 and Alluvium for expiry by owner and renewer (ignoring timestamps) and for each client-triggerable error (DELEGATION_TOKEN_NOT_FOUND, DELEGATION_TOKEN_OWNER_MISMATCH and DELEGATION_TOKEN_EXPIRED); normalized responses match for every supported version.

### [To Do] API-0513 Advertise ExpireDelegationToken v1-2 in ApiVersions
P1 · M4 · Task · deps: API-0509, API-0511, API-0512, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists ExpireDelegationToken with min 1 and max 2 through the generated list owned by API-F18; docs/parity.md marks ExpireDelegationToken supported; a test asserts the range.

## [To Do] API-F41 DescribeDelegationToken (v1-3)
P1 · M4 · Feature · deps: SEC-F10
Scope: Kafka 4.4.0 versions 1-3; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0514 DescribeDelegationToken handler module and router registration (broker and controller listeners)
P1 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeTokensRequest @4.4.0
Done when: `handlers/describe_delegation_token.rs` (module doc names the Kafka counterpart) decodes v1-3 with the generated codec and calls the delegation token service (SEC-F10); it is registered for the broker and controller listeners in the router (SRV-F04). Brokers answer from their token cache; nothing is forwarded. Unit test per listener.

### [To Do] API-0515 DescribeDelegationToken connection checks and disabled token auth
P1 · M4 · Task · deps: API-0514 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeTokensRequest @4.4.0
Done when: Requests on PLAINTEXT listeners, from one-way-TLS (anonymous) connections or from token-authenticated principals get DELEGATION_TOKEN_REQUEST_NOT_ALLOWED before any other check, as `KafkaApis.allowTokenRequests` decides; with token authentication disabled the response is DELEGATION_TOKEN_AUTH_DISABLED; an empty owners list returns no tokens and a null list every visible token; unit tests.

### [To Do] API-0516 DescribeDelegationToken v1-v2: token details with renewers
P1 · M4 · Task · deps: API-0515 · labels: security · ref: clients/src/main/resources/common/message/DescribeDelegationTokenResponse.json @4.4.0
Done when: Each token returns owner, issue, expiry and max timestamps, token id, HMAC and renewers; v2 accepts tagged fields; tests.

### [To Do] API-0517 DescribeDelegationToken v3: token requester (KIP-373)
P1 · M4 · Task · deps: API-0516 · ref: clients/src/main/resources/common/message/DescribeDelegationTokenResponse.json @4.4.0 (KIP-373)
Done when: v3 tokens carry token_requester_principal_type and name; below v3 they are omitted; per-version tests.

### [To Do] API-0518 DescribeDelegationToken error mapping
P1 · M4 · Task · deps: API-0514 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeTokensRequest @4.4.0
Done when: DELEGATION_TOKEN_REQUEST_NOT_ALLOWED and DELEGATION_TOKEN_AUTH_DISABLED map exhaustively at the top level; table test.

### [To Do] API-0519 DescribeDelegationToken throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0514, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; DescribeDelegationToken records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0520 DescribeDelegationToken request metrics, events and request-log summary
P1 · M4 · Task · deps: API-0514 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeDelegationToken; the request-log summary (SRV-F09) shows the owner filter size and the number of tokens returned, never HMACs. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0521 DescribeDelegationToken authorization: token visibility filtering
P1 · M4 · Task · deps: API-0517, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeTokensRequest @4.4.0
Done when: A token is returned when the caller is its owner or a renewer, has DESCRIBE on DELEGATION_TOKEN:<token id>, or has DESCRIBE_TOKENS on USER:<owner>, as `DelegationTokenManager.filterToken` decides; other tokens are silently omitted; tests and a differential case.

### [To Do] API-0522 DescribeDelegationToken handler unit tests across v1-3
P1 · M4 · Task · deps: API-0518, API-0519, API-0521 · labels: test
Done when: Table-driven tests send every version v1-v3 through the handler, with the delegation token service (SEC-F10) replaced by a fake, and check the service call and the encoded response; cases cover all owners, selected owners, renewers, requesters and disallowed connections.

### [To Do] API-0523 DescribeDelegationToken compatibility test with Java client 4.4.0 Admin.describeDelegationToken and kafka-delegation-tokens.sh --describe
P1 · M4 · Task · deps: API-0522, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that describes own, renewable and other users' tokens passes with Java client 4.4.0 Admin.describeDelegationToken and kafka-delegation-tokens.sh --describe.

### [To Do] API-0524 DescribeDelegationToken differential test against Kafka 4.4.0
P1 · M4 · Task · deps: API-0522, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeDelegationToken requests to Kafka 4.4.0 and Alluvium for descriptions with each owner filter (ignoring HMACs and timestamps) and for each client-triggerable error (DELEGATION_TOKEN_REQUEST_NOT_ALLOWED and DELEGATION_TOKEN_AUTH_DISABLED); normalized responses match for every supported version.

### [To Do] API-0525 Advertise DescribeDelegationToken v1-3 in ApiVersions
P1 · M4 · Task · deps: API-0520, API-0523, API-0524, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists DescribeDelegationToken with min 1 and max 3 through the generated list owned by API-F18; docs/parity.md marks DescribeDelegationToken supported; a test asserts the range.

## [To Do] API-F42 DeleteGroups (v0-3)
P1 · M5 · Feature · deps: GRP-F08
Scope: Kafka 4.4.0 versions 0-3; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0526 DeleteGroups handler module and router registration (broker listener)
P1 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteGroupsRequest @4.4.0
Done when: `handlers/delete_groups.rs` (module doc names the Kafka counterpart) decodes v0-3 with the generated codec and calls the group administration service (GRP-F08); it is registered for the broker listener in the router (SRV-F04). A DeleteGroups request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0527 DeleteGroups v0-v2: per-group results with duplicates collapsed
P1 · M5 · Task · deps: API-0526 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteGroupsRequest @4.4.0
Done when: Group ids are de-duplicated before deletion; each group returns NONE, NON_EMPTY_GROUP, GROUP_ID_NOT_FOUND or a coordinator error, and a coordinator-wide failure is copied to every authorized group; v2 accepts tagged fields; per-version tests.

### [To Do] API-0528 DeleteGroups v3: error_message and GROUP_DELETION_FAILED
P1 · M5 · Task · deps: API-0527 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteGroupsRequest @4.4.0
Done when: v3 results carry error_message and may return GROUP_DELETION_FAILED; for v0-v2 GROUP_DELETION_FAILED is downgraded to UNKNOWN_SERVER_ERROR and the message dropped; tests per version.

### [To Do] API-0529 DeleteGroups error mapping per version
P1 · M5 · Task · deps: API-0526 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteGroupsRequest @4.4.0
Done when: NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, NON_EMPTY_GROUP, GROUP_ID_NOT_FOUND, INVALID_GROUP_ID, GROUP_AUTHORIZATION_FAILED and GROUP_DELETION_FAILED (v3 only) map exhaustively; table test.

### [To Do] API-0530 DeleteGroups throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M5 · Task · deps: API-0526, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; DeleteGroups records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel; only v1+ clients throttle themselves (`shouldClientThrottle`), older ones rely on channel muting. Test with a quota-limited principal.

### [To Do] API-0531 DeleteGroups request metrics, events and request-log summary
P1 · M5 · Task · deps: API-0526 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DeleteGroups; the request-log summary (SRV-F09) shows group count and per-group outcome. Group deletions emit an event with reason `delete_groups_request` (GRP-F08). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0532 DeleteGroups authorization: DELETE on each group
P1 · M5 · Task · deps: API-0526, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteGroupsRequest @4.4.0
Done when: Groups without DELETE on GROUP get GROUP_AUTHORIZATION_FAILED while the rest are deleted; tests and differential cases.

### [To Do] API-0533 DeleteGroups handler unit tests across v0-3
P1 · M5 · Task · deps: API-0528, API-0529, API-0530, API-0532 · labels: test
Done when: Table-driven tests send every version v0-v3 through the handler, with the group administration service (GRP-F08) replaced by a fake, and check the service call and the encoded response; cases cover empty, non-empty, unknown and duplicate groups and the v3 downgrade.

### [To Do] API-0534 DeleteGroups compatibility test with Java client 4.4.0 Admin.deleteConsumerGroups and kafka-consumer-groups.sh --delete
P1 · M5 · Task · deps: API-0533, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that deletes inactive groups and refuses active ones passes with Java client 4.4.0 Admin.deleteConsumerGroups and kafka-consumer-groups.sh --delete.

### [To Do] API-0535 DeleteGroups differential test against Kafka 4.4.0
P1 · M5 · Task · deps: API-0533, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DeleteGroups requests to Kafka 4.4.0 and Alluvium for deletions at v0-v3 and for each client-triggerable error (NON_EMPTY_GROUP, GROUP_ID_NOT_FOUND and INVALID_GROUP_ID); normalized responses match for every supported version.

### [To Do] API-0536 Advertise DeleteGroups v0-3 in ApiVersions
P1 · M5 · Task · deps: API-0531, API-0534, API-0535, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists DeleteGroups with min 0 and max 3 through the generated list owned by API-F18; docs/parity.md marks DeleteGroups supported; a test asserts the range.

## [To Do] API-F43 ElectLeaders (v0-2)
P1 · M3 · Feature · deps: CTRL-F08
Scope: Kafka 4.4.0 versions 0-2; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0537 ElectLeaders handler module and router registration (broker and controller listeners)
P1 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleElectLeaders @4.4.0
Done when: `handlers/elect_leaders.rs` (module doc names the Kafka counterpart) decodes v0-2 with the generated codec and calls the controller election path (CTRL-F08); it is registered for the broker and controller listeners in the router (SRV-F04). The controller context deadline comes from timeout_ms. Unit test per listener.

### [To Do] API-0538 Forward ElectLeaders from brokers to the active controller in an Envelope
P1 · M3 · Task · deps: API-0537, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener ElectLeaders is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0539 ElectLeaders request validation
P1 · M3 · Task · deps: API-0537 · labels: security · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java electLeaders @4.4.0
Done when: An unknown election_type fails the whole request with INVALID_REQUEST, copied onto every listed partition; a null topic_partitions list targets every partition and omits ELECTION_NOT_NEEDED results; unknown topics or partitions get UNKNOWN_TOPIC_OR_PARTITION; unit tests.

### [To Do] API-0540 ElectLeaders v0: preferred elections only, errors per partition
P1 · M3 · Task · deps: API-0539 · ref: clients/src/main/java/org/apache/kafka/common/requests/ElectLeadersRequest.java @4.4.0
Done when: v0 requests have no election_type and always run a preferred election; v0 responses have no top-level error_code, so request-level failures appear on every partition (`ElectLeadersRequest.getErrorResponse`); tests.

### [To Do] API-0541 ElectLeaders v1-v2: election types (KIP-460) and top-level error_code
P1 · M3 · Task · deps: API-0539 · ref: clients/src/main/resources/common/message/ElectLeadersRequest.json @4.4.0 (KIP-460)
Done when: v1+ election_type selects PREFERRED (0) or UNCLEAN (1); the top-level error_code carries request-level failures while ELECTION_NOT_NEEDED, PREFERRED_LEADER_NOT_AVAILABLE and ELIGIBLE_LEADERS_NOT_AVAILABLE stay per partition; v2 is flexible; per-version tests.

### [To Do] API-0542 ElectLeaders error mapping per version
P1 · M3 · Task · deps: API-0537 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleElectLeaders @4.4.0
Done when: UNKNOWN_TOPIC_OR_PARTITION, ELECTION_NOT_NEEDED, PREFERRED_LEADER_NOT_AVAILABLE, ELIGIBLE_LEADERS_NOT_AVAILABLE, INVALID_REQUEST, REQUEST_TIMED_OUT and NOT_CONTROLLER map exhaustively with messages; table test per version.

### [To Do] API-0543 ElectLeaders throttle_time_ms through the shared response path
P1 · M3 · Task · deps: API-0537 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0544 ElectLeaders throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0543, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: ElectLeaders records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0545 ElectLeaders request metrics, events and request-log summary
P1 · M3 · Task · deps: API-0537 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ElectLeaders; the request-log summary (SRV-F09) shows election type, partition count or all-partitions flag and timeout_ms. Leader changes caused by the request carry reason `elect_leaders_request` (CTRL-F08). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0546 ElectLeaders handler unit tests across v0-2
P1 · M3 · Task · deps: API-0538, API-0540, API-0541, API-0542, API-0543 · labels: test
Done when: Table-driven tests send every version v0-v2 through the handler, with the controller election path (CTRL-F08) replaced by a fake, and check the service call and the encoded response; cases cover preferred and unclean elections, all-partitions requests, unknown partitions and v0 error placement.

### [To Do] API-0547 ElectLeaders compatibility test with Java client 4.4.0 Admin.electLeaders and kafka-leader-election.sh
P1 · M3 · Task · deps: API-0546, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that runs preferred elections for one topic and with --all-topic-partitions, and an unclean election after stopping the ISR, passes with Java client 4.4.0 Admin.electLeaders and kafka-leader-election.sh.

### [To Do] API-0548 ElectLeaders multi-node simulation test
P1 · M3 · Task · deps: API-0546, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) moves leadership away from preferred replicas, runs ElectLeaders through brokers during controller failover and checks every eligible partition ends on its preferred leader; a failing seed is printed and replays exactly.

### [To Do] API-0549 ElectLeaders differential test against Kafka 4.4.0
P1 · M3 · Task · deps: API-0546, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ElectLeaders requests to Kafka 4.4.0 and Alluvium for preferred and unclean elections at v0-v2 and for each client-triggerable error (ELECTION_NOT_NEEDED, UNKNOWN_TOPIC_OR_PARTITION, PREFERRED_LEADER_NOT_AVAILABLE and an unknown election type); normalized responses match for every supported version.

### [To Do] API-0550 Advertise ElectLeaders v0-2 in ApiVersions
P1 · M3 · Task · deps: API-0545, API-0547, API-0548, API-0549, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists ElectLeaders with min 0 and max 2 through the generated list owned by API-F18; docs/parity.md marks ElectLeaders supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0551 ElectLeaders authorization: ALTER on CLUSTER
P1 · M4 · Task · deps: API-0538, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleElectLeaders @4.4.0
Done when: Requests without ALTER on CLUSTER get CLUSTER_AUTHORIZATION_FAILED on every partition (and top-level from v1); tests and a differential case.

## [To Do] API-F44 IncrementalAlterConfigs (v0-1)
P0 · M7 · Feature · deps: CFG-F04, CTRL-F11
Scope: Kafka 4.4.0 versions 0-1; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0552 IncrementalAlterConfigs handler module and router registration (broker and controller listeners)
P0 · M7 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleIncrementalAlterConfigsRequest @4.4.0
Done when: `handlers/incremental_alter_configs.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the config change path (CTRL-F11, CFG-F04); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0553 IncrementalAlterConfigs broker-side preprocessing and BROKER_LOGGER
P0 · M7 · Task · deps: API-0552, OBS-F02 · labels: security · ref: core/src/main/scala/kafka/server/ConfigAdminManager.scala preprocess @4.4.0
Done when: Duplicate resources get INVALID_REQUEST; BROKER_LOGGER changes for this broker are applied locally through the runtime log-level control (OBS-F02) with SET and DELETE only, and other resources are forwarded; responses are reassembled in request order (`ConfigAdminManager.reassembleIncrementalResponse`); unit tests.

### [To Do] API-0554 Forward IncrementalAlterConfigs from brokers to the active controller in an Envelope
P0 · M7 · Task · deps: API-0553, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener IncrementalAlterConfigs is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0555 IncrementalAlterConfigs v0-v1: SET, DELETE, APPEND and SUBTRACT
P0 · M7 · Task · deps: API-0554 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleIncrementalAlterConfigs @4.4.0 (KIP-339)
Done when: Each config op is applied incrementally (APPEND and SUBTRACT only for list-type keys, else INVALID_CONFIG); validate_only validates without writing; v1 accepts tagged fields; tests per operation.

### [To Do] API-0556 IncrementalAlterConfigs error mapping
P0 · M7 · Task · deps: API-0552 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleIncrementalAlterConfigs @4.4.0
Done when: INVALID_REQUEST, INVALID_CONFIG, POLICY_VIOLATION, UNKNOWN_TOPIC_OR_PARTITION, UNSUPPORTED_VERSION and the authorization errors map exhaustively per resource with messages; table test.

### [To Do] API-0557 IncrementalAlterConfigs throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M7 · Task · deps: API-0552, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; IncrementalAlterConfigs records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal.

### [To Do] API-0558 IncrementalAlterConfigs request metrics, events and request-log summary
P0 · M7 · Task · deps: API-0552 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for IncrementalAlterConfigs; the request-log summary (SRV-F09) shows resource types and names, op counts and validate_only, never sensitive values. Config changes carry reason `incremental_alter_configs_request` (CTRL-F11). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0559 IncrementalAlterConfigs authorization: ALTER_CONFIGS per resource
P0 · M7 · Task · deps: API-0553, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala authorizeAlterResource @4.4.0
Done when: BROKER and BROKER_LOGGER resources need ALTER_CONFIGS on CLUSTER, TOPIC and GROUP resources on the resource; denied resources get the matching *_AUTHORIZATION_FAILED; tests and differential cases.

### [To Do] API-0560 IncrementalAlterConfigs handler unit tests across v0-1
P0 · M7 · Task · deps: API-0555, API-0556, API-0557, API-0559 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the config change path (CTRL-F11, CFG-F04) replaced by a fake, and check the service call and the encoded response; cases cover each op type on topic, broker, cluster-default, broker-logger, group and client-metrics resources, duplicates and validate_only.

### [To Do] API-0561 IncrementalAlterConfigs compatibility test with Java client 4.4.0 Admin.incrementalAlterConfigs, librdkafka and kafka-configs.sh --alter --add-config/--delete-config
P0 · M7 · Task · deps: API-0560, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that changes topic, broker, default and logger configs and verifies them with DescribeConfigs passes with Java client 4.4.0 Admin.incrementalAlterConfigs, librdkafka and kafka-configs.sh --alter --add-config/--delete-config.

### [To Do] API-0562 IncrementalAlterConfigs differential test against Kafka 4.4.0
P0 · M7 · Task · deps: API-0560, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical IncrementalAlterConfigs requests to Kafka 4.4.0 and Alluvium for each op type with and without validate_only and for each client-triggerable error (duplicate resources, INVALID_CONFIG for APPEND on non-list keys, POLICY_VIOLATION and unknown topics); normalized responses match for every supported version.

### [To Do] API-0563 Advertise IncrementalAlterConfigs v0-1 in ApiVersions
P0 · M7 · Task · deps: API-0558, API-0561, API-0562, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists IncrementalAlterConfigs with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks IncrementalAlterConfigs supported; a test asserts the range. Later tasks (M8) add behavior without changing the advertised range.

### [To Do] API-0564 IncrementalAlterConfigs for GROUP and CLIENT_METRICS resources
P0 · M8 · Task · deps: API-0555, TELEM-F03 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleIncrementalAlterConfigs @4.4.0
Done when: GROUP configs (share, streams and consumer group keys) and CLIENT_METRICS subscriptions are validated by CTRL-F11 with the group and telemetry key definitions; differential test.

## [To Do] API-F45 AlterPartitionReassignments (v0-1)
P1 · M3 · Feature · deps: CTRL-F10
Scope: Kafka 4.4.0 versions 0-1; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0565 AlterPartitionReassignments handler module and router registration (broker and controller listeners)
P1 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAlterPartitionReassignments @4.4.0
Done when: `handlers/alter_partition_reassignments.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the controller reassignment path (CTRL-F10); it is registered for the broker and controller listeners in the router (SRV-F04). The controller context deadline comes from timeout_ms. Unit test per listener.

### [To Do] API-0566 Forward AlterPartitionReassignments from brokers to the active controller in an Envelope
P1 · M3 · Task · deps: API-0565, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener AlterPartitionReassignments is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0567 AlterPartitionReassignments request validation
P1 · M3 · Task · deps: API-0565 · labels: security · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java alterPartitionReassignment @4.4.0
Done when: Target lists with duplicates, unknown brokers or no replicas get INVALID_REPLICA_ASSIGNMENT per partition; a null replicas list cancels and returns NO_REASSIGNMENT_IN_PROGRESS when none exists; unknown topics or partitions get UNKNOWN_TOPIC_OR_PARTITION; unit tests.

### [To Do] API-0568 AlterPartitionReassignments v0: start, change and cancel reassignments
P1 · M3 · Task · deps: API-0567 · ref: clients/src/main/resources/common/message/AlterPartitionReassignmentsRequest.json @4.4.0
Done when: v0 starts or changes reassignments for listed partitions and cancels those with null replicas; partition results carry error_code and error_message and the top-level error stays NONE unless the whole request fails; tests.

### [To Do] API-0569 AlterPartitionReassignments v1: allow_replication_factor_change (KIP-860)
P1 · M3 · Task · deps: API-0568 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java validatePartitionReplicationFactorUnchanged @4.4.0 (KIP-860)
Done when: With allow_replication_factor_change false (v1+), a target whose size differs from the current (or in-flight) replica set fails that partition with INVALID_REPLICATION_FACTOR 'The replication factor is changed from X to Y'; v0 behaves as true; the response echoes the flag; tests per version.

### [To Do] API-0570 AlterPartitionReassignments error mapping
P1 · M3 · Task · deps: API-0565 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAlterPartitionReassignments @4.4.0
Done when: INVALID_REPLICA_ASSIGNMENT, INVALID_REPLICATION_FACTOR, NO_REASSIGNMENT_IN_PROGRESS, UNKNOWN_TOPIC_OR_PARTITION, REQUEST_TIMED_OUT and NOT_CONTROLLER map exhaustively at partition or top level; table test.

### [To Do] API-0571 AlterPartitionReassignments throttle_time_ms through the shared response path
P1 · M3 · Task · deps: API-0565 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0572 AlterPartitionReassignments throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0571, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: AlterPartitionReassignments records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0573 AlterPartitionReassignments request metrics, events and request-log summary
P1 · M3 · Task · deps: API-0565 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for AlterPartitionReassignments; the request-log summary (SRV-F09) shows topic and partition counts, cancellation count and allow_replication_factor_change. Reassignment starts and cancellations carry reason `alter_partition_reassignments_request` (CTRL-F10). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0574 AlterPartitionReassignments handler unit tests across v0-1
P1 · M3 · Task · deps: API-0566, API-0569, API-0570, API-0571 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the controller reassignment path (CTRL-F10) replaced by a fake, and check the service call and the encoded response; cases cover start, change, cancel, invalid assignments, replication-factor changes and timeouts.

### [To Do] API-0575 AlterPartitionReassignments compatibility test with Java client 4.4.0 Admin.alterPartitionReassignments and kafka-reassign-partitions.sh --execute and --cancel
P1 · M3 · Task · deps: API-0574, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that moves replicas between brokers with throttles and cancels an in-flight move passes with Java client 4.4.0 Admin.alterPartitionReassignments and kafka-reassign-partitions.sh --execute and --cancel.

### [To Do] API-0576 AlterPartitionReassignments multi-node simulation test
P1 · M3 · Task · deps: API-0574, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) reassigns partitions under broker restarts and controller failover and checks each reassignment completes or cancels with the ISR never below the plan's minimum; a failing seed is printed and replays exactly.

### [To Do] API-0577 AlterPartitionReassignments differential test against Kafka 4.4.0
P1 · M3 · Task · deps: API-0574, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical AlterPartitionReassignments requests to Kafka 4.4.0 and Alluvium for starting and cancelling reassignments at v0-v1 and for each client-triggerable error (INVALID_REPLICA_ASSIGNMENT, NO_REASSIGNMENT_IN_PROGRESS, UNKNOWN_TOPIC_OR_PARTITION and INVALID_REPLICATION_FACTOR); normalized responses match for every supported version.

### [To Do] API-0578 Advertise AlterPartitionReassignments v0-1 in ApiVersions
P1 · M3 · Task · deps: API-0573, API-0575, API-0576, API-0577, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists AlterPartitionReassignments with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks AlterPartitionReassignments supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0579 AlterPartitionReassignments authorization: ALTER on CLUSTER
P1 · M4 · Task · deps: API-0566, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAlterPartitionReassignments @4.4.0
Done when: Requests without ALTER on CLUSTER get top-level CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

## [To Do] API-F46 ListPartitionReassignments (v0)
P1 · M3 · Feature · deps: CTRL-F10
Scope: Kafka 4.4.0 versions 0; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0580 ListPartitionReassignments handler module and router registration (broker and controller listeners)
P1 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleListPartitionReassignments @4.4.0
Done when: `handlers/list_partition_reassignments.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the controller reassignment path (CTRL-F10); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0581 Forward ListPartitionReassignments from brokers to the active controller in an Envelope
P1 · M3 · Task · deps: API-0580, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener ListPartitionReassignments is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0582 ListPartitionReassignments v0: all or selected ongoing reassignments
P1 · M3 · Task · deps: API-0580 · ref: clients/src/main/resources/common/message/ListPartitionReassignmentsRequest.json @4.4.0
Done when: A null topics list returns every ongoing reassignment; listed partitions without one, and unknown topics, are omitted; each entry carries replicas, adding_replicas and removing_replicas; tests.

### [To Do] API-0583 ListPartitionReassignments error mapping
P1 · M3 · Task · deps: API-0580 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleListPartitionReassignments @4.4.0
Done when: Top-level NOT_CONTROLLER, REQUEST_TIMED_OUT and CLUSTER_AUTHORIZATION_FAILED map exhaustively with error messages; table test.

### [To Do] API-0584 ListPartitionReassignments throttle_time_ms through the shared response path
P1 · M3 · Task · deps: API-0580 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0585 ListPartitionReassignments throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0584, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: ListPartitionReassignments records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0586 ListPartitionReassignments request metrics, events and request-log summary
P1 · M3 · Task · deps: API-0580 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ListPartitionReassignments; the request-log summary (SRV-F09) shows topic count or all-topics flag and the number of reassignments returned. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0587 ListPartitionReassignments handler unit tests at v0
P1 · M3 · Task · deps: API-0581, API-0582, API-0583, API-0584 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the controller reassignment path (CTRL-F10) replaced by a fake, and check the service call and the encoded response; cases cover no reassignments, all, selected and unknown topics.

### [To Do] API-0588 ListPartitionReassignments compatibility test with Java client 4.4.0 Admin.listPartitionReassignments and kafka-reassign-partitions.sh --verify
P1 · M3 · Task · deps: API-0587, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that lists an in-flight reassignment and then verifies its completion passes with Java client 4.4.0 Admin.listPartitionReassignments and kafka-reassign-partitions.sh --verify.

### [To Do] API-0589 ListPartitionReassignments differential test against Kafka 4.4.0
P1 · M3 · Task · deps: API-0587, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ListPartitionReassignments requests to Kafka 4.4.0 and Alluvium for all and selected listings and for each client-triggerable error (selections that include unknown topics); normalized responses match for every supported version.

### [To Do] API-0590 Advertise ListPartitionReassignments v0 in ApiVersions
P1 · M3 · Task · deps: API-0586, API-0588, API-0589, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists ListPartitionReassignments with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks ListPartitionReassignments supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0591 ListPartitionReassignments authorization: DESCRIBE on CLUSTER
P1 · M4 · Task · deps: API-0581, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleListPartitionReassignments @4.4.0
Done when: Requests without DESCRIBE on CLUSTER get CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

## [To Do] API-F47 OffsetDelete (v0)
P1 · M5 · Feature · deps: GRP-F07
Scope: Kafka 4.4.0 versions 0; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0592 OffsetDelete handler module and router registration (broker listener)
P1 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleOffsetDeleteRequest @4.4.0
Done when: `handlers/offset_delete.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the group coordinator (GRP-F07); it is registered for the broker listener in the router (SRV-F04). A OffsetDelete request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0593 OffsetDelete v0: delete committed offsets for unsubscribed partitions
P1 · M5 · Task · deps: API-0592 · ref: clients/src/main/resources/common/message/OffsetDeleteResponse.json @4.4.0
Done when: Offsets are deleted for listed partitions of topics the group is not subscribed to; subscribed topics get GROUP_SUBSCRIBED_TO_TOPIC per partition and unknown topics or partitions UNKNOWN_TOPIC_OR_PARTITION, while an unknown group gets GROUP_ID_NOT_FOUND and a non-empty classic group NON_EMPTY_GROUP at the top level; tests.

### [To Do] API-0594 OffsetDelete error mapping
P1 · M5 · Task · deps: API-0592 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleOffsetDeleteRequest @4.4.0
Done when: Top-level NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, GROUP_ID_NOT_FOUND, NON_EMPTY_GROUP, INVALID_GROUP_ID and GROUP_AUTHORIZATION_FAILED and partition-level GROUP_SUBSCRIBED_TO_TOPIC, UNKNOWN_TOPIC_OR_PARTITION and TOPIC_AUTHORIZATION_FAILED map exhaustively; table test.

### [To Do] API-0595 OffsetDelete throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M5 · Task · deps: API-0592, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; OffsetDelete records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0596 OffsetDelete request metrics, events and request-log summary
P1 · M5 · Task · deps: API-0592 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for OffsetDelete; the request-log summary (SRV-F09) shows group id and partition count. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0597 OffsetDelete authorization: DELETE on GROUP and READ on topics
P1 · M5 · Task · deps: API-0592, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleOffsetDeleteRequest @4.4.0
Done when: Without DELETE on the GROUP the request fails with GROUP_AUTHORIZATION_FAILED; topics without READ get TOPIC_AUTHORIZATION_FAILED per partition; tests and differential cases.

### [To Do] API-0598 OffsetDelete handler unit tests at v0
P1 · M5 · Task · deps: API-0593, API-0594, API-0595, API-0597 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the group coordinator (GRP-F07) replaced by a fake, and check the service call and the encoded response; cases cover empty and active groups, subscribed and unsubscribed topics and unknown partitions.

### [To Do] API-0599 OffsetDelete compatibility test with Java client 4.4.0 Admin.deleteConsumerGroupOffsets and kafka-consumer-groups.sh --delete-offsets
P1 · M5 · Task · deps: API-0598, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that deletes offsets for an unsubscribed topic passes with Java client 4.4.0 Admin.deleteConsumerGroupOffsets and kafka-consumer-groups.sh --delete-offsets.

### [To Do] API-0600 OffsetDelete differential test against Kafka 4.4.0
P1 · M5 · Task · deps: API-0598, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical OffsetDelete requests to Kafka 4.4.0 and Alluvium for deletions for classic and consumer groups and for each client-triggerable error (GROUP_SUBSCRIBED_TO_TOPIC, NON_EMPTY_GROUP and GROUP_ID_NOT_FOUND); normalized responses match for every supported version.

### [To Do] API-0601 Advertise OffsetDelete v0 in ApiVersions
P1 · M5 · Task · deps: API-0596, API-0599, API-0600, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists OffsetDelete with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks OffsetDelete supported; a test asserts the range.

## [To Do] API-F48 DescribeClientQuotas (v0-1)
P1 · M4 · Feature · deps: QUOTA-F08
Scope: Kafka 4.4.0 versions 0-1; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0602 DescribeClientQuotas handler module and router registration (broker listener)
P1 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeClientQuotasRequest @4.4.0
Done when: `handlers/describe_client_quotas.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the quota view in the metadata image (QUOTA-F08); it is registered for the broker listener in the router (SRV-F04). A DescribeClientQuotas request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0603 DescribeClientQuotas filter validation
P1 · M4 · Task · deps: API-0602 · labels: security · ref: metadata/src/main/java/org/apache/kafka/image/ClientQuotasImage.java describe @4.4.0
Done when: An empty entity type, an entity type listed twice, an EXACT match with a null match string, a DEFAULT or SPECIFIED match with a match string, an unknown match type, or ip combined with user or client-id get top-level INVALID_REQUEST with Kafka's message; an unsupported entity type gets UNSUPPORTED_VERSION; unit tests.

### [To Do] API-0604 DescribeClientQuotas v0-v1: strict matching and entries
P1 · M4 · Task · deps: API-0603 · ref: clients/src/main/resources/common/message/DescribeClientQuotasRequest.json @4.4.0
Done when: With strict set only entities with exactly the filtered entity types match, otherwise entities with extra types match too; each entry lists its entity and quota keys and values; v1 accepts tagged fields; tests.

### [To Do] API-0605 DescribeClientQuotas error mapping
P1 · M4 · Task · deps: API-0602 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeClientQuotasRequest @4.4.0
Done when: INVALID_REQUEST, UNSUPPORTED_VERSION and CLUSTER_AUTHORIZATION_FAILED are top-level with error_message and null entries; table test.

### [To Do] API-0606 DescribeClientQuotas throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0602, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; DescribeClientQuotas records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0607 DescribeClientQuotas request metrics, events and request-log summary
P1 · M4 · Task · deps: API-0602 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeClientQuotas; the request-log summary (SRV-F09) shows filter components, strict flag and entry count. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0608 DescribeClientQuotas authorization: DESCRIBE_CONFIGS on CLUSTER
P1 · M4 · Task · deps: API-0602, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeClientQuotasRequest @4.4.0
Done when: Requests need DESCRIBE_CONFIGS on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

### [To Do] API-0609 DescribeClientQuotas handler unit tests across v0-1
P1 · M4 · Task · deps: API-0604, API-0605, API-0606, API-0608 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the quota view in the metadata image (QUOTA-F08) replaced by a fake, and check the service call and the encoded response; cases cover user, client-id and ip filters, defaults, strict and non-strict matching and invalid filters.

### [To Do] API-0610 DescribeClientQuotas compatibility test with Java client 4.4.0 Admin.describeClientQuotas and kafka-configs.sh --describe for users, clients and ips
P1 · M4 · Task · deps: API-0609, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that describes user, client and ip quotas including defaults passes with Java client 4.4.0 Admin.describeClientQuotas and kafka-configs.sh --describe for users, clients and ips.

### [To Do] API-0611 DescribeClientQuotas differential test against Kafka 4.4.0
P1 · M4 · Task · deps: API-0609, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeClientQuotas requests to Kafka 4.4.0 and Alluvium for each match type with and without strict and for each client-triggerable error (each INVALID_REQUEST filter, UNSUPPORTED_VERSION and CLUSTER_AUTHORIZATION_FAILED); normalized responses match for every supported version.

### [To Do] API-0612 Advertise DescribeClientQuotas v0-1 in ApiVersions
P1 · M4 · Task · deps: API-0607, API-0610, API-0611, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists DescribeClientQuotas with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks DescribeClientQuotas supported; a test asserts the range.

## [To Do] API-F49 AlterClientQuotas (v0-1)
P1 · M4 · Feature · deps: QUOTA-F08, CTRL-F12
Scope: Kafka 4.4.0 versions 0-1; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0613 AlterClientQuotas handler module and router registration (broker and controller listeners)
P1 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAlterClientQuotas @4.4.0
Done when: `handlers/alter_client_quotas.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the controller quota path (CTRL-F12); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0614 Forward AlterClientQuotas from brokers to the active controller in an Envelope
P1 · M4 · Task · deps: API-0613, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener AlterClientQuotas is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0615 AlterClientQuotas entry validation
P1 · M4 · Task · deps: API-0613 · labels: security · ref: metadata/src/main/java/org/apache/kafka/controller/ClientQuotaControlManager.java @4.4.0
Done when: Empty entities, duplicate entity types, unhandled types, empty names, ip mixed with user or client-id, invalid IPs, unknown keys, non-finite, non-positive or fractional values for integer keys and duplicate keys or entities fail that entry with INVALID_REQUEST and Kafka's message (`ClientQuotaControlManager`); other entries proceed; unit tests.

### [To Do] API-0616 AlterClientQuotas v0-v1: set, remove and validate_only
P1 · M4 · Task · deps: API-0615 · ref: clients/src/main/resources/common/message/AlterClientQuotasRequest.json @4.4.0
Done when: Ops set or remove quota keys per entity and validate_only checks without writing records; results are per entry; v1 accepts tagged fields; tests.

### [To Do] API-0617 AlterClientQuotas error mapping
P1 · M4 · Task · deps: API-0613 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAlterClientQuotas @4.4.0
Done when: INVALID_REQUEST, NOT_CONTROLLER and CLUSTER_AUTHORIZATION_FAILED map exhaustively per entry with messages; table test.

### [To Do] API-0618 AlterClientQuotas throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0613, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; AlterClientQuotas records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal.

### [To Do] API-0619 AlterClientQuotas request metrics, events and request-log summary
P1 · M4 · Task · deps: API-0613 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for AlterClientQuotas; the request-log summary (SRV-F09) shows entity types, op count and validate_only. Quota changes emit an event with reason `alter_client_quotas_request` (CTRL-F12). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0620 AlterClientQuotas authorization: ALTER_CONFIGS on CLUSTER
P1 · M4 · Task · deps: API-0614, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAlterClientQuotas @4.4.0
Done when: Requests need ALTER_CONFIGS on CLUSTER or every entry gets CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

### [To Do] API-0621 AlterClientQuotas handler unit tests across v0-1
P1 · M4 · Task · deps: API-0616, API-0617, API-0618, API-0620 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the controller quota path (CTRL-F12) replaced by a fake, and check the service call and the encoded response; cases cover set and remove for user, client-id, ip and default entities, invalid keys and validate_only.

### [To Do] API-0622 AlterClientQuotas compatibility test with Java client 4.4.0 Admin.alterClientQuotas and kafka-configs.sh --alter for users, clients and ips
P1 · M4 · Task · deps: API-0621, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that sets, changes and removes quotas and observes throttling take effect passes with Java client 4.4.0 Admin.alterClientQuotas and kafka-configs.sh --alter for users, clients and ips.

### [To Do] API-0623 AlterClientQuotas differential test against Kafka 4.4.0
P1 · M4 · Task · deps: API-0621, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical AlterClientQuotas requests to Kafka 4.4.0 and Alluvium for set, remove and validate_only and for each client-triggerable error (each INVALID_REQUEST entry rule and CLUSTER_AUTHORIZATION_FAILED); normalized responses match for every supported version.

### [To Do] API-0624 Advertise AlterClientQuotas v0-1 in ApiVersions
P1 · M4 · Task · deps: API-0619, API-0622, API-0623, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists AlterClientQuotas with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks AlterClientQuotas supported; a test asserts the range.

## [To Do] API-F50 DescribeUserScramCredentials (v0)
P1 · M4 · Feature · deps: SEC-F04
Scope: Kafka 4.4.0 versions 0; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0625 DescribeUserScramCredentials handler module and router registration (broker and controller listeners)
P1 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeUserScramCredentialsRequest @4.4.0
Done when: `handlers/describe_user_scram_credentials.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the SCRAM credential view (SEC-F04); it is registered for the broker and controller listeners in the router (SRV-F04). Brokers and the controller both answer from their metadata image. Unit test per listener.

### [To Do] API-0626 DescribeUserScramCredentials v0: mechanisms and iterations per user
P1 · M4 · Task · deps: API-0625 · labels: security · ref: metadata/src/main/java/org/apache/kafka/image/ScramImage.java describe @4.4.0
Done when: A null users list describes every user with credentials; each result lists mechanism and iterations only, never salts or salted passwords; a user listed twice gets DUPLICATE_RESOURCE and an unknown user RESOURCE_NOT_FOUND, as `ScramImage.describe` does; tests.

### [To Do] API-0627 DescribeUserScramCredentials error mapping
P1 · M4 · Task · deps: API-0625 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeUserScramCredentialsRequest @4.4.0
Done when: Top-level CLUSTER_AUTHORIZATION_FAILED and per-user DUPLICATE_RESOURCE and RESOURCE_NOT_FOUND map exhaustively with messages; table test.

### [To Do] API-0628 DescribeUserScramCredentials throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0625, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; DescribeUserScramCredentials records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0629 DescribeUserScramCredentials request metrics, events and request-log summary
P1 · M4 · Task · deps: API-0625 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeUserScramCredentials; the request-log summary (SRV-F09) shows the number of users requested and returned. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0630 DescribeUserScramCredentials authorization: DESCRIBE on CLUSTER
P1 · M4 · Task · deps: API-0625, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeUserScramCredentialsRequest @4.4.0
Done when: Requests need DESCRIBE on CLUSTER or get top-level CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

### [To Do] API-0631 DescribeUserScramCredentials handler unit tests at v0
P1 · M4 · Task · deps: API-0626, API-0627, API-0628, API-0630 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the SCRAM credential view (SEC-F04) replaced by a fake, and check the service call and the encoded response; cases cover all users, selected users, duplicates and unknown users.

### [To Do] API-0632 DescribeUserScramCredentials compatibility test with Java client 4.4.0 Admin.describeUserScramCredentials and kafka-configs.sh --describe --entity-type users
P1 · M4 · Task · deps: API-0631, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that shows SCRAM-SHA-256 and SCRAM-SHA-512 iterations for users passes with Java client 4.4.0 Admin.describeUserScramCredentials and kafka-configs.sh --describe --entity-type users.

### [To Do] API-0633 DescribeUserScramCredentials differential test against Kafka 4.4.0
P1 · M4 · Task · deps: API-0631, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeUserScramCredentials requests to Kafka 4.4.0 and Alluvium for all and selected users and for each client-triggerable error (DUPLICATE_RESOURCE and RESOURCE_NOT_FOUND); normalized responses match for every supported version.

### [To Do] API-0634 Advertise DescribeUserScramCredentials v0 in ApiVersions
P1 · M4 · Task · deps: API-0629, API-0632, API-0633, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists DescribeUserScramCredentials with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks DescribeUserScramCredentials supported; a test asserts the range.

## [To Do] API-F51 AlterUserScramCredentials (v0)
P1 · M4 · Feature · deps: SEC-F04, CTRL-F14
Scope: Kafka 4.4.0 versions 0; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0635 AlterUserScramCredentials handler module and router registration (broker and controller listeners)
P1 · M4 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAlterUserScramCredentials @4.4.0
Done when: `handlers/alter_user_scram_credentials.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the controller SCRAM path (CTRL-F14); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0636 Forward AlterUserScramCredentials from brokers to the active controller in an Envelope
P1 · M4 · Task · deps: API-0635, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener AlterUserScramCredentials is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0637 AlterUserScramCredentials validation
P1 · M4 · Task · deps: API-0635 · labels: security · ref: metadata/src/main/java/org/apache/kafka/controller/ScramControlManager.java @4.4.0
Done when: Empty user names get UNACCEPTABLE_CREDENTIAL 'Username must not be empty', unknown mechanisms UNSUPPORTED_SASL_MECHANISM, iterations outside the mechanism's bounds UNACCEPTABLE_CREDENTIAL 'Too few iterations' or 'Too many iterations', a user changed twice for the same mechanism DUPLICATE_RESOURCE and deletion of a missing credential RESOURCE_NOT_FOUND, each per user as `ScramControlManager` does; unit tests.

### [To Do] API-0638 AlterUserScramCredentials v0: upsertions and deletions
P1 · M4 · Task · deps: API-0637 · labels: security · ref: clients/src/main/resources/common/message/AlterUserScramCredentialsRequest.json @4.4.0
Done when: Upsertions store salt, salted password and iterations and deletions remove one mechanism; results are per user with error_message; salts and salted passwords are never logged; tests.

### [To Do] API-0639 AlterUserScramCredentials error mapping
P1 · M4 · Task · deps: API-0635 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAlterUserScramCredentials @4.4.0
Done when: UNACCEPTABLE_CREDENTIAL, UNSUPPORTED_SASL_MECHANISM, DUPLICATE_RESOURCE, RESOURCE_NOT_FOUND, NOT_CONTROLLER and CLUSTER_AUTHORIZATION_FAILED map exhaustively; table test.

### [To Do] API-0640 AlterUserScramCredentials throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0635, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; AlterUserScramCredentials records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal.

### [To Do] API-0641 AlterUserScramCredentials request metrics, events and request-log summary
P1 · M4 · Task · deps: API-0635 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for AlterUserScramCredentials; the request-log summary (SRV-F09) shows user count, mechanisms and the number of upsertions and deletions. Credential changes emit an event with reason `alter_user_scram_credentials_request` (CTRL-F14). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0642 AlterUserScramCredentials authorization: ALTER on CLUSTER
P1 · M4 · Task · deps: API-0636, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAlterUserScramCredentials @4.4.0
Done when: Requests need ALTER on CLUSTER or every user gets CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

### [To Do] API-0643 AlterUserScramCredentials handler unit tests at v0
P1 · M4 · Task · deps: API-0638, API-0639, API-0640, API-0642 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the controller SCRAM path (CTRL-F14) replaced by a fake, and check the service call and the encoded response; cases cover upsert, replace, delete, bad iterations, unknown mechanisms and duplicates.

### [To Do] API-0644 AlterUserScramCredentials compatibility test with Java client 4.4.0 Admin.alterUserScramCredentials and kafka-configs.sh --alter --add-config 'SCRAM-SHA-256=[...]'
P1 · M4 · Task · deps: API-0643, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that creates a user, authenticates as it, rotates the password and deletes it passes with Java client 4.4.0 Admin.alterUserScramCredentials and kafka-configs.sh --alter --add-config 'SCRAM-SHA-256=[...]'.

### [To Do] API-0645 AlterUserScramCredentials differential test against Kafka 4.4.0
P1 · M4 · Task · deps: API-0643, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical AlterUserScramCredentials requests to Kafka 4.4.0 and Alluvium for upsertions and deletions and for each client-triggerable error (UNACCEPTABLE_CREDENTIAL, UNSUPPORTED_SASL_MECHANISM, DUPLICATE_RESOURCE and RESOURCE_NOT_FOUND); normalized responses match for every supported version.

### [To Do] API-0646 Advertise AlterUserScramCredentials v0 in ApiVersions
P1 · M4 · Task · deps: API-0641, API-0644, API-0645, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists AlterUserScramCredentials with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks AlterUserScramCredentials supported; a test asserts the range.

## [To Do] API-F52 Vote (v0-2)
P0 · M3 · Feature · deps: RAFT-F02
Scope: Kafka 4.4.0 versions 0-2; listeners: controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0647 Vote handler module and router registration (controller listener)
P0 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleVote @4.4.0
Done when: `handlers/vote.rs` (module doc names the Kafka counterpart) decodes v0-2 with the generated codec and calls the Raft driver (RAFT-F11); it is registered for the controller listener in the router (SRV-F04). A Vote request on a broker listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Requests are queued to the Raft driver, never handled inline. Unit test per listener.

### [To Do] API-0648 Vote request validation: cluster id, partition and voter key
P0 · M3 · Task · deps: API-0647 · labels: security · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleVoteRequest @4.4.0
Done when: A mismatched cluster_id gets top-level INCONSISTENT_CLUSTER_ID, a partition other than `__cluster_metadata-0` INVALID_REQUEST and a voter key that does not match this node INVALID_VOTER_KEY; unit tests.

### [To Do] API-0649 Vote v0: candidate id, epoch and last offset
P0 · M3 · Task · deps: API-0648 · ref: clients/src/main/resources/common/message/VoteRequest.json @4.4.0
Done when: v0 requests carry the candidate's id and epoch with last_offset_epoch and last_offset; responses return leader id, leader epoch and vote_granted per partition; tests.

### [To Do] API-0650 Vote v1: voter key, directory ids and leader endpoints (KIP-853)
P0 · M3 · Task · deps: API-0649 · ref: clients/src/main/resources/common/message/VoteRequest.json @4.4.0 (KIP-853)
Done when: v1 adds voter_id, replica_directory_id and voter_directory_id to the request and tagged node_endpoints to the response so the candidate learns the leader's address; v0 peers get no endpoints; per-version tests.

### [To Do] API-0651 Vote v2: pre-vote (KIP-996)
P0 · M3 · Task · deps: API-0650 · ref: clients/src/main/resources/common/message/VoteRequest.json @4.4.0 (KIP-996)
Done when: v2 adds pre_vote: pre-vote requests are answered without changing epoch or voted-for state; v0-v1 peers never receive one because the driver negotiates versions (PROTO-F14); tests.

### [To Do] API-0652 Vote error mapping
P0 · M3 · Task · deps: API-0647 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleVoteRequest @4.4.0
Done when: Top-level INCONSISTENT_CLUSTER_ID, INVALID_REQUEST and CLUSTER_AUTHORIZATION_FAILED and partition-level FENCED_LEADER_EPOCH, INVALID_VOTER_KEY and UNKNOWN_TOPIC_OR_PARTITION map exhaustively; table test.

### [To Do] API-0653 Vote request metrics, events and request-log summary
P0 · M3 · Task · deps: API-0647 · labels: observability · ref: core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for Vote; the request-log summary (SRV-F09) shows candidate id, epoch, pre-vote flag and the vote outcome. Grants and rejections are recorded in the Raft flight recorder with a reason (RAFT-F14). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0654 Vote handler unit tests across v0-2
P0 · M3 · Task · deps: API-0651, API-0652 · labels: test
Done when: Table-driven tests send every version v0-v2 through the handler, with the Raft driver (RAFT-F11) replaced by a fake, and check the service call and the encoded response; cases cover granted, rejected for a stale log, pre-vote, wrong cluster id and wrong voter key at v0-v2.

### [To Do] API-0655 Vote multi-node simulation test
P0 · M3 · Task · deps: API-0654, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) elects leaders in 3- and 5-voter quorums with partitions and message loss using real Vote encoding and checks there is at most one leader per epoch; a failing seed is printed and replays exactly.

### [To Do] API-0656 Vote differential test against Kafka 4.4.0
P0 · M3 · Task · deps: API-0654, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical Vote requests to Kafka 4.4.0 and Alluvium for votes sent to a Kafka 4.4.0 quorum member and for each client-triggerable error (INCONSISTENT_CLUSTER_ID and INVALID_VOTER_KEY); normalized responses match for every supported version.

### [To Do] API-0657 Advertise Vote v0-2 in ApiVersions
P0 · M3 · Task · deps: API-0653, API-0655, API-0656, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the controller listener lists Vote with min 0 and max 2 through the generated list owned by API-F18; docs/parity.md marks Vote supported; a test asserts the range. Vote is listed only on the controller listener. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0658 Vote authorization: CLUSTER_ACTION
P0 · M4 · Task · deps: API-0647, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleVote @4.4.0
Done when: Vote requests need CLUSTER_ACTION on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; test.

## [To Do] API-F53 BeginQuorumEpoch (v0-1)
P0 · M3 · Feature · deps: RAFT-F03
Scope: Kafka 4.4.0 versions 0-1; listeners: controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0659 BeginQuorumEpoch handler module and router registration (controller listener)
P0 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleBeginQuorumEpoch @4.4.0
Done when: `handlers/begin_quorum_epoch.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the Raft driver (RAFT-F11); it is registered for the controller listener in the router (SRV-F04). A BeginQuorumEpoch request on a broker listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Requests are queued to the Raft driver. Unit test per listener.

### [To Do] API-0660 BeginQuorumEpoch validation: cluster id, partition, voter key and endpoints
P0 · M3 · Task · deps: API-0659 · labels: security · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleBeginQuorumEpochRequest @4.4.0
Done when: A mismatched cluster_id gets INCONSISTENT_CLUSTER_ID, a wrong partition INVALID_REQUEST, a mismatched voter key INVALID_VOTER_KEY and a v1 request without leader_endpoints INVALID_REQUEST; unit tests.

### [To Do] API-0661 BeginQuorumEpoch v0: leader announcement
P0 · M3 · Task · deps: API-0660 · ref: clients/src/main/resources/common/message/BeginQuorumEpochRequest.json @4.4.0
Done when: v0 carries leader id and epoch per partition and the response echoes the receiver's view; tests.

### [To Do] API-0662 BeginQuorumEpoch v1: flexible versions, voter key and leader endpoints (KIP-853)
P0 · M3 · Task · deps: API-0661 · ref: clients/src/main/resources/common/message/BeginQuorumEpochRequest.json @4.4.0 (KIP-853)
Done when: v1 adds voter_id, voter_directory_id and leader_endpoints to the request and node_endpoints to the response; v0 peers get none; per-version tests.

### [To Do] API-0663 BeginQuorumEpoch error mapping
P0 · M3 · Task · deps: API-0659 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleBeginQuorumEpochRequest @4.4.0
Done when: INCONSISTENT_CLUSTER_ID, INVALID_REQUEST, INVALID_VOTER_KEY, FENCED_LEADER_EPOCH and CLUSTER_AUTHORIZATION_FAILED map exhaustively; table test.

### [To Do] API-0664 BeginQuorumEpoch request metrics, events and request-log summary
P0 · M3 · Task · deps: API-0659 · labels: observability · ref: core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for BeginQuorumEpoch; the request-log summary (SRV-F09) shows leader id, epoch and the resulting follower transition. Follower transitions caused by the request carry reason `begin_quorum_epoch` (RAFT-F14). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0665 BeginQuorumEpoch handler unit tests across v0-1
P0 · M3 · Task · deps: API-0662, API-0663 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the Raft driver (RAFT-F11) replaced by a fake, and check the service call and the encoded response; cases cover a new leader, a stale epoch, wrong cluster id and voter key at v0-v1.

### [To Do] API-0666 BeginQuorumEpoch multi-node simulation test
P0 · M3 · Task · deps: API-0665, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) announces leaders across partitions and restarts and checks followers converge on the newest epoch; a failing seed is printed and replays exactly.

### [To Do] API-0667 BeginQuorumEpoch differential test against Kafka 4.4.0
P0 · M3 · Task · deps: API-0665, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical BeginQuorumEpoch requests to Kafka 4.4.0 and Alluvium for announcements to a Kafka 4.4.0 quorum member and for each client-triggerable error (INCONSISTENT_CLUSTER_ID and INVALID_VOTER_KEY); normalized responses match for every supported version.

### [To Do] API-0668 Advertise BeginQuorumEpoch v0-1 in ApiVersions
P0 · M3 · Task · deps: API-0664, API-0666, API-0667, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the controller listener lists BeginQuorumEpoch with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks BeginQuorumEpoch supported; a test asserts the range. BeginQuorumEpoch is listed only on the controller listener. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0669 BeginQuorumEpoch authorization: CLUSTER_ACTION
P0 · M4 · Task · deps: API-0659, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleBeginQuorumEpoch @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; test.

## [To Do] API-F54 EndQuorumEpoch (v0-1)
P0 · M3 · Feature · deps: RAFT-F03
Scope: Kafka 4.4.0 versions 0-1; listeners: controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0670 EndQuorumEpoch handler module and router registration (controller listener)
P0 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleEndQuorumEpoch @4.4.0
Done when: `handlers/end_quorum_epoch.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the Raft driver (RAFT-F11); it is registered for the controller listener in the router (SRV-F04). A EndQuorumEpoch request on a broker listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Requests are queued to the Raft driver. Unit test per listener.

### [To Do] API-0671 EndQuorumEpoch validation: cluster id, partition, candidates and endpoints
P0 · M3 · Task · deps: API-0670 · labels: security · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleEndQuorumEpochRequest @4.4.0
Done when: A mismatched cluster_id gets INCONSISTENT_CLUSTER_ID; a wrong partition, a v1 request without leader_endpoints or preferred candidates without directory ids get INVALID_REQUEST; unit tests.

### [To Do] API-0672 EndQuorumEpoch v0: graceful resignation with preferred_successors
P0 · M3 · Task · deps: API-0671 · ref: clients/src/main/resources/common/message/EndQuorumEpochRequest.json @4.4.0
Done when: v0 carries the resigning leader and preferred_successors; receivers shorten their election timeout by their position in the list; tests.

### [To Do] API-0673 EndQuorumEpoch v1: preferred_candidates with directory ids and leader endpoints (KIP-853)
P0 · M3 · Task · deps: API-0672 · ref: clients/src/main/resources/common/message/EndQuorumEpochRequest.json @4.4.0 (KIP-853)
Done when: v1 replaces preferred_successors with preferred_candidates (id and directory id) and adds leader_endpoints and response node_endpoints; the receiver ranks itself by candidate key; per-version tests.

### [To Do] API-0674 EndQuorumEpoch error mapping
P0 · M3 · Task · deps: API-0670 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleEndQuorumEpochRequest @4.4.0
Done when: INCONSISTENT_CLUSTER_ID, INVALID_REQUEST, FENCED_LEADER_EPOCH and CLUSTER_AUTHORIZATION_FAILED map exhaustively; table test.

### [To Do] API-0675 EndQuorumEpoch request metrics, events and request-log summary
P0 · M3 · Task · deps: API-0670 · labels: observability · ref: core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for EndQuorumEpoch; the request-log summary (SRV-F09) shows resigning leader, epoch and the receiver's rank among candidates. Observed resignations carry reason `end_quorum_epoch` (RAFT-F14). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0676 EndQuorumEpoch handler unit tests across v0-1
P0 · M3 · Task · deps: API-0673, API-0674 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the Raft driver (RAFT-F11) replaced by a fake, and check the service call and the encoded response; cases cover resignation with and without the receiver among candidates, stale epochs and the v0/v1 layouts.

### [To Do] API-0677 EndQuorumEpoch multi-node simulation test
P0 · M3 · Task · deps: API-0676, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) shuts the leader down gracefully under load and checks a preferred candidate wins and no committed entry is lost; a failing seed is printed and replays exactly.

### [To Do] API-0678 EndQuorumEpoch differential test against Kafka 4.4.0
P0 · M3 · Task · deps: API-0676, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical EndQuorumEpoch requests to Kafka 4.4.0 and Alluvium for resignations sent to a Kafka 4.4.0 quorum member and for each client-triggerable error (INCONSISTENT_CLUSTER_ID and missing leader endpoints); normalized responses match for every supported version.

### [To Do] API-0679 Advertise EndQuorumEpoch v0-1 in ApiVersions
P0 · M3 · Task · deps: API-0675, API-0677, API-0678, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the controller listener lists EndQuorumEpoch with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks EndQuorumEpoch supported; a test asserts the range. EndQuorumEpoch is listed only on the controller listener. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0680 EndQuorumEpoch authorization: CLUSTER_ACTION
P0 · M4 · Task · deps: API-0670, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleEndQuorumEpoch @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; test.

## [To Do] API-F55 DescribeQuorum (v0-2)
P1 · M3 · Feature · deps: RAFT-F12
Scope: Kafka 4.4.0 versions 0-2; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0681 DescribeQuorum handler module and router registration (broker and controller listeners)
P1 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleDescribeQuorum @4.4.0
Done when: `handlers/describe_quorum.rs` (module doc names the Kafka counterpart) decodes v0-2 with the generated codec and calls the Raft quorum state (RAFT-F12); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0682 Forward DescribeQuorum from brokers to the active controller in an Envelope
P1 · M3 · Task · deps: API-0681, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener DescribeQuorum is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. DescribeQuorum is forwardable in `ApiKeys`, so brokers relay it like other controller-bound requests. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0683 DescribeQuorum v0: leader, high watermark, voters and observers
P1 · M3 · Task · deps: API-0681 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleDescribeQuorumRequest @4.4.0
Done when: The active controller returns leader id and epoch, the high watermark and each voter's and observer's log end offset; a non-leader controller returns NOT_LEADER_OR_FOLLOWER and a wrong partition UNKNOWN_TOPIC_OR_PARTITION; tests.

### [To Do] API-0684 DescribeQuorum v1: last_fetch_timestamp and last_caught_up_timestamp (KIP-836)
P1 · M3 · Task · deps: API-0683 · ref: clients/src/main/resources/common/message/DescribeQuorumResponse.json @4.4.0 (KIP-836)
Done when: v1+ replica states include last_fetch_timestamp and last_caught_up_timestamp (-1 when unknown); v0 omits them; per-version tests.

### [To Do] API-0685 DescribeQuorum v2: nodes, directory ids and error messages (KIP-853)
P1 · M3 · Task · deps: API-0684 · ref: clients/src/main/resources/common/message/DescribeQuorumResponse.json @4.4.0 (KIP-853)
Done when: v2 responses add error_message at top and partition level, replica_directory_id per replica and the nodes list with each voter's listeners; per-version tests.

### [To Do] API-0686 DescribeQuorum error mapping
P1 · M3 · Task · deps: API-0681 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleDescribeQuorumRequest @4.4.0
Done when: NOT_LEADER_OR_FOLLOWER, UNKNOWN_TOPIC_OR_PARTITION, INVALID_REQUEST and CLUSTER_AUTHORIZATION_FAILED map exhaustively, with messages from v2; table test.

### [To Do] API-0687 DescribeQuorum request metrics, events and request-log summary
P1 · M3 · Task · deps: API-0681 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeQuorum; the request-log summary (SRV-F09) shows the caller and whether the request was forwarded. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0688 DescribeQuorum handler unit tests across v0-2
P1 · M3 · Task · deps: API-0682, API-0685, API-0686 · labels: test
Done when: Table-driven tests send every version v0-v2 through the handler, with the Raft quorum state (RAFT-F12) replaced by a fake, and check the service call and the encoded response; cases cover leader and non-leader controllers, observers and each version's fields.

### [To Do] API-0689 DescribeQuorum compatibility test with Java client 4.4.0 Admin.describeMetadataQuorum and kafka-metadata-quorum.sh describe --status and --replication
P1 · M3 · Task · deps: API-0688, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that describes a 3-voter quorum through a broker and through --bootstrap-controller passes with Java client 4.4.0 Admin.describeMetadataQuorum and kafka-metadata-quorum.sh describe --status and --replication.

### [To Do] API-0690 DescribeQuorum multi-node simulation test
P1 · M3 · Task · deps: API-0688, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) describes the quorum during leader changes and checks the reported leader and lags match the Raft state; a failing seed is printed and replays exactly.

### [To Do] API-0691 DescribeQuorum differential test against Kafka 4.4.0
P1 · M3 · Task · deps: API-0688, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeQuorum requests to Kafka 4.4.0 and Alluvium for descriptions at v0-v2 and for each client-triggerable error (NOT_LEADER_OR_FOLLOWER from a follower controller); normalized responses match for every supported version.

### [To Do] API-0692 Advertise DescribeQuorum v0-2 in ApiVersions
P1 · M3 · Task · deps: API-0687, API-0689, API-0690, API-0691, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists DescribeQuorum with min 0 and max 2 through the generated list owned by API-F18; docs/parity.md marks DescribeQuorum supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0693 DescribeQuorum authorization: DESCRIBE on CLUSTER
P1 · M4 · Task · deps: API-0682, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleDescribeQuorum @4.4.0
Done when: Requests need DESCRIBE on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

## [To Do] API-F56 AlterPartition (v2-3)
P0 · M3 · Feature · deps: CTRL-F09
Scope: Kafka 4.4.0 versions 2-3; listeners: controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0694 AlterPartition handler module and router registration (controller listener)
P0 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAlterPartitionRequest @4.4.0
Done when: `handlers/alter_partition.rs` (module doc names the Kafka counterpart) decodes v2-3 with the generated codec and calls the controller ISR path (CTRL-F09); it is registered for the controller listener in the router (SRV-F04). A AlterPartition request on a broker listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0695 AlterPartition validation: broker epoch and topic ids
P0 · M3 · Task · deps: API-0694 · labels: security · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java alterPartition @4.4.0
Done when: A broker_epoch that does not match the registration gets top-level STALE_BROKER_EPOCH; unknown topic ids get UNKNOWN_TOPIC_ID per partition; unit tests.

### [To Do] API-0696 AlterPartition v2: topic ids, leader recovery state and KIP-841 errors
P0 · M3 · Task · deps: API-0695 · ref: clients/src/main/resources/common/message/AlterPartitionRequest.json @4.4.0 (KIP-704, KIP-841)
Done when: v2 addresses topics by id and passes new_isr, leader_recovery_state and partition_epoch to CTRL-F09; responses return the committed leader, ISR, epochs and recovery state, with INELIGIBLE_REPLICA, NEW_LEADER_ELECTED, INVALID_UPDATE_VERSION and FENCED_LEADER_EPOCH per partition; tests.

### [To Do] API-0697 AlterPartition v3: new_isr_with_epochs (KIP-903)
P0 · M3 · Task · deps: API-0696 · ref: clients/src/main/resources/common/message/AlterPartitionRequest.json @4.4.0 (KIP-903)
Done when: v3 replaces new_isr with new_isr_with_epochs so the controller rejects ISR members whose broker epoch is stale (INELIGIBLE_REPLICA); v2 members are treated as having epoch -1; per-version tests.

### [To Do] API-0698 AlterPartition error mapping
P0 · M3 · Task · deps: API-0694 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAlterPartitionRequest @4.4.0
Done when: Top-level NOT_CONTROLLER, STALE_BROKER_EPOCH and CLUSTER_AUTHORIZATION_FAILED and the partition errors above map exhaustively; table test.

### [To Do] API-0699 AlterPartition request-quota exemption (cluster action)
P0 · M4 · Task · deps: API-0694, QUOTA-F04 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala handleError @4.4.0
Done when: AlterPartition is a cluster-action API, so it is exempt from request quotas and throttle_time_ms stays 0; only CLUSTER_AUTHORIZATION_FAILED responses may be throttled, as in `RequestHandlerHelper.handleError`; test with a quota-limited principal.

### [To Do] API-0700 AlterPartition request metrics, events and request-log summary
P0 · M3 · Task · deps: API-0694 · labels: observability · ref: core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for AlterPartition; the request-log summary (SRV-F09) shows broker id, broker epoch, partition count and per-partition outcome. ISR changes carry reason `alter_partition` with the requesting leader (CTRL-F09). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0701 AlterPartition handler unit tests across v2-3
P0 · M3 · Task · deps: API-0697, API-0698 · labels: test
Done when: Table-driven tests send every version v2-v3 through the handler, with the controller ISR path (CTRL-F09) replaced by a fake, and check the service call and the encoded response; cases cover ISR shrink and expand, stale partition epochs, fenced members, recovery-state changes and v2/v3 layouts.

### [To Do] API-0702 AlterPartition multi-node simulation test
P0 · M3 · Task · deps: API-0701, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) shrinks and expands ISRs through AlterPartition with crashing followers and controller failover and checks the ISR never contains a fenced broker; a failing seed is printed and replays exactly.

### [To Do] API-0703 AlterPartition differential test against Kafka 4.4.0
P0 · M3 · Task · deps: API-0701, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical AlterPartition requests to Kafka 4.4.0 and Alluvium for ISR changes sent to a Kafka 4.4.0 controller and for each client-triggerable error (STALE_BROKER_EPOCH, INVALID_UPDATE_VERSION and INELIGIBLE_REPLICA); normalized responses match for every supported version.

### [To Do] API-0704 Advertise AlterPartition v2-3 in ApiVersions
P0 · M3 · Task · deps: API-0700, API-0702, API-0703, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the controller listener lists AlterPartition with min 2 and max 3 through the generated list owned by API-F18; docs/parity.md marks AlterPartition supported; a test asserts the range. AlterPartition is listed only on the controller listener. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0705 AlterPartition authorization: CLUSTER_ACTION
P0 · M4 · Task · deps: API-0694, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAlterPartitionRequest @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; test.

## [To Do] API-F57 UpdateFeatures (v0-2)
P1 · M3 · Feature · deps: CTRL-F17
Scope: Kafka 4.4.0 versions 0-2; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0706 UpdateFeatures handler module and router registration (broker and controller listeners)
P1 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleUpdateFeatures @4.4.0
Done when: `handlers/update_features.rs` (module doc names the Kafka counterpart) decodes v0-2 with the generated codec and calls the controller feature path (CTRL-F17); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0707 Forward UpdateFeatures from brokers to the active controller in an Envelope
P1 · M3 · Task · deps: API-0706, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener UpdateFeatures is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0708 UpdateFeatures validation: unknown features, levels and downgrade types
P1 · M3 · Task · deps: API-0706 · labels: security · ref: metadata/src/main/java/org/apache/kafka/controller/FeatureControlManager.java updateFeatures @4.4.0
Done when: Unknown feature names, levels outside every node's supported range, and a level below 1 with upgrade type UPGRADE fail with INVALID_UPDATE_VERSION or INVALID_REQUEST from CTRL-F17 with an error message; unit tests.

### [To Do] API-0709 UpdateFeatures v0: allow_downgrade mapped to SAFE_DOWNGRADE
P1 · M3 · Task · deps: API-0708 · ref: clients/src/main/java/org/apache/kafka/common/requests/UpdateFeaturesRequest.java featureUpdates @4.4.0
Done when: In v0 allow_downgrade true means SAFE_DOWNGRADE and false UPGRADE, as `UpdateFeaturesRequest` maps it; per-feature results are returned; tests.

### [To Do] API-0710 UpdateFeatures v1: upgrade_type and validate_only
P1 · M3 · Task · deps: API-0709 · ref: clients/src/main/resources/common/message/UpdateFeaturesRequest.json @4.4.0
Done when: v1 upgrade_type selects UPGRADE (1), SAFE_DOWNGRADE (2) or UNSAFE_DOWNGRADE (3); validate_only checks without writing records; per-feature results are still returned; tests.

### [To Do] API-0711 UpdateFeatures v2: top-level result only
P1 · M3 · Task · deps: API-0710 · ref: clients/src/main/resources/common/message/UpdateFeaturesResponse.json @4.4.0
Done when: v2 responses omit per-feature results and report the outcome only in the top-level error_code and error_message; tests compare v1 and v2 for the same failure.

### [To Do] API-0712 UpdateFeatures error mapping per version
P1 · M3 · Task · deps: API-0706 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleUpdateFeatures @4.4.0
Done when: INVALID_UPDATE_VERSION, INVALID_REQUEST, FEATURE_UPDATE_FAILED, NOT_CONTROLLER and CLUSTER_AUTHORIZATION_FAILED map exhaustively at the right level per version; table test.

### [To Do] API-0713 UpdateFeatures request-quota exemption (cluster action)
P1 · M4 · Task · deps: API-0706, QUOTA-F04 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala handleError @4.4.0
Done when: UpdateFeatures is a cluster-action API, so it is exempt from request quotas and throttle_time_ms stays 0; only CLUSTER_AUTHORIZATION_FAILED responses may be throttled, as in `RequestHandlerHelper.handleError`; test with a quota-limited principal.

### [To Do] API-0714 UpdateFeatures request metrics, events and request-log summary
P1 · M3 · Task · deps: API-0706 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for UpdateFeatures; the request-log summary (SRV-F09) shows feature names, target levels, upgrade types and validate_only. Finalized-level changes carry reason `update_features_request` (CTRL-F17). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0715 UpdateFeatures handler unit tests across v0-2
P1 · M3 · Task · deps: API-0707, API-0711, API-0712 · labels: test
Done when: Table-driven tests send every version v0-v2 through the handler, with the controller feature path (CTRL-F17) replaced by a fake, and check the service call and the encoded response; cases cover upgrade, safe and unsafe downgrade, validate_only, unknown features and the v0-v2 result shapes.

### [To Do] API-0716 UpdateFeatures compatibility test with Java client 4.4.0 Admin.updateFeatures and kafka-features.sh upgrade, downgrade and describe
P1 · M3 · Task · deps: API-0715, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that upgrades metadata.version and group.version and refuses an unsafe downgrade without the flag passes with Java client 4.4.0 Admin.updateFeatures and kafka-features.sh upgrade, downgrade and describe.

### [To Do] API-0717 UpdateFeatures differential test against Kafka 4.4.0
P1 · M3 · Task · deps: API-0715, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical UpdateFeatures requests to Kafka 4.4.0 and Alluvium for upgrades and validate_only at v0-v2 and for each client-triggerable error (unknown features, unsupported levels and downgrades without the right type); normalized responses match for every supported version.

### [To Do] API-0718 Advertise UpdateFeatures v0-2 in ApiVersions
P1 · M3 · Task · deps: API-0714, API-0716, API-0717, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists UpdateFeatures with min 0 and max 2 through the generated list owned by API-F18; docs/parity.md marks UpdateFeatures supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0719 UpdateFeatures authorization: ALTER on CLUSTER
P1 · M4 · Task · deps: API-0707, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleUpdateFeatures @4.4.0
Done when: Requests need ALTER on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

## [To Do] API-F58 Envelope (v0)
P0 · M2 · Feature · deps: CTRL-F21
Scope: Kafka 4.4.0 versions 0; listeners: controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0720 Envelope handler module and router registration (controller listener)
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleEnvelopeRequest @4.4.0
Done when: `handlers/envelope.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the controller request dispatcher; it is registered for the controller listener in the router (SRV-F04). A Envelope request on a broker listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0721 Envelope v0: unwrap the inner request, principal and client address
P0 · M2 · Task · deps: API-0720 · labels: security, decoder · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleEnvelopeRequest @4.4.0
Done when: The controller parses request_data as a request header plus body, rebuilds the request context with the serialized principal (SEC-F01 serde), client_host_address and the outer listener, and dispatches the inner request; a non-forwardable inner API or a bad client address returns INVALID_REQUEST, an undecodable principal PRINCIPAL_DESERIALIZATION_FAILURE and an unsupported inner version UNSUPPORTED_VERSION; fuzz target over request_data.

### [To Do] API-0722 Envelope response: serialize the inner response into response_data
P0 · M2 · Task · deps: API-0721 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleEnvelopeRequest @4.4.0
Done when: The inner response is encoded with its own response header at the inner version and returned in response_data with error_code NONE; envelope-level failures return a null response_data and the error; tests round-trip each forwardable API.

### [To Do] API-0723 Envelope error mapping
P0 · M2 · Task · deps: API-0722 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleEnvelopeRequest @4.4.0
Done when: Envelope-level errors are limited to INVALID_REQUEST, UNSUPPORTED_VERSION, PRINCIPAL_DESERIALIZATION_FAILURE, NOT_CONTROLLER and CLUSTER_AUTHORIZATION_FAILED; inner errors stay inside response_data; table test.

### [To Do] API-0724 Envelope size and nesting limits
P0 · M2 · Task · deps: API-0721 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleEnvelopeRequest @4.4.0
Done when: Nested envelopes are rejected with INVALID_REQUEST; request_data is bounded by `socket.request.max.bytes` and decoded with PROTO-F09 limits; test with a nested envelope and an oversized payload.

### [To Do] API-0725 Envelope request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0720 · labels: observability · ref: core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for Envelope; the request-log summary (SRV-F09) shows inner API key and version, forwarding broker and principal name. Inner requests are counted under their own API with an `is_forwarded` tag. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0726 Envelope handler unit tests at v0
P0 · M2 · Task · deps: API-0723, API-0724 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the controller request dispatcher replaced by a fake, and check the service call and the encoded response; cases cover every forwardable inner API, malformed payloads, nested envelopes and principal round-trip.

### [To Do] API-0727 Envelope broker-controller integration test
P0 · M2 · Task · deps: API-0726 · labels: test
Done when: An in-process integration test with separate broker and controller roles over loopback forwards CreateTopics, DeleteTopics and CreatePartitions from a broker-only node and checks the principal and client address seen by the controller.

### [To Do] API-0728 Envelope differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0727, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical Envelope requests to Kafka 4.4.0 and Alluvium for forwarded CreateTopics and DeleteTopics and for each client-triggerable error (INVALID_REQUEST for malformed and nested envelopes); normalized responses match for every supported version.

### [To Do] API-0729 Advertise Envelope v0 in ApiVersions
P0 · M2 · Task · deps: API-0725, API-0728, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the controller listener lists Envelope with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks Envelope supported; a test asserts the range. Envelope is listed only on the controller listener. Later tasks (M3, M4) add behavior without changing the advertised range.

### [To Do] API-0730 Envelope authorization: CLUSTER_ACTION and the forwarded principal
P0 · M4 · Task · deps: API-0721, SEC-F08, SEC-F16 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleEnvelopeRequest @4.4.0
Done when: Envelopes need CLUSTER_ACTION on CLUSTER from the broker principal or get CLUSTER_AUTHORIZATION_FAILED before the inner request is parsed; the inner request is then authorized as the forwarded principal; tests and a differential case.

### [To Do] API-0731 Envelope multi-node simulation test
P0 · M3 · Task · deps: API-0727, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) forwards admin requests from several brokers while the active controller fails over, checking every request gets exactly one response and NOT_CONTROLLER retries land on the new controller; a failing seed is printed and replays exactly.

## [To Do] API-F59 FetchSnapshot (v0-1)
P0 · M3 · Feature · deps: RAFT-F06
Scope: Kafka 4.4.0 versions 0-1; listeners: controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0732 FetchSnapshot handler module and router registration (controller listener)
P0 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleFetchSnapshot @4.4.0
Done when: `handlers/fetch_snapshot.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the Raft driver (RAFT-F11); it is registered for the controller listener in the router (SRV-F04). A FetchSnapshot request on a broker listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Requests are queued to the Raft driver. Unit test per listener.

### [To Do] API-0733 FetchSnapshot validation: cluster id, partition, snapshot id and position
P0 · M3 · Task · deps: API-0732 · labels: security · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleFetchSnapshotRequest @4.4.0
Done when: A mismatched cluster_id gets INCONSISTENT_CLUSTER_ID, a wrong partition UNKNOWN_TOPIC_OR_PARTITION, an unknown or bootstrap snapshot SNAPSHOT_NOT_FOUND and a position outside the snapshot POSITION_OUT_OF_RANGE; max_bytes bounds the chunk; unit tests.

### [To Do] API-0734 FetchSnapshot v0: chunked snapshot transfer with leader validation
P0 · M3 · Task · deps: API-0733 · ref: clients/src/main/resources/common/message/FetchSnapshotRequest.json @4.4.0
Done when: Responses return size, position and unaligned_records for the requested chunk plus tagged current_leader; a stale current_leader_epoch gets FENCED_LEADER_EPOCH and a non-leader NOT_LEADER_OR_FOLLOWER; tests.

### [To Do] API-0735 FetchSnapshot v1: replica_directory_id and leader node_endpoints (KIP-853)
P0 · M3 · Task · deps: API-0734 · ref: clients/src/main/resources/common/message/FetchSnapshotRequest.json @4.4.0 (KIP-853)
Done when: v1 requests carry tagged replica_directory_id and responses add node_endpoints; per-version tests.

### [To Do] API-0736 FetchSnapshot error mapping
P0 · M3 · Task · deps: API-0732 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleFetchSnapshotRequest @4.4.0
Done when: INCONSISTENT_CLUSTER_ID, INVALID_REQUEST, UNKNOWN_TOPIC_OR_PARTITION, SNAPSHOT_NOT_FOUND, POSITION_OUT_OF_RANGE, FENCED_LEADER_EPOCH, NOT_LEADER_OR_FOLLOWER and CLUSTER_AUTHORIZATION_FAILED map exhaustively; table test.

### [To Do] API-0737 FetchSnapshot throttle_time_ms through the shared response path
P0 · M3 · Task · deps: API-0732 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0738 FetchSnapshot throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0737, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: FetchSnapshot records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0739 FetchSnapshot request metrics, events and request-log summary
P0 · M3 · Task · deps: API-0732 · labels: observability · ref: core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for FetchSnapshot; the request-log summary (SRV-F09) shows replica id, snapshot id, position and bytes returned. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0740 FetchSnapshot handler unit tests across v0-1
P0 · M3 · Task · deps: API-0735, API-0736, API-0737 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the Raft driver (RAFT-F11) replaced by a fake, and check the service call and the encoded response; cases cover full transfers in several chunks, missing snapshots, bad positions and leader changes.

### [To Do] API-0741 FetchSnapshot multi-node simulation test
P0 · M3 · Task · deps: API-0740, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) brings a lagging voter and an observer up to date from a snapshot while the leader changes and checks the loaded image equals the leader's; a failing seed is printed and replays exactly.

### [To Do] API-0742 FetchSnapshot differential test against Kafka 4.4.0
P0 · M3 · Task · deps: API-0740, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical FetchSnapshot requests to Kafka 4.4.0 and Alluvium for snapshot chunks served to a Kafka 4.4.0 fetcher and for each client-triggerable error (SNAPSHOT_NOT_FOUND and POSITION_OUT_OF_RANGE); normalized responses match for every supported version.

### [To Do] API-0743 Advertise FetchSnapshot v0-1 in ApiVersions
P0 · M3 · Task · deps: API-0739, API-0741, API-0742, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the controller listener lists FetchSnapshot with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks FetchSnapshot supported; a test asserts the range. FetchSnapshot is listed only on the controller listener. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0744 FetchSnapshot authorization: CLUSTER_ACTION
P0 · M4 · Task · deps: API-0732, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleFetchSnapshot @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; test.

## [To Do] API-F60 DescribeCluster (v0-2)
P1 · M2 · Feature · deps: META-F09
Scope: Kafka 4.4.0 versions 0-2; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0745 DescribeCluster handler module and router registration (broker and controller listeners)
P1 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeCluster @4.4.0
Done when: `handlers/describe_cluster.rs` (module doc names the Kafka counterpart) decodes v0-2 with the generated codec and calls the broker metadata lookups (META-F09); it is registered for the broker and controller listeners in the router (SRV-F04). The controller answers with controller endpoints (`ControllerApis.handleDescribeCluster`). Unit test per listener.

### [To Do] API-0746 DescribeCluster v0: cluster id, controller id and alive brokers
P1 · M2 · Task · deps: API-0745 · ref: server/src/main/java/org/apache/kafka/server/AuthHelper.java computeDescribeClusterResponse @4.4.0
Done when: Brokers return cluster_id, a random alive broker as controller_id and the alive brokers on the request's listener with rack; tests with fenced and unfenced brokers.

### [To Do] API-0747 DescribeCluster v1: endpoint_type for brokers and controllers (KIP-919)
P1 · M2 · Task · deps: API-0745 · ref: server/src/main/java/org/apache/kafka/server/AuthHelper.java computeDescribeClusterResponse @4.4.0 (KIP-919)
Done when: endpoint_type 1 asks for brokers and 2 for controllers; a broker asked for controllers or a controller asked for brokers returns MISMATCHED_ENDPOINT_TYPE, an unknown type UNSUPPORTED_ENDPOINT_TYPE; the controller returns the active controller id and controller endpoints; below v1 only brokers are described; tests per listener.

### [To Do] API-0748 DescribeCluster v2: include_fenced_brokers and is_fenced (KIP-1073)
P1 · M2 · Task · deps: API-0746 · ref: clients/src/main/resources/common/message/DescribeClusterRequest.json @4.4.0 (KIP-1073)
Done when: With include_fenced_brokers set, fenced brokers are listed with is_fenced true; otherwise only unfenced brokers; tests per version.

### [To Do] API-0749 DescribeCluster error mapping
P1 · M2 · Task · deps: API-0747 · ref: server/src/main/java/org/apache/kafka/server/AuthHelper.java computeDescribeClusterResponse @4.4.0
Done when: Top-level error_code and error_message cover MISMATCHED_ENDPOINT_TYPE, UNSUPPORTED_ENDPOINT_TYPE and CLUSTER_AUTHORIZATION_FAILED with empty broker lists; table test.

### [To Do] API-0750 DescribeCluster throttle_time_ms through the shared response path
P1 · M2 · Task · deps: API-0745 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0751 DescribeCluster throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0750, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: DescribeCluster records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0752 DescribeCluster request metrics, events and request-log summary
P1 · M2 · Task · deps: API-0745 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeCluster; the request-log summary (SRV-F09) shows endpoint type, include_fenced_brokers and include_cluster_authorized_operations. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0753 DescribeCluster handler unit tests across v0-2
P1 · M2 · Task · deps: API-0748, API-0749, API-0750 · labels: test
Done when: Table-driven tests send every version v0-v2 through the handler, with the broker metadata lookups (META-F09) replaced by a fake, and check the service call and the encoded response; cases cover both listeners, both endpoint types, fenced brokers and mismatched types.

### [To Do] API-0754 DescribeCluster compatibility test with Java client 4.4.0 Admin.describeCluster (including bootstrap.controllers), librdkafka, franz-go and kafka-cluster.sh cluster-id
P1 · M2 · Task · deps: API-0753, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that describes the cluster through a broker and through a controller passes with Java client 4.4.0 Admin.describeCluster (including bootstrap.controllers), librdkafka, franz-go and kafka-cluster.sh cluster-id.

### [To Do] API-0755 DescribeCluster differential test against Kafka 4.4.0
P1 · M2 · Task · deps: API-0753, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeCluster requests to Kafka 4.4.0 and Alluvium for broker and controller descriptions at v0-v2 and for each client-triggerable error (MISMATCHED_ENDPOINT_TYPE and UNSUPPORTED_ENDPOINT_TYPE); normalized responses match for every supported version.

### [To Do] API-0756 Advertise DescribeCluster v0-2 in ApiVersions
P1 · M2 · Task · deps: API-0752, API-0754, API-0755, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists DescribeCluster with min 0 and max 2 through the generated list owned by API-F18; docs/parity.md marks DescribeCluster supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0757 DescribeCluster authorization: DESCRIBE on brokers, ALTER on controllers
P1 · M4 · Task · deps: API-0749, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleDescribeCluster @4.4.0
Done when: On brokers the request needs DESCRIBE on CLUSTER; on controllers it needs ALTER on CLUSTER, otherwise CLUSTER_AUTHORIZATION_FAILED; include_cluster_authorized_operations fills cluster_authorized_operations; tests and differential cases.

## [To Do] API-F61 DescribeProducers (v0)
P1 · M6 · Feature · deps: TXN-F13
Scope: Kafka 4.4.0 versions 0; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0758 DescribeProducers handler module and router registration (broker listener)
P1 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeProducersRequest @4.4.0
Done when: `handlers/describe_producers.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the producer-state view (TXN-F13); it is registered for the broker listener in the router (SRV-F04). A DescribeProducers request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0759 DescribeProducers v0: active producer state per partition
P1 · M6 · Task · deps: API-0758 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeProducersRequest @4.4.0
Done when: Each partition led by this broker returns its active producers (id, epoch, last sequence, last timestamp, coordinator epoch, current transaction start offset); invalid topic names get INVALID_TOPIC_EXCEPTION, unknown topics UNKNOWN_TOPIC_OR_PARTITION and non-leaders NOT_LEADER_OR_FOLLOWER, each with error_message; tests.

### [To Do] API-0760 DescribeProducers error mapping
P1 · M6 · Task · deps: API-0758 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeProducersRequest @4.4.0
Done when: INVALID_TOPIC_EXCEPTION, TOPIC_AUTHORIZATION_FAILED, UNKNOWN_TOPIC_OR_PARTITION, NOT_LEADER_OR_FOLLOWER and KAFKA_STORAGE_ERROR map exhaustively per partition with messages; table test.

### [To Do] API-0761 DescribeProducers throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M6 · Task · deps: API-0758, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; DescribeProducers records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0762 DescribeProducers request metrics, events and request-log summary
P1 · M6 · Task · deps: API-0758 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeProducers; the request-log summary (SRV-F09) shows topic and partition counts and the number of producers returned. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0763 DescribeProducers authorization: READ on each topic
P1 · M6 · Task · deps: API-0758, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeProducersRequest @4.4.0
Done when: Topics without READ on TOPIC get TOPIC_AUTHORIZATION_FAILED for all requested partitions; tests and a differential case.

### [To Do] API-0764 DescribeProducers handler unit tests at v0
P1 · M6 · Task · deps: API-0759, API-0760, API-0761, API-0763 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the producer-state view (TXN-F13) replaced by a fake, and check the service call and the encoded response; cases cover idempotent and transactional producers, open transactions, unknown topics and followers.

### [To Do] API-0765 DescribeProducers compatibility test with Java client 4.4.0 Admin.describeProducers and kafka-transactions.sh describe-producers
P1 · M6 · Task · deps: API-0764, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that shows open transactions and their start offsets passes with Java client 4.4.0 Admin.describeProducers and kafka-transactions.sh describe-producers.

### [To Do] API-0766 DescribeProducers differential test against Kafka 4.4.0
P1 · M6 · Task · deps: API-0764, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeProducers requests to Kafka 4.4.0 and Alluvium for partitions with active producers and for each client-triggerable error (UNKNOWN_TOPIC_OR_PARTITION, NOT_LEADER_OR_FOLLOWER and INVALID_TOPIC_EXCEPTION); normalized responses match for every supported version.

### [To Do] API-0767 Advertise DescribeProducers v0 in ApiVersions
P1 · M6 · Task · deps: API-0762, API-0765, API-0766, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists DescribeProducers with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks DescribeProducers supported; a test asserts the range.

## [To Do] API-F62 BrokerRegistration (v0-4)
P0 · M2 · Feature · deps: CTRL-F02
Scope: Kafka 4.4.0 versions 0-4; listeners: controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0768 BrokerRegistration handler module and router registration (controller listener)
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleBrokerRegistration @4.4.0
Done when: `handlers/broker_registration.rs` (module doc names the Kafka counterpart) decodes v0-4 with the generated codec and calls the cluster-control service (CTRL-F02); it is registered for the controller listener in the router (SRV-F04). A BrokerRegistration request on a broker listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0769 BrokerRegistration validation: cluster id, listeners and features
P0 · M2 · Task · deps: API-0768 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleBrokerRegistration @4.4.0
Done when: A cluster_id different from the controller's fails with INCONSISTENT_CLUSTER_ID; duplicate listener names, unknown security protocols and features outside the controller's finalized range fail with INVALID_REGISTRATION or UNSUPPORTED_VERSION from CTRL-F02; errors are returned in error_code; unit tests.

### [To Do] API-0770 BrokerRegistration v0-v1: registration fields and is_migrating_zk_broker
P0 · M2 · Task · deps: API-0769 · ref: clients/src/main/resources/common/message/BrokerRegistrationRequest.json @4.4.0
Done when: v0 registers id, incarnation id, listeners, features and rack and returns broker_epoch; v1's is_migrating_zk_broker is accepted only as false because ZooKeeper migration is gone in 4.x, true gets BROKER_ID_NOT_REGISTERED as CTRL-F02 decides; per-version tests.

### [To Do] API-0771 BrokerRegistration v2-v4: log_dirs, previous_broker_epoch and feature minimum 0
P0 · M2 · Task · deps: API-0769 · ref: clients/src/main/resources/common/message/BrokerRegistrationRequest.json @4.4.0 (KIP-858, KIP-966)
Done when: v2+ passes log directory ids (KIP-858; empty, reserved or duplicate ids or a directory owned by another broker give INVALID_REGISTRATION), v3+ previous_broker_epoch for clean-shutdown detection (KIP-966), and v4+ may carry features with minimum version 0 (KAFKA-17011) that v0-v3 senders omit; per-version tests.

### [To Do] API-0772 BrokerRegistration error mapping
P0 · M2 · Task · deps: API-0768 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleBrokerRegistration @4.4.0
Done when: NOT_CONTROLLER, INCONSISTENT_CLUSTER_ID, DUPLICATE_BROKER_REGISTRATION, INVALID_REGISTRATION, UNSUPPORTED_VERSION and BROKER_ID_NOT_REGISTERED map exhaustively; table test.

### [To Do] API-0773 BrokerRegistration request-quota exemption (cluster action)
P0 · M4 · Task · deps: API-0768, QUOTA-F04 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala handleError @4.4.0
Done when: BrokerRegistration is a cluster-action API, so it is exempt from request quotas and throttle_time_ms stays 0; only CLUSTER_AUTHORIZATION_FAILED responses may be throttled, as in `RequestHandlerHelper.handleError`; test with a quota-limited principal.

### [To Do] API-0774 BrokerRegistration request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0768 · labels: observability · ref: core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for BrokerRegistration; the request-log summary (SRV-F09) shows broker id, incarnation id, listener names and feature ranges. Registrations emit a controller event with reason `broker_registration` (CTRL-F02). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0775 BrokerRegistration handler unit tests across v0-4
P0 · M2 · Task · deps: API-0770, API-0771, API-0772 · labels: test
Done when: Table-driven tests send every version v0-v4 through the handler, with the cluster-control service (CTRL-F02) replaced by a fake, and check the service call and the encoded response; cases cover first registration, re-registration with a new incarnation, mismatched cluster id, bad listeners and each version's fields.

### [To Do] API-0776 BrokerRegistration broker-controller integration test
P0 · M2 · Task · deps: API-0775 · labels: test
Done when: An in-process integration test with separate broker and controller roles over loopback registers a broker-only node with a controller-only node, restarts it and checks the epoch changes.

### [To Do] API-0777 BrokerRegistration differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0776, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical BrokerRegistration requests to Kafka 4.4.0 and Alluvium for registration at v0-v4 and for each client-triggerable error (INCONSISTENT_CLUSTER_ID and DUPLICATE_BROKER_REGISTRATION); normalized responses match for every supported version.

### [To Do] API-0778 Advertise BrokerRegistration v0-4 in ApiVersions
P0 · M2 · Task · deps: API-0774, API-0777, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the controller listener lists BrokerRegistration with min 0 and max 4 through the generated list owned by API-F18; docs/parity.md marks BrokerRegistration supported; a test asserts the range. BrokerRegistration is listed only on the controller listener. Later tasks (M3, M4) add behavior without changing the advertised range.

### [To Do] API-0779 BrokerRegistration authorization: CLUSTER_ACTION
P0 · M4 · Task · deps: API-0768, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleBrokerRegistration @4.4.0
Done when: Registrations need CLUSTER_ACTION on CLUSTER or return CLUSTER_AUTHORIZATION_FAILED; tests.

### [To Do] API-0780 BrokerRegistration multi-node simulation test
P0 · M3 · Task · deps: API-0776, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) registers, fences and restarts brokers across controller failovers and checks broker epochs are monotonic; a failing seed is printed and replays exactly.

## [To Do] API-F63 BrokerHeartbeat (v0-2)
P0 · M2 · Feature · deps: CTRL-F02
Scope: Kafka 4.4.0 versions 0-2; listeners: controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0781 BrokerHeartbeat handler module and router registration (controller listener)
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleBrokerHeartBeatRequest @4.4.0
Done when: `handlers/broker_heartbeat.rs` (module doc names the Kafka counterpart) decodes v0-2 with the generated codec and calls the cluster-control service (CTRL-F02); it is registered for the controller listener in the router (SRV-F04). A BrokerHeartbeat request on a broker listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0782 BrokerHeartbeat v0: liveness, fencing and shutdown signals
P0 · M2 · Task · deps: API-0781 · ref: clients/src/main/resources/common/message/BrokerHeartbeatRequest.json @4.4.0
Done when: broker_epoch, current_metadata_offset, want_fence and want_shut_down are passed to CTRL-F02 and is_caught_up, is_fenced and should_shut_down are returned; a stale epoch gets STALE_BROKER_EPOCH and an unknown broker BROKER_ID_NOT_REGISTERED; tests.

### [To Do] API-0783 BrokerHeartbeat v1-v2: offline_log_dirs and cordoned_log_dirs (tagged)
P0 · M2 · Task · deps: API-0782 · ref: clients/src/main/resources/common/message/BrokerHeartbeatRequest.json @4.4.0
Done when: v1+ passes offline log directory ids and v2+ cordoned log directory ids (null means unchanged) to CTRL-F20; below those versions the fields are treated as empty; per-version tests.

### [To Do] API-0784 BrokerHeartbeat error mapping
P0 · M2 · Task · deps: API-0781 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleBrokerHeartBeatRequest @4.4.0
Done when: NOT_CONTROLLER, STALE_BROKER_EPOCH, BROKER_ID_NOT_REGISTERED and UNKNOWN_SERVER_ERROR map exhaustively; table test.

### [To Do] API-0785 BrokerHeartbeat request-quota exemption (cluster action)
P0 · M4 · Task · deps: API-0781, QUOTA-F04 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala handleError @4.4.0
Done when: BrokerHeartbeat is a cluster-action API, so it is exempt from request quotas and throttle_time_ms stays 0; only CLUSTER_AUTHORIZATION_FAILED responses may be throttled, as in `RequestHandlerHelper.handleError`; test with a quota-limited principal.

### [To Do] API-0786 BrokerHeartbeat request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0781 · labels: observability · ref: core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for BrokerHeartbeat; the request-log summary (SRV-F09) shows broker id, epoch, metadata offset and fence/shutdown flags. Fence, unfence and controlled-shutdown transitions are emitted by CTRL-F02 with the heartbeat as the reason source. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0787 BrokerHeartbeat handler unit tests across v0-2
P0 · M2 · Task · deps: API-0783, API-0784 · labels: test
Done when: Table-driven tests send every version v0-v2 through the handler, with the cluster-control service (CTRL-F02) replaced by a fake, and check the service call and the encoded response; cases cover caught-up and lagging brokers, fencing, controlled shutdown, stale epochs and log-dir fields.

### [To Do] API-0788 BrokerHeartbeat broker-controller integration test
P0 · M2 · Task · deps: API-0787 · labels: test
Done when: An in-process integration test with separate broker and controller roles over loopback keeps a broker-only node unfenced with heartbeats, then stops heartbeats and observes fencing after `broker.session.timeout.ms`.

### [To Do] API-0789 BrokerHeartbeat differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0788, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical BrokerHeartbeat requests to Kafka 4.4.0 and Alluvium for heartbeats at v0-v2 and for each client-triggerable error (STALE_BROKER_EPOCH and BROKER_ID_NOT_REGISTERED); normalized responses match for every supported version.

### [To Do] API-0790 Advertise BrokerHeartbeat v0-2 in ApiVersions
P0 · M2 · Task · deps: API-0786, API-0789, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the controller listener lists BrokerHeartbeat with min 0 and max 2 through the generated list owned by API-F18; docs/parity.md marks BrokerHeartbeat supported; a test asserts the range. BrokerHeartbeat is listed only on the controller listener. Later tasks (M3, M4) add behavior without changing the advertised range.

### [To Do] API-0791 BrokerHeartbeat authorization: CLUSTER_ACTION
P0 · M4 · Task · deps: API-0781, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleBrokerHeartBeatRequest @4.4.0
Done when: Heartbeats need CLUSTER_ACTION on CLUSTER or return CLUSTER_AUTHORIZATION_FAILED; tests.

### [To Do] API-0792 BrokerHeartbeat multi-node simulation test
P0 · M3 · Task · deps: API-0788, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) drops and delays heartbeats across controller failovers and checks fencing and controlled shutdown follow CTRL-F02 timing; a failing seed is printed and replays exactly.

## [To Do] API-F64 UnregisterBroker (v0)
P1 · M3 · Feature · deps: CTRL-F22
Scope: Kafka 4.4.0 versions 0; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0793 UnregisterBroker handler module and router registration (broker and controller listeners)
P1 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleUnregisterBroker @4.4.0
Done when: `handlers/unregister_broker.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the controller unregistration path (CTRL-F22); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0794 Forward UnregisterBroker from brokers to the active controller in an Envelope
P1 · M3 · Task · deps: API-0793, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener UnregisterBroker is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0795 UnregisterBroker v0: remove a broker registration
P1 · M3 · Task · deps: API-0793 · ref: clients/src/main/resources/common/message/UnregisterBrokerRequest.json @4.4.0
Done when: The controller removes the registration through CTRL-F22; an unknown broker gets BROKER_ID_NOT_REGISTERED and other refusals the CTRL-F22 error, with error_message; tests.

### [To Do] API-0796 UnregisterBroker error mapping
P1 · M3 · Task · deps: API-0793 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleUnregisterBroker @4.4.0
Done when: BROKER_ID_NOT_REGISTERED, NOT_CONTROLLER, UNSUPPORTED_VERSION and CLUSTER_AUTHORIZATION_FAILED map exhaustively with messages; table test.

### [To Do] API-0797 UnregisterBroker throttle_time_ms through the shared response path
P1 · M3 · Task · deps: API-0793 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0798 UnregisterBroker throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0797, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: UnregisterBroker records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0799 UnregisterBroker request metrics, events and request-log summary
P1 · M3 · Task · deps: API-0793 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for UnregisterBroker; the request-log summary (SRV-F09) shows the target broker id and outcome. Unregistration carries reason `unregister_broker_request` (CTRL-F22). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0800 UnregisterBroker handler unit tests at v0
P1 · M3 · Task · deps: API-0794, API-0795, API-0796, API-0797 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the controller unregistration path (CTRL-F22) replaced by a fake, and check the service call and the encoded response; cases cover fenced, unknown and active brokers.

### [To Do] API-0801 UnregisterBroker compatibility test with Java client 4.4.0 Admin.unregisterBroker and kafka-cluster.sh unregister
P1 · M3 · Task · deps: API-0800, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that unregisters a decommissioned broker so it disappears from DescribeCluster passes with Java client 4.4.0 Admin.unregisterBroker and kafka-cluster.sh unregister.

### [To Do] API-0802 UnregisterBroker differential test against Kafka 4.4.0
P1 · M3 · Task · deps: API-0800, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical UnregisterBroker requests to Kafka 4.4.0 and Alluvium for unregistering a stopped broker and for each client-triggerable error (BROKER_ID_NOT_REGISTERED); normalized responses match for every supported version.

### [To Do] API-0803 Advertise UnregisterBroker v0 in ApiVersions
P1 · M3 · Task · deps: API-0799, API-0801, API-0802, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists UnregisterBroker with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks UnregisterBroker supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0804 UnregisterBroker authorization: ALTER on CLUSTER
P1 · M4 · Task · deps: API-0794, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleUnregisterBroker @4.4.0
Done when: Requests need ALTER on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

## [To Do] API-F65 DescribeTransactions (v0)
P1 · M6 · Feature · deps: TXN-F13
Scope: Kafka 4.4.0 versions 0; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0805 DescribeTransactions handler module and router registration (broker listener)
P1 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeTransactionsRequest @4.4.0
Done when: `handlers/describe_transactions.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the transaction coordinator (TXN-F13); it is registered for the broker listener in the router (SRV-F04). A DescribeTransactions request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0806 DescribeTransactions v0: transaction state per transactional id
P1 · M6 · Task · deps: API-0805 · ref: clients/src/main/resources/common/message/DescribeTransactionsResponse.json @4.4.0
Done when: Each id returns state, timeout, start time, producer id and epoch and its partitions from TXN-F13; ids not owned by this coordinator get NOT_COORDINATOR and unknown ids TRANSACTIONAL_ID_NOT_FOUND; tests.

### [To Do] API-0807 DescribeTransactions error mapping
P1 · M6 · Task · deps: API-0805 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeTransactionsRequest @4.4.0
Done when: NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, TRANSACTIONAL_ID_NOT_FOUND and TRANSACTIONAL_ID_AUTHORIZATION_FAILED map exhaustively per id; table test.

### [To Do] API-0808 DescribeTransactions throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M6 · Task · deps: API-0805, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; DescribeTransactions records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0809 DescribeTransactions request metrics, events and request-log summary
P1 · M6 · Task · deps: API-0805 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeTransactions; the request-log summary (SRV-F09) shows transactional id count. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0810 DescribeTransactions authorization: DESCRIBE on ids and topic filtering
P1 · M6 · Task · deps: API-0805, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeTransactionsRequest @4.4.0
Done when: Ids without DESCRIBE on TRANSACTIONAL_ID get TRANSACTIONAL_ID_AUTHORIZATION_FAILED; partitions of topics without DESCRIBE are removed from the returned state; tests and differential cases.

### [To Do] API-0811 DescribeTransactions handler unit tests at v0
P1 · M6 · Task · deps: API-0806, API-0807, API-0808, API-0810 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the transaction coordinator (TXN-F13) replaced by a fake, and check the service call and the encoded response; cases cover ongoing, completed, unknown and foreign transactional ids.

### [To Do] API-0812 DescribeTransactions compatibility test with Java client 4.4.0 Admin.describeTransactions and kafka-transactions.sh describe
P1 · M6 · Task · deps: API-0811, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that describes an ongoing transaction passes with Java client 4.4.0 Admin.describeTransactions and kafka-transactions.sh describe.

### [To Do] API-0813 DescribeTransactions differential test against Kafka 4.4.0
P1 · M6 · Task · deps: API-0811, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeTransactions requests to Kafka 4.4.0 and Alluvium for ongoing and completed transactions and for each client-triggerable error (NOT_COORDINATOR and TRANSACTIONAL_ID_NOT_FOUND); normalized responses match for every supported version.

### [To Do] API-0814 Advertise DescribeTransactions v0 in ApiVersions
P1 · M6 · Task · deps: API-0809, API-0812, API-0813, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists DescribeTransactions with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks DescribeTransactions supported; a test asserts the range.

## [To Do] API-F66 ListTransactions (v0-2)
P1 · M6 · Feature · deps: TXN-F13
Scope: Kafka 4.4.0 versions 0-2; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0815 ListTransactions handler module and router registration (broker listener)
P1 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListTransactionsRequest @4.4.0
Done when: `handlers/list_transactions.rs` (module doc names the Kafka counterpart) decodes v0-2 with the generated codec and calls the transaction coordinator (TXN-F13); it is registered for the broker listener in the router (SRV-F04). A ListTransactions request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0816 ListTransactions v0: state and producer id filters
P1 · M6 · Task · deps: API-0815 · ref: clients/src/main/resources/common/message/ListTransactionsRequest.json @4.4.0
Done when: Transactions on this broker's coordinator partitions are filtered by state_filters and producer_id_filters; unknown state names are returned in unknown_state_filters; COORDINATOR_LOAD_IN_PROGRESS while loading; tests.

### [To Do] API-0817 ListTransactions v1: duration_filter (KIP-994)
P1 · M6 · Task · deps: API-0816 · ref: clients/src/main/resources/common/message/ListTransactionsRequest.json @4.4.0 (KIP-994)
Done when: v1 duration_filter returns only transactions running longer than the given milliseconds (-1 disables it); tests.

### [To Do] API-0818 ListTransactions v2: transactional_id_pattern (KIP-1152)
P1 · M6 · Task · deps: API-0817 · ref: clients/src/main/resources/common/message/ListTransactionsRequest.json @4.4.0 (KIP-1152)
Done when: v2 transactional_id_pattern filters ids by regular expression; an invalid pattern gets INVALID_REGULAR_EXPRESSION; null disables the filter; tests.

### [To Do] API-0819 ListTransactions error mapping
P1 · M6 · Task · deps: API-0815 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListTransactionsRequest @4.4.0
Done when: COORDINATOR_LOAD_IN_PROGRESS, COORDINATOR_NOT_AVAILABLE and INVALID_REGULAR_EXPRESSION (v2) map exhaustively at the top level; table test.

### [To Do] API-0820 ListTransactions throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M6 · Task · deps: API-0815, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; ListTransactions records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0821 ListTransactions request metrics, events and request-log summary
P1 · M6 · Task · deps: API-0815 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ListTransactions; the request-log summary (SRV-F09) shows state, producer id, duration and pattern filters and the count returned. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0822 ListTransactions authorization: DESCRIBE filtering
P1 · M6 · Task · deps: API-0815, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListTransactionsRequest @4.4.0
Done when: Transactions whose id lacks DESCRIBE on TRANSACTIONAL_ID are silently removed from the list; tests and a differential case.

### [To Do] API-0823 ListTransactions handler unit tests across v0-2
P1 · M6 · Task · deps: API-0818, API-0819, API-0820, API-0822 · labels: test
Done when: Table-driven tests send every version v0-v2 through the handler, with the transaction coordinator (TXN-F13) replaced by a fake, and check the service call and the encoded response; cases cover each filter alone and combined, unknown states and invalid patterns.

### [To Do] API-0824 ListTransactions compatibility test with Java client 4.4.0 Admin.listTransactions and kafka-transactions.sh list and find-hanging
P1 · M6 · Task · deps: API-0823, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that lists transactions by state, producer, duration and pattern passes with Java client 4.4.0 Admin.listTransactions and kafka-transactions.sh list and find-hanging.

### [To Do] API-0825 ListTransactions differential test against Kafka 4.4.0
P1 · M6 · Task · deps: API-0823, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ListTransactions requests to Kafka 4.4.0 and Alluvium for listings with every filter at v0-v2 and for each client-triggerable error (INVALID_REGULAR_EXPRESSION and unknown state filters); normalized responses match for every supported version.

### [To Do] API-0826 Advertise ListTransactions v0-2 in ApiVersions
P1 · M6 · Task · deps: API-0821, API-0824, API-0825, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists ListTransactions with min 0 and max 2 through the generated list owned by API-F18; docs/parity.md marks ListTransactions supported; a test asserts the range.

## [To Do] API-F67 AllocateProducerIds (v0)
P0 · M6 · Feature · deps: CTRL-F16
Scope: Kafka 4.4.0 versions 0; listeners: controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0827 AllocateProducerIds handler module and router registration (controller listener)
P0 · M6 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAllocateProducerIdsRequest @4.4.0
Done when: `handlers/allocate_producer_ids.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the controller producer-id path (CTRL-F16); it is registered for the controller listener in the router (SRV-F04). A AllocateProducerIds request on a broker listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0828 AllocateProducerIds v0: allocate a producer id block
P0 · M6 · Task · deps: API-0827 · ref: clients/src/main/resources/common/message/AllocateProducerIdsRequest.json @4.4.0
Done when: The controller allocates the next block through CTRL-F16 after checking broker_id and broker_epoch (STALE_BROKER_EPOCH for a mismatch, BROKER_ID_NOT_REGISTERED for unknown brokers) and returns producer_id_start and producer_id_len; exhausting the id space returns UNKNOWN_SERVER_ERROR as Kafka does; tests.

### [To Do] API-0829 AllocateProducerIds error mapping
P0 · M6 · Task · deps: API-0827 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAllocateProducerIdsRequest @4.4.0
Done when: NOT_CONTROLLER, STALE_BROKER_EPOCH, BROKER_ID_NOT_REGISTERED, CLUSTER_AUTHORIZATION_FAILED and UNKNOWN_SERVER_ERROR map exhaustively; table test.

### [To Do] API-0830 AllocateProducerIds request-quota exemption (cluster action)
P0 · M6 · Task · deps: API-0827, QUOTA-F04 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala handleError @4.4.0
Done when: AllocateProducerIds is a cluster-action API, so it is exempt from request quotas and throttle_time_ms stays 0; only CLUSTER_AUTHORIZATION_FAILED responses may be throttled, as in `RequestHandlerHelper.handleError`; test with a quota-limited principal.

### [To Do] API-0831 AllocateProducerIds request metrics, events and request-log summary
P0 · M6 · Task · deps: API-0827 · labels: observability · ref: core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for AllocateProducerIds; the request-log summary (SRV-F09) shows broker id, epoch and the allocated range. Block allocation emits a controller event with reason `allocate_producer_ids` (CTRL-F16). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0832 AllocateProducerIds authorization: CLUSTER_ACTION
P0 · M6 · Task · deps: API-0827, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAllocateProducerIdsRequest @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; test.

### [To Do] API-0833 AllocateProducerIds handler unit tests at v0
P0 · M6 · Task · deps: API-0828, API-0829, API-0830, API-0832 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the controller producer-id path (CTRL-F16) replaced by a fake, and check the service call and the encoded response; cases cover consecutive blocks, stale epochs and unknown brokers.

### [To Do] API-0834 AllocateProducerIds multi-node simulation test
P0 · M6 · Task · deps: API-0833, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) allocates blocks from several brokers across controller failovers and checks no producer id is handed out twice; a failing seed is printed and replays exactly.

### [To Do] API-0835 AllocateProducerIds differential test against Kafka 4.4.0
P0 · M6 · Task · deps: API-0833, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical AllocateProducerIds requests to Kafka 4.4.0 and Alluvium for allocation requests from a Kafka 4.4.0 broker and for each client-triggerable error (STALE_BROKER_EPOCH); normalized responses match for every supported version.

### [To Do] API-0836 Advertise AllocateProducerIds v0 in ApiVersions
P0 · M6 · Task · deps: API-0831, API-0834, API-0835, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the controller listener lists AllocateProducerIds with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks AllocateProducerIds supported; a test asserts the range. AllocateProducerIds is listed only on the controller listener.

## [To Do] API-F68 ConsumerGroupHeartbeat (v0-1)
P0 · M5 · Feature · deps: GRP-F05
Scope: Kafka 4.4.0 versions 0-1; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0837 ConsumerGroupHeartbeat handler module and router registration (broker listener)
P0 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleConsumerGroupHeartbeat @4.4.0
Done when: `handlers/consumer_group_heartbeat.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the consumer group coordinator (GRP-F05); it is registered for the broker listener in the router (SRV-F04). A ConsumerGroupHeartbeat request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. When group.version does not enable the consumer protocol the handler returns UNSUPPORTED_VERSION. Unit test per listener.

### [To Do] API-0838 ConsumerGroupHeartbeat request validation
P0 · M5 · Task · deps: API-0837 · labels: security · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java throwIfConsumerGroupHeartbeatRequestIsInvalid @4.4.0
Done when: Empty group_id, instance_id or rack_id, a missing member_id when required, a first heartbeat (epoch 0) without rebalance_timeout_ms, with non-empty topic_partitions or with neither subscribed names nor regex get INVALID_REQUEST with Kafka's message, and an unknown server assignor UNSUPPORTED_ASSIGNOR; unit tests.

### [To Do] API-0839 ConsumerGroupHeartbeat v0: server-generated member ids
P0 · M5 · Task · deps: API-0838 · ref: clients/src/main/resources/common/message/ConsumerGroupHeartbeatRequest.json @4.4.0 (KIP-848)
Done when: A v0 member joining with epoch 0 and an empty member_id gets a server-generated id; heartbeats return member epoch, heartbeat interval and the assignment (null when unchanged); tests.

### [To Do] API-0840 ConsumerGroupHeartbeat v1: client-generated member ids and subscribed_topic_regex (KIP-1082)
P0 · M5 · Task · deps: API-0839 · ref: clients/src/main/java/org/apache/kafka/common/requests/ConsumerGroupHeartbeatRequest.java CONSUMER_GENERATED_MEMBER_ID_REQUIRED_VERSION @4.4.0 (KIP-1082)
Done when: From v1 member_id must be set by the client on every heartbeat (INVALID_REQUEST otherwise) and subscribed_topic_regex is accepted, with INVALID_REGULAR_EXPRESSION for bad patterns; tests per version.

### [To Do] API-0841 ConsumerGroupHeartbeat error mapping per version
P0 · M5 · Task · deps: API-0837 · ref: clients/src/main/resources/common/message/ConsumerGroupHeartbeatResponse.json @4.4.0
Done when: NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, INVALID_REQUEST, GROUP_ID_NOT_FOUND, UNKNOWN_MEMBER_ID, FENCED_MEMBER_EPOCH, UNSUPPORTED_ASSIGNOR, UNRELEASED_INSTANCE_ID, GROUP_MAX_SIZE_REACHED, TOPIC_AUTHORIZATION_FAILED and INVALID_REGULAR_EXPRESSION (v1+) map exhaustively with error_message; table test.

### [To Do] API-0842 ConsumerGroupHeartbeat throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M5 · Task · deps: API-0837, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; ConsumerGroupHeartbeat records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0843 ConsumerGroupHeartbeat request metrics, events and request-log summary
P0 · M5 · Task · deps: API-0837 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ConsumerGroupHeartbeat; the request-log summary (SRV-F09) shows group id, member id, member epoch, subscription size or regex and the assignor. Member epoch changes are emitted by GRP-F05 with the heartbeat as the reason source. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0844 ConsumerGroupHeartbeat authorization: READ on GROUP and DESCRIBE on subscribed topics
P0 · M5 · Task · deps: API-0840, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleConsumerGroupHeartbeat @4.4.0
Done when: Without READ on the GROUP the response is GROUP_AUTHORIZATION_FAILED; if any subscribed topic name lacks DESCRIBE the response is TOPIC_AUTHORIZATION_FAILED naming the topics; regex subscriptions are filtered by GRP-F05; tests and differential cases.

### [To Do] API-0845 ConsumerGroupHeartbeat handler unit tests across v0-1
P0 · M5 · Task · deps: API-0841, API-0842, API-0844 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the consumer group coordinator (GRP-F05) replaced by a fake, and check the service call and the encoded response; cases cover joins at v0 and v1, steady heartbeats, leaves (epoch -1 and -2), fencing, regex subscriptions and the disabled-protocol path.

### [To Do] API-0846 ConsumerGroupHeartbeat compatibility test with Java client 4.4.0 consumers with group.protocol=consumer, librdkafka with the consumer protocol and kafka-consumer-groups.sh
P0 · M5 · Task · deps: API-0845, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that forms and rebalances consumer-protocol groups with server-side assignors and regex subscriptions passes with Java client 4.4.0 consumers with group.protocol=consumer, librdkafka with the consumer protocol and kafka-consumer-groups.sh.

### [To Do] API-0847 ConsumerGroupHeartbeat differential test against Kafka 4.4.0
P0 · M5 · Task · deps: API-0845, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ConsumerGroupHeartbeat requests to Kafka 4.4.0 and Alluvium for heartbeats at v0-v1 and for each client-triggerable error (FENCED_MEMBER_EPOCH, UNKNOWN_MEMBER_ID, UNSUPPORTED_ASSIGNOR, INVALID_REQUEST variants and INVALID_REGULAR_EXPRESSION); normalized responses match for every supported version.

### [To Do] API-0848 Advertise ConsumerGroupHeartbeat v0-1 in ApiVersions
P0 · M5 · Task · deps: API-0843, API-0846, API-0847, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists ConsumerGroupHeartbeat with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks ConsumerGroupHeartbeat supported; a test asserts the range. It is advertised regardless of group.version, as in Kafka, and answers UNSUPPORTED_VERSION when the consumer protocol is disabled.

## [To Do] API-F69 ConsumerGroupDescribe (v0-1)
P1 · M5 · Feature · deps: GRP-F08
Scope: Kafka 4.4.0 versions 0-1; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0849 ConsumerGroupDescribe handler module and router registration (broker listener)
P1 · M5 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleConsumerGroupDescribe @4.4.0
Done when: `handlers/consumer_group_describe.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the group administration service (GRP-F08); it is registered for the broker listener in the router (SRV-F04). A ConsumerGroupDescribe request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. When group.version does not enable the consumer protocol the handler returns UNSUPPORTED_VERSION. Unit test per listener.

### [To Do] API-0850 ConsumerGroupDescribe v0: epochs, assignor and member assignments
P1 · M5 · Task · deps: API-0849 · ref: clients/src/main/resources/common/message/ConsumerGroupDescribeResponse.json @4.4.0
Done when: Each group returns state, group and assignment epochs, assignor and members with subscriptions and current and target assignments (topic ids and names); unknown groups and groups that are not consumer groups get GROUP_ID_NOT_FOUND; tests.

### [To Do] API-0851 ConsumerGroupDescribe v1: member_type (KIP-1099)
P1 · M5 · Task · deps: API-0850 · ref: clients/src/main/resources/common/message/ConsumerGroupDescribeResponse.json @4.4.0 (KIP-1099)
Done when: v1 members carry member_type (classic or consumer) so mixed groups during migration are visible; v0 omits it; tests.

### [To Do] API-0852 ConsumerGroupDescribe error mapping
P1 · M5 · Task · deps: API-0849 · ref: clients/src/main/resources/common/message/ConsumerGroupDescribeResponse.json @4.4.0
Done when: Per-group NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, INVALID_REQUEST, INVALID_GROUP_ID, GROUP_ID_NOT_FOUND, GROUP_AUTHORIZATION_FAILED and TOPIC_AUTHORIZATION_FAILED map exhaustively with error_message; table test.

### [To Do] API-0853 ConsumerGroupDescribe throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M5 · Task · deps: API-0849, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; ConsumerGroupDescribe records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0854 ConsumerGroupDescribe request metrics, events and request-log summary
P1 · M5 · Task · deps: API-0849 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ConsumerGroupDescribe; the request-log summary (SRV-F09) shows group count and include_authorized_operations. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0855 ConsumerGroupDescribe authorization: DESCRIBE on groups and their topics
P1 · M5 · Task · deps: API-0849, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleConsumerGroupDescribe @4.4.0
Done when: Groups without DESCRIBE get GROUP_AUTHORIZATION_FAILED; with an authorizer, a group whose members reference a topic the caller cannot DESCRIBE gets TOPIC_AUTHORIZATION_FAILED 'The group has described topic(s) that the client is not authorized to describe.'; authorized_operations is filled when requested; tests and differential cases.

### [To Do] API-0856 ConsumerGroupDescribe handler unit tests across v0-1
P1 · M5 · Task · deps: API-0851, API-0852, API-0853, API-0855 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the group administration service (GRP-F08) replaced by a fake, and check the service call and the encoded response; cases cover stable and reconciling groups, mixed classic and consumer members, unknown groups and topic filtering.

### [To Do] API-0857 ConsumerGroupDescribe compatibility test with Java client 4.4.0 Admin.describeConsumerGroups and kafka-consumer-groups.sh --describe
P1 · M5 · Task · deps: API-0856, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that describes consumer-protocol groups, including one migrating from the classic protocol passes with Java client 4.4.0 Admin.describeConsumerGroups and kafka-consumer-groups.sh --describe.

### [To Do] API-0858 ConsumerGroupDescribe differential test against Kafka 4.4.0
P1 · M5 · Task · deps: API-0856, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ConsumerGroupDescribe requests to Kafka 4.4.0 and Alluvium for descriptions at v0-v1 and for each client-triggerable error (GROUP_ID_NOT_FOUND and INVALID_GROUP_ID); normalized responses match for every supported version.

### [To Do] API-0859 Advertise ConsumerGroupDescribe v0-1 in ApiVersions
P1 · M5 · Task · deps: API-0854, API-0857, API-0858, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists ConsumerGroupDescribe with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks ConsumerGroupDescribe supported; a test asserts the range.

## [To Do] API-F70 ControllerRegistration (v0)
P0 · M3 · Feature · deps: CTRL-F03
Scope: Kafka 4.4.0 versions 0; listeners: controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0860 ControllerRegistration handler module and router registration (controller listener)
P0 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleControllerRegistration @4.4.0
Done when: `handlers/controller_registration.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the controller registration path (CTRL-F03); it is registered for the controller listener in the router (SRV-F04). A ControllerRegistration request on a broker listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0861 ControllerRegistration v0: listeners, features and incarnation id
P0 · M3 · Task · deps: API-0860 · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java registerController @4.4.0
Done when: The active controller records the incarnation id, listeners and supported features through CTRL-F03; zk_migration_ready is ignored and recorded as false; a metadata.version without controller registration support gets UNSUPPORTED_VERSION with a message; tests.

### [To Do] API-0862 ControllerRegistration error mapping
P0 · M3 · Task · deps: API-0860 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleControllerRegistration @4.4.0
Done when: NOT_CONTROLLER, UNSUPPORTED_VERSION and CLUSTER_AUTHORIZATION_FAILED map exhaustively with messages; table test.

### [To Do] API-0863 ControllerRegistration throttle_time_ms through the shared response path
P0 · M3 · Task · deps: API-0860 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0864 ControllerRegistration throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0863, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: ControllerRegistration records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0865 ControllerRegistration request metrics, events and request-log summary
P0 · M3 · Task · deps: API-0860 · labels: observability · ref: core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ControllerRegistration; the request-log summary (SRV-F09) shows controller id, incarnation id and listener names. Registrations carry reason `controller_registration` (CTRL-F03). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0866 ControllerRegistration handler unit tests at v0
P0 · M3 · Task · deps: API-0861, API-0862, API-0863 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the controller registration path (CTRL-F03) replaced by a fake, and check the service call and the encoded response; cases cover first registration, re-registration with a new incarnation and an old metadata.version.

### [To Do] API-0867 ControllerRegistration multi-node simulation test
P0 · M3 · Task · deps: API-0866, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) restarts controllers and checks each controller's endpoints in the metadata image match its latest registration; a failing seed is printed and replays exactly.

### [To Do] API-0868 ControllerRegistration differential test against Kafka 4.4.0
P0 · M3 · Task · deps: API-0866, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ControllerRegistration requests to Kafka 4.4.0 and Alluvium for registration of a new controller with a Kafka 4.4.0 quorum and for each client-triggerable error (UNSUPPORTED_VERSION for an old metadata.version); normalized responses match for every supported version.

### [To Do] API-0869 Advertise ControllerRegistration v0 in ApiVersions
P0 · M3 · Task · deps: API-0865, API-0867, API-0868, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the controller listener lists ControllerRegistration with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks ControllerRegistration supported; a test asserts the range. ControllerRegistration is listed only on the controller listener. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0870 ControllerRegistration authorization: CLUSTER_ACTION
P0 · M4 · Task · deps: API-0860, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleControllerRegistration @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; test.

## [To Do] API-F71 GetTelemetrySubscriptions (v0)
P1 · M8 · Feature · deps: TELEM-F01
Scope: Kafka 4.4.0 versions 0; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0871 GetTelemetrySubscriptions handler module and router registration (broker listener)
P1 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleGetTelemetrySubscriptionsRequest @4.4.0
Done when: `handlers/get_telemetry_subscriptions.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the client metrics manager (TELEM-F01); it is registered for the broker listener in the router (SRV-F04). A GetTelemetrySubscriptions request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Any unexpected failure in the manager is answered with INVALID_REQUEST, as Kafka does. Unit test per listener.

### [To Do] API-0872 GetTelemetrySubscriptions v0: client instance id and matching subscription
P1 · M8 · Task · deps: API-0871 · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java processGetTelemetrySubscriptionRequest @4.4.0 (KIP-714)
Done when: A zero client_instance_id gets a newly generated id; the response carries the matching subscription id, accepted compression types, push interval, telemetry_max_bytes, delta temporality and requested metrics; requests faster than the push interval get THROTTLING_QUOTA_EXCEEDED unless the last error was UNKNOWN_SUBSCRIPTION_ID or UNSUPPORTED_COMPRESSION_TYPE; tests.

### [To Do] API-0873 GetTelemetrySubscriptions error mapping
P1 · M8 · Task · deps: API-0871 · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java @4.4.0
Done when: INVALID_REQUEST and THROTTLING_QUOTA_EXCEEDED map exhaustively; table test.

### [To Do] API-0874 GetTelemetrySubscriptions throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M8 · Task · deps: API-0871, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; GetTelemetrySubscriptions records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0875 GetTelemetrySubscriptions request metrics, events and request-log summary
P1 · M8 · Task · deps: API-0871 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for GetTelemetrySubscriptions; the request-log summary (SRV-F09) shows client instance id, subscription id and matched selectors. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0876 GetTelemetrySubscriptions handler unit tests at v0
P1 · M8 · Task · deps: API-0872, API-0873, API-0874 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the client metrics manager (TELEM-F01) replaced by a fake, and check the service call and the encoded response; cases cover new and known instances, subscription changes and early requests.

### [To Do] API-0877 GetTelemetrySubscriptions compatibility test with Java client 4.4.0 with enable.metrics.push and librdkafka
P1 · M8 · Task · deps: API-0876, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that obtains a subscription and pushes on schedule after kafka-client-metrics.sh --alter creates one passes with Java client 4.4.0 with enable.metrics.push and librdkafka.

### [To Do] API-0878 GetTelemetrySubscriptions differential test against Kafka 4.4.0
P1 · M8 · Task · deps: API-0876, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical GetTelemetrySubscriptions requests to Kafka 4.4.0 and Alluvium for subscription requests with and without a matching subscription and for each client-triggerable error (THROTTLING_QUOTA_EXCEEDED); normalized responses match for every supported version.

### [To Do] API-0879 Advertise GetTelemetrySubscriptions v0 in ApiVersions
P1 · M8 · Task · deps: API-0875, API-0877, API-0878, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists GetTelemetrySubscriptions with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks GetTelemetrySubscriptions supported; a test asserts the range. As in Kafka, it is listed only when a telemetry exporter is configured (the API-F18 gating task).

## [To Do] API-F72 PushTelemetry (v0)
P1 · M8 · Feature · deps: TELEM-F02
Scope: Kafka 4.4.0 versions 0; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0880 PushTelemetry handler module and router registration (broker listener)
P1 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handlePushTelemetryRequest @4.4.0
Done when: `handlers/push_telemetry.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the client metrics manager (TELEM-F02); it is registered for the broker listener in the router (SRV-F04). A PushTelemetry request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Any unexpected failure in the manager is answered with INVALID_REQUEST, as Kafka does. Unit test per listener.

### [To Do] API-0881 PushTelemetry v0: validate and accept a metrics payload
P1 · M8 · Task · deps: API-0880 · labels: security · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java processPushTelemetryRequest @4.4.0 (KIP-714)
Done when: Pushes are checked in Kafka's order: unknown client instance (INVALID_REQUEST), too early (THROTTLING_QUOTA_EXCEEDED), stale subscription id (UNKNOWN_SUBSCRIPTION_ID), compression not accepted (UNSUPPORTED_COMPRESSION_TYPE), payload above telemetry_max_bytes (TELEMETRY_TOO_LARGE), undecodable payload (INVALID_RECORD); terminating pushes are accepted once early; accepted payloads go to TELEM-F04 export; tests per check.

### [To Do] API-0882 PushTelemetry error mapping
P1 · M8 · Task · deps: API-0880 · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java @4.4.0
Done when: INVALID_REQUEST, THROTTLING_QUOTA_EXCEEDED, UNKNOWN_SUBSCRIPTION_ID, UNSUPPORTED_COMPRESSION_TYPE, TELEMETRY_TOO_LARGE and INVALID_RECORD map exhaustively; table test.

### [To Do] API-0883 PushTelemetry throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M8 · Task · deps: API-0880, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; PushTelemetry records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0884 PushTelemetry request metrics, events and request-log summary
P1 · M8 · Task · deps: API-0880 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for PushTelemetry; the request-log summary (SRV-F09) shows client instance id, subscription id, compression type and payload size, never metric values. Rejected pushes emit an event with the error as reason (TELEM-F05). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0885 PushTelemetry payload fuzz target
P1 · M8 · Task · deps: API-0881, TEST-F07 · labels: security, decoder, test · ref: server/src/main/java/org/apache/kafka/server/ClientMetricsManager.java @4.4.0
Done when: A fuzz target feeds arbitrary compressed and uncompressed payloads through the handler and the TELEM-F02 decoder; no panic and memory stays bounded by telemetry_max_bytes.

### [To Do] API-0886 PushTelemetry handler unit tests at v0
P1 · M8 · Task · deps: API-0882, API-0883, API-0885 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the client metrics manager (TELEM-F02) replaced by a fake, and check the service call and the encoded response; cases cover accepted pushes, every rejection, terminating pushes and compression types.

### [To Do] API-0887 PushTelemetry compatibility test with Java client 4.4.0 with enable.metrics.push and librdkafka
P1 · M8 · Task · deps: API-0886, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that pushes metrics that reach the configured exporter passes with Java client 4.4.0 with enable.metrics.push and librdkafka.

### [To Do] API-0888 PushTelemetry differential test against Kafka 4.4.0
P1 · M8 · Task · deps: API-0886, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical PushTelemetry requests to Kafka 4.4.0 and Alluvium for accepted pushes and for each client-triggerable error (UNKNOWN_SUBSCRIPTION_ID, TELEMETRY_TOO_LARGE and THROTTLING_QUOTA_EXCEEDED); normalized responses match for every supported version.

### [To Do] API-0889 Advertise PushTelemetry v0 in ApiVersions
P1 · M8 · Task · deps: API-0884, API-0887, API-0888, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists PushTelemetry with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks PushTelemetry supported; a test asserts the range. As in Kafka, it is listed only when a telemetry exporter is configured (the API-F18 gating task).

## [To Do] API-F73 AssignReplicasToDirs (v0)
P1 · M7 · Feature · deps: CTRL-F20
Scope: Kafka 4.4.0 versions 0; listeners: controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0890 AssignReplicasToDirs handler module and router registration (controller listener)
P1 · M7 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAssignReplicasToDirs @4.4.0
Done when: `handlers/assign_replicas_to_dirs.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the controller directory assignment path (CTRL-F20); it is registered for the controller listener in the router (SRV-F04). A AssignReplicasToDirs request on a broker listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0891 AssignReplicasToDirs v0: record replica placement per directory
P1 · M7 · Task · deps: API-0890 · ref: clients/src/main/resources/common/message/AssignReplicasToDirsRequest.json @4.4.0 (KIP-858)
Done when: The broker's assignments of partitions (topic id and index) to directory ids are recorded through CTRL-F20 after checking broker_id and broker_epoch; unknown topics or partitions and replicas not on the broker get per-partition errors from CTRL-F20; tests.

### [To Do] API-0892 AssignReplicasToDirs error mapping
P1 · M7 · Task · deps: API-0890 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAssignReplicasToDirs @4.4.0
Done when: Top-level NOT_CONTROLLER, STALE_BROKER_EPOCH, BROKER_ID_NOT_REGISTERED, UNSUPPORTED_VERSION and CLUSTER_AUTHORIZATION_FAILED and partition-level UNKNOWN_TOPIC_ID, UNKNOWN_TOPIC_OR_PARTITION and NOT_LEADER_OR_FOLLOWER map exhaustively; table test.

### [To Do] API-0893 AssignReplicasToDirs throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M7 · Task · deps: API-0890, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; AssignReplicasToDirs records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0894 AssignReplicasToDirs request metrics, events and request-log summary
P1 · M7 · Task · deps: API-0890 · labels: observability · ref: core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for AssignReplicasToDirs; the request-log summary (SRV-F09) shows broker id, directory count and partition count. Assignments carry reason `assign_replicas_to_dirs` (CTRL-F20). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0895 AssignReplicasToDirs authorization: CLUSTER_ACTION
P1 · M7 · Task · deps: API-0890, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAssignReplicasToDirs @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; test.

### [To Do] API-0896 AssignReplicasToDirs handler unit tests at v0
P1 · M7 · Task · deps: API-0891, API-0892, API-0893, API-0895 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the controller directory assignment path (CTRL-F20) replaced by a fake, and check the service call and the encoded response; cases cover new assignments, moves, unknown partitions and stale epochs.

### [To Do] API-0897 AssignReplicasToDirs multi-node simulation test
P1 · M7 · Task · deps: API-0896, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) fails a log directory on a JBOD broker and checks its replicas are reassigned and the metadata image matches; a failing seed is printed and replays exactly.

### [To Do] API-0898 AssignReplicasToDirs differential test against Kafka 4.4.0
P1 · M7 · Task · deps: API-0896, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical AssignReplicasToDirs requests to Kafka 4.4.0 and Alluvium for assignments from a Kafka 4.4.0 JBOD broker and for each client-triggerable error (STALE_BROKER_EPOCH and unknown topic ids); normalized responses match for every supported version.

### [To Do] API-0899 Advertise AssignReplicasToDirs v0 in ApiVersions
P1 · M7 · Task · deps: API-0894, API-0897, API-0898, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the controller listener lists AssignReplicasToDirs with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks AssignReplicasToDirs supported; a test asserts the range. AssignReplicasToDirs is listed only on the controller listener.

## [To Do] API-F74 ListConfigResources (v0-1)
P1 · M8 · Feature · deps: TELEM-F03
Scope: Kafka 4.4.0 versions 0-1; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0900 ListConfigResources handler module and router registration (broker listener)
P1 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListConfigResources @4.4.0
Done when: `handlers/list_config_resources.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the config resource listing (TELEM-F03); it is registered for the broker listener in the router (SRV-F04). A ListConfigResources request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0901 ListConfigResources v0: client-metrics resources (formerly ListClientMetricsResources)
P1 · M8 · Task · deps: API-0900 · ref: clients/src/main/resources/common/message/ListConfigResourcesRequest.json @4.4.0
Done when: v0 requests have no resource_types and return every client-metrics subscription name; tests.

### [To Do] API-0902 ListConfigResources v1: resource_types filter (KIP-1142)
P1 · M8 · Task · deps: API-0901 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListConfigResources @4.4.0 (KIP-1142)
Done when: v1 resource_types selects GROUP, CLIENT_METRICS, BROKER_LOGGER, BROKER and TOPIC resources (empty means all supported types) and each result carries resource_type; an unsupported type returns top-level UNSUPPORTED_VERSION; tests.

### [To Do] API-0903 ListConfigResources error mapping
P1 · M8 · Task · deps: API-0900 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListConfigResources @4.4.0
Done when: UNSUPPORTED_VERSION and CLUSTER_AUTHORIZATION_FAILED map exhaustively at the top level; table test.

### [To Do] API-0904 ListConfigResources throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M8 · Task · deps: API-0900, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; ListConfigResources records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0905 ListConfigResources request metrics, events and request-log summary
P1 · M8 · Task · deps: API-0900 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ListConfigResources; the request-log summary (SRV-F09) shows requested resource types and the count returned. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0906 ListConfigResources authorization: DESCRIBE_CONFIGS on CLUSTER
P1 · M8 · Task · deps: API-0900, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListConfigResources @4.4.0
Done when: Requests need DESCRIBE_CONFIGS on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

### [To Do] API-0907 ListConfigResources handler unit tests across v0-1
P1 · M8 · Task · deps: API-0902, API-0903, API-0904, API-0906 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the config resource listing (TELEM-F03) replaced by a fake, and check the service call and the encoded response; cases cover v0 listing, each resource type, all types and unsupported types.

### [To Do] API-0908 ListConfigResources compatibility test with Java client 4.4.0 Admin.listConfigResources and kafka-client-metrics.sh --list
P1 · M8 · Task · deps: API-0907, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that lists client-metrics subscriptions and other config resources passes with Java client 4.4.0 Admin.listConfigResources and kafka-client-metrics.sh --list.

### [To Do] API-0909 ListConfigResources differential test against Kafka 4.4.0
P1 · M8 · Task · deps: API-0907, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ListConfigResources requests to Kafka 4.4.0 and Alluvium for listings at v0-v1 and for each client-triggerable error (UNSUPPORTED_VERSION for unsupported types); normalized responses match for every supported version.

### [To Do] API-0910 Advertise ListConfigResources v0-1 in ApiVersions
P1 · M8 · Task · deps: API-0905, API-0908, API-0909, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists ListConfigResources with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks ListConfigResources supported; a test asserts the range.

## [To Do] API-F75 DescribeTopicPartitions (v0)
P0 · M2 · Feature · deps: META-F09
Scope: Kafka 4.4.0 versions 0; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0911 DescribeTopicPartitions handler module and router registration (broker listener)
P0 · M2 · Task · deps: none · ref: core/src/main/java/kafka/server/handlers/DescribeTopicPartitionsRequestHandler.java handleDescribeTopicPartitionsRequest @4.4.0
Done when: `handlers/describe_topic_partitions.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the broker metadata lookups (META-F09); it is registered for the broker listener in the router (SRV-F04). A DescribeTopicPartitions request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0912 DescribeTopicPartitions paging: response_partition_limit and cursor
P0 · M2 · Task · deps: API-0911 · labels: security · ref: core/src/main/java/kafka/server/handlers/DescribeTopicPartitionsRequestHandler.java @4.4.0
Done when: Partitions are returned in topic-name then partition order up to response_partition_limit (capped by `max.request.partition.size.limit`); next_cursor points at the first partition not returned; a cursor topic missing from the request gets INVALID_REQUEST; tests across page boundaries.

### [To Do] API-0913 DescribeTopicPartitions v0: partition fields with ELR, last known ELR and offline replicas
P0 · M2 · Task · deps: API-0911 · ref: clients/src/main/resources/common/message/DescribeTopicPartitionsResponse.json @4.4.0 (KIP-966)
Done when: Each partition returns leader, leader_epoch, replicas, isr, eligible_leader_replicas, last_known_elr and offline_replicas from the metadata image; an empty topic list describes every topic; unknown topics get UNKNOWN_TOPIC_OR_PARTITION; tests.

### [To Do] API-0914 DescribeTopicPartitions error mapping
P0 · M2 · Task · deps: API-0912 · ref: core/src/main/java/kafka/server/handlers/DescribeTopicPartitionsRequestHandler.java @4.4.0
Done when: Topic errors UNKNOWN_TOPIC_OR_PARTITION, INVALID_TOPIC_EXCEPTION and TOPIC_AUTHORIZATION_FAILED and the INVALID_REQUEST cursor error map exhaustively; table test.

### [To Do] API-0915 DescribeTopicPartitions throttle_time_ms through the shared response path
P0 · M2 · Task · deps: API-0911 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0916 DescribeTopicPartitions throttling under the request quota (KIP-219)
P0 · M4 · Task · deps: API-0915, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: DescribeTopicPartitions records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0917 DescribeTopicPartitions request metrics, events and request-log summary
P0 · M2 · Task · deps: API-0911 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeTopicPartitions; the request-log summary (SRV-F09) shows topic count, partition limit and whether a cursor was supplied. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0918 DescribeTopicPartitions handler unit tests at v0
P0 · M2 · Task · deps: API-0913, API-0914, API-0915 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the broker metadata lookups (META-F09) replaced by a fake, and check the service call and the encoded response; cases cover single and multi-page results, all-topics requests, unknown topics and ELR fields.

### [To Do] API-0919 DescribeTopicPartitions compatibility test with Java client 4.4.0 Admin.describeTopics and kafka-topics.sh --describe
P0 · M2 · Task · deps: API-0918, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that describes a topic with more partitions than one page and shows ELR fields passes with Java client 4.4.0 Admin.describeTopics and kafka-topics.sh --describe.

### [To Do] API-0920 DescribeTopicPartitions differential test against Kafka 4.4.0
P0 · M2 · Task · deps: API-0918, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeTopicPartitions requests to Kafka 4.4.0 and Alluvium for paged descriptions and for each client-triggerable error (UNKNOWN_TOPIC_OR_PARTITION and an invalid cursor); normalized responses match for every supported version.

### [To Do] API-0921 Advertise DescribeTopicPartitions v0 in ApiVersions
P0 · M2 · Task · deps: API-0917, API-0919, API-0920, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists DescribeTopicPartitions with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks DescribeTopicPartitions supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0922 DescribeTopicPartitions authorization: DESCRIBE on topics without disclosure
P0 · M4 · Task · deps: API-0913, SEC-F08 · labels: security · ref: core/src/main/java/kafka/server/handlers/DescribeTopicPartitionsRequestHandler.java @4.4.0
Done when: Topics without DESCRIBE are omitted when listing all topics and return TOPIC_AUTHORIZATION_FAILED with a zero topic id when named; topic_authorized_operations is filled for authorized topics; tests and differential cases.

## [To Do] API-F76 ShareGroupHeartbeat (v1)
P0 · M8 · Feature · deps: SHARE-F01
Scope: Kafka 4.4.0 versions 1; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0923 ShareGroupHeartbeat handler module and router registration (broker listener)
P0 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleShareGroupHeartbeat @4.4.0
Done when: `handlers/share_group_heartbeat.rs` (module doc names the Kafka counterpart) decodes v1 with the generated codec and calls the share group coordinator (SHARE-F01); it is registered for the broker listener in the router (SRV-F04). A ShareGroupHeartbeat request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. When share groups are disabled (share.version) the handler returns UNSUPPORTED_VERSION. Unit test per listener.

### [To Do] API-0924 ShareGroupHeartbeat request validation
P0 · M8 · Task · deps: API-0923 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleShareGroupHeartbeat @4.4.0
Done when: An invalid member id gets INVALID_REQUEST 'Invalid member id in the request.'; empty group id, missing subscriptions on the first heartbeat and other shape errors get INVALID_REQUEST from SHARE-F01 with a message; unit tests.

### [To Do] API-0925 ShareGroupHeartbeat v1: membership, epochs and assignments (KIP-932)
P0 · M8 · Task · deps: API-0924 · ref: clients/src/main/resources/common/message/ShareGroupHeartbeatRequest.json @4.4.0 (KIP-932)
Done when: v1 (the first stable version; v0 was early access and is rejected) passes member id, epoch, rack and subscriptions to SHARE-F01 and returns member epoch, heartbeat interval and the assignment; leaving uses epoch -1; tests.

### [To Do] API-0926 ShareGroupHeartbeat error mapping
P0 · M8 · Task · deps: API-0923 · ref: clients/src/main/resources/common/message/ShareGroupHeartbeatResponse.json @4.4.0
Done when: NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, GROUP_ID_NOT_FOUND, UNKNOWN_MEMBER_ID, GROUP_MAX_SIZE_REACHED, INVALID_REQUEST, GROUP_AUTHORIZATION_FAILED and TOPIC_AUTHORIZATION_FAILED map exhaustively with messages; table test.

### [To Do] API-0927 ShareGroupHeartbeat throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M8 · Task · deps: API-0923, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; ShareGroupHeartbeat records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0928 ShareGroupHeartbeat request metrics, events and request-log summary
P0 · M8 · Task · deps: API-0923 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ShareGroupHeartbeat; the request-log summary (SRV-F09) shows group id, member id, member epoch and subscription size. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0929 ShareGroupHeartbeat authorization: READ on GROUP and DESCRIBE on subscribed topics
P0 · M8 · Task · deps: API-0925, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleShareGroupHeartbeat @4.4.0
Done when: Without READ on GROUP the response is GROUP_AUTHORIZATION_FAILED; subscribed topics lacking DESCRIBE yield TOPIC_AUTHORIZATION_FAILED naming them; tests and differential cases.

### [To Do] API-0930 ShareGroupHeartbeat handler unit tests at v1
P0 · M8 · Task · deps: API-0926, API-0927, API-0929 · labels: test
Done when: Table-driven tests send v1 requests through the handler, with the share group coordinator (SHARE-F01) replaced by a fake, and check the service call and the encoded response; cases cover joins, steady heartbeats, leaves, fencing, disabled share groups and invalid member ids.

### [To Do] API-0931 ShareGroupHeartbeat compatibility test with Java client 4.4.0 KafkaShareConsumer and kafka-console-share-consumer.sh
P0 · M8 · Task · deps: API-0930, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that forms a share group of several consumers and rebalances when one leaves passes with Java client 4.4.0 KafkaShareConsumer and kafka-console-share-consumer.sh.

### [To Do] API-0932 ShareGroupHeartbeat differential test against Kafka 4.4.0
P0 · M8 · Task · deps: API-0930, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ShareGroupHeartbeat requests to Kafka 4.4.0 and Alluvium for heartbeats at v1 and for each client-triggerable error (UNKNOWN_MEMBER_ID, INVALID_REQUEST and UNSUPPORTED_VERSION when disabled); normalized responses match for every supported version.

### [To Do] API-0933 Advertise ShareGroupHeartbeat v1 in ApiVersions
P0 · M8 · Task · deps: API-0928, API-0931, API-0932, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists ShareGroupHeartbeat with min 1 and max 1 through the generated list owned by API-F18; docs/parity.md marks ShareGroupHeartbeat supported; a test asserts the range.

## [To Do] API-F77 ShareGroupDescribe (v1)
P1 · M8 · Feature · deps: SHARE-F06
Scope: Kafka 4.4.0 versions 1; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0934 ShareGroupDescribe handler module and router registration (broker listener)
P1 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleShareGroupDescribe @4.4.0
Done when: `handlers/share_group_describe.rs` (module doc names the Kafka counterpart) decodes v1 with the generated codec and calls the share group administration (SHARE-F06); it is registered for the broker listener in the router (SRV-F04). A ShareGroupDescribe request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. When share groups are disabled (share.version) the handler returns UNSUPPORTED_VERSION. Unit test per listener.

### [To Do] API-0935 ShareGroupDescribe v1: state, epochs, assignor and members
P1 · M8 · Task · deps: API-0934 · ref: clients/src/main/resources/common/message/ShareGroupDescribeResponse.json @4.4.0 (KIP-932)
Done when: Each group returns state, group and assignment epochs, assignor and members with subscriptions and assignments (topic ids and names); unknown or non-share groups get GROUP_ID_NOT_FOUND; v0 early-access requests are rejected; tests.

### [To Do] API-0936 ShareGroupDescribe error mapping
P1 · M8 · Task · deps: API-0934 · ref: clients/src/main/resources/common/message/ShareGroupDescribeResponse.json @4.4.0
Done when: Per-group NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, INVALID_GROUP_ID, GROUP_ID_NOT_FOUND, INVALID_REQUEST, GROUP_AUTHORIZATION_FAILED and TOPIC_AUTHORIZATION_FAILED map exhaustively with messages; table test.

### [To Do] API-0937 ShareGroupDescribe throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M8 · Task · deps: API-0934, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; ShareGroupDescribe records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0938 ShareGroupDescribe request metrics, events and request-log summary
P1 · M8 · Task · deps: API-0934 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ShareGroupDescribe; the request-log summary (SRV-F09) shows group count and include_authorized_operations. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0939 ShareGroupDescribe authorization: DESCRIBE on groups and their topics
P1 · M8 · Task · deps: API-0934, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleShareGroupDescribe @4.4.0
Done when: Groups without DESCRIBE get GROUP_AUTHORIZATION_FAILED; with an authorizer, groups referencing topics the caller cannot DESCRIBE get TOPIC_AUTHORIZATION_FAILED with Kafka's message; authorized_operations is filled when requested; tests and differential cases.

### [To Do] API-0940 ShareGroupDescribe handler unit tests at v1
P1 · M8 · Task · deps: API-0935, API-0936, API-0937, API-0939 · labels: test
Done when: Table-driven tests send v1 requests through the handler, with the share group administration (SHARE-F06) replaced by a fake, and check the service call and the encoded response; cases cover stable and empty groups, unknown groups and topic filtering.

### [To Do] API-0941 ShareGroupDescribe compatibility test with Java client 4.4.0 Admin.describeShareGroups and kafka-share-groups.sh --describe --members
P1 · M8 · Task · deps: API-0940, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that describes share groups and their members passes with Java client 4.4.0 Admin.describeShareGroups and kafka-share-groups.sh --describe --members.

### [To Do] API-0942 ShareGroupDescribe differential test against Kafka 4.4.0
P1 · M8 · Task · deps: API-0940, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ShareGroupDescribe requests to Kafka 4.4.0 and Alluvium for descriptions at v1 and for each client-triggerable error (GROUP_ID_NOT_FOUND and INVALID_GROUP_ID); normalized responses match for every supported version.

### [To Do] API-0943 Advertise ShareGroupDescribe v1 in ApiVersions
P1 · M8 · Task · deps: API-0938, API-0941, API-0942, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists ShareGroupDescribe with min 1 and max 1 through the generated list owned by API-F18; docs/parity.md marks ShareGroupDescribe supported; a test asserts the range.

## [To Do] API-F78 ShareFetch (v1-2)
P0 · M8 · Feature · deps: SHARE-F03
Scope: Kafka 4.4.0 versions 1-2; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0944 ShareFetch handler module and router registration (broker listener)
P0 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleShareFetchRequest @4.4.0
Done when: `handlers/share_fetch.rs` (module doc names the Kafka counterpart) decodes v1-2 with the generated codec and calls the share fetch service (SHARE-F03); it is registered for the broker listener in the router (SRV-F04). A ShareFetch request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. When share groups are disabled (share.version) the handler returns UNSUPPORTED_VERSION. Unit test per listener.

### [To Do] API-0945 ShareFetch request validation
P0 · M8 · Task · deps: API-0944 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala validateAcknowledgementBatches @4.4.0
Done when: A null group id or an invalid member id gets INVALID_REQUEST with Kafka's message; piggy-backed acknowledgement batches are validated (first <= last, non-overlapping, non-empty types, one type per offset when several, types in range) and invalid partitions get INVALID_REQUEST; unit tests per rule.

### [To Do] API-0946 ShareFetch share sessions: epochs, forgotten topics and limits
P0 · M8 · Task · deps: API-0945 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleShareFetchRequest @4.4.0
Done when: share_session_epoch selects open (0), incremental or close (-1) through SHARE-F03; SHARE_SESSION_NOT_FOUND, INVALID_SHARE_SESSION_EPOCH and SHARE_SESSION_LIMIT_REACHED are top-level errors; forgotten topics leave the session; unknown topic ids get UNKNOWN_TOPIC_ID; tests.

### [To Do] API-0947 ShareFetch v1: record acquisition with max_records and batch_size (KIP-932)
P0 · M8 · Task · deps: API-0946 · ref: clients/src/main/resources/common/message/ShareFetchRequest.json @4.4.0 (KIP-932)
Done when: v1 (the first stable version) passes max_wait_ms, min_bytes, max_bytes, max_records and batch_size to SHARE-F03 and returns records with acquired_records ranges and delivery counts plus acquisition_lock_timeout_ms; leader errors fill current_leader and node_endpoints; tests.

### [To Do] API-0948 ShareFetch v2: share_acquire_mode and renew acknowledgements (KIP-1206, KIP-1222)
P0 · M8 · Task · deps: API-0947 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleShareFetchRequest @4.4.0 (KIP-1206, KIP-1222)
Done when: v2 share_acquire_mode selects the acquisition mode and is_renew_ack allows acknowledge type 4 (renew); a renew fetch must set max_bytes, min_bytes, max_records and max_wait_ms to 0 or gets INVALID_REQUEST listing the offending fields; v1 rejects type 4; tests per version.

### [To Do] API-0949 ShareFetch error mapping
P0 · M8 · Task · deps: API-0944 · ref: clients/src/main/resources/common/message/ShareFetchResponse.json @4.4.0
Done when: Top-level and per-partition error_code and acknowledge_error_code follow the supported-error list of `ShareFetchResponse.json` (INVALID_RECORD_STATE only for acknowledgements); NOT_LEADER_OR_FOLLOWER and FENCED_LEADER_EPOCH fill current_leader; table test.

### [To Do] API-0950 ShareFetch throttle_time_ms and throttling under the fetch byte-rate and request quotas (KIP-219)
P0 · M8 · Task · deps: API-0944, QUOTA-F03, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; ShareFetch records usage against the fetch byte-rate and request quotas and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. The recorded bandwidth is un-recorded when the response is throttled, as for Fetch. Test with a quota-limited principal.

### [To Do] API-0951 ShareFetch request metrics, events and request-log summary
P0 · M8 · Task · deps: API-0944 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ShareFetch; the request-log summary (SRV-F09) shows group id, member id, session epoch, partition count, max_records and acknowledgement count. Session creation, eviction and limit rejections emit events with reasons (SHARE-F03). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0952 ShareFetch authorization: READ on GROUP and READ on topics
P0 · M8 · Task · deps: API-0946, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleShareFetchRequest @4.4.0
Done when: Without READ on GROUP the response is GROUP_AUTHORIZATION_FAILED; partitions of topics without READ get TOPIC_AUTHORIZATION_FAILED for fetch and acknowledgement; tests and differential cases.

### [To Do] API-0953 ShareFetch handler unit tests across v1-2
P0 · M8 · Task · deps: API-0948, API-0949, API-0950, API-0952 · labels: test
Done when: Table-driven tests send every version v1-v2 through the handler, with the share fetch service (SHARE-F03) replaced by a fake, and check the service call and the encoded response; cases cover session lifecycle, acquisition limits, piggy-backed acknowledgements, renew fetches and every validation rule.

### [To Do] API-0954 ShareFetch compatibility test with Java client 4.4.0 KafkaShareConsumer (implicit and explicit acknowledgement) and kafka-console-share-consumer.sh
P0 · M8 · Task · deps: API-0953, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that consumes a topic with several share consumers, redelivers released records and renews locks passes with Java client 4.4.0 KafkaShareConsumer (implicit and explicit acknowledgement) and kafka-console-share-consumer.sh.

### [To Do] API-0955 ShareFetch differential test against Kafka 4.4.0
P0 · M8 · Task · deps: API-0953, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ShareFetch requests to Kafka 4.4.0 and Alluvium for share fetches at v1-v2 and for each client-triggerable error (SHARE_SESSION_NOT_FOUND, INVALID_SHARE_SESSION_EPOCH, INVALID_REQUEST for bad batches and renew fields, and UNKNOWN_TOPIC_ID); normalized responses match for every supported version.

### [To Do] API-0956 Advertise ShareFetch v1-2 in ApiVersions
P0 · M8 · Task · deps: API-0951, API-0954, API-0955, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists ShareFetch with min 1 and max 2 through the generated list owned by API-F18; docs/parity.md marks ShareFetch supported; a test asserts the range.

## [To Do] API-F79 ShareAcknowledge (v1-2)
P0 · M8 · Feature · deps: SHARE-F04
Scope: Kafka 4.4.0 versions 1-2; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0957 ShareAcknowledge handler module and router registration (broker listener)
P0 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleShareAcknowledgeRequest @4.4.0
Done when: `handlers/share_acknowledge.rs` (module doc names the Kafka counterpart) decodes v1-2 with the generated codec and calls the share acknowledgement service (SHARE-F04); it is registered for the broker listener in the router (SRV-F04). A ShareAcknowledge request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. When share groups are disabled (share.version) the handler returns UNSUPPORTED_VERSION. Unit test per listener.

### [To Do] API-0958 ShareAcknowledge request validation
P0 · M8 · Task · deps: API-0957 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala validateAcknowledgementBatches @4.4.0
Done when: A null group id or invalid member id gets INVALID_REQUEST; acknowledgement batches are validated as for ShareFetch (ordering, non-empty types, one type per offset, type range, type 4 only with is_renew_ack) and invalid partitions get INVALID_REQUEST; unit tests per rule.

### [To Do] API-0959 ShareAcknowledge v1: accept, release and reject (KIP-932)
P0 · M8 · Task · deps: API-0958 · ref: clients/src/main/resources/common/message/ShareAcknowledgeRequest.json @4.4.0 (KIP-932)
Done when: Acknowledgement types 1-3 are applied through SHARE-F04 within the share session; INVALID_RECORD_STATE is returned for records not acquired by the member; leader errors fill current_leader and node_endpoints; tests.

### [To Do] API-0960 ShareAcknowledge v2: renew acknowledgements (KIP-1222)
P0 · M8 · Task · deps: API-0959 · ref: clients/src/main/resources/common/message/ShareAcknowledgeRequest.json @4.4.0 (KIP-1222)
Done when: With is_renew_ack (v2+) type 4 renews acquisition locks and the response returns acquisition_lock_timeout_ms; v1 requests with type 4 get INVALID_REQUEST; tests per version.

### [To Do] API-0961 ShareAcknowledge error mapping
P0 · M8 · Task · deps: API-0957 · ref: clients/src/main/resources/common/message/ShareAcknowledgeResponse.json @4.4.0
Done when: The supported-error list of `ShareAcknowledgeResponse.json` is enforced at top and partition level; table test.

### [To Do] API-0962 ShareAcknowledge throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M8 · Task · deps: API-0957, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; ShareAcknowledge records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-0963 ShareAcknowledge request metrics, events and request-log summary
P0 · M8 · Task · deps: API-0957 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ShareAcknowledge; the request-log summary (SRV-F09) shows group id, member id, session epoch and counts per acknowledgement type. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0964 ShareAcknowledge authorization: READ on GROUP and READ on topics
P0 · M8 · Task · deps: API-0959, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleShareAcknowledgeRequest @4.4.0
Done when: Without READ on GROUP the response is GROUP_AUTHORIZATION_FAILED; partitions of topics without READ get TOPIC_AUTHORIZATION_FAILED; tests and differential cases.

### [To Do] API-0965 ShareAcknowledge handler unit tests across v1-2
P0 · M8 · Task · deps: API-0960, API-0961, API-0962, API-0964 · labels: test
Done when: Table-driven tests send every version v1-v2 through the handler, with the share acknowledgement service (SHARE-F04) replaced by a fake, and check the service call and the encoded response; cases cover each acknowledgement type, invalid batches, records not acquired, renewals and closed sessions.

### [To Do] API-0966 ShareAcknowledge compatibility test with Java client 4.4.0 KafkaShareConsumer with explicit acknowledgement
P0 · M8 · Task · deps: API-0965, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that accepts, releases and rejects records and observes redelivery counts passes with Java client 4.4.0 KafkaShareConsumer with explicit acknowledgement.

### [To Do] API-0967 ShareAcknowledge differential test against Kafka 4.4.0
P0 · M8 · Task · deps: API-0965, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ShareAcknowledge requests to Kafka 4.4.0 and Alluvium for acknowledgements at v1-v2 and for each client-triggerable error (INVALID_RECORD_STATE, INVALID_REQUEST and SHARE_SESSION_NOT_FOUND); normalized responses match for every supported version.

### [To Do] API-0968 Advertise ShareAcknowledge v1-2 in ApiVersions
P0 · M8 · Task · deps: API-0963, API-0966, API-0967, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists ShareAcknowledge with min 1 and max 2 through the generated list owned by API-F18; docs/parity.md marks ShareAcknowledge supported; a test asserts the range.

## [To Do] API-F80 AddRaftVoter (v0-1)
P1 · M3 · Feature · deps: RAFT-F07
Scope: Kafka 4.4.0 versions 0-1; listeners: controller, broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0969 AddRaftVoter handler module and router registration (broker and controller listeners)
P1 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAddRaftVoter @4.4.0
Done when: `handlers/add_raft_voter.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the Raft dynamic-quorum path (RAFT-F07); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0970 Forward AddRaftVoter from brokers to the active controller in an Envelope
P1 · M3 · Task · deps: API-0969, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener AddRaftVoter is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0971 AddRaftVoter validation: cluster id, voter key and endpoints
P1 · M3 · Task · deps: API-0969 · labels: security · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleAddVoterRequest @4.4.0
Done when: A mismatched cluster_id gets INCONSISTENT_CLUSTER_ID, a non-leader controller NOT_LEADER_OR_FOLLOWER, a missing directory id or no endpoint for the controller listener INVALID_REQUEST and an existing voter DUPLICATE_VOTER; unit tests.

### [To Do] API-0972 AddRaftVoter v0: respond after the new voter set commits
P1 · M3 · Task · deps: API-0971 · ref: clients/src/main/resources/common/message/AddRaftVoterRequest.json @4.4.0 (KIP-853)
Done when: v0 responses are sent once the voter-set record commits, or with REQUEST_TIMED_OUT when timeout_ms expires; a change while another is pending gets the RAFT-F07 error; tests.

### [To Do] API-0973 AddRaftVoter v1: ack_when_committed
P1 · M3 · Task · deps: API-0972 · ref: clients/src/main/resources/common/message/AddRaftVoterRequest.json @4.4.0
Done when: With ack_when_committed false (v1+) the response is sent after the leader writes the change locally; v0 always behaves as true; per-version tests.

### [To Do] API-0974 AddRaftVoter error mapping
P1 · M3 · Task · deps: API-0969 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleAddVoterRequest @4.4.0
Done when: INCONSISTENT_CLUSTER_ID, NOT_LEADER_OR_FOLLOWER, INVALID_REQUEST, DUPLICATE_VOTER, REQUEST_TIMED_OUT, UNSUPPORTED_VERSION (kraft.version 0) and CLUSTER_AUTHORIZATION_FAILED map exhaustively with messages; table test.

### [To Do] API-0975 AddRaftVoter throttle_time_ms through the shared response path
P1 · M3 · Task · deps: API-0969 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0976 AddRaftVoter throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0975, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: AddRaftVoter records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0977 AddRaftVoter request metrics, events and request-log summary
P1 · M3 · Task · deps: API-0969 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for AddRaftVoter; the request-log summary (SRV-F09) shows voter id, directory id and ack_when_committed. Voter-set changes carry reason `add_raft_voter_request` (RAFT-F07). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0978 AddRaftVoter handler unit tests across v0-1
P1 · M3 · Task · deps: API-0970, API-0973, API-0974, API-0975 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the Raft dynamic-quorum path (RAFT-F07) replaced by a fake, and check the service call and the encoded response; cases cover adding a caught-up observer, duplicates, missing endpoints, timeouts and both ack modes.

### [To Do] API-0979 AddRaftVoter compatibility test with Java client 4.4.0 Admin.addRaftVoter and kafka-metadata-quorum.sh add-controller
P1 · M3 · Task · deps: API-0978, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that adds a new controller to a 3-voter quorum through a broker and through --bootstrap-controller passes with Java client 4.4.0 Admin.addRaftVoter and kafka-metadata-quorum.sh add-controller.

### [To Do] API-0980 AddRaftVoter multi-node simulation test
P1 · M3 · Task · deps: API-0978, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) adds voters while the leader fails over and checks the voter set changes one member at a time and stays committed; a failing seed is printed and replays exactly.

### [To Do] API-0981 AddRaftVoter differential test against Kafka 4.4.0
P1 · M3 · Task · deps: API-0978, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical AddRaftVoter requests to Kafka 4.4.0 and Alluvium for adding a voter at v0-v1 and for each client-triggerable error (DUPLICATE_VOTER, INCONSISTENT_CLUSTER_ID and missing endpoints); normalized responses match for every supported version.

### [To Do] API-0982 Advertise AddRaftVoter v0-1 in ApiVersions
P1 · M3 · Task · deps: API-0977, API-0979, API-0980, API-0981, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists AddRaftVoter with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks AddRaftVoter supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0983 AddRaftVoter authorization: ALTER on CLUSTER
P1 · M4 · Task · deps: API-0970, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleAddRaftVoter @4.4.0
Done when: Requests need ALTER on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

## [To Do] API-F81 RemoveRaftVoter (v0)
P1 · M3 · Feature · deps: RAFT-F07
Scope: Kafka 4.4.0 versions 0; listeners: controller, broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0984 RemoveRaftVoter handler module and router registration (broker and controller listeners)
P1 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleRemoveRaftVoter @4.4.0
Done when: `handlers/remove_raft_voter.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the Raft dynamic-quorum path (RAFT-F07); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-0985 Forward RemoveRaftVoter from brokers to the active controller in an Envelope
P1 · M3 · Task · deps: API-0984, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener RemoveRaftVoter is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-0986 RemoveRaftVoter v0: remove a voter by id and directory id
P1 · M3 · Task · deps: API-0984 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleRemoveVoterRequest @4.4.0
Done when: The leader removes the voter through RAFT-F07 and responds once the change commits; an unknown voter key gets VOTER_NOT_FOUND, a mismatched cluster_id INCONSISTENT_CLUSTER_ID, a non-leader NOT_LEADER_OR_FOLLOWER and an uncommitted change REQUEST_TIMED_OUT; tests.

### [To Do] API-0987 RemoveRaftVoter error mapping
P1 · M3 · Task · deps: API-0984 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleRemoveVoterRequest @4.4.0
Done when: VOTER_NOT_FOUND, INCONSISTENT_CLUSTER_ID, NOT_LEADER_OR_FOLLOWER, REQUEST_TIMED_OUT, INVALID_REQUEST, UNSUPPORTED_VERSION and CLUSTER_AUTHORIZATION_FAILED map exhaustively with messages; table test.

### [To Do] API-0988 RemoveRaftVoter throttle_time_ms through the shared response path
P1 · M3 · Task · deps: API-0984 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-0989 RemoveRaftVoter throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-0988, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: RemoveRaftVoter records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-0990 RemoveRaftVoter request metrics, events and request-log summary
P1 · M3 · Task · deps: API-0984 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for RemoveRaftVoter; the request-log summary (SRV-F09) shows voter id and directory id. Voter-set changes carry reason `remove_raft_voter_request` (RAFT-F07). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-0991 RemoveRaftVoter handler unit tests at v0
P1 · M3 · Task · deps: API-0985, API-0986, API-0987, API-0988 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the Raft dynamic-quorum path (RAFT-F07) replaced by a fake, and check the service call and the encoded response; cases cover removing a follower, removing the leader, unknown voters and timeouts.

### [To Do] API-0992 RemoveRaftVoter compatibility test with Java client 4.4.0 Admin.removeRaftVoter and kafka-metadata-quorum.sh remove-controller
P1 · M3 · Task · deps: API-0991, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that removes a controller from a 4-voter quorum passes with Java client 4.4.0 Admin.removeRaftVoter and kafka-metadata-quorum.sh remove-controller.

### [To Do] API-0993 RemoveRaftVoter multi-node simulation test
P1 · M3 · Task · deps: API-0991, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) removes voters, including the leader, under message loss and checks the quorum keeps a single leader; a failing seed is printed and replays exactly.

### [To Do] API-0994 RemoveRaftVoter differential test against Kafka 4.4.0
P1 · M3 · Task · deps: API-0991, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical RemoveRaftVoter requests to Kafka 4.4.0 and Alluvium for removing a voter and for each client-triggerable error (VOTER_NOT_FOUND and INCONSISTENT_CLUSTER_ID); normalized responses match for every supported version.

### [To Do] API-0995 Advertise RemoveRaftVoter v0 in ApiVersions
P1 · M3 · Task · deps: API-0990, API-0992, API-0993, API-0994, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists RemoveRaftVoter with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks RemoveRaftVoter supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-0996 RemoveRaftVoter authorization: ALTER on CLUSTER
P1 · M4 · Task · deps: API-0985, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleRemoveRaftVoter @4.4.0
Done when: Requests need ALTER on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.

## [To Do] API-F82 UpdateRaftVoter (v0)
P1 · M3 · Feature · deps: RAFT-F07
Scope: Kafka 4.4.0 versions 0; listeners: controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-0997 UpdateRaftVoter handler module and router registration (controller listener)
P1 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleUpdateRaftVoter @4.4.0
Done when: `handlers/update_raft_voter.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the Raft dynamic-quorum path (RAFT-F07); it is registered for the controller listener in the router (SRV-F04). A UpdateRaftVoter request on a broker listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. Unit test per listener.

### [To Do] API-0998 UpdateRaftVoter v0: update a voter's endpoints and supported kraft.version
P1 · M3 · Task · deps: API-0997 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleUpdateVoterRequest @4.4.0
Done when: The leader updates the voter's listeners and kraft.version range through RAFT-F07; a mismatched cluster_id gets INCONSISTENT_CLUSTER_ID, a stale current_leader_epoch FENCED_LEADER_EPOCH, an unknown voter VOTER_NOT_FOUND, and a missing directory id, a missing controller-listener endpoint or an unsupported kraft.version range INVALID_REQUEST; responses carry the tagged current_leader with host and port; tests.

### [To Do] API-0999 UpdateRaftVoter error mapping
P1 · M3 · Task · deps: API-0997 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleUpdateVoterRequest @4.4.0
Done when: INCONSISTENT_CLUSTER_ID, FENCED_LEADER_EPOCH, NOT_LEADER_OR_FOLLOWER, VOTER_NOT_FOUND, INVALID_REQUEST and CLUSTER_AUTHORIZATION_FAILED map exhaustively; table test.

### [To Do] API-1000 UpdateRaftVoter throttle_time_ms through the shared response path
P1 · M3 · Task · deps: API-0997 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-1001 UpdateRaftVoter throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-1000, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: UpdateRaftVoter records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-1002 UpdateRaftVoter request metrics, events and request-log summary
P1 · M3 · Task · deps: API-0997 · labels: observability · ref: core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for UpdateRaftVoter; the request-log summary (SRV-F09) shows voter id, directory id and listener names. Endpoint updates carry reason `update_raft_voter_request` (RAFT-F07). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1003 UpdateRaftVoter handler unit tests at v0
P1 · M3 · Task · deps: API-0998, API-0999, API-1000 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the Raft dynamic-quorum path (RAFT-F07) replaced by a fake, and check the service call and the encoded response; cases cover endpoint changes, unknown voters, stale epochs and unsupported versions.

### [To Do] API-1004 UpdateRaftVoter multi-node simulation test
P1 · M3 · Task · deps: API-1003, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) restarts voters with new addresses and checks the voter set converges on the new endpoints; a failing seed is printed and replays exactly.

### [To Do] API-1005 UpdateRaftVoter differential test against Kafka 4.4.0
P1 · M3 · Task · deps: API-1003, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical UpdateRaftVoter requests to Kafka 4.4.0 and Alluvium for endpoint updates from a Kafka 4.4.0 voter and for each client-triggerable error (VOTER_NOT_FOUND and FENCED_LEADER_EPOCH); normalized responses match for every supported version.

### [To Do] API-1006 Advertise UpdateRaftVoter v0 in ApiVersions
P1 · M3 · Task · deps: API-1002, API-1004, API-1005, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the controller listener lists UpdateRaftVoter with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks UpdateRaftVoter supported; a test asserts the range. UpdateRaftVoter is listed only on the controller listener. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-1007 UpdateRaftVoter authorization: CLUSTER_ACTION
P1 · M4 · Task · deps: API-0997, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleUpdateRaftVoter @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; test.

## [To Do] API-F83 InitializeShareGroupState (v0)
P0 · M8 · Feature · deps: SHARE-F05
Scope: Kafka 4.4.0 versions 0; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-1008 InitializeShareGroupState handler module and router registration (broker listener)
P0 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleInitializeShareGroupStateRequest @4.4.0
Done when: `handlers/initialize_share_group_state.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the share coordinator (SHARE-F05); it is registered for the broker listener in the router (SRV-F04). A InitializeShareGroupState request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. These are inter-broker requests to the share coordinator on the broker listener. Unit test per listener.

### [To Do] API-1009 InitializeShareGroupState v0: per-partition state operation
P0 · M8 · Task · deps: API-1008 · ref: clients/src/main/resources/common/message/InitializeShareGroupStateRequest.json @4.4.0 (KIP-932)
Done when: The share coordinator initializes state epoch and start offset per partition; FENCED_STATE_EPOCH for an older state epoch; partitions not owned by this coordinator get NOT_COORDINATOR; each result carries error_message; tests.

### [To Do] API-1010 InitializeShareGroupState error mapping
P0 · M8 · Task · deps: API-1008 · ref: clients/src/main/resources/common/message/InitializeShareGroupStateResponse.json @4.4.0
Done when: The supported-error list of `InitializeShareGroupStateResponse.json` (NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, INVALID_REQUEST and the fencing errors) plus CLUSTER_AUTHORIZATION_FAILED map exhaustively per partition; table test.

### [To Do] API-1011 InitializeShareGroupState request metrics, events and request-log summary
P0 · M8 · Task · deps: API-1008 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for InitializeShareGroupState; the request-log summary (SRV-F09) shows group id, partition count and epochs. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1012 InitializeShareGroupState authorization: CLUSTER_ACTION
P0 · M8 · Task · deps: API-1008, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleInitializeShareGroupStateRequest @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or every partition gets CLUSTER_AUTHORIZATION_FAILED; test.

### [To Do] API-1013 InitializeShareGroupState quota exemption
P0 · M8 · Task · deps: API-1008, QUOTA-F04 · ref: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java @4.4.0
Done when: InitializeShareGroupState is a cluster-action API without throttle_time_ms; responses are never throttled for quota-limited principals; test.

### [To Do] API-1014 InitializeShareGroupState handler unit tests at v0
P0 · M8 · Task · deps: API-1009, API-1010, API-1012, API-1013 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the share coordinator (SHARE-F05) replaced by a fake, and check the service call and the encoded response; cases cover owned and foreign partitions, fencing and unknown state.

### [To Do] API-1015 InitializeShareGroupState multi-node simulation test
P0 · M8 · Task · deps: API-1014, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) moves share-coordinator leadership while share consumers acknowledge and checks persisted state matches the acknowledgements; a failing seed is printed and replays exactly.

### [To Do] API-1016 InitializeShareGroupState differential test against Kafka 4.4.0
P0 · M8 · Task · deps: API-1014, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical InitializeShareGroupState requests to Kafka 4.4.0 and Alluvium for requests sent by a Kafka 4.4.0 share-partition leader and for each client-triggerable error (NOT_COORDINATOR and the fencing errors); normalized responses match for every supported version.

### [To Do] API-1017 Advertise InitializeShareGroupState v0 in ApiVersions
P0 · M8 · Task · deps: API-1011, API-1015, API-1016, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists InitializeShareGroupState with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks InitializeShareGroupState supported; a test asserts the range.

## [To Do] API-F84 ReadShareGroupState (v0)
P0 · M8 · Feature · deps: SHARE-F05
Scope: Kafka 4.4.0 versions 0; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-1018 ReadShareGroupState handler module and router registration (broker listener)
P0 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleReadShareGroupStateRequest @4.4.0
Done when: `handlers/read_share_group_state.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the share coordinator (SHARE-F05); it is registered for the broker listener in the router (SRV-F04). A ReadShareGroupState request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. These are inter-broker requests to the share coordinator on the broker listener. Unit test per listener.

### [To Do] API-1019 ReadShareGroupState v0: per-partition state operation
P0 · M8 · Task · deps: API-1018 · ref: clients/src/main/resources/common/message/ReadShareGroupStateRequest.json @4.4.0 (KIP-932)
Done when: The share coordinator returns state epoch, start offset and state batches; FENCED_LEADER_EPOCH for a stale leader epoch and GROUP_ID_NOT_FOUND or UNKNOWN_TOPIC_OR_PARTITION for unknown state; partitions not owned by this coordinator get NOT_COORDINATOR; each result carries error_message; tests.

### [To Do] API-1020 ReadShareGroupState error mapping
P0 · M8 · Task · deps: API-1018 · ref: clients/src/main/resources/common/message/ReadShareGroupStateResponse.json @4.4.0
Done when: The supported-error list of `ReadShareGroupStateResponse.json` (NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, INVALID_REQUEST and the fencing errors) plus CLUSTER_AUTHORIZATION_FAILED map exhaustively per partition; table test.

### [To Do] API-1021 ReadShareGroupState request metrics, events and request-log summary
P0 · M8 · Task · deps: API-1018 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ReadShareGroupState; the request-log summary (SRV-F09) shows group id, partition count and epochs. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1022 ReadShareGroupState authorization: CLUSTER_ACTION
P0 · M8 · Task · deps: API-1018, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleReadShareGroupStateRequest @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or every partition gets CLUSTER_AUTHORIZATION_FAILED; test.

### [To Do] API-1023 ReadShareGroupState quota exemption
P0 · M8 · Task · deps: API-1018, QUOTA-F04 · ref: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java @4.4.0
Done when: ReadShareGroupState is a cluster-action API without throttle_time_ms; responses are never throttled for quota-limited principals; test.

### [To Do] API-1024 ReadShareGroupState handler unit tests at v0
P0 · M8 · Task · deps: API-1019, API-1020, API-1022, API-1023 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the share coordinator (SHARE-F05) replaced by a fake, and check the service call and the encoded response; cases cover owned and foreign partitions, fencing and unknown state.

### [To Do] API-1025 ReadShareGroupState multi-node simulation test
P0 · M8 · Task · deps: API-1024, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) moves share-coordinator leadership while share consumers acknowledge and checks persisted state matches the acknowledgements; a failing seed is printed and replays exactly.

### [To Do] API-1026 ReadShareGroupState differential test against Kafka 4.4.0
P0 · M8 · Task · deps: API-1024, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ReadShareGroupState requests to Kafka 4.4.0 and Alluvium for requests sent by a Kafka 4.4.0 share-partition leader and for each client-triggerable error (NOT_COORDINATOR and the fencing errors); normalized responses match for every supported version.

### [To Do] API-1027 Advertise ReadShareGroupState v0 in ApiVersions
P0 · M8 · Task · deps: API-1021, API-1025, API-1026, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists ReadShareGroupState with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks ReadShareGroupState supported; a test asserts the range.

## [To Do] API-F85 WriteShareGroupState (v0-1)
P0 · M8 · Feature · deps: SHARE-F05
Scope: Kafka 4.4.0 versions 0-1; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-1028 WriteShareGroupState handler module and router registration (broker listener)
P0 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleWriteShareGroupStateRequest @4.4.0
Done when: `handlers/write_share_group_state.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the share coordinator (SHARE-F05); it is registered for the broker listener in the router (SRV-F04). A WriteShareGroupState request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. These are inter-broker requests to the share coordinator on the broker listener. Unit test per listener.

### [To Do] API-1029 WriteShareGroupState v0: per-partition state operation
P0 · M8 · Task · deps: API-1028 · ref: clients/src/main/resources/common/message/WriteShareGroupStateRequest.json @4.4.0 (KIP-932)
Done when: The share coordinator persists start offset and state batches; FENCED_LEADER_EPOCH and FENCED_STATE_EPOCH reject stale writers; partitions not owned by this coordinator get NOT_COORDINATOR; each result carries error_message; tests.

### [To Do] API-1030 WriteShareGroupState v1: delivery_complete_count (KIP-1226)
P0 · M8 · Task · deps: API-1029 · ref: clients/src/main/resources/common/message/WriteShareGroupStateRequest.json @4.4.0 (KIP-1226)
Done when: v1 writes carry delivery_complete_count per partition, stored for lag reporting; v0 writes store -1; tests per version.

### [To Do] API-1031 WriteShareGroupState error mapping
P0 · M8 · Task · deps: API-1028 · ref: clients/src/main/resources/common/message/WriteShareGroupStateResponse.json @4.4.0
Done when: The supported-error list of `WriteShareGroupStateResponse.json` (NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, INVALID_REQUEST and the fencing errors) plus CLUSTER_AUTHORIZATION_FAILED map exhaustively per partition; table test.

### [To Do] API-1032 WriteShareGroupState request metrics, events and request-log summary
P0 · M8 · Task · deps: API-1028 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for WriteShareGroupState; the request-log summary (SRV-F09) shows group id, partition count and epochs. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1033 WriteShareGroupState authorization: CLUSTER_ACTION
P0 · M8 · Task · deps: API-1028, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleWriteShareGroupStateRequest @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or every partition gets CLUSTER_AUTHORIZATION_FAILED; test.

### [To Do] API-1034 WriteShareGroupState quota exemption
P0 · M8 · Task · deps: API-1028, QUOTA-F04 · ref: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java @4.4.0
Done when: WriteShareGroupState is a cluster-action API without throttle_time_ms; responses are never throttled for quota-limited principals; test.

### [To Do] API-1035 WriteShareGroupState handler unit tests across v0-1
P0 · M8 · Task · deps: API-1030, API-1031, API-1033, API-1034 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the share coordinator (SHARE-F05) replaced by a fake, and check the service call and the encoded response; cases cover owned and foreign partitions, fencing and unknown state.

### [To Do] API-1036 WriteShareGroupState multi-node simulation test
P0 · M8 · Task · deps: API-1035, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) moves share-coordinator leadership while share consumers acknowledge and checks persisted state matches the acknowledgements; a failing seed is printed and replays exactly.

### [To Do] API-1037 WriteShareGroupState differential test against Kafka 4.4.0
P0 · M8 · Task · deps: API-1035, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical WriteShareGroupState requests to Kafka 4.4.0 and Alluvium for requests sent by a Kafka 4.4.0 share-partition leader and for each client-triggerable error (NOT_COORDINATOR and the fencing errors); normalized responses match for every supported version.

### [To Do] API-1038 Advertise WriteShareGroupState v0-1 in ApiVersions
P0 · M8 · Task · deps: API-1032, API-1036, API-1037, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists WriteShareGroupState with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks WriteShareGroupState supported; a test asserts the range.

## [To Do] API-F86 DeleteShareGroupState (v0)
P0 · M8 · Feature · deps: SHARE-F05
Scope: Kafka 4.4.0 versions 0; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-1039 DeleteShareGroupState handler module and router registration (broker listener)
P0 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteShareGroupStateRequest @4.4.0
Done when: `handlers/delete_share_group_state.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the share coordinator (SHARE-F05); it is registered for the broker listener in the router (SRV-F04). A DeleteShareGroupState request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. These are inter-broker requests to the share coordinator on the broker listener. Unit test per listener.

### [To Do] API-1040 DeleteShareGroupState v0: per-partition state operation
P0 · M8 · Task · deps: API-1039 · ref: clients/src/main/resources/common/message/DeleteShareGroupStateRequest.json @4.4.0 (KIP-932)
Done when: The share coordinator deletes the persisted state of the listed partitions; FENCED_STATE_EPOCH and GROUP_ID_NOT_FOUND as Kafka returns them; partitions not owned by this coordinator get NOT_COORDINATOR; each result carries error_message; tests.

### [To Do] API-1041 DeleteShareGroupState error mapping
P0 · M8 · Task · deps: API-1039 · ref: clients/src/main/resources/common/message/DeleteShareGroupStateResponse.json @4.4.0
Done when: The supported-error list of `DeleteShareGroupStateResponse.json` (NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, INVALID_REQUEST and the fencing errors) plus CLUSTER_AUTHORIZATION_FAILED map exhaustively per partition; table test.

### [To Do] API-1042 DeleteShareGroupState request metrics, events and request-log summary
P0 · M8 · Task · deps: API-1039 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DeleteShareGroupState; the request-log summary (SRV-F09) shows group id, partition count and epochs. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1043 DeleteShareGroupState authorization: CLUSTER_ACTION
P0 · M8 · Task · deps: API-1039, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteShareGroupStateRequest @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or every partition gets CLUSTER_AUTHORIZATION_FAILED; test.

### [To Do] API-1044 DeleteShareGroupState quota exemption
P0 · M8 · Task · deps: API-1039, QUOTA-F04 · ref: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java @4.4.0
Done when: DeleteShareGroupState is a cluster-action API without throttle_time_ms; responses are never throttled for quota-limited principals; test.

### [To Do] API-1045 DeleteShareGroupState handler unit tests at v0
P0 · M8 · Task · deps: API-1040, API-1041, API-1043, API-1044 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the share coordinator (SHARE-F05) replaced by a fake, and check the service call and the encoded response; cases cover owned and foreign partitions, fencing and unknown state.

### [To Do] API-1046 DeleteShareGroupState multi-node simulation test
P0 · M8 · Task · deps: API-1045, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) moves share-coordinator leadership while share consumers acknowledge and checks persisted state matches the acknowledgements; a failing seed is printed and replays exactly.

### [To Do] API-1047 DeleteShareGroupState differential test against Kafka 4.4.0
P0 · M8 · Task · deps: API-1045, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DeleteShareGroupState requests to Kafka 4.4.0 and Alluvium for requests sent by a Kafka 4.4.0 share-partition leader and for each client-triggerable error (NOT_COORDINATOR and the fencing errors); normalized responses match for every supported version.

### [To Do] API-1048 Advertise DeleteShareGroupState v0 in ApiVersions
P0 · M8 · Task · deps: API-1042, API-1046, API-1047, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists DeleteShareGroupState with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks DeleteShareGroupState supported; a test asserts the range.

## [To Do] API-F87 ReadShareGroupStateSummary (v0-1)
P0 · M8 · Feature · deps: SHARE-F05
Scope: Kafka 4.4.0 versions 0-1; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-1049 ReadShareGroupStateSummary handler module and router registration (broker listener)
P0 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleReadShareGroupStateSummaryRequest @4.4.0
Done when: `handlers/read_share_group_state_summary.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the share coordinator (SHARE-F05); it is registered for the broker listener in the router (SRV-F04). A ReadShareGroupStateSummary request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. These are inter-broker requests to the share coordinator on the broker listener. Unit test per listener.

### [To Do] API-1050 ReadShareGroupStateSummary v0: per-partition state operation
P0 · M8 · Task · deps: API-1049 · ref: clients/src/main/resources/common/message/ReadShareGroupStateSummaryRequest.json @4.4.0 (KIP-932)
Done when: The share coordinator returns state epoch, leader epoch and start offset without batches; FENCED_LEADER_EPOCH for stale readers; partitions not owned by this coordinator get NOT_COORDINATOR; each result carries error_message; tests.

### [To Do] API-1051 ReadShareGroupStateSummary v1: delivery_complete_count (KIP-1226)
P0 · M8 · Task · deps: API-1050 · ref: clients/src/main/resources/common/message/ReadShareGroupStateSummaryResponse.json @4.4.0 (KIP-1226)
Done when: v1 results return delivery_complete_count (-1 when unknown); v0 omits it; tests per version.

### [To Do] API-1052 ReadShareGroupStateSummary error mapping
P0 · M8 · Task · deps: API-1049 · ref: clients/src/main/resources/common/message/ReadShareGroupStateSummaryResponse.json @4.4.0
Done when: The supported-error list of `ReadShareGroupStateSummaryResponse.json` (NOT_COORDINATOR, COORDINATOR_NOT_AVAILABLE, COORDINATOR_LOAD_IN_PROGRESS, INVALID_REQUEST and the fencing errors) plus CLUSTER_AUTHORIZATION_FAILED map exhaustively per partition; table test.

### [To Do] API-1053 ReadShareGroupStateSummary request metrics, events and request-log summary
P0 · M8 · Task · deps: API-1049 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for ReadShareGroupStateSummary; the request-log summary (SRV-F09) shows group id, partition count and epochs. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1054 ReadShareGroupStateSummary authorization: CLUSTER_ACTION
P0 · M8 · Task · deps: API-1049, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleReadShareGroupStateSummaryRequest @4.4.0
Done when: Requests need CLUSTER_ACTION on CLUSTER or every partition gets CLUSTER_AUTHORIZATION_FAILED; test.

### [To Do] API-1055 ReadShareGroupStateSummary quota exemption
P0 · M8 · Task · deps: API-1049, QUOTA-F04 · ref: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java @4.4.0
Done when: ReadShareGroupStateSummary is a cluster-action API without throttle_time_ms; responses are never throttled for quota-limited principals; test.

### [To Do] API-1056 ReadShareGroupStateSummary handler unit tests across v0-1
P0 · M8 · Task · deps: API-1051, API-1052, API-1054, API-1055 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the share coordinator (SHARE-F05) replaced by a fake, and check the service call and the encoded response; cases cover owned and foreign partitions, fencing and unknown state.

### [To Do] API-1057 ReadShareGroupStateSummary multi-node simulation test
P0 · M8 · Task · deps: API-1056, TEST-F06 · labels: test
Done when: A deterministic simulation (TEST-F06) moves share-coordinator leadership while share consumers acknowledge and checks persisted state matches the acknowledgements; a failing seed is printed and replays exactly.

### [To Do] API-1058 ReadShareGroupStateSummary differential test against Kafka 4.4.0
P0 · M8 · Task · deps: API-1056, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical ReadShareGroupStateSummary requests to Kafka 4.4.0 and Alluvium for requests sent by a Kafka 4.4.0 share-partition leader and for each client-triggerable error (NOT_COORDINATOR and the fencing errors); normalized responses match for every supported version.

### [To Do] API-1059 Advertise ReadShareGroupStateSummary v0-1 in ApiVersions
P0 · M8 · Task · deps: API-1053, API-1057, API-1058, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists ReadShareGroupStateSummary with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks ReadShareGroupStateSummary supported; a test asserts the range.

## [To Do] API-F88 StreamsGroupHeartbeat (v0-1)
P0 · M8 · Feature · deps: STRM-F01
Scope: Kafka 4.4.0 versions 0-1; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-1060 StreamsGroupHeartbeat handler module and router registration (broker listener)
P0 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleStreamsGroupHeartbeat @4.4.0
Done when: `handlers/streams_group_heartbeat.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the streams group coordinator (STRM-F01); it is registered for the broker listener in the router (SRV-F04). A StreamsGroupHeartbeat request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. When streams groups are disabled (streams.version) the handler returns UNSUPPORTED_VERSION. Unit test per listener.

### [To Do] API-1061 StreamsGroupHeartbeat topology checks in the handler
P0 · M8 · Task · deps: API-1060 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleStreamsGroupHeartbeat @4.4.0
Done when: Topologies that use Kafka internal topics or invalid topic names get STREAMS_INVALID_TOPOLOGY with Kafka's message before reaching the coordinator; unit tests.

### [To Do] API-1062 StreamsGroupHeartbeat creation of missing internal topics
P0 · M8 · Task · deps: API-1061, API-F03 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleStreamsGroupHeartbeat @4.4.0
Done when: Internal topics the coordinator reports as missing are created through the auto-topic creation manager (API-F03) and the MISSING_INTERNAL_TOPICS status detail reports creation failures; tests.

### [To Do] API-1063 StreamsGroupHeartbeat v0: membership, tasks, endpoints and status codes (KIP-1071)
P0 · M8 · Task · deps: API-1061 · ref: clients/src/main/resources/common/message/StreamsGroupHeartbeatResponse.json @4.4.0 (KIP-1071)
Done when: v0 passes topology, tasks, process id, endpoints, client tags and task offsets to STRM-F01 and returns epochs, intervals, acceptable_recovery_lag (the int32 v0 field), assigned tasks, endpoint-to-partition maps and status codes 0-5; tests.

### [To Do] API-1064 StreamsGroupHeartbeat v1: topology_description_required and MISSING_CLIENT_TAGS (KIP-1331)
P0 · M8 · Task · deps: API-1063 · ref: clients/src/main/resources/common/message/StreamsGroupHeartbeatResponse.json @4.4.0 (KIP-1331)
Done when: v1 responses carry topology_description_required and the int64 acceptable_recovery_lag, and may return status code 6 MISSING_CLIENT_TAGS, which is never sent to v0 members (KAFKA-20744); tests per version.

### [To Do] API-1065 StreamsGroupHeartbeat error mapping
P0 · M8 · Task · deps: API-1060 · ref: clients/src/main/resources/common/message/StreamsGroupHeartbeatResponse.json @4.4.0
Done when: The supported-error list of `StreamsGroupHeartbeatResponse.json` (including STREAMS_INVALID_TOPOLOGY, STREAMS_INVALID_TOPOLOGY_EPOCH and STREAMS_TOPOLOGY_FENCED) maps exhaustively with error_message; table test.

### [To Do] API-1066 StreamsGroupHeartbeat throttle_time_ms and throttling under the request quota (KIP-219)
P0 · M8 · Task · deps: API-1060, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; StreamsGroupHeartbeat records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-1067 StreamsGroupHeartbeat request metrics, events and request-log summary
P0 · M8 · Task · deps: API-1060 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for StreamsGroupHeartbeat; the request-log summary (SRV-F09) shows group id, member id, member epoch, topology epoch and task counts. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1068 StreamsGroupHeartbeat authorization: GROUP, topics and internal-topic creation
P0 · M8 · Task · deps: API-1062, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleStreamsGroupHeartbeat @4.4.0
Done when: Without READ on GROUP the response is GROUP_AUTHORIZATION_FAILED; topology topics lacking DESCRIBE yield TOPIC_AUTHORIZATION_FAILED; internal topics are created only with CREATE on CLUSTER or the TOPIC, otherwise the status detail says 'Unauthorized to CREATE on topics ...'; tests and differential cases.

### [To Do] API-1069 StreamsGroupHeartbeat handler unit tests across v0-1
P0 · M8 · Task · deps: API-1064, API-1065, API-1066, API-1068 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the streams group coordinator (STRM-F01) replaced by a fake, and check the service call and the encoded response; cases cover joins, task assignment, topology updates, status codes per version and internal-topic creation.

### [To Do] API-1070 StreamsGroupHeartbeat compatibility test with Kafka Streams 4.4.0 with group.protocol=streams and kafka-streams-groups.sh
P0 · M8 · Task · deps: API-1069, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that runs a stateful application that scales out and back with standby tasks passes with Kafka Streams 4.4.0 with group.protocol=streams and kafka-streams-groups.sh.

### [To Do] API-1071 StreamsGroupHeartbeat differential test against Kafka 4.4.0
P0 · M8 · Task · deps: API-1069, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical StreamsGroupHeartbeat requests to Kafka 4.4.0 and Alluvium for heartbeats at v0-v1 and for each client-triggerable error (STREAMS_INVALID_TOPOLOGY, STREAMS_TOPOLOGY_FENCED, FENCED_MEMBER_EPOCH and status codes); normalized responses match for every supported version.

### [To Do] API-1072 Advertise StreamsGroupHeartbeat v0-1 in ApiVersions
P0 · M8 · Task · deps: API-1067, API-1070, API-1071, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists StreamsGroupHeartbeat with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks StreamsGroupHeartbeat supported; a test asserts the range.

## [To Do] API-F89 StreamsGroupDescribe (v0-1)
P1 · M8 · Feature · deps: STRM-F04
Scope: Kafka 4.4.0 versions 0-1; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-1073 StreamsGroupDescribe handler module and router registration (broker listener)
P1 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleStreamsGroupDescribe @4.4.0
Done when: `handlers/streams_group_describe.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the streams group administration (STRM-F04); it is registered for the broker listener in the router (SRV-F04). A StreamsGroupDescribe request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. When streams groups are disabled (streams.version) the handler returns UNSUPPORTED_VERSION. Unit test per listener.

### [To Do] API-1074 StreamsGroupDescribe v0: topology, members and task assignments
P1 · M8 · Task · deps: API-1073 · ref: clients/src/main/resources/common/message/StreamsGroupDescribeResponse.json @4.4.0 (KIP-1071)
Done when: Each group returns state, epochs, topology subtopologies and members with tasks, offsets, endpoints and client tags; unknown or non-streams groups get GROUP_ID_NOT_FOUND; tests.

### [To Do] API-1075 StreamsGroupDescribe v1: topology description, status and assignor name (KIP-1331, KIP-1357)
P1 · M8 · Task · deps: API-1074 · ref: clients/src/main/resources/common/message/StreamsGroupDescribeResponse.json @4.4.0 (KIP-1331, KIP-1357)
Done when: With include_topology_description (v1), groups return topology_description and topology_description_status, and assignor_name is filled; v0 omits all three; tests per version.

### [To Do] API-1076 StreamsGroupDescribe error mapping
P1 · M8 · Task · deps: API-1073 · ref: clients/src/main/resources/common/message/StreamsGroupDescribeResponse.json @4.4.0
Done when: The supported-error list of `StreamsGroupDescribeResponse.json` is enforced per group with messages; table test.

### [To Do] API-1077 StreamsGroupDescribe throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M8 · Task · deps: API-1073, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; StreamsGroupDescribe records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-1078 StreamsGroupDescribe request metrics, events and request-log summary
P1 · M8 · Task · deps: API-1073 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for StreamsGroupDescribe; the request-log summary (SRV-F09) shows group count, include_authorized_operations and include_topology_description. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1079 StreamsGroupDescribe authorization: DESCRIBE on groups and topology topics
P1 · M8 · Task · deps: API-1073, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleStreamsGroupDescribe @4.4.0
Done when: Groups without DESCRIBE get GROUP_AUTHORIZATION_FAILED; with an authorizer, groups whose topology uses topics the caller cannot DESCRIBE get TOPIC_AUTHORIZATION_FAILED 'The described group uses topics that the client is not authorized to describe.'; tests and differential cases.

### [To Do] API-1080 StreamsGroupDescribe handler unit tests across v0-1
P1 · M8 · Task · deps: API-1075, API-1076, API-1077, API-1079 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the streams group administration (STRM-F04) replaced by a fake, and check the service call and the encoded response; cases cover stable and reconciling groups, topology descriptions and topic filtering.

### [To Do] API-1081 StreamsGroupDescribe compatibility test with Java client 4.4.0 Admin.describeStreamsGroups and kafka-streams-groups.sh --describe
P1 · M8 · Task · deps: API-1080, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that describes a running streams application passes with Java client 4.4.0 Admin.describeStreamsGroups and kafka-streams-groups.sh --describe.

### [To Do] API-1082 StreamsGroupDescribe differential test against Kafka 4.4.0
P1 · M8 · Task · deps: API-1080, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical StreamsGroupDescribe requests to Kafka 4.4.0 and Alluvium for descriptions at v0-v1 and for each client-triggerable error (GROUP_ID_NOT_FOUND and INVALID_GROUP_ID); normalized responses match for every supported version.

### [To Do] API-1083 Advertise StreamsGroupDescribe v0-1 in ApiVersions
P1 · M8 · Task · deps: API-1078, API-1081, API-1082, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists StreamsGroupDescribe with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks StreamsGroupDescribe supported; a test asserts the range.

## [To Do] API-F90 DescribeShareGroupOffsets (v0-1)
P1 · M8 · Feature · deps: SHARE-F06
Scope: Kafka 4.4.0 versions 0-1; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-1084 DescribeShareGroupOffsets handler module and router registration (broker listener)
P1 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeShareGroupOffsetsRequest @4.4.0
Done when: `handlers/describe_share_group_offsets.rs` (module doc names the Kafka counterpart) decodes v0-1 with the generated codec and calls the share group administration (SHARE-F06); it is registered for the broker listener in the router (SRV-F04). A DescribeShareGroupOffsets request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. When share groups are disabled (share.version) the handler returns UNSUPPORTED_VERSION. Unit test per listener.

### [To Do] API-1085 DescribeShareGroupOffsets v0: start offsets per group and partition
P1 · M8 · Task · deps: API-1084 · ref: clients/src/main/resources/common/message/DescribeShareGroupOffsetsRequest.json @4.4.0 (KIP-932)
Done when: Each group returns per-partition start offset and leader epoch from SHARE-F05 state; a null topic list describes every partition with state; unknown groups get GROUP_ID_NOT_FOUND; tests.

### [To Do] API-1086 DescribeShareGroupOffsets v1: lag (KIP-1226)
P1 · M8 · Task · deps: API-1085 · ref: clients/src/main/resources/common/message/DescribeShareGroupOffsetsResponse.json @4.4.0 (KIP-1226)
Done when: v1 partitions report lag computed from the start offset, delivery complete count and the partition end offset (-1 when unknown); v0 omits it; tests per version.

### [To Do] API-1087 DescribeShareGroupOffsets error mapping
P1 · M8 · Task · deps: API-1084 · ref: clients/src/main/resources/common/message/DescribeShareGroupOffsetsResponse.json @4.4.0
Done when: The supported-error list of `DescribeShareGroupOffsetsResponse.json` is enforced at group and partition level; table test.

### [To Do] API-1088 DescribeShareGroupOffsets throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M8 · Task · deps: API-1084, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; DescribeShareGroupOffsets records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-1089 DescribeShareGroupOffsets request metrics, events and request-log summary
P1 · M8 · Task · deps: API-1084 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DescribeShareGroupOffsets; the request-log summary (SRV-F09) shows group count and topic count or all-topics flag. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1090 DescribeShareGroupOffsets authorization: DESCRIBE on GROUP and topics
P1 · M8 · Task · deps: API-1085, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeShareGroupOffsetsRequest @4.4.0
Done when: Groups without DESCRIBE get GROUP_AUTHORIZATION_FAILED; named topics without DESCRIBE get TOPIC_AUTHORIZATION_FAILED with its message and all-topic requests omit them; tests and differential cases.

### [To Do] API-1091 DescribeShareGroupOffsets handler unit tests across v0-1
P1 · M8 · Task · deps: API-1086, API-1087, API-1088, API-1090 · labels: test
Done when: Table-driven tests send every version v0-v1 through the handler, with the share group administration (SHARE-F06) replaced by a fake, and check the service call and the encoded response; cases cover several groups, all-topic and named requests, unknown groups and lag values.

### [To Do] API-1092 DescribeShareGroupOffsets compatibility test with Java client 4.4.0 Admin.listShareGroupOffsets and kafka-share-groups.sh --describe --offsets
P1 · M8 · Task · deps: API-1091, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that shows start offsets and lag for a share group passes with Java client 4.4.0 Admin.listShareGroupOffsets and kafka-share-groups.sh --describe --offsets.

### [To Do] API-1093 DescribeShareGroupOffsets differential test against Kafka 4.4.0
P1 · M8 · Task · deps: API-1091, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DescribeShareGroupOffsets requests to Kafka 4.4.0 and Alluvium for descriptions at v0-v1 and for each client-triggerable error (GROUP_ID_NOT_FOUND); normalized responses match for every supported version.

### [To Do] API-1094 Advertise DescribeShareGroupOffsets v0-1 in ApiVersions
P1 · M8 · Task · deps: API-1089, API-1092, API-1093, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists DescribeShareGroupOffsets with min 0 and max 1 through the generated list owned by API-F18; docs/parity.md marks DescribeShareGroupOffsets supported; a test asserts the range.

## [To Do] API-F91 AlterShareGroupOffsets (v0)
P1 · M8 · Feature · deps: SHARE-F06
Scope: Kafka 4.4.0 versions 0; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-1095 AlterShareGroupOffsets handler module and router registration (broker listener)
P1 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAlterShareGroupOffsetsRequest @4.4.0
Done when: `handlers/alter_share_group_offsets.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the share group administration (SHARE-F06); it is registered for the broker listener in the router (SRV-F04). A AlterShareGroupOffsets request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. When share groups are disabled (share.version) the handler returns UNSUPPORTED_VERSION. Unit test per listener.

### [To Do] API-1096 AlterShareGroupOffsets v0: change persisted share-group offsets
P1 · M8 · Task · deps: API-1095 · ref: clients/src/main/resources/common/message/AlterShareGroupOffsetsRequest.json @4.4.0 (KIP-932)
Done when: The coordinator sets each listed partition's start offset in the share state; a group with active members gets NON_EMPTY_GROUP; unknown groups get GROUP_ID_NOT_FOUND and unknown topics UNKNOWN_TOPIC_OR_PARTITION; results carry topic ids and messages; tests.

### [To Do] API-1097 AlterShareGroupOffsets error mapping
P1 · M8 · Task · deps: API-1095 · ref: clients/src/main/resources/common/message/AlterShareGroupOffsetsResponse.json @4.4.0
Done when: The supported-error list of `AlterShareGroupOffsetsResponse.json` is enforced at top and topic or partition level; table test.

### [To Do] API-1098 AlterShareGroupOffsets throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M8 · Task · deps: API-1095, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; AlterShareGroupOffsets records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-1099 AlterShareGroupOffsets request metrics, events and request-log summary
P1 · M8 · Task · deps: API-1095 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for AlterShareGroupOffsets; the request-log summary (SRV-F09) shows group id and topic or partition count. Offset changes emit an event with reason `alter_share_group_offsets_request` (SHARE-F06). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1100 AlterShareGroupOffsets authorization: READ on GROUP and READ on topics
P1 · M8 · Task · deps: API-1095, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleAlterShareGroupOffsetsRequest @4.4.0
Done when: Without READ on GROUP the request fails with GROUP_AUTHORIZATION_FAILED; topics without READ get TOPIC_AUTHORIZATION_FAILED while others proceed; tests and differential cases.

### [To Do] API-1101 AlterShareGroupOffsets handler unit tests at v0
P1 · M8 · Task · deps: API-1096, API-1097, API-1098, API-1100 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the share group administration (SHARE-F06) replaced by a fake, and check the service call and the encoded response; cases cover empty and active groups, unknown groups and topics and authorization filtering.

### [To Do] API-1102 AlterShareGroupOffsets compatibility test with Java client 4.4.0 Admin and kafka-share-groups.sh --reset-offsets
P1 · M8 · Task · deps: API-1101, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that changes offsets for an inactive share group and the next consumer starts accordingly passes with Java client 4.4.0 Admin and kafka-share-groups.sh --reset-offsets.

### [To Do] API-1103 AlterShareGroupOffsets differential test against Kafka 4.4.0
P1 · M8 · Task · deps: API-1101, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical AlterShareGroupOffsets requests to Kafka 4.4.0 and Alluvium for changes for an inactive group and for each client-triggerable error (NON_EMPTY_GROUP, GROUP_ID_NOT_FOUND and UNKNOWN_TOPIC_OR_PARTITION); normalized responses match for every supported version.

### [To Do] API-1104 Advertise AlterShareGroupOffsets v0 in ApiVersions
P1 · M8 · Task · deps: API-1099, API-1102, API-1103, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists AlterShareGroupOffsets with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks AlterShareGroupOffsets supported; a test asserts the range.

## [To Do] API-F92 DeleteShareGroupOffsets (v0)
P1 · M8 · Feature · deps: SHARE-F06
Scope: Kafka 4.4.0 versions 0; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-1105 DeleteShareGroupOffsets handler module and router registration (broker listener)
P1 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteShareGroupOffsetsRequest @4.4.0
Done when: `handlers/delete_share_group_offsets.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the share group administration (SHARE-F06); it is registered for the broker listener in the router (SRV-F04). A DeleteShareGroupOffsets request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. When share groups are disabled (share.version) the handler returns UNSUPPORTED_VERSION. Unit test per listener.

### [To Do] API-1106 DeleteShareGroupOffsets v0: change persisted share-group offsets
P1 · M8 · Task · deps: API-1105 · ref: clients/src/main/resources/common/message/DeleteShareGroupOffsetsRequest.json @4.4.0 (KIP-932)
Done when: The coordinator deletes the share state of the listed topics; a group with active members gets NON_EMPTY_GROUP; unknown groups get GROUP_ID_NOT_FOUND and unknown topics UNKNOWN_TOPIC_OR_PARTITION; results carry topic ids and messages; tests.

### [To Do] API-1107 DeleteShareGroupOffsets error mapping
P1 · M8 · Task · deps: API-1105 · ref: clients/src/main/resources/common/message/DeleteShareGroupOffsetsResponse.json @4.4.0
Done when: The supported-error list of `DeleteShareGroupOffsetsResponse.json` is enforced at top and topic or partition level; table test.

### [To Do] API-1108 DeleteShareGroupOffsets throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M8 · Task · deps: API-1105, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; DeleteShareGroupOffsets records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-1109 DeleteShareGroupOffsets request metrics, events and request-log summary
P1 · M8 · Task · deps: API-1105 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for DeleteShareGroupOffsets; the request-log summary (SRV-F09) shows group id and topic or partition count. Offset changes emit an event with reason `delete_share_group_offsets_request` (SHARE-F06). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1110 DeleteShareGroupOffsets authorization: DELETE on GROUP and READ on topics
P1 · M8 · Task · deps: API-1105, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteShareGroupOffsetsRequest @4.4.0
Done when: Without DELETE on GROUP the request fails with GROUP_AUTHORIZATION_FAILED; topics without READ get TOPIC_AUTHORIZATION_FAILED while others proceed; tests and differential cases.

### [To Do] API-1111 DeleteShareGroupOffsets handler unit tests at v0
P1 · M8 · Task · deps: API-1106, API-1107, API-1108, API-1110 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the share group administration (SHARE-F06) replaced by a fake, and check the service call and the encoded response; cases cover empty and active groups, unknown groups and topics and authorization filtering.

### [To Do] API-1112 DeleteShareGroupOffsets compatibility test with Java client 4.4.0 Admin and kafka-share-groups.sh --delete-offsets
P1 · M8 · Task · deps: API-1111, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that changes offsets for an inactive share group and the next consumer starts accordingly passes with Java client 4.4.0 Admin and kafka-share-groups.sh --delete-offsets.

### [To Do] API-1113 DeleteShareGroupOffsets differential test against Kafka 4.4.0
P1 · M8 · Task · deps: API-1111, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical DeleteShareGroupOffsets requests to Kafka 4.4.0 and Alluvium for changes for an inactive group and for each client-triggerable error (NON_EMPTY_GROUP, GROUP_ID_NOT_FOUND and UNKNOWN_TOPIC_OR_PARTITION); normalized responses match for every supported version.

### [To Do] API-1114 Advertise DeleteShareGroupOffsets v0 in ApiVersions
P1 · M8 · Task · deps: API-1109, API-1112, API-1113, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists DeleteShareGroupOffsets with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks DeleteShareGroupOffsets supported; a test asserts the range.

## [To Do] API-F93 StreamsGroupTopologyDescriptionUpdate (v0)
P1 · M8 · Feature · deps: STRM-F05
Scope: Kafka 4.4.0 versions 0; listeners: broker. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-1115 StreamsGroupTopologyDescriptionUpdate handler module and router registration (broker listener)
P1 · M8 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleStreamsGroupTopologyDescriptionUpdate @4.4.0
Done when: `handlers/streams_group_topology_description_update.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the streams group coordinator (STRM-F05); it is registered for the broker listener in the router (SRV-F04). A StreamsGroupTopologyDescriptionUpdate request on a controller listener closes the connection, as `Processor.parseRequestHeader` does for APIs disabled on a listener. When streams groups are disabled (streams.version) the handler returns UNSUPPORTED_VERSION. Unit test per listener.

### [To Do] API-1116 StreamsGroupTopologyDescriptionUpdate v0: store a member's topology description
P1 · M8 · Task · deps: API-1115 · ref: clients/src/main/resources/common/message/StreamsGroupTopologyDescriptionUpdateRequest.json @4.4.0 (KIP-1331)
Done when: The member's topology description for topology_epoch is passed to STRM-F05; unknown members get UNKNOWN_MEMBER_ID, unknown groups GROUP_ID_NOT_FOUND and a failed update STREAMS_TOPOLOGY_DESCRIPTION_UPDATE_FAILED with a message; tests.

### [To Do] API-1117 StreamsGroupTopologyDescriptionUpdate error mapping
P1 · M8 · Task · deps: API-1115 · ref: clients/src/main/resources/common/message/StreamsGroupTopologyDescriptionUpdateResponse.json @4.4.0
Done when: The supported-error list of `StreamsGroupTopologyDescriptionUpdateResponse.json` maps exhaustively with error_message; table test.

### [To Do] API-1118 StreamsGroupTopologyDescriptionUpdate throttle_time_ms and throttling under the request quota (KIP-219)
P1 · M8 · Task · deps: API-1115, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The throttle_time_ms field is present in every supported version; StreamsGroupTopologyDescriptionUpdate records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Test with a quota-limited principal.

### [To Do] API-1119 StreamsGroupTopologyDescriptionUpdate request metrics, events and request-log summary
P1 · M8 · Task · deps: API-1115 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for StreamsGroupTopologyDescriptionUpdate; the request-log summary (SRV-F09) shows group id, member id, topology epoch and description size. Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1120 StreamsGroupTopologyDescriptionUpdate authorization: READ on GROUP
P1 · M8 · Task · deps: API-1115, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleStreamsGroupTopologyDescriptionUpdate @4.4.0
Done when: Requests without READ on GROUP get GROUP_AUTHORIZATION_FAILED; tests and a differential case.

### [To Do] API-1121 StreamsGroupTopologyDescriptionUpdate handler unit tests at v0
P1 · M8 · Task · deps: API-1116, API-1117, API-1118, API-1120 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the streams group coordinator (STRM-F05) replaced by a fake, and check the service call and the encoded response; cases cover accepted updates, unknown members and groups and stale topology epochs.

### [To Do] API-1122 StreamsGroupTopologyDescriptionUpdate compatibility test with Kafka Streams 4.4.0
P1 · M8 · Task · deps: API-1121, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that sends its topology description when topology_description_required is set and kafka-streams-groups.sh shows it passes with Kafka Streams 4.4.0.

### [To Do] API-1123 StreamsGroupTopologyDescriptionUpdate differential test against Kafka 4.4.0
P1 · M8 · Task · deps: API-1121, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical StreamsGroupTopologyDescriptionUpdate requests to Kafka 4.4.0 and Alluvium for updates from a Kafka Streams member and for each client-triggerable error (UNKNOWN_MEMBER_ID and STREAMS_TOPOLOGY_DESCRIPTION_UPDATE_FAILED); normalized responses match for every supported version.

### [To Do] API-1124 Advertise StreamsGroupTopologyDescriptionUpdate v0 in ApiVersions
P1 · M8 · Task · deps: API-1119, API-1122, API-1123, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker listener lists StreamsGroupTopologyDescriptionUpdate with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks StreamsGroupTopologyDescriptionUpdate supported; a test asserts the range.

## [To Do] API-F94 UnregisterController (v0)
P1 · M3 · Feature · deps: CTRL-F22
Scope: Kafka 4.4.0 versions 0; listeners: broker, controller. Domain logic lives in the dependency features; this feature covers the handler, versions, authorization, errors, metrics, tests and advertising.

### [To Do] API-1125 UnregisterController handler module and router registration (broker and controller listeners)
P1 · M3 · Task · deps: none · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleUnregisterController @4.4.0
Done when: `handlers/unregister_controller.rs` (module doc names the Kafka counterpart) decodes v0 with the generated codec and calls the controller unregistration path (CTRL-F22); it is registered for the broker and controller listeners in the router (SRV-F04). Unit test per listener.

### [To Do] API-1126 Forward UnregisterController from brokers to the active controller in an Envelope
P1 · M3 · Task · deps: API-1125, API-F58, CTRL-F21 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManager.java forwardRequest @4.4.0
Done when: On the broker listener UnregisterController is forwarded to the active controller in an Envelope carrying the serialized request, principal and client address (CTRL-F21, API-F58); the controller response is relayed and an unsupported forwarded version maps to UNSUPPORTED_VERSION as in `handleInvalidVersionsDuringForwarding`. Integration test runs a broker-only node against a controller-only node.

### [To Do] API-1127 UnregisterController v0: remove a controller registration
P1 · M3 · Task · deps: API-1125 · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java unregisterController @4.4.0
Done when: The controller removes the registration through CTRL-F22; an unknown id gets CONTROLLER_ID_NOT_REGISTERED and a metadata.version without controller unregistration UNSUPPORTED_VERSION, each with error_message; tests.

### [To Do] API-1128 UnregisterController error mapping
P1 · M3 · Task · deps: API-1125 · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleUnregisterController @4.4.0
Done when: CONTROLLER_ID_NOT_REGISTERED, NOT_CONTROLLER, UNSUPPORTED_VERSION and CLUSTER_AUTHORIZATION_FAILED map exhaustively with messages; table test.

### [To Do] API-1129 UnregisterController throttle_time_ms through the shared response path
P1 · M3 · Task · deps: API-1125 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: The response is sent through the SRV maybe-throttle path; the throttle_time_ms field is present in every supported version; the value is the request's throttle decision (0 until quotas land in M4); unit test at each version boundary.

### [To Do] API-1130 UnregisterController throttling under the request quota (KIP-219)
P1 · M4 · Task · deps: API-1129, QUOTA-F04, QUOTA-F07 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala sendResponseMaybeThrottle @4.4.0
Done when: UnregisterController records usage against the request quota and, on violation, sends the response immediately with throttle_time_ms and then mutes the channel. Forwarded requests are throttled on the broker with the larger of the local and the controller's throttle (`sendForwardedResponse`); the controller never mutes a forwarded request. Test with a quota-limited principal shows throttle_time_ms > 0 and the muted channel.

### [To Do] API-1131 UnregisterController request metrics, events and request-log summary
P1 · M3 · Task · deps: API-1125 · labels: observability · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: Per-phase request metrics (SRV-F08) and per-error-code counts are recorded for UnregisterController; the request-log summary (SRV-F09) shows the target controller id and outcome. Unregistration carries reason `unregister_controller_request` (CTRL-F22). Tests assert the metric names and that no secret or record payload reaches the logs.

### [To Do] API-1132 UnregisterController handler unit tests at v0
P1 · M3 · Task · deps: API-1126, API-1127, API-1128, API-1129 · labels: test
Done when: Table-driven tests send v0 requests through the handler, with the controller unregistration path (CTRL-F22) replaced by a fake, and check the service call and the encoded response; cases cover registered, unknown and old-metadata.version cases.

### [To Do] API-1133 UnregisterController compatibility test with Java client 4.4.0 Admin.unregisterController, kafka-cluster.sh unregister-controller and kafka-metadata-quorum.sh remove-controller
P1 · M3 · Task · deps: API-1132, TEST-F08 · labels: test
Done when: A TEST-F08 scenario that unregisters a removed controller so it disappears from DescribeCluster with endpoint type controller passes with Java client 4.4.0 Admin.unregisterController, kafka-cluster.sh unregister-controller and kafka-metadata-quorum.sh remove-controller.

### [To Do] API-1134 UnregisterController differential test against Kafka 4.4.0
P1 · M3 · Task · deps: API-1132, TEST-F09 · labels: test
Done when: A TEST-F09 scenario sends identical UnregisterController requests to Kafka 4.4.0 and Alluvium for unregistering a removed controller and for each client-triggerable error (CONTROLLER_ID_NOT_REGISTERED); normalized responses match for every supported version.

### [To Do] API-1135 Advertise UnregisterController v0 in ApiVersions
P1 · M3 · Task · deps: API-1131, API-1133, API-1134, API-F18 · ref: server/src/main/java/org/apache/kafka/server/ApiVersionManager.java @4.4.0
Done when: ApiVersions on the broker and controller listeners lists UnregisterController with min 0 and max 0 through the generated list owned by API-F18; docs/parity.md marks UnregisterController supported; a test asserts the range. Later tasks (M4) add behavior without changing the advertised range.

### [To Do] API-1136 UnregisterController authorization: ALTER on CLUSTER
P1 · M4 · Task · deps: API-1126, SEC-F08 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleUnregisterController @4.4.0
Done when: Requests need ALTER on CLUSTER or get CLUSTER_AUTHORIZATION_FAILED; tests and a differential case.
