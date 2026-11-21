# SRV: Server runtime and request pipeline

> Crates: `alluvium-server` (layer 4) · Kafka counterpart: `KafkaApis`, `BrokerServer`, `ControllerServer`, `KafkaRequestHandler`, `DelayedOperationPurgatory` · Milestones: M2-M7

Node assembly, supervision, lifecycle, request routing and queuing, delayed operations, request metrics, log directory management, storage formatting, and the protocol-agnostic service layer.

## [To Do] SRV-F01 Node assembly and roles
P0 · M2 · Feature · deps: CFG-F02, OBS-F01
Scope: `process.roles` (broker, controller, combined), `node.id`, ordered component startup.

### [To Do] SRV-0001 Parse and validate `process.roles` and `node.id`
P0 · M2 · Task · deps: CFG-F07 · ref: core/src/main/scala/kafka/server/KafkaConfig.scala parseProcessRoles, validateValues @4.4.0
Done when: `broker`, `controller` or both parse into a `ProcessRoles` set; empty or unknown roles fail with Kafka's message (ZooKeeper mode is not supported); `node.id` and `broker.id` must match when both are set; unit tests per case.

### [To Do] SRV-0002 Role-dependent validation of quorum and listener settings
P0 · M2 · Task · deps: SRV-0001, CFG-F10, NET-F01 · ref: core/src/main/scala/kafka/server/KafkaConfig.scala validateValues @4.4.0
Done when: Controller role requires `controller.quorum.voters` or `controller.quorum.bootstrap.servers` and, with static voters, its own id among them; broker-only nodes must not list controller listeners in `listeners` and must not be voters; each rule tested with Kafka's wording.

### [To Do] SRV-0003 Startup step framework: ordered steps with start/end/elapsed logging, a startup deadline and reverse teardown
P0 · M2 · Task · deps: SRV-F02 · ref: core/src/main/scala/kafka/server/BrokerServer.scala startup (FutureUtils.waitWithLogging) @4.4.0
Done when: Startup runs a typed list of steps; each logs `startup_step_started`/`startup_step_completed{step, elapsed_ms}`; waits log what they wait for (Kafka `waitWithLogging`); `server.max.startup.time.ms` bounds the whole startup; a failing step tears down started steps in reverse; tests with fake steps on the simulated clock.

### [To Do] SRV-0004 Process status guard: SHUTDOWN, STARTING, STARTED, SHUTTING_DOWN
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/BrokerServer.scala maybeChangeStatus @4.4.0
Done when: Start and shutdown are idempotent (second calls are no-ops with a debug log) and shutdown during startup is handled; transition table tests.

### [To Do] SRV-0005 Shared components for combined mode: Raft manager, metadata loader, snapshot generator, metrics registry, fault handlers
P0 · M2 · Task · deps: SRV-0003 · ref: core/src/main/scala/kafka/server/SharedServer.scala startForBroker, startForController, stopForBroker, stopForController @4.4.0
Done when: Shared components start with the first role that needs them and stop after the last role stops; a broker-only, controller-only and combined node each start them exactly once; tests count starts and stops.

### [To Do] SRV-0006 Combined mode ordering: controller starts before broker, broker shuts down before controller
P0 · M2 · Task · deps: SRV-0005 · ref: core/src/main/scala/kafka/server/KafkaRaftServer.scala startup, shutdown @4.4.0
Done when: In combined mode the controller component completes startup first so its endpoints reach the Raft manager, and on shutdown the broker stops first so controlled shutdown can reach the controller; test asserts the order from step events.

### [To Do] SRV-0007 Controller startup sequence
P0 · M2 · Task · deps: SRV-0003, SRV-0005, NET-F01 · ref: core/src/main/scala/kafka/server/ControllerServer.scala startup @4.4.0
Done when: Steps follow `ControllerServer.startup`: metadata cache and controller publishers, controller-listener network (not accepting), credential and token caches, wait for voter connections, quorum controller, quota managers, request handlers, install publishers, controller registration, then enable request processing; each step is logged with timing; integration test of a controller-only node.

### [To Do] SRV-0008 Broker component construction order
P0 · M2 · Task · deps: SRV-0003, SRV-0005 · ref: core/src/main/scala/kafka/server/BrokerServer.scala startup @4.4.0
Done when: Steps follow `BrokerServer.startup`: scheduler, log-dir failure channel, metadata cache, lifecycle client, credential and token caches, controller channel, network (not accepting), quota managers, replica manager, coordinators, request handler pool, metadata publishers; each step logged with timing; test asserts the order.

### [To Do] SRV-0009 Broker registration and heartbeat client: incarnation id, registration retries, heartbeats, catch-up and unfence signals
P0 · M2 · Task · deps: SRV-0008, NET-F10 · ref: server/src/main/java/org/apache/kafka/server/BrokerLifecycleManager.java start, sendBrokerRegistration, sendBrokerHeartbeat @4.4.0
Done when: The broker registers with incarnation id, listeners, supported feature ranges, log-dir ids and rack; retries until `initial.broker.registration.timeout.ms` then shuts down; heartbeats every `broker.heartbeat.interval.ms` with `wantFence` until ready and the current metadata offset; `isCaughtUp` and `!isFenced` complete the catch-up and unfence signals; simulated-clock tests against a stub controller.

### [To Do] SRV-0010 Broker readiness waits: publishers installed, controller catch-up, first publish, unfence, authorizer ready, acceptors started
P0 · M2 · Task · deps: SRV-0008, SRV-0009 · ref: core/src/main/scala/kafka/server/BrokerServer.scala startup (initialCatchUpFuture, firstPublishFuture, setReadyToUnfence, enableRequestProcessing) @4.4.0
Done when: After construction the broker waits, in Kafka's order and each under the startup deadline, for publisher installation, the controller acknowledging catch-up, the first metadata publish, unfencing, authorizer readiness per endpoint and all acceptors started; early-start (controller) listeners open before the authorizer finishes; a timeout fails startup naming the wait.

### [To Do] SRV-0011 Startup failure: fatal log, ordered shutdown of started components and non-zero exit
P0 · M2 · Task · deps: SRV-0010, SRV-0007, SRV-0026 · ref: core/src/main/scala/kafka/server/BrokerServer.scala startup (catch block) @4.4.0
Done when: Any startup error logs once at error with the failed step, runs the shutdown sequence for what started and exits with the documented startup-failure code; test injects a failure at each step.

### [To Do] SRV-0012 Startup banner and build-info metric
P1 · M2 · Task · deps: SRV-0003, OBS-F04 · ref: clients/src/main/java/org/apache/kafka/common/utils/internals/AppInfoParser.java registerAppInfo @4.4.0
Done when: On start the node logs one info event with version, commit, node id, roles and cluster id, and exports `alluvium_build_info{version, commit}` and start time like Kafka's `app-info` metrics; test asserts both.

### [To Do] SRV-0013 Feed liveness and readiness from startup and broker state
P0 · M2 · Task · deps: SRV-0010 · labels: nfr
Done when: Liveness is true once the process runs; readiness becomes true only after the last startup step and false from the start of shutdown; the state is exposed through the readiness interface that OBS-F09 serves; test.

### [To Do] SRV-0014 Single-node combined-mode integration test: format, start, produce and consume, shut down
P0 · M2 · Task · deps: SRV-0011, SRV-0013, SRV-0030, TEST-F08 · labels: test
Done when: An in-process combined node starts from a formatted directory, the Java client produces and consumes, and the step events show the documented order with timings; no leaked tasks after shutdown.

### [To Do] SRV-0015 Document node assembly, roles and the startup sequence
P1 · M2 · Task · deps: SRV-0010, SRV-0007 · labels: docs
Done when: docs/architecture.md §6 and the server crate module docs describe roles, shared components and every startup step with what it waits for; doc build passes.

## [To Do] SRV-F02 Task supervisor
P0 · M2 · Feature · deps: TYPE-F06, OBS-F01
Scope: Named, traced, tracked tasks; panic escalation; shutdown ordering; `/debug/tasks`.

### [To Do] SRV-0016 Supervisor spawn API: named, owned, traced tasks with a shutdown group
P0 · M2 · Task · deps: none · ref: ADR-0002
Done when: `Supervisor::spawn(name, group, span, future)` is the only way to start a task; the handle exposes join and cancel; unit tests show the span is attached and the task is registered.

### [To Do] SRV-0017 Forbid direct `tokio::spawn`, `spawn_blocking` and `std::thread::spawn` outside the supervisor
P0 · M2 · Task · deps: SRV-0016, FND-F02
Done when: clippy `disallowed-methods` entries with reasons reject direct spawns; the supervisor module carries the single scoped `#[expect]`; a test crate using `tokio::spawn` fails lint.

### [To Do] SRV-0018 Task registry: name, group, state, start time and owner for every live task
P0 · M2 · Task · deps: SRV-0016
Done when: The registry snapshot lists running and stopping tasks with age; finished tasks are removed; per-connection tasks are aggregated by name prefix with counts; the snapshot is the data source for `/debug/tasks` (OBS-F08); unit tests.

### [To Do] SRV-0019 Escalate unexpected task exits: a critical task that returns or errors triggers fatal shutdown
P0 · M2 · Task · deps: SRV-0018 · ref: ADR-0004
Done when: Tasks marked critical that finish early or return an error log once at error with the task name and reason, then start shutdown with the fatal exit code; non-critical errors are logged and counted; tests for both kinds.

### [To Do] SRV-0020 Hierarchical cancellation and ordered shutdown groups with per-group timeouts
P0 · M2 · Task · deps: SRV-0018
Done when: Groups (network accept, request draining, fetchers and coordinators, logs, Raft) cancel in order; each group is awaited up to its timeout and stuck tasks are reported by name; tests with a task that ignores cancellation.

### [To Do] SRV-0021 Periodic scheduler for background jobs (`background.threads` equivalent)
P0 · M2 · Task · deps: SRV-0016 · ref: server-common/src/main/java/org/apache/kafka/server/util/KafkaScheduler.java @4.4.0
Done when: Named periodic and one-shot jobs with fixed-delay semantics run on supervised tasks, use the injected clock and stop on shutdown; a job that panics is a bug (abort); tests on the simulated clock.

### [To Do] SRV-0022 Task metrics: running tasks by group, spawns and exits by reason
P1 · M2 · Task · deps: SRV-0018, OBS-F04 · labels: nfr
Done when: `alluvium_tasks_running{group}`, `alluvium_tasks_spawned_total{group}` and `alluvium_tasks_exited_total{group, reason}` exist with bounded labels; tests.

### [To Do] SRV-0023 Test helper asserting no supervised tasks remain after shutdown
P0 · M2 · Task · deps: SRV-0020 · labels: test
Done when: `assert_no_tasks_left(&supervisor)` fails with the list of remaining task names; used by server integration tests.

### [To Do] SRV-0024 Supervisor module docs and rust.md section on spawning
P1 · M2 · Task · deps: SRV-0020, SRV-0021 · labels: docs
Done when: Module docs state the invariants (every task named, owned, cancellable); docs/standards/rust.md describes how to spawn and pick a group; doc build passes.

## [To Do] SRV-F03 Lifecycle and shutdown
P0 · M2 · Feature · deps: SRV-F02
Scope: Signal handling, ordered shutdown with timings, controlled-shutdown handshake with the controller.

### [To Do] SRV-0025 Signal handling: SIGTERM and SIGINT start graceful shutdown, a second signal forces exit
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/Kafka.scala main (shutdown hook) @4.4.0
Done when: The first signal starts the shutdown sequence with an info event; a second signal exits immediately with the forced-exit code; tests run the binary as a child process.

### [To Do] SRV-0026 Documented exit codes for clean exit, startup failure, fatal fault, shutdown timeout and forced exit
P0 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/utils/internals/Exit.java @4.4.0
Done when: Exit codes are one enum mapped in one place; docs list them; tests assert the code for each path.

### [To Do] SRV-0027 Fault handlers: fatal faults exit, non-fatal faults log and count
P0 · M2 · Task · deps: SRV-0026 · ref: core/src/main/scala/kafka/server/SharedServer.scala raftManagerFaultHandler, metadataLoaderFaultHandler, metadataPublishingFaultHandler @4.4.0
Done when: Raft manager, controller startup, initial broker metadata load and fatal controller faults terminate the process; metadata loading is fatal only with the controller role; metadata publishing and non-fatal controller faults log once and increment `metadata-apply-error-count`/`metadata-load-error-count`; tests per handler.

### [To Do] SRV-0028 Shutdown step framework: each step logged with start, end and elapsed_ms; errors logged and the sequence continues
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/BrokerServer.scala shutdown @4.4.0
Done when: Every step emits `shutdown_step_completed{step, elapsed_ms}`; a failing step is logged once and later steps still run (Kafka `Utils.swallow`); an overall deadline bounds the sequence and stuck steps are reported; tests.

### [To Do] SRV-0029 Controlled-shutdown handshake: `wantShutDown` heartbeats until the controller allows exit or the deadline passes
P0 · M2 · Task · deps: SRV-0028, SRV-0009 · ref: server/src/main/java/org/apache/kafka/server/BrokerLifecycleManager.java beginControlledShutdown, BrokerHeartbeatResponseEvent @4.4.0
Done when: With `controlled.shutdown.enable=true` the broker enters pending controlled shutdown, heartbeats with `wantShutDown`, sends the first follow-up after 50 ms, and continues when `shouldShutDown` is true or the deadline expires (error log); test against a stub controller.

### [To Do] SRV-0030 Broker shutdown sequence in Kafka's order
P0 · M2 · Task · deps: SRV-0029 · ref: core/src/main/scala/kafka/server/BrokerServer.scala shutdown @4.4.0
Done when: Steps: controlled shutdown, stop lifecycle, network stop processing, request handlers, scheduler, coordinators, replica manager, controller channels, log manager (clean shutdown marker with broker epoch), quota managers, network shutdown, lifecycle close; each step timed; test asserts the order.

### [To Do] SRV-0031 Controller and shared-component shutdown sequence
P0 · M2 · Task · deps: SRV-0028 · ref: core/src/main/scala/kafka/server/ControllerServer.scala shutdown, core/src/main/scala/kafka/server/SharedServer.scala stop @4.4.0
Done when: Controller: registration channel, network stop, quorum controller, handlers, quota managers, network shutdown; shared: metadata loader and snapshot generator begin shutdown, Raft manager last; each step timed; test asserts the order in combined mode.

### [To Do] SRV-0032 Controlled shutdown in a three-node cluster moves leadership away before exit
P0 · M3 · Task · deps: SRV-0030, TEST-F06 · labels: test
Done when: Simulation and real-process tests show partitions led by the stopping broker get new leaders before it exits and no acknowledged acks=all write is lost.

### [To Do] SRV-0033 Document shutdown order, timeouts and exit codes
P1 · M2 · Task · deps: SRV-0030, SRV-0031 · labels: docs
Done when: Operations-facing docs and architecture §6 list each shutdown step, its timeout and the exit codes; doc build passes.

## [To Do] SRV-F04 Request router
P0 · M2 · Feature · deps: PROTO-F07
Scope: ApiKey to handler dispatch, version validation, listener restrictions, UNSUPPORTED_VERSION handling.

### [To Do] SRV-0034 Handler interface and `handlers/<api>.rs` layout
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle @4.4.0
Done when: A handler receives the request context and decoded body and returns a response or a typed error; the module layout and docs are in place; a sample handler compiles with tests.

### [To Do] SRV-0035 Exhaustive ApiKey dispatch for broker and controller routers
P0 · M2 · Task · deps: SRV-0034 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handle, core/src/main/scala/kafka/server/ControllerApis.scala handle @4.4.0
Done when: Two routers (broker, controller) match every `ApiKey` without a `_ =>` arm; keys not handled by a role map to an explicit not-in-scope branch that is unreachable after the network gate (invariant); a test fails when an advertised API has no handler.

### [To Do] SRV-0036 Decode request bodies with DecodeLimits; decode failures close the connection
P0 · M2 · Task · deps: SRV-0035 · labels: security, decoder · ref: clients/src/main/java/org/apache/kafka/common/requests/RequestContext.java parseRequest @4.4.0
Done when: Body decode at the header version uses PROTO-F09 limits; a malformed body closes the connection after logging once at debug, as Kafka does when request parsing fails; tests with truncated bodies.

### [To Do] SRV-0037 Error responses: handler errors become the API's error response or close the connection
P0 · M2 · Task · deps: SRV-0035 · ref: core/src/main/scala/kafka/server/RequestHandlerHelper.scala handleError, sendErrorOrCloseConnection @4.4.0
Done when: Typed handler errors map through ERR-F02 to an error code and the API's `getErrorResponse` form; APIs without an error form close the connection with error counts recorded; unexpected errors become UNKNOWN_SERVER_ERROR and are logged once at error with the request context; tests per path.

### [To Do] SRV-0038 Advertised API ranges per listener type from the implemented-API registry
P0 · M2 · Task · deps: SRV-0035 · ref: server/src/main/java/org/apache/kafka/server/DefaultApiVersionManager.java apiVersionResponse @4.4.0
Done when: The ApiVersions data lists only implemented and tested versions for the listener type; on brokers, forwardable APIs are intersected with the active controller's ranges; unstable versions appear only with `unstable.api.versions.enable`; tests compare against the registry.

### [To Do] SRV-0039 Run deferred completion actions after each handler returns
P0 · M2 · Task · deps: SRV-0035, SRV-F07 · ref: server/src/main/java/org/apache/kafka/server/DelayedActionQueue.java @4.4.0
Done when: Actions queued during handling (for example completing delayed operations whose keys changed) run after the handler returns, outside partition locks, as Kafka's `tryCompleteActions`; test shows no re-entrant completion.

### [To Do] SRV-0040 Differential tests for unsupported, disabled and out-of-scope requests against Kafka 4.4.0
P0 · M2 · Task · deps: SRV-0037, SRV-0038, TEST-F09 · labels: test
Done when: The differential harness sends disabled APIs, unsupported versions and controller-only APIs to brokers and compares connection outcome and responses with Kafka.

## [To Do] SRV-F05 Request queue and handler pool
P0 · M2 · Feature · deps: SRV-F04, SRV-F02
Scope: `queued.max.requests`, `num.io.threads` mapping, bounded queue with muting, queue metrics.

### [To Do] SRV-0041 Bounded request queue sized by `queued.max.requests`
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/network/RequestChannel.scala sendRequest (requestQueue) @4.4.0
Done when: Requests from all connections of a role go through one bounded queue (default 500); enqueue waits when full so connections stop reading; nothing is dropped; tests fill and drain the queue.

### [To Do] SRV-0042 Request handler pool sized by `num.io.threads`
P0 · M2 · Task · deps: SRV-0041 · ref: core/src/main/scala/kafka/server/KafkaRequestHandler.scala KafkaRequestHandler.run, KafkaRequestHandlerPool @4.4.0
Done when: N supervised handler tasks pull from the queue; a handler error never stops the worker; dequeue time is stamped on the request; tests with N=1 and N=8.

### [To Do] SRV-0043 Callback queue for asynchronous completions, served before new requests
P0 · M2 · Task · deps: SRV-0042 · ref: core/src/main/scala/kafka/network/RequestChannel.scala receiveRequest, sendCallbackRequest @4.4.0
Done when: Completions from coordinators and other async work re-enter the pool through a bounded callback queue that handlers drain before taking new requests; callback time is added to local time; tests show priority.

### [To Do] SRV-0044 Separate queue and handler pool for the controller role
P0 · M2 · Task · deps: SRV-0042 · ref: core/src/main/scala/kafka/server/ControllerServer.scala startup (controllerApisHandlerPool) @4.4.0
Done when: Controller listeners use their own bounded queue and handler pool so broker load cannot starve controller requests in combined mode; test saturates the broker queue while controller requests complete.

### [To Do] SRV-0045 Queue and handler metrics: request and response queue sizes, capacity, idle percent
P0 · M2 · Task · deps: SRV-0042, OBS-F04 · labels: nfr · ref: core/src/main/scala/kafka/network/RequestChannel.scala RequestQueueSizeMetric, core/src/main/scala/kafka/server/KafkaRequestHandler.scala KafkaRequestHandlerPool @4.4.0
Done when: `alluvium_request_queue_size`, `..._capacity`, `alluvium_response_queue_size` and handler idle ratio per pool and aggregate mirror `RequestQueueSize`, `ResponseQueueSize` and `RequestHandlerAvgIdlePercent`; tests.

### [To Do] SRV-0046 Per-handler reusable buffers (`RequestLocal`) with a size cap
P2 · M2 · Task · deps: SRV-0042 · labels: perf · ref: server-common/src/main/java/org/apache/kafka/server/common/RequestLocal.java @4.4.0
Done when: Each handler owns a bounded buffer supplier used for decompression and record validation; buffers above the cap are not retained; benchmark shows reduced allocations on produce.

### [To Do] SRV-0047 Handler pool shutdown: stop taking requests, finish current ones within a bound
P0 · M2 · Task · deps: SRV-0042 · ref: core/src/main/scala/kafka/server/KafkaRequestHandler.scala initiateShutdown, awaitShutdown @4.4.0
Done when: Shutdown lets workers finish the request in hand, stops them taking more, and reports any worker that misses the deadline; test with a slow handler.

### [To Do] SRV-0048 Dynamic `num.io.threads` resize
P0 · M7 · Task · deps: SRV-0042, CFG-F04 · ref: core/src/main/scala/kafka/server/KafkaRequestHandler.scala resizeThreadPool, server/src/main/java/org/apache/kafka/server/DynamicThreadPool.java @4.4.0
Done when: Updating `num.io.threads` grows or shrinks the pool without dropping requests; validation follows Kafka's dynamic thread pool rules; test resizes under load.

## [To Do] SRV-F06 Request context
P0 · M2 · Feature · deps: SRV-F04, SEC-F01
Scope: Principal, listener, security protocol, client address, client software name and version, request timing.

### [To Do] SRV-0049 Request context: header fields, connection id, client address and port, principal, listener, protocol, client software, privileged flag
P0 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/requests/RequestContext.java @4.4.0
Done when: `RequestContext` carries every field of Kafka's `RequestContext` with domain types; `Debug` prints no secrets; constructed by the network handoff; unit tests.

### [To Do] SRV-0050 Session view (principal and client address) for authorization and quotas
P0 · M2 · Task · deps: SRV-0049 · ref: server/src/main/java/org/apache/kafka/network/Session.java @4.4.0
Done when: A `Session` borrowed from the context is what the authorizer and quota managers receive; tests.

### [To Do] SRV-0051 Request timing marks on the monotonic clock
P0 · M2 · Task · deps: SRV-0049 · ref: server/src/main/java/org/apache/kafka/network/Request.java requestDequeueTimeNanos, apiLocalCompleteTimeNanos, callbackRequestDequeueTimeNanos @4.4.0
Done when: The context records receive, dequeue, local-complete, callback dequeue/complete and response-complete instants from the injected clock; setters enforce monotonic order with `debug_assert!`; tests.

### [To Do] SRV-0052 Forwarded request context: inner request with the envelope's principal and client address
P0 · M2 · Task · deps: SRV-0049 · ref: server/src/main/java/org/apache/kafka/network/Request.java isForwarded, envelope, headerForLoggingOrThrottling @4.4.0
Done when: For requests unwrapped from Envelope the context takes the serialized principal (via the principal serde), the original client address and client id, and marks the request forwarded; throttling and logging use the right header (`headerForLoggingOrThrottling`); tests.

### [To Do] SRV-0053 Project the request context to the protocol-agnostic service context
P0 · M2 · Task · deps: SRV-0049, SRV-0078
Done when: A conversion builds `ServiceContext` (principal, client address, client id, deadline, span) for the service layer; unit test.

## [To Do] SRV-F07 Delayed operations framework
P0 · M2 · Feature · deps: SRV-F02, TYPE-F06
Scope: Purgatory equivalent: timer wheel, watch keys, try-complete, expiration, metrics.

### [To Do] SRV-0054 Delayed operation model: try-complete, on-complete and on-expiration with exactly-once completion
P0 · M2 · Task · deps: none · ref: server-common/src/main/java/org/apache/kafka/server/purgatory/DelayedOperation.java forceComplete, run @4.4.0
Done when: A delayed operation is completed exactly once either by `try_complete` succeeding, by force completion or by expiry; completion runs `on_complete` once and expiry also runs `on_expiration` first; concurrent attempts are safe; unit tests.

### [To Do] SRV-0055 Hierarchical timing wheel on the injected clock (1 ms tick, 20 slots, overflow wheels)
P0 · M2 · Task · deps: none · ref: server-common/src/main/java/org/apache/kafka/server/util/timer/TimingWheel.java, server-common/src/main/java/org/apache/kafka/server/util/timer/SystemTimer.java @4.4.0
Done when: Add, cancel and advance follow Kafka's `TimingWheel`; property test against a sorted-deadline reference model shows every task fires once, never before its deadline and within one tick after it.

### [To Do] SRV-0056 Expiration driver: advance the wheel and expire due operations
P0 · M2 · Task · deps: SRV-0055, SRV-0054 · ref: server-common/src/main/java/org/apache/kafka/server/purgatory/DelayedOperationPurgatory.java advanceClock, ExpiredOperationReaper @4.4.0
Done when: A supervised task sleeps until the next bucket deadline (bounded at 200 ms like Kafka's reaper) and expires due operations; shutdown stops it; simulated-clock tests.

### [To Do] SRV-0057 Watch keys and `try_complete_else_watch` with the try-watch-try ordering
P0 · M2 · Task · deps: SRV-0054, SRV-0055 · ref: server-common/src/main/java/org/apache/kafka/server/purgatory/DelayedOperationPurgatory.java tryCompleteElseWatch @4.4.0
Done when: An operation is tried, then watched on each key, tried again, and only then added to the timer; typed keys (topic-partition and others) hash into a fixed number of shards; tests cover completion between the two tries.

### [To Do] SRV-0058 `check_and_complete(key)`: try watched operations of a key and remove completed ones
P0 · M2 · Task · deps: SRV-0057 · ref: server-common/src/main/java/org/apache/kafka/server/purgatory/DelayedOperationPurgatory.java checkAndComplete, removeKeyIfEmpty @4.4.0
Done when: Returns the number completed; completed operations are removed from that key's list; empty keys are removed; tests.

### [To Do] SRV-0059 Purge completed operations from watcher lists by purge interval
P0 · M2 · Task · deps: SRV-0058, SRV-0056 · ref: server-common/src/main/java/org/apache/kafka/server/purgatory/DelayedOperationPurgatory.java advanceClock @4.4.0
Done when: When estimated watched operations minus delayed operations exceed the purgatory's purge interval (`fetch.purgatory.purge.interval.requests`, `producer.purgatory.purge.interval.requests`, `delete.records.purgatory.purge.interval.requests`), completed entries are purged; memory stays bounded in a long test with many keys.

### [To Do] SRV-0060 Cancel operations for a key and purgatory shutdown semantics
P0 · M2 · Task · deps: SRV-0058 · ref: server-common/src/main/java/org/apache/kafka/server/purgatory/DelayedOperationPurgatory.java cancelForKey, shutdown @4.4.0
Done when: `cancel_for_key` removes and returns watched operations without completing them (used when a partition is deleted); shutdown stops expiry and drops remaining operations with a count; tests.

### [To Do] SRV-0061 Purgatory metrics per purgatory: watched size, delayed count, completions and expirations
P0 · M2 · Task · deps: SRV-0059, OBS-F04 · labels: nfr · ref: server-common/src/main/java/org/apache/kafka/server/purgatory/DelayedOperationPurgatory.java (PurgatorySize, NumDelayedOperations) @4.4.0
Done when: `alluvium_purgatory_watched{purgatory}`, `..._delayed{purgatory}`, `..._completed_total` and `..._expired_total` map to `PurgatorySize` and `NumDelayedOperations`; tests.

### [To Do] SRV-0062 Property and simulation tests: exactly-once completion, no early expiry, bounded memory
P0 · M2 · Task · deps: SRV-0059, SRV-0060, TEST-F01 · labels: test
Done when: Random sequences of adds, key events and clock advances never complete an operation twice, never expire before its deadline and leave no completed operation watched after a purge; seeds printed on failure.

### [To Do] SRV-0063 Benchmark purgatory throughput and memory per watched operation
P2 · M2 · Task · deps: SRV-0059 · labels: perf
Done when: Criterion bench watches and completes 100k operations across 10k keys; results recorded; memory per operation documented.

### [To Do] SRV-0064 Delayed operations module docs
P1 · M2 · Task · deps: SRV-0057 · labels: docs
Done when: Module docs explain the try-watch-try ordering, completion guarantees and how callers define keys; doc build passes.

## [To Do] SRV-F08 Request metrics by phase
P0 · M2 · Feature · deps: SRV-F05, OBS-F04
Scope: Queue, local, remote, throttle, response-queue, send and total time per API.

### [To Do] SRV-0065 Compute request phases: queue, local, remote, throttle, response-queue, response-send, total
P0 · M2 · Task · deps: SRV-0051, NET-F06 · ref: server/src/main/java/org/apache/kafka/network/Request.java updateRequestMetrics @4.4.0
Done when: Phases use Kafka's formulas (callback time counted as local, not remote); table tests with simulated timestamps including forwarded and no-op responses.

### [To Do] SRV-0066 Per-API phase histograms, request counts by version and request size
P0 · M2 · Task · deps: SRV-0065 · ref: server/src/main/java/org/apache/kafka/network/metrics/RequestMetrics.java @4.4.0
Done when: `alluvium_request_phase_seconds{api, phase}`, `alluvium_requests_total{api, version}` and `alluvium_request_bytes{api}` mirror `RequestQueueTimeMs`, `LocalTimeMs`, `RemoteTimeMs`, `ThrottleTimeMs`, `ResponseQueueTimeMs`, `ResponseSendTimeMs`, `TotalTimeMs`, `RequestsPerSec` and `RequestBytes`; tests.

### [To Do] SRV-0067 Split metric names like Kafka: FetchConsumer and FetchFollower, AddPartitionsToTxnVerification, ListClientMetricsResources
P0 · M2 · Task · deps: SRV-0066 · ref: server/src/main/java/org/apache/kafka/network/Request.java updateRequestMetrics (overrideMetricNames) @4.4.0
Done when: Fetch requests are recorded under both `Fetch` and `FetchConsumer` or `FetchFollower`; verify-only AddPartitionsToTxn under `AddPartitionsToTxnVerification`; ListConfigResources v0 also under `ListClientMetricsResources`; tests per case.

### [To Do] SRV-0068 Error counters by API and error code from each response's error counts
P0 · M2 · Task · deps: SRV-0066 · ref: core/src/main/scala/kafka/network/RequestChannel.scala updateErrorMetrics @4.4.0
Done when: `alluvium_request_errors_total{api, error_code}` counts every error in the response (including partition-level) like `ErrorsPerSec`; tests with mixed partition errors.

### [To Do] SRV-0069 Count requests at deprecated API versions by client software
P2 · M2 · Task · deps: SRV-0066 · ref: server/src/main/java/org/apache/kafka/network/metrics/RequestMetrics.java deprecatedRequestRate @4.4.0
Done when: `alluvium_requests_deprecated_total{api, version, client_software_name}` with bounded client labels mirrors `DeprecatedRequestsPerSec`; test with a deprecated version.

### [To Do] SRV-0070 End-to-end phase test: phases add up to total for acks=all produce and long-poll fetch
P0 · M2 · Task · deps: SRV-0066 · labels: test
Done when: Integration test shows remote time dominates for a long-poll fetch and acks=all produce, and the sum of phases equals total within 1 ms.

## [To Do] SRV-F09 Request logging
P1 · M2 · Feature · deps: SRV-F05
Scope: Debug-level per-request summary like `kafka.request.logger`.

### [To Do] SRV-0071 Per-request completion log at debug under `alluvium::request`
P1 · M2 · Task · deps: none · ref: server/src/main/java/org/apache/kafka/network/Request.java updateRequestMetrics (REQUEST_LOGGER) @4.4.0
Done when: One debug event `request_completed` per request carries api, version, correlation id, client id, principal, conn id, listener, protocol, each phase in ms, response size, throttle and error summary; nothing is formatted when the target is disabled; tests capture the event.

### [To Do] SRV-0072 Trace-level request and response summaries with payload and credential redaction
P1 · M2 · Task · deps: SRV-0071, TYPE-F09 · labels: security · ref: server/src/main/java/org/apache/kafka/network/RequestConvertToJson.java @4.4.0
Done when: At trace the request and response bodies are summarized like `RequestConvertToJson`, with records as length and CRC and SASL tokens, passwords and SCRAM data redacted; tests per payload-bearing API.

### [To Do] SRV-0073 Log requests at deprecated API versions at info
P1 · M2 · Task · deps: SRV-0071 · ref: server/src/main/java/org/apache/kafka/network/Request.java updateRequestMetrics (isApiVersionDeprecated) @4.4.0
Done when: A request at a deprecated version logs its completion line at info, as Kafka does, rate-limited per client and API; test.

### [To Do] SRV-0074 Request logging overhead benchmark with the target disabled and enabled
P2 · M2 · Task · deps: SRV-0071 · labels: perf
Done when: Bench shows negligible cost when disabled and records the enabled cost; result in the PR.

### [To Do] SRV-0075 Redaction tests for request logs
P1 · M2 · Task · deps: SRV-0072 · labels: security, test
Done when: A test runs produce, SASL and SCRAM admin requests carrying planted marker values with debug and trace enabled and finds none of the markers in the captured output.

### [To Do] SRV-0076 Document how to enable request logging at runtime
P2 · M2 · Task · deps: SRV-0071, OBS-F02 · labels: docs
Done when: Docs show enabling `alluvium::request` through `/debug/log-level` and the `BROKER_LOGGER` resource, and the event fields.

## [To Do] SRV-F10 Protocol-agnostic service layer
P0 · M2 · Feature · deps: SRV-F04 · labels: foundation-for:X-NATIVE
Scope: Services (produce, fetch, offsets, admin, groups) called by Kafka handlers and later by native APIs.

### [To Do] SRV-0077 Spike: service layer boundaries, error model, authorization and quota placement
P0 · M2 · Spike · deps: none · labels: foundation-for:X-NATIVE
Done when: ADR defines service traits per area, protocol-agnostic request and result types, service error enums, where authorization and quota recording happen so every front-end shares them, and deadline and cancellation semantics; accepted ADR.

### [To Do] SRV-0078 Protocol-agnostic `ServiceContext`: principal, client address, client id, deadline, span
P0 · M2 · Task · deps: SRV-0077 · labels: foundation-for:X-NATIVE
Done when: The context type lives in the service module with no Kafka wire types; unit tests.

### [To Do] SRV-0079 Service error enums with one exhaustive mapping to Kafka error codes
P0 · M2 · Task · deps: SRV-0077, ERR-F02 · labels: foundation-for:X-NATIVE
Done when: Each service's error enum maps to `ErrorCode` in one match without `_ =>`; tests per variant.

### [To Do] SRV-0080 Produce service: append batches with acks, timeout and producer identity
P0 · M2 · Task · deps: SRV-0078, SRV-0079, REPL-F03, REPL-F08 · labels: foundation-for:X-NATIVE · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleProduceRequest @4.4.0
Done when: `ProduceService::append` takes per-partition batches, acks and timeout and returns base offset, log append time, log start offset or error per partition; the Produce handler uses it; tests through the service directly.

### [To Do] SRV-0081 Fetch service: read by partition and offset with isolation, byte limits and long polling
P0 · M2 · Task · deps: SRV-0078, SRV-0079, REPL-F09, REPL-F13 · labels: foundation-for:X-NATIVE · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleFetchRequest @4.4.0
Done when: `FetchService::fetch` returns record slices, high watermark, last stable offset, log start offset and aborted transactions per partition; Kafka fetch sessions stay in the Kafka adapter; tests.

### [To Do] SRV-0082 Offset lookup service: earliest, latest, by timestamp and max timestamp
P0 · M2 · Task · deps: SRV-0078, SRV-0079, REPL-F14 · labels: foundation-for:X-NATIVE · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleListOffsetRequest @4.4.0
Done when: `OffsetService::lookup` resolves offset specs per partition with isolation level; ListOffsets uses it; tests.

### [To Do] SRV-0083 Metadata service: topics, partitions, leaders and endpoints from one image
P0 · M2 · Task · deps: SRV-0078, META-F09 · labels: foundation-for:X-NATIVE · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTopicMetadataRequest @4.4.0
Done when: `MetadataService` answers topic and partition queries from a single image version with endpoints for a requested listener; Metadata and DescribeTopicPartitions use it; tests.

### [To Do] SRV-0084 Admin service for topics and partitions: create, delete, add partitions, describe configs
P0 · M2 · Task · deps: SRV-0078, SRV-0079, CTRL-F21 · labels: foundation-for:X-NATIVE
Done when: `AdminService` submits create/delete topics and create partitions to the controller (with validate-only) and describes configs from the image; Kafka handlers use it; tests.

### [To Do] SRV-0085 Admin service config alteration
P0 · M7 · Task · deps: SRV-0084, CFG-F04 · labels: foundation-for:X-NATIVE
Done when: `AdminService` alters and incrementally alters configs through the controller with validate-only; AlterConfigs and IncrementalAlterConfigs use it; tests.

### [To Do] SRV-0086 Group service: commit and fetch offsets, list, describe and delete groups
P0 · M5 · Task · deps: SRV-0078, SRV-0079, GRP-F07, GRP-F08 · labels: foundation-for:X-NATIVE
Done when: `GroupService` exposes offset commit/fetch and group listing, description and deletion independent of group protocol messages; OffsetCommit, OffsetFetch and group admin handlers use it; tests.

### [To Do] SRV-0087 Services authorize through the SEC-F08 helper so every front-end shares Kafka ACL semantics
P0 · M4 · Task · deps: SRV-0080, SRV-0081, SRV-0082, SRV-0083, SRV-0084, SEC-F08 · labels: foundation-for:X-NATIVE, security
Done when: Each service operation checks the same operation and resource as Kafka's handler before acting; tests show a denied principal gets the same outcome through the service and through Kafka.

### [To Do] SRV-0088 Services record produce, fetch and request quota usage and return a throttle decision
P0 · M4 · Task · deps: SRV-0080, SRV-0081, QUOTA-F02, QUOTA-F03 · labels: foundation-for:X-NATIVE
Done when: Services report bytes and time to the quota managers and return the computed throttle; the Kafka adapter turns it into `throttle_time_ms` and muting; tests.

### [To Do] SRV-0089 Conformance tests: service calls and Kafka requests produce identical state and results
P0 · M2 · Task · deps: SRV-0080, SRV-0081, SRV-0082, SRV-0083, SRV-0084 · labels: foundation-for:X-NATIVE, test
Done when: For produce, fetch, list offsets, metadata and topic admin, scenario tests drive the services directly on an in-process node and assert results and resulting log state match the Kafka semantics cited in each service's `// Parity:` reference; the same scenarios are reused by the Kafka handler tests.

### [To Do] SRV-0090 Service layer design doc for front-end authors
P1 · M2 · Task · deps: SRV-0089 · labels: foundation-for:X-NATIVE, docs
Done when: docs page describes each service, its errors, authorization and quota behavior, and how a new front-end (X-NATIVE) plugs in; doc build passes.

## [To Do] SRV-F11 Log directory manager
P0 · M2 · Feature · deps: STOR-F01
Scope: `log.dirs`, directory ids, `meta.properties`, lock files, offline directory handling.

### [To Do] SRV-0091 Resolve `log.dirs`/`log.dir` and `metadata.log.dir` with Kafka's rules
P0 · M2 · Task · deps: CFG-F07 · ref: server-common/src/main/java/org/apache/kafka/metadata/properties/MetaPropertiesEnsemble.java Loader.addLogDirs, addMetadataLogDir @4.4.0
Done when: At least one log dir is required, duplicates fail, `metadata.log.dir` defaults to the first log dir and only one metadata dir is allowed; tests per rule.

### [To Do] SRV-0092 `meta.properties` codec: Java properties format, versions 0 and 1, cluster.id, node.id, directory.id
P0 · M2 · Task · deps: none · labels: decoder · ref: server-common/src/main/java/org/apache/kafka/metadata/properties/MetaProperties.java, server-common/src/main/java/org/apache/kafka/metadata/properties/PropertiesUtils.java @4.4.0
Done when: Parse and write Java properties syntax (comments, escapes, separators) and the v0/v1 field rules; files written by Kafka 4.4.0 parse identically (checked-in fixtures); invalid input returns typed errors, never panics.

### [To Do] SRV-0093 Fuzz target for the `meta.properties` parser
P0 · M2 · Task · deps: SRV-0092, TEST-F07 · labels: security, decoder, test
Done when: `fuzz/fuzz_targets/meta_properties.rs` runs clean for 10 minutes; corpus includes Kafka-written files.

### [To Do] SRV-0094 Create missing log dirs and reject unreadable or non-directory paths
P0 · M2 · Task · deps: SRV-0091 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java createAndValidateLogDirs @4.4.0
Done when: Missing dirs are created; a path that is not a readable directory becomes an offline (error) dir with the Kafka message; duplicate canonical paths fail startup; tests with the fault-injecting fs.

### [To Do] SRV-0095 Exclusive `.lock` file per log dir
P0 · M2 · Task · deps: SRV-0094 · labels: security · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java lockLogDirs, LOCK_FILE_NAME @4.4.0
Done when: Each dir is locked with an advisory exclusive lock through the fs trait; a second process on the same dir fails with Kafka's 'Failed to acquire lock on file .lock' message; locks release on shutdown; test with two nodes on one dir.

### [To Do] SRV-0096 Load the meta.properties ensemble: empty, error and formatted dirs
P0 · M2 · Task · deps: SRV-0092, SRV-0094 · ref: server-common/src/main/java/org/apache/kafka/metadata/properties/MetaPropertiesEnsemble.java Loader.load @4.4.0
Done when: Each dir is classified as empty (no file), error (unreadable or corrupt) or formatted with its properties; the result is an immutable ensemble; tests.

### [To Do] SRV-0097 Verify the ensemble: version 1, consistent cluster id, node id match, valid and unique directory ids, metadata dir present
P0 · M2 · Task · deps: SRV-0096 · ref: server-common/src/main/java/org/apache/kafka/metadata/properties/MetaPropertiesEnsemble.java verify @4.4.0
Done when: Each check of `MetaPropertiesEnsemble.verify` fails startup with Kafka's message; tests per rule including a reserved directory id.

### [To Do] SRV-0098 Generate missing directory ids that are not reserved and not duplicated
P0 · M2 · Task · deps: SRV-0096 · ref: server-common/src/main/java/org/apache/kafka/metadata/properties/MetaPropertiesEnsemble.java Copier.generateValidDirectoryId @4.4.0
Done when: Formatted dirs lacking `directory.id` get a random id from the injected RNG that avoids reserved ids and existing ones; tests with a fixed seed.

### [To Do] SRV-0099 Startup initialization: verify, refuse unformatted dirs, write updated meta.properties atomically
P0 · M2 · Task · deps: SRV-0097, SRV-0098, SRV-0095 · ref: core/src/main/scala/kafka/server/KafkaRaftServer.scala initializeLogDirs @4.4.0
Done when: Startup verifies the ensemble, fails if the metadata dir is missing or errored or no dir is formatted ('have you run format'), writes generated directory ids with tmp+fsync+rename+dir fsync, and returns online and offline dirs; tests.

### [To Do] SRV-0100 Crash-consistency test for meta.properties rewrites
P0 · M2 · Task · deps: SRV-0099, TEST-F03 · labels: test
Done when: With the fault-injecting fs, a crash at each write point leaves either the old or the new valid file; recovery test passes for all points.

### [To Do] SRV-0101 Log directory failure channel: bounded, deduplicated per dir, consumed by a supervised handler
P0 · M2 · Task · deps: SRV-0099, SRV-F02 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogDirFailureChannel.java maybeAddOfflineLogDir, takeNextOfflineLogDir @4.4.0
Done when: Storage I/O errors report a dir once (later reports for the same dir are ignored); a supervised handler consumes them; tests with concurrent reports.

### [To Do] SRV-0102 Fatal handling when the metadata dir fails or no log dir remains online
P0 · M2 · Task · deps: SRV-0101, SRV-0026 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogManager.java handleLogDirFailure @4.4.0
Done when: A failure of the metadata log dir or of the last online dir logs an error event and shuts the node down with the fatal-fault exit code; test with the fault-injecting fs.

### [To Do] SRV-0103 Log directory events and metrics
P0 · M2 · Task · deps: SRV-0101, OBS-F04 · labels: nfr
Done when: `log_dir_offline{dir, reason}` at error and `log_dir_loaded` at info; `alluvium_log_dirs_offline` maps `OfflineLogDirectoryCount`; tests.

### [To Do] SRV-0104 Keep serving other directories when one of several log dirs fails
P0 · M7 · Task · deps: SRV-0102, SRV-0123
Done when: A non-metadata dir failure marks it offline, tells storage to close its logs and reports it to the lifecycle client while other dirs keep serving; integration test with two dirs.

### [To Do] SRV-0105 `cordoned.log.dirs`: validate entries and report cordoned dirs after catch-up
P2 · M7 · Task · deps: SRV-0104 · ref: core/src/main/scala/kafka/server/KafkaConfig.scala cordonedLogDirs, server/src/main/java/org/apache/kafka/server/BrokerLifecycleManager.java BrokerHeartbeatResponseEvent @4.4.0
Done when: Entries must be in `log.dirs` or be the single `*` value (Kafka messages); cordoned directory ids are sent in heartbeats after catch-up; tests.

### [To Do] SRV-0106 Document the meta.properties format and log dir layout
P1 · M2 · Task · deps: SRV-0092 · labels: docs
Done when: docs/formats/meta-properties.md shows each field, version rules and a sample file; `.lock` and directory ids are described.

## [To Do] SRV-F12 Storage formatting and bootstrap
P0 · M2 · Feature · deps: SRV-F11, META-F01
Scope: `alluvium format` (like `kafka-storage.sh format`): cluster id, meta.properties, bootstrap metadata, SCRAM bootstrap, standalone and initial-controllers modes.

### [To Do] SRV-0107 Formatter input validation: node id, cluster id, directories, controller listener, metadata dir
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/metadata/storage/Formatter.java run @4.4.0
Done when: Each precondition of `Formatter.run` fails with Kafka's message (negative node id, missing cluster id, no directories, missing controller listener name for controllers, metadata dir not among directories); tests per rule.

### [To Do] SRV-0108 Resolve `--release-version` and `--feature` into finalized feature levels
P0 · M2 · Task · deps: SRV-0107, META-F06 · ref: metadata/src/main/java/org/apache/kafka/metadata/storage/Formatter.java run (feature level calculation) @4.4.0
Done when: Release version maps to `metadata.version` (default latest production); `--feature name=level` overrides; unknown features, unstable levels without `unstable.feature.versions.enable`, and dependency violations fail with Kafka's messages; tests.

### [To Do] SRV-0109 Write bootstrap metadata records as the initial metadata snapshot
P0 · M2 · Task · deps: SRV-0108 · ref: metadata/src/main/java/org/apache/kafka/metadata/storage/Formatter.java writeBoostrapSnapshot, metadata/src/main/java/org/apache/kafka/metadata/bootstrap/BootstrapMetadata.java @4.4.0
Done when: Feature-level records plus additional bootstrap records are written as the bootstrap snapshot in the metadata dir; the node loads them on first start; round-trip test.

### [To Do] SRV-0110 Write meta.properties to each empty dir; `--ignore-formatted` and already-formatted errors
P0 · M2 · Task · deps: SRV-0107, SRV-0092, SRV-0098 · ref: metadata/src/main/java/org/apache/kafka/metadata/storage/Formatter.java run @4.4.0
Done when: Each target dir gets version 1 meta.properties with cluster id, node id and a new directory id, written atomically; a formatted dir fails ('already formatted') unless ignore-formatted is set; no available dir fails; tests.

### [To Do] SRV-0111 Quorum bootstrap modes: standalone, initial controllers, no initial controllers
P0 · M2 · Task · deps: SRV-0109 · ref: core/src/main/scala/kafka/tools/StorageTool.scala, metadata/src/main/java/org/apache/kafka/metadata/storage/Formatter.java setInitialControllers @4.4.0
Done when: Exactly one of the three modes is accepted for controllers with a dynamic quorum and none with static voters; `--initial-controllers` entries `id@host:port:dirid` are validated and the chosen mode is handed to the Raft bootstrap checkpoint builder (RAFT-F15); tests per combination with StorageTool's messages.

### [To Do] SRV-0112 Bootstrap SCRAM credentials with `--add-scram`
P0 · M4 · Task · deps: SRV-0109, SEC-F04 · labels: security · ref: metadata/src/main/java/org/apache/kafka/metadata/storage/ScramParser.java @4.4.0
Done when: Arguments like `SCRAM-SHA-256=[name=alice,password=...]` or with salt, iterations and salted password parse into SCRAM credential records; metadata versions without SCRAM support fail with Kafka's message; passwords never logged; tests.

### [To Do] SRV-0113 Storage info report: formatted dirs, mismatches and problems
P2 · M2 · Task · deps: SRV-0097 · ref: core/src/main/scala/kafka/tools/StorageTool.scala @4.4.0
Done when: A report lists each dir's meta.properties and ensemble problems (mismatched cluster ids, missing dirs) for the CLI to print; tests.

### [To Do] SRV-0114 Differential test: Alluvium and kafka-storage.sh format equivalent directories
P0 · M2 · Task · deps: SRV-0110, SRV-0111, TEST-F09 · labels: test
Done when: For the same inputs the meta.properties fields and decoded bootstrap records match Kafka's (ignoring directory ids and timestamps); each mode covered.

### [To Do] SRV-0115 Document formatting modes and bootstrap options
P1 · M2 · Task · deps: SRV-0111 · labels: docs
Done when: Docs explain each flag, quorum mode and failure message with examples; doc build passes.

### [To Do] SRV-0148 Refuse to start on storage formatted by a newer release
P0 · M2 · Task · deps: none · ref: KIP-1170
Done when: startup reads the format and feature levels recorded in `meta.properties` and the bootstrap checkpoint and exits with a clear error naming both versions if they are newer than this build supports; tests cover equal, older and newer formats.

## [To Do] SRV-F13 Broker lifecycle states
P0 · M3 · Feature · deps: CTRL-F02
Scope: STARTING, RECOVERY, RUNNING, PENDING_CONTROLLED_SHUTDOWN, SHUTTING_DOWN; fencing behavior.

### [To Do] SRV-0116 `BrokerState` enum with Kafka's wire values and the BrokerState metric
P0 · M3 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/metadata/BrokerState.java @4.4.0
Done when: NOT_RUNNING 0, STARTING 1, RECOVERY 2, RUNNING 3, PENDING_CONTROLLED_SHUTDOWN 6, SHUTTING_DOWN 7, UNKNOWN 127; `alluvium_broker_state` maps `kafka.server:type=KafkaServer,name=BrokerState`; tests.

### [To Do] SRV-0117 Heartbeat-driven state transitions with events and flight recorder
P0 · M3 · Task · deps: SRV-0116 · ref: server/src/main/java/org/apache/kafka/server/BrokerLifecycleManager.java BrokerHeartbeatResponseEvent @4.4.0
Done when: STARTING to RECOVERY on caught-up, RECOVERY to RUNNING when unfenced, RUNNING to PENDING_CONTROLLED_SHUTDOWN on controlled shutdown, SHUTTING_DOWN when allowed or stopping; responses in SHUTTING_DOWN are ignored; each transition emits `broker_state_changed{old, new, reason}` and is recorded; exhaustive tests.

### [To Do] SRV-0118 Heartbeat scheduling rules
P0 · M3 · Task · deps: SRV-0117 · ref: server/src/main/java/org/apache/kafka/server/BrokerLifecycleManager.java scheduleNextCommunication @4.4.0
Done when: 10 ms between heartbeats while STARTING, immediate after successful registration or state changes, the configured interval after success or failure (never immediate after failure); simulated-clock tests.

### [To Do] SRV-0119 Registration and heartbeat error handling
P0 · M3 · Task · deps: SRV-0117 · ref: server/src/main/java/org/apache/kafka/server/BrokerLifecycleManager.java BrokerRegistrationResponseEvent, RegistrationTimeoutEvent @4.4.0
Done when: Timeouts and NOT_CONTROLLER retry at the interval; registration errors (DUPLICATE_BROKER_REGISTRATION, INCONSISTENT_CLUSTER_ID, UNSUPPORTED_VERSION for feature ranges) and STALE_BROKER_EPOCH follow Kafka's handling and logging; registration timeout shuts down; tests per error.

### [To Do] SRV-0120 Re-register when the controller's registration for this broker diverges
P0 · M3 · Task · deps: SRV-0119 · ref: metadata/src/main/java/org/apache/kafka/image/publisher/BrokerRegistrationTracker.java @4.4.0
Done when: When metadata shows no registration or a different incarnation or feature set for this broker, it re-sends registration; tests with crafted images.

### [To Do] SRV-0121 Simulation: fencing on lost heartbeats and unfencing on reconnection
P0 · M3 · Task · deps: SRV-0120, TEST-F06, TEST-F04 · labels: test
Done when: A broker partitioned from the controller for longer than the session timeout is fenced, leaderships move, and after healing it returns to RUNNING; passes across 1000 seeds.

### [To Do] SRV-0122 Readiness follows broker state
P0 · M3 · Task · deps: SRV-0117 · labels: nfr
Done when: The broker reports ready only in RUNNING; RECOVERY and PENDING_CONTROLLED_SHUTDOWN report not ready with the state as reason through the readiness interface; test.

### [To Do] SRV-0123 Report offline log dirs in heartbeats and shut down if not acknowledged within `log.dir.failure.timeout.ms`
P0 · M7 · Task · deps: SRV-0117, SRV-0101 · ref: server/src/main/java/org/apache/kafka/server/BrokerLifecycleManager.java propagateDirectoryFailure, OfflineDirBrokerFailureEvent @4.4.0
Done when: Offline dirs are sent in `offlineLogDirs` until acknowledged; if not acknowledged in time the broker shuts down with an error log, as Kafka does; tests.

### [To Do] SRV-0124 Document broker states and transitions
P1 · M3 · Task · deps: SRV-0117 · labels: docs
Done when: Docs show the state diagram, what triggers each transition and how to see it (metric, events, `/debug`); doc build passes.

## [To Do] SRV-F14 Metadata publishers
P0 · M2 · Feature · deps: META-F04
Scope: Apply image deltas to replica manager, coordinators, ACL cache, configs and quotas in a defined order.

### [To Do] SRV-0125 Publisher interface and registration with an installation signal
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/image/publisher/MetadataPublisher.java @4.4.0
Done when: A publisher has a name and receives (delta, image, manifest) and controller-change notifications; registration returns when the publisher has been installed and caught up to the current image; tests.

### [To Do] SRV-0126 Atomic image swap so each request sees one image version
P0 · M2 · Task · deps: SRV-0125
Done when: The current image is published through an atomic shared pointer; handlers capture it once per request; a test interleaving publishes with requests never observes two versions in one request.

### [To Do] SRV-0127 Broker publisher order: metadata cache, managers, topic deltas, coordinators, configs, quotas, SCRAM, tokens, ACLs
P0 · M2 · Task · deps: SRV-0125 · ref: core/src/main/scala/kafka/server/metadata/BrokerMetadataPublisher.scala onMetadataUpdate @4.4.0
Done when: Updates apply in `BrokerMetadataPublisher` order: metadata cache, then (first publish) manager initialization, replica manager topic deltas, group, transaction and share coordinators, dynamic configs, client quotas, topic and cluster quotas, SCRAM, delegation tokens, ACLs; a recording test asserts the order.

### [To Do] SRV-0128 First publish: initialize managers once and signal first publish
P0 · M2 · Task · deps: SRV-0127, SRV-0027 · ref: core/src/main/scala/kafka/server/metadata/BrokerMetadataPublisher.scala initializeManagers @4.4.0
Done when: The first image starts the log manager, replica manager and coordinators in Kafka's order, then completes the first-publish signal used by startup; a failure goes to the fatal initial-load fault handler; tests.

### [To Do] SRV-0129 Controller-side publishers and order
P0 · M2 · Task · deps: SRV-0125 · ref: core/src/main/scala/kafka/server/ControllerServer.scala startup (metadata publishers) @4.4.0
Done when: Controllers install metadata cache, features, controller registrations, dynamic config, client quotas, SCRAM, delegation tokens and ACL publishers in Kafka's order; recording test.

### [To Do] SRV-0130 Publishing faults: report, count and continue
P0 · M2 · Task · deps: SRV-0127, SRV-0027 · ref: core/src/main/scala/kafka/server/SharedServer.scala metadataPublishingFaultHandler @4.4.0
Done when: An error in a publisher goes to the non-fatal publishing fault handler with the publisher name and offset, increments `metadata-apply-error-count`, and later updates are still applied; test with a failing publisher.

### [To Do] SRV-0131 Publishing metrics: last applied offset and timestamp, apply lag, per-publisher latency
P0 · M2 · Task · deps: SRV-0127, OBS-F04 · labels: nfr · ref: metadata/src/main/java/org/apache/kafka/image/loader/metrics/MetadataLoaderMetrics.java @4.4.0
Done when: `alluvium_metadata_last_applied_offset`, `..._last_applied_timestamp_seconds`, `..._apply_lag_seconds` and `alluvium_metadata_publish_seconds{publisher}` map Kafka's `last-applied-record-*` metrics; tests.

### [To Do] SRV-0132 Publish events and flight recorder entries
P0 · M2 · Task · deps: SRV-0130, OBS-F06 · labels: nfr
Done when: `metadata_published{offset, epoch, elapsed_ms}` at debug and a metadata flight recorder entry per publish, dumped on publishing faults; tests.

## [To Do] SRV-F15 Input journaling
P2 · M3 · Feature · deps: SRV-F02
Scope: Optional journaling of state-machine inputs (metadata only) and a replay tool.

### [To Do] SRV-0133 Spike: input journal format, scope, bounds and privacy
P2 · M3 · Spike · deps: none · labels: nfr
Done when: ADR defines the versioned, CRC-protected entry format, which driver inputs are journaled (metadata only, no payloads or secrets), rotation and size bounds, and the off-by-default switch; accepted ADR.

### [To Do] SRV-0134 Journal entry codec and fuzz target
P2 · M3 · Task · deps: SRV-0133, TEST-F07 · labels: decoder, test
Done when: Entries encode and decode per the ADR; decoding untrusted files never panics; fuzz target runs clean for 10 minutes.

### [To Do] SRV-0135 Bounded journal writer with rotation; never blocks a state machine
P2 · M3 · Task · deps: SRV-0134, CFG-F11
Done when: A per-driver writer task receives entries through a bounded channel; when full, entries are dropped and counted (`alluvium_journal_dropped_total`) and a gap marker is written; files rotate by size and count; enabled only by `alluvium.journal.enable`; tests.

### [To Do] SRV-0136 Driver journal API: typed per-component input records, zero cost when disabled
P2 · M3 · Task · deps: SRV-0135
Done when: Drivers register a component and record typed inputs (timer ticks with clock values, peer message metadata, client requests without payloads); with journaling off, recording compiles to a disabled branch with no allocation (benchmark); component hooks are owned by their epics (Raft has RAFT-0124); tests with a test state machine.

### [To Do] SRV-0137 Replay tool: rebuild a registered state machine from a journal and report its final state
P2 · M3 · Task · deps: SRV-0136, CLI-F01
Done when: `alluvium replay <journal> --component <name>` feeds recorded inputs into the component's registered replay entry point and prints the final state hash and flight recorder; gaps are reported, not hidden; tests with the testkit state machine.

### [To Do] SRV-0138 Record-and-replay test: a simulated run replays to identical state
P2 · M3 · Task · deps: SRV-0137, TEST-F04 · labels: test
Done when: A seeded simulation of a testkit state machine with journaling on is replayed and yields the same state hash and flight recorder.

### [To Do] SRV-0139 Journals contain no payloads or secrets
P2 · M3 · Task · deps: SRV-0136 · labels: security, test
Done when: A test journals inputs that carry planted payload, password and token markers and finds none of the markers in the journal files.

### [To Do] SRV-0140 Document journaling and replay in the debugging guide
P2 · M3 · Task · deps: SRV-0137 · labels: docs
Done when: docs/guides/debugging.md explains enabling the journal, its cost, and replay steps.

## [To Do] SRV-F16 Panic hook and crash reports
P0 · M2 · Feature · deps: OBS-F06
Scope: Backtrace, span stack and flight recorder dump before abort.

### [To Do] SRV-0141 Panic hook: message, location, task name and forced backtrace logged as one error event before abort
P0 · M2 · Task · deps: none · ref: ADR-0004
Done when: The binary installs a hook that writes one JSON error event with message, location, thread and supervised task name and a captured backtrace, then aborts; test runs a child process that panics.

### [To Do] SRV-0142 Include the active span stack in the crash report
P0 · M2 · Task · deps: SRV-0141
Done when: The report lists the panicking task's span stack with fields (for example connection, request and partition); test asserts span fields from a panic inside a request.

### [To Do] SRV-0143 Dump registered flight recorders on panic with bounded output
P0 · M2 · Task · deps: SRV-0141
Done when: All component flight recorders (or the ones linked to the panicking task first) are written up to a size cap; recorders are read with try-lock so a held lock cannot deadlock the hook; test.

### [To Do] SRV-0144 Write a crash report file next to the logs
P0 · M2 · Task · deps: SRV-0142, SRV-0143
Done when: A bounded JSON crash report is written to the configured log directory with owner-only permissions and picked up by `debug-bundle`; test checks the file and permissions.

### [To Do] SRV-0145 Nested panics and failures inside the hook still abort promptly
P0 · M2 · Task · deps: SRV-0144
Done when: A panic while dumping, or an I/O error writing the report, aborts without hanging; test forces both.

### [To Do] SRV-0146 Crash reports contain no secrets or payloads
P0 · M2 · Task · deps: SRV-0144 · labels: security, test
Done when: Test panics while secrets and payload buffers with planted markers are in scope and finds no marker in the report or log.

### [To Do] SRV-0147 Document how to read a crash report
P1 · M2 · Task · deps: SRV-0144 · labels: docs
Done when: docs/guides/debugging.md shows a sample report and how to map it to code and flight recorders.
