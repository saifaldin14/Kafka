# Kafka parity

This page tracks what Alluvium implements relative to **Apache Kafka 4.4.0** ([ADR-0001](adr/0001-kafka-4-4-compatibility-is-the-parity-contract.md)), and in what order. Update it in the same PR as the change.

**Status values**

- **Not started**
- **In progress (#issue)**
- **Done (versions)**: compatibility and differential tests pass, and the versions are advertised in `ApiVersions` ([testing](standards/testing.md#compatibility-and-differential-tests))
- **Not planned**

## Milestones

Each milestone ends with `main` green and its features proven by compatibility and differential tests.

| Milestone | Scope | Exit criteria |
|---|---|---|
| **M0: Skeleton and tooling** | Cargo workspace, `rust-toolchain.toml`, `[workspace.lints]`, `clippy.toml`, `deny.toml`, `xtask` (`ci`, `check-layers`, codegen scaffold), CI workflow, `alluvium-types` (newtypes, `Clock`, `Secret<T>`, seeded collections) | `cargo xtask ci` passes in CI; the layer check rejects a deliberately wrong dependency |
| **M1: Codecs** | Vendored schemas and code generator, protocol primitives, all request/response codecs, record batch v2, CRC32C, gzip/snappy/lz4/zstd, fuzz targets for every decoder | Round-trip property tests and fuzzing clean; codecs decode traffic captured from real clients |
| **M2: Single-node broker** | Network (framing, limits, TLS, muting), storage (segments, indexes, recovery, time/size retention), single-voter KRaft metadata, server drivers and task supervisor, config, observability (tracing, metrics, diagnostics endpoint, flight recorders), dump tools, compatibility and differential test infrastructure | Java client and librdkafka produce and consume (assigned partitions) against one node; `kafka-topics.sh` works |
| **M3: Multi-node cluster** | Multi-voter KRaft, broker registration and heartbeats, replication (ISR, high watermark, ELR, leader-epoch truncation), leader election, reassignment, simulation harness and checkers | Simulation suites pass across thousands of seeds; no acknowledged `acks=all` write lost under the fault plans |
| **M4: Security and quotas** | SASL (`PLAIN`, `SCRAM`, `OAUTHBEARER`), mTLS principal mapping, ACL authorizer, delegation tokens, client quotas and throttling | `kafka-acls.sh` and authenticated clients work; authorization differential tests pass |
| **M5: Consumer groups** | Log compaction, `__consumer_offsets`, classic and consumer (KIP-848) group protocols, offset APIs | Consumer groups rebalance correctly with Java and librdkafka; `kafka-consumer-groups.sh` works |
| **M6: Idempotence and transactions** | Producer state, `InitProducerId`, transaction coordinator, markers, last stable offset / `read_committed`, transactions v2 (KIP-890) | Exactly-once Kafka Streams application runs correctly; transaction differential tests pass |
| **M7: Admin completeness** | Describe/alter configs and dynamic config, `DeleteRecords`, log directories (JBOD), remaining admin APIs | All `kafka-*.sh` admin tools work |
| **M8: Newer group types** | Share groups (KIP-932), streams groups (KIP-1071), client telemetry (KIP-714) | Compatibility tests with clients that support them |
| **M9: Tiered storage** | KIP-405 remote log storage | Tiered topics behave like Kafka's under differential tests |

After M9, improvements beyond Kafka are proposed through ADRs ([adding a feature](guides/adding-a-kafka-feature.md#improving-on-kafka)).

## APIs

Generated from the request schemas at Kafka tag `4.4.0` (`clients/src/main/resources/common/message/*Request.json`). "Listeners" shows where Kafka serves the API: `broker` means the client-facing broker listeners, `controller` means the KRaft controller listeners.

| Key | API | Kafka 4.4.0 versions | Listeners | Area | Milestone | Status |
|---|---|---|---|---|---|---|
| 0 | `Produce` | 3-13 | broker | Core | M2 | Not started |
| 1 | `Fetch` | 4-18 | broker, controller | Core | M2 | Not started |
| 2 | `ListOffsets` | 1-11 | broker | Core | M2 | Not started |
| 3 | `Metadata` | 0-13 | broker | Core | M2 | Not started |
| 4 | LeaderAndIsr | removed in 4.0 | — | ZooKeeper era | — | Not planned |
| 5 | StopReplica | removed in 4.0 | — | ZooKeeper era | — | Not planned |
| 6 | UpdateMetadata | removed in 4.0 | — | ZooKeeper era | — | Not planned |
| 7 | ControlledShutdown | removed in 4.0 | — | ZooKeeper era | — | Not planned |
| 8 | `OffsetCommit` | 2-10 | broker | Consumer groups | M5 | Not started |
| 9 | `OffsetFetch` | 1-10 | broker | Consumer groups | M5 | Not started |
| 10 | `FindCoordinator` | 0-6 | broker | Consumer groups | M5 | Not started |
| 11 | `JoinGroup` | 0-9 | broker | Consumer groups | M5 | Not started |
| 12 | `Heartbeat` | 0-4 | broker | Consumer groups | M5 | Not started |
| 13 | `LeaveGroup` | 0-5 | broker | Consumer groups | M5 | Not started |
| 14 | `SyncGroup` | 0-5 | broker | Consumer groups | M5 | Not started |
| 15 | `DescribeGroups` | 0-6 | broker | Consumer groups | M5 | Not started |
| 16 | `ListGroups` | 0-5 | broker | Consumer groups | M5 | Not started |
| 17 | `SaslHandshake` | 0-1 | broker, controller | Security / quotas | M4 | Not started |
| 18 | `ApiVersions` | 0-5 | broker, controller | Core | M2 | Not started |
| 19 | `CreateTopics` | 2-7 | broker, controller | Core | M2 | Not started |
| 20 | `DeleteTopics` | 1-6 | broker, controller | Core | M2 | Not started |
| 21 | `DeleteRecords` | 0-2 | broker | Admin / configs / log dirs | M7 | Not started |
| 22 | `InitProducerId` | 0-6 (latest unstable) | broker | Idempotence / transactions | M6 | Not started |
| 23 | `OffsetForLeaderEpoch` | 2-4 | broker | KRaft / replication | M3 | Not started |
| 24 | `AddPartitionsToTxn` | 0-5 | broker | Idempotence / transactions | M6 | Not started |
| 25 | `AddOffsetsToTxn` | 0-4 | broker | Idempotence / transactions | M6 | Not started |
| 26 | `EndTxn` | 0-5 | broker | Idempotence / transactions | M6 | Not started |
| 27 | `WriteTxnMarkers` | 1-2 | broker | Idempotence / transactions | M6 | Not started |
| 28 | `TxnOffsetCommit` | 0-6 | broker | Idempotence / transactions | M6 | Not started |
| 29 | `DescribeAcls` | 1-3 | broker, controller | Security / quotas | M4 | Not started |
| 30 | `CreateAcls` | 1-3 | broker, controller | Security / quotas | M4 | Not started |
| 31 | `DeleteAcls` | 1-3 | broker, controller | Security / quotas | M4 | Not started |
| 32 | `DescribeConfigs` | 1-4 | broker, controller | Admin / configs / log dirs | M7 | Not started |
| 33 | `AlterConfigs` | 0-2 | broker, controller | Admin / configs / log dirs | M7 | Not started |
| 34 | `AlterReplicaLogDirs` | 1-2 | broker | Admin / configs / log dirs | M7 | Not started |
| 35 | `DescribeLogDirs` | 1-5 | broker | Admin / configs / log dirs | M7 | Not started |
| 36 | `SaslAuthenticate` | 0-2 | broker, controller | Security / quotas | M4 | Not started |
| 37 | `CreatePartitions` | 0-3 | broker, controller | Core | M2 | Not started |
| 38 | `CreateDelegationToken` | 1-3 | broker, controller | Security / quotas | M4 | Not started |
| 39 | `RenewDelegationToken` | 1-2 | broker, controller | Security / quotas | M4 | Not started |
| 40 | `ExpireDelegationToken` | 1-2 | broker, controller | Security / quotas | M4 | Not started |
| 41 | `DescribeDelegationToken` | 1-3 | broker, controller | Security / quotas | M4 | Not started |
| 42 | `DeleteGroups` | 0-3 | broker | Consumer groups | M5 | Not started |
| 43 | `ElectLeaders` | 0-2 | broker, controller | KRaft / replication | M3 | Not started |
| 44 | `IncrementalAlterConfigs` | 0-1 | broker, controller | Admin / configs / log dirs | M7 | Not started |
| 45 | `AlterPartitionReassignments` | 0-1 | broker, controller | KRaft / replication | M3 | Not started |
| 46 | `ListPartitionReassignments` | 0 | broker, controller | KRaft / replication | M3 | Not started |
| 47 | `OffsetDelete` | 0 | broker | Consumer groups | M5 | Not started |
| 48 | `DescribeClientQuotas` | 0-1 | broker | Security / quotas | M4 | Not started |
| 49 | `AlterClientQuotas` | 0-1 | broker, controller | Security / quotas | M4 | Not started |
| 50 | `DescribeUserScramCredentials` | 0 | broker, controller | Security / quotas | M4 | Not started |
| 51 | `AlterUserScramCredentials` | 0 | broker, controller | Security / quotas | M4 | Not started |
| 52 | `Vote` | 0-2 | controller | KRaft / replication | M3 | Not started |
| 53 | `BeginQuorumEpoch` | 0-1 | controller | KRaft / replication | M3 | Not started |
| 54 | `EndQuorumEpoch` | 0-1 | controller | KRaft / replication | M3 | Not started |
| 55 | `DescribeQuorum` | 0-2 | broker, controller | KRaft / replication | M3 | Not started |
| 56 | `AlterPartition` | 2-3 | controller | KRaft / replication | M3 | Not started |
| 57 | `UpdateFeatures` | 0-2 | broker, controller | KRaft / replication | M3 | Not started |
| 58 | `Envelope` | 0 | controller | KRaft / replication | M3 | Not started |
| 59 | `FetchSnapshot` | 0-1 | controller | KRaft / replication | M3 | Not started |
| 60 | `DescribeCluster` | 0-2 | broker, controller | Core | M2 | Not started |
| 61 | `DescribeProducers` | 0 | broker | Idempotence / transactions | M6 | Not started |
| 62 | `BrokerRegistration` | 0-4 | controller | KRaft / replication | M3 | Not started |
| 63 | `BrokerHeartbeat` | 0-2 | controller | KRaft / replication | M3 | Not started |
| 64 | `UnregisterBroker` | 0 | broker, controller | KRaft / replication | M3 | Not started |
| 65 | `DescribeTransactions` | 0 | broker | Idempotence / transactions | M6 | Not started |
| 66 | `ListTransactions` | 0-2 | broker | Idempotence / transactions | M6 | Not started |
| 67 | `AllocateProducerIds` | 0 | controller | Idempotence / transactions | M6 | Not started |
| 68 | `ConsumerGroupHeartbeat` | 0-1 | broker | Consumer groups | M5 | Not started |
| 69 | `ConsumerGroupDescribe` | 0-1 | broker | Consumer groups | M5 | Not started |
| 70 | `ControllerRegistration` | 0 | controller | KRaft / replication | M3 | Not started |
| 71 | `GetTelemetrySubscriptions` | 0 | broker | Client telemetry (KIP-714) | M8 | Not started |
| 72 | `PushTelemetry` | 0 | broker | Client telemetry (KIP-714) | M8 | Not started |
| 73 | `AssignReplicasToDirs` | 0 | controller | KRaft / replication | M3 | Not started |
| 74 | `ListConfigResources` | 0-1 | broker | Admin / configs / log dirs | M7 | Not started |
| 75 | `DescribeTopicPartitions` | 0 | broker | Core | M2 | Not started |
| 76 | `ShareGroupHeartbeat` | 1 | broker | Share groups (KIP-932) | M8 | Not started |
| 77 | `ShareGroupDescribe` | 1 | broker | Share groups (KIP-932) | M8 | Not started |
| 78 | `ShareFetch` | 1-2 | broker | Share groups (KIP-932) | M8 | Not started |
| 79 | `ShareAcknowledge` | 1-2 | broker | Share groups (KIP-932) | M8 | Not started |
| 80 | `AddRaftVoter` | 0-1 | broker, controller | KRaft / replication | M3 | Not started |
| 81 | `RemoveRaftVoter` | 0 | broker, controller | KRaft / replication | M3 | Not started |
| 82 | `UpdateRaftVoter` | 0 | controller | KRaft / replication | M3 | Not started |
| 83 | `InitializeShareGroupState` | 0 | broker | Share groups (KIP-932) | M8 | Not started |
| 84 | `ReadShareGroupState` | 0 | broker | Share groups (KIP-932) | M8 | Not started |
| 85 | `WriteShareGroupState` | 0-1 | broker | Share groups (KIP-932) | M8 | Not started |
| 86 | `DeleteShareGroupState` | 0 | broker | Share groups (KIP-932) | M8 | Not started |
| 87 | `ReadShareGroupStateSummary` | 0-1 | broker | Share groups (KIP-932) | M8 | Not started |
| 88 | `StreamsGroupHeartbeat` | 0-1 | broker | Streams groups (KIP-1071) | M8 | Not started |
| 89 | `StreamsGroupDescribe` | 0-1 | broker | Streams groups (KIP-1071) | M8 | Not started |
| 90 | `DescribeShareGroupOffsets` | 0-1 | broker | Share groups (KIP-932) | M8 | Not started |
| 91 | `AlterShareGroupOffsets` | 0 | broker | Share groups (KIP-932) | M8 | Not started |
| 92 | `DeleteShareGroupOffsets` | 0 | broker | Share groups (KIP-932) | M8 | Not started |
| 93 | `StreamsGroupTopologyDescriptionUpdate` | 0 | broker | Streams groups (KIP-1071) | M8 | Not started |
| 94 | `UnregisterController` | 0 | broker, controller | KRaft / replication | M3 | Not started |

## Features without their own API

| Feature | Area | Milestone | Status |
|---|---|---|---|
| Record batch v2, CRC32C, compression (gzip, snappy, lz4, zstd) | Records | M1 | Not started |
| Segments, offset and time indexes, log recovery | Storage | M2 | Not started |
| Time- and size-based retention | Storage | M2 | Not started |
| TLS listeners, mTLS | Security | M2 | Not started |
| Static configuration with Kafka key names | Config | M2 | Not started |
| ISR, high watermark, leader-epoch truncation | Replication | M3 | Not started |
| Eligible leader replicas (KIP-966), unclean leader election | Replication | M3 | Not started |
| Rack awareness, fetch from follower (KIP-392) | Replication | M3 | Not started |
| Request quotas and throttling | Quotas | M4 | Not started |
| Log compaction, tombstones | Storage | M5 | Not started |
| Idempotent producer | Producer | M6 | Not started |
| Transactions, `read_committed`, transactions v2 (KIP-890) | Producer | M6 | Not started |
| Dynamic broker and topic configuration | Config | M7 | Not started |
| JBOD (multiple log directories) | Storage | M7 | Not started |
| Tiered storage (KIP-405) | Storage | M9 | Not started |

## Intentional deviations

Each deviation has an ADR. None changes the protocol behavior that clients observe.

| Deviation | Kafka behavior | Alluvium behavior | ADR |
|---|---|---|---|
| Plaintext on non-loopback listeners | Allowed | Refused at startup unless `alluvium.allow.plaintext.external=true` | [0006](adr/0006-secure-defaults.md) |
| Connection limits | Unlimited by default | Finite defaults; configurable to Kafka's values | [0006](adr/0006-secure-defaults.md) |
| Metrics | JMX | Prometheus, with a documented name mapping | [0001](adr/0001-kafka-4-4-compatibility-is-the-parity-contract.md), [0005](adr/0005-observability-model.md) |
| On-disk layout (beyond record batches) | Kafka's segment and index formats | Alluvium's documented formats; Kafka log directories can't be mounted | [0007](adr/0007-record-batches-stored-in-wire-format.md) |
