# Kafka compatibility scope

This page defines the Kafka 4.4.0 protocol surface Alluvium implements ([ADR-0010](adr/0010-alluvium-is-its-own-platform.md)): which APIs, which versions, and where Alluvium intentionally differs. **Status and scheduling live in the [backlog](backlog/README.md)**, not here. Each API is one backlog feature, and the "Backlog" column links to it.

An API counts as supported only when its backlog feature is `Done`. That means its compatibility and differential tests pass, and its versions are advertised in `ApiVersions` ([testing](standards/testing.md#compatibility-and-differential-tests)). Update this page only if the scope changes: a new Kafka target version, or a new deviation.

## APIs

Generated from the request schemas at Kafka tag `4.4.0` (`clients/src/main/resources/common/message/*Request.json`). "Listeners" shows where Kafka serves the API: `broker` means the client-facing broker listeners, `controller` means the KRaft controller listeners.

| Key | API | Kafka 4.4.0 versions | Listeners | Backlog |
|---|---|---|---|---|
| 0 | `Produce` | 3-13 | broker | [API-F00](backlog/20-api-client-apis.md) |
| 1 | `Fetch` | 4-18 | broker, controller | [API-F01](backlog/20-api-client-apis.md) |
| 2 | `ListOffsets` | 1-11 | broker | [API-F02](backlog/20-api-client-apis.md) |
| 3 | `Metadata` | 0-13 | broker | [API-F03](backlog/20-api-client-apis.md) |
| 4 | LeaderAndIsr | removed in 4.0 | — | not planned (ZooKeeper era) |
| 5 | StopReplica | removed in 4.0 | — | not planned (ZooKeeper era) |
| 6 | UpdateMetadata | removed in 4.0 | — | not planned (ZooKeeper era) |
| 7 | ControlledShutdown | removed in 4.0 | — | not planned (ZooKeeper era) |
| 8 | `OffsetCommit` | 2-10 | broker | [API-F08](backlog/20-api-client-apis.md) |
| 9 | `OffsetFetch` | 1-10 | broker | [API-F09](backlog/20-api-client-apis.md) |
| 10 | `FindCoordinator` | 0-6 | broker | [API-F10](backlog/20-api-client-apis.md) |
| 11 | `JoinGroup` | 0-9 | broker | [API-F11](backlog/20-api-client-apis.md) |
| 12 | `Heartbeat` | 0-4 | broker | [API-F12](backlog/20-api-client-apis.md) |
| 13 | `LeaveGroup` | 0-5 | broker | [API-F13](backlog/20-api-client-apis.md) |
| 14 | `SyncGroup` | 0-5 | broker | [API-F14](backlog/20-api-client-apis.md) |
| 15 | `DescribeGroups` | 0-6 | broker | [API-F15](backlog/20-api-client-apis.md) |
| 16 | `ListGroups` | 0-5 | broker | [API-F16](backlog/20-api-client-apis.md) |
| 17 | `SaslHandshake` | 0-1 | broker, controller | [API-F17](backlog/20-api-client-apis.md) |
| 18 | `ApiVersions` | 0-5 | broker, controller | [API-F18](backlog/20-api-client-apis.md) |
| 19 | `CreateTopics` | 2-7 | broker, controller | [API-F19](backlog/20-api-client-apis.md) |
| 20 | `DeleteTopics` | 1-6 | broker, controller | [API-F20](backlog/20-api-client-apis.md) |
| 21 | `DeleteRecords` | 0-2 | broker | [API-F21](backlog/20-api-client-apis.md) |
| 22 | `InitProducerId` | 0-6 (latest unstable) | broker | [API-F22](backlog/20-api-client-apis.md) |
| 23 | `OffsetForLeaderEpoch` | 2-4 | broker | [API-F23](backlog/20-api-client-apis.md) |
| 24 | `AddPartitionsToTxn` | 0-5 | broker | [API-F24](backlog/20-api-client-apis.md) |
| 25 | `AddOffsetsToTxn` | 0-4 | broker | [API-F25](backlog/20-api-client-apis.md) |
| 26 | `EndTxn` | 0-5 | broker | [API-F26](backlog/20-api-client-apis.md) |
| 27 | `WriteTxnMarkers` | 1-2 | broker | [API-F27](backlog/20-api-client-apis.md) |
| 28 | `TxnOffsetCommit` | 0-6 | broker | [API-F28](backlog/20-api-client-apis.md) |
| 29 | `DescribeAcls` | 1-3 | broker, controller | [API-F29](backlog/20-api-client-apis.md) |
| 30 | `CreateAcls` | 1-3 | broker, controller | [API-F30](backlog/20-api-client-apis.md) |
| 31 | `DeleteAcls` | 1-3 | broker, controller | [API-F31](backlog/20-api-client-apis.md) |
| 32 | `DescribeConfigs` | 1-4 | broker, controller | [API-F32](backlog/20-api-client-apis.md) |
| 33 | `AlterConfigs` | 0-2 | broker, controller | [API-F33](backlog/20-api-client-apis.md) |
| 34 | `AlterReplicaLogDirs` | 1-2 | broker | [API-F34](backlog/20-api-client-apis.md) |
| 35 | `DescribeLogDirs` | 1-5 | broker | [API-F35](backlog/20-api-client-apis.md) |
| 36 | `SaslAuthenticate` | 0-2 | broker, controller | [API-F36](backlog/20-api-client-apis.md) |
| 37 | `CreatePartitions` | 0-3 | broker, controller | [API-F37](backlog/20-api-client-apis.md) |
| 38 | `CreateDelegationToken` | 1-3 | broker, controller | [API-F38](backlog/20-api-client-apis.md) |
| 39 | `RenewDelegationToken` | 1-2 | broker, controller | [API-F39](backlog/20-api-client-apis.md) |
| 40 | `ExpireDelegationToken` | 1-2 | broker, controller | [API-F40](backlog/20-api-client-apis.md) |
| 41 | `DescribeDelegationToken` | 1-3 | broker, controller | [API-F41](backlog/20-api-client-apis.md) |
| 42 | `DeleteGroups` | 0-3 | broker | [API-F42](backlog/20-api-client-apis.md) |
| 43 | `ElectLeaders` | 0-2 | broker, controller | [API-F43](backlog/20-api-client-apis.md) |
| 44 | `IncrementalAlterConfigs` | 0-1 | broker, controller | [API-F44](backlog/20-api-client-apis.md) |
| 45 | `AlterPartitionReassignments` | 0-1 | broker, controller | [API-F45](backlog/20-api-client-apis.md) |
| 46 | `ListPartitionReassignments` | 0 | broker, controller | [API-F46](backlog/20-api-client-apis.md) |
| 47 | `OffsetDelete` | 0 | broker | [API-F47](backlog/20-api-client-apis.md) |
| 48 | `DescribeClientQuotas` | 0-1 | broker | [API-F48](backlog/20-api-client-apis.md) |
| 49 | `AlterClientQuotas` | 0-1 | broker, controller | [API-F49](backlog/20-api-client-apis.md) |
| 50 | `DescribeUserScramCredentials` | 0 | broker, controller | [API-F50](backlog/20-api-client-apis.md) |
| 51 | `AlterUserScramCredentials` | 0 | broker, controller | [API-F51](backlog/20-api-client-apis.md) |
| 52 | `Vote` | 0-2 | controller | [API-F52](backlog/20-api-client-apis.md) |
| 53 | `BeginQuorumEpoch` | 0-1 | controller | [API-F53](backlog/20-api-client-apis.md) |
| 54 | `EndQuorumEpoch` | 0-1 | controller | [API-F54](backlog/20-api-client-apis.md) |
| 55 | `DescribeQuorum` | 0-2 | broker, controller | [API-F55](backlog/20-api-client-apis.md) |
| 56 | `AlterPartition` | 2-3 | controller | [API-F56](backlog/20-api-client-apis.md) |
| 57 | `UpdateFeatures` | 0-2 | broker, controller | [API-F57](backlog/20-api-client-apis.md) |
| 58 | `Envelope` | 0 | controller | [API-F58](backlog/20-api-client-apis.md) |
| 59 | `FetchSnapshot` | 0-1 | controller | [API-F59](backlog/20-api-client-apis.md) |
| 60 | `DescribeCluster` | 0-2 | broker, controller | [API-F60](backlog/20-api-client-apis.md) |
| 61 | `DescribeProducers` | 0 | broker | [API-F61](backlog/20-api-client-apis.md) |
| 62 | `BrokerRegistration` | 0-4 | controller | [API-F62](backlog/20-api-client-apis.md) |
| 63 | `BrokerHeartbeat` | 0-2 | controller | [API-F63](backlog/20-api-client-apis.md) |
| 64 | `UnregisterBroker` | 0 | broker, controller | [API-F64](backlog/20-api-client-apis.md) |
| 65 | `DescribeTransactions` | 0 | broker | [API-F65](backlog/20-api-client-apis.md) |
| 66 | `ListTransactions` | 0-2 | broker | [API-F66](backlog/20-api-client-apis.md) |
| 67 | `AllocateProducerIds` | 0 | controller | [API-F67](backlog/20-api-client-apis.md) |
| 68 | `ConsumerGroupHeartbeat` | 0-1 | broker | [API-F68](backlog/20-api-client-apis.md) |
| 69 | `ConsumerGroupDescribe` | 0-1 | broker | [API-F69](backlog/20-api-client-apis.md) |
| 70 | `ControllerRegistration` | 0 | controller | [API-F70](backlog/20-api-client-apis.md) |
| 71 | `GetTelemetrySubscriptions` | 0 | broker | [API-F71](backlog/20-api-client-apis.md) |
| 72 | `PushTelemetry` | 0 | broker | [API-F72](backlog/20-api-client-apis.md) |
| 73 | `AssignReplicasToDirs` | 0 | controller | [API-F73](backlog/20-api-client-apis.md) |
| 74 | `ListConfigResources` | 0-1 | broker | [API-F74](backlog/20-api-client-apis.md) |
| 75 | `DescribeTopicPartitions` | 0 | broker | [API-F75](backlog/20-api-client-apis.md) |
| 76 | `ShareGroupHeartbeat` | 1 | broker | [API-F76](backlog/20-api-client-apis.md) |
| 77 | `ShareGroupDescribe` | 1 | broker | [API-F77](backlog/20-api-client-apis.md) |
| 78 | `ShareFetch` | 1-2 | broker | [API-F78](backlog/20-api-client-apis.md) |
| 79 | `ShareAcknowledge` | 1-2 | broker | [API-F79](backlog/20-api-client-apis.md) |
| 80 | `AddRaftVoter` | 0-1 | controller, broker | [API-F80](backlog/20-api-client-apis.md) |
| 81 | `RemoveRaftVoter` | 0 | controller, broker | [API-F81](backlog/20-api-client-apis.md) |
| 82 | `UpdateRaftVoter` | 0 | controller | [API-F82](backlog/20-api-client-apis.md) |
| 83 | `InitializeShareGroupState` | 0 | broker | [API-F83](backlog/20-api-client-apis.md) |
| 84 | `ReadShareGroupState` | 0 | broker | [API-F84](backlog/20-api-client-apis.md) |
| 85 | `WriteShareGroupState` | 0-1 | broker | [API-F85](backlog/20-api-client-apis.md) |
| 86 | `DeleteShareGroupState` | 0 | broker | [API-F86](backlog/20-api-client-apis.md) |
| 87 | `ReadShareGroupStateSummary` | 0-1 | broker | [API-F87](backlog/20-api-client-apis.md) |
| 88 | `StreamsGroupHeartbeat` | 0-1 | broker | [API-F88](backlog/20-api-client-apis.md) |
| 89 | `StreamsGroupDescribe` | 0-1 | broker | [API-F89](backlog/20-api-client-apis.md) |
| 90 | `DescribeShareGroupOffsets` | 0-1 | broker | [API-F90](backlog/20-api-client-apis.md) |
| 91 | `AlterShareGroupOffsets` | 0 | broker | [API-F91](backlog/20-api-client-apis.md) |
| 92 | `DeleteShareGroupOffsets` | 0 | broker | [API-F92](backlog/20-api-client-apis.md) |
| 93 | `StreamsGroupTopologyDescriptionUpdate` | 0 | broker | [API-F93](backlog/20-api-client-apis.md) |
| 94 | `UnregisterController` | 0 | broker, controller | [API-F94](backlog/20-api-client-apis.md) |

## Kafka behavior without its own API

These are tracked as backlog features:

| Behavior | Backlog |
|---|---|
| Record batch v2, CRC32C, compression (gzip, snappy, lz4, zstd) | [REC](backlog/04-rec-records.md) |
| Segments, offset and time indexes, log recovery, retention, compaction | [STOR](backlog/07-stor-storage.md) |
| TLS, SASL, ACLs, delegation tokens | [NET](backlog/09-net-network.md), [SEC](backlog/17-sec-security.md) |
| ISR, high watermark, leader-epoch truncation, ELR (KIP-966), fetch from follower (KIP-392) | [REPL](backlog/12-repl-replication.md), [CTRL](backlog/11-ctrl-controller.md) |
| Quotas and throttling | [QUOTA](backlog/19-quota-quotas.md) |
| Idempotent producer, transactions, `read_committed`, transactions v2 (KIP-890) | [TXN](backlog/23-txn-transactions.md) |
| Static and dynamic configuration with Kafka key names | [CFG](backlog/16-cfg-configuration.md) |
| JBOD (multiple log directories) | [STOR](backlog/07-stor-storage.md) |
| Tiered storage (KIP-405) | [TIER](backlog/28-tier-tiered-storage.md) |
| Kafka command-line tools | [CLI](backlog/15-cli-cli-and-tools.md) |

## Intentional deviations

Each deviation has an ADR. None changes the protocol behavior that clients observe.

| Deviation | Kafka behavior | Alluvium behavior | ADR |
|---|---|---|---|
| Plaintext on non-loopback listeners | Allowed | Refused at startup unless `alluvium.allow.plaintext.external=true` | [0006](adr/0006-secure-defaults.md) |
| Connection limits | Unlimited by default | Finite defaults; configurable to Kafka's values | [0006](adr/0006-secure-defaults.md) |
| Metrics | JMX | Prometheus, with a documented name mapping | [0005](adr/0005-observability-model.md), [0010](adr/0010-alluvium-is-its-own-platform.md) |
| On-disk layout (beyond record batches) | Kafka's segment and index formats | Alluvium's documented formats; Kafka log directories can't be mounted | [0007](adr/0007-record-batches-stored-in-wire-format.md) |
| Features beyond Kafka | — | Available only when enabled; off by default | [0010](adr/0010-alluvium-is-its-own-platform.md) |
