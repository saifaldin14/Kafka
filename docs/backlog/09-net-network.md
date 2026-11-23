# NET: Network layer

> Crates: `alluvium-network` (layer 3) · Kafka counterpart: `network/SocketServer`, `common/network/Selector`, `NetworkClient` · Milestones: M2-M7

Listeners, framing, TLS, SASL driving, connection limits, per-connection ordering, backpressure via muting, zero-copy responses, and the outbound inter-node client.

## [To Do] NET-F01 Listener configuration
P0 · M2 · Feature · deps: CFG-F01
Scope: `listeners`, `advertised.listeners`, `listener.security.protocol.map`, inter-broker and controller listener names, loopback default and plaintext-external guard (ADR-0006).

### [To Do] NET-0001 Listener model: `ListenerName` normalisation, `SecurityProtocol` enum and `Endpoint` type
P0 · M2 · Task · deps: CFG-F07 · ref: clients/src/main/java/org/apache/kafka/common/network/ListenerName.java normalised, configPrefix @4.4.0
Done when: `ListenerName` upper-cases like Kafka and exposes the `listener.name.<lower>.` config prefix; `SecurityProtocol` has PLAINTEXT, SSL, SASL_PLAINTEXT, SASL_SSL with Kafka ids; unit tests cover normalisation and prefix.

### [To Do] NET-0002 Parse `listeners` and `advertised.listeners` URIs (`NAME://host:port`, IPv6, empty host, port 0)
P0 · M2 · Task · deps: NET-0001 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java listenerListToEndPoints @4.4.0
Done when: Parser accepts `NAME://host:port`, `[::1]` and empty host (all interfaces) and port 0 (ephemeral); malformed URIs, bad ports and duplicate names return a typed `ListenerConfigError` naming the entry; table tests ported from Kafka cases.

### [To Do] NET-0003 Resolve `listener.security.protocol.map` with Kafka defaults and missing-mapping errors
P0 · M2 · Task · deps: NET-0001 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java LISTENER_SECURITY_PROTOCOL_MAP_DEFAULT @4.4.0
Done when: Default map sends each protocol name to itself; a listener name without a mapping is a startup error naming it; the map is case-insensitive like Kafka; unit tests.

### [To Do] NET-0004 Broker listener cross-checks: unique names and ports, advertised subset, no 0.0.0.0 advertised, inter-broker listener advertised
P0 · M2 · Task · deps: NET-0002, NET-0003 · ref: core/src/main/scala/kafka/server/KafkaConfig.scala validateValues @4.4.0
Done when: Each rule from `KafkaConfig.validateValues` for brokers fails startup with Kafka's wording: duplicate listener names, advertised names not in `listeners`, `inter.broker.listener.name` not advertised, `0.0.0.0` advertised, both `inter.broker.listener.name` and `security.inter.broker.protocol` set; one test per rule.

### [To Do] NET-0005 Controller listener cross-checks: `controller.listener.names` present in `listeners`, never advertised by brokers, `early.start.listeners` subset
P0 · M2 · Task · deps: NET-0002, NET-0003 · ref: core/src/main/scala/kafka/server/KafkaConfig.scala validateValues, earlyStartListeners @4.4.0
Done when: Controller-role and broker-role rules for `controller.listener.names` and `early.start.listeners` match `KafkaConfig.validateValues`; each violation is a startup error tested individually.

### [To Do] NET-0006 Loopback default listener `PLAINTEXT://127.0.0.1:9092` instead of Kafka's `PLAINTEXT://:9092`
P0 · M2 · Task · deps: NET-0002 · labels: security · ref: ADR-0006
Done when: With no `listeners` set the node binds 127.0.0.1:9092; the deviation is recorded in ADR-0006 and docs/parity.md; test asserts the bound address.

### [To Do] NET-0007 Refuse plaintext listeners on non-loopback addresses unless `alluvium.allow.plaintext.external=true`
P0 · M2 · Task · deps: NET-0004, CFG-F11 · labels: security · ref: ADR-0006
Done when: Startup fails with an error naming the listener when PLAINTEXT or SASL_PLAINTEXT binds a non-loopback or wildcard address and the opt-in is false; with the opt-in it starts and logs a warn event `plaintext_external_listener`; tests for 0.0.0.0, ::, a LAN IP and loopback.

### [To Do] NET-0008 Per-listener config overrides via `listener.name.<listener>.<key>` lookup
P0 · M2 · Task · deps: NET-0001 · ref: clients/src/main/java/org/apache/kafka/common/config/AbstractConfig.java valuesWithPrefixOverride @4.4.0
Done when: A typed lookup returns the listener-prefixed value or the global value (Kafka `valuesWithPrefixOverride`); used for ssl.*, sasl.*, max.connections and max.connection.creation.rate; tests show precedence and that mechanism-scoped keys `listener.name.<l>.<mechanism>.<key>` resolve.

### [To Do] NET-0009 Listener role resolution: inter-broker listener, controller listeners, early-start listeners, privileged listener flag
P0 · M2 · Task · deps: NET-0004, NET-0005 · ref: core/src/main/scala/kafka/network/SocketServer.scala createDataPlaneAcceptorAndProcessors @4.4.0
Done when: A `ListenerSet` answers which listener is inter-broker (`inter.broker.listener.name` or derived from `security.inter.broker.protocol`), which are controller listeners and early-start listeners, and marks the privileged (inter-broker) listener used for forwarding checks; unit tests.

### [To Do] NET-0010 Publish actual bound ports for port-0 listeners in advertised endpoints and broker registration
P0 · M2 · Task · deps: NET-0009 · ref: core/src/main/scala/kafka/network/SocketServer.scala boundPort @4.4.0
Done when: After bind, listeners configured with port 0 report their bound port through `ListenerSet::bound_endpoints()`; registration and Metadata responses use it; test binds port 0 and connects to the advertised port.

### [To Do] NET-0011 Module docs for listener configuration and listener docs page
P1 · M2 · Task · deps: NET-0007, NET-0010 · labels: docs
Done when: `//!` docs of the listener module state responsibility and invariants; docs page lists every consumed key, the loopback default and the plaintext guard with links to ADR-0006; doc build passes.

### [To Do] NET-0012 Compatibility test: Java client and librdkafka bootstrap through advertised listeners on a multi-listener node
P0 · M2 · Task · deps: NET-0010, TEST-F08 · labels: test
Done when: In the compat harness a node with an internal and an external listener serves Metadata with the correct per-listener endpoints and both clients produce and consume through each listener.

## [To Do] NET-F02 Acceptor and connection lifecycle
P0 · M2 · Feature · deps: NET-F01, SRV-F02
Scope: Accept loop, connection ids, total and per-IP limits with overrides, idle timeout (`connections.max.idle.ms`).

### [To Do] NET-0013 Bind listener sockets with `socket.listen.backlog.size`, deferred bind for fixed ports and typed bind errors
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/network/SocketServer.scala Acceptor.openServerSocket @4.4.0
Done when: Fixed-port listeners bind when request processing is enabled (Kafka defers opening so readiness probes are not fooled), port-0 listeners bind at construction; `socket.listen.backlog.size` (default 50) is applied; an address-in-use error names the listener; tests on loopback.

### [To Do] NET-0014 Sans-I/O connection state machine with typed close reasons
P0 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/network/KafkaChannel.java @4.4.0
Done when: `ConnectionState` (Handshaking, Authenticating, Ready, Closing, Closed) and `CloseReason` (PeerClosed, IdleTimeout, AuthenticationFailed, AuthenticationTimeout, InvalidRequest, TooManyConnections, SessionExpired, Shutdown, IoError) are pure; every transition is returned as an action; exhaustive transition tests; no Tokio dependency.

### [To Do] NET-0015 Supervised accept loop per listener that survives per-connection errors
P0 · M2 · Task · deps: NET-0013, NET-0014 · ref: core/src/main/scala/kafka/network/SocketServer.scala Acceptor.run, acceptNewConnections @4.4.0
Done when: Each listener runs one supervised task `acceptor-<listener>`; errors on one accepted socket are logged once and the loop continues; EMFILE/ENFILE backs off with a bounded sleep on the injected clock and increments a counter; shutdown cancels the loop; tests inject accept errors.

### [To Do] NET-0016 Connection ids `<local addr>-<peer addr>-<index>` with wrapping index
P0 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/network/ServerConnectionId.java generateConnectionId @4.4.0
Done when: Ids follow `ServerConnectionId` format; the index increments per listener and wraps at `i32::MAX` to 0; ids are never reused while a closed connection's requests are in flight (test with wrap-around); `conn_id` field matches the debuggability standard.

### [To Do] NET-0017 Connection driver task: owns the socket, runs the connection state machine, bounded handoff from the acceptor
P0 · M2 · Task · deps: NET-0015, NET-0016 · ref: core/src/main/scala/kafka/network/SocketServer.scala Acceptor.assignNewConnection, Processor.accept @4.4.0
Done when: Each accepted socket gets a supervised task with a `connection{conn_id, listener, peer_addr}` span; the acceptor hands off through a bounded channel and blocks only that listener when full (Kafka `CONNECTION_QUEUE_SIZE` = 20 semantics); task end always runs the close path; loopback integration test.

### [To Do] NET-0018 Connection accounting per IP, per listener and in total with exact increment/decrement pairing
P0 · M2 · Task · deps: NET-0017 · ref: core/src/main/scala/kafka/network/SocketServer.scala ConnectionQuotas.inc, dec @4.4.0
Done when: Counts increment on accept before configuration and decrement exactly once on every close path (including configuration failure and shutdown); a decrement without a matching increment is an invariant violation; property test over random accept/close sequences keeps counts equal to live connections.

### [To Do] NET-0019 Enforce `max.connections.per.ip`: close the new connection when the IP is at its limit
P0 · M2 · Task · deps: NET-0018 · labels: security · ref: core/src/main/scala/kafka/network/SocketServer.scala ConnectionQuotas.inc TooManyConnectionsException @4.4.0
Done when: A connection from an IP already holding `max.connections.per.ip` connections is closed immediately and logged at info with ip and limit (Kafka wording); counts stay balanced; Alluvium's finite default comes from ADR-0006; tests at limit-1, limit and limit+1.

### [To Do] NET-0020 `max.connections.per.ip.overrides`: parse `host:count` entries, resolve hostnames once, take precedence over the default
P0 · M2 · Task · deps: NET-0019 · ref: core/src/main/scala/kafka/network/SocketServer.scala ConnectionQuotas.maxConnectionsPerIpOverrides @4.4.0
Done when: Overrides accept IPs and hostnames (resolved at config load); an override wins over `max.connections.per.ip`; `max.connections.per.ip=0` is valid only with overrides (Kafka validation message); tests for IPv4, IPv6, hostname and invalid entries.

### [To Do] NET-0021 Broker-wide `max.connections`: the acceptor waits for a free slot instead of accepting
P0 · M2 · Task · deps: NET-0018 · labels: security · ref: core/src/main/scala/kafka/network/SocketServer.scala ConnectionQuotas.waitForConnectionSlot, connectionSlotAvailable @4.4.0
Done when: When the total reaches `max.connections` the acceptor stops accepting (connections queue in the OS backlog) until a slot frees; the wait is woken by any close and by shutdown; time spent blocked feeds `AcceptorBlockedPercent`; simulated-clock test.

### [To Do] NET-0022 Idle connection reaping with `connections.max.idle.ms`
P0 · M2 · Task · deps: NET-0017 · ref: clients/src/main/java/org/apache/kafka/common/network/Selector.java IdleExpiryManager @4.4.0
Done when: A connection with no completed read or write for `connections.max.idle.ms` (default 600000, negative disables) is closed with `CloseReason::IdleTimeout`; activity on either direction resets the timer; tests on the simulated clock including a muted connection waiting on a long response.

### [To Do] NET-0023 Delayed close after failed authentication (`connection.failed.authentication.delay.ms`)
P0 · M2 · Task · deps: NET-0017 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/network/Selector.java delayedClosingChannels @4.4.0
Done when: On authentication failure the error response is flushed, reads stop and the socket closes after `connection.failed.authentication.delay.ms` (default 100; must be below `connections.max.idle.ms` per Kafka validation); pending delayed closes are bounded and all flushed on shutdown; simulated-clock test.

### [To Do] NET-0024 Single close path: drop pending responses, release buffers, decrement counts, notify disconnect listeners
P0 · M2 · Task · deps: NET-0018 · ref: core/src/main/scala/kafka/network/SocketServer.scala Processor.close, processDisconnected @4.4.0
Done when: Every close goes through one function that drops queued responses (recording their request metrics), returns buffers to the pool, decrements counts and notifies registered disconnect listeners (bounded registry, a failing listener is logged and skipped); leak test asserts zero buffers and counts after 10k random closes.

### [To Do] NET-0025 Dynamic reconfiguration of `max.connections.per.ip`, `max.connections.per.ip.overrides` and `max.connections`
P0 · M7 · Task · deps: NET-0020, NET-0021, CFG-F04 · ref: core/src/main/scala/kafka/network/SocketServer.scala SocketServer.reconfigure @4.4.0
Done when: Updates via dynamic broker config apply without restart: per-IP limits apply to new connections, lowering `max.connections` wakes the acceptor and excess connections are closed by the excess-connection policy; info log per change as Kafka; tests.

### [To Do] NET-0026 Add and remove listeners dynamically through `listeners` updates
P2 · M7 · Task · deps: NET-0025 · ref: core/src/main/scala/kafka/network/SocketServer.scala SocketServer.addListeners, removeListeners @4.4.0
Done when: A dynamic `listeners` change starts new acceptors (after the authorizer is ready) and stops removed ones, closing their connections and removing their quotas; follows `DynamicListenerConfig` validation; integration test.

## [To Do] NET-F03 Framing
P0 · M2 · Feature · deps: NET-F02, PROTO-F06
Scope: Size-prefixed frames, `socket.request.max.bytes` check before reading, partial reads, buffer reuse.

### [To Do] NET-0027 Read the 4-byte size prefix and reject negative or oversized frames before allocating
P0 · M2 · Task · deps: none · labels: security, decoder · ref: clients/src/main/java/org/apache/kafka/common/network/NetworkReceive.java readFrom @4.4.0
Done when: A frame with size < 0 or > `socket.request.max.bytes` (default 104857600) closes the connection with `CloseReason::InvalidRequest` and a debug log carrying size and limit, without allocating the body; boundary tests at limit and limit+1.

### [To Do] NET-0028 Frame assembly across partial reads without re-allocation
P0 · M2 · Task · deps: NET-0027 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/network/NetworkReceive.java @4.4.0
Done when: Frames split at any byte boundary (including inside the size prefix) are assembled into one buffer allocated once; property test feeds arbitrary chunkings of random frame sequences and gets identical frames.

### [To Do] NET-0029 Request memory pool bounded by `queued.max.request.bytes`
P0 · M2 · Task · deps: NET-0027 · labels: security · ref: core/src/main/scala/kafka/network/SocketServer.scala SocketServer.memoryPool @4.4.0
Done when: Request buffers come from a pool capped by `queued.max.request.bytes` (validated >= `socket.request.max.bytes` as in Kafka); when the pool cannot satisfy a frame the connection stops reading until memory is released; Alluvium's finite default is recorded in ADR-0006; tests exhaust the pool and recover.

### [To Do] NET-0030 Return request buffers to the pool on completion, error and close
P0 · M2 · Task · deps: NET-0029 · ref: server/src/main/java/org/apache/kafka/network/Request.java releaseBuffer @4.4.0
Done when: Buffers are released after the handler is done with the request, on decode error and on connection close; a test harness counting outstanding buffers reaches zero after mixed success, error and disconnect scenarios.

### [To Do] NET-0031 Memory pool metrics: available, used and depleted time
P1 · M2 · Task · deps: NET-0029, OBS-F04 · labels: nfr · ref: core/src/main/scala/kafka/network/SocketServer.scala memoryPoolSensor @4.4.0
Done when: `alluvium_network_request_memory_available_bytes`, `..._used_bytes` and `..._depleted_seconds_total` mirror `MemoryPoolAvailable`, `MemoryPoolUsed` and `MemoryPoolAvgDepletedPercent`; test asserts values move with allocations.

### [To Do] NET-0032 Fuzz target for the frame reader with arbitrary byte streams and chunking
P0 · M2 · Task · deps: NET-0028, TEST-F07 · labels: security, decoder, test
Done when: `fuzz/fuzz_targets/net_frame_reader.rs` drives the reader with arbitrary bytes and chunk sizes; no panic, hang or allocation above the configured limit in a 10-minute run; seed corpus includes valid frames and TLS/HTTP prefixes.

## [To Do] NET-F04 Request header handling
P0 · M2 · Feature · deps: NET-F03, SRV-F04
Scope: Header decode, unsupported-version handling (ApiVersions v0 error response), handoff to the router.

### [To Do] NET-0033 Decode the request header from the frame and choose header version from API key and version
P0 · M2 · Task · deps: none · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/requests/RequestHeader.java parse @4.4.0
Done when: Header v0-v2 is decoded through the generated codec with `DecodeLimits`; a malformed header closes the connection with `CloseReason::InvalidRequest` and a debug log; unit tests per header version.

### [To Do] NET-0034 Gate API key and version per listener type: unknown keys, disabled APIs and unsupported versions close the connection
P0 · M2 · Task · deps: NET-0033 · ref: server/src/main/java/org/apache/kafka/network/Processor.java parseRequestHeader @4.4.0
Done when: Unknown key ids, keys out of scope for the listener (broker vs controller) and versions outside the advertised range close the connection (Kafka raises InvalidRequestException or UnsupportedVersionException there), except ApiVersions; unstable last versions are accepted only with `unstable.api.versions.enable`; table tests per case.

### [To Do] NET-0035 ApiVersions at an unsupported version: parse as v0 and answer with a v0 UNSUPPORTED_VERSION response listing supported ranges
P0 · M2 · Task · deps: NET-0034 · ref: clients/src/main/java/org/apache/kafka/common/requests/RequestContext.java parseRequest, isUnsupportedApiVersionsRequest @4.4.0
Done when: An ApiVersions request at a version above the supported maximum gets a v0-encoded response (response header v0) with error 35 UNSUPPORTED_VERSION and the full ApiKeys list, as KIP-511 clients expect; raw-bytes test with version 127 and Java client test.

### [To Do] NET-0036 Capture client software name and version from ApiVersions (KIP-511) into connection metadata
P0 · M2 · Task · deps: NET-0033 · ref: server/src/main/java/org/apache/kafka/network/Processor.java processCompletedReceives (KIP-511) @4.4.0
Done when: A valid ApiVersions v3+ request records `client_software_name`/`client_software_version` on the connection (invalid values per Kafka's regex are ignored); the values appear in the request context and connection metadata; tests for valid, invalid and absent values.

### [To Do] NET-0037 Open the `request` span with standard fields when the header is decoded
P0 · M2 · Task · deps: NET-0033, OBS-F03 · labels: nfr
Done when: Each request runs in `request{api, api_version, correlation_id, client_id, principal}` nested in the connection span, using OBS-F03 constructors; a test subscriber asserts the fields on a decoded request.

### [To Do] NET-0038 Hand the request to the router through a bounded queue with its connection context
P0 · M2 · Task · deps: NET-0034, NET-0036, SRV-F05 · ref: server/src/main/java/org/apache/kafka/network/Processor.java processCompletedReceives @4.4.0
Done when: The handoff carries header, body bytes, conn id, peer address and port, listener, security protocol, principal, client info, privileged-listener flag and receive timestamp; the send awaits queue capacity (backpressure, no drop); unit test with a stub router.

### [To Do] NET-0039 Property test: any frame yields exactly one of handoff, ApiVersions fallback or close, never a panic
P0 · M2 · Task · deps: NET-0035, NET-0038 · labels: test, security
Done when: Proptest feeds random headers and bodies (valid and invalid keys and versions) through the header path and asserts one outcome per frame and no panic; committed regressions under proptest-regressions/.

## [To Do] NET-F05 Ordering, in-flight limits and muting
P0 · M2 · Feature · deps: NET-F04
Scope: Per-connection response ordering, max in-flight, muting and unmuting on queue pressure and throttling.

### [To Do] NET-0040 Sans-I/O mute state machine with Kafka's five mute states and four events
P0 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/network/KafkaChannel.java ChannelMuteState, ChannelMuteEvent, handleChannelMuteEvent, maybeUnmute @4.4.0
Done when: `MuteState` (NotMuted, Muted, MutedAndResponsePending, MutedAndThrottled, MutedAndThrottledAndResponsePending) and `MuteEvent` (RequestReceived, ResponseSent, ThrottleStarted, ThrottleEnded) follow Kafka's transition table; an invalid transition is an invariant violation; exhaustive table test.

### [To Do] NET-0041 One request in flight per connection: stop reading after handoff until the response is done
P0 · M2 · Task · deps: NET-0040 · ref: server/src/main/java/org/apache/kafka/network/Processor.java processCompletedReceives (selector.mute) @4.4.0
Done when: After a request is handed off the connection is muted; the next request is read only after the response is written (or a no-op response arrives), so pipelined requests wait in the socket buffer as in Kafka; property test with random handler latencies shows responses in request order.

### [To Do] NET-0042 Handle response kinds: send, no-op (acks=0), close-connection, start-throttling and end-throttling
P0 · M2 · Task · deps: NET-0041 · ref: core/src/main/scala/kafka/network/SocketServer.scala Processor.processNewResponses @4.4.0
Done when: A no-op response unmutes and resumes reading, a close response closes the connection, throttle start/end drive the mute machine; one test per kind in combination with pending responses.

### [To Do] NET-0043 Throttle muting: keep the connection muted after its response until throttling ends
P0 · M2 · Task · deps: NET-0042 · ref: core/src/main/scala/kafka/network/SocketServer.scala Processor.tryUnmuteChannel @4.4.0
Done when: When throttling starts before the response is sent the connection stays muted until both the response is sent and throttling ended, in either order; simulated-clock tests cover both orderings and connection close while throttled.

### [To Do] NET-0044 Queue-pressure backpressure: stop reading while the request queue is full
P0 · M2 · Task · deps: NET-0041 · ref: core/src/main/scala/kafka/network/RequestChannel.scala sendRequest @4.4.0
Done when: A connection whose handoff is waiting on a full request queue reads nothing more; time spent waiting is recorded; no request is dropped; test fills the queue and drains it.

### [To Do] NET-0045 Memory-pressure muting: pause reads when the request pool is exhausted and resume when memory frees
P0 · M2 · Task · deps: NET-0040, NET-0029 · ref: clients/src/main/java/org/apache/kafka/common/network/Selector.java poll (outOfMemory) @4.4.0
Done when: Connections muted for memory are unmuted when available memory rises above the low threshold, but connections muted for other reasons stay muted; test with an exhausted pool and mixed mute reasons.

### [To Do] NET-0046 Ordering invariant checks: response order equals request order per connection
P0 · M2 · Task · deps: NET-0043, NET-0044 · labels: test
Done when: The writer asserts (`debug_assert!`) that the correlation id of each written response matches the oldest outstanding request; a property test with random completion order across many connections never violates it.

## [To Do] NET-F06 Response writing and zero-copy
P0 · M2 · Feature · deps: NET-F05, STOR-F03
Scope: Ordered writes, vectored I/O, `sendfile` for plaintext fetch responses, TLS path.

### [To Do] NET-0047 Build response sends: response header version per API, size prefix, body and record slices
P0 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/requests/RequestContext.java buildResponseSend @4.4.0
Done when: Responses encode header v0 or v1 per API version (ApiVersions always v0), prefix the total size, and keep record data as shared slices from PROTO-F08; golden-byte tests against PROTO-F10 vectors.

### [To Do] NET-0048 Vectored writes with partial-write resumption and a bounded per-connection write buffer
P0 · M2 · Task · deps: NET-0047
Done when: Multi-part responses are written with `writev`; partial writes resume at the right part and offset; per-connection buffered bytes never exceed one response; test with a socket accepting 1 byte at a time.

### [To Do] NET-0049 Spike: zero-copy file-to-socket transfer under `forbid(unsafe_code)`
P0 · M2 · Spike · deps: none · labels: perf · ref: ADR-0007
Done when: ADR chooses how plaintext fetch responses send segment bytes (sendfile through a vetted dependency, Linux and macOS behavior, fallback copy path) with a benchmark comparing options; accepted ADR in docs/adr/.

### [To Do] NET-0050 Plaintext fetch responses: send file regions without copying through user space
P0 · M2 · Task · deps: NET-0049, NET-0048, PROTO-F08 · labels: perf
Done when: File-region parts, the `Segment::Deferred` placeholders of an `EncodedFrame` (PROTO-0103), go from segment file to socket per the zero-copy ADR, resuming partial transfers; a segment shorter than the region (deleted or truncated) ends the response as Kafka's `FileRecords.writeTo` does and closes the connection with an event; test compares bytes received with the segment.

### [To Do] NET-0051 TLS response path: stream file regions through rustls with bounded buffers
P0 · M2 · Task · deps: NET-0048, NET-F07
Done when: On TLS connections file regions are read in bounded chunks and encrypted; memory per connection stays below the configured chunk size plus one TLS record; integration test fetches a 100 MiB batch set over TLS.

### [To Do] NET-0052 Responses for closed connections are dropped with metrics updated
P0 · M2 · Task · deps: NET-0047 · ref: core/src/main/scala/kafka/network/SocketServer.scala Processor.sendResponse @4.4.0
Done when: A response whose connection is gone is discarded, its buffers released and its request metrics recorded (Kafka logs a warn); test closes the client before the handler completes.

### [To Do] NET-0053 Record response-dequeue and send-complete timestamps for request phase metrics
P0 · M2 · Task · deps: NET-0047 · ref: server/src/main/java/org/apache/kafka/network/Request.java responseDequeueTimeNanos, updateRequestMetrics @4.4.0
Done when: Each response records when the connection task picked it up and when the last byte was written; SRV-F08 phases response-queue and response-send are computed from them; unit test on the simulated clock.

## [To Do] NET-F07 TLS
P0 · M2 · Feature · deps: NET-F02
Scope: rustls server config, PEM and PKCS#12 key stores, client auth modes, protocol and cipher config, hot reload, inter-broker hostname verification.

### [To Do] NET-0054 Spike: key store formats under rustls (PEM, PKCS#12; JKS rejected) and unsupported JSSE keys
P0 · M2 · Spike · deps: none · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/DefaultSslEngineFactory.java @4.4.0
Done when: ADR decides supported `ssl.keystore.type`/`ssl.truststore.type` values, the PKCS#12 parsing dependency, and how JSSE-only keys (`ssl.provider`, `ssl.keymanager.algorithm`, `ssl.trustmanager.algorithm`, `ssl.secure.random.implementation`, `ssl.engine.factory.class`) are treated; accepted ADR.

### [To Do] NET-0055 PEM key stores: `ssl.keystore.location` PEM files and inline `ssl.keystore.certificate.chain`/`ssl.keystore.key` with `ssl.key.password`
P0 · M2 · Task · deps: NET-0054, SEC-F11 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/DefaultSslEngineFactory.java PEM_TYPE @4.4.0
Done when: Server config loads PEM chains and PKCS#8 keys from file or inline values, decrypts encrypted keys with `ssl.key.password`, and loads PEM truststores (`ssl.truststore.location` or `ssl.truststore.certificates`); passwords are `Secret`; errors name the key and listener without leaking contents; unit tests with generated certificates.

### [To Do] NET-0056 PKCS#12 key stores and trust stores with `ssl.keystore.password` and `ssl.truststore.password`
P0 · M2 · Task · deps: NET-0054, NET-0055 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/DefaultSslEngineFactory.java @4.4.0
Done when: PKCS#12 files load per the ADR; wrong password and missing key entries give clear startup errors; tests with keystores generated by Java keytool (checked-in fixtures).

### [To Do] NET-0057 Reject JKS stores and non-default JSSE-only settings at startup
P0 · M2 · Task · deps: NET-0054 · labels: security
Done when: `ssl.keystore.type=JKS` and non-default values of the JSSE-only keys fail startup with a message pointing to the docs (deviation recorded per the ADR); defaults pass; tests per key.

### [To Do] NET-0058 `ssl.client.auth` required, requested and none, including the SASL_SSL listener override rule
P0 · M2 · Task · deps: NET-0055 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/SslFactory.java clientAuthConfigOverride @4.4.0
Done when: required rejects clients without a valid certificate, requested accepts both, none never asks; on SASL_SSL listeners only a listener-prefixed `ssl.client.auth` enables client auth (Kafka `clientAuthConfigOverride`); TLS tests per mode.

### [To Do] NET-0059 `ssl.enabled.protocols` and `ssl.protocol` mapped to rustls TLS 1.2 and 1.3
P0 · M2 · Task · deps: NET-0055 · labels: security
Done when: TLSv1.2 and TLSv1.3 map to rustls versions; TLSv1 and TLSv1.1 fail startup with a clear error (deviation in ADR-0006); a client offering only TLS 1.2 is refused when the list is TLSv1.3 only; tests.

### [To Do] NET-0060 `ssl.cipher.suites`: map IANA/JSSE suite names to rustls suites
P0 · M2 · Task · deps: NET-0055 · labels: security
Done when: Configured names map to rustls suites; unknown or unsupported names fail startup naming the suite; empty means rustls defaults; handshake test proves the negotiated suite is in the list.

### [To Do] NET-0061 TLS handshake driver: failure handling, failed-authentication metric and delayed close
P0 · M2 · Task · deps: NET-0058, NET-0023 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/network/SslTransportLayer.java handshake @4.4.0
Done when: Handshake failures log at warn with peer and listener (no key material), count as failed authentication, and close after `connection.failed.authentication.delay.ms`; no Kafka request is read before the handshake completes; tests with an untrusted client certificate and a plaintext client on a TLS port.

### [To Do] NET-0062 Format peer certificate subjects exactly like Java's `X500Principal.getName()` (RFC 2253)
P0 · M2 · Task · deps: none · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/DefaultKafkaPrincipalBuilder.java @4.4.0
Done when: Subject DNs render with Java's attribute keywords, OID fallback, escaping and ordering; golden tests compare against strings produced by the JVM for a corpus of tricky certificates (multi-valued RDNs, UTF-8, emailAddress).

### [To Do] NET-0063 mTLS principal: peer certificate DN through `ssl.principal.mapping.rules`; ANONYMOUS without a certificate
P0 · M2 · Task · deps: NET-0061, NET-0062, SEC-F01 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/DefaultKafkaPrincipalBuilder.java build @4.4.0
Done when: The authenticated principal of an SSL connection is `User:<mapped DN>` using SEC-F01 mapping rules, or `User:ANONYMOUS` without a client certificate; recorded on the connection span; tests for required and requested modes.

### [To Do] NET-0064 Validate replacement key and trust stores: no adding or removing stores, DN and SAN unchanged unless allowed
P0 · M2 · Task · deps: NET-0055 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/SslFactory.java createNewSslEngineFactory, CertificateEntries.ensureCompatible @4.4.0
Done when: A pure validation rejects adding a keystore where none existed, removing one, adding a truststore, or changing the certificate DN or SANs unless `ssl.allow.dn.changes`/`ssl.allow.san.changes`; error messages follow Kafka; one test per rule.

### [To Do] NET-0065 Atomic TLS config swap: new handshakes use the new certificates, existing connections continue
P0 · M2 · Task · deps: NET-0064 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/SslFactory.java reconfigure @4.4.0
Done when: A validated new server config replaces the old one atomically; in-flight connections keep their session; an info event `tls_config_reloaded{listener}` is emitted; test reloads while connections are open.

### [To Do] NET-0066 Trigger TLS reload from dynamic `listener.name.<l>.ssl.*` updates, including same-path file changes
P0 · M7 · Task · deps: NET-0065, CFG-F04 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/DefaultSslEngineFactory.java shouldBeRebuilt @4.4.0
Done when: IncrementalAlterConfigs on listener SSL keys reloads that listener; re-submitting the same path reloads when the file content changed (Kafka `shouldBeRebuilt`); invalid updates are rejected with INVALID_REQUEST-mapped config errors; test with `kafka-configs.sh`.

### [To Do] NET-0067 Outbound TLS client config: key store for mTLS, trust store and `ssl.endpoint.identification.algorithm`
P0 · M2 · Task · deps: NET-0055 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/SslFactory.java createSslEngine @4.4.0
Done when: Inter-node clients build a rustls client config from the listener's ssl.* keys; `https` (default) verifies hostname or IP SANs against the configured peer host without reverse DNS, empty disables verification with a warn event; tests for match, mismatch and IP SAN.

### [To Do] NET-0068 TLS conformance tests with the Java client and librdkafka
P0 · M2 · Task · deps: NET-0063, NET-0056, NET-0060, TEST-F08 · labels: test, security
Done when: Compat harness covers TLS 1.2 and 1.3, required and requested client auth, wrong CA, expired certificate and PKCS#12 stores; each case passes or fails exactly as against Kafka 4.4.0.

## [To Do] NET-F08 SASL driving
P0 · M4 · Feature · deps: NET-F04, SEC-F02
Scope: SaslHandshake v0/v1, SaslAuthenticate, mechanism dispatch, session lifetime, re-authentication (KIP-368).

### [To Do] NET-0069 Sans-I/O SASL server authenticator state machine (INITIAL_REQUEST to COMPLETE/FAILED)
P0 · M4 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java SaslState @4.4.0
Done when: States INITIAL_REQUEST, HANDSHAKE_OR_VERSIONS_REQUEST, HANDSHAKE_REQUEST, AUTHENTICATE, COMPLETE, FAILED with Kafka's transitions are a pure machine in the network crate fed with frames; exhaustive transition tests; no I/O.

### [To Do] NET-0070 Only ApiVersions and SaslHandshake are accepted before authentication
P0 · M4 · Task · deps: NET-0069 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java handleKafkaRequest @4.4.0
Done when: Any other API before the handshake closes the connection (InvalidRequestException semantics) without parsing the body; a non-Kafka first frame logs the wrong-security-protocol hint at debug and closes; tests per case.

### [To Do] NET-0071 ApiVersions during authentication: answered by the authenticator, at most once before the handshake
P0 · M4 · Task · deps: NET-0069, NET-0035 · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java handleApiVersionsRequest @4.4.0
Done when: ApiVersions before the handshake returns the full API list; an unsupported version gets the v0 UNSUPPORTED_VERSION response, an invalid one INVALID_REQUEST; after it only SaslHandshake is accepted; tests for ordering violations.

### [To Do] NET-0072 SaslHandshake v0/v1: mechanism selection, UNSUPPORTED_SASL_MECHANISM with the enabled list
P0 · M4 · Task · deps: NET-0069 · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java handleHandshakeRequest @4.4.0
Done when: A mechanism enabled on the listener moves to AUTHENTICATE; an unknown one returns error 33 with `sasl.enabled.mechanisms` and closes; v1 requires SaslAuthenticate framing; tests for both versions.

### [To Do] NET-0073 SaslHandshake v0: raw size-prefixed SASL tokens without Kafka headers
P0 · M4 · Task · deps: NET-0072 · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java authenticate @4.4.0
Done when: After a v0 handshake, tokens are exchanged as raw size-prefixed frames and the response is a raw token; tests with a handcrafted v0 client for PLAIN.

### [To Do] NET-0074 SaslAuthenticate v0-v2: token exchange, SASL_AUTHENTICATION_FAILED with message, ILLEGAL_SASL_STATE
P0 · M4 · Task · deps: NET-0072, NET-0023 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java handleSaslToken @4.4.0
Done when: Tokens are passed to the mechanism machine; failures return error 58 with the mechanism's client-safe message and then delayed close; a non-SaslAuthenticate request after a v1 handshake returns ILLEGAL_SASL_STATE; an unsupported version returns UNSUPPORTED_VERSION; tokens are never logged; tests per error.

### [To Do] NET-0075 On success build the principal and mark the connection Ready
P0 · M4 · Task · deps: NET-0074, SEC-F01 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/auth/SaslAuthenticationContext.java @4.4.0
Done when: The principal comes from SEC-F01's builder with a SASL authentication context (mechanism, authorization id, listener, client address, TLS session for SASL_SSL); it is set on the connection span and request contexts; test per mechanism stub.

### [To Do] NET-0076 Session lifetime: min of credential expiry and `connections.max.reauth.ms`, sent as `session_lifetime_ms`
P0 · M4 · Task · deps: NET-0075 · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java ReauthInfo.calcCompletionTimesAndReturnSessionLifetimeMs (KIP-368) @4.4.0
Done when: Lifetime follows `calcCompletionTimesAndReturnSessionLifetimeMs` (listener- and mechanism-prefixed `connections.max.reauth.ms`, credential expiry from the mechanism, zero when neither); SaslAuthenticate v1+ carries it; table tests.

### [To Do] NET-0077 Close connections whose session expired when they send a request other than a re-authentication handshake
P0 · M4 · Task · deps: NET-0076 · labels: security · ref: core/src/main/scala/kafka/network/SocketServer.scala Processor.processCompletedReceives (serverAuthenticationSessionExpired) @4.4.0
Done when: A request arriving after session expiry closes the connection with `CloseReason::SessionExpired` and increments the expired-connections-killed counter (`ExpiredConnectionsKilledCount`); simulated-clock test.

### [To Do] NET-0078 Authenticator applies only to SASL_PLAINTEXT and SASL_SSL listeners and reads per-listener mechanism config
P0 · M4 · Task · deps: NET-0075 · ref: clients/src/main/java/org/apache/kafka/common/network/ChannelBuilders.java serverChannelBuilder @4.4.0
Done when: SASL listeners create the authenticator with their mechanism set; SSL and PLAINTEXT listeners never do; SASL_SSL runs SASL after the TLS handshake; integration test with one listener of each protocol.

## [To Do] NET-F09 Connection protections
P1 · M2 · Feature · deps: NET-F02, OBS-F04
Scope: Per-listener limits, connection creation rate, pre-auth request size limits, slow-client protection.

### [To Do] NET-0079 Per-listener `listener.name.<l>.max.connections`: the acceptor waits for a listener slot
P1 · M2 · Task · deps: NET-0021 · labels: security · ref: core/src/main/scala/kafka/network/SocketServer.scala ConnectionQuotas.ListenerConnectionQuota, connectionSlotAvailable @4.4.0
Done when: A listener at its limit stops accepting until one of its connections closes, independent of other listeners; tests with two listeners.

### [To Do] NET-0080 Protect the inter-broker listener from the broker-wide limit and close excess connections LRU-first
P1 · M2 · Task · deps: NET-0079 · labels: security · ref: core/src/main/scala/kafka/network/SocketServer.scala ConnectionQuotas.protectedListener, Processor.closeExcessConnections @4.4.0
Done when: With more than one listener the inter-broker listener ignores `max.connections`; when the total exceeds the limit (after a decrease) the least recently active non-protected connection is closed per iteration; tests.

### [To Do] NET-0081 Acceptor throttle hook: delay or reject accepts for a computed throttle time
P1 · M2 · Task · deps: NET-0021 · ref: core/src/main/scala/kafka/network/SocketServer.scala Acceptor.closeThrottledConnections, ConnectionQuotas.waitForConnectionSlot @4.4.0
Done when: The acceptor accepts a throttle decision per new connection: delay accepting for N ms (interruptible by config change and shutdown) or hold the socket and close it after N ms; held sockets are bounded and closed on shutdown; unit tests on the simulated clock.

### [To Do] NET-0082 Authentication deadline: close connections not Ready within `alluvium.connections.authentication.timeout.ms`
P1 · M2 · Task · deps: CFG-F11 · labels: security · ref: ADR-0006
Done when: A connection still in TLS handshake or SASL exchange at the deadline is closed with `CloseReason::AuthenticationTimeout` and counted; deviation recorded in ADR-0006 (Kafka relies on idle timeout); simulated-clock tests for TLS and (from M4) SASL.

### [To Do] NET-0083 Pre-authentication frame limit `sasl.server.max.receive.size`
P1 · M4 · Task · deps: NET-0069 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java configure (saslAuthRequestMaxReceiveSize) @4.4.0
Done when: Before SASL completes, frames larger than `sasl.server.max.receive.size` (default 524288) fail authentication and close; after authentication `socket.request.max.bytes` applies; boundary tests.

### [To Do] NET-0084 Slow-client protection: close connections whose response write stalls beyond a bound
P1 · M2 · Task · deps: NET-0048, CFG-F11 · labels: security
Done when: A response that makes no write progress for `alluvium.connections.write.stall.timeout.ms` closes the connection with an event; deviation recorded in ADR-0006; test with a client that never reads.

### [To Do] NET-0085 Dynamic per-listener `max.connections` and `max.connection.creation.rate` with Kafka validation
P1 · M7 · Task · deps: NET-0080, CFG-F04 · ref: core/src/main/scala/kafka/network/SocketServer.scala ListenerConnectionQuota.validateReconfiguration, reconfigure @4.4.0
Done when: Listener-prefixed updates apply without restart and wake waiting acceptors; values <= 0 are rejected with Kafka's messages; tests.

### [To Do] NET-0086 Connection-flood test: many connections from many IPs stay within memory bounds while existing clients keep working
P2 · M2 · Task · deps: NET-0080, NET-0084, NET-0082 · labels: test, nfr, security
Done when: A loopback test opens connections beyond every limit from several source addresses; RSS growth stays within the documented per-connection budget and a pre-existing client's produce latency stays within 2x baseline.

## [To Do] NET-F10 Inter-node client
P0 · M2 · Feature · deps: NET-F07, PROTO-F14
Scope: Outbound connection management, version negotiation, timeouts, retries with backoff; used by replication, Raft and controller channels.

### [To Do] NET-0087 Sans-I/O outbound connection states with exponential reconnect backoff and connection-setup timeout
P0 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/clients/ClusterConnectionStates.java @4.4.0
Done when: States DISCONNECTED, CONNECTING, CHECKING_API_VERSIONS, READY, AUTHENTICATION_FAILED; `reconnect.backoff.ms`/`.max.ms` and `socket.connection.setup.timeout.ms`/`.max.ms` grow exponentially with 0.2 jitter from the injected RNG; unit tests on the simulated clock.

### [To Do] NET-0088 Negotiate versions on connect with ApiVersions, retrying at the version the peer reports
P0 · M2 · Task · deps: NET-0087 · ref: clients/src/main/java/org/apache/kafka/clients/NetworkClient.java handleApiVersionsResponse @4.4.0
Done when: A new connection sends ApiVersions at our latest version, retries at the peer's maximum on UNSUPPORTED_VERSION, then stores the peer ranges and lets PROTO-F14 choose per-request versions; tests against a stub peer at older versions.

### [To Do] NET-0089 In-flight tracking per peer with a configured cap and correlation-id matching
P0 · M2 · Task · deps: NET-0087 · ref: clients/src/main/java/org/apache/kafka/clients/InFlightRequests.java, clients/src/main/java/org/apache/kafka/clients/NetworkClient.java correlate @4.4.0
Done when: Requests carry increasing correlation ids; a response whose id does not match the oldest in-flight request disconnects the peer (Kafka `correlate`); the in-flight cap per peer is configurable per caller; unit tests.

### [To Do] NET-0090 Request timeouts: disconnect the peer and fail all its in-flight requests
P0 · M2 · Task · deps: NET-0089 · ref: clients/src/main/java/org/apache/kafka/clients/NetworkClient.java handleTimedOutRequests @4.4.0
Done when: A request older than its timeout (`request.timeout.ms`, `replica.socket.timeout.ms` or `controller.socket.timeout.ms` per caller) disconnects the peer and fails every in-flight request with a typed timeout error; simulated-clock tests.

### [To Do] NET-0091 Disconnect handling: fail in-flight requests with typed errors and enter backoff
P0 · M2 · Task · deps: NET-0089 · ref: clients/src/main/java/org/apache/kafka/clients/NetworkClient.java processDisconnection @4.4.0
Done when: Peer close, I/O error and authentication failure each fail outstanding requests with a distinct error variant and move the peer to backoff (authentication failure is not retried until backoff expires); tests.

### [To Do] NET-0092 Tokio driver: one supervised task per peer with a bounded request queue and cancellation
P0 · M2 · Task · deps: NET-0090, NET-0091, SRV-F02
Done when: Each peer connection runs as a named supervised task; callers enqueue through a bounded channel and await; shutdown cancels queued requests with a `Shutdown` error; tests for backpressure and shutdown.

### [To Do] NET-0093 Outbound TLS connections with endpoint identification
P0 · M2 · Task · deps: NET-0092, NET-0067 · labels: security
Done when: Peers on SSL listeners connect with the NET-F07 client config and hostname verification; a certificate mismatch is an authentication failure with backoff; integration test against a TLS listener.

### [To Do] NET-0094 Choose outbound endpoints: inter-broker listener for brokers, controller listener for controllers
P0 · M2 · Task · deps: NET-0092, NET-0009
Done when: Broker-to-broker connections use the target's `inter.broker.listener.name` endpoint from metadata; controller connections use `controller.quorum.bootstrap.servers` or voters on the first `controller.listener.names` entry; tests with multi-listener metadata.

### [To Do] NET-0095 Retry policy with `retry.backoff.ms` and `retry.backoff.max.ms` for callers
P0 · M2 · Task · deps: NET-0087 · ref: clients/src/main/java/org/apache/kafka/common/utils/internals/ExponentialBackoff.java @4.4.0
Done when: A reusable retry policy (exponential with jitter, max attempts or deadline) is used by replication, Raft and controller channels; unit tests on the simulated clock; no retry loop without a deadline.

### [To Do] NET-0096 Node-to-controller channel: active-controller discovery, NOT_CONTROLLER re-routing and per-request deadlines
P0 · M2 · Task · deps: NET-0092, NET-0095, NET-0094, RAFT-F10 · ref: server/src/main/java/org/apache/kafka/server/NodeToControllerRequestThread.java @4.4.0
Done when: Requests to the active controller are queued (bounded) with deadlines; the target follows the Raft leader; NOT_CONTROLLER or disconnect triggers rediscovery and retry until the deadline; used by broker lifecycle, AlterPartition and forwarding; tests with a changing leader.

### [To Do] NET-0097 Outbound client metrics per peer role: connections, reconnects, request latency and in-flight count
P1 · M2 · Task · deps: NET-0092, OBS-F04 · labels: nfr
Done when: Metrics `alluvium_network_client_*` labelled by client role (replica_fetcher, raft, controller_channel) and never by peer id cardinality beyond node ids; test asserts counts after reconnects.

## [To Do] NET-F11 Network metrics and events
P1 · M2 · Feature · deps: NET-F02, OBS-F04
Scope: Connection counts, rates, bytes, auth failures, mute events, per listener.

### [To Do] NET-0098 Per-listener connection metrics: count, creation and close rates and totals
P1 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/network/Selector.java SelectorMetrics @4.4.0
Done when: `alluvium_network_connections`, `..._connections_created_total` and `..._connections_closed_total{reason}` per listener map to Kafka's `connection-count`, `connection-creation-rate/total` and `connection-close-rate/total`; test asserts values after opens and closes.

### [To Do] NET-0099 Per-listener byte and request/response counters
P1 · M2 · Task · deps: NET-0098 · ref: clients/src/main/java/org/apache/kafka/common/network/Selector.java SelectorMetrics @4.4.0
Done when: Incoming and outgoing bytes, request and response counts and size histograms per listener map to `incoming-byte-rate`, `outgoing-byte-rate`, `request-rate`, `response-rate`, `request-size-avg/max`; tests.

### [To Do] NET-0100 Authentication metrics for TLS: successful and failed authentication counters
P1 · M2 · Task · deps: NET-0098, NET-0061 · labels: security
Done when: `alluvium_network_authentications_total{listener, result}` counts TLS handshakes as Kafka's `successful-authentication-*` and `failed-authentication-*`; tests with good and bad certificates.

### [To Do] NET-0101 SASL authentication metrics: per mechanism success and failure, re-authentication counts and latency, no-reauth total
P1 · M4 · Task · deps: NET-0100, NET-0075 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/network/Selector.java SelectorMetrics @4.4.0
Done when: Counters and a latency histogram labelled by listener and mechanism cover `successful-authentication-*`, `failed-authentication-*`, `successful-reauthentication-*`, `failed-reauthentication-*`, `reauthentication-latency-*`, `successful-authentication-no-reauth-total`; tests.

### [To Do] NET-0102 Connections by client software name and version (KIP-511) with bounded cardinality
P1 · M2 · Task · deps: NET-0098, NET-0036 · ref: clients/src/main/java/org/apache/kafka/common/network/ChannelMetadataRegistry.java @4.4.0
Done when: A gauge counts connections per (client_software_name, client_software_version) with a cap on distinct pairs (excess bucketed as `other`); test exceeds the cap.

### [To Do] NET-0103 Network event catalog: connection accepted, rejected, closed, muted, TLS and authentication failures
P1 · M2 · Task · deps: NET-0098, OBS-F07 · labels: nfr
Done when: `events.rs` lists `connection_accepted` (debug), `connection_rejected{reason}` (info), `connection_closed{reason}` (debug), `connection_muted`/`connection_unmuted{reason}` (trace), `tls_handshake_failed` (warn), `authentication_failed` (info) with fields; docs/events.md generated; tests capture each event.

### [To Do] NET-0104 Per-connection flight recorder of state and mute transitions, dumped on error close
P1 · M2 · Task · deps: NET-0103, NET-0040, OBS-F06 · labels: nfr
Done when: Each connection keeps a small bounded ring of typed transitions; it is logged when the connection closes for an error reason and included in the panic dump; memory per connection is fixed; test triggers an error close and checks the dump.

### [To Do] NET-0105 Mute metrics: currently muted connections and mute events by reason
P1 · M2 · Task · deps: NET-0098, NET-0045, NET-0043
Done when: `alluvium_network_muted_connections{reason}` and `..._mute_events_total{reason}` cover queue pressure, memory pressure, throttling and one-in-flight; tests per reason.

## [To Do] NET-F12 Socket options
P2 · M2 · Feature · deps: NET-F02
Scope: `socket.send.buffer.bytes`, `socket.receive.buffer.bytes`, TCP_NODELAY, keepalive.

### [To Do] NET-0106 Apply `socket.send.buffer.bytes` and `socket.receive.buffer.bytes` (-1 for OS default) to listeners and accepted sockets
P2 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/network/SocketServer.scala Acceptor.configureAcceptedSocketChannel, openServerSocket @4.4.0
Done when: The receive buffer is set on the listening socket before bind (so window scaling applies) and the send buffer on accepted sockets; -1 leaves OS defaults; requested and actual sizes are logged at debug; getsockopt test on loopback.

### [To Do] NET-0107 Enable TCP_NODELAY and SO_KEEPALIVE on accepted sockets
P2 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/network/SocketServer.scala Acceptor.configureAcceptedSocketChannel @4.4.0
Done when: Accepted sockets have TCP_NODELAY and SO_KEEPALIVE set as Kafka does; getsockopt test.

### [To Do] NET-0108 Set SO_REUSEADDR on listening sockets so restarts do not fail during TIME_WAIT
P2 · M2 · Task · deps: none
Done when: A node restarted immediately after shutdown with connections in TIME_WAIT binds the same port; test restarts a listener with open client sockets.

### [To Do] NET-0109 Outbound socket options for inter-node connections
P2 · M2 · Task · deps: NET-0092
Done when: Inter-node sockets set TCP_NODELAY and the caller's send/receive buffer sizes (`replica.socket.receive.buffer.bytes` for fetchers, -1 default otherwise); getsockopt test.

## [To Do] NET-F13 Graceful drain
P0 · M2 · Feature · deps: NET-F05, SRV-F03
Scope: Stop accepting, drain in-flight requests within a bound, close connections on shutdown.

### [To Do] NET-0110 Stop accepting: close listener sockets and stop acceptor tasks first on shutdown
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/network/SocketServer.scala SocketServer.stopProcessingRequests @4.4.0
Done when: Shutdown closes every listening socket before anything else, so new connection attempts are refused; event `listeners_closed` with elapsed_ms; test connects after the step and is refused.

### [To Do] NET-0111 Drain in-flight requests within `alluvium.shutdown.drain.timeout.ms`, then close connections
P0 · M2 · Task · deps: NET-0110, CFG-F11
Done when: After accepting stops, connections stop reading new requests, in-flight requests may complete and their responses are written until the drain timeout, then all connections close; event reports drained and dropped counts; deviation from Kafka's immediate close recorded in ADR-0006; simulated-clock test.

### [To Do] NET-0112 Drop queued requests that never started at shutdown with metrics
P0 · M2 · Task · deps: NET-0111 · ref: core/src/main/scala/kafka/network/RequestChannel.scala clear @4.4.0
Done when: Requests still in the router queue when draining ends are dropped (Kafka `RequestChannel.clear`), buffers released and counted in `alluvium_server_requests_dropped_total{reason="shutdown"}`; test.

### [To Do] NET-0113 Two-phase network shutdown API for the server shutdown sequence
P0 · M2 · Task · deps: NET-0112 · ref: core/src/main/scala/kafka/network/SocketServer.scala SocketServer.shutdown @4.4.0
Done when: The network exposes `stop_processing()` (stop accept, drain) and `shutdown()` (release listeners, quotas, pools) called at the positions SRV-F03 defines; calling them out of order is a no-op with a warn; test of the sequence.

### [To Do] NET-0114 Drain behaviour tests: acks=1 produce completes, long-poll fetch bounded by the timeout
P0 · M2 · Task · deps: NET-0113 · labels: test
Done when: Integration test shuts down during an acks=1 produce (client receives its response) and during a 30 s long-poll fetch (connection closed at the drain timeout); no leaked tasks reported by the supervisor.

## [To Do] NET-F14 Simulated transport
P0 · M3 · Feature · deps: NET-F10, TEST-F02
Scope: Transport backed by the simulated network for driver-level simulation.

### [To Do] NET-0115 Transport seam: listener and stream traits implemented by Tokio TCP
P0 · M3 · Task · deps: none · labels: test
Done when: A documented test-seam trait covers bind, accept, connect, read, write and shutdown; the production implementation wraps Tokio TCP with no behavior change (existing tests pass unchanged).

### [To Do] NET-0116 Simulated transport backed by the simulated network
P0 · M3 · Task · deps: NET-0115 · labels: test
Done when: Streams over TEST-F02 deliver bytes in order with configured latency; drops and partitions surface as connection resets or connect timeouts; seeded and replayable; unit tests.

### [To Do] NET-0117 Stream fault injection: half-open connections, zero-window readers, mid-frame disconnects and short writes
P0 · M3 · Task · deps: NET-0116 · labels: test
Done when: Fault plan options produce each fault deterministically; network-layer tests show idle reaping, write-stall closing and frame resumption behave under each.

### [To Do] NET-0118 Run the server network layer (accept, framing, muting, responses) on the simulated transport
P0 · M3 · Task · deps: NET-0116 · labels: test
Done when: A simulated node accepts simulated clients; framing, mute and ordering invariants hold across 1000 seeds with the fault plan.

### [To Do] NET-0119 Run the inter-node client on the simulated transport
P0 · M3 · Task · deps: NET-0116 · labels: test
Done when: Backoff, timeouts, controller re-routing and version negotiation are exercised in simulation; failures print the seed and connection flight recorders.

### [To Do] NET-0120 TLS and SASL over the simulated transport
P0 · M3 · Task · deps: NET-0118, NET-0119 · labels: test, security
Done when: rustls and the SASL authenticator run over simulated streams so authentication timeouts and re-authentication are simulated; tests with seeded certificates.

### [To Do] NET-0121 Determinism check: the same seed yields the same network event trace
P0 · M3 · Task · deps: NET-0118, NET-0119 · labels: test
Done when: A test runs a scenario twice with one seed and compares hashes of the connection event log; different seeds differ.

## [To Do] NET-F15 Network performance
P2 · M2 · Feature · deps: NET-F06, TEST-F11
Scope: Benchmarks and tuning: buffer sizes, `num.network.threads` mapping, write batching.

### [To Do] NET-0122 Criterion benchmarks for frame reading, header decode and response encoding
P2 · M2 · Task · deps: none · labels: perf
Done when: Benches under `crates/alluvium-network/benches/` with baselines checked into the benchmark job; results recorded in the PR.

### [To Do] NET-0123 Loopback produce and fetch throughput benchmark compared with Kafka 4.4.0
P2 · M2 · Task · deps: NET-0122 · labels: perf
Done when: The macro harness runs the standard workloads against both systems on the same host and records throughput and p99 latency for the network path.

### [To Do] NET-0124 Tune read buffer sizing with benchmark evidence
P2 · M2 · Task · deps: NET-0123 · labels: perf
Done when: Initial read size and growth policy are chosen from benchmark results that are recorded in the PR; no regression in the macro benchmark.

### [To Do] NET-0125 Decide and document how `num.network.threads` maps onto the Tokio runtime
P2 · M2 · Task · deps: NET-0122 · labels: docs
Done when: The key is accepted for compatibility, its effect (or lack of one) is documented in docs/config.md and docs/parity.md, and dynamic updates do not fail; test.

### [To Do] NET-0126 Evaluate response write coalescing and keep it only with measured gain
P2 · M2 · Task · deps: NET-0123 · labels: perf
Done when: Coalescing of small writes per connection is benchmarked; it is merged only if p99 or throughput improves by a recorded margin, otherwise the result is documented and the change dropped.

### [To Do] NET-0127 Measure and assert idle-connection memory footprint
P2 · M2 · Task · deps: NET-0122, NFR-F05 · labels: nfr
Done when: A test opens 10k idle connections and asserts memory per connection stays under the budget set in NFR-F05; the number is recorded in docs.

### [To Do] NET-0128 Add network benchmarks to the CI regression job
P2 · M2 · Task · deps: NET-0122, NET-0123 · labels: perf
Done when: Network benches run in the benchmark job with regression thresholds; a deliberate slowdown fails the job.
