# X-TENANT: Multi-tenancy

> Crates: `alluvium-controller`, `alluvium-server`, `alluvium-security` · Kafka counterpart: none (Alluvium enhancement) · Milestones: M14

Namespaces with isolation: transparent name mapping for Kafka clients, per-tenant limits and quotas, fair scheduling, delegated administration and usage metering.

## [To Do] X-TENANT-F01 Tenancy design
P0 · M14 · Feature · deps: META-F07
Scope: Spike and ADR: namespace model and Kafka-client mapping.

### [To Do] X-TENANT-0001 Multi-tenancy ADR: namespace model, Kafka-client name mapping, isolation scope, limits, fairness, metering and opt-in
P0 · M14 · Spike · deps: none
Done when: docs/adr/NNNN-multi-tenancy.md is accepted and decides: flat namespaces identified by `NamespaceId` (TYPE-F10) with a default namespace that keeps plain Kafka names valid; principal-to-namespace resolution; transparent qualification of topics, groups and transactional ids for Kafka clients; topic id isolation; which resources are tenant-scoped; tenant limits, quotas and weighted fair scheduling; delegated administration; metering export; deletion with crypto-shredding; the errors returned to Kafka clients for suspended namespaces and exceeded limits; off-by-default gating; and rejected alternatives (prefix ACL conventions, proxy-based virtual clusters, cluster per tenant).

### [To Do] X-TENANT-0002 Tenancy threat model: cross-tenant access, name and id oracles, noisy neighbors, metric leakage and admin escalation
P0 · M14 · Task · deps: X-TENANT-0001 · labels: security, docs
Done when: docs/standards/security.md gains rows for each threat with its enforcing task; reviewed in the M14 security review.

### [To Do] X-TENANT-0003 Name-bearing field inventory for every Kafka 4.4.0 API with its rewrite rule
P0 · M14 · Task · deps: X-TENANT-0001 · ref: clients/src/main/resources/common/message/ProduceRequest.json entityType @4.4.0
Done when: An ADR appendix lists every request and response field annotated `entityType: topicName`, `groupId` or `transactionalId` in the vendored schemas plus unannotated resource-name fields (ACL and config resources, quota entities), each with its qualification, filtering and error rule; a script check confirms the list matches the schemas.

### [To Do] X-TENANT-0004 Glossary entries for namespace, tenant, qualified name, default namespace, tenant admin and usage record
P1 · M14 · Task · deps: X-TENANT-0001 · labels: docs
Done when: docs/glossary.md defines the terms with links to the ADR; link check passes.

## [To Do] X-TENANT-F02 Namespace lifecycle
P0 · M14 · Feature · deps: X-TENANT-F01
Scope: Create, update, suspend and delete namespaces.

### [To Do] X-TENANT-0005 Controller namespace manager: create, update, suspend, resume and delete with validation
P0 · M14 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/controller/AclControlManager.java @4.4.0
Done when: A sans-I/O manager in `alluvium-controller` validates names (TYPE-F10 rules, reserved `default` and `__alluvium` prefixes, uniqueness), emits META-F07 records, and enforces the state machine Active, Suspended, Deleting; one test per transition and per rejection.

### [To Do] X-TENANT-0006 Namespace state record for Active, Suspended and Deleting with a reason
P0 · M14 · Task · deps: X-TENANT-0005 · labels: decoder
Done when: `NamespaceStateRecord` schema, codec round-trip, replay with monotonic validation and fuzz coverage; reasons are a closed enum shared with events.

### [To Do] X-TENANT-0007 Namespace config resource: an Alluvium config resource type for namespaces stored in ConfigRecord
P0 · M14 · Task · deps: X-TENANT-0005, CFG-F04 · ref: metadata/src/main/resources/common/metadata/ConfigRecord.json @4.4.0
Done when: ConfigRecord with the Alluvium namespace resource type replays into a namespace config map; only registered `alluvium.tenant.*` keys are accepted; Kafka DescribeConfigs never returns this resource type to Kafka clients; tests.

### [To Do] X-TENANT-0008 `NamespacesImage` with lookups by id and name, published to broker components
P0 · M14 · Task · deps: X-TENANT-0006, X-TENANT-0007
Done when: Image built from namespace records and configs; SRV-F14 publishers deliver it in the defined order; lookups are O(1); snapshot round-trip test.

### [To Do] X-TENANT-0009 Default namespace: every pre-existing and unqualified resource belongs to it
P0 · M14 · Task · deps: X-TENANT-0008
Done when: Upgrading a cluster with existing topics, groups and ACLs places them in the default namespace with no record rewrite; Kafka clients see no change; tests.

### [To Do] X-TENANT-0010 Suspend semantics: data-plane and admin requests of a suspended namespace rejected, connections closed
P0 · M14 · Task · deps: X-TENANT-0008
Done when: Requests from principals of a suspended namespace get the ADR error, existing connections are closed within one metadata propagation, and resuming restores service; tests.

### [To Do] X-TENANT-0011 Feature level `alluvium.tenant.version` gating namespace records, downgrade blocked while non-default namespaces exist
P0 · M14 · Task · deps: X-TENANT-0005 · ref: server-common/src/main/java/org/apache/kafka/server/common/MetadataVersion.java @4.4.0
Done when: With level 0 namespace APIs are unavailable and no records are emitted; downgrade with namespaces present fails with INVALID_UPDATE_VERSION; tests.

### [To Do] X-TENANT-0012 Native admin IDL and handlers: Create, Update, Suspend, Resume, Delete, List and Describe namespaces
P0 · M14 · Task · deps: X-TENANT-0005, X-NATIVE-F08
Done when: Handlers require ALTER or DESCRIBE on the cluster (or the namespace for tenant admins per X-TENANT-F08) and forward to the controller; validate-only supported; tests per method.

### [To Do] X-TENANT-0013 `alluvium admin namespace` commands with `--json`
P0 · M14 · Task · deps: X-TENANT-0012, X-NATIVE-F14
Done when: Commands for every lifecycle operation built on X-NATIVE-F14; golden-output tests.

### [To Do] X-TENANT-0014 Namespace lifecycle events and controller flight-recorder entries
P0 · M14 · Task · deps: X-TENANT-0006 · labels: observability
Done when: `namespace_created`, `namespace_updated`, `namespace_suspended`, `namespace_resumed`, `namespace_deleting` and `namespace_deleted` carry namespace id, name and reason; tests.

## [To Do] X-TENANT-F03 Kafka-client name mapping
P0 · M14 · Feature · deps: X-TENANT-F02
Scope: Principal-to-namespace mapping; transparent prefixing of topics, groups and transactional ids.

### [To Do] X-TENANT-0015 Principal-to-namespace resolution at authentication: mapping rules, OAuth claim and listener default
P0 · M14 · Task · deps: SRV-F06, CFG-F11 · labels: config · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/DefaultKafkaPrincipalBuilder.java @4.4.0
Done when: `alluvium.tenant.principal.mapping.rules` (rule syntax like `ssl.principal.mapping.rules`), `alluvium.tenant.oauth.claim` and `alluvium.tenant.listener.<listener>.namespace` resolve a namespace once per session and store it in the request context; precedence tested; keys registered with docs.

### [To Do] X-TENANT-0016 Unmapped principals: `alluvium.tenant.unmapped.principal.policy` = default or reject
P0 · M14 · Task · deps: X-TENANT-0015 · labels: security
Done when: With `reject`, authentication succeeds but every request except ApiVersions fails with the ADR error and the connection closes; with `default` the principal uses the default namespace; tests.

### [To Do] X-TENANT-0017 Codegen: name-rewrite visitors for every field annotated `entityType` topicName, groupId or transactionalId
P0 · M14 · Task · deps: PROTO-F05 · ref: clients/src/main/resources/common/message/ProduceRequest.json entityType @4.4.0
Done when: The protocol generator emits `qualify` and `unqualify` visitors per request and response version; the drift check covers them; a test asserts every annotated field in the inventory is visited.

### [To Do] X-TENANT-0018 Request qualification before handlers for all name-bearing fields
P0 · M14 · Task · deps: X-TENANT-0015, X-TENANT-0017
Done when: Requests from non-default namespaces have names qualified before authorization and dispatch; tests per API with name fields across all supported versions.

### [To Do] X-TENANT-0019 Response unqualification and filtering of other namespaces from list-style responses
P0 · M14 · Task · deps: X-TENANT-0018 · ref: core/src/main/scala/kafka/server/KafkaApis.scala @4.4.0
Done when: Metadata (all topics), ListGroups, ListTransactions, DescribeProducers, DescribeTopicPartitions cursors and ConsumerGroupDescribe show only the caller's namespace with unqualified names; tests per API.

### [To Do] X-TENANT-0020 Topic id isolation: ids of other namespaces behave as unknown
P0 · M14 · Task · deps: X-TENANT-0018 · labels: security
Done when: Produce v13, Fetch v13+, DeleteTopics and Metadata by id for another namespace's topic id return UNKNOWN_TOPIC_ID exactly as for a random id; tests.

### [To Do] X-TENANT-0021 Name length and character rules for qualified names
P0 · M14 · Task · deps: X-TENANT-0018, TYPE-F10
Done when: Tenants can use names up to Kafka's limits because the namespace is stored separately from the name, as decided in the ADR; `.` and `_` collision detection works within a namespace; tests.

### [To Do] X-TENANT-0022 Internal topics hidden from tenants; group and transaction coordinator keys use qualified ids
P0 · M14 · Task · deps: X-TENANT-0018
Done when: Tenants never see `__consumer_offsets`, `__transaction_state` or Alluvium internal topics; FindCoordinator hashes qualified ids; two tenants using the same group id get independent groups; tests.

### [To Do] X-TENANT-0023 ACL APIs in a namespace: resource names qualified, wildcard and prefix patterns confined to the namespace
P0 · M14 · Task · deps: X-TENANT-0018 · labels: security · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizer.java @4.4.0
Done when: CreateAcls, DescribeAcls and DeleteAcls from a tenant only see and create ACLs inside the namespace; `*` means namespace-wide; empty prefixes cannot escape; tests.

### [To Do] X-TENANT-0024 Config APIs in a namespace: TOPIC, GROUP and CLIENT_METRICS resources mapped, BROKER resources denied
P0 · M14 · Task · deps: X-TENANT-0018
Done when: DescribeConfigs, IncrementalAlterConfigs and AlterConfigs map tenant resource names and deny broker and broker-logger resources to tenant principals; tests.

### [To Do] X-TENANT-0025 Error messages never reveal qualified names or other namespaces
P0 · M14 · Task · deps: X-TENANT-0019 · labels: security
Done when: A response check strips qualification from every `error_message`; fuzzed requests produce no qualified names in responses; tests.

### [To Do] X-TENANT-0026 Native API namespace resolution matches the Kafka path
P0 · M14 · Task · deps: X-TENANT-0015, X-NATIVE-F09
Done when: Native requests resolve the namespace from the principal (or an explicit namespace for cluster admins); tests compare native and Kafka paths.

### [To Do] X-TENANT-0027 Property test: qualify and unqualify are inverse and never cross namespaces
P0 · M14 · Task · deps: X-TENANT-0017, X-TENANT-0021 · labels: test
Done when: For arbitrary names and namespaces, round-trips are identity and no qualified name maps into another namespace.

### [To Do] X-TENANT-0028 Compatibility test: two tenants use the same topic, group and transactional ids with Java, librdkafka, franz-go and kafka-*.sh
P0 · M14 · Task · deps: X-TENANT-0019, X-TENANT-0022, X-TENANT-0023, X-TENANT-0024, TEST-F08 · labels: test
Done when: Both tenants run produce, consume, transactions and admin tools with identical names and see only their own data.

## [To Do] X-TENANT-F04 Tenant resource limits
P0 · M14 · Feature · deps: X-TENANT-F02
Scope: Partitions, storage bytes, topics and connections per tenant.

### [To Do] X-TENANT-0029 Namespace limit keys: `alluvium.tenant.max.topics`, `alluvium.tenant.max.partitions`, `alluvium.tenant.max.storage.bytes`, `alluvium.tenant.max.connections.per.broker`
P0 · M14 · Task · deps: CFG-F11 · labels: config
Done when: Keys registered as namespace config keys with validators and docs; unset means unlimited; tests.

### [To Do] X-TENANT-0030 Controller usage counters per namespace for topics and partitions
P0 · M14 · Task · deps: X-TENANT-0029
Done when: Counters derived from the image are exact after replay and snapshot load; property test against a recount.

### [To Do] X-TENANT-0031 Topic and partition limits enforced atomically in CreateTopics, CreatePartitions and elastic splits
P0 · M14 · Task · deps: X-TENANT-0030 · ref: metadata/src/main/java/org/apache/kafka/controller/ReplicationControlManager.java createTopics @4.4.0
Done when: Requests exceeding a limit fail with POLICY_VIOLATION and a message with the limit; concurrent requests forwarded from different brokers never exceed it; tests.

### [To Do] X-TENANT-0032 Storage usage reporting: brokers report per-namespace bytes of partitions they lead
P0 · M14 · Task · deps: X-TENANT-0029 · labels: decoder
Done when: An Alluvium-internal RPC with fuzz coverage carries per-namespace byte totals at a bounded interval; the controller keeps them in memory only; tests.

### [To Do] X-TENANT-0033 `NamespaceLimitStateRecord`: namespace id, limit kind, state (within, exceeded) and observed value
P0 · M14 · Task · deps: X-TENANT-0032 · labels: decoder
Done when: Schema, codec round-trip, replay into `NamespacesImage` and fuzz coverage; only state changes are written, never periodic usage; tests.

### [To Do] X-TENANT-0034 Storage limit enforcement on produce with soft and hard thresholds
P0 · M14 · Task · deps: X-TENANT-0033
Done when: Crossing the hard limit writes `NamespaceLimitStateRecord(storage_exceeded)`; produce from that namespace fails with the ADR error until usage drops; at 90% an event warns; tests.

### [To Do] X-TENANT-0035 Per-broker connection limit per namespace enforced after authentication
P0 · M14 · Task · deps: X-TENANT-0029, NET-F02 · ref: core/src/main/scala/kafka/network/SocketServer.scala ConnectionQuotas @4.4.0
Done when: Connections beyond the limit are closed after authentication with an event; counts drop on close; tests.

### [To Do] X-TENANT-0036 Usage and limits exposed in DescribeNamespace
P0 · M14 · Task · deps: X-TENANT-0030, X-TENANT-0032
Done when: DescribeNamespace returns current topics, partitions, storage bytes, connections and limits; tests.

### [To Do] X-TENANT-0037 Limit events: `tenant_limit_warning` and `tenant_limit_exceeded` with reasons
P0 · M14 · Task · deps: X-TENANT-0031, X-TENANT-0034 · labels: observability
Done when: Events rate-limited per namespace; tests.

## [To Do] X-TENANT-F05 Tenant quotas
P0 · M14 · Feature · deps: X-TENANT-F02, QUOTA-F01
Scope: Tenant-level throughput and request quotas.

### [To Do] X-TENANT-0038 Quota entity type for namespaces in ClientQuotaRecord and Describe/AlterClientQuotas
P0 · M14 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/ClientQuotaRecord.json @4.4.0
Done when: An Alluvium entity type (name from the ADR) is accepted by AlterClientQuotas and stored in ClientQuotaRecord; Kafka admin clients that pass it work; validation rejects mixing it with ip entities; tests.

### [To Do] X-TENANT-0039 Namespace quota applies in addition to user and client-id quotas
P0 · M14 · Task · deps: X-TENANT-0038 · ref: server/src/main/java/org/apache/kafka/server/quota/ClientQuotaManager.java @4.4.0
Done when: The effective throttle is the maximum of the namespace throttle and the existing Kafka-resolved throttle; property tests over quota combinations.

### [To Do] X-TENANT-0040 Produce and fetch byte-rate quotas per namespace
P0 · M14 · Task · deps: X-TENANT-0039, QUOTA-F02, QUOTA-F03
Done when: Per-broker sensors per namespace compute throttle times like QUOTA-F02 and QUOTA-F03; tests with the simulated clock.

### [To Do] X-TENANT-0041 Request-time quotas per namespace
P0 · M14 · Task · deps: X-TENANT-0039, QUOTA-F04 · ref: server/src/main/java/org/apache/kafka/server/quota/ClientRequestQuotaManager.java @4.4.0
Done when: Handler time per namespace is measured like QUOTA-F04; tests.

### [To Do] X-TENANT-0042 Controller mutation quotas per namespace
P0 · M14 · Task · deps: X-TENANT-0039, QUOTA-F05 · ref: server/src/main/java/org/apache/kafka/server/quota/ControllerMutationQuotaManager.java @4.4.0
Done when: Topic and partition mutations are charged to the namespace like QUOTA-F05; tests.

### [To Do] X-TENANT-0043 Throttle semantics unchanged for clients
P0 · M14 · Task · deps: X-TENANT-0040
Done when: throttle_time_ms and channel muting follow QUOTA-F07 per API version; tests.

### [To Do] X-TENANT-0044 Quota breach events and bounded-cardinality metrics
P0 · M14 · Task · deps: X-TENANT-0040, X-TENANT-0041 · labels: observability
Done when: `tenant_quota_breached` events are rate-limited; `alluvium_tenant_throttle_seconds_total{namespace}` respects the namespace label cap; tests.

## [To Do] X-TENANT-F06 Fair scheduling
P1 · M14 · Feature · deps: X-TENANT-F02, SRV-F05
Scope: Weighted fair request queues per tenant.

### [To Do] X-TENANT-0045 Weighted deficit round robin request queue per namespace
P1 · M14 · Task · deps: none · ref: core/src/main/scala/kafka/network/RequestChannel.scala @4.4.0
Done when: A sans-I/O scheduler with per-namespace bounded sub-queues of `alluvium.tenant.queue.max.requests` and the global `queued.max.requests` bound; unit tests for weights and empty queues.

### [To Do] X-TENANT-0046 Request cost model for scheduling: bytes for produce, max bytes for fetch, fixed cost for metadata and admin
P1 · M14 · Task · deps: X-TENANT-0045
Done when: Costs are computed before enqueue from the decoded header and size; documented; tests.

### [To Do] X-TENANT-0047 Register scheduling keys: `alluvium.tenant.fair.scheduling.enable`, `alluvium.tenant.scheduling.weight`, `alluvium.tenant.queue.max.requests`, `alluvium.tenant.max.delayed.operations`
P1 · M14 · Task · deps: X-TENANT-0045, CFG-F11 · labels: config
Done when: Broker and namespace keys defined in the CFG-F11 registry with validators, defaults and docs; weight changes apply without restart; tests.

### [To Do] X-TENANT-0048 Scheduler integration in the request pipeline behind `alluvium.tenant.fair.scheduling.enable`
P1 · M14 · Task · deps: X-TENANT-0045, X-TENANT-0047, SRV-F05
Done when: When enabled the handler pool pulls from the scheduler instead of the FIFO queue; disabled is identical to today (test).

### [To Do] X-TENANT-0049 Per-namespace muting when a sub-queue is full
P1 · M14 · Task · deps: X-TENANT-0048
Done when: Only connections of the full namespace are muted; other tenants keep flowing; tests.

### [To Do] X-TENANT-0050 Inter-broker, controller and internal requests bypass tenant queues
P1 · M14 · Task · deps: X-TENANT-0048
Done when: Replication, Raft and controller traffic use a priority lane; tests under tenant saturation.

### [To Do] X-TENANT-0051 Per-namespace limit on delayed operations: `alluvium.tenant.max.delayed.operations`
P1 · M14 · Task · deps: X-TENANT-0048, SRV-F07
Done when: Over-limit delayed produce and fetch complete immediately with the ADR behavior; tests.

### [To Do] X-TENANT-0052 Property test: fairness bound for backlogged namespaces
P1 · M14 · Task · deps: X-TENANT-0046, X-TENANT-0047 · labels: test
Done when: Each backlogged namespace receives at least its weight share minus one quantum over any window; regressions persisted.

### [To Do] X-TENANT-0053 Scheduler metrics: per-namespace queue depth and wait time with the label cap, sub-queue full events
P1 · M14 · Task · deps: X-TENANT-0048 · labels: observability
Done when: Metrics and `tenant_queue_full` events registered and asserted; tests.

### [To Do] X-TENANT-0054 Benchmark scheduler overhead versus FIFO
P1 · M14 · Task · deps: X-TENANT-0048, TEST-F11 · labels: perf
Done when: Throughput and p99 latency with fair scheduling on and one tenant stay within the ADR budget; results in the benchmark job.

## [To Do] X-TENANT-F07 I/O isolation
P2 · M14 · Feature · deps: X-TENANT-F06
Scope: Disk and network bandwidth shares.

### [To Do] X-TENANT-0055 Network bandwidth shares: per-namespace token buckets for bytes in and out, weighted under contention
P2 · M14 · Task · deps: none
Done when: Buckets per broker apply only when the listener is saturated; tests with simulated load.

### [To Do] X-TENANT-0056 Disk read and flush shares in the storage I/O executor
P2 · M14 · Task · deps: STOR-F18
Done when: The STOR-F18 executor queues per namespace with weights; cold reads of one tenant cannot starve another; tests.

### [To Do] X-TENANT-0057 Replication traffic attribution or exemption as decided in the ADR
P2 · M14 · Task · deps: X-TENANT-0055 · ref: ADR from X-TENANT-0001 (multi-tenancy)
Done when: Follower fetch bytes are exempt or charged per the ADR; documented; tests.

### [To Do] X-TENANT-0058 Register I/O share keys
P2 · M14 · Task · deps: X-TENANT-0055, CFG-F11 · labels: config
Done when: `alluvium.tenant.io.network.weight` and `alluvium.tenant.io.disk.weight` defined as namespace config keys with validators and docs.

### [To Do] X-TENANT-0059 I/O isolation metrics with the namespace label cap
P2 · M14 · Task · deps: X-TENANT-0055, X-TENANT-0056 · labels: observability
Done when: Per-namespace bytes, waits and throttling exported; tests.

### [To Do] X-TENANT-0060 Adversarial I/O test: one tenant saturating disk reads and network keeps other tenants' p99 within bound
P2 · M14 · Task · deps: X-TENANT-0055, X-TENANT-0056, TEST-F12 · labels: test
Done when: A multi-process test records victim p99 latency within the ADR target.

## [To Do] X-TENANT-F08 Delegated administration
P1 · M14 · Feature · deps: X-TENANT-F02, SEC-F07
Scope: Tenant admins and namespace-scoped ACLs.

### [To Do] X-TENANT-0061 Tenant admin role: namespace-scoped admin principals from `alluvium.tenant.admins`
P1 · M14 · Task · deps: CFG-F11
Done when: Namespace config lists tenant admins; they may administer resources only in their namespace; tests.

### [To Do] X-TENANT-0062 Namespace-scoped ACL evaluation in the authorizer
P1 · M14 · Task · deps: X-TENANT-0061 · labels: security · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizer.java @4.4.0
Done when: The authorizer evaluates qualified resources and never matches ACLs across namespaces; super users remain cluster-wide; property tests over random ACL sets.

### [To Do] X-TENANT-0063 Tenant admins can grant only within their namespace and never CLUSTER operations
P1 · M14 · Task · deps: X-TENANT-0062 · labels: security
Done when: CreateAcls from a tenant admin for CLUSTER or another namespace fails with CLUSTER_AUTHORIZATION_FAILED for cluster resources and the resource's authorization error for other namespaces; tests.

### [To Do] X-TENANT-0064 Tenant topic config policy: allowed keys `alluvium.tenant.alterable.topic.configs` and value caps
P1 · M14 · Task · deps: X-TENANT-0061 · ref: clients/src/main/java/org/apache/kafka/server/policy/CreateTopicPolicy.java @4.4.0
Done when: Tenant CreateTopics and config changes outside the allow-list or above caps fail with POLICY_VIOLATION; cluster admins are unaffected; tests.

### [To Do] X-TENANT-0065 Native admin endpoints for tenant admins
P1 · M14 · Task · deps: X-TENANT-0063, X-NATIVE-F08
Done when: Tenant admins can list, create and alter resources in their namespace over the native API with the same checks; tests.

### [To Do] X-TENANT-0066 Audit events for tenant admin actions
P1 · M14 · Task · deps: X-TENANT-0063, X-SECX-F07 · labels: security
Done when: Every tenant admin mutation is written to the X-SECX-F07 audit stream with principal, namespace and outcome; tests.

### [To Do] X-TENANT-0067 Escalation tests: tenant principals cannot reach other namespaces or cluster resources
P1 · M14 · Task · deps: X-TENANT-0063, X-TENANT-0064 · labels: security, test
Done when: A property-based test generates requests from tenant principals against all APIs and asserts no cross-namespace or cluster-level effect.

## [To Do] X-TENANT-F09 Usage metering
P1 · M14 · Feature · deps: X-TENANT-F02
Scope: Per-tenant usage records and export.

### [To Do] X-TENANT-0068 Usage record model: namespace, window, bytes in and out, storage byte-hours, partition-hours, requests and peak connections
P1 · M14 · Task · deps: none · labels: decoder
Done when: Versioned record format documented in docs/formats/usage-record.md; encoder and bounded decoder with a fuzz target.

### [To Do] X-TENANT-0069 Per-namespace counters on brokers with fixed memory per namespace
P1 · M14 · Task · deps: X-TENANT-0068
Done when: Counters updated in the request pipeline without locks on the hot path; benchmark shows overhead within budget.

### [To Do] X-TENANT-0070 Usage export to the internal topic `__alluvium_usage` with idempotent writes and record keys
P1 · M14 · Task · deps: X-TENANT-0069, CFG-F11
Done when: Each broker writes one record per namespace per `alluvium.tenant.metering.window.ms` window keyed by (broker, namespace, window); consumers can deduplicate; topic created with retention from `alluvium.tenant.metering.retention.ms`; tests.

### [To Do] X-TENANT-0071 Storage byte-hours sampled from partition sizes
P1 · M14 · Task · deps: X-TENANT-0069
Done when: Sampling interval and integration documented; accuracy test against a synthetic size trace.

### [To Do] X-TENANT-0072 Native GetUsage aggregated by namespace and time range
P1 · M14 · Task · deps: X-TENANT-0070, X-NATIVE-F08
Done when: Aggregates the usage topic with authorization (cluster admin or the namespace's tenant admin); tests.

### [To Do] X-TENANT-0073 Metering accuracy test within 1% of a synthetic workload
P1 · M14 · Task · deps: X-TENANT-0070, X-TENANT-0071 · labels: test
Done when: A simulation with known traffic compares exported totals with ground truth.

## [To Do] X-TENANT-F10 Tenant-aware features
P1 · M14 · Feature · deps: X-TENANT-F02
Scope: Schema contexts; metrics labels with bounded cardinality.

### [To Do] X-TENANT-0074 Schema registry contexts per namespace
P1 · M14 · Task · deps: X-SCHEMA-F10
Done when: Tenant principals resolve subjects in their namespace's context; Confluent serializers work unchanged; tests.

### [To Do] X-TENANT-0075 Namespace metric label with a cap: `alluvium.tenant.metrics.max.namespaces`
P1 · M14 · Task · deps: CFG-F11 · labels: observability, config
Done when: Only the top namespaces by traffic get their own label value, the rest aggregate as `other`; cardinality test.

### [To Do] X-TENANT-0076 `namespace` field in request spans and the standard field table
P1 · M14 · Task · deps: OBS-F03 · labels: observability
Done when: The field is added to the debuggability standard and constants; every request span carries it; tests.

### [To Do] X-TENANT-0077 Diagnostics filtered by namespace
P1 · M14 · Task · deps: OBS-F08
Done when: `/debug/namespaces/{name}` shows usage, limits, queue state and flight recorder; tests.

### [To Do] X-TENANT-0078 SDK namespace awareness
P2 · M14 · Task · deps: X-NATIVE-F12
Done when: The SDK exposes the resolved namespace and an explicit namespace option for cluster admins; tests.

## [To Do] X-TENANT-F11 Tenant data deletion
P1 · M14 · Feature · deps: X-TENANT-F02, X-SECX-F02, X-SECX-F10
Scope: Full deletion including crypto-shredding.

### [To Do] X-TENANT-0079 Deletion orchestrator: delete topics, groups, transactional ids, ACLs, quotas, configs and schema subjects of a Deleting namespace
P1 · M14 · Task · deps: none
Done when: Resumable controller-driven deletion with progress records; resumes after controller failover; tests cut the stream at each record.

### [To Do] X-TENANT-0080 Crypto-shredding: destroy the namespace key so remaining encrypted data is unrecoverable
P1 · M14 · Task · deps: X-TENANT-0079 · labels: security
Done when: Namespace data keys from X-SECX-F02 are wrapped by a namespace key; deletion destroys it through the KMS; tests show data from backups cannot be decrypted.

### [To Do] X-TENANT-0081 Remote data deletion: tiered segments and diskless objects of the namespace
P1 · M14 · Task · deps: X-TENANT-0079, TIER-F05
Done when: Deletion waits for remote and object deletions to complete; tests with simulated stores.

### [To Do] X-TENANT-0082 Deletion completion report and `namespace_deleted` event
P1 · M14 · Task · deps: X-TENANT-0079
Done when: DescribeNamespace reports per-resource-type progress; completion emits the event with counts; tests.

### [To Do] X-TENANT-0083 Namespace names reusable after deletion with a new id
P1 · M14 · Task · deps: X-TENANT-0082
Done when: A recreated namespace gets a fresh `NamespaceId` and sees none of the old resources; tests.

### [To Do] X-TENANT-0084 Residual-data verification after deletion
P1 · M14 · Task · deps: X-TENANT-0080, X-TENANT-0081 · labels: test
Done when: A test scans log directories, metadata and the usage topic after deletion and finds no data of the namespace except metering records within retention.

## [To Do] X-TENANT-F12 Tenancy testing and documentation
P0 · M14 · Feature · deps: X-TENANT-F06, TEST-F06
Scope: Isolation under adversarial load; guide.

### [To Do] X-TENANT-0085 Checker: no request of one namespace reads, writes or describes resources of another
P0 · M14 · Task · deps: TEST-F05 · labels: test
Done when: A TEST-F05 checker inspects every response in simulation for foreign resources and fails with the seed.

### [To Do] X-TENANT-0086 Checker: namespace topic, partition and connection limits are never exceeded
P0 · M14 · Task · deps: TEST-F05 · labels: test
Done when: After every step the checker compares namespace usage in the image and on brokers with the limits.

### [To Do] X-TENANT-0087 Checker: tenants within their quotas are never throttled and backlogged tenants get their weight share
P0 · M14 · Task · deps: TEST-F05 · labels: test
Done when: The checker reads throttle decisions and scheduler dispatch counts after every step.

### [To Do] X-TENANT-0088 Simulation: noisy-neighbor tenant saturating produce, fetch, metadata and connections
P0 · M14 · Task · deps: X-TENANT-0085, X-TENANT-0086, X-TENANT-0087 · labels: test
Done when: Victim tenants keep p99 latency within the ADR bound and are never throttled; seeds in the fixed set.

### [To Do] X-TENANT-0089 Fuzzing name mapping with crafted names: prefixes, unicode, `.` and `_` collisions
P0 · M14 · Task · deps: TEST-F07 · labels: security, test
Done when: A fuzz target drives request rewriting; no input escapes the namespace or panics.

### [To Do] X-TENANT-0090 Tenancy is off by default: no namespace records, no rewriting, no scheduler
P0 · M14 · Task · deps: none · labels: test
Done when: With default config the feature level is 0, requests take the existing path with no rewrite, and responses are byte-identical to a build without tenancy (golden test).

### [To Do] X-TENANT-0091 Kafka compatibility and differential suites pass with tenancy enabled but every client in the default namespace
P0 · M14 · Task · deps: X-TENANT-0090, TEST-F09 · labels: test
Done when: CI job runs TEST-F08 and TEST-F09 suites with the feature level finalized and fair scheduling on; zero new diffs.

### [To Do] X-TENANT-0092 Compatibility suite run as a non-default tenant
P0 · M14 · Task · deps: X-TENANT-0091 · labels: test
Done when: The TEST-F08 suite and kafka-*.sh tools pass for a client mapped to a non-default namespace.

### [To Do] X-TENANT-0093 Benchmarks: overhead of name rewriting and fair scheduling
P0 · M14 · Task · deps: TEST-F11 · labels: perf, nfr
Done when: Throughput and p99 with tenancy on versus off recorded in the benchmark job with regression thresholds.

### [To Do] X-TENANT-0094 Concepts guide: namespaces, name mapping, limits, quotas and fairness
P0 · M14 · Task · deps: none · labels: docs
Done when: docs/guides/multi-tenancy.md reviewed against the ADR.

### [To Do] X-TENANT-0095 Operator guide: creating namespaces, mapping rules, limits, quotas, metering export and deletion
P0 · M14 · Task · deps: X-TENANT-0094 · labels: docs
Done when: Includes runbooks for quota breaches, suspended tenants and stuck deletions; links check.

### [To Do] X-TENANT-0096 Tenant admin guide: managing topics, ACLs and configs inside a namespace
P0 · M14 · Task · deps: X-TENANT-0094 · labels: docs
Done when: Examples with kafka-*.sh tools and the native CLI.

### [To Do] X-TENANT-0097 Migration and opt-in guide: enabling tenancy and moving prefix-based setups into namespaces
P0 · M14 · Task · deps: X-TENANT-0094 · labels: docs
Done when: Covers feature level upgrade, mapping existing principals, moving topics by mirroring, and rollback limits.
