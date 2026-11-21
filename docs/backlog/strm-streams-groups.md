# STRM: Streams groups

> Crates: `alluvium-group-coordinator` · Kafka counterpart: `group-coordinator/…/streams` (KIP-1071) · Milestones: M8

The broker-side group protocol for Kafka Streams applications (KIP-1071).

## [To Do] STRM-F01 Streams group membership
P0 · M8 · Feature · deps: GRP-F05
Scope: StreamsGroupHeartbeat, member lifecycle, epochs.

### [To Do] STRM-0001 Streams group and member model
P0 · M8 · Task · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/StreamsGroup.java @4.4.0
Done when: streams groups hold group epoch, target assignment epoch, metadata hash, topology and its validated epoch, members, target assignments, current task owners per epoch, task offsets and endpoint information epoch on timeline structures; members hold epochs, state, process id, rack id, instance id, client tags, user endpoint, rebalance timeout and active/standby/warm-up tasks; unit tests.

### [To Do] STRM-0002 Streams group record replay
P0 · M8 · Task · deps: STRM-0001 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/StreamsCoordinatorRecordHelpers.java @4.4.0
Done when: StreamsGroupMetadata, StreamsGroupTopology, StreamsGroupMemberMetadata, StreamsGroupTargetAssignmentMetadata/Member and StreamsGroupCurrentMemberAssignment records and tombstones replay into the model; round-trip tests per record.

### [To Do] STRM-0003 Gate streams groups on `streams.version` and enabled protocols
P0 · M8 · Task · deps: STRM-0001, META-F06 · ref: server-common/src/main/java/org/apache/kafka/server/common/StreamsVersion.java @4.4.0
Done when: StreamsGroupHeartbeat returns UNSUPPORTED_VERSION when the finalized `streams.version` is 0 or `streams` is not in `group.coordinator.rebalance.protocols`; tests.

### [To Do] STRM-0004 StreamsGroupHeartbeat request validation
P0 · M8 · Task · deps: STRM-0001 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java throwIfStreamsGroupHeartbeatRequestIsInvalid @4.4.0
Done when: INVALID_REQUEST is returned for an empty member id, group id, instance id or rack id; epoch 0 without rebalance timeout, with non-empty active/standby/warm-up tasks or without a topology; epoch -2 without instance id; epochs below -2; only some task lists set; a topology on a non-zero epoch; source topic regexes (not supported in 4.4.0); one test per rule.

### [To Do] STRM-0005 Streams member join and group creation
P0 · M8 · Task · deps: STRM-0002 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java getOrCreateStreamsGroup @4.4.0
Done when: epoch 0 creates the group if absent and creates the member with process id, client tags, user endpoint and rebalance timeout; a group of another type returns GROUP_ID_NOT_FOUND (empty consumer or classic groups are converted as Kafka does); tests.

### [To Do] STRM-0006 Enforce `group.streams.max.size`
P0 · M8 · Task · deps: STRM-0005 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java throwIfStreamsGroupIsFull @4.4.0
Done when: a new member beyond `group.streams.max.size` gets GROUP_MAX_SIZE_REACHED; static replacements are not counted; tests.

### [To Do] STRM-0007 Streams member epoch fencing and task ownership checks
P0 · M8 · Task · deps: STRM-0005 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java throwIfStreamsGroupMemberEpochIsInvalid @4.4.0
Done when: an unknown member with a non-zero epoch returns UNKNOWN_MEMBER_ID; a larger epoch FENCED_MEMBER_EPOCH; a smaller epoch is accepted only if it is the previous epoch and the owned tasks are contained in the assigned tasks, else FENCED_MEMBER_EPOCH; owned tasks naming unknown subtopologies or partitions return INVALID_REQUEST; tests.

### [To Do] STRM-0008 Static streams members
P0 · M8 · Task · deps: STRM-0007 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java getOrMaybeCreateStaticStreamsGroupMember @4.4.0
Done when: a static member rejoining after leaving with epoch -2 replaces its previous member and keeps its assignment; replacing an active member returns UNRELEASED_INSTANCE_ID; a member id mismatch FENCED_INSTANCE_ID; an unknown instance with non-zero epoch UNKNOWN_MEMBER_ID; tests.

### [To Do] STRM-0009 Group epoch bumps for streams groups
P0 · M8 · Task · deps: STRM-0005 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java hasStreamsMemberMetadataChanged @4.4.0
Done when: a new member, a member leaving, a topology change, a metadata hash change of source topics, or a change of epoch-relevant member metadata (process id, rack, client tags) bumps the group epoch; unchanged heartbeats do not; tests per trigger.

### [To Do] STRM-0010 Streams reconciliation: compute task revocations
P0 · M8 · Task · deps: STRM-0009 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/CurrentAssignmentBuilder.java @4.4.0
Done when: a STABLE member behind the target moves to UNREVOKED_TASKS when it owns active, standby or warm-up tasks not in its target, otherwise to the target epoch; the current assignment record is written; tests.

### [To Do] STRM-0011 Streams reconciliation on revocation acknowledgement
P0 · M8 · Task · deps: STRM-0010 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/CurrentAssignmentBuilder.java @4.4.0
Done when: once the heartbeat's owned tasks drop the revoked tasks the member advances; active tasks still owned by another member keep the member in UNRELEASED_TASKS until released; UNKNOWN state returns FENCED_MEMBER_EPOCH; tests.

### [To Do] STRM-0012 Streams revocation timeout
P0 · M8 · Task · deps: STRM-0011 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java scheduleStreamsGroupRebalanceTimeout @4.4.0
Done when: a member in UNREVOKED_TASKS that does not acknowledge within its rebalance timeout is fenced and its next heartbeat gets FENCED_MEMBER_EPOCH; the timer is cancelled on acknowledgement.

### [To Do] STRM-0013 Task ownership epochs invariant
P0 · M8 · Task · deps: STRM-0010 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/StreamsGroup.java @4.4.0
Done when: each active task is owned by at most one member epoch while standby and warm-up owners are tracked per process; adding an owned active task already owned panics as an invariant violation; property test over random reconciliations.

### [To Do] STRM-0014 Streams session timeout and leave
P0 · M8 · Task · deps: STRM-0005 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java scheduleStreamsGroupSessionTimeout @4.4.0
Done when: heartbeats reschedule the session timer from `streams.session.timeout.ms` (bounded by `group.streams.min/max.session.timeout.ms`); expiry or a leave with epoch -1 fences the member and bumps the group epoch; epoch -2 keeps a static member's assignment until its session expires; tests.

### [To Do] STRM-0015 StreamsGroupHeartbeat response content
P0 · M8 · Task · deps: STRM-0010 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java streamsGroupHeartbeat @4.4.0
Done when: responses carry member epoch, heartbeat interval (`streams.heartbeat.interval.ms`), task offset interval (`streams.task.offset.interval.ms`, at least `group.streams.min.task.offset.interval.ms`), and active, standby and warm-up tasks when joining or changed; tests.

### [To Do] STRM-0016 Task offsets and end offsets reported by members
P0 · M8 · Task · deps: STRM-0005 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/MemberTaskOffsets.java @4.4.0
Done when: `task_offsets` and `task_end_offsets` from heartbeats update per-member offset sums used for warm-up readiness and standby ranking; malformed task ids return INVALID_REQUEST; tests.

### [To Do] STRM-0017 Interactive-query endpoint information
P0 · M8 · Task · deps: STRM-0015 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/topics/EndpointToPartitionsManager.java @4.4.0
Done when: a user endpoint change, or a task change of a member with an endpoint, bumps the endpoint information epoch; members whose epoch differs receive partitions by user endpoint (active and standby) computed from the metadata image; stale cache entries are invalidated; tests.

### [To Do] STRM-0018 Application shutdown requests
P0 · M8 · Task · deps: STRM-0015 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java streamsGroupHeartbeat @4.4.0
Done when: a heartbeat with `shutdown_application` records the requesting member and all members then receive the SHUTDOWN_APPLICATION status with its detail until the group empties; tests.

### [To Do] STRM-0019 MISSING_CLIENT_TAGS status
P0 · M8 · Task · deps: STRM-0015 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java streamsGroupHeartbeat @4.4.0
Done when: when rack-aware assignment tags are configured (`streams.rack.aware.assignment.tags` or `group.streams.rack.aware.assignment.tags`) and the member lacks any of them, StreamsGroupHeartbeat v1+ responses include status MISSING_CLIENT_TAGS(6) listing them, and v0 responses never do; tests per version.

### [To Do] STRM-0020 Streams group config keys
P0 · M8 · Task · deps: STRM-0014, GRP-F08 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java @4.4.0
Done when: `streams.session.timeout.ms`, `streams.heartbeat.interval.ms`, `streams.num.standby.replicas`, `streams.initial.rebalance.delay.ms`, `streams.assignment.interval.ms`, `streams.rack.aware.assignment.tags`, `streams.assignor.offload.enable`, `streams.task.offset.interval.ms`, `streams.num.warmup.replicas`, `streams.acceptable.recovery.lag` and `streams.assignor.name` are validated against broker bounds and applied on the next heartbeat; tests per key.

### [To Do] STRM-0021 Property test of streams group invariants
P0 · M8 · Task · deps: STRM-0011, STRM-0014, STRM-0008 · labels: test
Done when: random heartbeats, leaves, timeouts and topology updates keep member epoch <= target epoch <= group epoch and no active task owned twice, and every committed state reloads identically.

### [To Do] STRM-0022 Port Kafka's streams membership tests
P1 · M8 · Task · deps: STRM-0021, TEST-F10 · labels: test · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java @4.4.0
Done when: streams cases from `GroupMetadataManagerTest` and streams `CurrentAssignmentBuilderTest` in the TEST-F10 inventory are ported with attributions and pass.

## [To Do] STRM-F02 Topology metadata
P0 · M8 · Feature · deps: STRM-F01
Scope: Subtopologies, source, repartition and changelog topics, internal topic validation and creation.

### [To Do] STRM-0023 Topology model and persistence
P0 · M8 · Task · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/StreamsTopology.java @4.4.0
Done when: the topology (epoch, subtopologies with source topics, repartition sink topics, state changelog topics, repartition source topics with partition counts and configs, copartition groups) is stored as a StreamsGroupTopology record and replayed; tests.

### [To Do] STRM-0024 Topology request validation
P0 · M8 · Task · deps: STRM-0023 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java throwIfInvalidTopology @4.4.0
Done when: internal topics declared with explicit partition counts where Kafka forbids them, and other malformed topologies, return STREAMS_INVALID_TOPOLOGY; a topology epoch lower than the stored one with a different topology is reported as STALE_TOPOLOGY status; tests.

### [To Do] STRM-0025 Topology epochs and STALE_TOPOLOGY status
P0 · M8 · Task · deps: STRM-0023 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java maybeUpdateTopology @4.4.0
Done when: the first member's topology is stored; a member with a different topology epoch than the group's gets status STALE_TOPOLOGY(0); a higher epoch from a joining member replaces the topology only when the group allows it as Kafka does; tests.

### [To Do] STRM-0026 Missing source topics
P0 · M8 · Task · deps: STRM-0023 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/topics/InternalTopicManager.java configureTopics @4.4.0
Done when: when any source topic is missing from the metadata image the topology is not configured, the group is NotReady and members get status MISSING_SOURCE_TOPICS(1) naming them; tests.

### [To Do] STRM-0027 Repartition topic partition counts
P0 · M8 · Task · deps: STRM-0026 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/topics/RepartitionTopics.java @4.4.0
Done when: repartition topic partition counts are derived from upstream source topics and explicit counts, iterating to a fixed point; an undecidable count raises STREAMS_INVALID_TOPOLOGY; ported `RepartitionTopicsTest` cases pass.

### [To Do] STRM-0028 Changelog topic partition counts
P0 · M8 · Task · deps: STRM-0026 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/topics/ChangelogTopics.java @4.4.0
Done when: changelog topics take the maximum partition count of their subtopology's source topics (or an explicit count); a subtopology without source topics raises STREAMS_INVALID_TOPOLOGY; ported tests pass.

### [To Do] STRM-0029 Copartitioning enforcement
P0 · M8 · Task · deps: STRM-0027 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/topics/CopartitionedTopicsEnforcer.java @4.4.0
Done when: external topics in a copartition group must have equal partition counts, else status INCORRECTLY_PARTITIONED_TOPICS(2); repartition topics in the group take the external count or the maximum; all-undefined groups raise STREAMS_INVALID_TOPOLOGY; ported `CopartitionedTopicsEnforcerTest` cases pass.

### [To Do] STRM-0030 Missing internal topics and creation requests
P0 · M8 · Task · deps: STRM-0028, STRM-0029 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/topics/InternalTopicManager.java missingInternalTopics @4.4.0
Done when: internal topics absent from the metadata image are returned as creation requests (name, partitions, replication factor, configs) with status MISSING_INTERNAL_TOPICS(3); internal topics that exist with a wrong partition count are reported with INCORRECTLY_PARTITIONED_TOPICS; tests.

### [To Do] STRM-0031 Create internal topics requested by the coordinator
P0 · M8 · Task · deps: STRM-0030, CTRL-F04
Done when: the heartbeat path creates the requested internal topics through the controller with the member's principal (requiring CREATE on the topic or cluster, as Kafka checks), de-duplicates in-flight creations, and reports creation failures in the status detail; integration test.

### [To Do] STRM-0032 Configured topology and readiness
P0 · M8 · Task · deps: STRM-0030 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/topics/ConfiguredTopology.java @4.4.0
Done when: a configured topology with task counts per subtopology is cached with the metadata hash it was computed for, recomputed when the hash changes, and the validated topology epoch is persisted; only a ready topology permits assignment; tests.

### [To Do] STRM-0033 Port Kafka's topology configuration tests
P1 · M8 · Task · deps: STRM-0032, TEST-F10 · labels: test · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/topics/InternalTopicManager.java @4.4.0
Done when: `InternalTopicManagerTest` and related cases in the TEST-F10 inventory are ported with attributions and pass.

## [To Do] STRM-F03 Task assignment
P0 · M8 · Feature · deps: STRM-F02
Scope: Active, standby and warm-up tasks; sticky assignor.

### [To Do] STRM-0034 Streams task assignor interface
P0 · M8 · Task · deps: none · ref: group-coordinator/group-coordinator-api/src/main/java/org/apache/kafka/coordinator/group/api/streams/assignor/TaskAssignor.java @4.4.0
Done when: the assignor takes a group spec (members with process id, rack, client tags, current active/standby/warm-up tasks and task offsets; assignment configs) and a topology describer (subtopologies, partition counts, stateful flag) and returns active, standby and warm-up tasks per member; errors are a typed `TaskAssignorError`.

### [To Do] STRM-0035 Sticky active task assignment
P0 · M8 · Task · deps: STRM-0034 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/assignor/StickyTaskAssignor.java assignActive @4.4.0
Done when: active tasks keep previous owners when within quota, then go to the member of the same process or with least load, with per-member quotas computed as Kafka does; ported `StickyTaskAssignorTest` active cases pass.

### [To Do] STRM-0036 Sticky standby task assignment
P0 · M8 · Task · deps: STRM-0035 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/assignor/StickyTaskAssignor.java assignStandby @4.4.0
Done when: up to `num.standby.replicas` standbys per stateful task are placed on processes other than the active's, preferring previous standbys ranked by reported offset sums, then least load; insufficient members are logged; ported standby cases pass.

### [To Do] STRM-0037 Assignment configuration passed to the assignor
P0 · M8 · Task · deps: STRM-0034, STRM-0020 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/assignor/AssignmentConfigsImpl.java @4.4.0
Done when: `num.standby.replicas` (capped by `group.streams.max.standby.replicas`), `num.warmup.replicas` (capped by `group.streams.max.warmup.replicas`), `acceptable.recovery.lag` and rack-aware assignment tags reach the assignor from group or broker config; the assignor is selected from `group.streams.assignors` by `streams.assignor.name`; tests.

### [To Do] STRM-0038 Warm-up refinement of the target assignment
P0 · M8 · Task · deps: STRM-0035, STRM-0016 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/AssignmentRefiner.java @4.4.0
Done when: a stateful task moving to a member without its state stays with its current owner while the new owner gets a warm-up task, at most `num.warmup.replicas` warm-ups run at once, and the task moves once the warm-up's lag is within `acceptable.recovery.lag`; ported refiner tests pass.

### [To Do] STRM-0039 Streams target assignment computation and persistence
P0 · M8 · Task · deps: STRM-0038 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/TargetAssignmentBuilder.java @4.4.0
Done when: when the group epoch exceeds the target epoch and the topology is ready, the target is recomputed and refined, changed member targets and the target metadata are written, and static replacements keep their targets; assignor errors are logged once and fail the heartbeat with UNKNOWN_SERVER_ERROR.

### [To Do] STRM-0040 Assignment interval, offload and initial delay for streams groups
P0 · M8 · Task · deps: STRM-0039 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java maybeUpdateStreamsTargetAssignment @4.4.0
Done when: recomputation runs at most once per `streams.assignment.interval.ms`, optionally on the executor per `streams.assignor.offload.enable` with stale results dropped, and the first assignment waits `streams.initial.rebalance.delay.ms` reporting status ASSIGNMENT_DELAYED(5) meanwhile; tests with the simulated clock.

### [To Do] STRM-0041 Streams group state transitions
P0 · M8 · Task · deps: STRM-0010, STRM-0032 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/StreamsGroup.java @4.4.0
Done when: the state is Empty with no members, NotReady while the topology is missing or not configured, Assigning while the target epoch is behind, Reconciling until members reach the target, Stable afterwards and Dead on deletion; each transition is returned with a reason; one test per transition.

### [To Do] STRM-0042 Property tests for the streams assignor
P0 · M8 · Task · deps: STRM-0036, STRM-0038 · labels: test
Done when: for random groups every task has exactly one active owner, standbys never share a process with their active, warm-ups never exceed the limit, and unchanged inputs give unchanged outputs.

### [To Do] STRM-0043 Benchmark streams task assignment
P2 · M8 · Task · deps: STRM-0039, TEST-F11 · labels: perf
Done when: a criterion benchmark for 500 members and 5 000 tasks with 2 standbys records latency against a budget in docs/nfr.md.

## [To Do] STRM-F04 StreamsGroupDescribe
P1 · M8 · Feature · deps: STRM-F01
Scope: Describe semantics.

### [To Do] STRM-0044 StreamsGroupDescribe
P1 · M8 · Task · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java streamsGroupDescribe @4.4.0
Done when: each group returns state, epochs, topology (subtopologies with source, repartition and changelog topics and their configured partitions) and members with process id, client tags, endpoints, task offsets and current and target active/standby/warm-up tasks; a missing or non-streams group returns GROUP_ID_NOT_FOUND; reads use the committed state; tests.

### [To Do] STRM-0045 StreamsGroupDescribe v1: assignor name and topology description status
P1 · M8 · Task · deps: STRM-0044, STRM-0050 · ref: clients/src/main/resources/common/message/StreamsGroupDescribeResponse.json @4.4.0
Done when: v1 responses carry `AssignorName` (the assignor for the next computation, null on error) and `TopologyDescriptionStatus` NOT_REQUESTED(0) when `IncludeTopologyDescription` is false or NOT_STORED(1) when nothing is stored; v0 omits both; tests per version.

### [To Do] STRM-0046 Return stored topology descriptions in StreamsGroupDescribe
P2 · M8 · Task · deps: STRM-0045, STRM-0053 · ref: clients/src/main/resources/common/message/StreamsGroupDescribeResponse.json @4.4.0
Done when: with a description store configured, `IncludeTopologyDescription` returns the stored description with status AVAILABLE(3), or ERROR(2) when fetching it fails (logged); tests.

### [To Do] STRM-0047 Streams groups in classic DescribeGroups and ListGroups
P1 · M8 · Task · deps: STRM-0044 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java describeGroups @4.4.0
Done when: DescribeGroups for a streams group returns GROUP_ID_NOT_FOUND as Kafka does, and ListGroups with the `streams` type filter returns streams groups with their states; tests.

### [To Do] STRM-0048 Document streams group describe output
P2 · M8 · Task · deps: STRM-0044 · labels: docs
Done when: module docs map each describe field to its source in the group state.

## [To Do] STRM-F05 Topology description updates
P1 · M8 · Feature · deps: STRM-F02
Scope: StreamsGroupTopologyDescriptionUpdate handling.

### [To Do] STRM-0049 Decide how to support the topology description plugin (KIP-1331)
P1 · M8 · Spike · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/StreamsGroupTopologyDescriptionManager.java @4.4.0
Done when: ADR `streams-topology-description-plugin` accepted in docs/adr/ recording that `group.streams.topology.description.plugin.class` defaults to unset (feature disabled) in 4.4.0, and deciding whether Alluvium offers a built-in store, a native plugin interface, or only the disabled behavior.

### [To Do] STRM-0050 Disabled-plugin behavior
P1 · M8 · Task · deps: STRM-0049 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java streamsGroupTopologyDescriptionUpdate @4.4.0
Done when: with no plugin configured StreamsGroupTopologyDescriptionUpdate returns UNSUPPORTED_VERSION, heartbeats never set `TopologyDescriptionRequired`, and empty streams groups convert to classic groups directly; tests.

### [To Do] STRM-0051 StreamsGroupTopologyDescriptionUpdate validation
P1 · M8 · Task · deps: STRM-0049 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java streamsGroupTopologyDescriptionUpdate @4.4.0
Done when: an inactive coordinator returns COORDINATOR_NOT_AVAILABLE; an empty member or group id or a null description returns INVALID_REQUEST; a request from a member not in the group or with a mismatched topology epoch is rejected as Kafka's `validateStreamsGroupTopologyDescriptionUpdate` does; tests.

### [To Do] STRM-0052 Stored description topology epochs
P2 · M8 · Task · deps: STRM-0051 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupMetadataManager.java setStoredDescriptionTopologyEpoch @4.4.0
Done when: the group persists the stored description epoch (NONE -1, UNCERTAIN -2 or a topology epoch); a successful push records its epoch, failures record the failed epoch, and the UNCERTAIN barrier is written before deletes; heartbeats v1 set `TopologyDescriptionRequired` when the stored epoch is behind; tests per transition.

### [To Do] STRM-0053 Topology description storage per the ADR
P2 · M8 · Task · deps: STRM-0052
Done when: the storage chosen in the ADR implements set, get and delete with bounded size per group; failures map to STREAMS_TOPOLOGY_DESCRIPTION_UPDATE_FAILED with a message; tests.

### [To Do] STRM-0054 Backoff for failed description pushes
P2 · M8 · Task · deps: STRM-0053 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/StreamsGroupTopologyDescriptionBackoff.java @4.4.0
Done when: transient failures arm a per-group backoff that suppresses `TopologyDescriptionRequired` and conversion deletes until it expires, cleared on success; tests with the simulated clock.

### [To Do] STRM-0055 Description cleanup on group deletion and conversion
P2 · M8 · Task · deps: STRM-0053 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java cleanupTopologyBeforeConversion @4.4.0
Done when: deleting or converting an empty streams group deletes its stored description behind the UNCERTAIN barrier first; a failed delete keeps the group and makes the join fail with REBALANCE_IN_PROGRESS; a periodic cycle every `offsets.retention.check.interval.ms` reclaims eligible groups; tests.

### [To Do] STRM-0056 Topology description metrics
P2 · M8 · Task · deps: STRM-0055, OBS-F04 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/metrics/GroupCoordinatorMetrics.java @4.4.0
Done when: equivalents of the streams-group-topology-description get/set/delete success and error rates and cleanup cycle and eligible counts are exported with docs/metrics.md rows.

## [To Do] STRM-F06 Streams group administration
P1 · M8 · Feature · deps: STRM-F01
Scope: Offsets, deletion, listing.

### [To Do] STRM-0057 Offset commit and fetch for streams groups
P1 · M8 · Task · deps: GRP-F07 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/StreamsGroup.java validateOffsetCommit @4.4.0
Done when: epoch < 0 with no members is accepted; an unknown member returns UNKNOWN_MEMBER_ID; OffsetCommit below v9 returns UNSUPPORTED_VERSION; a newer epoch STALE_MEMBER_EPOCH; an older epoch is accepted per partition only if its task was assigned at an epoch <= the received one; OffsetFetch with an epoch mismatch returns STALE_MEMBER_EPOCH; tests.

### [To Do] STRM-0058 OffsetDelete for streams groups
P1 · M8 · Task · deps: GRP-F07 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/StreamsGroup.java validateOffsetDelete @4.4.0
Done when: offsets of topics that are source topics of the group's topology return GROUP_SUBSCRIBED_TO_TOPIC while the group is not empty, and other offsets are deleted; tests.

### [To Do] STRM-0059 DeleteGroups for streams groups
P1 · M8 · Task · deps: GRP-F08 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/StreamsGroup.java validateDeleteGroup @4.4.0
Done when: a non-empty streams group returns NON_EMPTY_GROUP; an empty one has its offsets and group records tombstoned (after topology description cleanup when a plugin is configured); tests.

### [To Do] STRM-0060 Offset expiry for streams groups
P1 · M8 · Task · deps: GRP-F07 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/streams/StreamsGroup.java @4.4.0
Done when: offsets of streams groups expire with the same rules as consumer groups (commit timestamp, unsubscribed topics only while members exist) and empty streams groups with no offsets become Dead; tests.

## [To Do] STRM-F07 Streams group testing
P0 · M8 · Feature · deps: STRM-F03, TEST-F08
Scope: Kafka Streams applications running against Alluvium with the streams protocol.

### [To Do] STRM-0061 Simulation of streams groups with simulated Streams clients
P0 · M8 · Task · deps: TEST-F06 · labels: test
Done when: testkit streams clients heartbeat with topologies, report task offsets and process assignments; simulations under crashes and network faults keep at most one active owner per task and converge to Stable over the CI seed set.

### [To Do] STRM-0062 Kafka Streams application compatibility test with `group.protocol=streams`
P0 · M8 · Task · deps: none · labels: test
Done when: a Kafka Streams 4.4.0 application with stateful operators, repartitioning and standby replicas runs against Alluvium with the streams protocol through scale-out, scale-in and instance restarts, producing correct output.

### [To Do] STRM-0063 Interactive queries and shutdown in the compatibility test
P0 · M8 · Task · deps: STRM-0062 · labels: test
Done when: the test app's interactive-query metadata matches its endpoints after rebalances, and a requested application shutdown stops all instances.

### [To Do] STRM-0064 Differential tests of the streams group protocol against Kafka
P0 · M8 · Task · deps: TEST-F09 · labels: test
Done when: scripted StreamsGroupHeartbeat and StreamsGroupDescribe sequences produce identical error codes, statuses and assignments (modulo member ids) on Kafka 4.4.0 and Alluvium.

### [To Do] STRM-0065 Streams group metrics and events
P1 · M8 · Task · deps: OBS-F04 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/metrics/GroupCoordinatorMetrics.java @4.4.0
Done when: equivalents of streams-group-count by state and streams-group-rebalance-rate/count are exported with docs/metrics.md rows, and `streams_group_state_change`, `streams_member_fenced` and `streams_topology_updated` events with reasons are listed in `events.rs`.
