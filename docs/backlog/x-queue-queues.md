# X-QUEUE: Queue semantics

> Crates: `alluvium-queue` (new, layer 2) on `SHARE-F08` · Kafka counterpart: none (Alluvium enhancement) · Milestones: M11

Full message-queue semantics on top of the log: per-message acknowledgement, visibility timeouts, redelivery with backoff, dead-letter queues, delayed delivery, TTL, priorities and FIFO groups.

## [To Do] X-QUEUE-F01 Queue model design
P0 · M11 · Feature · deps: SHARE-F08
Scope: Spike and ADR: queue semantics on the per-record state store; exposure to Kafka clients.

### [To Do] X-QUEUE-0001 Queue model design spike: queue semantics on the SHARE-F08 per-record state store, record states, persistence, delay index, priority lanes, FIFO groups, deduplication and exposure to Kafka share consumers
P0 · M11 · Spike · deps: none · labels: docs · ref: KIP-932
Done when: an ADR "Queue semantics" is accepted defining: a queue as a topic with `alluvium.queue.enable=true` consumed through named queue groups (share groups with queue extensions); record states (AVAILABLE, DELAYED, ACQUIRED, ACKNOWLEDGED, ARCHIVING, ARCHIVED) and transitions; at-least-once guarantees and their limits (no exactly-once delivery); how extended state is persisted; the delay-index, priority-lane, FIFO and dedup designs with memory bounds; alignment with Kafka 4.4.0 share-group DLQ (KIP-1191) and RENEW acknowledgements; rejected alternatives.

### [To Do] X-QUEUE-0002 Queue invariants specification docs/queues/invariants.md: at-least-once, single live lease per record, attempts bounded, never delivered before deliver-at or after expiry, per-group FIFO, dedup within the window
P0 · M11 · Task · deps: X-QUEUE-0001 · labels: docs, test · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: each invariant is stated formally with the checker task that enforces it; the doc is linked from the ADR and the testkit checker list.

### [To Do] X-QUEUE-0003 Queue header contract under the REC-F13 reserved prefix: `alluvium.deliver.at`, `alluvium.delay.ms`, `alluvium.ttl.ms`, `alluvium.priority`, `alluvium.message.group`, `alluvium.message.id`
P0 · M11 · Task · deps: X-QUEUE-0001, REC-F13 · ref: REC-F13
Done when: each header has a type, byte encoding, valid range and precedence rule versus native API fields, documented in docs/queues/headers.md; the parser lives in `alluvium-queue` with typed errors.

### [To Do] X-QUEUE-0004 Threat model rows for queues: delay-index flooding, unbounded FIFO groups, dedup-id flooding, DLQ amplification, redrive loops, peek data exposure
P0 · M11 · Task · deps: X-QUEUE-0001 · labels: security, docs · ref: docs/standards/security.md
Done when: docs/standards/security.md has one row per threat with its bound and implementing task id.

## [To Do] X-QUEUE-F02 Queue topics
P0 · M11 · Feature · deps: X-QUEUE-F01
Scope: Queue topic type or queue view over a topic; configuration.

### [To Do] X-QUEUE-0005 `alluvium-queue` crate (layer 2): `QueueConfig` with cross-field validation (backoff max at least initial, attempts at least 1, maximum delay below retention)
P0 · M11 · Task · deps: none · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: the crate is registered in the layer table with a `//!` doc; validation errors name both conflicting keys; unit tests cover every rule.

### [To Do] X-QUEUE-0006 Topic config keys `alluvium.queue.*` (enable, visibility timeout and maximum, max delivery attempts, backoff policy keys, DLQ keys, maximum delay, message TTL, priority levels and weights, FIFO, dedup window) registered with defaults, docs and dynamic-update rules
P0 · M11 · Task · deps: X-QUEUE-0005, CFG-F11, CFG-F05 · ref: CFG-F11
Done when: `alluvium.queue.enable` defaults to false; every key appears in the generated config reference; keys are hidden from Kafka DescribeConfigs unless set (CFG-F11 rule); invalid combinations are rejected by CreateTopics and IncrementalAlterConfigs.

### [To Do] X-QUEUE-0007 Extended per-record state on the SHARE-F08 store: attempts, visible-at, deliver-at, expires-at, priority lane, FIFO group hash, last failure reason code
P0 · M11 · Task · deps: X-QUEUE-0005 · labels: perf · ref: server/src/main/java/org/apache/kafka/server/share/fetch/InFlightState.java @4.4.0
Done when: state stays a fixed-size struct with no heap per record; memory per in-flight record is measured and documented; Kafka share-group state is unchanged when queue features are off.

### [To Do] X-QUEUE-0008 Queue partition state machine (sans-I/O): inputs Acquire, Ack, Nack, ExtendVisibility, Tick, RecordsAppended, LeaderChange; outputs deliveries, persistence writes, DLQ writes and timer requests
P0 · M11 · Task · deps: X-QUEUE-0007 · ref: core/src/main/java/kafka/server/share/SharePartition.java @4.4.0
Done when: transitions follow the ADR table; every transition returns a reason enum; property tests drive arbitrary input sequences without panics; no clock reads or I/O.

### [To Do] X-QUEUE-0009 Queue state persistence: a versioned extension of share-group state written through the share coordinator; replay restores attempts and deadlines
P0 · M11 · Task · deps: X-QUEUE-0008, SHARE-F05, TEST-F07 · labels: decoder · ref: server-common/src/main/java/org/apache/kafka/server/share/persister/PersisterStateBatch.java @4.4.0
Done when: Kafka-format share state records are byte-identical when no queue extension is present; the extension decoder is bounded and has a fuzz target; replay round-trips under proptest.

### [To Do] X-QUEUE-0010 Queue partition driver: owns one state machine per (queue group, partition), arms timers through the SRV-F07 timer wheel, records transitions in a flight recorder
P0 · M11 · Task · deps: X-QUEUE-0008, SRV-F07, OBS-F06 · ref: SRV-F07
Done when: timers fire in simulation with the simulated clock; the driver is a named supervised task; the flight recorder shows the last 256 transitions per queue partition.

### [To Do] X-QUEUE-0011 Enabling and disabling queue mode on an existing topic, with `queue_mode_changed` events
P0 · M11 · Task · deps: X-QUEUE-0006, X-QUEUE-0009 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: enabling on a topic with existing share groups converts their state lazily; disabling is refused while extended state exists unless `alluvium.queue.disable.force=true`, which archives queue-only state with an event; tests cover both paths.

### [To Do] X-QUEUE-0012 Queue groups: a queue is consumed through named groups, each with independent delivery state (fan-out to several queues from one topic)
P0 · M11 · Task · deps: X-QUEUE-0008 · ref: KIP-932
Done when: two groups on one queue topic each receive every message; acknowledging in one group does not affect the other; group creation uses share-group membership (SHARE-F01).

### [To Do] X-QUEUE-0013 Leadership change: the new leader loads queue state and re-arms visibility, backoff and delay timers from persisted deadlines; leases from the old leader expire
P0 · M11 · Task · deps: X-QUEUE-0009, X-QUEUE-0010 · ref: core/src/main/java/kafka/server/share/SharePartition.java @4.4.0
Done when: a simulation moves leadership with in-flight leases; no record is lost and no record has two live leases after the move.

### [To Do] X-QUEUE-0014 Crash-consistency test: a crash between acquisition and persistence never loses an unacknowledged message and never decrements attempts
P0 · M11 · Task · deps: X-QUEUE-0009, TEST-F03 · labels: test · ref: STOR-F20
Done when: the fault-injecting fs crashes at every write point of the persistence path; recovery satisfies the invariants in docs/queues/invariants.md.

## [To Do] X-QUEUE-F03 Per-message acknowledgement
P0 · M11 · Feature · deps: X-QUEUE-F02, X-NATIVE-F06
Scope: Ack, nack and visibility-timeout extension over the native API.

### [To Do] X-QUEUE-0015 `QueueService` in `queue.proto`: Receive, ReceiveStream, Ack, Nack and ExtendVisibility with HTTP mappings under `/v1/queues/{topic}/groups/{group}`
P0 · M11 · Task · deps: X-NATIVE-F02 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: messages carry receipt handles, delivery attempt, visibility deadline and record data; lint and codegen pass; OpenAPI regenerates.

### [To Do] X-QUEUE-0016 Receipt handle v1: opaque base64url of (topic id, partition, offset, group, delivery attempt, lease epoch), validated against live state
P0 · M11 · Task · deps: none · labels: decoder, security · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: forged or stale handles cannot affect another lease (test); decode errors are typed; the fuzz target runs per PR.

### [To Do] X-QUEUE-0017 `Receive` RPC: long-poll up to `max_messages` (at most `alluvium.queue.receive.max.messages`), visibility override within the maximum, fair across partitions
P0 · M11 · Task · deps: X-QUEUE-0015, X-QUEUE-0016 · ref: core/src/main/java/kafka/server/share/SharePartition.java acquire @4.4.0
Done when: READ on the topic and the group is required; an empty queue waits up to `wait_ms` then returns empty; fairness test shows no partition starved; each delivery carries its attempt number.

### [To Do] X-QUEUE-0018 `Ack` RPC: batch acknowledgement by receipt handles with per-handle results
P0 · M11 · Task · deps: X-QUEUE-0017 · ref: clients/src/main/java/org/apache/kafka/clients/consumer/AcknowledgeType.java ACCEPT @4.4.0
Done when: acknowledging twice is idempotent; a handle whose lease expired returns RECEIPT_HANDLE_EXPIRED for that handle only; acknowledged records are never delivered again (property test).

### [To Do] X-QUEUE-0019 `Nack` RPC: release with an optional delay (bounded) and a reason code and message (truncated to 1 KiB) kept for DLQ headers
P0 · M11 · Task · deps: X-QUEUE-0017 · ref: clients/src/main/java/org/apache/kafka/clients/consumer/AcknowledgeType.java RELEASE @4.4.0
Done when: a nack without delay applies the redelivery policy; a nack with delay overrides it within bounds; the reason is persisted with the record state.

### [To Do] X-QUEUE-0020 Visibility timeout extend rule: `ExtendVisibility` sets the deadline to now plus the requested timeout, capped so the total lease since acquisition stays within `alluvium.queue.visibility.timeout.max.ms`; only the current lease holder may extend
P0 · M11 · Task · deps: X-QUEUE-0017 · ref: clients/src/main/java/org/apache/kafka/clients/consumer/AcknowledgeType.java RENEW @4.4.0
Done when: property test over random extend sequences never exceeds the cap; extending an expired or foreign lease fails per handle.

### [To Do] X-QUEUE-0021 Visibility timeout expire rule: at the deadline the record leaves ACQUIRED (to AVAILABLE, backoff or DLQ), the lease epoch increments and late acknowledgements are rejected
P0 · M11 · Task · deps: X-QUEUE-0017 · ref: server/src/main/java/org/apache/kafka/server/share/fetch/AcquisitionLockTimeoutHandler.java @4.4.0
Done when: simulation with stalled consumers shows every expired record redelivered exactly once per expiry; a late Ack returns RECEIPT_HANDLE_EXPIRED; a `visibility_expired` counter increments.

### [To Do] X-QUEUE-0022 Redelivery count rule: the attempt counter increments at acquisition and is persisted before delivery; deliveries expose `delivery_attempt`
P0 · M11 · Task · deps: X-QUEUE-0017 · ref: server/src/main/java/org/apache/kafka/server/share/fetch/DeliveryCountOps.java @4.4.0
Done when: property test: attempts never exceed `alluvium.queue.max.delivery.attempts`; a crash after acquisition does not reset the count (test).

### [To Do] X-QUEUE-0023 `ReceiveStream` RPC: server stream of deliveries with credit; acknowledgements by unary Ack or in-stream ack frames
P1 · M11 · Task · deps: X-QUEUE-0018 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: deliveries never exceed credit; stream close releases nothing until visibility expiry (documented and tested); stall protection from X-NATIVE applies.

### [To Do] X-QUEUE-0024 In-flight limits: per-partition lease limit (share-partition max record locks) and per-receiver unacknowledged maximum, with backpressure
P0 · M11 · Task · deps: X-QUEUE-0017 · labels: nfr · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java SHARE_PARTITION_MAX_RECORD_LOCKS_CONFIG @4.4.0
Done when: Receive returns fewer messages when limits are reached; memory stays bounded under 1M available messages (test); limits are exported as gauges.

### [To Do] X-QUEUE-0025 State machine property test: random interleavings of Receive, Ack, Nack, ExtendVisibility and Tick never deliver an acknowledged record and never hold two live leases on one offset
P0 · M11 · Task · deps: X-QUEUE-0018, X-QUEUE-0019, X-QUEUE-0020, X-QUEUE-0021, X-QUEUE-0022 · labels: test · ref: TEST-F16
Done when: 10 000 proptest cases pass against a reference model; regressions are persisted.

## [To Do] X-QUEUE-F04 Redelivery with backoff
P0 · M11 · Feature · deps: X-QUEUE-F03
Scope: Fixed and exponential redelivery delays per attempt.

### [To Do] X-QUEUE-0026 Backoff policy model: none, fixed and exponential (initial, multiplier, maximum) with a jitter fraction, as a pure function of the attempt number
P0 · M11 · Task · deps: none · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: unit tests cover each policy; arithmetic is checked (no overflow for attempt 2^15 or multiplier 10); invalid configs are rejected by QueueConfig.

### [To Do] X-QUEUE-0027 Fixed backoff rule: after a failed attempt (nack without delay or visibility expiry) the record is not acquirable before now plus `alluvium.queue.redelivery.backoff.ms`
P0 · M11 · Task · deps: X-QUEUE-0026 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: property test with a manual clock shows no delivery inside the backoff and delivery within one tick after it.

### [To Do] X-QUEUE-0028 Exponential backoff rule: delay equals the minimum of initial times multiplier to the power of (attempt minus one) and the maximum
P0 · M11 · Task · deps: X-QUEUE-0026 · ref: KIP-580
Done when: property test: delays are non-decreasing in attempt and never exceed the maximum; golden table for attempts 1-10.

### [To Do] X-QUEUE-0029 Jitter: a fraction drawn from the injected seeded RNG, deterministic in simulation
P1 · M11 · Task · deps: X-QUEUE-0026, TYPE-F07 · ref: KIP-580
Done when: the same seed reproduces identical schedules; a distribution test shows delays within the configured band.

### [To Do] X-QUEUE-0030 Per-nack delay override: an explicit Nack delay replaces the policy delay, capped at `alluvium.queue.redelivery.backoff.max.ms`
P0 · M11 · Task · deps: X-QUEUE-0026 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: tests cover zero delay (immediate), within-cap and over-cap requests.

### [To Do] X-QUEUE-0031 Backoff deadlines persisted as visible-at so restarts and leader changes keep the remaining delay
P0 · M11 · Task · deps: X-QUEUE-0027, X-QUEUE-0028, TEST-F03 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: crash and failover tests show no early redelivery after recovery.

### [To Do] X-QUEUE-0032 Backoff and window accounting: backed-off records count towards in-flight limits; behaviour when the window is full
P0 · M11 · Task · deps: X-QUEUE-0027 · labels: nfr · ref: core/src/main/java/kafka/server/share/SharePartition.java @4.4.0
Done when: with the window full of backed-off records, Receive returns empty without busy-looping (CPU-bound test); a gauge exposes backed-off records.

### [To Do] X-QUEUE-0033 Simulation: no redelivery earlier than the computed backoff under clock jumps, crashes and failover
P0 · M11 · Task · deps: X-QUEUE-0031, TEST-F06 · labels: test · ref: TEST-F06
Done when: 1000 seeds pass nightly with the deliver-not-before checker.

## [To Do] X-QUEUE-F05 Dead-letter queues
P0 · M11 · Feature · deps: X-QUEUE-F03, SHARE-F11
Scope: Move after max attempts with failure headers; redrive.

### [To Do] X-QUEUE-0034 DLQ configuration: `alluvium.queue.dlq.topic` (must not start with `__`, must differ from the queue), `alluvium.queue.dlq.copy.record`, and precedence with the KIP-1191 group config `errors.deadletterqueue.topic.name`
P0 · M11 · Task · deps: SHARE-F07 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java ERRORS_DEADLETTERQUEUE_TOPIC_NAME_CONFIG @4.4.0
Done when: validation errors name the key; the precedence rule (group config wins when set, as the more specific scope) is documented and tested.

### [To Do] X-QUEUE-0035 DLQ triggers: attempts exhausted, explicit reject, and TTL expiry when `alluvium.queue.dlq.on.expiry=true`; the record enters ARCHIVING
P0 · M11 · Task · deps: X-QUEUE-0034 · ref: KIP-1191
Done when: tests drive each trigger; without a DLQ topic records go straight to ARCHIVED, as in Kafka.

### [To Do] X-QUEUE-0036 DLQ write path: the broker produces to the DLQ topic as an internal principal with acks=all and idempotence; ARCHIVING becomes ARCHIVED only after acknowledgement; retries with backoff
P0 · M11 · Task · deps: X-QUEUE-0035 · ref: server/src/main/java/org/apache/kafka/server/share/dlq/ShareGroupDLQStateManager.java @4.4.0
Done when: a broker crash during the write leaves the record in ARCHIVING and retries after recovery (test); duplicates in the DLQ carry identical original coordinates.

### [To Do] X-QUEUE-0037 DLQ headers: Kafka 4.4.0 `__dlq.errors.topic`, `.partition`, `.offset`, `.group`, `.delivery.count` and `.message`, plus `alluvium.dlq.reason`, `alluvium.dlq.first.attempt.ts`, `alluvium.dlq.last.attempt.ts` and `alluvium.dlq.last.error`
P0 · M11 · Task · deps: X-QUEUE-0036 · ref: server/src/main/java/org/apache/kafka/server/share/dlq/ShareGroupDLQStateManager.java HEADER_DLQ_ERRORS_TOPIC @4.4.0
Done when: golden tests check every header; original key, headers and timestamp are preserved when copying the record; `alluvium.dlq.last.error` is truncated to 1 KiB.

### [To Do] X-QUEUE-0038 DLQ outage handling: a missing or unavailable DLQ topic keeps records in ARCHIVING (bounded), with a metric and `queue_dlq_write_failed` events with reason
P0 · M11 · Task · deps: X-QUEUE-0036 · labels: nfr · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: deleting the DLQ topic during a test loses no record; ARCHIVING records count towards in-flight limits; recovery drains them once the topic returns.

### [To Do] X-QUEUE-0039 Redrive job model: source DLQ, target queue, filter (offset range, time range, reason, maximum messages), rate limit, persisted progress
P0 · M11 · Task · deps: X-QUEUE-0036 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: job state is persisted so a restarted broker resumes from the last committed source offset; at most `alluvium.queue.redrive.max.jobs` jobs run per broker.

### [To Do] X-QUEUE-0040 Redrive execution: copy matching DLQ records to the target with `alluvium.redrive.count` incremented and prior `__dlq.errors.*` headers kept under `alluvium.redrive.prior.*`
P0 · M11 · Task · deps: X-QUEUE-0039 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: at-least-once copy with resumable progress (crash test); the rate limit holds within 10% (simulated clock); the source DLQ is not modified.

### [To Do] X-QUEUE-0041 Redrive loop guard: records with `alluvium.redrive.count` at or above `alluvium.queue.redrive.max.count` are skipped and reported
P0 · M11 · Task · deps: X-QUEUE-0040 · labels: security · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: a DLQ-to-queue-to-DLQ loop stops after the configured count (test); skipped counts appear in job status.

### [To Do] X-QUEUE-0042 DLQ authorization: setting `alluvium.queue.dlq.topic` requires WRITE on the DLQ topic for the principal altering the config; redrive requires READ on the DLQ and WRITE on the target
P0 · M11 · Task · deps: X-QUEUE-0034, SEC-F08 · labels: security · ref: KIP-1191
Done when: tests cover allowed and denied config changes and redrive starts.

### [To Do] X-QUEUE-0043 Simulation: every message is eventually acknowledged or in the DLQ, never both available and dead-lettered; DLQ duplicates carry identical original coordinates
P0 · M11 · Task · deps: X-QUEUE-0037, X-QUEUE-0038, X-QUEUE-0040, TEST-F06 · labels: test · ref: TEST-F06
Done when: 1000 seeds pass nightly with DLQ-topic faults injected.

## [To Do] X-QUEUE-F06 Delayed delivery
P0 · M11 · Feature · deps: X-QUEUE-F02
Scope: Deliver-at timestamps, persisted delay index, timer wheel.

### [To Do] X-QUEUE-0044 Deliver-at rule: `alluvium.deliver.at` (epoch ms) and `alluvium.delay.ms` headers or native fields parsed at append against broker append time; past times deliver immediately; beyond `alluvium.queue.delay.max.ms` rejected with INVALID_RECORD
P0 · M11 · Task · deps: REC-F12 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: tests cover each case and both headers present (deliver-at wins); rejected records carry record errors with the reason.

### [To Do] X-QUEUE-0045 Delay index format: per-partition entries (deliver-at, offset, queue group) in a versioned, CRC-protected append-only file with snapshots
P0 · M11 · Task · deps: TEST-F07 · labels: decoder · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: the decoder is bounded with typed errors and a fuzz target; the format is documented in docs/formats/queue-delay-index.md.

### [To Do] X-QUEUE-0046 Hierarchical timer wheel (sans-I/O, injected clock) for near-term deliveries with bounded memory; far-future entries stay only in the persisted index
P0 · M11 · Task · deps: TYPE-F06 · labels: perf · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: memory is bounded by `alluvium.queue.delay.memory.max.entries` regardless of delayed message count; property test: entries fire in deliver-at order.

### [To Do] X-QUEUE-0047 Deferred records do not hold the share-partition window: records with a future deliver-at move to the delay index and are skipped when the start offset advances
P0 · M11 · Task · deps: X-QUEUE-0045 · ref: core/src/main/java/kafka/server/share/SharePartition.java @4.4.0
Done when: a test with 1M delayed records followed by immediate records delivers the immediate ones without waiting; the window size stays within limits.

### [To Do] X-QUEUE-0048 Re-injection: at deliver-at the record becomes AVAILABLE; records with the same deliver-at are delivered in offset order
P0 · M11 · Task · deps: X-QUEUE-0047, X-QUEUE-0046 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: property test checks ordering; re-injected records then follow normal ack, backoff and DLQ rules.

### [To Do] X-QUEUE-0049 Delay index persistence and recovery with crash-consistency tests at every write point
P0 · M11 · Task · deps: X-QUEUE-0045, TEST-F03 · ref: STOR-F20
Done when: recovery never loses a delayed record and never delivers early; torn writes are truncated at the last valid entry with an event.

### [To Do] X-QUEUE-0050 Leader change: the new leader rebuilds the delay index from the persisted index plus a scan of records appended after the last snapshot
P0 · M11 · Task · deps: X-QUEUE-0049 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: a failover simulation keeps every schedule; the scan is bounded by the snapshot interval.

### [To Do] X-QUEUE-0051 Retention interplay: delayed records removed by retention or DeleteRecords before delivery are dropped with `queue_delayed_message_dropped` events and a counter
P0 · M11 · Task · deps: X-QUEUE-0048, STOR-F13 · ref: STOR-F13
Done when: tests cover time retention, size retention and DeleteRecords; documentation warns that the maximum delay must stay below retention (validated by QueueConfig).

### [To Do] X-QUEUE-0052 Native produce fields `deliver_at` and `delay` mapped to the queue headers on the X-NATIVE produce path
P0 · M11 · Task · deps: X-QUEUE-0044, X-NATIVE-F05 · ref: X-NATIVE-F05
Done when: native and header-based delays behave identically (test); conflicting field and header values are rejected.

### [To Do] X-QUEUE-0053 Simulation and property test: no delivery before deliver-at; lateness bounded by the tick plus scheduling bound under load; restarts preserve schedules
P0 · M11 · Task · deps: X-QUEUE-0048, X-QUEUE-0050, TEST-F06 · labels: test · ref: TEST-F06
Done when: 1000 seeds pass nightly with the deliver-not-before checker and a lateness histogram assertion.

### [To Do] X-QUEUE-0054 Benchmark: 10M delayed messages, measuring index size, resident memory and re-injection throughput
P1 · M11 · Task · deps: X-QUEUE-0048, TEST-F11 · labels: perf · ref: TEST-F11
Done when: results are in docs/queues/performance.md; memory stays within the configured bound.

## [To Do] X-QUEUE-F07 Message TTL
P1 · M11 · Feature · deps: X-QUEUE-F02
Scope: Per-message and per-queue expiry.

### [To Do] X-QUEUE-0055 TTL rule: expires-at equals the record's append time plus the smaller of `alluvium.ttl.ms` and `alluvium.queue.message.ttl.ms`; invalid TTL headers are rejected with INVALID_RECORD
P1 · M11 · Task · deps: none · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: tests cover per-message only, per-queue only, both, and invalid values.

### [To Do] X-QUEUE-0056 Lazy expiry at acquisition: expired records become ARCHIVED with reason `ttl_expired` instead of being delivered
P1 · M11 · Task · deps: X-QUEUE-0055 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: property test: no record is delivered at or after its expires-at.

### [To Do] X-QUEUE-0057 Background expiry sweeper with bounded work per tick so depth metrics stay accurate
P1 · M11 · Task · deps: X-QUEUE-0056 · labels: perf · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: depth converges to the unexpired count within N ticks for 1M expired records; work per tick is capped.

### [To Do] X-QUEUE-0058 In-flight expiry: a leased record that expires is not redelivered after its lease ends; acknowledgements of expired leased records still succeed
P1 · M11 · Task · deps: X-QUEUE-0056 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: tests cover ack before and after expiry and lease expiry after record expiry.

### [To Do] X-QUEUE-0059 Expiry to DLQ when `alluvium.queue.dlq.on.expiry=true`, with `alluvium.dlq.reason=ttl_expired`
P1 · M11 · Task · deps: X-QUEUE-0056, X-QUEUE-F05 · ref: X-QUEUE-F05
Done when: tests show expired records in the DLQ with the reason header.

### [To Do] X-QUEUE-0060 TTL property test: expiry never removes acknowledged state and never delivers expired records, across clock jumps
P1 · M11 · Task · deps: X-QUEUE-0057, X-QUEUE-0058 · labels: test · ref: TEST-F16
Done when: 10 000 proptest cases pass.

## [To Do] X-QUEUE-F08 Priorities
P1 · M11 · Feature · deps: X-QUEUE-F02
Scope: Priority lanes with strict or weighted scheduling.

### [To Do] X-QUEUE-0061 Priority lanes: lane from `alluvium.priority` (0 is highest) or the native field, `alluvium.queue.priority.levels` from 1 (off) to 10; out-of-range values rejected or clamped per `alluvium.queue.priority.out.of.range`
P1 · M11 · Task · deps: none · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: parsing and lane assignment are unit-tested; one level means no scheduling change (test).

### [To Do] X-QUEUE-0062 Per-lane available index within the acquisition window, with lookahead bounded by `alluvium.queue.priority.lookahead.records`
P1 · M11 · Task · deps: X-QUEUE-0061 · labels: perf · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: higher-priority records beyond the current window are found within the lookahead; memory per lane is bounded.

### [To Do] X-QUEUE-0063 Strict priority scheduling rule: acquisition always drains the highest non-empty lane
P1 · M11 · Task · deps: X-QUEUE-0062 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: property test: no lower-priority record is delivered while a higher-priority record is available within the lookahead.

### [To Do] X-QUEUE-0064 Weighted priority scheduling rule: deficit round robin over lanes with `alluvium.queue.priority.weights`
P1 · M11 · Task · deps: X-QUEUE-0062 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: property test: delivery shares stay within 5% of the weights over 10 000 deliveries when all lanes are backlogged; empty lanes donate their share.

### [To Do] X-QUEUE-0065 Priority interplay with delays (lane at re-injection), backoff and FIFO groups (group order takes precedence over priority)
P1 · M11 · Task · deps: X-QUEUE-0063, X-QUEUE-0064 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: tests cover each combination; the precedence rules are documented.

### [To Do] X-QUEUE-0066 Priority simulation under failover: scheduling rules hold after leader changes; lane indexes rebuild from state
P1 · M11 · Task · deps: X-QUEUE-0065, TEST-F06 · labels: test · ref: TEST-F06
Done when: 1000 seeds pass nightly with a priority-order checker.

## [To Do] X-QUEUE-F09 FIFO message groups
P1 · M11 · Feature · deps: X-QUEUE-F02
Scope: Ordered delivery per group key, parallel across groups.

### [To Do] X-QUEUE-0067 FIFO group key: `alluvium.message.group` header or native field (up to 128 bytes); native producers route by group key so a group lives in one partition
P1 · M11 · Task · deps: none · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: oversized keys are rejected; Kafka producers are documented to key records by group for co-location; a mismatch (same group in two partitions) yields independent ordering per partition, documented and tested.

### [To Do] X-QUEUE-0068 Active-group table per partition: at most one leased record per group, bounded by `alluvium.queue.fifo.max.active.groups` with backpressure
P1 · M11 · Task · deps: X-QUEUE-0067 · labels: nfr · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: memory per active group is measured; reaching the bound stops new groups from being leased without blocking active ones.

### [To Do] X-QUEUE-0069 FIFO ordering rule: the next record of a group becomes acquirable only after the previous one is acknowledged or dead-lettered; other groups are scanned past blocked ones within the window
P1 · M11 · Task · deps: X-QUEUE-0068 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: property test: per-group delivery order equals log order of non-dead-lettered records and never two leases per group.

### [To Do] X-QUEUE-0070 Head-of-line rule: backoff or visibility expiry of a group's head delays only that group; TTL expiry or DLQ of the head unblocks it
P1 · M11 · Task · deps: X-QUEUE-0069 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: tests show other groups progress while one group backs off.

### [To Do] X-QUEUE-0071 Group blocking state persisted and restored on leader change
P1 · M11 · Task · deps: X-QUEUE-0069 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: failover tests never deliver two records of one group concurrently.

### [To Do] X-QUEUE-0072 FIFO simulation with consumer crashes and failover using the FIFO checker
P1 · M11 · Task · deps: X-QUEUE-0070, X-QUEUE-0071, TEST-F06 · labels: test · ref: TEST-F06
Done when: 1000 seeds pass nightly.

### [To Do] X-QUEUE-0073 Benchmark: 1M distinct groups, throughput and memory
P2 · M11 · Task · deps: X-QUEUE-0069, TEST-F11 · labels: perf · ref: TEST-F11
Done when: results are in docs/queues/performance.md.

## [To Do] X-QUEUE-F10 Publish deduplication
P1 · M11 · Feature · deps: X-QUEUE-F02
Scope: Deduplication window by message id.

### [To Do] X-QUEUE-0074 Message id: `alluvium.message.id` header or native field, up to 128 bytes; records without an id are never deduplicated
P1 · M11 · Task · deps: none · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: oversized ids are rejected with INVALID_RECORD; tests cover missing and empty ids.

### [To Do] X-QUEUE-0075 Dedup store per partition: bounded map from id to (offset, append time) with the window `alluvium.queue.dedup.window.ms` and `alluvium.queue.dedup.max.entries`; deterministic eviction
P1 · M11 · Task · deps: X-QUEUE-0074 · labels: nfr · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: the store never exceeds its bounds (property test); eviction order is by append time then offset; hash flooding is prevented by the seeded hasher.

### [To Do] X-QUEUE-0076 Native produce deduplication: a duplicate within the window returns the original offset with `duplicate=true`
P1 · M11 · Task · deps: X-QUEUE-0075, X-NATIVE-F05 · ref: X-NATIVE-F05
Done when: tests cover duplicates within one request and across requests.

### [To Do] X-QUEUE-0077 Kafka produce deduplication: batches containing duplicate ids are rebuilt without them before append (CRC and compression recomputed); the response reports the stored batch's base offset
P1 · M11 · Task · deps: X-QUEUE-0075, REC-F04 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java @4.4.0
Done when: idempotent-producer sequence state stays valid (test with retries); a batch made only of duplicates appends nothing and succeeds; behaviour is documented as a deliberate, opt-in deviation.

### [To Do] X-QUEUE-0078 Leader change: the new leader rebuilds the dedup store by scanning records within the window, located through the time index
P1 · M11 · Task · deps: X-QUEUE-0075, STOR-F05 · ref: STOR-F05
Done when: the scan is bounded by the window; failover tests keep deduplication intact.

### [To Do] X-QUEUE-0079 Dedup window rule: duplicates outside the window or after count-based eviction are appended; best-effort beyond the window
P1 · M11 · Task · deps: X-QUEUE-0076, X-QUEUE-0077 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: property test with a manual clock checks inside, boundary and outside cases.

### [To Do] X-QUEUE-0080 Simulation: producer retries across leader failover never store two records with the same id within the window
P1 · M11 · Task · deps: X-QUEUE-0078, X-QUEUE-0079, TEST-F06 · labels: test · ref: TEST-F06
Done when: 1000 seeds pass nightly with a dedup checker.

### [To Do] X-QUEUE-0081 Benchmark: produce throughput with deduplication on versus off
P2 · M11 · Task · deps: X-QUEUE-0075, TEST-F11 · labels: perf · ref: TEST-F11
Done when: overhead is recorded in docs/queues/performance.md.

## [To Do] X-QUEUE-F11 Queue administration
P1 · M11 · Feature · deps: X-QUEUE-F02
Scope: Purge, depth, peek, move, redrive.

### [To Do] X-QUEUE-0082 Depth: available, delayed, in-flight and archived counts per partition and group from state, without scanning the log; `GetDepth` RPC and HTTP route
P1 · M11 · Task · deps: none · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: DESCRIBE on the topic and group is required; counts match a log scan in tests.

### [To Do] X-QUEUE-0083 Peek: read up to N available records without acquiring them (no attempt increment); `Peek` RPC
P1 · M11 · Task · deps: none · labels: security · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: READ on the topic is required; attempts are unchanged after peek (test); N is capped by `alluvium.queue.peek.max.messages`.

### [To Do] X-QUEUE-0084 Purge: archive all available and delayed records up to a log offset for a group; `Purge` RPC with `queue_purged` events carrying counts
P1 · M11 · Task · deps: X-QUEUE-0082 · ref: ADR Queue semantics (X-QUEUE-0001)
Done when: ALTER on the group and DELETE on the topic are required; leased records are left to finish; tests cover purge to the log end and to an offset.

### [To Do] X-QUEUE-0085 Move: copy matching records to another topic and archive them in the source through the redrive engine; `Move` RPC
P1 · M11 · Task · deps: X-QUEUE-0082, X-QUEUE-F05 · ref: X-QUEUE-F05
Done when: at-least-once with persisted progress; READ on the source and WRITE on the target are required.

### [To Do] X-QUEUE-0086 Redrive RPCs: StartRedrive, GetRedriveStatus and CancelRedrive
P1 · M11 · Task · deps: X-QUEUE-F05 · ref: X-QUEUE-F05
Done when: status shows copied, skipped and remaining counts; cancellation stops after the current batch; authorization follows the DLQ rules.

### [To Do] X-QUEUE-0087 `alluvium admin queues depth|peek|purge|move|redrive` commands with `--json` and confirmation for destructive operations
P1 · M11 · Task · deps: X-QUEUE-0082, X-QUEUE-0083, X-QUEUE-0084, X-QUEUE-0085, X-QUEUE-0086, X-NATIVE-F14 · ref: X-NATIVE-F14
Done when: golden tests cover every command; purge and move refuse to run without `--yes` in non-interactive shells.

## [To Do] X-QUEUE-F12 Kafka-protocol exposure
P1 · M11 · Feature · deps: X-QUEUE-F05, SHARE-F03, SHARE-F11
Scope: Opt-in share-group extensions (DLQ, delays) for Kafka clients.

### [To Do] X-QUEUE-0088 Mapping between queue semantics and share-group features: visibility timeout and `share.record.lock.duration.ms`, max attempts and `share.delivery.count.limit`, RENEW and ExtendVisibility, RELEASE and Nack, REJECT and DLQ
P1 · M11 · Task · deps: none · labels: docs · ref: KIP-932
Done when: docs/queues/kafka-share-consumers.md has the table with precedence rules; unit tests encode the mapping in one function.

### [To Do] X-QUEUE-0089 ShareFetch acquisition on queue topics honours deliver-at, backoff, priority lanes, FIFO groups and TTL
P1 · M11 · Task · deps: X-QUEUE-0088 · ref: core/src/main/java/kafka/server/share/SharePartition.java acquire @4.4.0
Done when: a Java KafkaShareConsumer never receives a record early, out of FIFO order or expired (compat test); non-queue topics follow Kafka behaviour unchanged.

### [To Do] X-QUEUE-0090 ShareAcknowledge on queue topics: RELEASE applies backoff, REJECT routes to the DLQ, RENEW extends visibility within the maximum
P1 · M11 · Task · deps: X-QUEUE-0088 · ref: clients/src/main/java/org/apache/kafka/clients/consumer/AcknowledgeType.java @4.4.0
Done when: tests cover each acknowledgement type with queue features on and off.

### [To Do] X-QUEUE-0091 Kafka producers on queue topics: header contract validated at append; invalid values rejected with INVALID_RECORD and record errors
P1 · M11 · Task · deps: X-QUEUE-0088 · ref: X-QUEUE-0003
Done when: a Java producer setting each header gets the documented behaviour; non-queue topics ignore the headers.

### [To Do] X-QUEUE-0092 Compatibility test: the Java KafkaShareConsumer 4.4.0 consumes a queue topic with delays, backoff, DLQ and FIFO honoured
P1 · M11 · Task · deps: X-QUEUE-0089, X-QUEUE-0090, X-QUEUE-0091 · labels: test · ref: TEST-F08
Done when: the scenario runs in `cargo xtask compat` with consumer rebalancing.

### [To Do] X-QUEUE-0093 Differential test: share-group API responses on non-queue topics are identical to Kafka 4.4.0 while queue support is enabled cluster-wide
P1 · M11 · Task · deps: X-QUEUE-0089, X-QUEUE-0090, TEST-F09 · labels: test · ref: TEST-F09
Done when: the share-group differential scenarios pass with `alluvium.queue.*` features available.

## [To Do] X-QUEUE-F13 Queue metrics
P0 · M11 · Feature · deps: X-QUEUE-F02, OBS-F04
Scope: Depth, oldest-message age, in-flight, redeliveries, DLQ rate.

### [To Do] X-QUEUE-0094 Queue gauges: depth (available), delayed, in-flight and oldest available message age in seconds per queue topic and group
P0 · M11 · Task · deps: none · ref: docs/standards/debuggability.md#metrics
Done when: labels are limited to topic and group with the cardinality cap; values match state in tests.

### [To Do] X-QUEUE-0095 Queue counters: deliveries, acknowledgements, nacks, redeliveries, visibility expirations, DLQ writes, TTL expirations, dedup hits
P0 · M11 · Task · deps: none · ref: docs/standards/debuggability.md#metrics
Done when: each counter is asserted in the state-machine driver tests.

### [To Do] X-QUEUE-0096 Queue histograms: acknowledgement latency (acquire to ack), end-to-end latency (append to ack), delivery lateness for delayed messages
P1 · M11 · Task · deps: none · ref: docs/standards/debuggability.md#metrics
Done when: buckets are documented; tests assert observations with the simulated clock.

### [To Do] X-QUEUE-0097 Events with reasons and flight-recorder entries: `record_archived{reason}`, `queue_dlq_write_failed`, `redrive_started`, `redrive_completed`, `queue_mode_changed`, `queue_purged`
P0 · M11 · Task · deps: OBS-F06, OBS-F07 · ref: docs/standards/debuggability.md#events-for-state-transitions
Done when: events are in the crate's `events.rs` catalog; payloads never appear; per-record events stay at debug level.

### [To Do] X-QUEUE-0098 Diagnostics route `/debug/queues/<topic>/<partition>`: state summary, lanes, active FIFO groups (count), delay index size, without payloads
P1 · M11 · Task · deps: OBS-F08 · ref: OBS-F08
Done when: the route is bounded and authenticated per OBS-F08.

### [To Do] X-QUEUE-0099 Queue metrics and events in docs/metrics.md and the events catalog, with alert suggestions (oldest age, DLQ rate, ARCHIVING backlog)
P1 · M11 · Task · deps: X-QUEUE-0094, X-QUEUE-0095, X-QUEUE-0096, X-QUEUE-0097 · labels: docs · ref: DOCS-F05
Done when: generated references include every queue metric and event.

## [To Do] X-QUEUE-F14 Queue testing
P0 · M11 · Feature · deps: X-QUEUE-F03, TEST-F06
Scope: Simulation of at-least-once delivery, no loss, bounded redelivery.

### [To Do] X-QUEUE-0100 Invariant checkers in the testkit for every queue invariant in docs/queues/invariants.md
P0 · M11 · Task · deps: TEST-F05 · labels: test · ref: TEST-F05
Done when: each checker has a deliberately broken state machine test proving it fires.

### [To Do] X-QUEUE-0101 Simulation: at-least-once and no loss under consumer crashes, broker crashes, leader failover and network partitions
P0 · M11 · Task · deps: X-QUEUE-0100 · labels: test · ref: TEST-F06
Done when: 1000 seeds pass nightly; thousands of seeds pass in the milestone exit run.

### [To Do] X-QUEUE-0102 Simulation: bounded redelivery; no record is delivered more than the maximum attempts and exhausted records reach the DLQ
P0 · M11 · Task · deps: X-QUEUE-0100, X-QUEUE-F05 · labels: test · ref: TEST-F06
Done when: 1000 seeds pass nightly.

### [To Do] X-QUEUE-0103 Simulation with all queue features combined: delay, priority, FIFO, TTL and deduplication
P0 · M11 · Task · deps: X-QUEUE-0100, X-QUEUE-F06, X-QUEUE-F07, X-QUEUE-F08, X-QUEUE-F09, X-QUEUE-F10 · labels: test · ref: TEST-F06
Done when: all checkers pass for 1000 seeds nightly.

### [To Do] X-QUEUE-0104 Model-based property test of the queue state machine against a reference model for arbitrary command sequences
P0 · M11 · Task · deps: none · labels: test · ref: TEST-F16
Done when: 10 000 cases pass; shrinking yields minimal counterexamples; regressions persisted.

### [To Do] X-QUEUE-0105 Off by default: default topics have no queue behaviour; share-group state records are byte-identical to Kafka's; `alluvium.queue.*` keys are absent from Kafka DescribeConfigs
P0 · M11 · Task · deps: X-QUEUE-F06, X-QUEUE-F07, X-QUEUE-F08, X-QUEUE-F09, X-QUEUE-F10, X-QUEUE-F12 · labels: test · ref: ADR-0010
Done when: a test compares share-group state records and DescribeConfigs output with queue support compiled in but unused.

### [To Do] X-QUEUE-0106 Kafka compatibility and differential suites pass with queue support enabled but unused
P0 · M11 · Task · deps: X-QUEUE-0105, TEST-F08, TEST-F09 · labels: test · ref: ADR-0010
Done when: a CI job variant enables queue support cluster-wide without queue topics; results equal the default job's.

## [To Do] X-QUEUE-F15 Queue benchmarks and documentation
P1 · M11 · Feature · deps: X-QUEUE-F03
Scope: Benchmarks and user guide.

### [To Do] X-QUEUE-0107 Benchmark: receive and acknowledge throughput and p99 versus plain share groups with 1 KiB messages and 1, 100 and 10 000 consumers
P1 · M11 · Task · deps: TEST-F11 · labels: perf · ref: TEST-F11
Done when: results are in docs/queues/performance.md with regression thresholds.

### [To Do] X-QUEUE-0108 Benchmark: DLQ and redrive throughput, and timer overhead with 1M backed-off records
P2 · M11 · Task · deps: X-QUEUE-F05, TEST-F11 · labels: perf · ref: TEST-F11
Done when: results are recorded with CPU and memory.

### [To Do] X-QUEUE-0109 User guide docs/queues/: concepts, configuration keys, header contract, delivery guarantees and their limits
P1 · M11 · Task · deps: none · labels: docs · ref: DOCS-F01
Done when: examples run as docs tests against a dev node; guarantees link to the invariants doc.

### [To Do] X-QUEUE-0110 Guide for Kafka share consumers on queue topics: configuration and semantic mapping
P1 · M11 · Task · deps: X-QUEUE-F12 · labels: docs · ref: X-QUEUE-F12
Done when: a Java example is executed in the docs test.

### [To Do] X-QUEUE-0111 Opt-in and migration guide: enabling queue mode on a topic, mapping from SQS and RabbitMQ concepts, rollback
P1 · M11 · Task · deps: X-QUEUE-0109 · labels: docs · ref: ADR-0010
Done when: the guide lists each operation's effect on existing share groups and the rollback steps.
