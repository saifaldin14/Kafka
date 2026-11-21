# CTRL: Controller

> Crates: `alluvium-controller` (layer 2) · Kafka counterpart: `metadata/src/main/java/org/apache/kafka/controller`, `ControllerApis.scala` · Milestones: M2-M7

The quorum controller: an event-driven state machine that validates admin requests and writes metadata records; manages brokers, topics, partitions, leaders, ISR, reassignment, configs, security records and features.

## [To Do] CTRL-F01 Quorum controller core
P0 · M2 · Feature · deps: RAFT-F10, META-F03
Scope: Deterministic event processing, record generation, completion deferred until commit, active/standby transitions.

### [To Do] CTRL-0001 Spike: controller state machine design (event model, uncommitted state and revert, bounded queue)
P0 · M2 · Spike · deps: none · labels: docs · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java @4.4.0; server-common/src/main/java/org/apache/kafka/timeline/SnapshotRegistry.java @4.4.0; ADR-0002
Done when: ADR 'Quorum controller state machine' is accepted in docs/adr/; it decides how uncommitted state is kept and reverted (timeline snapshots per offset versus committed image plus overlay), the event types (read, write, control), deadline handling, the bounded event queue's full-queue behavior and its error mapping, and how results are deferred until commit.

### [To Do] CTRL-0002 Timeline collections: snapshot at offset, read at snapshot, revert to snapshot
P0 · M2 · Task · deps: CTRL-0001 · ref: server-common/src/main/java/org/apache/kafka/timeline/SnapshotRegistry.java @4.4.0; server-common/src/main/java/org/apache/kafka/timeline/TimelineHashMap.java @4.4.0; server-common/src/main/java/org/apache/kafka/timeline/TimelineHashSet.java @4.4.0
Done when: Timeline map, set and scalar types support creating a snapshot at an offset, reading as of any live snapshot, reverting to a snapshot and deleting snapshots up to an offset, with deterministic iteration; a property test checks reads and reverts against a model that copies state at every snapshot.

### [To Do] CTRL-0003 ControllerError enum with one exhaustive Kafka error-code mapping
P0 · M2 · Task · deps: CTRL-0001, ERR-F02 · ref: metadata/src/main/java/org/apache/kafka/controller/errors/ControllerExceptions.java @4.4.0; metadata/src/main/java/org/apache/kafka/controller/errors/EventHandlerExceptionInfo.java @4.4.0
Done when: alluvium-controller defines a thiserror enum covering not controller (with leader hint), timeout, stale broker epoch, invalid request, policy violation, throttling and every validation error the managers raise, each carrying its identifying fields; one exhaustive match maps to ErrorCode without `_`; tests per variant.

### [To Do] CTRL-0004 ControllerResult and record batching: atomic results in one batch, non-atomic results split
P0 · M2 · Task · deps: CTRL-0003 · ref: metadata/src/main/java/org/apache/kafka/controller/ControllerResult.java @4.4.0; metadata/src/main/java/org/apache/kafka/controller/QuorumController.java appendRecords @4.4.0
Done when: Operations return records plus a response and an atomic flag; atomic results larger than max records per batch (10000) fail the operation; non-atomic results split into batches of at most 10000; per-operation record lists are bounded at 10000 (MAX_RECORDS_PER_USER_OP) with a typed error; unit tests at the boundaries.

### [To Do] CTRL-0005 Replication state: replay TopicRecord, PartitionRecord and PartitionChangeRecord into timeline maps
P0 · M2 · Task · deps: CTRL-0002, META-F01 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java replay @4.4.0; metadata/src/main/java/org/apache/kafka/controller/BrokersToIsrs.java @4.4.0; metadata/src/main/java/org/apache/kafka/metadata/PartitionRegistration.java merge @4.4.0
Done when: Topics by id and name, partition registrations (merged with change records), the collision-character name index, the ISR membership index (partitions per broker and leaderless partitions) and the imbalanced-partition set are maintained on replay; replaying records out of order or for unknown topics is a fatal replay error; tests per record.

### [To Do] CTRL-0006 Offset control: committed offset and epoch, last stable offset and metadata transactions
P0 · M2 · Task · deps: CTRL-0002 · ref: metadata/src/main/java/org/apache/kafka/controller/OffsetControlManager.java @4.4.0
Done when: OffsetControlManager tracks next write offset, last committed offset/epoch/timestamp and the last stable offset that excludes in-progress BeginTransactionRecord..EndTransactionRecord ranges (AbortTransactionRecord ends them); it creates a timeline snapshot at each batch end and deletes snapshots up to the stable offset; tests with transactions and aborts.

### [To Do] CTRL-0007 Bounded controller event queue with per-event deadlines
P0 · M2 · Task · deps: CTRL-0001, CTRL-0003 · ref: server-common/src/main/java/org/apache/kafka/queue/KafkaEventQueue.java @4.4.0; metadata/src/main/java/org/apache/kafka/controller/QuorumController.java ControllerEvent @4.4.0
Done when: Read, write and control events run one at a time in FIFO order; an event past its deadline fails with a timeout error (REQUEST_TIMED_OUT to clients) without running; a full queue rejects per the ADR; queue time and processing time are measured per event; tests with the simulated clock.

### [To Do] CTRL-0008 Write events: run on the active controller, replay immediately, append to raft, defer completion
P0 · M2 · Task · deps: CTRL-0004, CTRL-0007, CTRL-0006 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java ControllerWriteEvent @4.4.0
Done when: A write event on a non-active controller fails NOT_CONTROLLER with the known leader; otherwise the operation runs, its records are replayed at the next offsets, appended via prepare_append/schedule_prepared_append, and the response is completed when the committed offset reaches the last record; operations with no records wait for the highest pending offset; tests.

### [To Do] CTRL-0009 Deferred completion queue: complete in offset order, fail all on renounce
P0 · M2 · Task · deps: CTRL-0008 · ref: server-common/src/main/java/org/apache/kafka/deferred/DeferredEventQueue.java @4.4.0
Done when: Pending responses are keyed by offset, completed up to the last stable offset in order and failed with NOT_CONTROLLER when the controller renounces; the queue is bounded by the event queue; tests.

### [To Do] CTRL-0010 Read events observe only committed state
P0 · M2 · Task · deps: CTRL-0006, CTRL-0007 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java ControllerReadEvent @4.4.0
Done when: Read operations (find topic ids/names, describe configs, list reassignments, finalized features) read the timeline as of the last stable offset, so uncommitted writes are never visible; tests with a pending write.

### [To Do] CTRL-0011 Commit handling: complete deferred writes on the active, replay committed batches on standbys
P0 · M2 · Task · deps: CTRL-0009, CTRL-0005 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java QuorumMetaLogListener handleCommit, replay @4.4.0
Done when: On handle_commit the active controller advances committed offsets, completes deferred events and deletes old snapshots; a standby replays every record through one exhaustive record-type dispatch to the managers and advances offsets; unknown record types are fatal; tests for both roles.

### [To Do] CTRL-0012 Snapshot load on standbys resets and rebuilds all manager state
P0 · M2 · Task · deps: CTRL-0011, RAFT-F06 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java QuorumMetaLogListener handleLoadSnapshot @4.4.0
Done when: handle_load_snapshot on a standby clears every manager, replays the snapshot records and sets offsets to the snapshot id; receiving a snapshot load while active is an invariant violation; tests.

### [To Do] CTRL-0013 Activation records on becoming active: bootstrap on empty log, abort in-flight transactions
P0 · M2 · Task · deps: CTRL-0008, META-F06 · ref: metadata/src/main/java/org/apache/kafka/controller/ActivationRecordsGenerator.java @4.4.0
Done when: With an empty log the new active controller writes the bootstrap records (metadata.version and bootstrap metadata) atomically or wrapped in a metadata transaction; with a non-empty log it writes AbortTransactionRecord(reason 'Controller failover') if a transaction is in progress; a non-empty log without metadata.version is a fatal error; tests ported from ActivationRecordsGeneratorTest.

### [To Do] CTRL-0014 Claim leadership: activate managers and run the activation event
P0 · M2 · Task · deps: CTRL-0013 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java claim, CompleteActivationEvent @4.4.0
Done when: On a leader change naming this node, after the listener has caught up to the epoch start, the controller sets its claim epoch and next write offset, activates managers (fresh broker heartbeat sessions for registered brokers), starts periodic tasks and runs the activation event; NewActiveControllersCount and ActiveControllerCount change; tests.

### [To Do] CTRL-0015 Renounce leadership: revert to committed state and fail pending work
P0 · M2 · Task · deps: CTRL-0014 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java renounce @4.4.0
Done when: On losing leadership or an unexpected write failure the controller reverts the timeline to the last committed offset, fails deferred events with NOT_CONTROLLER, deactivates managers and periodic tasks, and resigns the raft epoch when it caused the renounce; tests that uncommitted records are gone after renounce.

### [To Do] CTRL-0016 Event failure classification: timeouts, expected errors, unexpected errors and replay faults
P0 · M2 · Task · deps: CTRL-0015 · ref: metadata/src/main/java/org/apache/kafka/controller/errors/EventHandlerExceptionInfo.java @4.4.0; metadata/src/main/java/org/apache/kafka/controller/QuorumController.java handleEventException @4.4.0
Done when: Expected API errors complete the request; timeouts count as timed-out events; unexpected exceptions on the active controller renounce; replay failures increment the metadata error signal and are fatal on the active controller; the classification matches EventHandlerExceptionInfo with tests per class.

### [To Do] CTRL-0017 Periodic task scheduler for the active controller
P0 · M2 · Task · deps: CTRL-0014 · ref: metadata/src/main/java/org/apache/kafka/controller/PeriodicTaskControlManager.java @4.4.0; metadata/src/main/java/org/apache/kafka/controller/PeriodicTask.java @4.4.0
Done when: Named periodic tasks run as write events only while active, reschedule immediately when they report more work, back off after failures, and stop on renounce; tests for each case with the simulated clock.

### [To Do] CTRL-0018 No-op record writer driven by metadata.max.idle.interval.ms
P0 · M2 · Task · deps: CTRL-0017 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java registerWriteNoOpRecord @4.4.0
Done when: The active controller appends a NoOpRecord every metadata.max.idle.interval.ms (0 disables) when the metadata.version supports it, keeping the high watermark advancing for broker liveness detection; tests.

### [To Do] CTRL-0019 Controller request context: principal, header, deadline and mutation-quota hook
P0 · M2 · Task · deps: CTRL-0007 · ref: metadata/src/main/java/org/apache/kafka/controller/ControllerRequestContext.java @4.4.0
Done when: Each request carries the principal, request header, deadline (now + request timeout) and a partition-mutation quota hook that is a no-op until QUOTA-F05; forwarded requests carry the original client principal; tests.

### [To Do] CTRL-0020 Controller operations API consumed by API handlers
P0 · M2 · Task · deps: CTRL-0008, CTRL-0010, CTRL-0019 · labels: docs · ref: metadata/src/main/java/org/apache/kafka/controller/Controller.java @4.4.0
Done when: One async method per controller operation (mirroring Controller.java) returns a future of the typed response; handlers in alluvium-server call it; module docs describe consistency guarantees (read-after-write via deferred completion).

### [To Do] CTRL-0021 Controller driver: supervised task bridging raft listener callbacks, timers and requests
P0 · M2 · Task · deps: CTRL-0011, CTRL-0014, SRV-F02 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java @4.4.0; core/src/main/scala/kafka/server/ControllerServer.scala @4.4.0
Done when: A named, supervised Tokio task owns the controller state machine, turns raft commit/snapshot/leader-change callbacks and API calls into events, fires deadlines and periodic tasks from timers, and shuts down after request draining and before the raft client; tests with the simulated clock.

### [To Do] CTRL-0022 Property test: standby replay reproduces the active controller's state
P0 · M2 · Task · deps: CTRL-0015, CTRL-0012 · labels: test
Done when: For random operation sequences (create/delete topics, register/fence brokers, configs), the state built by a standby from the committed records equals the active's state, and the active's state after renounce equals the committed prefix; seeds are printed and replayable.

### [To Do] CTRL-0023 Controller crate docs: event model, consistency and invariants
P1 · M2 · Task · deps: CTRL-0020 · labels: docs
Done when: alluvium-controller's lib.rs `//!` documents responsibility, layer, Kafka counterpart, the event model, read-after-write semantics, activation/renounce and invariants; docs/glossary.md gains active controller, metadata transaction and broker epoch.

### [To Do] CTRL-0191 Input journaling hook for the controller
P2 · M3 · Task · deps: SRV-F15
Done when: with `alluvium.journal.enable` set, the controller driver journals every event fed to the controller state machine (metadata only); the replay tool reproduces the same records and state from the journal in a test.

## [To Do] CTRL-F02 Cluster control
P0 · M2 · Feature · deps: CTRL-F01
Scope: Broker registration, heartbeats, session timeouts, fencing and unfencing, broker epochs, controlled shutdown.

### [To Do] CTRL-0024 Broker registration replay: RegisterBrokerRecord, UnregisterBrokerRecord and change records
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java replay @4.4.0; metadata/src/main/resources/common/metadata/RegisterBrokerRecord.json @4.4.0; metadata/src/main/resources/common/metadata/BrokerRegistrationChangeRecord.json @4.4.0
Done when: Replaying RegisterBrokerRecord stores the registration (epoch, incarnation id, listeners, rack, features, fenced, directories) and its offset; UnregisterBrokerRecord with a mismatched epoch is a fatal replay error; FenceBrokerRecord, UnfenceBrokerRecord and BrokerRegistrationChangeRecord update fenced/controlled-shutdown/directories; tests per record.

### [To Do] CTRL-0025 Broker registration validation: cluster id, duplicate session, ZK brokers
P0 · M2 · Task · deps: CTRL-0024 · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java registerBroker @4.4.0
Done when: Registration on an inactive controller fails NOT_CONTROLLER; a different cluster id fails INCONSISTENT_CLUSTER_ID; an existing live session with another incarnation id fails DUPLICATE_BROKER_REGISTRATION; is_migrating_zk_broker fails BROKER_ID_NOT_REGISTERED; tests per error.

### [To Do] CTRL-0026 Registration log directory validation
P0 · M2 · Task · deps: CTRL-0025 · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java registerBroker @4.4.0
Done when: When the metadata.version supports directory assignment, an empty directory list, a reserved directory id, duplicate ids, or a directory owned by another broker fail INVALID_REGISTRATION; accepted directories are stored and indexed; tests.

### [To Do] CTRL-0027 Registration feature compatibility checks
P0 · M2 · Task · deps: CTRL-0025, CTRL-F17 · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java processRegistrationFeature @4.4.0
Done when: Every advertised feature range must contain the finalized level (metadata.version required, others default 0), otherwise UNSUPPORTED_VERSION naming feature and range; a missing metadata.version fails INVALID_REGISTRATION; finalized features the broker does not advertise must be at level 0; tests per rule.

### [To Do] CTRL-0028 Broker epochs and incarnations: new incarnation gets a new epoch, same incarnation amends
P0 · M2 · Task · deps: CTRL-0025 · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java registerBroker @4.4.0; metadata/src/main/java/org/apache/kafka/metadata/BrokerRegistrationReply.java @4.4.0
Done when: A new incarnation id gets broker epoch = the offset of its RegisterBrokerRecord and a reset heartbeat session; re-registration with the same incarnation keeps the epoch and the fenced and controlled-shutdown flags; the record version follows the metadata.version; the reply carries the broker epoch; tests.

### [To Do] CTRL-0029 Heartbeat manager: sessions with broker.session.timeout.ms on the monotonic clock
P0 · M2 · Task · deps: CTRL-0024 · ref: metadata/src/main/java/org/apache/kafka/controller/BrokerHeartbeatManager.java @4.4.0; metadata/src/main/java/org/apache/kafka/controller/BrokerHeartbeatTracker.java @4.4.0
Done when: Sessions are created on registration, touched by heartbeats (recording metadata offset), expire after broker.session.timeout.ms (default 9000 ms), and are removed on unregistration or new incarnations; the lowest metadata offset over active (unfenced) brokers is computed; contact times are tracked off the event queue; tests with the simulated clock.

### [To Do] CTRL-0030 Broker control state machine: FENCED, UNFENCED, CONTROLLED_SHUTDOWN, SHUTDOWN_NOW
P0 · M2 · Task · deps: CTRL-0029 · ref: metadata/src/main/java/org/apache/kafka/controller/BrokerHeartbeatManager.java calculateNextBrokerState @4.4.0; metadata/src/main/java/org/apache/kafka/controller/BrokerControlState.java @4.4.0
Done when: Next state follows Kafka's table: fenced brokers unfence only when not asking to stay fenced and caught up to their registration offset, or shut down at once if asked; unfenced brokers asking to shut down enter controlled shutdown when they lead partitions, else SHUTDOWN_NOW; a table-driven test covers every (state, want_fence, want_shutdown, caught_up, has_leaderships) case.

### [To Do] CTRL-0031 Partition change builder core: target ISR, clean online election and epoch semantics
P0 · M2 · Task · deps: CTRL-0005 · ref: metadata/src/main/java/org/apache/kafka/controller/PartitionChangeBuilder.java build, electAnyLeader, changeRecordIsNoOp @4.4.0
Done when: The builder takes a partition, a target ISR and an acceptable-leader predicate, keeps the current leader when valid, otherwise elects the first acceptable ISR member in replica order (or none), bumps the partition epoch on any change and the leader epoch on leader change, emits no record for a no-op, and picks the record version from the metadata.version; property tests.

### [To Do] CTRL-0032 Leader and ISR updates when a broker is fenced, unfenced or enters controlled shutdown
P0 · M2 · Task · deps: CTRL-0031, CTRL-0030 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java handleBrokerFenced, handleBrokerUnfenced, handleBrokerInControlledShutdown, generateLeaderAndIsrUpdates @4.4.0
Done when: Fencing removes the broker from every ISR it is in (electing new leaders) and writes BrokerRegistrationChangeRecord(fenced); unfencing writes the change record and elects leaders for leaderless partitions it can lead; controlled shutdown writes the in-controlled-shutdown change and moves its leaderships; an ISR is never emptied when ELR is disabled; tests per case.

### [To Do] CTRL-0033 BrokerHeartbeat processing and reply
P0 · M2 · Task · deps: CTRL-0032 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java processBrokerHeartbeat @4.4.0; metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java checkBrokerEpoch @4.4.0
Done when: A heartbeat with an unknown broker or wrong epoch fails STALE_BROKER_EPOCH; state changes produce the records above; the session is touched with the reported metadata offset; the reply carries is_caught_up, is_fenced, in_controlled_shutdown and should_shut_down; tests per state.

### [To Do] CTRL-0034 New incarnation cleanup of the previous incarnation's leaderships and ISR membership
P0 · M2 · Task · deps: CTRL-0032, CTRL-0028 · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java registerBroker @4.4.0
Done when: Registering a new incarnation first generates the shutdown records for the old one (as a fence) then the RegisterBrokerRecord last, as a non-atomic result so a failover retries cleanly; tests that the old incarnation's leaderships move.

### [To Do] CTRL-0035 Periodic fencing of brokers whose session expired, one broker per run
P0 · M2 · Task · deps: CTRL-0032, CTRL-0017 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java maybeFenceOneStaleBroker @4.4.0; metadata/src/main/java/org/apache/kafka/controller/QuorumController.java maybeFenceStaleBrokerPeriodNs @4.4.0
Done when: A periodic task every max(1 ms, session timeout / 8) fences the first expired broker (removing stale tracker entries for unknown or older epochs without fencing) and reschedules immediately when it fenced one; tests that a broker is fenced within 112.5% of the session timeout.

### [To Do] CTRL-0036 Heartbeats that time out in the queue only refresh the session
P0 · M2 · Task · deps: CTRL-0033, CTRL-0007 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java processExpiredBrokerHeartbeat @4.4.0; metadata/src/main/java/org/apache/kafka/controller/QuorumController.java processBrokerHeartbeat @4.4.0
Done when: A heartbeat whose deadline passes while queued is processed by touching the session without any state change, logged at error and counted for TimedOutBrokerHeartbeatCount; tests.

### [To Do] CTRL-0037 Controlled shutdown completion: no leaderships and metadata caught up
P0 · M2 · Task · deps: CTRL-0033 · ref: metadata/src/main/java/org/apache/kafka/controller/BrokerHeartbeatManager.java calculateNextBrokerState, lowestActiveOffset @4.4.0
Done when: A broker in controlled shutdown gets should_shut_down only after it leads no partitions and the lowest active broker metadata offset has reached its controlled-shutdown offset; tests with a lagging peer.

### [To Do] CTRL-0038 Usable brokers view for placement and leader acceptance
P0 · M2 · Task · deps: CTRL-0024 · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java usableBrokers, isActive @4.4.0
Done when: The cluster view lists registered brokers with rack and fenced flag for the placer, and is_active (registered, unfenced, not in controlled shutdown) for leader and ISR acceptance; tests.

### [To Do] CTRL-0039 Port Kafka's cluster control and heartbeat tests
P0 · M2 · Task · deps: CTRL-0034, CTRL-0035, CTRL-0037, CTRL-0036, CTRL-0026, CTRL-0027, CTRL-0038 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/ClusterControlManagerTest.java @4.4.0; metadata/src/test/java/org/apache/kafka/controller/BrokerHeartbeatManagerTest.java @4.4.0
Done when: Cases from ClusterControlManagerTest and BrokerHeartbeatManagerTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F03 Controller registration
P0 · M3 · Feature · deps: CTRL-F01
Scope: ControllerRegistration handling and controller endpoints in metadata.

### [To Do] CTRL-0040 ControllerRegistration operation and RegisterControllerRecord
P0 · M3 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java registerController @4.4.0; metadata/src/main/resources/common/metadata/RegisterControllerRecord.json @4.4.0
Done when: registerController fails UNSUPPORTED_VERSION when the metadata.version predates controller registration; otherwise it writes RegisterControllerRecord (id, incarnation, endpoints, supported features, zk_migration_ready false) atomically; tests.

### [To Do] CTRL-0041 Controller registrations replay and supported-features view
P0 · M3 · Task · deps: CTRL-0040 · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java replay, controllerSupportedFeatures @4.4.0
Done when: Replay stores controller registrations; the supported feature ranges of all registered controllers are exposed to feature validation; tests.

### [To Do] CTRL-0042 Controller-node registration manager: register with the active controller after catching up
P0 · M3 · Task · deps: CTRL-0041, NET-F10 · ref: server/src/main/java/org/apache/kafka/server/controller/ControllerRegistrationManager.java @4.4.0
Done when: Each controller sends ControllerRegistration to the active controller once its metadata has caught up and the metadata.version supports it, retries with backoff until the record is seen in metadata, and re-registers after a restart with a new incarnation; tests with a fake channel.

### [To Do] CTRL-0043 Controller endpoints from registrations for brokers and DescribeCluster
P0 · M3 · Task · deps: CTRL-0041, META-F09 · ref: metadata/src/main/java/org/apache/kafka/image/publisher/ControllerRegistrationsPublisher.java @4.4.0
Done when: Brokers resolve controller endpoints per listener from controller registrations in the image (falling back to the static quorum configuration before registration exists); DescribeCluster on the controller listener returns them; tests.

## [To Do] CTRL-F04 Topic creation
P0 · M2 · Feature · deps: CTRL-F01, CTRL-F07, CFG-F05
Scope: Validation, topic ids, replica assignment (explicit or placed), configs, policy hooks, validate-only.

### [To Do] CTRL-0044 Reject CreateTopics batches adding more than 10000 partitions
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java validateTotalNumberOfPartitions @4.4.0
Done when: The total of requested partitions (defaults for -1, assignments when given) above 10000 fails the whole request with POLICY_VIOLATION 'Excessively large number of partitions per request.' before allocating; tests at 10000 and 10001.

### [To Do] CTRL-0045 Topic name validation, collision detection and TOPIC_ALREADY_EXISTS
P0 · M2 · Task · deps: TYPE-F04 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java validateNewTopicNames, createTopics @4.4.0
Done when: Invalid names fail INVALID_TOPIC_EXCEPTION; a name colliding with an existing topic after '.'/'_' normalization fails INVALID_TOPIC_EXCEPTION naming the existing topic; existing names fail TOPIC_ALREADY_EXISTS; tests per case.

### [To Do] CTRL-0046 Creation configs: null values rejected, validated and emitted after the TopicRecord
P0 · M2 · Task · deps: CTRL-0045, CTRL-F11 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java computeConfigChanges, createTopic @4.4.0
Done when: A null config value fails INVALID_CONFIG listing the keys; other configs are validated through the configuration manager (INVALID_CONFIG on bad keys or values) and their ConfigRecords are placed after the TopicRecord and before PartitionRecords; tests.

### [To Do] CTRL-0047 Manual replica assignments in CreateTopics
P0 · M2 · Task · deps: CTRL-0045, CTRL-0038 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createTopic, validateManualPartitionAssignment @4.4.0
Done when: With assignments, replication factor and partition count must be -1 (INVALID_REQUEST); duplicate partition indexes, empty lists, unregistered brokers, repeated brokers or inconsistent replica counts fail INVALID_REPLICA_ASSIGNMENT; partitions must be 0..n-1; an assignment whose brokers are all fenced or shutting down fails INVALID_REPLICA_ASSIGNMENT; ISR is the active replicas in order and the leader the first of them; tests per rule.

### [To Do] CTRL-0048 Placed assignments: defaults, replication factor and partition count validation
P0 · M2 · Task · deps: CTRL-0045 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createTopic @4.4.0
Done when: Replication factor 0 or < -1 fails INVALID_REPLICATION_FACTOR and partition count 0 or < -1 fails INVALID_PARTITIONS; -1 uses default.replication.factor and num.partitions; placer failures map to INVALID_REPLICATION_FACTOR with Kafka's message; a partition whose replicas are all inactive fails INVALID_REPLICATION_FACTOR; tests.

### [To Do] CTRL-0049 Spike: create and alter config policies without Java plugins
P0 · M2 · Spike · deps: none · labels: docs · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java maybeCheckCreateTopicPolicy @4.4.0; ADR-0001
Done when: ADR 'Topic and config policies' is accepted: it decides how create.topic.policy.class.name and alter.config.policy.class.name are supported (built-in named policies behind a Rust trait, startup rejection of unknown class names, later WASM), the request metadata passed, and error mapping to POLICY_VIOLATION.

### [To Do] CTRL-0050 Create topic policy hook invocation
P0 · M2 · Task · deps: CTRL-0049, CTRL-0047, CTRL-0048 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java maybeCheckCreateTopicPolicy @4.4.0
Done when: When a policy is configured it receives name, partition count, replication factor or assignments, and configs; a violation fails POLICY_VIOLATION with its message and other policy errors map through the error mapping (unknown errors logged with the request); tests with a test policy.

### [To Do] CTRL-0051 Topic id generation from the injected RNG
P0 · M2 · Task · deps: TYPE-F03 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createTopic @4.4.0
Done when: Topic ids come from the controller's seeded RNG, never the zero or reserved metadata ids and never an existing id; simulation runs with the same seed produce the same ids; tests.

### [To Do] CTRL-0052 Record generation and validate-only for CreateTopics
P0 · M2 · Task · deps: CTRL-0046, CTRL-0050, CTRL-0051 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createTopics, buildPartitionRegistration @4.4.0
Done when: Successful topics produce TopicRecord, ConfigRecords and one PartitionRecord per partition (leader epoch 0, partition epoch 0, RECOVERED, ELR fields only when enabled) in one atomic result; validate_only returns the same response with no records; tests decode the records.

### [To Do] CTRL-0053 CreateTopics response: ids, partition counts, replication factor and effective configs
P0 · M2 · Task · deps: CTRL-0052 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createTopic @4.4.0; metadata/src/main/java/org/apache/kafka/controller/ConfigurationControlManager.java computeEffectiveTopicConfigs @4.4.0
Done when: Each result has error code/message, topic id, partition count and replication factor; effective configs (name, value with sensitive values null, read-only, source, sensitive) are returned only for describe-authorized topics, otherwise topic_config_error_code is TOPIC_AUTHORIZATION_FAILED; tests.

### [To Do] CTRL-0054 Controller mutation quota on topic creation
P0 · M4 · Task · deps: CTRL-0052, QUOTA-F05 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createTopic @4.4.0; server/src/main/java/org/apache/kafka/server/quota/ControllerMutationQuotaManager.java @4.4.0
Done when: Each topic's partition count is charged to the request's mutation quota; when exceeded the topic fails THROTTLING_QUOTA_EXCEEDED with the throttle time and no records; tests with QUOTA-F05's quota.

### [To Do] CTRL-0055 Port Kafka's CreateTopics controller tests
P0 · M2 · Task · deps: CTRL-0053, CTRL-0044 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/ReplicationControlManagerTest.java @4.4.0
Done when: createTopics cases from ReplicationControlManagerTest at 4.4.0 pass with attribution, including validate-only and collision cases.

## [To Do] CTRL-F05 Topic deletion
P0 · M2 · Feature · deps: CTRL-F01
Scope: Delete by name or id, partition record removal, broker-side cleanup trigger.

### [To Do] CTRL-0056 Topic id and name lookup at the committed offset
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java findTopicIds, findTopicNames, findAllTopicIds @4.4.0
Done when: find_topic_ids returns INVALID_REQUEST for a null name and UNKNOWN_TOPIC_OR_PARTITION for unknown names; find_topic_names returns INVALID_REQUEST for null or zero ids and UNKNOWN_TOPIC_ID for unknown ids; find_all_topic_ids lists committed topics; uncommitted topics are invisible; tests.

### [To Do] CTRL-0057 DeleteTopics operation: RemoveTopicRecord per topic id
P0 · M2 · Task · deps: CTRL-0056 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java deleteTopics, deleteTopic @4.4.0
Done when: Each known topic id yields one RemoveTopicRecord; unknown ids fail UNKNOWN_TOPIC_ID; results are per id in one atomic result; an info log line summarizes results; tests.

### [To Do] CTRL-0058 RemoveTopicRecord replay cleans every index and the topic's configs
P0 · M2 · Task · deps: CTRL-0057 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java replay @4.4.0
Done when: Replay removes the topic from id and name maps, the collision index, the reassigning set, ISR and ELR indexes, the imbalanced set and the directory index, and deletes its config records; an unknown id is a fatal replay error; tests check every index.

### [To Do] CTRL-0059 Controller mutation quota on topic deletion
P0 · M4 · Task · deps: CTRL-0057, QUOTA-F05 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java deleteTopic @4.4.0
Done when: Each topic's partition count is charged; when exceeded the topic fails THROTTLING_QUOTA_EXCEEDED with throttle time and no record; tests.

### [To Do] CTRL-0060 Delete then recreate a topic with the same name gets a new id and fresh partitions
P0 · M2 · Task · deps: CTRL-0058, CTRL-F04, REPL-F12 · labels: test
Done when: A test deletes a topic and recreates it; the new topic has a new id, brokers stop and delete the old replicas (REPL-F12) and serve the new partitions empty.

## [To Do] CTRL-F06 Partition creation
P0 · M2 · Feature · deps: CTRL-F04
Scope: CreatePartitions with assignments and validation.

### [To Do] CTRL-0061 Reject CreatePartitions requests adding more than 10000 partitions
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java validateTotalNumberOfPartitions @4.4.0
Done when: The sum of additional partitions over existing topics above 10000 fails with POLICY_VIOLATION 'Excessively large number of additional partitions per request.'; tests.

### [To Do] CTRL-0062 CreatePartitions count and assignment validation
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createPartitions @4.4.0
Done when: Unknown topics fail UNKNOWN_TOPIC_OR_PARTITION; a count equal to the current fails INVALID_PARTITIONS 'Topic already has N partition(s).'; a smaller count fails INVALID_PARTITIONS; an assignment count different from the increase fails INVALID_REPLICA_ASSIGNMENT; tests per message.

### [To Do] CTRL-0063 New partition assignments: manual with the existing replication factor, or placed
P0 · M2 · Task · deps: CTRL-0062, CTRL-0047, CTRL-F07 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createPartitions, validateManualPartitionAssignment @4.4.0
Done when: Manual assignments are validated against the topic's replication factor and must include an active broker (else INVALID_REPLICA_ASSIGNMENT); otherwise the placer places from the next partition id; partitions whose replicas are all inactive fail INVALID_REPLICATION_FACTOR; tests.

### [To Do] CTRL-0064 PartitionRecords for new partitions, atomic per request, validate-only
P0 · M2 · Task · deps: CTRL-0063, CTRL-0061 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createPartitions @4.4.0; metadata/src/main/java/org/apache/kafka/controller/QuorumController.java createPartitions @4.4.0
Done when: New PartitionRecords use consecutive ids from the current count; results are per topic and the whole request is one atomic result; validate_only produces no records; tests.

### [To Do] CTRL-0065 Controller mutation quota on partition creation
P0 · M4 · Task · deps: CTRL-0064, QUOTA-F05 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createPartitions @4.4.0
Done when: Additional partitions are charged; when exceeded the topic fails THROTTLING_QUOTA_EXCEEDED; tests.

### [To Do] CTRL-0066 Port Kafka's CreatePartitions controller tests
P0 · M2 · Task · deps: CTRL-0064 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/ReplicationControlManagerTest.java @4.4.0
Done when: createPartitions cases from ReplicationControlManagerTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F07 Replica placement
P0 · M2 · Feature · deps: CTRL-F01
Scope: Striped, rack-aware placement equivalent to Kafka's placer.

### [To Do] CTRL-0067 Placement interfaces: PlacementSpec, ClusterDescriber, UsableBroker, TopicAssignment
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/metadata/placement/PlacementSpec.java @4.4.0; metadata/src/main/java/org/apache/kafka/metadata/placement/ClusterDescriber.java @4.4.0; metadata/src/main/java/org/apache/kafka/metadata/placement/UsableBroker.java @4.4.0; metadata/src/main/java/org/apache/kafka/metadata/placement/TopicAssignment.java @4.4.0
Done when: Types mirror Kafka's placement API (start partition, count, replication factor; usable brokers with rack and fenced flag; per-partition replica lists and directories); module docs explain the goals and constraints.

### [To Do] CTRL-0068 Rack list: racks sorted by name, fenced and unfenced broker lists, seeded start offsets
P0 · M2 · Task · deps: CTRL-0067 · ref: metadata/src/main/java/org/apache/kafka/metadata/placement/StripedReplicaPlacer.java RackList, Rack @4.4.0
Done when: Brokers without a rack form one rack; within each rack fenced and unfenced brokers are kept in sorted lists; the rack list and each rack get starting offsets from the injected RNG; tests with fixed seeds.

### [To Do] CTRL-0069 Striped placement: round-robin racks per partition, unfenced before fenced, unfenced leader
P0 · M2 · Task · deps: CTRL-0068 · ref: metadata/src/main/java/org/apache/kafka/metadata/placement/StripedReplicaPlacer.java place, RackList place @4.4.0
Done when: Each partition takes replicas from successive racks starting at an offset that advances per partition, round-robin within a rack, unfenced brokers before fenced ones, never the same broker twice, with the first replica always unfenced; golden tests reproduce Kafka's examples for fixed offsets.

### [To Do] CTRL-0070 Placement constraint errors
P0 · M2 · Task · deps: CTRL-0069 · ref: metadata/src/main/java/org/apache/kafka/metadata/placement/StripedReplicaPlacer.java place @4.4.0
Done when: Replication factor <= 0 fails 'Invalid replication factor', no unfenced usable broker fails 'All brokers are currently fenced, or have all their log directories cordoned.', and a replication factor above the registered broker count fails with Kafka's message; all map to INVALID_REPLICATION_FACTOR; tests.

### [To Do] CTRL-0071 Property tests for placement balance and constraints
P0 · M2 · Task · deps: CTRL-0070 · labels: test
Done when: For random clusters and seeds: no duplicate broker per partition, the leader is unfenced, each partition spans min(RF, racks) racks when racks are equal-sized, and leader counts across unfenced brokers differ by at most one for counts that are multiples of the broker count.

### [To Do] CTRL-0072 Placement benchmark at scale
P2 · M2 · Task · deps: CTRL-0069 · labels: perf
Done when: A criterion benchmark places 100000 partitions with RF 3 over 1000 brokers in 50 racks; results are recorded and a regression threshold is set.

### [To Do] CTRL-0073 Port Kafka's StripedReplicaPlacer tests
P0 · M2 · Task · deps: CTRL-0070 · labels: test · ref: metadata/src/test/java/org/apache/kafka/metadata/placement/StripedReplicaPlacerTest.java @4.4.0
Done when: Cases from StripedReplicaPlacerTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F08 Leader election
P0 · M3 · Feature · deps: CTRL-F02
Scope: Preferred, unclean and ELR-based (KIP-966) elections, effective min ISR and ELR bookkeeping, ElectLeaders, automatic leader rebalance.

### [To Do] CTRL-0074 Preferred election: preferred replica, current leader, other ISR, last known leader
P0 · M3 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/PartitionChangeBuilder.java electPreferredLeader, isValidNewLeader @4.4.0
Done when: Election.PREFERRED picks the first replica if it is a valid leader, else keeps a valid current leader, else the next valid replica, else the last known ELR leader when allowed, else no leader; valid means in the ISR (or ELR when the ISR is empty) and acceptable; tests per branch.

### [To Do] CTRL-0075 Unclean election: any acceptable replica, ISR reset to the leader, RECOVERING state
P0 · M3 · Task · deps: CTRL-0074 · ref: metadata/src/main/java/org/apache/kafka/controller/PartitionChangeBuilder.java electAnyLeader, tryElection @4.4.0
Done when: Election.UNCLEAN falls back to the first acceptable replica outside the ISR; the ISR becomes [leader], the leader recovery state becomes RECOVERING, ELR fields are cleared, and the change is logged at info and counted for UncleanLeaderElectionsPerSec; tests.

### [To Do] CTRL-0076 Effective min ISR per topic
P0 · M3 · Task · deps: CTRL-F11 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java getTopicEffectiveMinIsr @4.4.0
Done when: The effective min ISR is min(topic min.insync.replicas resolved through topic, cluster and static defaults, replication factor); tests.

### [To Do] CTRL-0077 ELR maintenance in partition changes (KIP-966)
P0 · M3 · Task · deps: CTRL-0076, CTRL-0031 · ref: metadata/src/main/java/org/apache/kafka/controller/PartitionChangeBuilder.java maybePopulateTargetElr, maybeUpdateRecordElr @4.4.0
Done when: With ELR enabled: an ISR at or above min ISR clears ELR and last known ELR; below min ISR the ELR becomes (old ISR plus ELR) minus the new ISR minus unclean-shutdown replicas, and last known ELR collects the others; an unclean election or disabled ELR clears both; the ISR may become empty; property tests against a model.

### [To Do] CTRL-0078 Electing from eligible leader replicas when the ISR is empty (KIP-966)
P0 · M3 · Task · deps: CTRL-0074, CTRL-0077 · ref: metadata/src/main/java/org/apache/kafka/controller/PartitionChangeBuilder.java tryElection @4.4.0; KIP-966
Done when: With ELR enabled and an empty ISR, an acceptable ELR member becomes leader, the ISR becomes [leader] and it leaves the ELR; counted for ElectionFromEligibleLeaderReplicasPerSec; tests.

### [To Do] CTRL-0079 Last known leader recovery when ISR and ELR are empty
P0 · M3 · Task · deps: CTRL-0078 · ref: metadata/src/main/java/org/apache/kafka/controller/PartitionChangeBuilder.java canElectLastKnownLeader, maybeUpdateLastKnownLeader @4.4.0
Done when: With ELR and last-known-leader recovery enabled, a partition whose ISR and ELR are empty can elect its single last-known-ELR member if acceptable (unclean); the last known leader is recorded the first time a partition becomes leaderless and cleared once it has a leader; tests.

### [To Do] CTRL-0080 Leader epoch bumps for replica removal and for ISR shrink on old metadata versions
P0 · M3 · Task · deps: CTRL-0031 · ref: metadata/src/main/java/org/apache/kafka/controller/PartitionChangeBuilder.java triggerLeaderEpochBumpForReplicaReassignmentIfNeeded, triggerLeaderEpochBumpForIsrShrinkIfNeeded @4.4.0
Done when: A change that removes replicas present before forces a leader epoch bump even without a leader change; an ISR shrink forces one when the metadata.version requires it (before 3.6); expansions never do; tests per metadata.version.

### [To Do] CTRL-0081 ElectLeaders operation for preferred and unclean elections
P0 · M3 · Task · deps: CTRL-0075 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java electLeaders, electLeader @4.4.0
Done when: An unknown election type fails INVALID_REQUEST; null partitions means all partitions with ELECTION_NOT_NEEDED results omitted; per partition: UNKNOWN_TOPIC_OR_PARTITION, ELECTION_NOT_NEEDED (preferred already leader, or unclean with a leader), PREFERRED_LEADER_NOT_AVAILABLE or ELIGIBLE_LEADERS_NOT_AVAILABLE when no record results; records are bounded per operation; tests per error.

### [To Do] CTRL-0082 Imbalanced partition tracking on replay
P0 · M3 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java imbalancedPartitions, arePartitionLeadersImbalanced @4.4.0
Done when: A partition whose leader is not its preferred replica is in the imbalanced set; the set updates on PartitionRecord, PartitionChangeRecord and RemoveTopicRecord replay and feeds PreferredReplicaImbalanceCount; tests.

### [To Do] CTRL-0083 Automatic preferred leader rebalance
P0 · M3 · Task · deps: CTRL-0082, CTRL-0074, CTRL-0017 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java maybeBalancePartitionLeaders @4.4.0; metadata/src/main/java/org/apache/kafka/controller/QuorumController.java registerElectPreferred @4.4.0
Done when: With auto.leader.rebalance.enable, a periodic task every leader.imbalance.check.interval.seconds runs preferred elections for at most 1000 imbalanced partitions per run and reschedules immediately when the limit was hit; disabled config registers no task; tests.

### [To Do] CTRL-0084 Periodic unclean election for leaderless partitions
P0 · M3 · Task · deps: CTRL-0075, CTRL-0017, CTRL-F11 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java maybeElectUncleanLeaders @4.4.0; metadata/src/main/java/org/apache/kafka/controller/QuorumController.java registerElectUnclean @4.4.0; metadata/src/main/java/org/apache/kafka/controller/ConfigurationControlManager.java uncleanLeaderElectionEnabledForTopic @4.4.0
Done when: A periodic task every unclean.leader.election.interval.ms elects leaders for leaderless partitions of topics whose unclean.leader.election.enable (topic or cluster default, dynamic) is true, bounded per run; tests.

### [To Do] CTRL-0085 Fencing uses unclean election for topics that allow it
P0 · M3 · Task · deps: CTRL-0075, CTRL-F11 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java generateLeaderAndIsrUpdates @4.4.0
Done when: When a fence or ISR change leaves a partition without a valid clean leader, topics with unclean election enabled elect uncleanly in the same change; tests.

### [To Do] CTRL-0086 Port Kafka's election tests
P0 · M3 · Task · deps: CTRL-0081, CTRL-0083, CTRL-0084, CTRL-0079, CTRL-0080, CTRL-0085 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/PartitionChangeBuilderTest.java @4.4.0
Done when: Election cases from PartitionChangeBuilderTest and ReplicationControlManagerTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F09 AlterPartition handling
P0 · M3 · Feature · deps: CTRL-F08
Scope: ISR changes with epoch validation; ELR records and the config rules that change them; min ISR semantics on AlterPartition.

### [To Do] CTRL-0087 AlterPartition broker epoch and topic id checks
P0 · M3 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java alterPartition @4.4.0
Done when: A request from an unknown broker or with a stale broker epoch fails STALE_BROKER_EPOCH; a null, zero or unknown topic id fails UNKNOWN_TOPIC_ID for every partition of that topic; v2 new_isr lists are converted to entries with broker epoch -1; tests.

### [To Do] CTRL-0088 AlterPartition partition validation chain
P0 · M3 · Task · deps: CTRL-0087 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java validateAlterPartitionData @4.4.0
Done when: In Kafka's order: unknown partition UNKNOWN_TOPIC_OR_PARTITION; request leader or partition epoch above the current NOT_CONTROLLER; lower leader epoch FENCED_LEADER_EPOCH; sender not the leader INVALID_REQUEST; lower partition epoch INVALID_UPDATE_VERSION; ISR not within replicas or missing the leader INVALID_REQUEST; RECOVERING with more than one ISR member or RECOVERED to RECOVERING INVALID_REQUEST; a test per rule.

### [To Do] CTRL-0089 Ineligible ISR members fail INELIGIBLE_REPLICA
P0 · M3 · Task · deps: CTRL-0088 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java ineligibleReplicasForIsr @4.4.0
Done when: Proposed ISR members that are not registered, in controlled shutdown, fenced, or whose broker epoch (unless -1) differs from their registration fail the partition with INELIGIBLE_REPLICA, logging the reasons; tests per reason.

### [To Do] CTRL-0090 Apply the ISR change and build the AlterPartition response
P0 · M3 · Task · deps: CTRL-0089, CTRL-0031 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java alterPartition @4.4.0
Done when: Valid changes go through the partition change builder (unclean when the topic allows); the response carries leader, ISR, leader recovery state, leader epoch and partition epoch (current values for no-ops); tests for expansion, shrink and no-op.

### [To Do] CTRL-0091 NEW_LEADER_ELECTED when an ISR change completes a reassignment and moves leadership
P0 · M3 · Task · deps: CTRL-0090, CTRL-0099 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java alterPartition @4.4.0
Done when: If the change completes an ongoing reassignment and the leader becomes another broker, the partition gets NEW_LEADER_ELECTED while the record is still written; tests.

### [To Do] CTRL-0092 Leader recovery state RECOVERING to RECOVERED via AlterPartition
P0 · M3 · Task · deps: CTRL-0090 · ref: metadata/src/main/java/org/apache/kafka/controller/PartitionChangeBuilder.java setTargetLeaderRecoveryState @4.4.0; metadata/src/main/java/org/apache/kafka/metadata/LeaderRecoveryState.java @4.4.0
Done when: A leader elected uncleanly reports RECOVERED through AlterPartition and the change record updates the state; RECOVERING requests with a single-member ISR are accepted; tests.

### [To Do] CTRL-0093 Broker to ELR index maintained on replay
P0 · M3 · Task · deps: CTRL-0077 · ref: metadata/src/main/java/org/apache/kafka/controller/BrokersToElrs.java @4.4.0
Done when: The index of partitions per ELR member updates on partition records and topic removal and is used when brokers are unregistered or shut down uncleanly; tests.

### [To Do] CTRL-0094 Unclean broker shutdown removes the broker from ISR and ELR
P0 · M3 · Task · deps: CTRL-0093, CTRL-0034 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java handleBrokerShutdown @4.4.0; metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java registerBroker @4.4.0
Done when: With ELR enabled, a new incarnation whose previous broker epoch does not match the stored epoch is treated as an unclean shutdown and the broker is removed from every ISR and ELR; a clean shutdown is handled like a fence; tests for both.

### [To Do] CTRL-0095 ClearElrRecord when min.insync.replicas changes
P0 · M3 · Task · deps: CTRL-0093, CTRL-F11 · ref: metadata/src/main/java/org/apache/kafka/controller/ConfigurationControlManager.java createClearElrRecordsAsNeeded @4.4.0; metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java replay @4.4.0; metadata/src/main/resources/common/metadata/ClearElrRecord.json @4.4.0
Done when: Changing min.insync.replicas on a topic emits ClearElrRecord for that topic; on a broker default or cluster resource it emits a ClearElrRecord for all topics; replay clears ELR and last known ELR accordingly; tests.

### [To Do] CTRL-0096 min.insync.replicas restrictions and safety records when ELR is enabled
P0 · M3 · Task · deps: CTRL-0095, CTRL-F17 · ref: metadata/src/main/java/org/apache/kafka/controller/ConfigurationControlManager.java isDisallowedBrokerMinIsrTransition, maybeGenerateElrSafetyRecords @4.4.0
Done when: With ELR enabled, broker-level min.insync.replicas changes and removing the cluster-level value fail INVALID_CONFIG; enabling eligible.leader.replicas.version writes a cluster-level min.insync.replicas from the static config and removes broker-level overrides; tests.

### [To Do] CTRL-0097 ELR feature gating by eligible.leader.replicas.version
P0 · M3 · Task · deps: CTRL-0077, CTRL-F17 · ref: server-common/src/main/java/org/apache/kafka/server/common/EligibleLeaderReplicasVersion.java @4.4.0; metadata/src/main/java/org/apache/kafka/controller/FeatureControlManager.java isElrFeatureEnabled @4.4.0
Done when: ELR fields are written in PartitionRecord and PartitionChangeRecord only when the feature is finalized at >= 1; downgrading it clears all ELRs; tests.

### [To Do] CTRL-0098 Port Kafka's AlterPartition and ELR controller tests
P0 · M3 · Task · deps: CTRL-0091, CTRL-0092, CTRL-0094, CTRL-0096, CTRL-0097 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/ReplicationControlManagerTest.java @4.4.0
Done when: AlterPartition and ELR cases from ReplicationControlManagerTest and PartitionChangeBuilderTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F10 Partition reassignment
P0 · M3 · Feature · deps: CTRL-F08
Scope: Alter/ListPartitionReassignments, adding and removing replicas, cancellation.

### [To Do] CTRL-0099 Reassignment replica sets: adding, removing and the merged target list
P0 · M3 · Task · deps: none · labels: foundation-for:X-AUTO · ref: metadata/src/main/java/org/apache/kafka/controller/PartitionReassignmentReplicas.java @4.4.0
Done when: From the current and target assignments compute adding, removing and the combined replica list (merging with an ongoing reassignment); completion requires all adding replicas in the ISR, a non-empty ISR after removing, and when shrinking the replication factor all target replicas in the ISR; property tests.

### [To Do] CTRL-0100 AlterPartitionReassignments: per-partition validation and change records
P0 · M3 · Task · deps: CTRL-0099, CTRL-0047 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java alterPartitionReassignments, changePartitionReassignment, validatePartitionReplicationFactorUnchanged @4.4.0
Done when: Unknown topics or partitions fail UNKNOWN_TOPIC_OR_PARTITION; targets are validated like manual assignments (INVALID_REPLICA_ASSIGNMENT); without allow_replication_factor_change a different replica count fails INVALID_REPLICATION_FACTOR; valid targets write a PartitionChangeRecord with replicas, adding and removing; results are per partition in one atomic result; tests.

### [To Do] CTRL-0101 Cancel a reassignment and revert replicas and ISR
P0 · M3 · Task · deps: CTRL-0100 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java cancelPartitionReassignment @4.4.0; metadata/src/main/java/org/apache/kafka/controller/PartitionReassignmentRevert.java @4.4.0
Done when: A null target cancels: no reassignment fails NO_REASSIGNMENT_IN_PROGRESS; the revert restores the original replicas and an ISR without adding replicas, and a revert needing an unclean election fails INVALID_REPLICA_ASSIGNMENT unless the topic allows unclean election; tests.

### [To Do] CTRL-0102 Completing reassignments when the ISR catches up
P0 · M3 · Task · deps: CTRL-0100, CTRL-0080 · ref: metadata/src/main/java/org/apache/kafka/controller/PartitionChangeBuilder.java completeReassignmentIfNeeded @4.4.0
Done when: When an ISR change makes the reassignment complete, the same change record drops removing replicas from replicas and ISR, clears adding/removing, elects a new leader if the current one is removed and bumps the leader epoch; tests through AlterPartition and through unfencing.

### [To Do] CTRL-0103 ListPartitionReassignments at the committed offset
P0 · M3 · Task · deps: CTRL-0104 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java listPartitionReassignments @4.4.0
Done when: Null topics lists every reassigning topic; named topics that do not exist are omitted; only partitions with a reassignment in progress are returned with replicas, adding and removing; tests.

### [To Do] CTRL-0104 Reassigning topics index maintained on replay
P0 · M3 · Task · deps: CTRL-0099 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java replay @4.4.0
Done when: The topic to reassigning-partitions index updates on partition records, change records and topic removal and feeds the reassigning partitions count; tests.

### [To Do] CTRL-0105 Port Kafka's reassignment controller tests
P0 · M3 · Task · deps: CTRL-0101, CTRL-0102, CTRL-0103 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/PartitionReassignmentReplicasTest.java @4.4.0
Done when: Reassignment cases from ReplicationControlManagerTest and PartitionReassignmentReplicasTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F11 Configuration records
P0 · M2 · Feature · deps: CTRL-F01, CFG-F05
Scope: Validate and write config changes; create and alter policies.

### [To Do] CTRL-0106 ConfigRecord replay per resource with deletion by null value
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ConfigurationControlManager.java replay @4.4.0; metadata/src/main/resources/common/metadata/ConfigRecord.json @4.4.0
Done when: Configs are stored per (resource type, name) for BROKER (including the '' cluster default), TOPIC, CLIENT_METRICS and GROUP; a null value deletes; topic configs are removed when the topic is deleted; tests.

### [To Do] CTRL-0107 IncrementalAlterConfigs operations SET, DELETE, APPEND and SUBTRACT
P0 · M2 · Task · deps: CTRL-0106 · ref: metadata/src/main/java/org/apache/kafka/controller/ConfigurationControlManager.java incrementalAlterConfigs, incrementalAlterConfigResource @4.4.0
Done when: SET and DELETE apply to any key; APPEND and SUBTRACT only to list-type keys (INVALID_CONFIG 'Can't APPEND to key' otherwise); records are written even when the value is unchanged; appending past the value size limit fails INVALID_CONFIG; per-resource results; tests per op.

### [To Do] CTRL-0108 Legacy AlterConfigs full-replacement semantics
P0 · M2 · Task · deps: CTRL-0107 · ref: metadata/src/main/java/org/apache/kafka/controller/ConfigurationControlManager.java legacyAlterConfigs, legacyAlterConfigResource @4.4.0
Done when: Keys absent from the request are deleted, and implicit deletions are not passed to the alter policy; per-resource results; tests.

### [To Do] CTRL-0109 Config resource existence checks
P0 · M2 · Task · deps: CTRL-0106 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java ConfigResourceExistenceChecker @4.4.0
Done when: BROKER resources must be '' or a numeric id (INVALID_REQUEST) of a registered broker (BROKER_ID_NOT_REGISTERED); TOPIC resources must exist (UNKNOWN_TOPIC_OR_PARTITION) except during creation; tests.

### [To Do] CTRL-0110 Topic config validation through the config schema
P0 · M2 · Task · deps: CTRL-0107 · ref: core/src/main/scala/kafka/server/ControllerConfigurationValidator.scala @4.4.0; metadata/src/main/java/org/apache/kafka/controller/ConfigurationValidator.java @4.4.0
Done when: Unknown topic keys and invalid values (type, range, list, enum) fail INVALID_CONFIG with the validator's message using CFG-F05 definitions; tests for representative keys.

### [To Do] CTRL-0111 Alter config policy hook
P0 · M2 · Task · deps: CTRL-0110, CTRL-0049 · ref: metadata/src/main/java/org/apache/kafka/controller/ConfigurationControlManager.java validateAlterConfig @4.4.0
Done when: When configured, the policy receives the resource and the resulting configs; violations fail POLICY_VIOLATION; tests with a test policy.

### [To Do] CTRL-0112 validate_only for config changes
P0 · M2 · Task · deps: CTRL-0107, CTRL-0108 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java incrementalAlterConfigs @4.4.0
Done when: validate_only runs every check and returns the same results without records; tests.

### [To Do] CTRL-0113 DescribeConfigs read operation on the controller
P0 · M2 · Task · deps: CTRL-0106 · ref: metadata/src/main/java/org/apache/kafka/controller/ConfigurationControlManager.java describeConfigs @4.4.0
Done when: Describing resources returns the committed configured values per resource (missing resources fail per resource with Kafka's errors); synonym and source semantics come from CFG-F06; tests.

### [To Do] CTRL-0114 Dynamic broker config validation on the controller
P0 · M7 · Task · deps: CTRL-0110, CFG-F04, SEC-F11 · ref: core/src/main/scala/kafka/server/ControllerConfigurationValidator.scala @4.4.0
Done when: Broker and cluster-default config changes are validated with CFG-F04's dynamic config rules (read-only keys and invalid values fail INVALID_CONFIG; password values are stored encrypted per SEC-F11); tests.

### [To Do] CTRL-0115 Port Kafka's configuration controller tests
P0 · M2 · Task · deps: CTRL-0111, CTRL-0112, CTRL-0109, CTRL-0113 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/ConfigurationControlManagerTest.java @4.4.0
Done when: Cases from ConfigurationControlManagerTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F12 Client quota records
P0 · M4 · Feature · deps: CTRL-F01
Scope: Quota entity validation and records.

### [To Do] CTRL-0116 Client quota entity validation
P0 · M4 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ClientQuotaControlManager.java validateEntity @4.4.0
Done when: Entities must be non-empty, have no duplicate entity types, use only user, client-id and ip, not combine ip with others, have valid IPs or resolvable hosts for ip, and not use empty names; each violation fails INVALID_REQUEST with Kafka's message; tests per rule.

### [To Do] CTRL-0117 Client quota key and value validation
P0 · M4 · Task · deps: CTRL-0116 · ref: metadata/src/main/java/org/apache/kafka/controller/ClientQuotaControlManager.java alterClientQuotas @4.4.0
Done when: Keys must be valid for the entity (connection_creation_rate only for ip; producer_byte_rate, consumer_byte_rate, request_percentage, controller_mutation_rate otherwise); values must be finite and > 0, integer-only keys reject fractions; INVALID_REQUEST messages match Kafka; tests per key.

### [To Do] CTRL-0118 ClientQuotaRecord generation, removal, validate-only and replay
P0 · M4 · Task · deps: CTRL-0117 · ref: metadata/src/main/java/org/apache/kafka/controller/ClientQuotaControlManager.java replay @4.4.0; metadata/src/main/resources/common/metadata/ClientQuotaRecord.json @4.4.0
Done when: Set and remove operations write ClientQuotaRecord (remove = true) per key; validate_only writes nothing; replay maintains the quota map per entity; per-entity results; tests.

### [To Do] CTRL-0119 Port Kafka's client quota controller tests
P0 · M4 · Task · deps: CTRL-0118 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/ClientQuotaControlManagerTest.java @4.4.0
Done when: Cases from ClientQuotaControlManagerTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F13 ACL records
P0 · M4 · Feature · deps: CTRL-F01
Scope: ACL creation and deletion with validation and ids.

### [To Do] CTRL-0120 ACL binding validation
P0 · M4 · Task · deps: SEC-F01 · ref: metadata/src/main/java/org/apache/kafka/controller/AclControlManager.java validateNewAcl, validateHostPattern @4.4.0
Done when: Resource type, pattern type (LITERAL or PREFIXED), operation and permission must be concrete (INVALID_REQUEST), the resource name non-empty, the principal parseable, the host non-empty; CIDR host patterns need metadata.version 4.4-IV1 (UNSUPPORTED_VERSION) and valid notation (INVALID_REQUEST); tests per rule.

### [To Do] CTRL-0121 CreateAcls: ids from the injected RNG, duplicates skipped
P0 · M4 · Task · deps: CTRL-0120 · ref: metadata/src/main/java/org/apache/kafka/controller/AclControlManager.java createAcls @4.4.0; metadata/src/main/resources/common/metadata/AccessControlEntryRecord.json @4.4.0
Done when: Each new binding gets a unique id from the seeded RNG and an AccessControlEntryRecord; a binding identical to an existing one succeeds without a record; per-creation results; tests.

### [To Do] CTRL-0122 DeleteAcls: filter validation, matching and bounded deletions
P0 · M4 · Task · deps: CTRL-0121 · ref: metadata/src/main/java/org/apache/kafka/controller/AclControlManager.java deleteAcls, validateFilter @4.4.0
Done when: Unknown pattern or entry filters fail INVALID_REQUEST; matching bindings get RemoveAccessControlEntryRecord; more than 10000 matches fails the whole request with INVALID_REQUEST rather than partial results; per-filter results list deleted bindings; tests.

### [To Do] CTRL-0123 ACL record replay with integrity checks
P0 · M4 · Task · deps: CTRL-0121 · ref: metadata/src/main/java/org/apache/kafka/controller/AclControlManager.java replay @4.4.0
Done when: Replay adds and removes ACLs by id; a duplicate id or a removal of an unknown id is a fatal replay error; tests.

### [To Do] CTRL-0124 Port Kafka's ACL controller tests
P0 · M4 · Task · deps: CTRL-0122, CTRL-0123 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/AclControlManagerTest.java @4.4.0
Done when: Cases from AclControlManagerTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F14 SCRAM credential records
P0 · M4 · Feature · deps: CTRL-F01
Scope: User SCRAM credential upserts and deletions.

### [To Do] CTRL-0125 AlterUserScramCredentials validation
P0 · M4 · Task · deps: none · labels: security · ref: metadata/src/main/java/org/apache/kafka/controller/ScramControlManager.java alterCredentials, validateUpsertion @4.4.0
Done when: A user appearing twice across deletions and upsertions fails DUPLICATE_RESOURCE for that user; an empty user fails UNACCEPTABLE_CREDENTIAL; an unknown mechanism fails UNSUPPORTED_SASL_MECHANISM; iterations below the mechanism minimum (4096) or above 16384 fail UNACCEPTABLE_CREDENTIAL; deleting a missing credential fails RESOURCE_NOT_FOUND; an unsupported metadata.version fails UNSUPPORTED_VERSION; tests per rule.

### [To Do] CTRL-0126 SCRAM credential records and replay
P0 · M4 · Task · deps: CTRL-0125 · labels: security · ref: metadata/src/main/java/org/apache/kafka/controller/ScramControlManager.java replay @4.4.0; metadata/src/main/resources/common/metadata/UserScramCredentialRecord.json @4.4.0
Done when: Upsertions write UserScramCredentialRecord (name, mechanism, salt, salted password, iterations); deletions write RemoveUserScramCredentialRecord; replay maintains credentials per (user, mechanism); a removal of a missing credential is a fatal replay error; tests.

### [To Do] CTRL-0127 SCRAM material never appears in logs, debug output or events
P0 · M4 · Task · deps: CTRL-0126, TYPE-F08 · labels: security, test
Done when: Salt and salted password are held as Secret values in the controller and records' Debug output is redacted; a redaction test over logs and flight recorder output finds no credential bytes.

### [To Do] CTRL-0128 Port Kafka's SCRAM controller tests
P0 · M4 · Task · deps: CTRL-0126 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/QuorumControllerTest.java @4.4.0; metadata/src/test/java/org/apache/kafka/controller/ScramCredentialValueTest.java @4.4.0
Done when: SCRAM cases from QuorumControllerTest and ScramCredentialValueTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F15 Delegation token records
P1 · M4 · Feature · deps: CTRL-F01
Scope: Token creation, renewal and expiry records.

### [To Do] CTRL-0129 CreateDelegationToken: enablement, owner type, lifetime and HMAC
P1 · M4 · Task · deps: none · labels: security · ref: metadata/src/main/java/org/apache/kafka/controller/DelegationTokenControlManager.java createDelegationToken @4.4.0
Done when: Without delegation.token.secret.key tokens fail DELEGATION_TOKEN_AUTH_DISABLED; an unsupported metadata.version fails UNSUPPORTED_VERSION; a non-User owner fails INVALID_PRINCIPAL_TYPE; the token id comes from the seeded RNG, the HMAC-SHA512 is keyed by the secret, max lifetime is min(request, delegation.token.max.lifetime.ms) and expiry min(now + delegation.token.expiry.time.ms, max); a DelegationTokenRecord is written; tests.

### [To Do] CTRL-0130 RenewDelegationToken validation and new expiry
P1 · M4 · Task · deps: CTRL-0129 · labels: security · ref: metadata/src/main/java/org/apache/kafka/controller/DelegationTokenControlManager.java renewDelegationToken @4.4.0
Done when: Unknown tokens fail DELEGATION_TOKEN_NOT_FOUND, expired tokens DELEGATION_TOKEN_EXPIRED, renewers other than owner and listed renewers DELEGATION_TOKEN_OWNER_MISMATCH; the new expiry is min(now + renew period, max timestamp); tests.

### [To Do] CTRL-0131 ExpireDelegationToken: immediate removal or new expiry
P1 · M4 · Task · deps: CTRL-0129 · labels: security · ref: metadata/src/main/java/org/apache/kafka/controller/DelegationTokenControlManager.java expireDelegationToken @4.4.0
Done when: Unknown tokens fail DELEGATION_TOKEN_NOT_FOUND, mismatched principals DELEGATION_TOKEN_OWNER_MISMATCH; a negative period removes the token (RemoveDelegationTokenRecord); otherwise the expiry is updated, and an already expired token fails DELEGATION_TOKEN_EXPIRED; tests.

### [To Do] CTRL-0132 Periodic sweep of expired delegation tokens
P1 · M4 · Task · deps: CTRL-0131, CTRL-0017 · ref: metadata/src/main/java/org/apache/kafka/controller/DelegationTokenControlManager.java sweepExpiredDelegationTokens @4.4.0; metadata/src/main/java/org/apache/kafka/controller/QuorumController.java registerExpireDelegationTokens @4.4.0
Done when: Every delegation.token.expiry.check.interval.ms the active controller removes expired tokens with RemoveDelegationTokenRecord, bounded per run; tests with the simulated clock.

### [To Do] CTRL-0133 Delegation token record replay; HMACs and secrets redacted
P1 · M4 · Task · deps: CTRL-0129 · labels: security · ref: metadata/src/main/java/org/apache/kafka/controller/DelegationTokenControlManager.java replay @4.4.0; metadata/src/main/resources/common/metadata/DelegationTokenRecord.json @4.4.0
Done when: Replay maintains tokens by id; HMAC values and the secret key are Secret-wrapped and never logged; tests including a redaction check.

### [To Do] CTRL-0134 Port Kafka's delegation token controller tests
P1 · M4 · Task · deps: CTRL-0130, CTRL-0132, CTRL-0133 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/QuorumControllerTest.java @4.4.0
Done when: Delegation token cases from QuorumControllerTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F16 Producer id allocation
P0 · M6 · Feature · deps: CTRL-F01
Scope: Block allocation via AllocateProducerIds.

### [To Do] CTRL-0135 AllocateProducerIds: broker epoch check and 1000-id blocks
P0 · M6 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ProducerIdControlManager.java generateNextProducerId @4.4.0; server-common/src/main/java/org/apache/kafka/server/common/ProducerIdsBlock.java @4.4.0
Done when: A stale broker epoch fails STALE_BROKER_EPOCH; the next block starts at the stored next producer id with size 1000; exhausting the id space fails UNKNOWN_SERVER_ERROR; a ProducerIdsRecord (broker id, broker epoch, next producer id) is written; tests.

### [To Do] CTRL-0136 ProducerIdsRecord replay with monotonicity check
P0 · M6 · Task · deps: CTRL-0135 · ref: metadata/src/main/java/org/apache/kafka/controller/ProducerIdControlManager.java replay @4.4.0; metadata/src/main/resources/common/metadata/ProducerIdsRecord.json @4.4.0
Done when: Replay sets the next producer id; a record that would move it backwards is a fatal replay error; tests.

### [To Do] CTRL-0137 Port Kafka's producer id controller tests
P0 · M6 · Task · deps: CTRL-0136 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/ProducerIdControlManagerTest.java @4.4.0
Done when: Cases from ProducerIdControlManagerTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F17 Feature management
P0 · M2 · Feature · deps: CTRL-F01, META-F06
Scope: UpdateFeatures, finalized feature levels, upgrades and downgrades.

### [To Do] CTRL-0138 FeatureLevelRecord replay and finalized feature view
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/FeatureControlManager.java replay @4.4.0; metadata/src/main/resources/common/metadata/FeatureLevelRecord.json @4.4.0
Done when: Replay sets or (level 0) removes finalized features; a metadata.version this controller does not support is a fatal replay error; the finalized features as of an offset are readable; tests.

### [To Do] CTRL-0139 UpdateFeatures per-feature validation and upgrade types
P0 · M2 · Task · deps: CTRL-0138 · ref: metadata/src/main/java/org/apache/kafka/controller/FeatureControlManager.java updateFeature, invalidUpdateVersion @4.4.0
Done when: Unknown upgrade types, negative levels, downgrades without SAFE_DOWNGRADE/UNSAFE_DOWNGRADE, 'downgrades' to a higher level, and unsupported downgrades fail INVALID_UPDATE_VERSION with Kafka's messages; tests per rule.

### [To Do] CTRL-0140 Cluster-wide support check for new feature levels
P0 · M2 · Task · deps: CTRL-0139, CTRL-0024 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumFeatures.java @4.4.0; metadata/src/main/java/org/apache/kafka/controller/ClusterFeatureSupportDescriber.java @4.4.0
Done when: A level is accepted only if the local controller, every registered controller and every registered broker support it; otherwise INVALID_UPDATE_VERSION names the unsupported nodes; tests.

### [To Do] CTRL-0141 Feature dependency validation
P0 · M2 · Task · deps: CTRL-0139 · ref: server-common/src/main/java/org/apache/kafka/server/common/Feature.java validateVersion @4.4.0
Done when: Feature levels that depend on other features (for example on metadata.version) are rejected with INVALID_UPDATE_VERSION when the dependency is not met, using Kafka's dependency table; tests per feature.

### [To Do] CTRL-0142 metadata.version upgrades and refused downgrades
P0 · M2 · Task · deps: CTRL-0139 · ref: metadata/src/main/java/org/apache/kafka/controller/FeatureControlManager.java updateMetadataVersion @4.4.0
Done when: metadata.version updates outside the supported range fail; downgrades are refused (unsafe downgrades unsupported; safe downgrades refused when metadata could be lost) with INVALID_UPDATE_VERSION; upgrades are logged at warn; tests.

### [To Do] CTRL-0143 UpdateFeatures results, records and validate-only
P0 · M2 · Task · deps: CTRL-0140, CTRL-0141, CTRL-0142 · ref: metadata/src/main/java/org/apache/kafka/controller/FeatureControlManager.java updateFeatures @4.4.0; metadata/src/main/java/org/apache/kafka/controller/QuorumController.java updateFeatures @4.4.0
Done when: Accepted updates write FeatureLevelRecord atomically; results are per feature with top-level error when any fails; validate_only writes nothing; tests.

### [To Do] CTRL-0144 kraft.version updates delegated to the raft client
P0 · M3 · Task · deps: CTRL-0143, RAFT-F07 · ref: metadata/src/main/java/org/apache/kafka/controller/QuorumController.java updateFeatures @4.4.0; metadata/src/main/java/org/apache/kafka/controller/RaftClientKRaftVersionAccessor.java @4.4.0
Done when: Updating kraft.version calls the raft upgrade (RAFT-F07) with the current epoch and validate_only; its errors map to INVALID_UPDATE_VERSION; tests.

### [To Do] CTRL-0145 Port Kafka's feature controller tests
P0 · M2 · Task · deps: CTRL-0143 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/FeatureControlManagerTest.java @4.4.0
Done when: Cases from FeatureControlManagerTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F18 Snapshot scheduling
P1 · M2 · Feature · deps: CTRL-F01, RAFT-F06
Scope: Snapshots by bytes since last snapshot and by maximum interval.

### [To Do] CTRL-0146 Snapshot trigger by bytes since the last snapshot
P1 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/image/publisher/SnapshotGenerator.java @4.4.0
Done when: After each committed batch the generator adds its size and requests a snapshot when the total reaches metadata.log.max.record.bytes.between.snapshots; tests.

### [To Do] CTRL-0147 Snapshot trigger by metadata.log.max.snapshot.interval.ms
P1 · M2 · Task · deps: CTRL-0146 · ref: metadata/src/main/java/org/apache/kafka/image/publisher/SnapshotGenerator.java @4.4.0
Done when: A snapshot is requested when the time since the last one exceeds the interval and new records exist; 0 disables the time trigger; tests with the simulated clock.

### [To Do] CTRL-0148 Snapshot emission at the committed image with one snapshot in flight
P1 · M2 · Task · deps: CTRL-0146, META-F05 · ref: metadata/src/main/java/org/apache/kafka/image/publisher/SnapshotEmitter.java @4.4.0; metadata/src/main/java/org/apache/kafka/image/publisher/SnapshotGenerator.java @4.4.0
Done when: A requested snapshot is written from the image at its committed offset through META-F05's emitter and the raft create_snapshot API; requests while one is in progress are coalesced; an image already snapshotted is skipped; tests.

### [To Do] CTRL-0149 Snapshot generation failure handling
P1 · M2 · Task · deps: CTRL-0148
Done when: A failure writing a snapshot is reported through the fault handler, logged with the snapshot id and does not stop later attempts; partial files are cleaned by RAFT-F06 recovery; tests with an injected disk fault.

### [To Do] CTRL-0150 Snapshot metrics: latest generated bytes and age
P1 · M2 · Task · deps: CTRL-0148, OBS-F04 · labels: observability · ref: metadata/src/main/java/org/apache/kafka/image/publisher/metrics/SnapshotEmitterMetrics.java @4.4.0
Done when: Gauges for the latest snapshot size and age (Kafka LatestSnapshotGeneratedBytes and LatestSnapshotGeneratedAgeMs) update after each emission; tests.

## [To Do] CTRL-F19 Controller metrics and events
P1 · M2 · Feature · deps: CTRL-F01, OBS-F04
Scope: Active controller, offline partitions, fenced brokers, event queue timings, events with reasons.

### [To Do] CTRL-0151 Controller event catalog with reasons
P1 · M2 · Task · deps: OBS-F07 · labels: observability
Done when: alluvium-controller's events.rs lists controller_activated, controller_renounced, broker_registered, broker_fenced, broker_unfenced, broker_controlled_shutdown, broker_unregistered, topic_created, topic_deleted, partition_leader_elected (reason: preferred, online, unclean, elr, reassignment) and isr_changed; the driver logs them; tests assert emission.

### [To Do] CTRL-0152 Controller flight recorder
P1 · M2 · Task · deps: OBS-F06 · labels: observability
Done when: The driver records activation, renounce, event timeouts, fault classifications and broker state changes in a 256-entry flight recorder dumped on fatal errors and readable through the diagnostics endpoint; tests.

### [To Do] CTRL-0153 Metrics ActiveControllerCount and NewActiveControllersCount
P1 · M2 · Task · deps: none · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/QuorumControllerMetrics.java @4.4.0
Done when: ActiveControllerCount is 1 only on the active controller and NewActiveControllersCount counts activations; tests across failover.

### [To Do] CTRL-0154 Metrics EventQueueTimeMs and EventQueueProcessingTimeMs
P1 · M2 · Task · deps: none · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/QuorumControllerMetrics.java updateEventQueueTime, updateEventQueueProcessingTime @4.4.0
Done when: Histograms record per-event queue wait and processing time; tests with the simulated clock.

### [To Do] CTRL-0155 Metrics EventQueueOperationsStartedCount and EventQueueOperationsTimedOutCount
P1 · M2 · Task · deps: none · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/QuorumControllerMetrics.java @4.4.0
Done when: Counters increment when an event starts and when one times out; tests.

### [To Do] CTRL-0156 Metric AvgIdleRatio for the controller event loop
P2 · M2 · Task · deps: none · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/QuorumControllerMetrics.java @4.4.0
Done when: The ratio of idle to total time of the event loop is exported; tests.

### [To Do] CTRL-0157 Metrics LastAppliedRecordOffset, LastCommittedRecordOffset, LastAppliedRecordTimestamp, LastAppliedRecordLagMs
P1 · M2 · Task · deps: none · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/QuorumControllerMetrics.java @4.4.0
Done when: Gauges reflect the offset control state and record timestamp lag; tests.

### [To Do] CTRL-0158 Metric TimedOutBrokerHeartbeatCount
P1 · M2 · Task · deps: none · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/QuorumControllerMetrics.java @4.4.0
Done when: The counter increments for heartbeats expired in the queue; tests.

### [To Do] CTRL-0159 Metrics FencedBrokerCount, ActiveBrokerCount and ControlledShutdownBrokerCount
P1 · M2 · Task · deps: none · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/ControllerMetadataMetrics.java @4.4.0; metadata/src/main/java/org/apache/kafka/controller/metrics/ControllerMetadataMetricsPublisher.java @4.4.0
Done when: Gauges computed from registrations after each committed batch; tests.

### [To Do] CTRL-0160 Metrics GlobalTopicCount and GlobalPartitionCount
P1 · M2 · Task · deps: none · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/ControllerMetadataMetrics.java @4.4.0
Done when: Gauges computed from committed metadata; tests.

### [To Do] CTRL-0161 Metrics OfflinePartitionsCount and PreferredReplicaImbalanceCount
P1 · M2 · Task · deps: none · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/ControllerMetadataMetrics.java @4.4.0
Done when: Offline counts partitions without a leader; imbalance counts partitions led by a non-preferred replica; tests.

### [To Do] CTRL-0162 Metrics UncleanLeaderElectionsPerSec and ElectionFromEligibleLeaderReplicasPerSec
P1 · M3 · Task · deps: CTRL-F08 · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/ControllerMetadataMetrics.java @4.4.0
Done when: Counters increment for each unclean and each ELR election committed; tests.

### [To Do] CTRL-0163 Metric MetadataErrorCount
P1 · M2 · Task · deps: none · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/ControllerMetadataMetrics.java @4.4.0
Done when: The counter increments for each non-fatal metadata replay error; tests.

### [To Do] CTRL-0164 Per-broker metrics BrokerRegistrationState and TimeSinceLastHeartbeatReceivedMs
P1 · M2 · Task · deps: none · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/BrokerRegistrationState.java @4.4.0; metadata/src/main/java/org/apache/kafka/controller/metrics/QuorumControllerMetrics.java addTimeSinceLastHeartbeatMetric @4.4.0
Done when: Gauges per registered broker id (bounded by the cluster size) are added on registration and removed on unregistration; tests.

### [To Do] CTRL-0165 Metric IgnoredStaticVoters
P2 · M2 · Task · deps: RAFT-F15 · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/metrics/QuorumControllerMetrics.java @4.4.0
Done when: The gauge is 1 when controller.quorum.voters is set but ignored because a voter set exists in the log; tests.

### [To Do] CTRL-0166 Slow event logging (controller.performance.sample.period.ms, controller.performance.always.log.threshold.ms)
P1 · M2 · Task · deps: none · labels: observability · ref: metadata/src/main/java/org/apache/kafka/controller/EventPerformanceMonitor.java @4.4.0
Done when: Events slower than the always-log threshold are logged with their name and duration; a periodic summary logs the slowest event per sample period; tests.

## [To Do] CTRL-F20 Directory assignment
P1 · M7 · Feature · deps: CTRL-F02
Scope: AssignReplicasToDirs and offline directory handling in metadata.

### [To Do] CTRL-0167 Default directory for new replicas
P1 · M7 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java defaultDir @4.4.0; metadata/src/main/java/org/apache/kafka/metadata/placement/DefaultDirProvider.java @4.4.0
Done when: New partition replicas get the broker's only online directory, or UNASSIGNED when it has several; PartitionRecord directories follow the replica order; tests.

### [To Do] CTRL-0168 AssignReplicasToDirs handling
P1 · M7 · Task · deps: CTRL-0167 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java handleAssignReplicasToDirs @4.4.0
Done when: Unsupported metadata.version fails UNSUPPORTED_VERSION; stale epochs STALE_BROKER_EPOCH; unregistered brokers BROKER_ID_NOT_REGISTERED; per partition UNKNOWN_TOPIC_ID, UNKNOWN_TOPIC_OR_PARTITION, or NOT_LEADER_OR_FOLLOWER when the broker is not a replica; assignments write PartitionChangeRecord directories, and assignments to offline directories trigger leader and ISR updates; tests.

### [To Do] CTRL-0169 Offline directories reported in heartbeats
P1 · M7 · Task · deps: CTRL-0170 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java handleDirectoriesOffline @4.4.0
Done when: Newly offline directories remove the broker from ISR and leadership for partitions in them and write BrokerRegistrationChangeRecord v2 with the remaining online directories; already-offline directories are ignored; tests.

### [To Do] CTRL-0170 Directory to partitions index
P1 · M7 · Task · deps: CTRL-0167 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java updatePartitionDirectories @4.4.0
Done when: Partition directory assignments are indexed on replay (reserved ids ignored) and removed with topics; tests.

### [To Do] CTRL-0171 Cordoned log directories
P1 · M7 · Task · deps: CTRL-0169 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java handleDirectoriesCordoned @4.4.0; metadata/src/main/java/org/apache/kafka/controller/ConfigurationControlManager.java isCordonedLogDirsInvalid @4.4.0
Done when: Heartbeats reporting changed cordoned directories write BrokerRegistrationChangeRecord v3; brokers whose directories are all cordoned are excluded from placement and fail manual assignments with INVALID_REPLICA_ASSIGNMENT; cordoned.log.dirs config changes via controllers are validated (INVALID_CONFIG); tests.

### [To Do] CTRL-0172 Port Kafka's directory assignment controller tests
P1 · M7 · Task · deps: CTRL-0168, CTRL-0171 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/ReplicationControlManagerTest.java @4.4.0
Done when: Directory assignment cases from ReplicationControlManagerTest and ClusterControlManagerTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F21 Request forwarding
P0 · M2 · Feature · deps: CTRL-F01, NET-F10
Scope: Brokers forward controller-bound admin requests in Envelope with principal propagation.

### [To Do] CTRL-0173 Forwardable API set from the schemas
P0 · M2 · Task · deps: PROTO-F07 · ref: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java forwardable @4.4.0; server/src/main/java/org/apache/kafka/server/EnvelopeUtils.java @4.4.0
Done when: The forwardable set equals Kafka 4.4.0's ApiKeys forwardable flags (CreateTopics, DeleteTopics, CreateAcls, DeleteAcls, AlterConfigs, IncrementalAlterConfigs, CreatePartitions, the three delegation token APIs, ElectLeaders, Alter/ListPartitionReassignments, AlterClientQuotas, AlterUserScramCredentials, DescribeQuorum, UpdateFeatures, UnregisterBroker, AllocateProducerIds, Add/RemoveRaftVoter, UnregisterController) and is part of the API key registry; an Envelope for any other API fails INVALID_REQUEST; a test compares the set with Kafka's.

### [To Do] CTRL-0174 Principal serialization for Envelope (DefaultPrincipalData)
P0 · M2 · Task · deps: SEC-F01, TEST-F07 · labels: security, decoder · ref: clients/src/main/resources/common/message/DefaultPrincipalData.json @4.4.0
Done when: Principals serialize to DefaultPrincipalData (type, name, token-authenticated flag) and back; malformed bytes yield a typed error, never a panic; a fuzz target covers the decoder; round-trip tests.

### [To Do] CTRL-0175 Broker side: wrap the original request in an Envelope
P0 · M2 · Task · deps: CTRL-0173, CTRL-0174 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManagerImpl.java @4.4.0; clients/src/main/resources/common/message/EnvelopeRequest.json @4.4.0
Done when: Through NET-F10's controller channel the forwarding broker sends EnvelopeRequest with the original request header and body bytes, the serialized principal and the client address bytes; queue and remote time are recorded; tests decode the envelope.

### [To Do] CTRL-0176 Broker side: response handling for forwarded requests
P0 · M2 · Task · deps: CTRL-0175 · ref: server/src/main/java/org/apache/kafka/server/ForwardingManagerImpl.java @4.4.0
Done when: An envelope UNSUPPORTED_VERSION closes the client connection so it renegotiates versions; any other envelope error returns UNKNOWN_SERVER_ERROR in the inner response; an undecodable inner response returns UNKNOWN_SERVER_ERROR; timeouts return REQUEST_TIMED_OUT; tests per case.

### [To Do] CTRL-0177 Controller side: unwrap an Envelope into a forwarded request context
P0 · M2 · Task · deps: CTRL-0173, CTRL-0174 · labels: security · ref: server/src/main/java/org/apache/kafka/server/EnvelopeUtils.java handleEnvelopeRequest @4.4.0
Done when: The controller parses the client address (INVALID_REQUEST if invalid), the inner header and body (UNSUPPORTED_VERSION if unparseable), and the principal (PRINCIPAL_DESERIALIZATION_FAILURE), rejects non-forwardable APIs, and runs the inner request with the forwarded principal and client address; tests per error.

### [To Do] CTRL-0178 Forwarding metrics: QueueTimeMs, RemoteTimeMs, QueueLength
P1 · M2 · Task · deps: CTRL-0175, OBS-F04 · labels: observability · ref: server/src/main/java/org/apache/kafka/server/metrics/ForwardingManagerMetrics.java @4.4.0
Done when: Histograms for queue and remote time and a gauge for the forwarding queue length are exported; tests.

### [To Do] CTRL-0179 Forwarded admin request end to end
P0 · M2 · Task · deps: CTRL-0176, CTRL-0177, TEST-F08 · labels: test
Done when: kafka-topics.sh --create against a broker listener in a node whose controller is reached through forwarding succeeds, and the controller sees the client's principal; compatibility test.

## [To Do] CTRL-F22 Unregistration
P1 · M3 · Feature · deps: CTRL-F02
Scope: UnregisterBroker and UnregisterController.

### [To Do] CTRL-0180 UnregisterBroker: remove from ISR and ELR, write UnregisterBrokerRecord
P1 · M3 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java unregisterBroker, handleBrokerUnregistered @4.4.0
Done when: Unregistering an unknown broker fails BROKER_ID_NOT_REGISTERED; otherwise the broker leaves every ISR and ELR (new leaders elected) and an UnregisterBrokerRecord with its epoch is written; the heartbeat session is removed on replay; tests.

### [To Do] CTRL-0181 UnregisterController with metadata.version and registration checks
P1 · M3 · Task · deps: CTRL-F03 · ref: metadata/src/main/java/org/apache/kafka/controller/ClusterControlManager.java unregisterController @4.4.0; metadata/src/main/resources/common/metadata/UnregisterControllerRecord.json @4.4.0
Done when: An unsupported metadata.version fails UNSUPPORTED_VERSION; an unknown controller id fails CONTROLLER_ID_NOT_REGISTERED; otherwise UnregisterControllerRecord is written and replay removes the registration; tests.

### [To Do] CTRL-0182 Port Kafka's unregistration tests
P1 · M3 · Task · deps: CTRL-0180, CTRL-0181 · labels: test · ref: metadata/src/test/java/org/apache/kafka/controller/ClusterControlManagerTest.java @4.4.0
Done when: Unregistration cases from ReplicationControlManagerTest and ClusterControlManagerTest at 4.4.0 pass with attribution.

## [To Do] CTRL-F23 Controller simulation testing
P0 · M3 · Feature · deps: CTRL-F08, TEST-F06
Scope: Election, fencing and reassignment correctness under faults.

### [To Do] CTRL-0183 Controller simulation scaffold: controller quorum plus simulated brokers
P0 · M3 · Task · deps: RAFT-F13 · labels: test
Done when: A sim test runs 3 controllers on the raft simulation with simulated brokers that register, heartbeat, report ISR changes and shut down; seeds are printed with replay commands; fixed seeds run in CI.

### [To Do] CTRL-0184 Invariant: at most one active controller per epoch, all writes from it
P0 · M3 · Task · deps: CTRL-0183 · labels: test
Done when: A checker asserts no two controllers are active in the same raft epoch and every committed record batch was written by the active controller of its epoch.

### [To Do] CTRL-0185 Invariant: leaders are registered, unfenced replicas in the ISR (or recovering after unclean)
P0 · M3 · Task · deps: CTRL-0183 · labels: test
Done when: After every committed batch each partition leader is a replica, unfenced and in the ISR, or the partition is leaderless; ISR is a subset of replicas; ISR is non-empty unless ELR is enabled; leader epochs never decrease.

### [To Do] CTRL-0186 Invariant: a new active controller's state equals the committed metadata
P0 · M3 · Task · deps: CTRL-0183 · labels: test
Done when: After failover the new active controller's state equals the state rebuilt from committed records; no acknowledged admin operation is lost; pending ones fail NOT_CONTROLLER or time out.

### [To Do] CTRL-0187 Scenario: broker crashes and partitions are fenced within the session timeout and leaders move
P0 · M3 · Task · deps: CTRL-0185 · labels: test
Done when: Crashed or partitioned brokers are fenced within 112.5% of broker.session.timeout.ms after the last heartbeat and their leaderships move to live ISR members.

### [To Do] CTRL-0188 Scenario: controlled shutdown moves all leaderships before granting shutdown
P0 · M3 · Task · deps: CTRL-0185 · labels: test
Done when: A broker in controlled shutdown is granted shutdown only after leading no partitions; no partition becomes leaderless when another ISR member is live.

### [To Do] CTRL-0189 Scenario: reassignments complete or cancel correctly under faults
P0 · M3 · Task · deps: CTRL-0185, CTRL-F10 · labels: test
Done when: Random reassignments under broker crashes and controller failover eventually complete when target replicas are live, and cancellation restores the original replicas.

### [To Do] CTRL-0190 Nightly controller simulation with thousands of seeds
P1 · M3 · Task · deps: CTRL-0184, CTRL-0186, CTRL-0187, CTRL-0188, CTRL-0189 · labels: test
Done when: A nightly job runs every controller sim test with thousands of seeds; failing seeds are reported with replay commands and added to the fixed set.
