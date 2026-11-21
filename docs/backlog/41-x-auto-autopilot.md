# X-AUTO: Operational autopilot

> Crates: `alluvium-autopilot` (new; layer 2 logic plus driver) · Kafka counterpart: none (Alluvium enhancement) · Milestones: M17

Automatic balancing, self-healing, broker add/remove workflows, hot-partition handling, autoscaling signals and a Kubernetes operator.

## [To Do] X-AUTO-F01 Optimizer design
P0 · M17 · Feature · deps: CTRL-F10
Scope: Spike and ADR: goals, constraints, sans-I/O optimizer.

### [To Do] X-AUTO-0001 Autopilot architecture ADR: optimizer placement, sans-I/O optimizer and executor, goal model, load sample transport and opt-in modes
P0 · M17 · Spike · deps: none · ref: ADR-0002
Done when: ADR accepted in docs/adr/ deciding (1) whether the optimizer runs on the active controller or a separate process, (2) the pure `propose(model, goals, constraints) -> Result<Proposal, OptimizationFailure>` interface, (3) hard vs soft goals and the default goal priority order, (4) how broker load samples reach the optimizer, (5) execution through KIP-455 reassignment with executor state persisted across controller failover, (6) the `off`/`recommend`/`approve`/`auto` modes; includes a comparison with Cruise Control's model and lists rejected alternatives.

### [To Do] X-AUTO-0002 Optimizer search prototype: goal-ordered greedy search vs solver-based search on synthetic clusters
P0 · M17 · Spike · deps: X-AUTO-0001 · labels: perf
Done when: prototype benchmarks at 10 and 100 brokers with 10k and 200k replicas record per-goal result quality, runtime and peak memory for both approaches; the numbers and the chosen approach are appended to the autopilot ADR.

### [To Do] X-AUTO-0003 Autopilot threat model: unauthorized rebalances, forged or stale load samples, runaway data movement, operator credential misuse
P0 · M17 · Spike · deps: X-AUTO-0001 · labels: security · ref: docs/standards/security.md
Done when: docs/standards/security.md threat table gains autopilot rows, each naming the defending task in this epic (authorization, sample epoch checks, guardrail rate limits, audit); reviewed with the SEC-F14 checklist.

### [To Do] X-AUTO-0004 Create `alluvium-autopilot` (layer 2) with the top-level sans-I/O `Autopilot` state machine and its `AutopilotInput` (image delta, load sample, tick, operator command) and `AutopilotAction` enums
P0 · M17 · Task · deps: X-AUTO-0001 · ref: ADR-0002
Done when: crate registered in the xtask layer table and the docs/architecture.md code map; `//!` docs state responsibility and invariants; `cargo xtask check-layers` passes with no Tokio dependency; unit test shows a tick in mode `off` returns no actions.

### [To Do] X-AUTO-0005 `alluvium.autopilot.mode` master switch (`off` default, `recommend`, `approve`, `auto`) as a cluster-wide dynamic key
P0 · M17 · Task · deps: X-AUTO-0001, CFG-F11, CFG-F04 · labels: config
Done when: key registered through the CFG-F11 registry with default `off`; unknown values rejected with INVALID_CONFIG; a change through IncrementalAlterConfigs on the cluster-default broker resource reaches the autopilot without restart; `check-config` shows the value source; docs/config.md entry links the ADR.

### [To Do] X-AUTO-0006 Autopilot driver in `alluvium-server`: supervised `autopilot` task started on controller activation and stopped on resignation, feeding image deltas and ticks
P0 · M17 · Task · deps: X-AUTO-0004, X-AUTO-0005, SRV-F02
Done when: `/debug/tasks` lists `autopilot` only on the active controller; a simulation test with repeated controller failover shows at most one active driver at any step; every returned transition is recorded by the driver.

### [To Do] X-AUTO-0007 Autopilot flight recorder and `events.rs` catalog, starting with `autopilot_mode_changed` and its `ModeChangeReason`
P0 · M17 · Task · deps: X-AUTO-0006, OBS-F06, OBS-F07 · labels: observability · ref: docs/standards/debuggability.md
Done when: bounded ring of typed entries (default 256) readable at `/debug/autopilot/flight-recorder` and dumped on error and panic; a dynamic mode change emits the event with old mode, new mode and reason; the event appears in docs/events.md.

## [To Do] X-AUTO-F02 Load model
P0 · M17 · Feature · deps: X-AUTO-F01
Scope: Per-partition bytes, request rates, disk and CPU estimates.

### [To Do] X-AUTO-0008 `ClusterModel`: brokers (id, rack, liveness), log dirs, replicas with leader flag, and per-`Resource` (Cpu, NetworkIn, NetworkOut, Disk) load vectors built from a `MetadataImage`
P0 · M17 · Task · deps: none
Done when: `ClusterModel::from_image` builds from MetadataImage fixtures (rack-less, racked, JBOD); iteration is deterministic (BTreeMap); unit tests cover lookups by broker, rack, topic and log dir.

### [To Do] X-AUTO-0009 Model operations: replica move, leadership move and intra-broker log dir move, each updating per-broker and per-dir load totals
P0 · M17 · Task · deps: X-AUTO-0008
Done when: unit test per operation; property test over random operation sequences: cluster-wide load per resource is conserved and replica counts per partition never change.

### [To Do] X-AUTO-0010 `BrokerLoadSample` and `PartitionLoadSample` types (window start, broker id, broker epoch, values) with a versioned binary encoding
P0 · M17 · Task · deps: X-AUTO-0008 · labels: decoder
Done when: round-trip property test; decoder bounds every length and count before allocating and returns `LoadSampleDecodeError` with the byte offset; unknown versions rejected; no panics on truncated input.

### [To Do] X-AUTO-0011 Fuzz target `fuzz/fuzz_targets/autopilot_load_sample.rs` for the load sample decoder
P1 · M17 · Task · deps: X-AUTO-0010, TEST-F07 · labels: decoder, test, security
Done when: target runs in the per-PR fuzz job with a seed corpus generated from round-trip tests; a one-hour local run finds no panic, hang or allocation beyond the configured cap.

### [To Do] X-AUTO-0012 Load input: per-partition leader bytes-in per window, recorded on the leader append path
P0 · M17 · Task · deps: X-AUTO-0010
Done when: accumulator keyed by topic id and partition counts appended batch bytes on leaders only (follower appends excluded); a produce test with known batch sizes matches exactly; accumulator memory is bounded by hosted partition count.

### [To Do] X-AUTO-0013 Load input: per-partition consumer bytes-out per window from fetch responses to non-replica fetchers
P0 · M17 · Task · deps: X-AUTO-0010
Done when: consumer fetch bytes counted per partition, including sendfile responses by region length; follower fetches excluded; test with a scripted consumer matches returned byte counts.

### [To Do] X-AUTO-0014 Load input: replication traffic per partition, as bytes-out on the leader and bytes-in on each follower
P0 · M17 · Task · deps: X-AUTO-0010
Done when: follower fetch bytes attributed to the leader's NetworkOut and the follower's NetworkIn; test with RF=3 shows leader replication bytes-out equal to twice the produced bytes.

### [To Do] X-AUTO-0015 Load input: per-partition produce and fetch request rates per window
P0 · M17 · Task · deps: X-AUTO-0010
Done when: A multi-partition request counts once per partition it touches; produce and fetch tracked separately; test with scripted requests matches expected counts.

### [To Do] X-AUTO-0016 Load input: per-partition local log size and per-log-dir total and usable bytes
P0 · M17 · Task · deps: X-AUTO-0010
Done when: partition size counts local segments only (tiered bytes reported separately and excluded from Disk); dir capacity read through the fs trait; offline dirs reported as unavailable, not as empty.

### [To Do] X-AUTO-0017 Load input: broker process CPU utilization per window through an injectable `CpuSource` (Linux `/proc/self/stat`, macOS `getrusage`)
P0 · M17 · Task · deps: X-AUTO-0010
Done when: utilization normalized to the configured core count; `ManualCpuSource` drives tests; unparseable `/proc` content yields a typed error and an `autopilot_sample_invalid` event, never a panic.

### [To Do] X-AUTO-0018 Per-partition CPU estimation: attribute broker CPU to partitions with a linear model over bytes-in, bytes-out, replication bytes and request rates
P0 · M17 · Task · deps: X-AUTO-0017, X-AUTO-0012, X-AUTO-0013, X-AUTO-0014, X-AUTO-0015
Done when: coefficients fitted per broker by least squares over recent windows, with fallback to configured defaults when the fit is ill-conditioned; property test: attributed CPU sums to measured broker CPU; the leader-only share moves with leadership in the model.

### [To Do] X-AUTO-0019 Broker capacity resolution for CPU, network in, network out and per-dir disk, with per-broker overrides
P0 · M17 · Task · deps: X-AUTO-0008, CFG-F11, CFG-F04 · labels: config
Done when: keys `alluvium.autopilot.capacity.cpu.cores`, `alluvium.autopilot.capacity.network.in.bytes.per.sec` and `alluvium.autopilot.capacity.network.out.bytes.per.sec` registered via CFG-F11 with per-broker dynamic overrides; disk capacity defaults to filesystem size; a broker with unknown capacity is marked `capacity_unknown` and capacity goals skip it with a reason; tests for defaults and overrides.

### [To Do] X-AUTO-0020 Load sample transport from every broker to the optimizer per the ADR, bounded per broker and rejecting samples from stale broker epochs
P0 · M17 · Task · deps: X-AUTO-0010, X-AUTO-0006 · labels: security
Done when: samples from every live broker reach the optimizer within one window in an integration test; samples with an epoch older than the registered broker epoch are dropped and counted in `alluvium_autopilot_samples_rejected_total{reason}`; per-broker backlog bounded.

### [To Do] X-AUTO-0021 Windowed aggregation: ring of `alluvium.autopilot.load.num.windows` windows of `alluvium.autopilot.load.window.ms` per partition with per-window completeness
P0 · M17 · Task · deps: X-AUTO-0008, X-AUTO-0010, CFG-F11
Done when: aggregator is pure (inputs are samples and ticks); memory bounded at windows x partitions x fixed entry size, asserted at 200k partitions; windows with missing samples are marked incomplete, never zero-filled; keys registered via CFG-F11.

### [To Do] X-AUTO-0022 Model readiness: no model when the valid-partition ratio is below `alluvium.autopilot.load.min.valid.partition.ratio` or fewer than `alluvium.autopilot.load.min.valid.windows` windows are complete
P0 · M17 · Task · deps: X-AUTO-0021
Done when: returns `ModelNotReady { reason, ratio, missing_brokers }`; rate-limited `autopilot_model_not_ready` event; tests for a new cluster, a broker missing samples and a freshly created topic.

### [To Do] X-AUTO-0023 Load model metrics: `alluvium_autopilot_model_completeness_ratio`, `alluvium_autopilot_model_build_seconds`, `alluvium_autopilot_model_memory_bytes`, `alluvium_autopilot_sample_age_seconds`
P0 · M17 · Task · deps: X-AUTO-0022, X-AUTO-0007 · labels: observability
Done when: metrics registered on the explicit registry with bounded labels; integration test asserts values after a model build; rows added to docs/metrics.md.

### [To Do] X-AUTO-0024 Load model benchmarks: aggregation throughput and model build time at 200k partitions
P1 · M17 · Task · deps: X-AUTO-0021, X-AUTO-0009 · labels: perf
Done when: criterion benches in `crates/alluvium-autopilot/benches/load_model.rs` tracked by the TEST-F11 benchmark job; results within the budget recorded in the ADR.

### [To Do] X-AUTO-0025 docs/autopilot/load-model.md: every load input, its source, units, windowing, CPU estimation and known inaccuracies
P1 · M17 · Task · deps: X-AUTO-0018, X-AUTO-0019, X-AUTO-0021 · labels: docs
Done when: page lists each input with its code path, config keys and metric; linked from the autopilot guide; reviewed against the code.

## [To Do] X-AUTO-F03 Goal-based optimizer
P0 · M17 · Feature · deps: X-AUTO-F02
Scope: Rack, capacity, replica, leader and disk balance goals.

### [To Do] X-AUTO-0026 `Goal` interface and `GoalSet`: `is_hard`, `stats`, `accepts(action)`, `optimize`, ordered by `alluvium.autopilot.goals` with `alluvium.autopilot.hard.goals`
P0 · M17 · Task · deps: none
Done when: contract documented in `//!` docs; config validation rejects unknown goal names, duplicates and a hard goal listed after a soft goal; keys registered via CFG-F11.

### [To Do] X-AUTO-0027 `Proposal` type: ordered actions (replica move, leadership move, intra-broker move), per-goal before/after stats, bytes to move, source image offset, stable JSON rendering
P0 · M17 · Task · deps: X-AUTO-0026
Done when: JSON schema documented in docs/autopilot/proposal.md; golden-file rendering test; the proposal records the metadata image offset it was computed from.

### [To Do] X-AUTO-0028 Optimizer engine: goals optimized in priority order, an action accepted only if every already-optimized goal accepts it, deterministic tie-breaking
P0 · M17 · Task · deps: X-AUTO-0027
Done when: `propose` is pure; identical inputs yield byte-identical proposals (property test); actions capped by `alluvium.autopilot.optimizer.max.actions` and work capped by `alluvium.autopilot.optimizer.max.steps` (no clock in the optimizer).

### [To Do] X-AUTO-0029 Optimization constraints: excluded topics regex, brokers excluded from replica moves or leadership, frozen partitions under reassignment, offline and under-min-ISR partitions untouched
P0 · M17 · Task · deps: X-AUTO-0028
Done when: one unit test per constraint proves no action violates it; internal topics (`__consumer_offsets`, `__transaction_state`, `__share_group_state`) move only when `alluvium.autopilot.optimizer.move.internal.topics=true`; invalid regexes rejected at config validation.

### [To Do] X-AUTO-0030 Hard-goal failure: `OptimizationFailure { goal, reason, brokers }` when a hard goal cannot be met, with no partial proposal
P0 · M17 · Task · deps: X-AUTO-0028
Done when: tests for RF greater than rack count, insufficient disk capacity and all brokers excluded each return the named failure and emit `autopilot_optimization_failed` with the reason.

### [To Do] X-AUTO-0031 Goal: rack awareness (hard) - no two replicas of a partition on one rack when racks >= RF
P0 · M17 · Task · deps: X-AUTO-0028 · ref: metadata/src/main/java/org/apache/kafka/metadata/placement/StripedReplicaPlacer.java @4.4.0
Done when: fixes violations with replica moves only; property test: after optimization no partition has two replicas on one rack when racks >= RF; actions proposed by later goals that would break it are rejected.

### [To Do] X-AUTO-0032 Goal: replica capacity (hard) - at most `alluvium.autopilot.goal.replica.capacity.max.replicas.per.broker` replicas per broker
P0 · M17 · Task · deps: X-AUTO-0028
Done when: over-capacity brokers shed replicas to brokers below the cap; property test: no broker exceeds the cap after optimization; failure reported when total replicas exceed cluster capacity.

### [To Do] X-AUTO-0033 Goal: disk capacity (hard) - broker and log dir usage at most capacity x `alluvium.autopilot.goal.capacity.threshold.disk`
P0 · M17 · Task · deps: X-AUTO-0028
Done when: moves the largest-impact replicas off over-threshold brokers and dirs; property test: no broker or dir over threshold after optimization; brokers with unknown capacity skipped with a reason.

### [To Do] X-AUTO-0034 Goal: network-in capacity (hard) - produce plus replication bytes-in at most capacity x `alluvium.autopilot.goal.capacity.threshold.network.in`
P0 · M17 · Task · deps: X-AUTO-0028
Done when: uses leadership moves before replica moves where that lowers bytes-in; property test: no broker over threshold after optimization on generated clusters where a solution exists.

### [To Do] X-AUTO-0035 Goal: network-out capacity (hard) - consumer plus replication bytes-out at most capacity x `alluvium.autopilot.goal.capacity.threshold.network.out`
P0 · M17 · Task · deps: X-AUTO-0028
Done when: prefers leadership moves (consumer bytes-out follows the leader); property test: no broker over threshold after optimization on generated clusters where a solution exists.

### [To Do] X-AUTO-0036 Goal: CPU capacity (hard) - estimated CPU at most cores x `alluvium.autopilot.goal.capacity.threshold.cpu`
P0 · M17 · Task · deps: X-AUTO-0028
Done when: uses the per-partition CPU estimates; property test: no broker over threshold after optimization; brokers with an ill-conditioned CPU model flagged in goal stats.

### [To Do] X-AUTO-0037 Goal: replica distribution (soft) - replica counts within `alluvium.autopilot.goal.replica.balance.percentage` of the cluster mean
P0 · M17 · Task · deps: X-AUTO-0028
Done when: skewed fixture reaches the band or reports the blocking higher-priority goal; stats before/after in the proposal; never worsens any hard goal (property test).

### [To Do] X-AUTO-0038 Goal: leader distribution (soft) - leader counts within `alluvium.autopilot.goal.leader.balance.percentage`, using leadership swaps before replica moves
P0 · M17 · Task · deps: X-AUTO-0028
Done when: fixture with all leaders on one broker reaches the band using only leadership moves when ISR allows; property test: leadership moves only target in-sync replicas.

### [To Do] X-AUTO-0039 Goal: topic replica distribution (soft) - each topic's replicas spread across brokers within `alluvium.autopilot.goal.topic.replica.balance.percentage`
P0 · M17 · Task · deps: X-AUTO-0028
Done when: fixture with a topic packed on a few brokers is spread within the band; topics with fewer replicas than brokers handled without oscillation; excluded topics untouched.

### [To Do] X-AUTO-0040 Goal: leader bytes-in distribution (soft) - leader produce bytes-in balanced across brokers within `alluvium.autopilot.goal.leader.bytes.in.balance.percentage`
P0 · M17 · Task · deps: X-AUTO-0028
Done when: fixture with hot leaders on one broker is balanced via leadership moves first; stats report the max/mean ratio before and after.

### [To Do] X-AUTO-0041 Goal: preferred leader election - the first replica in the assignment leads wherever it is in ISR, as Kafka's preferred-leader semantics define
P0 · M17 · Task · deps: X-AUTO-0028 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java maybeBalancePartitionLeaders @4.4.0
Done when: produces leadership actions only where the preferred replica is in ISR and not fenced; ported cases from Kafka's imbalance check agree on which partitions are imbalanced.

### [To Do] X-AUTO-0042 Goal: intra-broker disk balance (soft, JBOD) - usage across a broker's log dirs within `alluvium.autopilot.goal.intra.broker.disk.balance.percentage` using intra-broker moves only
P0 · M17 · Task · deps: X-AUTO-0028, STOR-F22
Done when: JBOD fixture with one full dir is balanced without inter-broker moves; offline dirs never chosen as targets; single-dir brokers produce no actions.

### [To Do] X-AUTO-0043 Goal: resource usage distribution (soft) for Disk, NetworkIn, NetworkOut and Cpu, one implementation parameterized by `Resource`, within `alluvium.autopilot.goal.resource.balance.percentage`
P1 · M17 · Task · deps: X-AUTO-0028
Done when: each resource variant balanced on a skewed fixture; four goal names (`disk_usage_distribution`, ...) registered; property test: never violates any hard goal.

### [To Do] X-AUTO-0044 Seeded synthetic cluster generator for autopilot tests: skewed load, rack imbalance, JBOD, heterogeneous capacity
P0 · M17 · Task · deps: X-AUTO-0008 · labels: test
Done when: same seed reproduces identical clusters; generator used by optimizer property tests and benchmarks; failing seeds print a replay command.

### [To Do] X-AUTO-0045 Optimizer property suite: hard goals never violated, soft goal stats never worse, constraints respected, determinism
P0 · M17 · Task · deps: X-AUTO-0044, X-AUTO-0029, X-AUTO-0030, X-AUTO-0031, X-AUTO-0032, X-AUTO-0033, X-AUTO-0034, X-AUTO-0035, X-AUTO-0036, X-AUTO-0037, X-AUTO-0038, X-AUTO-0039, X-AUTO-0040, X-AUTO-0041, X-AUTO-0042 · labels: test
Done when: at least 1,000 generated cases per CI run with persisted proptest regressions; applying a proposal to the model and re-running yields no further actions (idempotence).

### [To Do] X-AUTO-0046 Optimizer benchmark: proposal time and memory for 10, 100 and 300 brokers with up to 200k replicas, with a budget gate in the benchmark job
P1 · M17 · Task · deps: X-AUTO-0044, X-AUTO-0028 · labels: perf
Done when: criterion benches in `crates/alluvium-autopilot/benches/optimizer.rs`; the benchmark job fails when time or memory exceeds the ADR budget by more than the configured tolerance.

### [To Do] X-AUTO-0047 Optimizer observability: `autopilot_proposal_computed` event (goal stats, action counts, reason), `alluvium_autopilot_goal_violated{goal}`, `alluvium_autopilot_optimization_seconds`
P0 · M17 · Task · deps: X-AUTO-0028, X-AUTO-0007 · labels: observability
Done when: event and metrics emitted by the driver for each computation; flight recorder entry per proposal; metric rows in docs/metrics.md; test asserts values on a fixture.

### [To Do] X-AUTO-0048 docs/autopilot/goals.md: each goal's definition, hard or soft default, thresholds, config keys and the default priority order
P1 · M17 · Task · deps: X-AUTO-0045 · labels: docs
Done when: every registered goal name is documented (a test compares the goal registry with the page headings); examples of before/after stats.

## [To Do] X-AUTO-F04 Plan execution
P0 · M17 · Feature · deps: X-AUTO-F03
Scope: Throttled reassignment, concurrency limits, cancellation.

### [To Do] X-AUTO-0049 Alluvium metadata records `AutopilotPlanRecord` and `AutopilotTaskRecord` (plan id, task id, action, state, reason), gated by an Alluvium feature level
P0 · M17 · Task · deps: META-F06
Done when: records added with Alluvium-reserved ids following the META-F07 conventions; replayed into an autopilot section of the metadata image; rejected unless the feature level is finalized; covered by the metadata record round-trip tests and fuzz target.

### [To Do] X-AUTO-0050 Executor state machine (sans-I/O): tasks move Pending -> InProgress -> Completed, Aborted or Dead; inputs are image deltas and ticks, outputs are reassignment and election submissions
P0 · M17 · Task · deps: X-AUTO-0049
Done when: unit test per transition; illegal transitions are invariant violations; each transition returns `TaskTransition { reason }` that the driver records in the flight recorder.

### [To Do] X-AUTO-0051 Execution step: inter-broker replica movement batches limited by `alluvium.autopilot.execution.max.partition.movements.per.broker` and `alluvium.autopilot.execution.max.inter.broker.movements`
P0 · M17 · Task · deps: X-AUTO-0050, CTRL-F10 · ref: KIP-455
Done when: submissions go through the controller's partition reassignment path with the autopilot principal; property test: in-flight moves never exceed either limit; slots free as tasks complete.

### [To Do] X-AUTO-0052 Execution step: leadership movements by reordering replicas and running a preferred election, limited by `alluvium.autopilot.execution.max.leadership.movements`
P0 · M17 · Task · deps: X-AUTO-0050, CTRL-F08 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java electLeaders @4.4.0
Done when: A partition's leadership task runs after its replica move completes; elections use PREFERRED semantics; test shows leader changes per tick never exceed the limit.

### [To Do] X-AUTO-0053 Execution step: intra-broker log dir moves with AlterReplicaLogDirs semantics, limited by `alluvium.autopilot.execution.max.intra.broker.movements`
P0 · M17 · Task · deps: X-AUTO-0050, STOR-F22
Done when: future-log copy and swap observed through metadata and broker state; limit respected per broker; moves to offline dirs are rejected with a reason.

### [To Do] X-AUTO-0054 Execution step: replication throttles for moving replicas, from `alluvium.autopilot.execution.throttle.bytes.per.sec`, removed when the plan ends
P0 · M17 · Task · deps: X-AUTO-0051, REPL-F16, CFG-F04 · ref: tools/src/main/java/org/apache/kafka/tools/reassign/ReassignPartitionsCommand.java calculateLeaderThrottles, calculateFollowerThrottles, clearAllThrottles @4.4.0
Done when: throttled-replica lists computed as ReassignPartitionsCommand does (ported test cases); broker `leader.replication.throttled.rate`/`follower.replication.throttled.rate` set and cleared; a test fails if any autopilot-set throttle remains after completion, cancellation or abort.

### [To Do] X-AUTO-0055 Progress tracking from the metadata image: task complete when target replicas are all in ISR and adding/removing sets are empty; bytes remaining and ETA per plan
P0 · M17 · Task · deps: X-AUTO-0050
Done when: progress derived only from image deltas (no polling); ETA from observed copy rate; test with a scripted image sequence.

### [To Do] X-AUTO-0056 Stuck and dead task detection: per-task timeout `alluvium.autopilot.execution.task.timeout.ms` and dead brokers in the target set mark the task Dead and cancel its reassignment
P0 · M17 · Task · deps: X-AUTO-0055
Done when: task becomes Dead with reason `timeout` or `target_broker_dead`; its reassignment is cancelled; other tasks continue; test for each reason.

### [To Do] X-AUTO-0057 Plan cancellation: stop scheduling, cancel in-flight reassignments (reverting to original replicas), remove throttles, record `autopilot_plan_cancelled` with reason
P0 · M17 · Task · deps: X-AUTO-0051, X-AUTO-0054 · ref: KIP-455
Done when: cancelling mid-plan leaves every partition with its original or completed target assignment, never a mix of adding replicas; throttles cleared; event emitted.

### [To Do] X-AUTO-0058 Controller failover: the new active controller rebuilds executor state from the autopilot image and resumes or aborts per `alluvium.autopilot.execution.resume.on.failover`
P0 · M17 · Task · deps: X-AUTO-0049, X-AUTO-0055
Done when: simulation with failover mid-plan: plan resumes without duplicate submissions, or aborts cleanly with reason `controller_failover`; no task is lost.

### [To Do] X-AUTO-0059 Coexistence with operator-initiated reassignments: externally reassigned partitions are excluded, and an external reassignment of a planned partition aborts that task with reason `external_reassignment`
P0 · M17 · Task · deps: X-AUTO-0050
Done when: test: a reassignment from kafka-reassign-partitions.sh during a plan is not cancelled or overridden by autopilot; affected task aborted with the reason.

### [To Do] X-AUTO-0060 Executor error mapping: reassignment and election error codes to typed `ExecutionError` variants and task outcomes
P0 · M17 · Task · deps: X-AUTO-0051, X-AUTO-0052
Done when: covers `INVALID_REPLICA_ASSIGNMENT`, `NO_REASSIGNMENT_IN_PROGRESS`, `UNKNOWN_TOPIC_ID`, `NOT_CONTROLLER`, `ELECTION_NOT_NEEDED` and `PREFERRED_LEADER_NOT_AVAILABLE` in one exhaustive match with no wildcard arm; each error has a test with its resulting task state (retry, Dead or Completed).

### [To Do] X-AUTO-0061 Executor observability: events `autopilot_task_state_changed` and `autopilot_plan_finished` plus in-flight, bytes-remaining and outcome metrics
P0 · M17 · Task · deps: X-AUTO-0050, X-AUTO-0007 · labels: observability
Done when: metrics `alluvium_autopilot_inflight_movements{kind}`, `alluvium_autopilot_bytes_remaining` and `alluvium_autopilot_tasks_total{outcome}` registered; events carry plan id, task id, topic, partition and reason; metric rows in docs/metrics.md; integration test asserts values during a plan.

### [To Do] X-AUTO-0062 Executor simulation: plans under broker crashes, controller failover and network partitions
P0 · M17 · Task · deps: X-AUTO-0057, X-AUTO-0058, X-AUTO-0056, TEST-F06 · labels: test
Done when: thousands of seeds in nightly runs; checkers: replica count never below RF when a plan ends, no acknowledged acks=all write lost, every autopilot throttle cleared, no task stuck InProgress at quiescence.

### [To Do] X-AUTO-0063 docs/autopilot/execution.md: execution steps, concurrency limits, throttles, cancellation and failover behavior
P1 · M17 · Task · deps: X-AUTO-0057, X-AUTO-0058 · labels: docs
Done when: every execution config key and event is documented with an example plan walkthrough.

## [To Do] X-AUTO-F05 Leader balancing
P1 · M17 · Feature · deps: X-AUTO-F03
Scope: Load-aware leadership balancing.

### [To Do] X-AUTO-0064 Leadership-only proposals: optimize leader distribution, leader bytes-in and preferred leader goals with leadership moves among in-sync replicas only
P1 · M17 · Task · deps: none
Done when: proposal contains no replica moves; fixture with skewed leadership is balanced; partitions with a single in-sync replica are skipped with a reason.

### [To Do] X-AUTO-0065 Periodic leader balancing: `alluvium.autopilot.leader.balance.interval.ms` and trigger threshold `alluvium.autopilot.leader.balance.trigger.percentage`
P1 · M17 · Task · deps: X-AUTO-0064 · labels: config
Done when: no proposal while imbalance is under the threshold; keys registered via CFG-F11; test with the simulated clock.

### [To Do] X-AUTO-0066 Anti-oscillation: minimum leadership tenure `alluvium.autopilot.leader.balance.min.tenure.ms` and a minimum improvement before a leader moves again
P1 · M17 · Task · deps: X-AUTO-0064
Done when: property test over noisy load sequences: no partition changes leader more than once per tenure period.

### [To Do] X-AUTO-0067 Coexistence with `auto.leader.rebalance.enable`: autopilot reorders replicas so the controller's preferred-leader rebalance converges to the same leaders
P1 · M17 · Task · deps: X-AUTO-0064, CTRL-F08 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java maybeBalancePartitionLeaders @4.4.0
Done when: simulation with both enabled shows no leadership ping-pong; with autopilot leader balancing enabled the controller's imbalance check finds nothing to do at quiescence.

### [To Do] X-AUTO-0068 Leader eligibility: only ISR members, never ELR-only, fenced or controlled-shutdown brokers, and never an unclean election
P1 · M17 · Task · deps: X-AUTO-0064
Done when: unit tests for each exclusion; `unclean.leader.election.enable` is never changed by autopilot.

### [To Do] X-AUTO-0069 Leadership move rate limit per interval (`alluvium.autopilot.leader.balance.max.moves`) to bound client metadata refreshes and NOT_LEADER_OR_FOLLOWER retries
P1 · M17 · Task · deps: X-AUTO-0065
Done when: moves per interval never exceed the limit (property test); remaining moves carried to the next interval.

### [To Do] X-AUTO-0070 Leader balancing observability: event `autopilot_leadership_moved` (old leader, new leader, reason), metrics `alluvium_autopilot_leader_imbalance_ratio` and `alluvium_autopilot_leadership_moves_total`
P1 · M17 · Task · deps: X-AUTO-0064, X-AUTO-0007 · labels: observability
Done when: event and metrics asserted in an integration test; rows in docs/metrics.md and docs/events.md.

### [To Do] X-AUTO-0071 Leader balancing simulation: converges under steady load, no flapping over 10k steps with load noise, safe across broker restarts
P1 · M17 · Task · deps: X-AUTO-0066, X-AUTO-0069, X-AUTO-0068, TEST-F06 · labels: test
Done when: seeded simulation with checkers for one leader per epoch, tenure respected and convergence within the expected number of intervals.

### [To Do] X-AUTO-0072 Compatibility test: Java client and librdkafka producers during leader balancing see only retriable NOT_LEADER_OR_FOLLOWER and lose no acks=all records
P1 · M17 · Task · deps: X-AUTO-0071, TEST-F08 · labels: test
Done when: `cargo xtask compat` scenario runs continuous produce and consume while leaders move; zero lost or duplicated acks=all records with idempotence on.

### [To Do] X-AUTO-0073 Leader balancing section in the autopilot guide and its relation to `auto.leader.rebalance.enable` and `leader.imbalance.per.broker.percentage`
P2 · M17 · Task · deps: X-AUTO-0067 · labels: docs
Done when: documents recommended settings when both are on and when to turn Kafka's rebalance off.

## [To Do] X-AUTO-F06 Self-healing
P0 · M17 · Feature · deps: X-AUTO-F04
Scope: Re-replication after broker or disk failure, after a grace period.

### [To Do] X-AUTO-0074 Anomaly framework (sans-I/O): `Anomaly` enum (BrokerFailure, DiskFailure, GoalViolation, SlowBroker) with detection time, grace deadline and fix state
P0 · M17 · Task · deps: none
Done when: unit tests for detection, grace expiry and resolution transitions using the injected clock; anomalies keyed deterministically.

### [To Do] X-AUTO-0075 Trigger: broker failure - broker fenced or unregistered longer than `alluvium.autopilot.self.healing.broker.failure.grace.ms` leads to re-replication of its replicas
P0 · M17 · Task · deps: X-AUTO-0074
Done when: fix proposal moves only the failed broker's replicas, honouring rack and capacity goals; test with the simulated clock just before and after the grace period.

### [To Do] X-AUTO-0076 Trigger: log directory failure - replicas on an offline dir moved to healthy dirs or brokers after `alluvium.autopilot.self.healing.disk.failure.grace.ms`
P0 · M17 · Task · deps: X-AUTO-0074, CTRL-F20
Done when: prefers healthy dirs on the same broker when capacity allows; test with a JBOD fixture and an offline dir record.

### [To Do] X-AUTO-0077 Trigger: goal violation - check every `alluvium.autopilot.self.healing.goal.violation.check.interval.ms`; a violated hard goal or soft goal over threshold leads to a rebalance proposal
P0 · M17 · Task · deps: X-AUTO-0074
Done when: no proposal when only goals listed in `alluvium.autopilot.self.healing.ignored.goals` are violated; test per case.

### [To Do] X-AUTO-0078 Trigger: slow broker - produce local time and log flush latency outliers versus the cluster median for `alluvium.autopilot.self.healing.slow.broker.windows` windows lead to leadership demotion
P2 · M17 · Task · deps: X-AUTO-0074
Done when: only leadership moves (no data); a broker recovering is re-promoted after the same window count; test with scripted latency samples.

### [To Do] X-AUTO-0079 Grace and flap suppression: recovery within the grace period cancels the pending fix; repeated flaps back off exponentially up to `alluvium.autopilot.self.healing.max.backoff.ms`
P0 · M17 · Task · deps: X-AUTO-0075, X-AUTO-0076
Done when: tests for crash-and-return within grace (no moves) and a broker flapping five times (backoff grows, bounded).

### [To Do] X-AUTO-0080 Safety limits: no self-healing when more than `alluvium.autopilot.self.healing.max.failed.brokers` brokers are failed or the controller quorum is unstable; anomaly re-validated before execution
P0 · M17 · Task · deps: X-AUTO-0075 · labels: security
Done when: simulation of a network partition isolating half the brokers triggers no data movement; event `autopilot_self_healing_suppressed` with reason.

### [To Do] X-AUTO-0081 Self-healing goes through guardrails: mode `recommend` only records the fix, rate limits and pause apply
P0 · M17 · Task · deps: X-AUTO-0077, X-AUTO-0141, X-AUTO-0143, X-AUTO-0144
Done when: test per mode; a paused autopilot records the anomaly but executes nothing.

### [To Do] X-AUTO-0082 Self-healing observability: events `autopilot_anomaly_detected`, `autopilot_anomaly_fix_started`, `autopilot_anomaly_resolved` with reasons; metric `alluvium_autopilot_anomalies{type,state}`
P0 · M17 · Task · deps: X-AUTO-0074, X-AUTO-0007 · labels: observability
Done when: events and metrics asserted in tests; flight recorder entries for each anomaly transition.

### [To Do] X-AUTO-0083 Self-healing simulation: broker crash restores RF after grace; crash-and-return within grace causes no moves; mass failure causes no healing; JBOD dir failure moves its replicas
P0 · M17 · Task · deps: X-AUTO-0079, X-AUTO-0080, X-AUTO-0081, TEST-F06 · labels: test
Done when: each scenario is a seeded simulation test with invariant checkers; nightly runs over many seeds.

### [To Do] X-AUTO-0084 Self-healing documentation: triggers, grace periods, safety limits and a runbook for disabling self-healing during incidents
P1 · M17 · Task · deps: X-AUTO-0080 · labels: docs
Done when: every self-healing key and event documented; runbook linked from the alert for unresolved anomalies.

## [To Do] X-AUTO-F07 Broker add and remove
P0 · M17 · Feature · deps: X-AUTO-F04
Scope: Decommission and onboarding workflows.

### [To Do] X-AUTO-0085 Broker workflow state machine: `AddBroker`, `RemoveBroker` and `DemoteBroker` workflows with states persisted in the autopilot metadata records
P0 · M17 · Task · deps: none
Done when: unit test per transition; workflows survive controller failover (replay test); one active workflow per broker.

### [To Do] X-AUTO-0086 Add workflow (onboarding): move load only onto the added brokers until they reach the balance target, with no moves among existing brokers
P0 · M17 · Task · deps: X-AUTO-0085
Done when: proposal destinations are only new brokers; fixture with one new broker reaches the replica and leader distribution bands.

### [To Do] X-AUTO-0087 Remove workflow (decommission): move leadership off first, then every replica; refuse with a reason when the remaining brokers cannot satisfy RF or hard goals
P0 · M17 · Task · deps: X-AUTO-0085
Done when: refusal returns `insufficient_brokers` or the violated goal; accepted workflow drains the broker; test for each.

### [To Do] X-AUTO-0088 Decommission completion: report the broker empty; UnregisterBroker stays an explicit operator step unless `alluvium.autopilot.broker.remove.unregister=true`
P0 · M17 · Task · deps: X-AUTO-0087
Done when: workflow ends in `Drained`; with the key set, the broker is unregistered after shutdown; test for both.

### [To Do] X-AUTO-0089 Demote workflow: move all leadership off a broker and make it ineligible as preferred leader without data movement; undo restores eligibility
P0 · M17 · Task · deps: X-AUTO-0085
Done when: broker ends with zero leaders and keeps its replicas; undo returns leadership through preferred election; test.

### [To Do] X-AUTO-0090 New-broker detection: a newly registered broker with no replicas starts an Add workflow when `alluvium.autopilot.broker.add.auto=true`
P1 · M17 · Task · deps: X-AUTO-0086
Done when: test with a broker registration in the image; nothing happens with the key unset (default false).

### [To Do] X-AUTO-0091 Native admin API for workflows (start, describe, cancel) with ALTER on CLUSTER to start or cancel and DESCRIBE on CLUSTER to read
P0 · M17 · Task · deps: X-AUTO-0086, X-AUTO-0087, X-AUTO-0089, X-NATIVE-F08, SEC-F08 · labels: security
Done when: authorization denials return the native permission error and emit `authz_decision`; contract tests for every operation.

### [To Do] X-AUTO-0092 Kafka-compatible trigger: per-broker dynamic config `alluvium.autopilot.broker.workflow` (`none`, `remove`, `demote`) settable with kafka-configs.sh
P2 · M17 · Task · deps: X-AUTO-0087, X-AUTO-0089, CFG-F04
Done when: setting the key starts the workflow; DescribeConfigs shows it; ALTER_CONFIGS on CLUSTER required as for any broker config; compatibility test with kafka-configs.sh.

### [To Do] X-AUTO-0093 `alluvium autopilot broker add|remove|demote|status|cancel` CLI over the native API, with `--json`
P1 · M17 · Task · deps: X-AUTO-0091, X-NATIVE-F14
Done when: CLI integration tests for every subcommand; exit codes documented.

### [To Do] X-AUTO-0094 Workflow observability: event `autopilot_broker_workflow_changed`, metric `alluvium_autopilot_broker_workflow_progress_ratio{workflow}`
P0 · M17 · Task · deps: X-AUTO-0085, X-AUTO-0007 · labels: observability
Done when: events carry broker id, workflow, state and reason; metric asserted during a decommission test.

### [To Do] X-AUTO-0095 Workflow simulation: decommission under produce load ends with zero replicas and no acks=all loss; add reaches the balance target; demote ends with zero leaders
P0 · M17 · Task · deps: X-AUTO-0088, X-AUTO-0086, X-AUTO-0089, TEST-F06 · labels: test
Done when: seeded simulations with checkers; includes a controller failover during each workflow.

### [To Do] X-AUTO-0096 Runbooks: scaling out, decommissioning and maintenance demotion with autopilot, replacing kafka-reassign-partitions.sh procedures
P1 · M17 · Task · deps: X-AUTO-0093 · labels: docs
Done when: step-by-step runbooks with expected events and metrics; linked from the operations guide.

## [To Do] X-AUTO-F08 Hot partition handling
P1 · M17 · Feature · deps: X-AUTO-F02, X-ELASTIC-F12
Scope: Detect hot partitions and trigger splits.

### [To Do] X-AUTO-0097 Hot partition detector: bytes-in or request rate above `alluvium.autopilot.hot.partition.threshold.ratio` x topic median for `alluvium.autopilot.hot.partition.min.windows` windows
P1 · M17 · Task · deps: none
Done when: unit tests with scripted load windows: single spike ignored, sustained skew detected, tiny topics (fewer than three partitions) use the cluster median.

### [To Do] X-AUTO-0098 Remedy for classic topics: relieve the hosting broker by moving other replicas or leadership away, never splitting
P1 · M17 · Task · deps: X-AUTO-0097
Done when: proposal reduces the hosting broker's load while the hot partition stays; no split request is ever issued for a non-elastic topic (test).

### [To Do] X-AUTO-0099 Remedy for elastic topics: request a split through the X-ELASTIC-F12 resize hook with the hot key range
P1 · M17 · Task · deps: X-AUTO-0097
Done when: split requested with topic, range and reason `hot_partition`; hook errors are retried with backoff; test with an elastic topic fixture.

### [To Do] X-AUTO-0100 Split storm protection: per-topic cooldown `alluvium.autopilot.hot.partition.split.cooldown.ms`, cluster cap `alluvium.autopilot.hot.partition.max.splits.per.hour` and a maximum lineage depth
P1 · M17 · Task · deps: X-AUTO-0099
Done when: property test: split requests never exceed the caps; event `autopilot_split_suppressed` with reason.

### [To Do] X-AUTO-0101 Cold range merge recommendation: adjacent ranges below `alluvium.autopilot.cold.range.threshold.ratio` for `alluvium.autopilot.cold.range.min.windows` windows lead to a merge request through the hook
P2 · M17 · Task · deps: X-AUTO-0099
Done when: merge requested only for adjacent ranges of the same parent lineage; never within the split cooldown; test.

### [To Do] X-AUTO-0102 Observability: `autopilot_hot_partition_detected` event, `alluvium_autopilot_hot_partitions` gauge, `alluvium_autopilot_resize_requests_total{kind,outcome}`
P1 · M17 · Task · deps: X-AUTO-0097, X-AUTO-0007 · labels: observability
Done when: metrics and events asserted in tests; topic-only labels.

### [To Do] X-AUTO-0103 Simulation: a skewed key workload on an elastic topic triggers one split and stabilizes; a classic topic is never split; cooldown prevents repeated splits
P1 · M17 · Task · deps: X-AUTO-0100, X-AUTO-0098, TEST-F06 · labels: test
Done when: seeded simulation with checkers for per-key ordering (from X-ELASTIC) and split counts.

### [To Do] X-AUTO-0104 Docs: hot partition handling, thresholds, and how it relates to elastic partitions
P2 · M17 · Task · deps: X-AUTO-0100 · labels: docs
Done when: page documents each key and event with an example timeline.

## [To Do] X-AUTO-F09 Autoscaling signals
P1 · M17 · Feature · deps: X-AUTO-F02
Scope: Scaling recommendations and hooks.

### [To Do] X-AUTO-0105 Scale-out recommendation: the minimum broker count that satisfies all hard capacity goals at `alluvium.autopilot.autoscaling.target.utilization`
P1 · M17 · Task · deps: none
Done when: pure function over the cluster model; property test: the recommended count satisfies every capacity goal and one fewer does not.

### [To Do] X-AUTO-0106 Scale-in recommendation: which brokers can be removed while RF and hard goals hold with headroom
P1 · M17 · Task · deps: X-AUTO-0105
Done when: names specific brokers (fewest replicas first); never recommends going below the largest RF or rack count; property test.

### [To Do] X-AUTO-0107 Recommendation stabilization: a change is published only after it persists for `alluvium.autopilot.autoscaling.stabilization.window.ms`
P1 · M17 · Task · deps: X-AUTO-0105, X-AUTO-0106
Done when: test with oscillating load shows one stable recommendation; scale-out reacts faster than scale-in per the configured windows.

### [To Do] X-AUTO-0108 Exposure: metric `alluvium_autopilot_recommended_brokers{direction}`, read-only `/debug/autopilot/recommendations` and a native admin endpoint
P1 · M17 · Task · deps: X-AUTO-0107, OBS-F08, X-NATIVE-F08
Done when: all three return the same values in an integration test; DESCRIBE on CLUSTER required for the native endpoint.

### [To Do] X-AUTO-0109 Outbound webhook for recommendations and anomalies: operator-configured HTTPS URL only, HMAC-SHA256 signed payloads, bounded queue, timeout and capped retries
P1 · M17 · Task · deps: X-AUTO-0107 · labels: security
Done when: `alluvium.autopilot.webhook.url` must be https (http only for loopback); secret is a password-type key never logged; full queue drops oldest with a counter; tests for signature, timeout and retry cap.

### [To Do] X-AUTO-0110 Kubernetes scaling integration: HPA and KEDA external-metric configuration examples using the recommendation metric, tested on kind
P2 · M17 · Task · deps: X-AUTO-0108
Done when: example manifests in `deploy/examples/autoscaling/` applied in a kind test that scales a node pool from a recommendation.

### [To Do] X-AUTO-0111 Tests: recommendation property tests and webhook delivery tests against a local HTTPS receiver
P1 · M17 · Task · deps: X-AUTO-0109, X-AUTO-0108 · labels: test
Done when: suite covers signature verification, retry exhaustion and stabilization timing with the simulated clock.

### [To Do] X-AUTO-0112 Docs: autoscaling signals, webhook payload schema and signature verification
P2 · M17 · Task · deps: X-AUTO-0109 · labels: docs
Done when: payload JSON schema published; verification example in Python and Go.

## [To Do] X-AUTO-F10 Kubernetes operator
P1 · M17 · Feature · deps: X-AUTO-F07
Scope: CRDs and reconciler for clusters, topics, users and ACLs.

### [To Do] X-AUTO-0113 Operator design: kube-rs dependency justification, CRD group `alluvium.io` and versions, ownership and adoption rules, upgrade strategy
P1 · M17 · Spike · deps: none
Done when: addendum to the autopilot ADR accepted; dependency justification per docs/standards/security.md (maintenance, unsafe usage, transitive count); CRD list fixed.

### [To Do] X-AUTO-0114 `alluvium-operator` binary crate: controller runtime, leader election via Lease, supervised reconcilers, graceful shutdown
P1 · M17 · Task · deps: X-AUTO-0113
Done when: two operator replicas run with exactly one active (test on kind); shutdown completes in-flight reconciles within a bound.

### [To Do] X-AUTO-0115 CRD `AlluviumCluster`: version, node pools (roles, replicas, storage class and size, resources, rack label), listeners and TLS, config overrides, with OpenAPI schema and CEL validation
P1 · M17 · Task · deps: X-AUTO-0114
Done when: invalid specs (no controller pool, even voter count, unknown listener protocol) rejected by the API server; generated CRD YAML checked in with a drift check.

### [To Do] X-AUTO-0116 CRD `AlluviumTopic`: name, partitions, replication factor, configs, `deletionPolicy` (`retain` default)
P1 · M17 · Task · deps: X-AUTO-0114
Done when: CEL rules reject partitions below 1 and invalid names; schema documents each field; drift check.

### [To Do] X-AUTO-0117 CRD `AlluviumUser`: authentication (SCRAM-SHA-512 with a generated Secret, or mTLS with an issued certificate) and client quotas
P1 · M17 · Task · deps: X-AUTO-0114 · labels: security
Done when: secrets referenced, never inlined; CEL validation of mechanism fields; drift check.

### [To Do] X-AUTO-0118 CRD `AlluviumAcl`: principal, resource type and pattern (literal or prefixed), operations, permission and host
P1 · M17 · Task · deps: X-AUTO-0114 · labels: security
Done when: validation rejects unknown operations and resource types; mirrors Kafka ACL binding fields exactly.

### [To Do] X-AUTO-0119 CRD `AlluviumRebalance`: requested goals and mode, proposal summary in status, approval via `spec.approved`
P1 · M17 · Task · deps: X-AUTO-0114
Done when: status schema includes proposal id, bytes to move and per-goal stats; drift check.

### [To Do] X-AUTO-0120 CRD versioning: `v1alpha1` storage version, schema evolution rules and the conversion webhook plan
P2 · M17 · Task · deps: X-AUTO-0115, X-AUTO-0116, X-AUTO-0117, X-AUTO-0118, X-AUTO-0119
Done when: documented compatibility rules; a CI check fails on non-additive schema changes within a version.

### [To Do] X-AUTO-0121 Cluster reconcile: StatefulSets per node pool, headless services, ConfigMaps, storage formatting init container with a stable cluster id, PodDisruptionBudgets
P1 · M17 · Task · deps: X-AUTO-0115, OPS-F04
Done when: A new `AlluviumCluster` on kind becomes Ready with all nodes registered; re-running reconcile is a no-op (idempotence test).

### [To Do] X-AUTO-0122 Status conditions `Ready`, `Progressing` and `Degraded` with `observedGeneration` and reasons on every CR kind
P1 · M17 · Task · deps: X-AUTO-0121
Done when: each condition transition tested; reasons are stable identifiers documented in the CRD reference.

### [To Do] X-AUTO-0123 Rolling restart for static config changes: one node at a time, waiting for readiness and zero under-replicated partitions, controllers last
P1 · M17 · Task · deps: X-AUTO-0121
Done when: live producer with acks=all sees no lost writes during the roll (kind test); a node that never becomes ready halts the roll with `Degraded`.

### [To Do] X-AUTO-0124 Dynamic config changes applied with IncrementalAlterConfigs without restart when the key is dynamic
P1 · M17 · Task · deps: X-AUTO-0121
Done when: test changes a dynamic key and asserts no pod restarts; static keys still trigger the roll.

### [To Do] X-AUTO-0125 Scale up: new pods register, then an autopilot Add workflow starts and its progress is shown in status
P1 · M17 · Task · deps: X-AUTO-0121
Done when: kind test scales 3 to 4 brokers; status shows workflow progress until balanced.

### [To Do] X-AUTO-0126 Scale down: run the Remove workflow, wait until the broker is drained, unregister it, then delete the pod and, per policy, its PVC
P1 · M17 · Task · deps: X-AUTO-0121
Done when: pod deletion never happens before `Drained`; kind test with a live producer shows no acks=all loss; refusal reasons surface in status.

### [To Do] X-AUTO-0127 Version upgrade: rolling image update, then `metadata.version` finalized with UpdateFeatures only after every node runs the new version
P1 · M17 · Task · deps: X-AUTO-0123
Done when: kind test upgrades across two versions; finalization waits while any node is old; rollback before finalization documented and tested.

### [To Do] X-AUTO-0128 Storage expansion: PVC resize when the storage class allows expansion; shrink requests refused with a status reason
P2 · M17 · Task · deps: X-AUTO-0121
Done when: test with an expandable storage class; shrink results in `Degraded` reason `storage_shrink_unsupported`.

### [To Do] X-AUTO-0129 TLS certificates: cluster CA and node certificates in Secrets or via cert-manager, rotation with hot reload and no restart
P1 · M17 · Task · deps: X-AUTO-0121 · labels: security
Done when: rotation test: certificate replaced, clients reconnect, no pod restart; private keys never logged.

### [To Do] X-AUTO-0130 Topic reconcile: create, increase partitions and alter configs; refuse partition decrease; delete only with `deletionPolicy: delete`; adopt existing topics
P1 · M17 · Task · deps: X-AUTO-0116
Done when: tests for each path, including an unmanaged topic adopted without changes and a decrease refused with a status reason.

### [To Do] X-AUTO-0131 User reconcile: SCRAM credential upsert with AlterUserScramCredentials, generated password Secret, quotas with AlterClientQuotas
P1 · M17 · Task · deps: X-AUTO-0117 · labels: security
Done when: password rotation by Secret change; deleting the CR removes credentials and quotas; tests.

### [To Do] X-AUTO-0132 ACL reconcile: diff desired bindings against DescribeAcls and apply CreateAcls/DeleteAcls, touching only bindings this operator owns
P1 · M17 · Task · deps: X-AUTO-0118 · labels: security
Done when: unmanaged ACLs are never deleted (test); ownership tracking documented.

### [To Do] X-AUTO-0133 Rebalance reconcile: create an autopilot proposal, publish its summary in status, execute after `spec.approved`, reflect progress
P1 · M17 · Task · deps: X-AUTO-0119, X-AUTO-F11
Done when: end-to-end kind test from CR creation to completion; a stale proposal is recomputed before execution.

### [To Do] X-AUTO-0134 Finalizers and deletion order: cluster deletion waits for owned resources; topics, users and ACLs cleaned per deletion policy
P1 · M17 · Task · deps: X-AUTO-0121, X-AUTO-0130, X-AUTO-0131, X-AUTO-0132
Done when: deleting a cluster with managed topics completes without orphaned Kubernetes objects; `retain` keeps data.

### [To Do] X-AUTO-0135 Operator security: least-privilege RBAC, its own Alluvium principal with minimal ACLs, Secrets never logged
P1 · M17 · Task · deps: X-AUTO-0114 · labels: security
Done when: RBAC manifest reviewed against the API calls used (test fails on a forbidden call); log redaction test for Secret contents.

### [To Do] X-AUTO-0136 Operator observability: reconcile spans with CR kind, name and generation, metrics `alluvium_operator_reconcile_total{kind,result}` and `alluvium_operator_reconcile_seconds{kind}`, Kubernetes Events with reasons
P1 · M17 · Task · deps: X-AUTO-0114 · labels: observability
Done when: metrics and Events asserted in tests; field names follow the standard table.

### [To Do] X-AUTO-0137 Reconciler unit tests against a fake API server for every reconcile behavior
P1 · M17 · Task · deps: X-AUTO-0122, X-AUTO-0123, X-AUTO-0126, X-AUTO-0130, X-AUTO-0131, X-AUTO-0132 · labels: test
Done when: each behavior has success, conflict and error-path tests; no real time or sleeps.

### [To Do] X-AUTO-0138 End-to-end operator tests on kind: create, scale up and down, rolling config change and upgrade with live clients
P1 · M17 · Task · deps: X-AUTO-0137, X-AUTO-0127, X-AUTO-0125 · labels: test
Done when: nightly job; zero lost acks=all records and all CRs Ready at the end.

### [To Do] X-AUTO-0139 Operator Helm chart and CRD install manifests published with releases
P1 · M17 · Task · deps: X-AUTO-0114, X-AUTO-0135
Done when: chart lint and install test on kind; CRDs installed and upgraded by the chart.

### [To Do] X-AUTO-0140 Operator guide: generated CRD reference, install, day-2 operations and migration from Strimzi resources
P1 · M17 · Task · deps: X-AUTO-0134, X-AUTO-0139 · labels: docs
Done when: field-by-field mapping from Strimzi `Kafka`, `KafkaTopic`, `KafkaUser` and `KafkaRebalance` to Alluvium CRDs.

## [To Do] X-AUTO-F11 Autopilot guardrails
P0 · M17 · Feature · deps: X-AUTO-F04
Scope: Dry runs, approvals, rate limits, pause.

### [To Do] X-AUTO-0141 Mode enforcement: `recommend` records proposals without executing (dry run), `approve` waits for approval, `auto` executes; downgrading the mode stops new executions
P0 · M17 · Task · deps: none
Done when: test per mode, including a downgrade from `auto` to `recommend` mid-plan (in-flight tasks finish, no new ones start).

### [To Do] X-AUTO-0142 Approval workflow: pending proposals with id and expiry `alluvium.autopilot.approval.timeout.ms`; proposals computed on an image older than `alluvium.autopilot.approval.max.image.lag` are recomputed before execution
P0 · M17 · Task · deps: X-AUTO-0141
Done when: approve, reject and expiry tested; executing an approved but stale proposal is refused with reason `stale_proposal`.

### [To Do] X-AUTO-0143 Data movement limits: `alluvium.autopilot.guardrail.max.bytes.moved.per.hour` and `alluvium.autopilot.guardrail.max.executions.per.day` enforced by the executor
P0 · M17 · Task · deps: X-AUTO-0141
Done when: property test: moved bytes in any sliding hour never exceed the limit; blocked work resumes when budget returns.

### [To Do] X-AUTO-0144 Pause and resume: a global pause stops scheduling; `alluvium.autopilot.pause.cancel.inflight` decides whether in-flight moves are cancelled; pause persists across controller failover
P0 · M17 · Task · deps: X-AUTO-0141
Done when: pause state stored in autopilot records; failover test keeps the pause; both cancel settings tested.

### [To Do] X-AUTO-0145 Cluster-health guard: block execution while offline partitions, under-min-ISR partitions not caused by autopilot, or controller instability exist
P0 · M17 · Task · deps: X-AUTO-0141
Done when: each condition blocks with its reason in a deterministic test; execution resumes after recovery.

### [To Do] X-AUTO-0146 SLO guard: pause execution when produce or fetch p99 total time exceeds `alluvium.autopilot.slo.produce.p99.ms` or `alluvium.autopilot.slo.fetch.p99.ms` for `alluvium.autopilot.slo.windows` windows
P0 · M17 · Task · deps: X-AUTO-0141, SRV-F08
Done when: uses the request phase metrics; resumes after the same number of healthy windows; test with scripted latency inputs.

### [To Do] X-AUTO-0147 Maintenance windows: `alluvium.autopilot.execution.allowed.windows` (UTC day and time ranges) restrict automatic execution
P2 · M17 · Task · deps: X-AUTO-0141
Done when: parser rejects malformed ranges; execution outside windows is blocked with reason `outside_window`; tests across midnight and week boundaries.

### [To Do] X-AUTO-0148 Authorization: DESCRIBE on CLUSTER to read proposals; ALTER on CLUSTER to approve, reject, pause, resume and change mode
P0 · M17 · Task · deps: X-AUTO-0142, X-AUTO-0144, SEC-F08 · labels: security
Done when: denials return CLUSTER_AUTHORIZATION_FAILED on Kafka paths and the native permission error on native paths, with `authz_decision` events; tests per operation.

### [To Do] X-AUTO-0149 Audit: mode changes, approvals, rejections, pauses and plan start and end written to the audit log with principal and reason
P0 · M17 · Task · deps: X-AUTO-0148, X-SECX-F07 · labels: security
Done when: audit records verified in an integration test; no proposal payload beyond ids and stats.

### [To Do] X-AUTO-0150 Guardrail observability: event `autopilot_execution_blocked` with reason (`paused`, `rate_limited`, `unhealthy`, `slo_breach`, `outside_window`, `awaiting_approval`) and metric `alluvium_autopilot_blocked{reason}`
P0 · M17 · Task · deps: X-AUTO-0141, X-AUTO-0007 · labels: observability
Done when: each reason emitted in its guard's test; metric rows documented.

### [To Do] X-AUTO-0151 Guardrail simulation: each guard blocks execution, approval expiry, pause surviving controller failover
P0 · M17 · Task · deps: X-AUTO-0143, X-AUTO-0144, X-AUTO-0145, X-AUTO-0146, X-AUTO-0142, TEST-F06 · labels: test
Done when: seeded simulation tests per guard with checkers that no action executes while blocked.

### [To Do] X-AUTO-0152 Guardrails documentation and the recommended rollout path (`recommend`, then `approve`, then `auto`)
P1 · M17 · Task · deps: X-AUTO-0151 · labels: docs
Done when: every guardrail key and blocked reason documented with an example.

## [To Do] X-AUTO-F12 Autopilot observability, testing and documentation
P0 · M17 · Feature · deps: X-AUTO-F04, TEST-F06
Scope: Metrics, simulation tests, guide.

### [To Do] X-AUTO-0153 Diagnostics endpoint `/debug/autopilot`: mode, model readiness, current proposal, executor tasks, anomalies, workflows and flight recorder (read-only)
P0 · M17 · Task · deps: OBS-F08 · labels: observability
Done when: JSON and human output; secrets redacted; integration test checks each section during a plan.

### [To Do] X-AUTO-0154 Alert rules and dashboard: execution blocked too long, anomaly unresolved past grace, plan stuck, model not ready; Grafana panels and one runbook per alert
P1 · M17 · Task · deps: X-AUTO-0153, OPS-F11 · labels: observability
Done when: rules validated with promtool tests; runbooks linked from alert annotations.

### [To Do] X-AUTO-0155 Debug bundle includes autopilot state (proposal, executor, anomalies, workflows, flight recorder)
P1 · M17 · Task · deps: X-AUTO-0153, CLI-F05
Done when: `alluvium debug-bundle` output contains an `autopilot/` section; redaction test passes.

### [To Do] X-AUTO-0156 Off by default: with default config there is no load sample traffic, no autopilot metadata records, no reassignments, elections or throttle changes by autopilot
P0 · M17 · Task · deps: X-AUTO-F06, X-AUTO-F07, X-AUTO-F09, X-AUTO-F11, TEST-F08 · labels: test
Done when: scripted workload on a default cluster: metadata log contains no autopilot record types and no autopilot-issued changes; autopilot metrics absent or zero; `alluvium.autopilot.mode` reports `off` with source `default`.

### [To Do] X-AUTO-0157 Compatibility and differential suites pass with `alluvium.autopilot.mode=auto` on a balanced cluster (enabled but with nothing to do)
P0 · M17 · Task · deps: X-AUTO-F05, X-AUTO-F06, X-AUTO-F07, X-AUTO-F08, X-AUTO-F11, TEST-F08, TEST-F09 · labels: test
Done when: `cargo xtask compat` passes and differential diffs are unchanged versus mode `off`; the run asserts zero autopilot tasks executed.

### [To Do] X-AUTO-0158 Churn simulation: thousands of seeds with broker crashes, additions, removals and shifting load with autopilot in `auto`
P0 · M17 · Task · deps: X-AUTO-F06, X-AUTO-F07, X-AUTO-F11, TEST-F06 · labels: test
Done when: checkers: no acks=all loss, RF restored, hard goals hold at quiescence, guardrail limits never exceeded; nightly with failing seeds added to the fixed set.

### [To Do] X-AUTO-0159 Chaos test on a real multi-process cluster with autopilot in `auto` under broker kills and network faults
P1 · M17 · Task · deps: TEST-F12 · labels: test
Done when: jepsen-style no-loss and ordering checks pass; autopilot never leaves throttles or half-applied plans after faults end.

### [To Do] X-AUTO-0160 Rebalance impact benchmark: produce and consume p99 during a 100-partition rebalance, with and without throttles
P0 · M17 · Task · deps: TEST-F11 · labels: perf
Done when: openMessaging Benchmark runs in the macro benchmark job; results published; p99 increase within the ADR budget with default throttles.

### [To Do] X-AUTO-0161 Autopilot operator guide `docs/autopilot/README.md`: concepts, modes, goals, execution, self-healing, workflows and troubleshooting
P0 · M17 · Task · deps: X-AUTO-0153 · labels: docs
Done when: guide links every autopilot page; troubleshooting covers each blocked reason and failure event.

### [To Do] X-AUTO-0162 Migration and opt-in guide: from Cruise Control (goal and config mapping, disabling Cruise Control first), staged enablement, rollback to `off`
P0 · M17 · Task · deps: X-AUTO-0161 · labels: docs
Done when: mapping table for every Cruise Control goal named in this epic; rollback procedure tested in a kind or local cluster run.
