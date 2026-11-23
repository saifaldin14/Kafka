# Kafka compatibility scope

This page defines the Kafka 4.4.0 protocol surface Alluvium implements ([ADR-0010](adr/0010-alluvium-is-its-own-platform.md)): which APIs, which versions, and where Alluvium intentionally differs. **Status and scheduling live in the [backlog](backlog/README.md)**, not here. Each API is one backlog feature, and the "Backlog" column links to it.

An API counts as supported only when its backlog feature is `Done`. That means its compatibility and differential tests pass, and its versions are advertised in `ApiVersions` ([testing](standards/testing.md#compatibility-and-differential-tests)). Update this page only if the scope changes: a new Kafka target version, or a new deviation.

## APIs

Every API key of Kafka 4.4.0 and the backlog feature that implements it. Each API's versions, flexible versions and listeners are in [protocol/apis.md](protocol/apis.md), which `cargo xtask codegen` generates from the vendored schemas, so they are never copied by hand ([updating the Kafka schemas](guides/updating-kafka-schemas.md) covers a version change).

| Key | API | Backlog |
|---|---|---|
| 0 | `Produce` | [API-F00](backlog/20-api-client-apis.md) |
| 1 | `Fetch` | [API-F01](backlog/20-api-client-apis.md) |
| 2 | `ListOffsets` | [API-F02](backlog/20-api-client-apis.md) |
| 3 | `Metadata` | [API-F03](backlog/20-api-client-apis.md) |
| 4 | LeaderAndIsr | not planned (ZooKeeper era) |
| 5 | StopReplica | not planned (ZooKeeper era) |
| 6 | UpdateMetadata | not planned (ZooKeeper era) |
| 7 | ControlledShutdown | not planned (ZooKeeper era) |
| 8 | `OffsetCommit` | [API-F08](backlog/20-api-client-apis.md) |
| 9 | `OffsetFetch` | [API-F09](backlog/20-api-client-apis.md) |
| 10 | `FindCoordinator` | [API-F10](backlog/20-api-client-apis.md) |
| 11 | `JoinGroup` | [API-F11](backlog/20-api-client-apis.md) |
| 12 | `Heartbeat` | [API-F12](backlog/20-api-client-apis.md) |
| 13 | `LeaveGroup` | [API-F13](backlog/20-api-client-apis.md) |
| 14 | `SyncGroup` | [API-F14](backlog/20-api-client-apis.md) |
| 15 | `DescribeGroups` | [API-F15](backlog/20-api-client-apis.md) |
| 16 | `ListGroups` | [API-F16](backlog/20-api-client-apis.md) |
| 17 | `SaslHandshake` | [API-F17](backlog/20-api-client-apis.md) |
| 18 | `ApiVersions` | [API-F18](backlog/20-api-client-apis.md) |
| 19 | `CreateTopics` | [API-F19](backlog/20-api-client-apis.md) |
| 20 | `DeleteTopics` | [API-F20](backlog/20-api-client-apis.md) |
| 21 | `DeleteRecords` | [API-F21](backlog/20-api-client-apis.md) |
| 22 | `InitProducerId` | [API-F22](backlog/20-api-client-apis.md) |
| 23 | `OffsetForLeaderEpoch` | [API-F23](backlog/20-api-client-apis.md) |
| 24 | `AddPartitionsToTxn` | [API-F24](backlog/20-api-client-apis.md) |
| 25 | `AddOffsetsToTxn` | [API-F25](backlog/20-api-client-apis.md) |
| 26 | `EndTxn` | [API-F26](backlog/20-api-client-apis.md) |
| 27 | `WriteTxnMarkers` | [API-F27](backlog/20-api-client-apis.md) |
| 28 | `TxnOffsetCommit` | [API-F28](backlog/20-api-client-apis.md) |
| 29 | `DescribeAcls` | [API-F29](backlog/20-api-client-apis.md) |
| 30 | `CreateAcls` | [API-F30](backlog/20-api-client-apis.md) |
| 31 | `DeleteAcls` | [API-F31](backlog/20-api-client-apis.md) |
| 32 | `DescribeConfigs` | [API-F32](backlog/20-api-client-apis.md) |
| 33 | `AlterConfigs` | [API-F33](backlog/20-api-client-apis.md) |
| 34 | `AlterReplicaLogDirs` | [API-F34](backlog/20-api-client-apis.md) |
| 35 | `DescribeLogDirs` | [API-F35](backlog/20-api-client-apis.md) |
| 36 | `SaslAuthenticate` | [API-F36](backlog/20-api-client-apis.md) |
| 37 | `CreatePartitions` | [API-F37](backlog/20-api-client-apis.md) |
| 38 | `CreateDelegationToken` | [API-F38](backlog/20-api-client-apis.md) |
| 39 | `RenewDelegationToken` | [API-F39](backlog/20-api-client-apis.md) |
| 40 | `ExpireDelegationToken` | [API-F40](backlog/20-api-client-apis.md) |
| 41 | `DescribeDelegationToken` | [API-F41](backlog/20-api-client-apis.md) |
| 42 | `DeleteGroups` | [API-F42](backlog/20-api-client-apis.md) |
| 43 | `ElectLeaders` | [API-F43](backlog/20-api-client-apis.md) |
| 44 | `IncrementalAlterConfigs` | [API-F44](backlog/20-api-client-apis.md) |
| 45 | `AlterPartitionReassignments` | [API-F45](backlog/20-api-client-apis.md) |
| 46 | `ListPartitionReassignments` | [API-F46](backlog/20-api-client-apis.md) |
| 47 | `OffsetDelete` | [API-F47](backlog/20-api-client-apis.md) |
| 48 | `DescribeClientQuotas` | [API-F48](backlog/20-api-client-apis.md) |
| 49 | `AlterClientQuotas` | [API-F49](backlog/20-api-client-apis.md) |
| 50 | `DescribeUserScramCredentials` | [API-F50](backlog/20-api-client-apis.md) |
| 51 | `AlterUserScramCredentials` | [API-F51](backlog/20-api-client-apis.md) |
| 52 | `Vote` | [API-F52](backlog/20-api-client-apis.md) |
| 53 | `BeginQuorumEpoch` | [API-F53](backlog/20-api-client-apis.md) |
| 54 | `EndQuorumEpoch` | [API-F54](backlog/20-api-client-apis.md) |
| 55 | `DescribeQuorum` | [API-F55](backlog/20-api-client-apis.md) |
| 56 | `AlterPartition` | [API-F56](backlog/20-api-client-apis.md) |
| 57 | `UpdateFeatures` | [API-F57](backlog/20-api-client-apis.md) |
| 58 | `Envelope` | [API-F58](backlog/20-api-client-apis.md) |
| 59 | `FetchSnapshot` | [API-F59](backlog/20-api-client-apis.md) |
| 60 | `DescribeCluster` | [API-F60](backlog/20-api-client-apis.md) |
| 61 | `DescribeProducers` | [API-F61](backlog/20-api-client-apis.md) |
| 62 | `BrokerRegistration` | [API-F62](backlog/20-api-client-apis.md) |
| 63 | `BrokerHeartbeat` | [API-F63](backlog/20-api-client-apis.md) |
| 64 | `UnregisterBroker` | [API-F64](backlog/20-api-client-apis.md) |
| 65 | `DescribeTransactions` | [API-F65](backlog/20-api-client-apis.md) |
| 66 | `ListTransactions` | [API-F66](backlog/20-api-client-apis.md) |
| 67 | `AllocateProducerIds` | [API-F67](backlog/20-api-client-apis.md) |
| 68 | `ConsumerGroupHeartbeat` | [API-F68](backlog/20-api-client-apis.md) |
| 69 | `ConsumerGroupDescribe` | [API-F69](backlog/20-api-client-apis.md) |
| 70 | `ControllerRegistration` | [API-F70](backlog/20-api-client-apis.md) |
| 71 | `GetTelemetrySubscriptions` | [API-F71](backlog/20-api-client-apis.md) |
| 72 | `PushTelemetry` | [API-F72](backlog/20-api-client-apis.md) |
| 73 | `AssignReplicasToDirs` | [API-F73](backlog/20-api-client-apis.md) |
| 74 | `ListConfigResources` | [API-F74](backlog/20-api-client-apis.md) |
| 75 | `DescribeTopicPartitions` | [API-F75](backlog/20-api-client-apis.md) |
| 76 | `ShareGroupHeartbeat` | [API-F76](backlog/20-api-client-apis.md) |
| 77 | `ShareGroupDescribe` | [API-F77](backlog/20-api-client-apis.md) |
| 78 | `ShareFetch` | [API-F78](backlog/20-api-client-apis.md) |
| 79 | `ShareAcknowledge` | [API-F79](backlog/20-api-client-apis.md) |
| 80 | `AddRaftVoter` | [API-F80](backlog/20-api-client-apis.md) |
| 81 | `RemoveRaftVoter` | [API-F81](backlog/20-api-client-apis.md) |
| 82 | `UpdateRaftVoter` | [API-F82](backlog/20-api-client-apis.md) |
| 83 | `InitializeShareGroupState` | [API-F83](backlog/20-api-client-apis.md) |
| 84 | `ReadShareGroupState` | [API-F84](backlog/20-api-client-apis.md) |
| 85 | `WriteShareGroupState` | [API-F85](backlog/20-api-client-apis.md) |
| 86 | `DeleteShareGroupState` | [API-F86](backlog/20-api-client-apis.md) |
| 87 | `ReadShareGroupStateSummary` | [API-F87](backlog/20-api-client-apis.md) |
| 88 | `StreamsGroupHeartbeat` | [API-F88](backlog/20-api-client-apis.md) |
| 89 | `StreamsGroupDescribe` | [API-F89](backlog/20-api-client-apis.md) |
| 90 | `DescribeShareGroupOffsets` | [API-F90](backlog/20-api-client-apis.md) |
| 91 | `AlterShareGroupOffsets` | [API-F91](backlog/20-api-client-apis.md) |
| 92 | `DeleteShareGroupOffsets` | [API-F92](backlog/20-api-client-apis.md) |
| 93 | `StreamsGroupTopologyDescriptionUpdate` | [API-F93](backlog/20-api-client-apis.md) |
| 94 | `UnregisterController` | [API-F94](backlog/20-api-client-apis.md) |

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
| A known tagged field whose value doesn't fill its declared size (malformed input only) | Read past or short of the declared size, failing later or not at all | Fails at that field with `TaggedFieldSizeMismatch` or `Truncated` | [0020](adr/0020-known-tagged-fields-read-within-their-declared-size.md) |
