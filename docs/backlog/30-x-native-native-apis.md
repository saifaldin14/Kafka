# X-NATIVE: Native HTTP and gRPC APIs and SDK

> Crates: `alluvium-api` (new, layer 4), `alluvium-client` (SDK) · Kafka counterpart: none (Alluvium enhancement) · Milestones: M10

First-class native APIs alongside the Kafka protocol: gRPC and HTTP/JSON over the shared service layer, streaming consumption, a Rust SDK and generated SDKs, and a native admin CLI.

## [To Do] X-NATIVE-F01 Native API design
P0 · M10 · Feature · deps: SRV-F10
Scope: Spike and ADR: resource model, transports (gRPC and HTTP/JSON), versioning, IDL as source of truth.

### [To Do] X-NATIVE-0001 Native API design spike: resource model, gRPC and HTTP/JSON over the shared service layer, versioning, IDL as source of truth
P0 · M10 · Spike · deps: none · labels: docs · ref: ADR-0010
Done when: an ADR "Native APIs" is accepted in docs/adr/ covering: the resource model (topics, partitions, records, cursors, groups, configs, ACLs, quotas, API keys); gRPC plus HTTP/JSON transcoding from one Protobuf IDL; package versioning (`alluvium.v1`); placement of `alluvium-api`, `alluvium-client` and the generated proto crate in the layer table of docs/architecture.md and `xtask check-layers`; leader routing (leader hint, no proxying); idempotence through producer ids; alternatives (REST-only, Connect RPC, GraphQL) with reasons for rejection.

### [To Do] X-NATIVE-0002 Dependency selection for gRPC, HTTP/1.1, HTTP/2, Protobuf and WebSocket (tonic, prost, hyper, pbjson, tungstenite or alternatives)
P0 · M10 · Spike · deps: X-NATIVE-0001 · labels: security · ref: docs/standards/security.md#dependencies
Done when: the five dependency-review questions are answered per crate in an appendix of the native API ADR, with `cargo tree` transitive counts and `unsafe` usage; `deny.toml` admits the chosen crates; the pinned `buf` CLI version used by `xtask` is recorded.

### [To Do] X-NATIVE-0003 Threat model rows for native listeners: request smuggling, HTTP/2 rapid reset and floods, JSON depth bombs, slowloris, CORS/CSRF, credentials in URLs, WebSocket hijacking
P0 · M10 · Task · deps: X-NATIVE-0001 · labels: security, docs · ref: docs/standards/security.md
Done when: docs/standards/security.md has one threat row per item with its defense and the backlog task id implementing it; the security review checklist (SEC-F14) gains a native-API section.

## [To Do] X-NATIVE-F02 IDL and schema
P0 · M10 · Feature · deps: X-NATIVE-F01
Scope: Protobuf definitions for all native services; generated OpenAPI; IDL lint and breaking-change checks.

### [To Do] X-NATIVE-0004 Proto source tree `proto/alluvium/v1/`, `buf.yaml` workspace and `cargo xtask proto lint` (buf STANDARD rules plus required doc comments)
P0 · M10 · Task · deps: none · labels: docs · ref: ADR Native APIs (X-NATIVE-0001)
Done when: `cargo xtask proto lint` runs inside `cargo xtask ci`; fixtures prove it fails on a missing comment, a non-`v1` package and an enum without an `_UNSPECIFIED = 0` value.

### [To Do] X-NATIVE-0005 Common messages in `types.proto`: TopicRef (name or id), PartitionRef, Record (key, value, headers, timestamp, timestamp_type), Header, Cursor, PageToken, Acks and IsolationLevel enums
P0 · M10 · Task · deps: X-NATIVE-0004 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: enum values mirror the TYPE-F12 wire values with `_UNSPECIFIED = 0`; every field documents its limit; lint passes; the messages are reused by all service files (no duplicate record types).

### [To Do] X-NATIVE-0006 Field validation options `(alluvium.v1.field)` (required, max_len, max_items, range) and a generated `validate()` per request message
P0 · M10 · Task · deps: X-NATIVE-0005 · labels: security · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the generator emits `validate()` returning field-path violations; unit tests reject a missing topic, a 300-character topic name, 10 001 records and an oversized header count before any service call; violations surface as BadRequest details (X-NATIVE-F10).

### [To Do] X-NATIVE-0007 Rust code generation for the IDL in `cargo xtask codegen` with committed output in the generated proto crate and a drift check
P0 · M10 · Task · deps: X-NATIVE-0005 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: generated messages are committed; `cargo xtask codegen --check` fails when a `.proto` file changes without regeneration; the generated crate has no Tokio or networking dependency (layer check passes); service stubs are generated only for the server and SDK crates.

### [To Do] X-NATIVE-0008 `ProduceService` in `produce.proto`: Produce, ProduceBatch, ProduceStream and InitProducer messages
P0 · M10 · Task · deps: X-NATIVE-0005, X-NATIVE-0006 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: request/response messages carry topic, partition, records, acks, timeout, producer id/epoch/base sequence and per-record results; streaming frames carry client request ids; lint and validation options pass review against the X-NATIVE-F05 task designs.

### [To Do] X-NATIVE-0009 `ConsumeService` in `consume.proto`: Fetch, FetchStream and ResolveCursor messages, including stream credit frames
P0 · M10 · Task · deps: X-NATIVE-0005, X-NATIVE-0006 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: messages carry cursor, max_records, max_bytes, max_wait_ms, isolation level, watermarks and next cursor; credit and heartbeat frames are defined for FetchStream; lint passes.

### [To Do] X-NATIVE-0010 `GroupService` in `group.proto`: Subscribe stream frames, Commit, FetchCommitted and Leave
P1 · M10 · Task · deps: X-NATIVE-0005, X-NATIVE-0006 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: client frames (subscription, revocation ack, credit) and server frames (assignment, records, heartbeat request) are defined with member id, member epoch and instance id; lint passes.

### [To Do] X-NATIVE-0011 Topic admin RPCs in `admin.proto`: CreateTopic, DeleteTopic, ListTopics, DescribeTopic, CreatePartitions, DeleteRecords
P1 · M10 · Task · deps: X-NATIVE-0005, X-NATIVE-0006 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: list RPCs use `page_size`/`page_token`; mutating RPCs carry `validate_only`; DescribeTopic returns partitions, leaders, replicas, ISR, ELR and authorized operations; lint passes.

### [To Do] X-NATIVE-0012 Config, ACL, group, quota and cluster admin RPCs in `admin.proto`
P1 · M10 · Task · deps: X-NATIVE-0005, X-NATIVE-0006 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: messages exist for Describe/UpdateTopicConfig, Describe/UpdateBrokerConfig, List/Create/DeleteAcls, ListGroups, DescribeGroup, DeleteGroup, AlterGroupOffsets, DeleteGroupOffsets, Describe/AlterQuotas and DescribeCluster, each documenting its Kafka API equivalent; lint passes.

### [To Do] X-NATIVE-0013 `ApiKeyService` (CreateApiKey, ListApiKeys, RevokeApiKey) and `ServerInfoService` (GetServerInfo, WhoAmI) in `auth.proto` and `server_info.proto`
P0 · M10 · Task · deps: X-NATIVE-0005, X-NATIVE-0006 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the API key secret appears only in the CreateApiKey response and is annotated as sensitive for redaction; scopes are typed (resource type, pattern type, name, operations); lint passes.

### [To Do] X-NATIVE-0014 Error detail messages: vendored `google/rpc/status.proto` and `error_details.proto` with attribution, plus `alluvium.v1.ErrorDetail` (error name, retriable, leader endpoint hint, throttle time)
P0 · M10 · Task · deps: X-NATIVE-0004 · ref: https://github.com/googleapis/googleapis google/rpc/error_details.proto
Done when: vendored files are unmodified with license and checksum manifest; `ErrorDetail` is documented; codegen includes them.

### [To Do] X-NATIVE-0015 `google.api.http` annotations for every unary and server-streaming RPC, with vendored annotation protos and a route-collision lint
P0 · M10 · Task · deps: X-NATIVE-0008, X-NATIVE-0009, X-NATIVE-0010, X-NATIVE-0011, X-NATIVE-0012, X-NATIVE-0013 · ref: https://github.com/googleapis/googleapis google/api/http.proto
Done when: every RPC except bidirectional streams has exactly one HTTP route; the lint fails on two RPCs mapping to the same verb and path template; routes follow `/v1/<collection>/{name}` and `:<verb>` custom methods.

### [To Do] X-NATIVE-0016 Generated OpenAPI 3.1 document `openapi/alluvium-v1.yaml` from the IDL, committed with a drift check
P1 · M10 · Task · deps: X-NATIVE-0015 · labels: docs · ref: https://spec.openapis.org/oas/v3.1.0
Done when: `cargo xtask codegen --check` fails on drift; the document validates against the OpenAPI 3.1 schema in CI; field docs and limits from the protos appear in it.

### [To Do] X-NATIVE-0017 Breaking-change check in CI: `buf breaking` against the last release tag
P0 · M10 · Task · deps: X-NATIVE-0007 · ref: https://buf.build/docs/breaking/overview
Done when: CI fails when a field number is reused, a field type changes or an RPC is removed in a stable package; the check runs from the first release tag on and is skipped with a logged reason before any tag exists.

## [To Do] X-NATIVE-F03 gRPC server
P0 · M10 · Feature · deps: X-NATIVE-F02, NET-F07
Scope: gRPC listener, TLS, HTTP/2 limits, reflection, health service.

### [To Do] X-NATIVE-0018 Config keys `alluvium.native.grpc.listeners` (default empty) and per-listener TLS through `listener.name.<name>.ssl.*`, with the loopback default and plaintext-external guard
P0 · M10 · Task · deps: CFG-F11 · labels: security · ref: ADR-0006
Done when: keys are registered in the alluvium key registry with docs; an empty value starts no gRPC listener; a non-loopback plaintext listener is refused at startup unless `alluvium.allow.plaintext.external=true`; `check-config` shows each value's source.

### [To Do] X-NATIVE-0019 gRPC listener under the task supervisor: bind, accept, HTTP/2 with ALPN `h2`, named tasks and a defined place in startup and shutdown order
P0 · M10 · Task · deps: X-NATIVE-0018, SRV-F03 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the listener starts after Kafka listeners are ready and stops accepting before Kafka listeners during shutdown; every task is named and visible in `/debug/tasks`; a bind failure aborts startup with a clear error naming the listener.

### [To Do] X-NATIVE-0020 TLS for gRPC listeners: rustls server config shared with NET-F07, client-auth modes (none, requested, required), certificate hot reload
P0 · M10 · Task · deps: X-NATIVE-0019 · labels: security · ref: NET-F07
Done when: TLS 1.2 and 1.3 handshakes succeed with grpcurl; required client auth rejects a missing certificate; a reloaded certificate is served to new connections without restart; tests cover an expired client certificate and ALPN mismatch.

### [To Do] X-NATIVE-0021 HTTP/2 settings: `alluvium.native.grpc.max.concurrent.streams`, initial stream and connection windows, max frame size and `alluvium.native.grpc.max.header.list.bytes`
P0 · M10 · Task · deps: X-NATIVE-0019 · labels: security · ref: https://www.rfc-editor.org/rfc/rfc9113
Done when: advertised SETTINGS match config; a client exceeding the stream limit receives REFUSED_STREAM; an oversized header list is rejected before decoding; defaults are documented with their memory cost per connection.

### [To Do] X-NATIVE-0022 HTTP/2 abuse protections: rapid reset (CVE-2023-44487), SETTINGS and PING floods, CONTINUATION floods, empty-frame floods
P0 · M10 · Task · deps: X-NATIVE-0021 · labels: security · ref: https://nvd.nist.gov/vuln/detail/CVE-2023-44487
Done when: a raw h2 test client replays each attack; the server answers with GOAWAY ENHANCE_YOUR_CALM; `alluvium_native_h2_abuse_total{kind}` increments; server memory stays within the per-connection budget in each test.

### [To Do] X-NATIVE-0023 Connection limits and lifetimes: per-listener and per-IP caps, `alluvium.native.connection.idle.timeout.ms`, `alluvium.native.connection.max.age.ms` with GOAWAY grace, keepalive ping policy
P0 · M10 · Task · deps: X-NATIVE-0019, NET-F09 · labels: security · ref: NET-F09
Done when: connections over the cap are closed at accept with a counted reason; idle connections close after the timeout (simulated clock); max-age connections receive GOAWAY and in-flight calls complete; clients pinging faster than allowed get ENHANCE_YOUR_CALM.

### [To Do] X-NATIVE-0024 Message size and compression limits: `alluvium.native.grpc.max.message.bytes` checked from the length prefix before allocation; `grpc-encoding` identity, gzip and zstd with bounded decompression
P0 · M10 · Task · deps: X-NATIVE-0019, REC-F10 · labels: security · ref: https://github.com/grpc/grpc/blob/master/doc/compression.md
Done when: an over-limit length prefix is rejected without allocating (allocation-counting test); a gzip bomb is rejected at the limit; an unsupported encoding returns UNIMPLEMENTED; responses honour `grpc-accept-encoding`.

### [To Do] X-NATIVE-0025 Service dispatch: native RequestContext (principal, listener, peer address, client software from `user-agent`, deadline, request id) and calls into the SRV-F10 service layer
P0 · M10 · Task · deps: X-NATIVE-0019, SRV-F06, SRV-F10 · ref: SRV-F10
Done when: every generated service method routes to a handler module `handlers/native/<service>.rs`; unknown methods return UNIMPLEMENTED; the request id is returned in the `x-alluvium-request-id` response header and appears in the request span.

### [To Do] X-NATIVE-0026 Native authorization adapter: one exhaustive table from each RPC to Kafka (resource type, operation) pairs evaluated through the SEC-F08 helper before any action
P0 · M10 · Task · deps: X-NATIVE-0025, SEC-F08 · labels: security · ref: SEC-F08
Done when: a unit test fails if an RPC lacks a mapping; `authz_decision` events carry `transport=native`; denied calls never reach the service layer (test with a counting service fake at the transport boundary).

### [To Do] X-NATIVE-0027 `grpc.health.v1.Health` Check and Watch mapped to OBS-F09 readiness, unauthenticated only on loopback listeners
P1 · M10 · Task · deps: X-NATIVE-0025, OBS-F09 · ref: https://github.com/grpc/grpc/blob/master/doc/health-checking.md
Done when: NOT_SERVING is reported before metadata catch-up and during shutdown; Watch streams a transition when readiness changes; non-loopback listeners require authentication like other RPCs; the response carries no state beyond readiness.

### [To Do] X-NATIVE-0028 Server reflection (`grpc.reflection.v1`), off by default via `alluvium.native.grpc.reflection.enable=false`, authenticated when on
P2 · M10 · Task · deps: X-NATIVE-0025 · labels: security · ref: https://github.com/grpc/grpc/blob/master/doc/server-reflection.md
Done when: grpcurl `list` fails with UNIMPLEMENTED by default and succeeds for an authenticated principal when enabled; reflection never runs on unauthenticated listeners (startup validation).

### [To Do] X-NATIVE-0029 Graceful drain: GOAWAY on shutdown, unary calls complete within `alluvium.native.shutdown.drain.ms`, streams end with UNAVAILABLE and a retry hint
P0 · M10 · Task · deps: X-NATIVE-0019, NET-F13 · ref: NET-F13
Done when: a shutdown test with 100 in-flight calls completes them or ends them with UNAVAILABLE within the bound; shutdown logs the drain duration and the number of streams cut.

### [To Do] X-NATIVE-0030 Transport conformance: h2spec in CI against the gRPC listener, plus tonic-client and grpcurl smoke tests over TLS and mTLS
P1 · M10 · Task · deps: X-NATIVE-0020, X-NATIVE-0021, X-NATIVE-0024 · labels: test · ref: https://github.com/summerwind/h2spec
Done when: h2spec passes all applicable sections (exceptions listed with reasons); smoke tests run in `cargo xtask ci` with a temporary port and certificate.

## [To Do] X-NATIVE-F04 HTTP/JSON gateway
P0 · M10 · Feature · deps: X-NATIVE-F02, NET-F07
Scope: REST mapping of services, content negotiation, OpenAPI serving.

### [To Do] X-NATIVE-0031 Config keys `alluvium.native.http.listeners` (default empty) with per-listener TLS, HTTP/1.1 and HTTP/2 via ALPN
P0 · M10 · Task · deps: CFG-F11 · labels: security · ref: ADR-0006
Done when: an empty value starts no HTTP listener; plaintext non-loopback is refused unless explicitly allowed; ALPN negotiates h2 or http/1.1; `check-config` shows sources.

### [To Do] X-NATIVE-0032 HTTP listener under the task supervisor with named tasks, request ids and graceful drain
P0 · M10 · Task · deps: X-NATIVE-0031, SRV-F03 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: startup and shutdown order matches the gRPC listener; in-flight requests finish or receive 503 within `alluvium.native.shutdown.drain.ms`; `x-alluvium-request-id` is set on every response.

### [To Do] X-NATIVE-0033 Route table generated from `google.api.http` annotations: path templates, verbs, body and query parameter binding
P0 · M10 · Task · deps: X-NATIVE-0032 · ref: https://github.com/googleapis/googleapis google/api/http.proto
Done when: generated routes are committed with a drift check; percent-encoded path segments decode once; unknown query parameters return 400 with the parameter name; a table test covers every RPC route.

### [To Do] X-NATIVE-0034 Proto3 JSON transcoding: int64 as strings, enums by name, unknown fields ignored and counted, nesting limit `alluvium.native.http.json.max.depth`
P0 · M10 · Task · deps: X-NATIVE-0033 · labels: security · ref: https://protobuf.dev/programming-guides/json/
Done when: round-trip tests cover every message type; a 10 000-level nested body is rejected without stack overflow; unknown fields increment `alluvium_native_json_unknown_fields_total`; duplicate keys are rejected.

### [To Do] X-NATIVE-0035 Record key and value encodings in JSON: base64 by default, `utf8` and `json` encodings selected per request
P0 · M10 · Task · deps: X-NATIVE-0034 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: invalid base64 or non-UTF-8 input returns 400 naming the record index and field; `json` encoding embeds JSON values without double escaping; responses use the encoding requested in `value_encoding`.

### [To Do] X-NATIVE-0036 Content negotiation: `application/json`, `application/x-protobuf`, `application/x-ndjson`; 406 and 415 responses; gzip and zstd response compression
P0 · M10 · Task · deps: X-NATIVE-0033 · ref: https://www.rfc-editor.org/rfc/rfc9110#section-12
Done when: tests cover each media type, quality values in `Accept`, unsupported types (406/415) and compressed request bodies with bounded decompression.

### [To Do] X-NATIVE-0037 Server-streaming RPCs over HTTP as chunked NDJSON with per-message flush and a final status object
P0 · M10 · Task · deps: X-NATIVE-0036 · ref: https://github.com/ndjson/ndjson-spec
Done when: FetchStream over HTTP delivers one JSON object per line; a mid-stream error ends with a status object carrying the problem details; client disconnect cancels the server stream within one poll.

### [To Do] X-NATIVE-0038 HTTP limits: `alluvium.native.http.max.body.bytes`, `alluvium.native.http.max.header.bytes`, request-line length and `alluvium.native.http.read.timeout.ms` against slow bodies
P0 · M10 · Task · deps: X-NATIVE-0032 · labels: security · ref: docs/standards/security.md
Done when: each limit returns 413, 431 or 408 before buffering beyond the limit; a slowloris test keeps the connection count bounded; limits are documented with defaults.

### [To Do] X-NATIVE-0039 HTTP/1.1 parsing hardening: reject conflicting Content-Length and Transfer-Encoding, obsolete line folding and invalid header bytes
P0 · M10 · Task · deps: X-NATIVE-0032 · labels: security · ref: https://www.rfc-editor.org/rfc/rfc9112#section-6.3
Done when: a request-smuggling corpus (CL.TE, TE.CL, TE.TE variants) is rejected with 400 and the connection closed; tests run in CI.

### [To Do] X-NATIVE-0040 CORS off by default; `alluvium.native.http.cors.allowed.origins` allow-list with preflight handling
P1 · M10 · Task · deps: X-NATIVE-0033 · labels: security · ref: https://fetch.spec.whatwg.org/#http-cors-protocol
Done when: no CORS headers are sent by default; an allowed origin gets exact-origin headers; `*` is refused at startup together with credentialed auth methods; preflight caching is bounded.

### [To Do] X-NATIVE-0041 Serve the generated OpenAPI document at `GET /v1/openapi.json` and `/v1/openapi.yaml` with ETag caching
P1 · M10 · Task · deps: X-NATIVE-0033 · labels: docs · ref: https://spec.openapis.org/oas/v3.1.0
Done when: the served document equals the committed file byte for byte; `If-None-Match` returns 304; the route requires the same authentication as other routes unless `alluvium.native.http.openapi.public=true`.

## [To Do] X-NATIVE-F05 Native produce
P0 · M10 · Feature · deps: X-NATIVE-F03
Scope: Unary, batch and streaming produce; idempotency keys; acks semantics.

### [To Do] X-NATIVE-0042 Record conversion: native records to record batch v2 through the REC batch builder, with compression, headers, CreateTime and the topic `message.timestamp.type`
P0 · M10 · Task · deps: X-NATIVE-F10 · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MemoryRecordsBuilder.java @4.4.0
Done when: property test: a native record converted to a batch and parsed back yields the same key, value, headers and timestamp; LogAppendTime topics overwrite timestamps as for Kafka producers; compression honours the request field or topic `compression.type`.

### [To Do] X-NATIVE-0043 Partition selection: explicit partition, murmur2 key hashing identical to the Kafka default partitioner, round-robin for null keys
P0 · M10 · Task · deps: TYPE-F11 · ref: clients/src/main/java/org/apache/kafka/clients/producer/internals/BuiltInPartitioner.java partitionForKey @4.4.0
Done when: golden vectors from the Java partitioner for 10 000 keys and partition counts 1-1000 match; an explicit partition out of range returns NOT_FOUND with the partition count.

### [To Do] X-NATIVE-0044 Acks semantics: ACKS_NONE, ACKS_LEADER and ACKS_ALL mapped to service-layer acks, delayed-produce completion and REQUEST_TIMED_OUT
P0 · M10 · Task · deps: X-NATIVE-0042, REPL-F08 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: ACKS_ALL responds only after the high watermark passes the batch (simulation test); NOT_ENOUGH_REPLICAS maps per X-NATIVE-F10; ACKS_NONE still returns a response without offsets, documented as fire-and-forget.

### [To Do] X-NATIVE-0045 `Produce` RPC (`POST /v1/topics/{topic}/records`): one topic, up to `alluvium.native.produce.max.records.per.request` records, per-record results
P0 · M10 · Task · deps: X-NATIVE-0042, X-NATIVE-0043, X-NATIVE-0044, X-NATIVE-F04 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the response lists partition, offset, timestamp or per-record error for each record in request order; WRITE on the topic is required (PERMISSION_DENIED with TOPIC_AUTHORIZATION_FAILED otherwise); unknown topic gives NOT_FOUND; a record over `max.message.bytes` fails only that record's partition batch with its index; tests cover gRPC and HTTP.

### [To Do] X-NATIVE-0046 `ProduceBatch` RPC (`POST /v1/records:batch`): records for many topics and partitions, non-atomic, per-partition results
P0 · M10 · Task · deps: X-NATIVE-0045 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: partial failure returns OK with per-partition errors (X-NATIVE-F10 rules); authorization is checked per topic; one partition's NOT_LEADER does not fail the others; request byte cap enforced.

### [To Do] X-NATIVE-0047 `InitProducer` RPC: allocate a producer id and epoch through TXN-F02 for idempotent native producers
P0 · M10 · Task · deps: X-NATIVE-F10, TXN-F02 · ref: KIP-679
Done when: authorization follows KIP-679 (WRITE on any topic or IDEMPOTENT_WRITE on the cluster); ids come from the controller-allocated blocks; repeated calls return new ids; tests cover denial and allocation failure.

### [To Do] X-NATIVE-0048 Idempotent produce: `producer_id`, `producer_epoch` and `base_sequence` on Produce and ProduceBatch; duplicates return the original offsets
P0 · M10 · Task · deps: X-NATIVE-0045, X-NATIVE-0047, TXN-F01 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java @4.4.0
Done when: a resent request within the last five batches returns the original offsets without appending (test reads the log); OUT_OF_ORDER_SEQUENCE_NUMBER and INVALID_PRODUCER_EPOCH map per X-NATIVE-F10; sequences wrap as in Kafka.

### [To Do] X-NATIVE-0049 `ProduceStream` RPC: bidirectional stream with client request ids, ordered acks, a credit window of `alluvium.native.produce.stream.max.inflight.bytes` and implicit idempotence per stream
P0 · M10 · Task · deps: X-NATIVE-0048 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: acks arrive in request order; a client exceeding its credit receives RESOURCE_EXHAUSTED; a reconnect resending unacked requests with the same producer id and sequences produces no duplicates (test); HTTP has no route for this RPC (documented).

### [To Do] X-NATIVE-0050 Leader routing: requests for partitions this node does not lead fail fast with NOT_LEADER_OR_FOLLOWER details carrying the leader's native endpoint
P0 · M10 · Task · deps: X-NATIVE-0045, META-F09 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: ErrorDetail contains the leader id and its native listener endpoint for the same listener name; no request is proxied; a test moves leadership and checks the hint.

### [To Do] X-NATIVE-0051 Cross-protocol test: natively produced records read by the Java consumer, and Kafka-produced records read natively, are identical
P0 · M10 · Task · deps: X-NATIVE-0045, TEST-F08 · labels: test · ref: TEST-F08
Done when: key, value, headers, timestamp and timestamp type match for compressed and uncompressed topics in both directions; the test runs in `cargo xtask compat`.

### [To Do] X-NATIVE-0052 Native produce benchmark: unary, batch and stream throughput and p99 latency versus Kafka-protocol produce on the same node
P1 · M10 · Task · deps: X-NATIVE-0049, TEST-F11 · labels: perf · ref: TEST-F11
Done when: results for 1 KiB and 64 KiB records are recorded in docs/native-api/performance.md; stream throughput is within the target set in the ADR or the gap is explained with a profile.

## [To Do] X-NATIVE-F06 Native consume
P0 · M10 · Feature · deps: X-NATIVE-F03
Scope: Cursor reads, long-poll, server streaming, SSE and WebSocket for HTTP.

### [To Do] X-NATIVE-0053 Cursor token v1: versioned binary (topic id, partition, offset, leader epoch) encoded as base64url, with typed decode errors and a fuzz target
P0 · M10 · Task · deps: TEST-F07 · labels: decoder, security · ref: ADR Native APIs (X-NATIVE-0001)
Done when: round-trip property test passes; malformed, truncated or wrong-version tokens return INVALID_ARGUMENT naming the cursor field; the fuzz target runs in CI; the format is documented in docs/native-api/cursors.md.

### [To Do] X-NATIVE-0054 `ResolveCursor` RPC (`GET /v1/topics/{topic}/partitions/{partition}:resolve`): earliest, latest, by timestamp or explicit offset, returning a cursor and the log start offset, high watermark and last stable offset
P0 · M10 · Task · deps: X-NATIVE-0053, REPL-F14, X-NATIVE-F10 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: DESCRIBE on the topic is required; timestamp resolution matches ListOffsets semantics (tests port ListOffsets cases); an offset outside the log returns OUT_OF_RANGE with the valid range.

### [To Do] X-NATIVE-0055 Batch-to-record conversion for native reads: bounded decompression, trimming records before the cursor offset, `max_records` and `max_bytes` limits, at-least-one-record progress
P0 · M10 · Task · deps: REC-F10 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: property test over random batches and cursors returns exactly the records from the cursor on; a record larger than `max_bytes` is still returned alone; decompression is capped by `max.message.bytes`.

### [To Do] X-NATIVE-0056 Isolation for native reads: read_committed stops at the last stable offset and filters aborted transactional records and control batches server-side
P0 · M10 · Task · deps: X-NATIVE-0055, TXN-F11 · ref: clients/src/main/java/org/apache/kafka/clients/consumer/internals/CompletedFetch.java @4.4.0
Done when: a test with interleaved committed and aborted transactions returns only committed records; control records never reach native clients; read_uncommitted returns aborted data up to the high watermark, as Kafka consumers see it.

### [To Do] X-NATIVE-0057 `Fetch` RPC (`GET /v1/topics/{topic}/partitions/{partition}/records`): cursor read with `max_wait_ms` long-poll through the delayed fetch
P0 · M10 · Task · deps: X-NATIVE-0053, X-NATIVE-0055, X-NATIVE-0056, REPL-F09, X-NATIVE-F04, X-NATIVE-F10 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: READ on the topic is required; the response carries records, next cursor and watermarks; a long-poll completes on new data or after `max_wait_ms` (simulated clock); OFFSET_OUT_OF_RANGE and NOT_LEADER map per X-NATIVE-F10 with leader hint.

### [To Do] X-NATIVE-0058 Cursor leader-epoch validation: a cursor whose epoch diverged after truncation reports the safe resume offset
P0 · M10 · Task · deps: X-NATIVE-0057, STOR-F11 · ref: KIP-320
Done when: a test truncates the log after an unclean election; Fetch with the stale cursor returns FAILED_PRECONDITION with the divergence end offset; the SDK can resume from it.

### [To Do] X-NATIVE-0059 `FetchStream` RPC: server-streaming reads from a cursor with credit-based flow control (`max_inflight_bytes`), idle heartbeats and stream end on leadership change
P0 · M10 · Task · deps: X-NATIVE-0057 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the server never sends more than the granted credit (property test); heartbeats carry the latest watermarks every `alluvium.native.consume.stream.heartbeat.ms`; leadership loss ends the stream with UNAVAILABLE plus a leader hint and the next cursor.

### [To Do] X-NATIVE-0060 Slow-consumer protection: streams without credit for `alluvium.native.consume.stream.stall.timeout.ms` are closed with RESOURCE_EXHAUSTED; buffered bytes per stream are bounded
P0 · M10 · Task · deps: X-NATIVE-0059 · labels: security, nfr · ref: NFR-F07
Done when: a test with a stalled reader keeps server buffering at or below the stream budget and closes the stream after the timeout; `native_stream_closed` is emitted with reason `stalled`.

### [To Do] X-NATIVE-0061 Server-Sent Events: `GET /v1/topics/{topic}/partitions/{partition}/records:sse` with cursors as event ids and `Last-Event-ID` resume
P1 · M10 · Task · deps: X-NATIVE-0059, X-NATIVE-F04 · ref: https://html.spec.whatwg.org/multipage/server-sent-events.html
Done when: a browser-compatible client receives `event: record` messages; reconnecting with `Last-Event-ID` resumes after the last record; SSE uses TCP backpressure instead of credits and is subject to the stall timeout.

### [To Do] X-NATIVE-0062 WebSocket endpoint `GET /v1/ws` with subprotocol `alluvium.v1`: subscribe, credit and close messages, Origin check, max frame size and ping timeout
P1 · M10 · Task · deps: X-NATIVE-0059, X-NATIVE-F04 · labels: security · ref: https://www.rfc-editor.org/rfc/rfc6455
Done when: the handshake rejects an Origin outside the CORS allow-list; frames over `alluvium.native.ws.max.frame.bytes` close with 1009; an unanswered ping closes the socket; authentication happens during the upgrade request only.

### [To Do] X-NATIVE-0063 Cross-protocol read test: native reads of transactional, compressed (all four codecs) and compacted topics match the Java consumer's view record for record
P0 · M10 · Task · deps: X-NATIVE-0057, TEST-F08 · labels: test · ref: TEST-F08
Done when: the comparison test runs in `cargo xtask compat` for read_committed and read_uncommitted.

### [To Do] X-NATIVE-0064 Native consume benchmark: Fetch long-poll and FetchStream throughput and end-to-end latency versus Kafka Fetch
P1 · M10 · Task · deps: X-NATIVE-0059, TEST-F11 · labels: perf · ref: TEST-F11
Done when: results are recorded in docs/native-api/performance.md with CPU per MiB for decompression and conversion.

## [To Do] X-NATIVE-F07 Native consumer groups
P1 · M10 · Feature · deps: X-NATIVE-F06, GRP-F05
Scope: Managed subscriptions, assignment and commits over native APIs.

### [To Do] X-NATIVE-0065 Native group member model: a Subscribe stream maps to a KIP-848 member (member id, epoch, rebalance timeout); the server drives heartbeats over the stream; stream loss starts the session timeout
P1 · M10 · Task · deps: X-NATIVE-F10 · ref: KIP-848
Done when: unit tests map stream open, idle and close to ConsumerGroupHeartbeat-equivalent inputs of the GRP-F05 state machine; a dropped stream fences the member after `group.consumer.session.timeout.ms` (simulated clock).

### [To Do] X-NATIVE-0066 `Subscribe` RPC: bidirectional stream with a subscription (topic list or RE2 regex) and assignment updates
P1 · M10 · Task · deps: X-NATIVE-0065, X-NATIVE-F04 · ref: KIP-848
Done when: READ on the group and DESCRIBE on topics are required; invalid regex returns INVALID_ARGUMENT; assignment frames carry the member epoch; tests cover subscription change mid-stream.

### [To Do] X-NATIVE-0067 Assignment reconciliation: revoked partitions must be acknowledged by the client before reassignment; the rebalance timeout fences the member with FENCED_MEMBER_EPOCH
P1 · M10 · Task · deps: X-NATIVE-0066 · ref: KIP-848
Done when: a two-member test moves a partition only after the revocation ack; a client that never acks is fenced after the rebalance timeout; no partition is assigned to two members at once (property test).

### [To Do] X-NATIVE-0068 Record delivery inside `Subscribe`: reads from the committed offset or `auto_offset_reset` (earliest, latest, none) for each assigned partition, interleaved fairly within the stream credit
P1 · M10 · Task · deps: X-NATIVE-0066, GRP-F07 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: records for revoked partitions stop before the revocation frame; `none` without a committed offset returns a per-partition error frame; fairness test shows no partition starved over 1000 deliveries.

### [To Do] X-NATIVE-0069 `Commit` RPC (`POST /v1/groups/{group}/offsets:commit`): commit offsets with member id and epoch validation and optional metadata
P1 · M10 · Task · deps: X-NATIVE-0065, GRP-F07, X-NATIVE-F04 · ref: KIP-848
Done when: READ on the group and topics is required; STALE_MEMBER_EPOCH and UNKNOWN_MEMBER_ID map per X-NATIVE-F10; commits from a fenced member are rejected; committed values are visible to Kafka OffsetFetch.

### [To Do] X-NATIVE-0070 `FetchCommitted` RPC (`GET /v1/groups/{group}/offsets`): committed offsets with `require_stable` semantics
P1 · M10 · Task · deps: X-NATIVE-0069 · ref: KIP-447
Done when: DESCRIBE on the group is required; `require_stable` waits or returns UNSTABLE_OFFSET_COMMIT while a transactional commit is pending, as in Kafka; partitions without commits are reported as such.

### [To Do] X-NATIVE-0071 `Leave` RPC and stream close: member epoch -1 leaves; static membership through `instance_id` (epoch -2) keeps the assignment for the session timeout
P1 · M10 · Task · deps: X-NATIVE-0065 · ref: KIP-848
Done when: tests cover dynamic leave, static leave, and a static member rejoining within the timeout keeping its partitions.

### [To Do] X-NATIVE-0072 Mixed groups: native members and Java KIP-848 consumers in one group get disjoint assignments and see each other's commits
P1 · M10 · Task · deps: X-NATIVE-0068, X-NATIVE-0069, TEST-F08 · labels: test · ref: TEST-F08
Done when: the compat test runs a rebalance with both member kinds joining and leaving; no partition is consumed by two members; commits from either side are honoured after rebalance.

### [To Do] X-NATIVE-0073 Simulation: native members under stream drops, broker restarts and coordinator failover never hold overlapping assignments and never commit backwards implicitly
P1 · M10 · Task · deps: X-NATIVE-0067, X-NATIVE-0068, X-NATIVE-0069, TEST-F06 · labels: test · ref: TEST-F06
Done when: the simulation runs 1000 seeds in nightly CI with an invariant checker for exclusive ownership; failing seeds replay with `ALLUVIUM_SIM_SEED`.

## [To Do] X-NATIVE-F08 Native admin
P1 · M10 · Feature · deps: X-NATIVE-F03
Scope: Topics, configs, ACLs, groups, quotas, cluster information.

### [To Do] X-NATIVE-0074 Admin RPC conventions: opaque page tokens with `page_size` up to 1000, `validate_only`, and `wait_for_metadata` completion
P1 · M10 · Task · deps: X-NATIVE-F10, X-NATIVE-F04 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: page tokens are bound to the request filter and reject reuse with another filter; `wait_for_metadata` returns only after the local metadata image reflects the change or the deadline passes with DEADLINE_EXCEEDED; shared helpers are unit-tested.

### [To Do] X-NATIVE-0075 `CreateTopic` RPC (`POST /v1/topics`): partitions, replication factor or explicit assignments, configs, `validate_only`
P1 · M10 · Task · deps: X-NATIVE-0074, CTRL-F04 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createTopics @4.4.0
Done when: CREATE on the cluster or topic is required; TOPIC_ALREADY_EXISTS maps to ALREADY_EXISTS, POLICY_VIOLATION and INVALID_REPLICATION_FACTOR to INVALID_ARGUMENT; the topic id is returned; tests cover each error.

### [To Do] X-NATIVE-0076 `DeleteTopic` RPC (`DELETE /v1/topics/{topic}`): by name or topic id
P1 · M10 · Task · deps: X-NATIVE-0074, CTRL-F05 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java deleteTopics @4.4.0
Done when: DELETE on the topic is required; unknown topic returns NOT_FOUND; deleting an internal topic is refused; tests cover name and id addressing.

### [To Do] X-NATIVE-0077 `ListTopics` RPC (`GET /v1/topics`): paged, with internal-topic filter and name prefix
P1 · M10 · Task · deps: X-NATIVE-0074, META-F09 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTopicMetadataRequest @4.4.0
Done when: topics the principal cannot DESCRIBE are omitted silently, as in Kafka Metadata; results are sorted by name; paging is stable across concurrent topic creation.

### [To Do] X-NATIVE-0078 `DescribeTopic` RPC (`GET /v1/topics/{topic}`): partitions, leaders with native endpoints, replicas, ISR, ELR, offline replicas, topic id and authorized operations
P1 · M10 · Task · deps: X-NATIVE-0074, META-F09 · ref: core/src/main/java/kafka/server/handlers/DescribeTopicPartitionsRequestHandler.java @4.4.0
Done when: DESCRIBE on the topic is required (PERMISSION_DENIED with TOPIC_AUTHORIZATION_FAILED, as Kafka returns for explicitly named topics); partition paging works for 10 000-partition topics.

### [To Do] X-NATIVE-0079 `CreatePartitions` RPC (`POST /v1/topics/{topic}:addPartitions`): new total count, optional assignments, `validate_only`
P1 · M10 · Task · deps: X-NATIVE-0074, CTRL-F06 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createPartitions @4.4.0
Done when: ALTER on the topic is required; decreasing the count returns INVALID_ARGUMENT with INVALID_PARTITIONS; tests cover validate_only and explicit assignments.

### [To Do] X-NATIVE-0080 `DeleteRecords` RPC (`POST /v1/topics/{topic}:deleteRecords`): advance the log start offset per partition
P1 · M10 · Task · deps: X-NATIVE-0074, STOR-F13 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteRecordsRequest @4.4.0
Done when: DELETE on the topic is required; an offset above the high watermark returns OUT_OF_RANGE; the new low watermark is returned per partition.

### [To Do] X-NATIVE-0081 `DescribeTopicConfig` RPC (`GET /v1/topics/{topic}/configs`): values with source, synonyms, read-only and sensitive flags
P1 · M10 · Task · deps: X-NATIVE-0074, CFG-F06 · ref: core/src/main/scala/kafka/server/ConfigHelper.scala @4.4.0
Done when: DESCRIBE_CONFIGS on the topic is required; sensitive values are null; output matches DescribeConfigs for the same topic (within-Alluvium differential test).

### [To Do] X-NATIVE-0082 `UpdateTopicConfig` RPC (`PATCH /v1/topics/{topic}/configs`): incremental SET, DELETE, APPEND and SUBTRACT with `validate_only`
P1 · M10 · Task · deps: X-NATIVE-0081, CTRL-F11 · ref: KIP-339
Done when: ALTER_CONFIGS on the topic is required; invalid values return INVALID_ARGUMENT naming the key; alter policies apply; tests cover each operation type.

### [To Do] X-NATIVE-0083 `DescribeBrokerConfig` RPC (`GET /v1/brokers/{broker}/configs`, `GET /v1/cluster/configs`): per-broker and cluster-default configs
P1 · M10 · Task · deps: X-NATIVE-0074, CFG-F04 · ref: core/src/main/scala/kafka/server/ConfigHelper.scala @4.4.0
Done when: DESCRIBE_CONFIGS on the cluster is required; password values are redacted; tests cover dynamic per-broker overrides.

### [To Do] X-NATIVE-0084 `UpdateBrokerConfig` RPC (`PATCH /v1/brokers/{broker}/configs`, `PATCH /v1/cluster/configs`): incremental changes to dynamic broker configs
P1 · M10 · Task · deps: X-NATIVE-0083 · ref: KIP-339
Done when: ALTER_CONFIGS on the cluster is required; read-only keys return INVALID_ARGUMENT; changes take effect through the metadata log (test observes reconfiguration).

### [To Do] X-NATIVE-0085 `ListAcls` RPC (`GET /v1/acls`): filter by resource type, pattern type, name, principal, host, operation and permission
P1 · M10 · Task · deps: X-NATIVE-0074, SEC-F09 · ref: core/src/main/scala/kafka/server/AclApis.scala @4.4.0
Done when: DESCRIBE on the cluster is required; filter semantics match DescribeAcls (MATCH and ANY pattern filters); tests port Kafka filter cases.

### [To Do] X-NATIVE-0086 `CreateAcls` RPC (`POST /v1/acls`): one or more bindings with per-binding results
P1 · M10 · Task · deps: X-NATIVE-0085 · labels: security · ref: core/src/main/scala/kafka/server/AclApis.scala @4.4.0
Done when: ALTER on the cluster is required; invalid bindings (ANY types, empty principal) are rejected per binding; created ACLs are visible to Kafka DescribeAcls.

### [To Do] X-NATIVE-0087 `DeleteAcls` RPC (`POST /v1/acls:delete`): filters with the matched bindings returned
P1 · M10 · Task · deps: X-NATIVE-0085 · labels: security · ref: core/src/main/scala/kafka/server/AclApis.scala @4.4.0
Done when: ALTER on the cluster is required; each filter reports matched and deleted bindings; tests cover overlapping filters.

### [To Do] X-NATIVE-0088 `ListGroups` RPC (`GET /v1/groups`): filter by type (classic, consumer, share, streams) and state
P1 · M10 · Task · deps: X-NATIVE-0074, GRP-F08 · ref: KIP-1043
Done when: groups the principal cannot DESCRIBE are omitted; results aggregate across coordinators; tests cover every type filter.

### [To Do] X-NATIVE-0089 `DescribeGroup` RPC (`GET /v1/groups/{group}`): type, state, epoch, members with assignments and authorized operations
P1 · M10 · Task · deps: X-NATIVE-0074, GRP-F08 · ref: KIP-848
Done when: DESCRIBE on the group is required; consumer and classic groups both render; NOT_FOUND for unknown groups; output matches ConsumerGroupDescribe for the same group.

### [To Do] X-NATIVE-0090 `DeleteGroup` RPC (`DELETE /v1/groups/{group}`)
P1 · M10 · Task · deps: X-NATIVE-0074, GRP-F08 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDeleteGroupsRequest @4.4.0
Done when: DELETE on the group is required; NON_EMPTY_GROUP maps to FAILED_PRECONDITION; committed offsets are removed (verified with OffsetFetch).

### [To Do] X-NATIVE-0091 `AlterGroupOffsets` RPC (`POST /v1/groups/{group}/offsets:reset`): reset to earliest, latest, timestamp or explicit offsets for an empty group, with dry run
P1 · M10 · Task · deps: X-NATIVE-0074, GRP-F07 · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ConsumerGroupCommand.java @4.4.0
Done when: READ on the group and topics is required; resetting an active group returns FAILED_PRECONDITION; dry run returns the computed offsets without writing.

### [To Do] X-NATIVE-0092 `DeleteGroupOffsets` RPC (`POST /v1/groups/{group}/offsets:delete`)
P2 · M10 · Task · deps: X-NATIVE-0074, GRP-F07 · ref: KIP-496
Done when: DELETE on the group and READ on the topics are required; GROUP_SUBSCRIBED_TO_TOPIC maps to FAILED_PRECONDITION per partition.

### [To Do] X-NATIVE-0093 `DescribeQuotas` RPC (`GET /v1/quotas`): filter by entity type and name, including defaults
P1 · M10 · Task · deps: X-NATIVE-0074, QUOTA-F08 · ref: KIP-546
Done when: DESCRIBE_CONFIGS on the cluster is required; output matches DescribeClientQuotas for the same filter.

### [To Do] X-NATIVE-0094 `AlterQuotas` RPC (`PATCH /v1/quotas`): set and remove quota values per entity with `validate_only`
P1 · M10 · Task · deps: X-NATIVE-0093 · ref: KIP-546
Done when: ALTER_CONFIGS on the cluster is required; invalid keys or negative values return INVALID_ARGUMENT; changes take effect for Kafka and native clients.

### [To Do] X-NATIVE-0095 `DescribeCluster` RPC (`GET /v1/cluster`): cluster id, controller id, brokers with racks and native endpoints, finalized features, authorized operations
P1 · M10 · Task · deps: X-NATIVE-0074, META-F09 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleDescribeCluster @4.4.0
Done when: DESCRIBE on the cluster is required for authorized operations only (basic info open to any authenticated principal, as in Kafka); native endpoints are listed per listener name.

### [To Do] X-NATIVE-0096 Admin parity test: each native admin RPC leaves the same metadata, config, ACL, quota and group state as the equivalent Kafka Admin API call
P1 · M10 · Task · deps: X-NATIVE-0075, X-NATIVE-0082, X-NATIVE-0086, X-NATIVE-0094, X-NATIVE-0091, TEST-F08 · labels: test · ref: TEST-F08
Done when: a scripted scenario runs once through native RPCs and once through the Java Admin client on fresh clusters; metadata images compare equal except for ids and timestamps.

## [To Do] X-NATIVE-F09 Native authentication and authorization
P0 · M10 · Feature · deps: X-NATIVE-F03, SEC-F07
Scope: mTLS, OAuth bearer/JWT, scoped API keys; same principals and ACLs as Kafka.

### [To Do] X-NATIVE-0097 Per-listener authentication config `alluvium.native.listener.<name>.authentication` (none, mtls, bearer, api_key; list)
P0 · M10 · Task · deps: CFG-F11 · labels: security · ref: ADR-0006
Done when: `none` maps callers to User:ANONYMOUS and is refused on non-loopback listeners unless `alluvium.allow.plaintext.external=true`; an empty list is a startup error; `check-config` shows the effective methods.

### [To Do] X-NATIVE-0098 mTLS authentication: client certificate subject mapped with `ssl.principal.mapping.rules` to the same principal a Kafka TLS listener would produce
P0 · M10 · Task · deps: X-NATIVE-0097, SEC-F01 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/SslPrincipalMapper.java @4.4.0
Done when: a table test compares principals for the same certificates on a Kafka TLS listener and a native listener; a certificate without a usable subject is rejected with UNAUTHENTICATED.

### [To Do] X-NATIVE-0099 Bearer/JWT authentication: `Authorization: Bearer` (HTTP) and `authorization` metadata (gRPC) validated with the SEC-F05 JWKS cache, `sub` claim mapping, audience, expiry and scopes
P0 · M10 · Task · deps: X-NATIVE-0097, SEC-F05 · labels: security · ref: KIP-768
Done when: no network call happens on the request path (test with JWKS endpoint down after warm-up); expired, wrong-audience, unknown-kid and `alg=none` tokens are rejected with UNAUTHENTICATED; the principal matches the OAUTHBEARER mapping for the same token.

### [To Do] X-NATIVE-0100 API key data model: key id, owner principal, scopes (resource pattern plus operations), expiry, salted hashed secret, as Alluvium metadata records gated by a feature level
P0 · M10 · Task · deps: X-NATIVE-0097, META-F06, META-F02 · labels: security · ref: ADR Native APIs (X-NATIVE-0001)
Done when: record types are added to the vendored-plus-Alluvium metadata schema with an Alluvium feature level; replay into the image is unit-tested; records are absent unless the feature level is finalized; snapshot round-trip test passes.

### [To Do] X-NATIVE-0101 API key secret format and verification: `alk_<key-id>_<secret>` token, PBKDF2-HMAC-SHA-512 with per-key salt and `alluvium.native.apikey.hash.iterations`, constant-time comparison
P0 · M10 · Task · deps: X-NATIVE-0100, TYPE-F08 · labels: security · ref: RFC 8018
Done when: secrets live only in `Secret<T>`; verification uses `subtle`; tests cover wrong secret, truncated token and a key id that does not exist (same response timing class).

### [To Do] X-NATIVE-0102 Controller handling for API key create and revoke: validation, record generation, limit `alluvium.native.apikey.max.per.principal`
P0 · M10 · Task · deps: X-NATIVE-0100, CTRL-F01 · labels: security · ref: ADR Native APIs (X-NATIVE-0001)
Done when: creating a key with scopes broader than the owner's ACLs is allowed but scopes only narrow access (documented); revocation writes a record visible to all brokers within one metadata propagation; tests cover the limit.

### [To Do] X-NATIVE-0103 `ApiKeyService` RPCs: CreateApiKey (secret returned once), ListApiKeys, RevokeApiKey
P0 · M10 · Task · deps: X-NATIVE-0101, X-NATIVE-0102 · labels: security · ref: ADR Native APIs (X-NATIVE-0001)
Done when: principals manage their own keys; managing other principals' keys needs ALTER on the cluster; List never returns secrets or hashes; `api_key_created` and `api_key_revoked` events carry key id and owner only.

### [To Do] X-NATIVE-0104 API key authentication: `Authorization: ApiKey <token>` and gRPC metadata; revoked or expired keys rejected from the current image; scopes intersected with ACLs
P0 · M10 · Task · deps: X-NATIVE-0101, X-NATIVE-0100 · labels: security · ref: ADR Native APIs (X-NATIVE-0001)
Done when: a revoked key fails on the next call after the image update; scopes deny an operation the ACLs allow (test); expiry uses the injected clock.

### [To Do] X-NATIVE-0105 `WhoAmI` RPC (`GET /v1/whoami`): authenticated principal, method, listener and API key scopes
P1 · M10 · Task · deps: X-NATIVE-0098, X-NATIVE-0099, X-NATIVE-0104 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: each authentication method returns its principal; no token material is echoed; useful for support documented in the auth guide.

### [To Do] X-NATIVE-0106 Credential expiry on long-lived streams: expiring bearer tokens and API keys end the stream with UNAUTHENTICATED unless refreshed with an in-stream `RefreshCredentials` frame
P0 · M10 · Task · deps: X-NATIVE-0099, X-NATIVE-0104 · labels: security · ref: KIP-368
Done when: a FetchStream with a token expiring mid-stream ends at expiry (simulated clock); a refresh with a token for a different principal is rejected and ends the stream.

### [To Do] X-NATIVE-0107 Failed-authentication protection: per-IP failure counter with delayed responses (`alluvium.native.authentication.failure.delay.ms`), metric and `native_auth_failed` event with a reason enum
P0 · M10 · Task · deps: X-NATIVE-0097, OBS-F04 · labels: security · ref: docs/standards/security.md
Done when: repeated failures from one IP are delayed while other IPs are unaffected; the event never contains credential material (redaction test); counters have bounded labels.

### [To Do] X-NATIVE-0108 Native authentication security tests: forged and expired JWTs, wrong audience, revoked keys, scope escalation, ANONYMOUS on non-loopback, header injection, token redaction
P0 · M10 · Task · deps: X-NATIVE-0098, X-NATIVE-0099, X-NATIVE-0104, X-NATIVE-0106, OBS-F11 · labels: security, test · ref: SEC-F13
Done when: each case has a named test; logs, metrics and `/debug` output never contain tokens or API key secrets (OBS-F11 checker); the fuzz target for the Authorization header parser runs in CI.

## [To Do] X-NATIVE-F10 Native error model
P0 · M10 · Feature · deps: X-NATIVE-F02, ERR-F02
Scope: gRPC status with details, RFC 9457 problem+json, mapping from internal errors.

### [To Do] X-NATIVE-0109 Native error type: one exhaustive `match` from service-layer errors (every Kafka ErrorCode plus native-only errors) to gRPC code, HTTP status and problem type
P0 · M10 · Task · deps: none · ref: ADR-0004
Done when: the match has no `_ =>` arm; a test iterates every ErrorCode and asserts a mapping exists; the table is the single source used by gRPC, HTTP and the docs generator.

### [To Do] X-NATIVE-0110 gRPC rich status: `google.rpc.Status` details with ErrorInfo (reason = Kafka error name, domain `alluvium`), RetryInfo, BadRequest field violations, ResourceInfo and `alluvium.v1.ErrorDetail`
P0 · M10 · Task · deps: X-NATIVE-0109 · ref: https://cloud.google.com/apis/design/errors
Done when: golden tests decode the `grpc-status-details-bin` trailer for NOT_LEADER (with leader hint), throttling (with RetryInfo) and validation errors (with field paths).

### [To Do] X-NATIVE-0111 RFC 9457 `application/problem+json` for HTTP: `type` URI per error, title, status, detail, instance (request id) and extensions `error_name`, `retriable`, `field_violations`
P0 · M10 · Task · deps: X-NATIVE-0109 · ref: https://www.rfc-editor.org/rfc/rfc9457
Done when: golden JSON tests cover each problem class; the `type` URIs resolve to anchors in docs/native-api/errors.md; content type is exact.

### [To Do] X-NATIVE-0112 Partial-failure rules: whole-request errors versus per-item errors for Produce, ProduceBatch and batch admin RPCs
P0 · M10 · Task · deps: X-NATIVE-0109 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the rule (request-level errors only for authentication, validation of the envelope and overload) is documented and enforced by tests for each batch RPC.

### [To Do] X-NATIVE-0113 Retry signalling: retriable flag from the ERR-F01 registry, `Retry-After` header and RetryInfo for throttling, overload and leader moves
P0 · M10 · Task · deps: X-NATIVE-0110, X-NATIVE-0111, ERR-F01 · ref: https://www.rfc-editor.org/rfc/rfc9110#section-10.2.3
Done when: retriable flags equal Kafka's for every shared error; throttled responses carry the throttle time; the SDK retry policy consumes only these signals (documented).

### [To Do] X-NATIVE-0114 Error message hygiene: no payload bytes, secrets, file paths or stack traces in messages; request id correlates to the server log line
P0 · M10 · Task · deps: X-NATIVE-0109, OBS-F11 · labels: security · ref: docs/standards/security.md
Done when: a test sweeps every error path in the native test suite and checks messages against a deny-list; the request id appears in the problem `instance` and in the request span.

### [To Do] X-NATIVE-0115 Generated error catalog `docs/native-api/errors.md` from the mapping table, with a drift check
P1 · M10 · Task · deps: X-NATIVE-0109, X-NATIVE-0111 · labels: docs · ref: DOCS-F07
Done when: each entry lists gRPC code, HTTP status, problem type, retriable flag and client action; `cargo xtask codegen --check` fails on drift.

## [To Do] X-NATIVE-F11 Native limits and backpressure
P0 · M10 · Feature · deps: X-NATIVE-F03, QUOTA-F01
Scope: Quotas, flow control, max message sizes, timeouts.

### [To Do] X-NATIVE-0116 Deadlines: `grpc-timeout` and `X-Alluvium-Timeout-Ms` clamped to `alluvium.native.request.timeout.max.ms`, default `alluvium.native.request.timeout.default.ms`, propagated to delayed produce and fetch
P0 · M10 · Task · deps: none · ref: https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md
Done when: a long-poll never outlives the client deadline; expired deadlines return DEADLINE_EXCEEDED (504 over HTTP); clamping is visible in the request span.

### [To Do] X-NATIVE-0117 Produce byte-rate quotas for native produce: same entities (principal, client id from `x-alluvium-client-id`) as Kafka; throttle by delaying the response and reporting `throttle_time_ms`
P0 · M10 · Task · deps: X-NATIVE-F05, QUOTA-F02, QUOTA-F07 · ref: KIP-219
Done when: a mixed Kafka and native producer under one principal shares one quota (test); throttle time appears in ErrorDetail and the response; no request is rejected for quota alone.

### [To Do] X-NATIVE-0118 Fetch byte-rate quotas for Fetch, FetchStream and Subscribe: unary responses delayed, streams throttled by withholding credit
P0 · M10 · Task · deps: X-NATIVE-F06, QUOTA-F03 · ref: KIP-219
Done when: measured stream throughput converges to the quota within 10% (simulated clock test); throttle state is visible in `/debug/native/streams`.

### [To Do] X-NATIVE-0119 Request-time quotas for native handlers: handler time recorded per principal and client id, throttling as for Kafka requests
P1 · M10 · Task · deps: QUOTA-F04 · ref: KIP-124
Done when: a CPU-heavy admin loop is throttled; handler time includes JSON transcoding (test).

### [To Do] X-NATIVE-0120 Controller mutation quotas on native CreateTopic, DeleteTopic and CreatePartitions
P1 · M10 · Task · deps: X-NATIVE-F08, QUOTA-F05 · ref: KIP-599
Done when: THROTTLING_QUOTA_EXCEEDED maps to RESOURCE_EXHAUSTED with RetryInfo; quotas are shared with Kafka Admin calls by the same principal.

### [To Do] X-NATIVE-0121 Global native memory budget `alluvium.native.memory.max.bytes`: admission control for request bodies and stream buffers
P0 · M10 · Task · deps: NFR-F07 · labels: nfr · ref: NFR-F07
Done when: over budget, new calls receive RESOURCE_EXHAUSTED (503 with Retry-After over HTTP) before buffering; Kafka listeners are unaffected (test); a gauge exports usage and capacity.

### [To Do] X-NATIVE-0122 Stream caps: `alluvium.native.max.streams.per.principal` and per-connection stream limits for long-lived streams
P0 · M10 · Task · deps: none · labels: security · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the 1+Nth stream for a principal fails with RESOURCE_EXHAUSTED; closing a stream frees the slot immediately; counts are visible as a gauge without principal labels.

### [To Do] X-NATIVE-0123 Size limits: per-record size checked against the topic `max.message.bytes` before batch building; key size, header count and header size caps
P0 · M10 · Task · deps: X-NATIVE-F05 · labels: security · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java @4.4.0
Done when: violations fail before any batch allocation for the record; RECORD_TOO_LARGE maps per X-NATIVE-F10 with the record index; limits documented.

### [To Do] X-NATIVE-0124 Overload test: 10 000 concurrent streams with stalled readers keep RSS under the budget and Kafka listener p99 within the NFR-F07 bound
P1 · M10 · Task · deps: X-NATIVE-0121, X-NATIVE-0122, X-NATIVE-F06, TEST-F11 · labels: nfr, test · ref: NFR-F07
Done when: the macro test runs nightly with recorded RSS and latency; a regression fails the job.

## [To Do] X-NATIVE-F12 Rust SDK
P0 · M10 · Feature · deps: X-NATIVE-F05, X-NATIVE-F06
Scope: `alluvium-client`: produce, consume, groups, admin, retries, docs.

### [To Do] X-NATIVE-0125 Create the `alluvium-client` crate: layer registration, `ClientConfig` (bootstrap endpoints, rustls TLS, timeouts) and a connection pool over gRPC channels
P0 · M10 · Task · deps: none · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the crate depends only on the generated proto crate, alluvium-types and approved dependencies (layer check); config validation errors name the field; `//!` docs state responsibility and invariants.

### [To Do] X-NATIVE-0126 SDK authentication providers: mTLS identity, bearer token provider with refresh before expiry, API key from file or environment
P0 · M10 · Task · deps: X-NATIVE-0125, X-NATIVE-F09 · labels: security · ref: ADR Native APIs (X-NATIVE-0001)
Done when: secrets are held in `Secret<T>` and never logged; the token provider refreshes at 80% of lifetime and sends `RefreshCredentials` on streams; no API to pass secrets on the command line in examples.

### [To Do] X-NATIVE-0127 Metadata cache and leader routing: DescribeCluster and DescribeTopic refresh, leader hints update the cache, bounded refresh backoff
P0 · M10 · Task · deps: X-NATIVE-0125, X-NATIVE-F08 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: a leader move is followed within one retry; refreshes are coalesced; cache size is bounded by topics in use.

### [To Do] X-NATIVE-0128 Retry policy: retriable errors from ErrorDetail, exponential backoff with jitter from an injected RNG, overall deadline
P0 · M10 · Task · deps: X-NATIVE-0125 · ref: KIP-580
Done when: non-retriable errors surface immediately; retries stop at the deadline; deterministic tests with a seeded RNG check the backoff sequence.

### [To Do] X-NATIVE-0129 Producer: batching by `linger` and `batch_size`, murmur2 partitioner, idempotent sequences via InitProducer, delivery futures, bounded `buffer_memory` with backpressure
P0 · M10 · Task · deps: X-NATIVE-0127, X-NATIVE-0128 · ref: clients/src/main/java/org/apache/kafka/clients/producer/KafkaProducer.java @4.4.0
Done when: `send` blocks or fails per config when the buffer is full; partitioning matches the Java client for the same keys; integration test produces 1M records without duplicates under injected connection drops.

### [To Do] X-NATIVE-0130 Producer transport over ProduceStream with an in-flight window and resend of unacknowledged requests on reconnect, preserving sequences
P0 · M10 · Task · deps: X-NATIVE-0129 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: a reconnect test shows no duplicates and no reordering per partition; fallback to unary Produce for HTTP-only deployments is documented as unsupported or implemented behind a config.

### [To Do] X-NATIVE-0131 Partition reader: cursor-based Fetch and FetchStream, resume after reconnect, divergence handling
P0 · M10 · Task · deps: X-NATIVE-0127, X-NATIVE-0128 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: a reader survives leader moves without gaps or duplicates; divergence resumes from the reported offset; credit is granted as the application consumes.

### [To Do] X-NATIVE-0132 Group consumer: Subscribe stream, rebalance callbacks (assigned, revoked), explicit commit and commit-on-revoke
P1 · M10 · Task · deps: X-NATIVE-0131, X-NATIVE-F07 · ref: KIP-848
Done when: callbacks run before the revocation ack; an integration test with three SDK members and one Java member rebalances correctly.

### [To Do] X-NATIVE-0133 Admin client: typed wrappers for every AdminService RPC with paging iterators
P1 · M10 · Task · deps: X-NATIVE-0125, X-NATIVE-F08 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: one wrapper per RPC exists with doc examples; paging iterators stop on the last page; errors are typed.

### [To Do] X-NATIVE-0134 Typed `ClientError` with Kafka error name, retriable flag and context fields; `Display` without payloads
P0 · M10 · Task · deps: X-NATIVE-0125 · ref: ADR-0004
Done when: every ErrorDetail reason maps to a variant or a typed `Server { name, .. }` fallback; a test asserts no payload bytes appear in `Display` or `Debug`.

### [To Do] X-NATIVE-0135 SDK observability: `tracing` spans per call with standard field names; optional metrics through a caller-supplied registry
P1 · M10 · Task · deps: X-NATIVE-0125 · ref: docs/standards/debuggability.md
Done when: spans carry topic, partition and request id; no global metrics registry is used; disabled tracing adds no allocations on the produce hot path (benchmark).

### [To Do] X-NATIVE-0136 SDK integration tests against an in-process Alluvium node: produce and consume round trip, idempotence under connection drops, group rebalance
P0 · M10 · Task · deps: X-NATIVE-0130, X-NATIVE-0131, X-NATIVE-0132 · labels: test · ref: TEST-F08
Done when: tests run in `cargo xtask ci` without fixed ports; failure output includes the server flight recorder.

### [To Do] X-NATIVE-0137 SDK documentation: crate guide and runnable doctest examples for producer, reader, group consumer and admin
P1 · M10 · Task · deps: X-NATIVE-0129, X-NATIVE-0131, X-NATIVE-0132, X-NATIVE-0133 · labels: docs · ref: DOCS-F12
Done when: `cargo doc -D warnings` passes; doctests compile and run against the test server where possible; README links the native API guides.

### [To Do] X-NATIVE-0138 SDK release readiness: crates.io metadata, declared MSRV, `cargo semver-checks` in CI
P1 · M10 · Task · deps: X-NATIVE-0137 · ref: OPS-F01
Done when: `cargo publish --dry-run` succeeds; semver-checks fails CI on a breaking public API change without a major version bump.

### [To Do] X-NATIVE-0139 SDK benchmark: producer throughput and end-to-end latency versus librdkafka on the same node
P2 · M10 · Task · deps: X-NATIVE-0130, X-NATIVE-0131, TEST-F11 · labels: perf · ref: TEST-F11
Done when: results are recorded in docs/native-api/performance.md with the hardware and settings used.

## [To Do] X-NATIVE-F13 Generated SDKs
P2 · M10 · Feature · deps: X-NATIVE-F02
Scope: TypeScript, Python, Go and Java SDKs generated from the IDL, with tests.

### [To Do] X-NATIVE-0140 SDK generation pipeline `cargo xtask sdk-gen` with pinned generators per language and a drift check
P2 · M10 · Task · deps: none · ref: ADR Native APIs (X-NATIVE-0001)
Done when: generator versions are pinned by checksum; generated code for TypeScript, Python, Go and Java is reproducible; the drift check runs in CI.

### [To Do] X-NATIVE-0141 TypeScript SDK: generated client, package skeleton, Node (gRPC) and browser (HTTP/JSON) transports
P2 · M10 · Task · deps: X-NATIVE-0140 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the package builds with type checks; a smoke test produces and fetches one record over each transport.

### [To Do] X-NATIVE-0142 TypeScript SDK: producer, reader, group consumer and admin wrappers with auth providers and retries
P2 · M10 · Task · deps: X-NATIVE-0141 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: wrappers mirror the Rust SDK concepts; API keys and tokens are never logged; unit tests cover retry classification.

### [To Do] X-NATIVE-0143 TypeScript SDK passes the native conformance suite
P2 · M10 · Task · deps: X-NATIVE-0142, X-NATIVE-F17 · labels: test · ref: X-NATIVE-F17
Done when: the conformance runner for TypeScript passes all scenarios in CI or each skipped scenario has an issue link.

### [To Do] X-NATIVE-0144 TypeScript SDK packaging: npm build with provenance, README and version aligned with the IDL version
P3 · M10 · Task · deps: X-NATIVE-0143 · ref: OPS-F01
Done when: `npm pack` output is reproducible; the release job publishes with provenance attestation in a dry run.

### [To Do] X-NATIVE-0145 Python SDK: generated client, package skeleton, gRPC and HTTP/JSON transports, typed stubs
P2 · M10 · Task · deps: X-NATIVE-0140 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the package builds a wheel; mypy passes on the generated stubs; smoke test produces and fetches one record.

### [To Do] X-NATIVE-0146 Python SDK: producer, reader, group consumer and admin wrappers (sync and asyncio) with auth providers and retries
P2 · M10 · Task · deps: X-NATIVE-0145 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: wrappers mirror the Rust SDK concepts; unit tests cover retry classification and credential redaction.

### [To Do] X-NATIVE-0147 Python SDK passes the native conformance suite
P2 · M10 · Task · deps: X-NATIVE-0146, X-NATIVE-F17 · labels: test · ref: X-NATIVE-F17
Done when: the conformance runner for Python passes all scenarios in CI or each skipped scenario has an issue link.

### [To Do] X-NATIVE-0148 Python SDK packaging: wheels and sdist with provenance and README
P3 · M10 · Task · deps: X-NATIVE-0147 · ref: OPS-F01
Done when: the release job builds and verifies wheels in a dry run.

### [To Do] X-NATIVE-0149 Go SDK: generated client, module skeleton and gRPC transport
P2 · M10 · Task · deps: X-NATIVE-0140 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: `go vet` and `go build` pass; smoke test produces and fetches one record.

### [To Do] X-NATIVE-0150 Go SDK: producer, reader, group consumer and admin wrappers with context deadlines, auth providers and retries
P2 · M10 · Task · deps: X-NATIVE-0149 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: wrappers mirror the Rust SDK concepts; unit tests cover retry classification and credential redaction.

### [To Do] X-NATIVE-0151 Go SDK passes the native conformance suite
P2 · M10 · Task · deps: X-NATIVE-0150, X-NATIVE-F17 · labels: test · ref: X-NATIVE-F17
Done when: the conformance runner for Go passes all scenarios in CI or each skipped scenario has an issue link.

### [To Do] X-NATIVE-0152 Go SDK release: module tagging scheme and README
P3 · M10 · Task · deps: X-NATIVE-0151 · ref: OPS-F01
Done when: tagged releases resolve with `go get` in a dry-run job.

### [To Do] X-NATIVE-0153 Java SDK: generated client, Gradle build and gRPC transport
P2 · M10 · Task · deps: X-NATIVE-0140 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the build passes on JDK 17 and 21; smoke test produces and fetches one record.

### [To Do] X-NATIVE-0154 Java SDK: producer, reader, group consumer and admin wrappers with auth providers and retries
P2 · M10 · Task · deps: X-NATIVE-0153 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: wrappers mirror the Rust SDK concepts; unit tests cover retry classification and credential redaction.

### [To Do] X-NATIVE-0155 Java SDK passes the native conformance suite
P2 · M10 · Task · deps: X-NATIVE-0154, X-NATIVE-F17 · labels: test · ref: X-NATIVE-F17
Done when: the conformance runner for Java passes all scenarios in CI or each skipped scenario has an issue link.

### [To Do] X-NATIVE-0156 Java SDK packaging: Maven artifacts with signatures and README
P3 · M10 · Task · deps: X-NATIVE-0155 · ref: OPS-F01
Done when: the release job stages signed artifacts in a dry run.

## [To Do] X-NATIVE-F14 Native admin CLI
P1 · M10 · Feature · deps: X-NATIVE-F12
Scope: `alluvium admin` commands over the native API.

### [To Do] X-NATIVE-0157 `alluvium admin` command group: `--endpoint`, `--tls-ca`, `--api-key-file`, `--token-file`, profile file and `--json` output through the CLI-F01 output module
P1 · M10 · Task · deps: CLI-F01 · labels: security · ref: CLI-F01
Done when: secrets are accepted only from files or environment variables; connection errors print the endpoint and cause; exit codes follow CLI-F01; a test runs `alluvium admin --help`.

### [To Do] X-NATIVE-0158 `alluvium admin topics create` (partitions, replication factor, `--config k=v`, `--validate-only`)
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: human and `--json` output have golden tests; server errors print the problem title and detail with exit code 1.

### [To Do] X-NATIVE-0159 `alluvium admin topics delete` with confirmation prompt and `--yes`
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: without `--yes` in a non-interactive shell it refuses; golden tests cover success and NOT_FOUND.

### [To Do] X-NATIVE-0160 `alluvium admin topics list` (prefix filter, `--internal`, paging through all pages)
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: output is sorted and complete across pages; golden tests for table and JSON.

### [To Do] X-NATIVE-0161 `alluvium admin topics describe` (partitions, leaders, replicas, ISR, ELR, offline)
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: offline and under-replicated partitions are highlighted in table output; golden tests.

### [To Do] X-NATIVE-0162 `alluvium admin topics config get|set|delete|append|subtract`
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: each subcommand maps to Describe/UpdateTopicConfig; sensitive values print as `<redacted>`; golden tests.

### [To Do] X-NATIVE-0163 `alluvium admin topics add-partitions` with optional assignments
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: decreasing the count fails with a clear message; golden tests.

### [To Do] X-NATIVE-0164 `alluvium admin topics delete-records --before <offset>` per partition or from a JSON file
P2 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: confirmation is required; resulting low watermarks are printed; golden tests.

### [To Do] X-NATIVE-0165 `alluvium admin brokers config get|set|delete` for one broker or the cluster default
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: read-only keys fail with the server message; passwords are read from files only; golden tests.

### [To Do] X-NATIVE-0166 `alluvium admin acls list` with Kafka-equivalent filters
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: filters match `kafka-acls.sh --list` semantics (test against the same ACL set); golden tests.

### [To Do] X-NATIVE-0167 `alluvium admin acls create` (principal, host, operation, permission, resource, pattern type)
P1 · M10 · Task · deps: X-NATIVE-0157 · labels: security · ref: X-NATIVE-F08
Done when: per-binding results are printed; invalid combinations are rejected client-side and server-side; golden tests.

### [To Do] X-NATIVE-0168 `alluvium admin acls delete` with a preview of matched bindings and confirmation
P1 · M10 · Task · deps: X-NATIVE-0157 · labels: security · ref: X-NATIVE-F08
Done when: the preview lists exactly the bindings deleted; golden tests.

### [To Do] X-NATIVE-0169 `alluvium admin groups list` with type and state filters
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: every group type renders; golden tests.

### [To Do] X-NATIVE-0170 `alluvium admin groups describe` with members, assignments, committed offsets and lag
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: lag is computed from committed offsets and high watermarks; golden tests.

### [To Do] X-NATIVE-0171 `alluvium admin groups delete` with confirmation
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: a non-empty group fails with a clear message; golden tests.

### [To Do] X-NATIVE-0172 `alluvium admin groups reset-offsets` (`--to-earliest`, `--to-latest`, `--to-datetime`, `--to-offset`, `--dry-run` default)
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ConsumerGroupCommand.java @4.4.0
Done when: dry run is the default and `--execute` applies; output matches `kafka-consumer-groups.sh --reset-offsets` columns; golden tests.

### [To Do] X-NATIVE-0173 `alluvium admin groups delete-offsets --topic <t>`
P2 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: per-partition errors are printed; golden tests.

### [To Do] X-NATIVE-0174 `alluvium admin quotas describe` by user, client id or IP, including defaults
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: output matches DescribeQuotas; golden tests.

### [To Do] X-NATIVE-0175 `alluvium admin quotas alter --add k=v --delete k`
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: `--validate-only` is supported; golden tests.

### [To Do] X-NATIVE-0176 `alluvium admin cluster describe` (cluster id, controller, brokers, native endpoints, features)
P1 · M10 · Task · deps: X-NATIVE-0157 · ref: X-NATIVE-F08
Done when: golden tests for table and JSON.

### [To Do] X-NATIVE-0177 `alluvium admin api-keys create` with scopes and expiry; the secret is written to a file with mode 0600 or printed once on request
P1 · M10 · Task · deps: X-NATIVE-0157, X-NATIVE-F09 · labels: security · ref: X-NATIVE-F09
Done when: the secret never appears in `--json` output unless `--show-secret` is given; file permissions are checked in a test.

### [To Do] X-NATIVE-0178 `alluvium admin api-keys list` (key id, owner, scopes, expiry, created time)
P1 · M10 · Task · deps: X-NATIVE-0157, X-NATIVE-F09 · labels: security · ref: X-NATIVE-F09
Done when: no secret or hash is printed; golden tests.

### [To Do] X-NATIVE-0179 `alluvium admin api-keys revoke <key-id>`
P1 · M10 · Task · deps: X-NATIVE-0157, X-NATIVE-F09 · labels: security · ref: X-NATIVE-F09
Done when: revocation is confirmed by a follow-up list in the test; golden tests.

## [To Do] X-NATIVE-F15 Native API observability
P1 · M10 · Feature · deps: X-NATIVE-F03, OBS-F04
Scope: Spans, metrics and events per endpoint.

### [To Do] X-NATIVE-0180 Span conventions for native calls: `native_request{transport, service, method, request_id, principal, listener}` under the `connection` span
P1 · M10 · Task · deps: OBS-F03 · ref: docs/standards/debuggability.md
Done when: new field names are added to the debuggability field table; a test asserts the span fields for one gRPC and one HTTP call; payloads are never span fields.

### [To Do] X-NATIVE-0181 Per-endpoint metrics: `alluvium_native_requests_total{service,method,transport,code}`, `alluvium_native_request_duration_seconds`, in-flight gauge, bytes in and out
P1 · M10 · Task · deps: X-NATIVE-0180 · ref: docs/standards/debuggability.md#metrics
Done when: label sets are bounded (methods from the generated table); tests assert values after calls through the explicit registry.

### [To Do] X-NATIVE-0182 Stream metrics and events: active streams gauge per method, messages and bytes per stream type, `native_stream_opened` and `native_stream_closed` with a reason enum
P1 · M10 · Task · deps: X-NATIVE-0181, OBS-F07 · ref: docs/standards/debuggability.md#events-for-state-transitions
Done when: reasons cover client_closed, deadline, stalled, unauthenticated, leader_moved, shutdown and error; events are in the crate's `events.rs` catalog.

### [To Do] X-NATIVE-0183 Listener and connection observability: `native_listener_started` and `native_listener_stopped` events, connection open and close with reason, per-connection flight-recorder entries (GOAWAY, resets, abuse detections)
P1 · M10 · Task · deps: X-NATIVE-0181, OBS-F06 · ref: docs/standards/debuggability.md#flight-recorders
Done when: the flight recorder dump for a connection killed by abuse protection shows the triggering frames; counts per listener are exported.

### [To Do] X-NATIVE-0184 Request log line at debug level: one summary per call with timings by phase (authentication, admission, service, encoding)
P2 · M10 · Task · deps: X-NATIVE-0180, SRV-F09 · ref: SRV-F09
Done when: the line format is documented next to the Kafka request log; timings sum to the total within 1 ms in a test.

### [To Do] X-NATIVE-0185 Diagnostics route `/debug/native/streams`: active streams with principal, method, age, credit and throttle state, without payloads
P1 · M10 · Task · deps: X-NATIVE-0182, OBS-F08 · ref: OBS-F08
Done when: the route is bounded (top N by age with a count of the rest); authenticated per OBS-F08 rules.

### [To Do] X-NATIVE-0186 Native metrics and events documented in docs/metrics.md and the events catalog; redaction checks cover native spans and events
P1 · M10 · Task · deps: X-NATIVE-0181, X-NATIVE-0182, X-NATIVE-0183, OBS-F11 · labels: docs · ref: DOCS-F05
Done when: generated docs list every native metric and event; the OBS-F11 checker runs over the native test suite logs.

## [To Do] X-NATIVE-F16 Native API versioning policy
P1 · M10 · Feature · deps: X-NATIVE-F02
Scope: Stability levels, deprecation and compatibility rules.

### [To Do] X-NATIVE-0187 Versioning policy docs/native-api/versioning.md: stability levels (alpha `v1alphaN`, beta `v1betaN`, stable `v1`), compatibility rules per level, deprecation window
P1 · M10 · Task · deps: none · labels: docs · ref: https://cloud.google.com/apis/design/versioning
Done when: the policy defines allowed changes per level, the minimum deprecation period (two minor releases) and how SDK versions map to API versions; linked from the native API ADR.

### [To Do] X-NATIVE-0188 Per-level breaking-change enforcement: strict `buf breaking` rules for stable packages, warnings for alpha, exceptions only with an ADR link
P1 · M10 · Task · deps: X-NATIVE-0187 · ref: https://buf.build/docs/breaking/overview
Done when: CI fixtures prove a stable-package break fails and an alpha-package break warns; the exception list format is validated.

### [To Do] X-NATIVE-0189 `GetServerInfo` RPC (`GET /v1/info`): server version, supported API packages with stability levels and deprecations
P1 · M10 · Task · deps: X-NATIVE-0187 · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the response is generated from the IDL package list; available without authorization beyond authentication; SDKs log a warning when calling a deprecated package.

### [To Do] X-NATIVE-0190 Deprecation signalling: proto `deprecated` options produce `Deprecation` and `Sunset` HTTP headers and an `alluvium-deprecated` gRPC header, plus a usage metric per deprecated method
P1 · M10 · Task · deps: X-NATIVE-0187 · ref: https://www.rfc-editor.org/rfc/rfc9745
Done when: a fixture method marked deprecated emits headers and increments `alluvium_native_deprecated_calls_total{method}`.

### [To Do] X-NATIVE-0191 Forward-compatibility handling: unknown fields from newer clients are ignored; unknown enum values map to `_UNSPECIFIED` and are rejected where the field is required
P1 · M10 · Task · deps: X-NATIVE-0187 · ref: https://protobuf.dev/programming-guides/proto3/#unknowns
Done when: tests send messages with extra fields and future enum values over gRPC and HTTP and observe the documented behaviour.

### [To Do] X-NATIVE-0192 SDK and server compatibility matrix test: the previous released Rust SDK runs the conformance suite against the current server
P2 · M10 · Task · deps: X-NATIVE-0189, X-NATIVE-F17 · labels: test · ref: X-NATIVE-F17
Done when: the CI job pins the previous SDK release; failures block release unless the change is allowed by the policy with an ADR.

## [To Do] X-NATIVE-F17 Native API testing
P0 · M10 · Feature · deps: X-NATIVE-F05, X-NATIVE-F06
Scope: Contract tests, conformance suite, fuzzing of HTTP and gRPC inputs.

### [To Do] X-NATIVE-0193 Language-neutral conformance suite `tests/native/conformance/*.yaml`: scenarios for produce, fetch, groups, admin and errors, with a runner interface SDKs implement
P0 · M10 · Task · deps: none · labels: test · ref: ADR Native APIs (X-NATIVE-0001)
Done when: the Rust SDK runner passes all scenarios; scenario format is documented; each scenario asserts results and error names, never timings.

### [To Do] X-NATIVE-0194 Contract tests: golden request/response pairs for every RPC exercised over gRPC and HTTP/JSON give identical results
P0 · M10 · Task · deps: X-NATIVE-F04, X-NATIVE-F07, X-NATIVE-F08 · labels: test · ref: ADR Native APIs (X-NATIVE-0001)
Done when: a test enumerates the generated RPC table and fails if an RPC lacks a contract case; both transports run per case.

### [To Do] X-NATIVE-0195 Fuzz targets: Protobuf decode plus `validate()` for every request message, generated with the IDL
P0 · M10 · Task · deps: TEST-F07 · labels: test, security, decoder · ref: TEST-F07
Done when: one target per request type exists in `fuzz/fuzz_targets/native_*`; short runs execute per PR and long runs nightly.

### [To Do] X-NATIVE-0196 Fuzz targets for JSON transcoding, HTTP path and query binding, and the WebSocket frame handler
P0 · M10 · Task · deps: X-NATIVE-F04, TEST-F07 · labels: test, security, decoder · ref: TEST-F07
Done when: targets run per PR; a seed corpus from the contract tests is committed.

### [To Do] X-NATIVE-0197 Simulation: native acks=all and idempotent-stream produce plus FetchStream across leader failover lose no acknowledged record and duplicate nothing from one stream
P0 · M10 · Task · deps: TEST-F06 · labels: test · ref: TEST-F06
Done when: 1000 seeds pass nightly with the replication invariant checker; failing seeds print the replay command.

### [To Do] X-NATIVE-0198 Off by default: a node with default config opens no native listener, registers no native metrics beyond zero-valued families, and `check-config` shows native keys at defaults
P0 · M10 · Task · deps: X-NATIVE-F04, X-NATIVE-F07, X-NATIVE-F08, X-NATIVE-F09 · labels: test · ref: ADR-0010
Done when: a test lists the node's listening sockets and finds only Kafka and diagnostics listeners; Kafka ApiVersions and Metadata responses equal the parity differential baseline.

### [To Do] X-NATIVE-0199 Kafka compatibility and differential suites pass with native listeners enabled but unused
P0 · M10 · Task · deps: X-NATIVE-0198, TEST-F08, TEST-F09 · labels: test · ref: ADR-0010
Done when: a CI job variant runs the full compat and differential suites with gRPC and HTTP listeners configured; results equal the default job's.

### [To Do] X-NATIVE-0200 Mixed-load test: concurrent Kafka-protocol and native clients on the same topics keep per-partition ordering and offsets consistent
P1 · M10 · Task · deps: TEST-F08 · labels: test · ref: TEST-F08
Done when: a verifier reads all partitions and confirms contiguous offsets and per-producer order across both client types.

### [To Do] X-NATIVE-0201 Gateway overhead benchmark: HTTP/JSON versus gRPC for Produce and Fetch with 1 KiB and 64 KiB records
P2 · M10 · Task · deps: X-NATIVE-F04, TEST-F11 · labels: perf · ref: TEST-F11
Done when: criterion and macro results are recorded in docs/native-api/performance.md; the CI regression threshold covers JSON transcoding per record.

## [To Do] X-NATIVE-F18 Native API documentation
P1 · M10 · Feature · deps: X-NATIVE-F02
Scope: Generated reference and guides.

### [To Do] X-NATIVE-0202 Generated API reference from proto comments in `docs/native-api/reference/`, published in the docs site with a drift check
P1 · M10 · Task · deps: DOCS-F01 · labels: docs · ref: DOCS-F12
Done when: every service, RPC, message and field has a rendered entry with its HTTP route and limits; drift fails CI.

### [To Do] X-NATIVE-0203 Native API quickstart: enable a listener, create an API key, produce and consume with curl, grpcurl and the Rust SDK
P1 · M10 · Task · deps: X-NATIVE-F12, X-NATIVE-F09 · labels: docs · ref: DOCS-F02
Done when: every command in the page is executed by a docs test against a dev node.

### [To Do] X-NATIVE-0204 Guides: produce semantics (acks, idempotence, partitioning), consuming with cursors and streams, consumer groups over native APIs
P1 · M10 · Task · deps: X-NATIVE-F05, X-NATIVE-F06, X-NATIVE-F07 · labels: docs · ref: ADR Native APIs (X-NATIVE-0001)
Done when: each guide states guarantees and failure behaviour with links to the error catalog; examples compile as doctests.

### [To Do] X-NATIVE-0205 Authentication and authorization guide: mTLS, JWT and API keys, and how they map to Kafka principals and ACLs
P1 · M10 · Task · deps: X-NATIVE-F09 · labels: docs, security · ref: DOCS-F09
Done when: the guide includes a principal-mapping table and a hardening checklist; reviewed against the security standard.

### [To Do] X-NATIVE-0206 Opt-in and migration guide: running native listeners next to Kafka listeners, moving an application from a Kafka client to the native SDK, and rollback
P1 · M10 · Task · deps: X-NATIVE-F12 · labels: docs · ref: ADR-0010
Done when: the guide lists config changes, compatibility guarantees (Kafka clients unaffected), a step-by-step migration and a rollback procedure.

### [To Do] X-NATIVE-0207 Operator page: native listener configuration reference, limits, metrics and suggested alerts
P1 · M10 · Task · deps: X-NATIVE-F11, X-NATIVE-F15 · labels: docs · ref: DOCS-F08
Done when: every `alluvium.native.*` key appears with default and tuning advice; alerts cover auth failures, stalled streams and memory budget saturation.
