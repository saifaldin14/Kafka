# META: Metadata records and image

> Crates: `alluvium-metadata` (layer 1) · Kafka counterpart: `metadata/src/main/java/org/apache/kafka/image`, `metadata/src/main/resources/common/metadata` · Milestones: M2-M6

Generated metadata record types, replay into an immutable image, deltas, publishing, snapshots, feature levels, and the metadata extensions for namespaces and key ranges.

## [To Do] META-F01 Metadata record codegen
P0 · M2 · Feature · deps: PROTO-F05
Scope: Generate all metadata record types from the vendored schemas.

### [To Do] META-0001 Vendor the 27 Kafka 4.4.0 metadata record schemas under `schemas/kafka-4.4.0/metadata/`
P0 · M2 · Task · deps: none · labels: decoder · ref: metadata/src/main/resources/common/metadata/ @4.4.0
Done when: the 27 JSON files are byte-identical to metadata/src/main/resources/common/metadata at tag 4.4.0 and listed in the PROTO-F01 checksum manifest; the documented schema update procedure covers this directory; CI fails if any vendored file is edited.

### [To Do] META-0002 Codegen: accept `type: metadata` schemas and emit the `MetadataRecordType` registry
P0 · M2 · Task · deps: META-0001 · ref: generator/src/main/java/org/apache/kafka/message/MetadataRecordTypeGenerator.java @4.4.0
Done when: the generator parses metadata specs; generated `MetadataRecordType` lists all 27 ids (0-29 except 6, 13 and 16) with names and version ranges (RegisterBrokerRecord 0-4, BrokerRegistrationChangeRecord 0-3, PartitionRecord and PartitionChangeRecord 0-2, all others 0); `from_id` returns `UnknownRecordType { id }` for other ids; `cargo xtask codegen --check` reports no drift.

### [To Do] META-0003 Codegen: metadata record structs with per-version encode/decode in `alluvium-metadata`
P0 · M2 · Task · deps: META-0002 · ref: metadata/src/main/resources/common/metadata/ @4.4.0
Done when: one generated struct per record with typed fields (Uuid topic ids and directories, nested EndPoints and Features collections) and every version and tagged field (for example PartitionChangeRecord tags and the v2 EligibleLeaderReplicas/LastKnownElr tags); unknown tagged fields are preserved on re-encode; `alluvium-metadata` exists with its `//!` crate doc and is registered as layer 1 in `xtask check-layers`.

### [To Do] META-0004 Metadata record frame codec: frame version, record type and record version varints
P0 · M2 · Task · deps: META-0003, PROTO-F09 · labels: decoder, security · ref: metadata/src/main/java/org/apache/kafka/metadata/MetadataRecordSerde.java, server-common/src/main/java/org/apache/kafka/server/common/serialization/AbstractApiMessageSerde.java @4.4.0
Done when: encode/decode of `[frame version = 1][type id][record version][record]` as unsigned varints matches Kafka byte for byte; errors `UnsupportedFrameVersion`, `UnknownRecordType`, `UnsupportedRecordVersion { record_type, version }` and truncation carry the byte offset; PROTO-F09 `DecodeLimits` bound every length before allocating.

### [To Do] META-0005 `VersionedMetadataRecord`: an exhaustive enum of all record types carrying the record version
P0 · M2 · Task · deps: META-0003 · ref: server-common/src/main/java/org/apache/kafka/server/common/ApiMessageAndVersion.java @4.4.0
Done when: the enum has one variant per record type plus the version (ApiMessageAndVersion equivalent) with `record_type()`, `version()` and `encoded_size()`; replay and writer code match on it without a `_` arm, so adding a record type fails compilation until handled (compile-fail test).

### [To Do] META-0006 Redacted `Debug` and `Display` for metadata records
P0 · M2 · Task · deps: META-0005 · labels: security · ref: metadata/src/main/java/org/apache/kafka/metadata/util/RecordRedactor.java @4.4.0
Done when: UserScramCredentialRecord prints `salt=(redacted)`, `storedKey=(redacted)`, `serverKey=(redacted)`; DelegationTokenRecord and RemoveDelegationTokenRecord token ids are redacted; ConfigRecord values are redacted when an injected sensitivity predicate (supplied by CFG, since layer 1 cannot read config definitions) says the key is sensitive; canary tests show no secret in any formatted record.

### [To Do] META-0007 Golden vectors: Kafka 4.4.0-serialized bytes for every metadata record type and version
P0 · M2 · Task · deps: META-0004, PROTO-F10 · labels: test, decoder · ref: metadata/src/main/resources/common/metadata/ @4.4.0
Done when: the PROTO-F10 golden-vector generator (a pinned Kafka 4.4.0 program) writes fixtures for every (record type, version), including variants with and without each tagged field; decode then encode is byte-identical for all of them; fixtures and the generator program are committed.

### [To Do] META-0008 Fuzz target `metadata_record_frame` for the frame and record decoders
P0 · M2 · Task · deps: META-0004, TEST-F07 · labels: decoder, security, test
Done when: the cargo-fuzz target decodes arbitrary bytes as single frames and as sequences of frames without panics, hangs or allocation beyond `DecodeLimits`; the seed corpus comes from the golden vectors; the target runs in the per-PR fuzz job.

### [To Do] META-0009 docs/formats/metadata-records.md: frame layout and the record type table
P1 · M2 · Task · deps: META-0002, META-0004 · labels: docs
Done when: the page documents the frame layout with a hex example and lists every record type (id, versions, fields per version, the metadata.version that introduced each version, the image component it updates); a test fails when the table and `MetadataRecordType` disagree.

## [To Do] META-F02 Record replay
P0 · M2 · Feature · deps: META-F01, META-F03
Scope: One task per record type: replay into the delta with validation and tests.

### [To Do] META-0010 Replay `RegisterBrokerRecord` v0-v4 into `ClusterDelta`
P0 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/RegisterBrokerRecord.json, metadata/src/main/java/org/apache/kafka/image/ClusterDelta.java replay @4.4.0
Done when: every version decodes into a `BrokerRegistration` (v1 InControlledShutdown, v2 IsMigratingZkBroker, v3 LogDirs tag, v4 CordonedLogDirs tag); a re-registration replaces the previous registration, including its epoch and incarnation id; unknown security protocol ids, ports outside 0-65535 and feature ranges with min > max return `ReplayError::InvalidBrokerRegistration` with the broker id; one test per version and per error.

### [To Do] META-0011 Replay `UnregisterBrokerRecord` into `ClusterDelta`
P0 · M3 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/UnregisterBrokerRecord.json, metadata/src/main/java/org/apache/kafka/image/ClusterDelta.java replay @4.4.0
Done when: the broker is removed from the next image; as in Kafka the epoch is carried but not compared and an unknown broker id is a no-op; register-then-unregister within one delta leaves no broker; tests for each case.

### [To Do] META-0012 Replay `TopicRecord` into `TopicsDelta`
P0 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/TopicRecord.json, metadata/src/main/java/org/apache/kafka/image/TopicsDelta.java replay @4.4.0
Done when: a new topic with no partitions is created and indexed by id and name; delete-and-recreate of the same name within one delta resolves to the latest id (Kafka's createdTopics rule); a name already bound to a different live topic id returns `ReplayError::DuplicateTopicName { name, existing_id, new_id }`; tests.

### [To Do] META-0013 Replay `PartitionRecord` v0-v2 into `TopicDelta`
P0 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/PartitionRecord.json, metadata/src/main/java/org/apache/kafka/image/TopicDelta.java replay @4.4.0
Done when: every version builds a `PartitionRegistration` (v1 Directories, v2 EligibleLeaderReplicas and LastKnownElr tags; LeaderRecoveryState tag at all versions); an existing partition is replaced and unclean/ELR election counters are updated as in TopicDelta.updateElectionStats; an unknown topic id returns `ReplayError::UnknownTopicId`; registration invariants (leader in ISR or -1, ISR subset of replicas, directories length) return typed errors; tests per version.

### [To Do] META-0014 Replay `ConfigRecord` into `ConfigurationsDelta`
P0 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/ConfigRecord.json, metadata/src/main/java/org/apache/kafka/image/ConfigurationsDelta.java, metadata/src/main/java/org/apache/kafka/image/ConfigurationDelta.java @4.4.0
Done when: a non-null value sets the key and a null value deletes it; resources with no keys left disappear from the image; keys rejected by the injected `SupportedConfigChecker` are skipped and counted, as Kafka does for removed configs; an unknown resource type id returns `ReplayError::UnknownConfigResourceType`; tests for topic, broker, broker-default (empty name), client-metrics and group resources.

### [To Do] META-0015 Replay `PartitionChangeRecord` v0-v2 into `TopicDelta`
P0 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/PartitionChangeRecord.json, metadata/src/main/java/org/apache/kafka/metadata/PartitionRegistration.java merge @4.4.0
Done when: only fields present in the record change (Leader `-2` = no change, `-1` = no leader); ISR, Replicas, Adding/RemovingReplicas, LeaderRecoveryState, Directories (v1) and ELR/LastKnownElr (v2) tags merge as in PartitionRegistration.merge; leader and partition epochs advance as in Kafka; an unknown topic or partition returns `ReplayError::UnknownPartition { topic_id, partition }`; tests per version and per tagged field.

### [To Do] META-0016 Replay `FenceBrokerRecord` into `ClusterDelta`
P0 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/FenceBrokerRecord.json, metadata/src/main/java/org/apache/kafka/image/ClusterDelta.java getBrokerOrThrow @4.4.0
Done when: the registration is cloned with `fenced = true`; an unregistered broker returns `ReplayError::UnknownBroker { broker_id }` and an epoch mismatch returns `ReplayError::BrokerEpochMismatch { broker_id, expected, actual }` (getBrokerOrThrow semantics); tests.

### [To Do] META-0017 Replay `UnfenceBrokerRecord` into `ClusterDelta`
P0 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/UnfenceBrokerRecord.json, metadata/src/main/java/org/apache/kafka/image/ClusterDelta.java getBrokerOrThrow @4.4.0
Done when: the registration is cloned with `fenced = false`; unknown broker and epoch mismatch return the same typed errors as fencing; tests including fence-then-unfence within one delta.

### [To Do] META-0018 Replay `RemoveTopicRecord` into `TopicsDelta` and `ConfigurationsDelta`
P0 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/RemoveTopicRecord.json, metadata/src/main/java/org/apache/kafka/image/TopicsDelta.java replay, metadata/src/main/java/org/apache/kafka/image/ConfigurationsDelta.java replay @4.4.0
Done when: the topic and all its partitions leave the next image and its topic configs are deleted; a topic created and removed in the same delta never appears; an unknown topic id returns `ReplayError::UnknownTopicId`; tests for each case, including removal followed by re-creation with a new id.

### [To Do] META-0019 Replay `DelegationTokenRecord` into `DelegationTokenDelta`
P1 · M4 · Task · deps: none · labels: security · ref: metadata/src/main/resources/common/metadata/DelegationTokenRecord.json, metadata/src/main/java/org/apache/kafka/image/DelegationTokenDelta.java @4.4.0
Done when: owner, requester, renewers, issue, max and expiration timestamps and token id build a `DelegationTokenData`; replay of an existing token id updates it (renewal); principals that do not parse as `type:name` return `ReplayError::InvalidPrincipal`; tests.

### [To Do] META-0020 Replay `RemoveDelegationTokenRecord` into `DelegationTokenDelta`
P1 · M4 · Task · deps: none · labels: security · ref: metadata/src/main/resources/common/metadata/RemoveDelegationTokenRecord.json, metadata/src/main/java/org/apache/kafka/image/DelegationTokenDelta.java @4.4.0
Done when: the token leaves the next image; add-then-remove within one delta leaves nothing; removing an unknown token id is a no-op as in Kafka; tests.

### [To Do] META-0021 Replay `UserScramCredentialRecord` into `ScramDelta`
P0 · M4 · Task · deps: none · labels: security · ref: metadata/src/main/resources/common/metadata/UserScramCredentialRecord.json, metadata/src/main/java/org/apache/kafka/image/ScramDelta.java @4.4.0
Done when: mechanism 1 (SCRAM-SHA-256) and 2 (SCRAM-SHA-512) store salt, stored key, server key (as `Secret`) and iterations per user; an unknown mechanism returns `ReplayError::UnknownScramMechanism`; iterations below the mechanism minimum return `ReplayError::InvalidScramIterations`; replacing a credential works; tests.

### [To Do] META-0022 Replay `RemoveUserScramCredentialRecord` into `ScramDelta`
P0 · M4 · Task · deps: none · labels: security · ref: metadata/src/main/resources/common/metadata/RemoveUserScramCredentialRecord.json, metadata/src/main/java/org/apache/kafka/image/ScramDelta.java @4.4.0
Done when: the (user, mechanism) credential leaves the next image while the user's other mechanism stays; an unknown mechanism returns the typed error; tests.

### [To Do] META-0023 Replay `FeatureLevelRecord` into `FeaturesDelta`
P0 · M2 · Task · deps: META-0094 · ref: metadata/src/main/resources/common/metadata/FeatureLevelRecord.json, metadata/src/main/java/org/apache/kafka/image/FeaturesDelta.java replay, metadata/src/main/java/org/apache/kafka/image/MetadataDelta.java replay(FeatureLevelRecord) @4.4.0
Done when: `metadata.version` levels outside 7-33 (or 34 with unstable versions enabled) return `ReplayError::UnsupportedMetadataVersion` carrying Kafka's upgrade guidance text; `kraft.version` records are skipped (KAFKA-18979); level 0 removes a feature; an MV change notifies every component delta; tests for each branch.

### [To Do] META-0024 Replay `ClientQuotaRecord` into `ClientQuotasDelta`
P1 · M4 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/ClientQuotaRecord.json, metadata/src/main/java/org/apache/kafka/image/ClientQuotaDelta.java @4.4.0
Done when: entity parts (user, client-id, ip; null name = default entity) select the quota entity; `Remove = true` deletes the key and otherwise sets the f64 value; an unknown entity type or an ip combined with user/client-id returns `ReplayError::InvalidQuotaEntity`; tests for every entity combination.

### [To Do] META-0025 Replay `ProducerIdsRecord` into `ProducerIdsDelta`
P0 · M6 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/ProducerIdsRecord.json, metadata/src/main/java/org/apache/kafka/image/ProducerIdsDelta.java, metadata/src/main/java/org/apache/kafka/controller/ProducerIdControlManager.java replay @4.4.0
Done when: `NextProducerId` becomes the image's next producer id; a value not greater than the current one returns `ReplayError::ProducerIdNotIncreasing` (mirrors ProducerIdControlManager.replay); broker id and epoch are kept for diagnostics; tests.

### [To Do] META-0026 Replay `BrokerRegistrationChangeRecord` v0-v3 into `ClusterDelta`
P0 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/BrokerRegistrationChangeRecord.json, metadata/src/main/java/org/apache/kafka/image/ClusterDelta.java replay(BrokerRegistrationChangeRecord) @4.4.0
Done when: Fenced (-1 unfence, 0 none, 1 fence) and InControlledShutdown (v1) apply through the fencing and controlled-shutdown change enums, LogDirs (v2, empty = no change) and CordonedLogDirs (v3) replace directories; unknown enum values return `ReplayError::InvalidBrokerChange`; a change that alters nothing is not recorded; unknown broker and epoch mismatch errors as for fencing; tests per version.

### [To Do] META-0027 Replay `AccessControlEntryRecord` into `AclsDelta`
P0 · M4 · Task · deps: none · labels: security · ref: metadata/src/main/resources/common/metadata/AccessControlEntryRecord.json, metadata/src/main/java/org/apache/kafka/image/AclsDelta.java, metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAclWithId.java @4.4.0
Done when: the record builds a `StandardAcl` keyed by its Uuid id; unknown resource type, pattern type, operation or permission codes and unparsable principals return `ReplayError::InvalidAcl { id }`; tests for every resource type and pattern type.

### [To Do] META-0028 Replay `RemoveAccessControlEntryRecord` into `AclsDelta`
P0 · M4 · Task · deps: none · labels: security · ref: metadata/src/main/resources/common/metadata/RemoveAccessControlEntryRecord.json, metadata/src/main/java/org/apache/kafka/image/AclsDelta.java replay @4.4.0
Done when: an id in the image is marked removed; an id added in the same delta is dropped from the changes; any other id returns `ReplayError::UnknownAclId` (Kafka's IllegalStateException); tests for the three cases.

### [To Do] META-0029 Replay `ClearElrRecord` into `TopicsDelta`
P0 · M3 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/ClearElrRecord.json, metadata/src/main/java/org/apache/kafka/image/TopicsDelta.java replay(ClearElrRecord) @4.4.0
Done when: a non-empty TopicName clears ELR and LastKnownElr on the latest id for that name (created topics checked first); an empty name clears every live topic including ones created in the delta; deleted topic ids are skipped; an unknown name returns `ReplayError::UnknownTopicName`; tests port TopicsDeltaTest ClearElr cases.

### [To Do] META-0030 Replay `NoOpRecord`: advance provenance only
P1 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/NoOpRecord.json, metadata/src/main/java/org/apache/kafka/image/MetadataDelta.java replay @4.4.0
Done when: replaying a NoOpRecord creates no component delta (asserted), and the published image differs from the previous one only in provenance offset and epoch; test.

### [To Do] META-0031 Accept `ZkMigrationStateRecord` as a no-op
P2 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/ZkMigrationRecord.json, metadata/src/main/java/org/apache/kafka/image/MetadataDelta.java replay @4.4.0
Done when: records with any ZkMigrationState value 0-4 are accepted and ignored, as Kafka 4.x does for clusters migrated in 3.x; the writer never emits the record (test); a log containing it replays to the same image as one without it.

### [To Do] META-0032 Replay `RegisterControllerRecord` into `ClusterDelta`
P0 · M3 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/RegisterControllerRecord.json, metadata/src/main/java/org/apache/kafka/image/ClusterDelta.java replay(RegisterControllerRecord) @4.4.0
Done when: the record builds a `ControllerRegistration` (incarnation id, ZkMigrationReady, endpoints, supported features) that replaces any earlier one for the id; endpoint and feature-range validation errors as for brokers; tests.

### [To Do] META-0033 Replay `UnregisterControllerRecord` into `ClusterDelta`
P0 · M3 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/UnregisterControllerRecord.json, metadata/src/main/java/org/apache/kafka/image/ClusterDelta.java replay(UnregisterControllerRecord) @4.4.0
Done when: the controller registration leaves the next image; an unknown controller id is a no-op as in Kafka; tests.

### [To Do] META-0034 `BeginTransactionRecord`: metadata transaction start in the transaction state machine
P0 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/BeginTransactionRecord.json, metadata/src/main/java/org/apache/kafka/image/loader/MetadataBatchLoader.java @4.4.0
Done when: a pure `MetadataTransactionState` (none, started, continued, ended, aborted) moves to started and keeps the optional Name tag; a Begin while a transaction is open returns `ReplayError::NestedTransaction`; `MetadataDelta::replay` rejects the record so only the loader handles it; tests.

### [To Do] META-0035 `EndTransactionRecord`: metadata transaction commit in the transaction state machine
P0 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/EndTransactionRecord.json, metadata/src/main/java/org/apache/kafka/image/loader/MetadataBatchLoader.java @4.4.0
Done when: End moves an open transaction to ended; End without an open transaction returns `ReplayError::NoOpenTransaction`; tests.

### [To Do] META-0036 `AbortTransactionRecord`: metadata transaction abort in the transaction state machine
P0 · M2 · Task · deps: none · ref: metadata/src/main/resources/common/metadata/AbortTransactionRecord.json, metadata/src/main/java/org/apache/kafka/image/loader/MetadataBatchLoader.java @4.4.0
Done when: Abort moves an open transaction to aborted and keeps the optional Reason tag for the abort event; Abort without an open transaction returns `ReplayError::NoOpenTransaction`; tests.

### [To Do] META-0037 Replay coverage check: every record type and version has a replay test fixture
P1 · M2 · Task · deps: none · labels: test
Done when: a test enumerates `MetadataRecordType` and each supported version and fails when a (type, version) has no replay fixture; record types whose component lands in a later milestone are listed in an explicit, shrinking allow-list.

### [To Do] META-0038 Property test: one delta or many deltas over the same record stream give the same image
P0 · M2 · Task · deps: META-0010, META-0012, META-0013, META-0015, META-0014, META-0018, META-0016, META-0017, META-0023, META-0026, TEST-F16 · labels: test
Done when: a generator of consistent record streams (brokers, fencing, topics, partitions, partition changes, configs, features, removals) replays the stream in one delta and with arbitrary split points; the final images are equal and `check_invariants()` passes after every apply; seeds print on failure.

### [To Do] META-0039 Differential replay against Kafka 4.4.0: topics, partitions, configs, features and brokers
P0 · M2 · Task · deps: META-0038, META-0061, TEST-F09 · labels: test · ref: shell/src/main/java/org/apache/kafka/shell/MetadataShell.java @4.4.0
Done when: a TEST-F09 scenario captures the metadata records of a Kafka 4.4.0 node (topic create/delete, partition add, config changes, broker registration and fencing); Alluvium replays the same frames and its image tree equals kafka-metadata-shell's view of Kafka's image after normalization.

### [To Do] META-0040 Differential replay against Kafka 4.4.0: ACLs, SCRAM, delegation tokens, quotas and producer ids
P1 · M6 · Task · deps: META-0039, META-0027, META-0028, META-0021, META-0022, META-0019, META-0020, META-0024, META-0025 · labels: test
Done when: the differential scenario is extended with ACL create/delete, SCRAM upsert/delete, delegation token create/renew/expire, client quota changes and producer id block allocation; Alluvium's image tree equals Kafka's with secrets redacted on both sides.

## [To Do] META-F03 Image and delta
P0 · M2 · Feature · deps: TYPE-F05
Scope: Immutable `MetadataImage` (cluster, topics, configs, quotas, producer ids, ACLs, SCRAM, tokens, features, provenance) and `MetadataDelta`.

### [To Do] META-0041 Spike: persistent representation of `MetadataImage` (structural sharing, apply cost, memory)
P0 · M2 · Spike · deps: none · labels: perf
Done when: docs/adr/NNNN-metadata-image-representation.md is accepted; it compares Arc-shared BTreeMaps per component, persistent maps (for example `imbl` or `rpds`) and Kafka-style copy-on-apply using a prototype benchmark (apply a 1-partition delta to a 1M-partition image; memory per partition; deterministic iteration order) and justifies any new dependency.

### [To Do] META-0042 `MetadataProvenance` and the empty image
P0 · M2 · Task · deps: META-0041 · ref: metadata/src/main/java/org/apache/kafka/image/MetadataProvenance.java @4.4.0
Done when: `MetadataProvenance { last_contained_offset, last_contained_epoch, last_contained_log_time_ms, is_offset_batch_aligned }` derives the snapshot id (offset + 1, epoch); `MetadataImage::EMPTY` has offset -1 and empty components; unit tests.

### [To Do] META-0043 `BrokerRegistration`: listeners, supported features, rack, fencing, controlled shutdown and directories
P0 · M2 · Task · deps: META-0003 · ref: metadata/src/main/java/org/apache/kafka/metadata/BrokerRegistration.java @4.4.0
Done when: the type holds id, epoch, incarnation id, listeners with security protocol, supported feature ranges, rack, fenced, in-controlled-shutdown, is-migrating-zk-broker, sorted directories and cordoned directories; `from_record`, `to_record(record_version, loss_handler)` (the caller picks the version from the metadata.version) and `clone_with` for fencing, shutdown and directory changes; `has_online_dir`; ported BrokerRegistrationTest cases pass.

### [To Do] META-0044 `ControllerRegistration`: incarnation id, endpoints and supported features
P0 · M2 · Task · deps: META-0003 · ref: metadata/src/main/java/org/apache/kafka/metadata/ControllerRegistration.java @4.4.0
Done when: `from_record`/`to_record(record_version)` round-trip RegisterControllerRecord; equality and display; ported ControllerRegistrationTest cases pass.

### [To Do] META-0045 `PartitionRegistration` and `LeaderRecoveryState`: replicas, directories, ISR, ELR, reassignment and epochs
P0 · M2 · Task · deps: META-0003 · ref: metadata/src/main/java/org/apache/kafka/metadata/PartitionRegistration.java, metadata/src/main/java/org/apache/kafka/metadata/LeaderRecoveryState.java @4.4.0
Done when: fields follow PartitionRecord v2; `merge(PartitionChangeRecord)` applies only present fields with Kafka's leader sentinels (-1 no leader, -2 no change); `is_reassigning`, `has_leader`, `election_was_unclean`; `to_record(record_version)` takes the version chosen by the writer; ported PartitionRegistrationTest cases pass.

### [To Do] META-0046 Security domain types: `StandardAcl`, `ScramCredentialData` and `DelegationTokenData`
P0 · M4 · Task · deps: META-0003 · labels: security · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAcl.java, metadata/src/main/java/org/apache/kafka/metadata/ScramCredentialData.java, metadata/src/main/java/org/apache/kafka/metadata/DelegationTokenData.java @4.4.0
Done when: StandardAcl covers resource type, name, pattern type, principal, host, operation and permission with typed decode errors for unknown codes; SCRAM salt, stored key and server key and token HMAC-relevant ids are held in `Secret` and redacted in `Debug`; unit tests.

### [To Do] META-0047 `FeaturesImage` and `FeaturesDelta`
P0 · M2 · Task · deps: META-0041 · ref: metadata/src/main/java/org/apache/kafka/image/FeaturesImage.java, metadata/src/main/java/org/apache/kafka/image/FeaturesDelta.java @4.4.0
Done when: the image holds finalized levels in a BTreeMap plus the metadata.version feature level (typed conversion lives in META-F06, which depends on this feature); the delta tracks set and removed features and the MV change; `apply`, `finish_snapshot` (features not seen in a snapshot are removed) and `metadata_version_or_error`; unit tests.

### [To Do] META-0048 `ClusterImage` and `ClusterDelta`: brokers and controllers
P0 · M2 · Task · deps: META-0041, META-0043, META-0044 · ref: metadata/src/main/java/org/apache/kafka/image/ClusterImage.java, metadata/src/main/java/org/apache/kafka/image/ClusterDelta.java @4.4.0
Done when: maps of broker and controller registrations; the delta records replacements and removals and answers `broker(id)` through pending changes; `finish_snapshot` removes nodes absent from a snapshot; `apply` reuses unchanged entries; ported ClusterImageTest cases pass.

### [To Do] META-0049 `TopicsImage`/`TopicImage` and `TopicsDelta`/`TopicDelta`: topics by id and by name with partitions
P0 · M2 · Task · deps: META-0041, META-0045 · ref: metadata/src/main/java/org/apache/kafka/image/TopicsImage.java, metadata/src/main/java/org/apache/kafka/image/TopicsDelta.java, metadata/src/main/java/org/apache/kafka/image/TopicDelta.java @4.4.0
Done when: id and name indexes stay consistent; deltas track created topics (name to latest id), deleted ids and per-topic partition changes; `apply` reports a missing topic id as a typed error; ported TopicsImageTest cases pass.

### [To Do] META-0050 `LocalReplicaChanges`: per-broker leader, follower, delete and directory changes from a `TopicsDelta`
P0 · M2 · Task · deps: META-0049 · ref: metadata/src/main/java/org/apache/kafka/image/LocalReplicaChanges.java, metadata/src/main/java/org/apache/kafka/image/TopicDelta.java localChanges @4.4.0
Done when: for a broker id the result lists partitions that become leader, become follower, are deleted (with topic id), changed leader epoch, and changed directory ids, matching TopicDelta.localChanges for every case in ported TopicsDeltaTest/LocalReplicaChangesTest cases; REPL consumes it through SRV-F14.

### [To Do] META-0051 `ConfigurationsImage`/`ConfigurationImage` and their deltas
P0 · M2 · Task · deps: META-0041 · ref: metadata/src/main/java/org/apache/kafka/image/ConfigurationsImage.java, metadata/src/main/java/org/apache/kafka/image/ConfigurationsDelta.java @4.4.0
Done when: config resource (type, name) maps to key/value pairs; set, delete, `delete_all` for removed topics, and removal of empty resources; unsupported keys are filtered through an injected `SupportedConfigChecker`; unit tests.

### [To Do] META-0052 `ClientQuotasImage`/`ClientQuotaImage` and their deltas
P1 · M4 · Task · deps: META-0058 · ref: metadata/src/main/java/org/apache/kafka/image/ClientQuotasImage.java, metadata/src/main/java/org/apache/kafka/image/ClientQuotasDelta.java @4.4.0
Done when: quota entities (user, client-id, ip; default entity = no name) map to quota keys with f64 values; removal and `finish_snapshot`; the component is wired into `MetadataImage`/`MetadataDelta` and the image tree view; unit tests.

### [To Do] META-0053 `ClientQuotasImage::describe`: DescribeClientQuotas filter matching
P1 · M4 · Task · deps: META-0052 · ref: metadata/src/main/java/org/apache/kafka/image/ClientQuotasImage.java describe @4.4.0
Done when: exact, default and any match types and strict mode follow ClientQuotasImage.describe, including INVALID_REQUEST for duplicate entity types and for ip combined with user or client-id; a property test compares with a brute-force matcher; a benchmark over 100k entities is recorded.

### [To Do] META-0054 `ProducerIdsImage` and `ProducerIdsDelta`
P0 · M6 · Task · deps: META-0058 · ref: metadata/src/main/java/org/apache/kafka/image/ProducerIdsImage.java, metadata/src/main/java/org/apache/kafka/image/ProducerIdsDelta.java @4.4.0
Done when: the image holds the next producer id (-1 when empty) with `is_empty`; the component is wired into the aggregates and the tree view; unit tests.

### [To Do] META-0055 `AclsImage` and `AclsDelta`
P0 · M4 · Task · deps: META-0046, META-0058 · labels: security · ref: metadata/src/main/java/org/apache/kafka/image/AclsImage.java, metadata/src/main/java/org/apache/kafka/image/AclsDelta.java @4.4.0
Done when: ACL id to StandardAcl; additions and removals with add-then-remove cancellation inside one delta; `finish_snapshot`; wired into the aggregates and the tree view; ported AclsDeltaTest cases pass.

### [To Do] META-0056 `ScramImage` and `ScramDelta`, with `describe` for DescribeUserScramCredentials
P0 · M4 · Task · deps: META-0046, META-0058 · labels: security · ref: metadata/src/main/java/org/apache/kafka/image/ScramImage.java, metadata/src/main/java/org/apache/kafka/image/ScramDelta.java @4.4.0
Done when: mechanism to user to credential; removal per mechanism; `describe(users)` returns mechanisms and iterations per user, RESOURCE_NOT_FOUND for unknown users and DUPLICATE_RESOURCE for repeated users as in ScramImage.describe; wired into the aggregates and the tree view (redacted); tests.

### [To Do] META-0057 `DelegationTokenImage` and `DelegationTokenDelta`
P1 · M4 · Task · deps: META-0046, META-0058 · labels: security · ref: metadata/src/main/java/org/apache/kafka/image/DelegationTokenImage.java, metadata/src/main/java/org/apache/kafka/image/DelegationTokenDelta.java @4.4.0
Done when: token id to DelegationTokenData; add, renew and remove; `finish_snapshot`; wired into the aggregates and the tree view (redacted); unit tests.

### [To Do] META-0058 `MetadataImage` and `MetadataDelta` aggregates with structural sharing on apply
P0 · M2 · Task · deps: META-0042, META-0047, META-0048, META-0049, META-0051 · ref: metadata/src/main/java/org/apache/kafka/image/MetadataImage.java, metadata/src/main/java/org/apache/kafka/image/MetadataDelta.java @4.4.0
Done when: component deltas are created lazily; `apply(provenance)` returns an image whose untouched components are pointer-equal to the previous image's (test); `metadata_version_changed()` and `finish_snapshot()` delegate to all components; components added in later milestones plug into the same aggregates.

### [To Do] META-0059 `MetadataDelta::replay` dispatch and the typed `ReplayError`
P0 · M2 · Task · deps: META-0058, META-0005 · ref: metadata/src/main/java/org/apache/kafka/image/MetadataDelta.java replay @4.4.0
Done when: dispatch matches every record type without a `_` arm; types whose component is not built yet return `ReplayError::UnsupportedRecordType`; Begin/End/AbortTransaction are rejected here because the loader owns them; `ReplayError` variants carry broker id and epoch, topic id, partition, ACL id or feature name, and render a fault description used by META-F04; unit tests.

### [To Do] META-0060 Image invariant checker run after every apply in debug builds
P0 · M2 · Task · deps: META-0058 · labels: test
Done when: `MetadataImage::check_invariants()` verifies the topic name/id bijection, dense partition ids, ISR subset of replicas, leader in ISR or -1, ELR disjoint from ISR, directory list length, non-negative broker epochs and unique ACL ids, returning every violation; `apply` calls it under `debug_assert!`; one test per violation.

### [To Do] META-0061 Image tree view for diagnostics and dumps (MetadataNode equivalent) with redaction
P1 · M2 · Task · deps: META-0058 · ref: metadata/src/main/java/org/apache/kafka/image/node/MetadataNode.java, metadata/src/main/java/org/apache/kafka/image/node/printer/MetadataNodeRedactionCriteria.java @4.4.0
Done when: a read-only tree with kafka-metadata-shell's paths (`/features`, `/cluster/brokers/<id>`, `/topics/byName/<name>/<partition>`, `/topics/byId/<id>`, `/configs/<type>/<name>`, `/provenance`) renders as text and JSON; SCRAM, token and sensitive config values are redacted unless a `RedactionCriteria` allows them; tests compare paths with Kafka's node classes.

### [To Do] META-0062 Benchmark: delta apply cost and memory per partition on large images
P1 · M2 · Task · deps: META-0058, TEST-F11 · labels: perf, nfr
Done when: criterion benches measure applying 1, 1k and 100k partition changes to 10k-topic and 1M-partition images and building an image from scratch; results and per-partition memory are recorded as budgets in the NFR-F02 and NFR-F05 docs; the CI benchmark job flags regressions.

### [To Do] META-0063 Module docs for image and delta: lifecycle, invariants and ownership
P1 · M2 · Task · deps: META-0058, META-0060 · labels: docs
Done when: `alluvium-metadata` crate and module `//!` docs describe image immutability, the delta lifecycle, `finish_snapshot` semantics, the invariant list and who reads the image; docs/architecture.md links them; doc tests compile.

## [To Do] META-F04 Image loading and publishing
P0 · M2 · Feature · deps: META-F03, RAFT-F10
Scope: Loader that applies committed batches and snapshots, then publishes images to subscribers.

### [To Do] META-0064 Loader state machine types: inputs, outputs and manifests
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/image/loader/LoaderManifest.java, metadata/src/main/java/org/apache/kafka/image/loader/LogDeltaManifest.java, metadata/src/main/java/org/apache/kafka/image/loader/SnapshotManifest.java @4.4.0
Done when: a sans-I/O `MetadataLoader` takes `CommittedBatch { base_offset, last_offset, epoch, append_time_ms, size_bytes, records }`, `SnapshotLoaded`, `LeaderChanged` and `HighWatermark` inputs and returns `Publish { delta, image, manifest }`, `ControllerChanged` and `Fault` outputs; `LogDeltaManifest` and `SnapshotManifest` carry provenance, leader and epoch, batch count, elapsed time from the injected Clock and bytes; unit tests.

### [To Do] META-0065 Committed batch loading: accumulate batches into one delta and flush at the end of a commit
P0 · M2 · Task · deps: META-0064 · ref: metadata/src/main/java/org/apache/kafka/image/loader/MetadataBatchLoader.java @4.4.0
Done when: records replay in order with error context (offset, index in batch, record type); the delta is published when the commit reader is exhausted (MetadataBatchLoader.loadBatch/maybeFlushBatches) with `is_offset_batch_aligned` set; a replay error leaves the image unchanged and returns `Fault`; ported MetadataBatchLoaderTest batch cases pass.

### [To Do] META-0066 Metadata transactions (KIP-868) in the loader: no publish mid-transaction, abort restores the prior image
P0 · M2 · Task · deps: META-0065, META-0034, META-0035, META-0036 · ref: metadata/src/main/java/org/apache/kafka/image/loader/MetadataBatchLoader.java, KIP-868 @4.4.0
Done when: a transaction spanning batches publishes once after End; Abort discards the delta and resets to the image before Begin; records before a Begin in the same batch are flushed first (Kafka's rule); ported MetadataBatchLoaderTest transaction cases pass.

### [To Do] META-0067 Snapshot loading: rebuild the image from a snapshot reader and publish it with a SnapshotManifest
P0 · M2 · Task · deps: META-0064 · ref: metadata/src/main/java/org/apache/kafka/image/loader/MetadataLoader.java loadSnapshot @4.4.0
Done when: the delta starts from the empty image; control records (SnapshotHeader, SnapshotFooter, KRaftVersion, Voters, LeaderChange) are skipped; `finish_snapshot` runs before apply; a replay error names the snapshot id and record index; a snapshot-load counter output exists; tests.

### [To Do] META-0068 Catch-up gating: publish only after reaching the high watermark and seeing a record
P0 · M2 · Task · deps: META-0065 · ref: metadata/src/main/java/org/apache/kafka/image/loader/MetadataLoader.java stillNeedToCatchUp @4.4.0
Done when: publishing is withheld while the high watermark is unknown, the last applied offset is below HWM - 1, or no record has been loaded (stillNeedToCatchUp); a one-time `CaughtUp` output feeds the OBS-F09 `metadata_caught_up` readiness condition; one test per branch.

### [To Do] META-0069 Publisher registry: install, initial full-image delivery, removal and deterministic order
P0 · M2 · Task · deps: META-0068 · ref: metadata/src/main/java/org/apache/kafka/image/publisher/MetadataPublisher.java, metadata/src/main/java/org/apache/kafka/image/loader/MetadataLoader.java installPublishers @4.4.0
Done when: `MetadataPublisher` has `name`, `on_metadata_update(delta, image, manifest)` and `on_controller_change`; a newly installed publisher first receives the current image as a delta from the empty image with a SnapshotManifest; publishers run in installation order; installs before catch-up are deferred; removal closes the publisher; tests.

### [To Do] META-0070 Fault reporting for replay and publisher errors, with a role-specific policy
P0 · M2 · Task · deps: META-0069 · ref: core/src/main/scala/kafka/server/SharedServer.scala @4.4.0
Done when: a failing publisher does not stop later publishers and yields `Fault { publisher, offset }`; replay faults yield `Fault { offset, record_type, error }`; the driver policy is fatal on controllers and count-and-continue on brokers (load and apply error counters), as in SharedServer's fault handlers; tests for both roles.

### [To Do] META-0071 Controller leader changes: notify publishers and track the current controller id
P1 · M2 · Task · deps: META-0069 · ref: metadata/src/main/java/org/apache/kafka/image/loader/MetadataLoader.java handleLeaderChange @4.4.0
Done when: `LeaderChanged` calls `on_controller_change` on every installed publisher in order and updates the current-controller-id output that feeds the CurrentControllerId metric; test.

### [To Do] META-0072 Loader driver in `alluvium-server`: bounded input from the Raft client on a supervised task
P0 · M2 · Task · deps: META-0070, META-0071, SRV-F02, RAFT-F11
Done when: the driver receives RAFT-F10 commit, snapshot and leader notifications through a bounded channel (full channel = backpressure on the Raft listener, never dropped input), runs the loader on the supervised `metadata-loader` task inside a `driver{component}` span, executes Publish outputs and closes publishers in reverse install order on shutdown; integration test with a fake Raft client.

### [To Do] META-0073 Loader observability: events, flight recorder entries and loader metrics
P1 · M2 · Task · deps: META-0072, OBS-F04, OBS-F06, OBS-F07 · ref: metadata/src/main/java/org/apache/kafka/image/loader/metrics/MetadataLoaderMetrics.java @4.4.0
Done when: events `metadata_snapshot_loaded`, `metadata_caught_up`, `metadata_version_changed` (old, new, reason) and `metadata_publish_failed` (publisher, offset) are in events.rs; each transition is recorded in the loader's flight recorder; the driver registers `alluvium_metadata_loader_current_metadata_version`, `alluvium_metadata_loader_snapshot_loads_total`, `alluvium_metadata_loader_last_applied_offset`, `alluvium_metadata_loader_current_controller_id`, `alluvium_metadata_loader_idle_ratio` and `alluvium_metadata_finalized_feature_level{feature}`; tests assert each.

### [To Do] META-0074 Simulation test: the loader under arbitrary batch boundaries, snapshot loads and aborted transactions
P0 · M2 · Task · deps: META-0066, META-0067, META-0071, TEST-F04 · labels: test
Done when: a seeded TEST-F04 run feeds a reference record stream with random batch splits, snapshot loads at random offsets, aborted transactions and leader changes; after every publish the image equals a reference replay of the committed, non-aborted records up to the provenance offset; failures print the seed and replay command.

### [To Do] META-0075 Benchmark: replay throughput and commit-to-publish latency
P1 · M2 · Task · deps: META-0065, TEST-F11 · labels: perf
Done when: criterion benches report records per second for PartitionChangeRecord-heavy batches and commit-to-publish time on a 1M-partition image; budgets are recorded with NFR-F01; the CI benchmark job flags regressions.

## [To Do] META-F05 Image snapshots
P0 · M2 · Feature · deps: META-F03, RAFT-F06
Scope: Write an image as snapshot records and load it back; round-trip tests.

### [To Do] META-0076 `ImageWriter` and `ImageWriterOptions`: target metadata.version and loss handler
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/image/writer/ImageWriter.java, metadata/src/main/java/org/apache/kafka/image/writer/ImageWriterOptions.java, metadata/src/main/java/org/apache/kafka/image/writer/RecordListWriter.java @4.4.0
Done when: the writer trait has `write(record)` and `close(complete)`; options carry the target MV and a loss handler receiving `UnwritableMetadata { what, reason }`; writes after close return `WriterClosed`; `RecordListWriter` collects records for tests; unit tests.

### [To Do] META-0077 Snapshot round-trip for `FeaturesImage`: metadata.version first, then finalized features
P0 · M2 · Task · deps: META-0076, META-F02 · ref: metadata/src/main/java/org/apache/kafka/image/FeaturesImage.java write @4.4.0
Done when: the MV FeatureLevelRecord is written first and other features follow sorted by name; `kraft.version` is never written; write, replay into an empty delta, `finish_snapshot` and apply returns an equal image (test).

### [To Do] META-0078 Snapshot round-trip for `ClusterImage`: brokers and controllers
P0 · M2 · Task · deps: META-0076, META-0095, META-F02 · ref: metadata/src/main/java/org/apache/kafka/image/ClusterImage.java write @4.4.0
Done when: RegisterBrokerRecord uses the MV's register-broker version with fencing and controlled-shutdown state; RegisterControllerRecord is written only when the MV supports controller registration; directories and cordoned directories below their introducing MV go to the loss handler; round-trip tests at MVs 3.3-IV3, 3.7-IV2 and 4.4-IV2.

### [To Do] META-0079 Snapshot round-trip for `TopicsImage`
P0 · M2 · Task · deps: META-0076, META-0095, META-F02 · ref: metadata/src/main/java/org/apache/kafka/image/TopicsImage.java write @4.4.0
Done when: TopicRecord then one PartitionRecord per partition at the MV's partition record version, topics in id order; ELR fields below the ELR MV and directories below the directory-assignment MV go to the loss handler; round-trip tests for topics with reassignments, ELR and directories.

### [To Do] META-0080 Snapshot round-trip for `ConfigurationsImage`
P0 · M2 · Task · deps: META-0076, META-F02 · ref: metadata/src/main/java/org/apache/kafka/image/ConfigurationsImage.java write @4.4.0
Done when: one ConfigRecord per key, resources and keys in sorted order; round-trip test for every resource type including the broker-default resource with an empty name.

### [To Do] META-0081 Write the full `MetadataImage` in Kafka's component order
P0 · M2 · Task · deps: META-0077, META-0078, META-0079, META-0080 · ref: metadata/src/main/java/org/apache/kafka/image/MetadataImage.java write @4.4.0
Done when: components are written as features, cluster, topics, configs, client quotas, producer ids, ACLs, SCRAM, delegation tokens (MetadataImage.write), with later components slotting into this order; the empty image writes only the MV record; test.

### [To Do] META-0082 Raft snapshot writer adapter: write an image into a RAFT-F06 snapshot in bounded batches
P0 · M2 · Task · deps: META-0081 · ref: metadata/src/main/java/org/apache/kafka/image/writer/RaftSnapshotWriter.java @4.4.0
Done when: records are grouped into batches bounded by bytes and record count; the snapshot is frozen only after a complete write and is aborted and deleted on error; test against an in-memory snapshot store.

### [To Do] META-0083 Snapshot round-trip for `MetadataProvenance`
P0 · M2 · Task · deps: META-0082 · ref: metadata/src/main/java/org/apache/kafka/image/MetadataProvenance.java @4.4.0
Done when: the snapshot id is (last contained offset + 1, epoch) and the SnapshotHeader's last-contained log timestamp comes from the provenance; loading the snapshot restores identical provenance; test.

### [To Do] META-0084 Snapshot round-trip for `ClientQuotasImage`
P1 · M4 · Task · deps: META-0081, META-0052 · ref: metadata/src/main/java/org/apache/kafka/image/ClientQuotasImage.java write @4.4.0
Done when: one ClientQuotaRecord per (entity, key) in sorted order; round-trip tests for user, client-id, ip and default entities.

### [To Do] META-0085 Snapshot round-trip for `ProducerIdsImage`
P0 · M6 · Task · deps: META-0081, META-0054 · ref: metadata/src/main/java/org/apache/kafka/image/ProducerIdsImage.java write @4.4.0
Done when: a non-empty image writes one ProducerIdsRecord with the next producer id; an empty image writes nothing; round-trip test.

### [To Do] META-0086 Snapshot round-trip for `AclsImage`
P0 · M4 · Task · deps: META-0081, META-0055 · labels: security · ref: metadata/src/main/java/org/apache/kafka/image/AclsImage.java write @4.4.0
Done when: one AccessControlEntryRecord per ACL in id order; CIDR host entries below the CIDR-ACL MV go to the loss handler; round-trip test for every resource and pattern type.

### [To Do] META-0087 Snapshot round-trip for `ScramImage`
P0 · M4 · Task · deps: META-0081, META-0056 · labels: security · ref: metadata/src/main/java/org/apache/kafka/image/ScramImage.java write @4.4.0
Done when: one UserScramCredentialRecord per (user, mechanism); below the SCRAM MV credentials go to the loss handler; round-trip test; no credential bytes appear in logs during the test.

### [To Do] META-0088 Snapshot round-trip for `DelegationTokenImage`
P1 · M4 · Task · deps: META-0081, META-0057 · labels: security · ref: metadata/src/main/java/org/apache/kafka/image/DelegationTokenImage.java write @4.4.0
Done when: one DelegationTokenRecord per token; below the delegation-token MV tokens go to the loss handler; round-trip test.

### [To Do] META-0089 Snapshot emitter: generate a snapshot on request with size and age metrics and an event
P1 · M2 · Task · deps: META-0082, OBS-F04, OBS-F07 · ref: metadata/src/main/java/org/apache/kafka/image/publisher/SnapshotEmitter.java @4.4.0
Done when: `emit(image)` writes through the Raft adapter and returns bytes written; the driver records `alluvium_metadata_snapshot_latest_bytes` and `alluvium_metadata_snapshot_latest_age_seconds` and emits `metadata_snapshot_generated` (snapshot id, bytes, elapsed_ms); write errors are reported without crashing; the schedule stays with CTRL-F18; tests.

### [To Do] META-0090 Lossy writes at an older metadata.version: loss cases and fail-on-loss mode
P1 · M2 · Task · deps: META-0081, META-0095 · ref: metadata/src/main/java/org/apache/kafka/image/writer/ImageWriterOptions.java, metadata/src/main/java/org/apache/kafka/image/writer/UnwritableMetadataException.java @4.4.0
Done when: every loss case (ELR fields, directories, cordoned directories, controller registrations) produces a descriptive `UnwritableMetadata`; options make loss fatal (controller snapshots) or reported-only (downgrade with `--unsafe` in CTRL-F17); table-driven tests.

### [To Do] META-0091 Property test: image to snapshot to image is the identity and re-writing is byte-identical
P0 · M2 · Task · deps: META-0081, META-0083, META-F02, TEST-F16 · labels: test
Done when: for arbitrary images, writing then loading (replay into an empty delta plus `finish_snapshot`) returns an equal image, and writing the loaded image again yields identical bytes; seeds print on failure.

### [To Do] META-0092 Differential: load a Kafka 4.4.0-written snapshot and write it back
P0 · M2 · Task · deps: META-0082, META-0061, TEST-F09 · labels: test · ref: shell/src/main/java/org/apache/kafka/shell/MetadataShell.java @4.4.0
Done when: a TEST-F09 scenario produces a `.checkpoint` from Kafka 4.4.0 with topics, configs, brokers and features; Alluvium loads it into an image whose tree equals kafka-metadata-shell's view, and Alluvium's own snapshot of that image holds the same records per component (order-insensitive within a component).

### [To Do] META-0093 docs/formats/metadata-snapshot.md: snapshot layout, component order and loss cases
P1 · M2 · Task · deps: META-0082, META-0090 · labels: docs
Done when: the page documents header and footer control records, record and component order, batch sizing, MV-dependent record versions and every loss case, with a hex example of a small snapshot.

## [To Do] META-F06 Feature levels
P0 · M2 · Feature · deps: META-F03
Scope: `metadata.version` and other finalized features; gating of record types and behaviors.

### [To Do] META-0094 `MetadataVersion`: levels 7 (3.3-IV3) to 33 (4.4-IV2) plus unstable 34 (4.5-IV0)
P0 · M2 · Task · deps: none · ref: server-common/src/main/java/org/apache/kafka/server/common/MetadataVersion.java @4.4.0
Done when: every Kafka 4.4.0 level has its feature level, release string and did-metadata-change flag; MINIMUM is 3.3-IV3 and LATEST_PRODUCTION is 4.4-IV2; `from_feature_level` and `from_version_string` ("4.4", "4.4-IV2") return typed errors; level 34 is accepted only when unstable feature versions are enabled; a table test checks every value against Kafka's enum.

### [To Do] META-0095 metadata.version capability predicates and record-version selectors
P0 · M2 · Task · deps: META-0094 · ref: server-common/src/main/java/org/apache/kafka/server/common/MetadataVersion.java @4.4.0
Done when: SCRAM, metadata transactions, delegation tokens, directory assignment, ELR, CIDR ACLs, migration, cordoned log dirs, controller registration and unregistration predicates, plus register-broker, register-controller, partition and partition-change record versions and the inter-broker Fetch and ListOffsets request versions, return Kafka's values for every level (table test generated from Kafka 4.4.0).

### [To Do] META-0096 Feature registry: kraft.version, transaction.version, group.version, eligible.leader.replicas.version, share.version, streams.version
P0 · M2 · Task · deps: META-0094 · ref: server-common/src/main/java/org/apache/kafka/server/common/Feature.java, server-common/src/main/java/org/apache/kafka/server/common/KRaftVersion.java, server-common/src/main/java/org/apache/kafka/server/common/TransactionVersion.java, server-common/src/main/java/org/apache/kafka/server/common/GroupVersion.java, server-common/src/main/java/org/apache/kafka/server/common/EligibleLeaderReplicasVersion.java, server-common/src/main/java/org/apache/kafka/server/common/ShareVersion.java, server-common/src/main/java/org/apache/kafka/server/common/StreamsVersion.java @4.4.0
Done when: each feature's levels, latest production level and per-level metadata.version requirement match Kafka 4.4.0; test-only features exist only in test builds; `supported_range()` per feature for this build; table tests.

### [To Do] META-0097 Feature dependency validation and release-version defaults
P0 · M2 · Task · deps: META-0096 · ref: server-common/src/main/java/org/apache/kafka/server/common/Feature.java validateVersion, server-common/src/main/java/org/apache/kafka/server/common/FeatureVersion.java @4.4.0
Done when: `validate(feature, level, finalized)` rejects levels whose dependencies are unmet with Kafka's error text; `default_level(feature, release_version)` reproduces `kafka-storage.sh version-mapping` and `feature-dependencies` output for every release from 3.3 to 4.4; tests.

### [To Do] META-0098 Record gating: which record types and versions may be written at a metadata.version
P0 · M2 · Task · deps: META-0095, META-0002
Done when: `RecordGate::check(record_type, version, mv)` returns `RecordNotAllowed { record_type, version, mv }` for records introduced after the MV (UserScramCredentialRecord, RegisterControllerRecord, Begin/End/AbortTransactionRecord, ClearElrRecord and others) and for versions above the MV's selector; the controller record writer (CTRL) and snapshot writer use it; table test over all 27 types.

### [To Do] META-0099 metadata.version change propagation through the delta
P0 · M2 · Task · deps: META-0094 · ref: metadata/src/main/java/org/apache/kafka/image/MetadataVersionChange.java, metadata/src/main/java/org/apache/kafka/image/MetadataDelta.java replay(FeatureLevelRecord) @4.4.0
Done when: an MV change in a delta calls every component's `handle_metadata_version_change`; `metadata_version_changed()` returns a `MetadataVersionChange { old, new, kind: upgrade | downgrade }` that META-F04 publishes and logs; downgrades across did-metadata-change levels are flagged lossy for CTRL-F17; tests for 3.3-IV3 to 4.4-IV2 and 4.4-IV2 to 4.3-IV0.

### [To Do] META-0100 Spike: Alluvium feature levels and the reserved metadata record id block
P1 · M2 · Spike · deps: none · labels: foundation-for:X-TENANT, foundation-for:X-ELASTIC · ref: docs/adr/0010-alluvium-is-its-own-platform.md
Done when: an accepted ADR fixes the `alluvium.<area>.version` naming, whether Alluvium features appear in ApiVersions responses to Kafka clients, the reserved record type id block for Alluvium records (proposed from 10000, with sub-ranges for namespaces, key ranges and each enhancement epic), the schema location `schemas/alluvium/metadata/`, and the off-by-default rule (level 0 means no Alluvium record is written and snapshots match a Kafka-only build).

### [To Do] META-0101 Alluvium feature registry entries and the Alluvium schema codegen path
P1 · M2 · Task · deps: META-0100, META-0096, META-0002 · labels: foundation-for:X-TENANT, foundation-for:X-ELASTIC
Done when: `alluvium.*` features are declared through the same registry with levels and dependencies; codegen reads `schemas/alluvium/metadata/` and fails on ids that collide with Kafka's or fall outside the reserved block; a test-build-only `alluvium.test.version` proves gating end to end.

### [To Do] META-0102 Supported feature ranges advertised by this node
P0 · M2 · Task · deps: META-0096, META-0101
Done when: `supported_features(roles)` returns min/max per feature (metadata.version 7-33, kraft.version 0-1 and the others) for BrokerRegistration, ControllerRegistration and the ApiVersions SupportedFeatures field, equal to Kafka 4.4.0's for the same settings; Alluvium features appear only when enabled (test).

### [To Do] META-0103 docs/metadata-versions.md generated from the metadata.version and feature registries
P2 · M2 · Task · deps: META-0095, META-0097, FND-F04 · labels: docs
Done when: `cargo xtask docs metadata-versions` writes the table of levels (release, what each gates, record versions) and of features with dependencies; CI fails when the page is stale.

## [To Do] META-F07 Namespace records
P1 · M2 · Feature · deps: META-F03, TYPE-F10 · labels: foundation-for:X-TENANT
Scope: Alluvium metadata records for namespaces, gated by an Alluvium feature level.

### [To Do] META-0104 Spike: namespace metadata foundation (record shapes, default namespace, topic binding)
P1 · M2 · Spike · deps: META-0100 · labels: foundation-for:X-TENANT
Done when: an accepted ADR defines NamespaceRecord and RemoveNamespaceRecord fields, how a topic binds to a namespace without changing Kafka's TopicRecord (a binding record keyed by topic id), the implicit default namespace from TYPE-F10 that is never stored, name rules, the gating feature level, and what X-TENANT-F01 may still change.

### [To Do] META-0105 Schemas and codegen for NamespaceRecord, RemoveNamespaceRecord and TopicNamespaceRecord
P1 · M2 · Task · deps: META-0104, META-0101, META-0008 · labels: foundation-for:X-TENANT
Done when: JSON schemas under `schemas/alluvium/metadata/` use ids from the namespace sub-range, version 0, flexible 0+; generated codecs round-trip; golden vectors and the `metadata_record_frame` fuzz target cover them.

### [To Do] META-0106 `NamespacesImage` and `NamespacesDelta`
P1 · M2 · Task · deps: META-0105 · labels: foundation-for:X-TENANT
Done when: namespaces by id and by name plus topic-to-namespace bindings; the default namespace is implicit; `apply` shares unchanged data; the component is wired into the aggregates and the tree view (`/namespaces`); unit tests.

### [To Do] META-0107 Replay namespace records with validation
P1 · M2 · Task · deps: META-0106 · labels: foundation-for:X-TENANT
Done when: duplicate namespace name or id, removal of an unknown namespace or of one that still has bound topics, and binding an unknown topic or namespace return typed `ReplayError`s; RemoveTopicRecord drops the topic's binding; one test per record and per error.

### [To Do] META-0108 Gate namespace records behind the namespace feature level
P1 · M2 · Task · deps: META-0107, META-0098 · labels: foundation-for:X-TENANT
Done when: with the level at 0, writing or replaying namespace records fails through `RecordGate`; a delta lowering the level while namespaces exist is rejected; tests.

### [To Do] META-0109 Snapshot round-trip for namespaces
P1 · M2 · Task · deps: META-0107, META-0081 · labels: foundation-for:X-TENANT
Done when: namespace records are written after Kafka's components only when the level is at least 1; round-trip test; with level 0 the snapshot bytes equal those of a build without namespace support.

### [To Do] META-0110 Namespace-qualified topic lookups
P1 · M2 · Task · deps: META-0106 · labels: foundation-for:X-TENANT
Done when: (namespace, topic name) resolves to a topic id and back; plain Kafka names resolve in the default namespace; META-F09 lookups are unchanged when the level is 0; property test over random bindings.

### [To Do] META-0111 Off-by-default proof for namespace metadata
P1 · M2 · Task · deps: META-0108, META-0109, TEST-F09 · labels: foundation-for:X-TENANT, test
Done when: CI runs the TEST-F08 compatibility and TEST-F09 differential suites with the namespace level at 0 and at 1 with no namespaces created; there are no new diffs and Metadata responses are byte-identical.

### [To Do] META-0112 Document namespace records in docs/formats/metadata-records.md
P2 · M2 · Task · deps: META-0105, META-0009 · labels: foundation-for:X-TENANT, docs
Done when: the Alluvium record section lists the namespace records with ids, fields, gating level and replay rules, and links the ADR and X-TENANT.

## [To Do] META-F08 Key-range mapping records
P1 · M3 · Feature · deps: META-F03, TYPE-F11 · labels: foundation-for:X-ELASTIC
Scope: Records mapping key ranges to partitions with lineage epochs.

### [To Do] META-0113 Spike: key-range mapping records (ranges, mapping epochs, reserved id sub-range)
P1 · M3 · Spike · deps: META-0100 · labels: foundation-for:X-ELASTIC
Done when: an accepted ADR defines KeyRangeMappingRecord (topic id, mapping epoch, ranges over the TYPE-F11 key-hash space mapped to partitions), its invariants (full coverage, no overlap, epochs strictly increasing per topic), the id sub-range reserved for key-range and later X-ELASTIC records, the gating feature `alluvium.elastic.version` level 1, and the implicit Kafka-compatible default mapping.

### [To Do] META-0114 Schema and codegen for KeyRangeMappingRecord
P1 · M3 · Task · deps: META-0113, META-0101, META-0008 · labels: foundation-for:X-ELASTIC
Done when: the schema lives under `schemas/alluvium/metadata/` with an id from the key-range sub-range; the generated codec round-trips; golden vectors and the `metadata_record_frame` fuzz target cover it.

### [To Do] META-0115 `KeyRangesImage` and its delta
P1 · M3 · Task · deps: META-0114 · labels: foundation-for:X-ELASTIC
Done when: per topic the image keeps the current mapping epoch and the ordered ranges; `apply` shares unchanged topics; the component is wired into the aggregates and the tree view (`/keyRanges/<topic>`); unit tests.

### [To Do] META-0116 Replay KeyRangeMappingRecord with validation
P1 · M3 · Task · deps: META-0115 · labels: foundation-for:X-ELASTIC
Done when: gaps, overlaps, empty ranges, partitions missing from the topic and non-increasing mapping epochs return typed `ReplayError`s carrying the offending ranges; RemoveTopicRecord drops the mapping; tests per error.

### [To Do] META-0117 Implicit default mapping for topics without key-range records
P1 · M3 · Task · deps: META-0115, TYPE-F11 · labels: foundation-for:X-ELASTIC · ref: clients/src/main/java/org/apache/kafka/clients/producer/internals/BuiltInPartitioner.java @4.4.0
Done when: lookups for a topic with no mapping record use Kafka's murmur2 partitioner (hash & 0x7fffffff modulo partition count) and nothing is stored; a property test shows lookups equal the Java client's DefaultPartitioner logic for random keys and partition counts.

### [To Do] META-0118 Gate key-range records behind `alluvium.elastic.version` level 1
P1 · M3 · Task · deps: META-0116, META-0098 · labels: foundation-for:X-ELASTIC
Done when: the feature is registered with level 1 meaning mapping records only (X-ELASTIC-F02 extends it later); with level 0 `RecordGate` rejects writing and replaying the record; lowering the level while mappings exist is rejected; tests.

### [To Do] META-0119 Snapshot round-trip for key-range mappings
P1 · M3 · Task · deps: META-0116, META-0081 · labels: foundation-for:X-ELASTIC
Done when: mapping records are written only when the level is at least 1; round-trip test; with level 0 snapshots are byte-identical to a build without the records.

### [To Do] META-0120 Property test: random split and merge sequences keep mapping invariants
P1 · M3 · Task · deps: META-0116, TEST-F16 · labels: foundation-for:X-ELASTIC, test
Done when: generated sequences of range splits and merges (TYPE-F11 arithmetic) produce records that always replay without error, always cover the hash space exactly once, and strictly increase the epoch; invalid mutations are always rejected.

### [To Do] META-0121 Off-by-default proof for key-range metadata
P1 · M3 · Task · deps: META-0118, META-0119, TEST-F09 · labels: foundation-for:X-ELASTIC, test
Done when: CI runs the TEST-F08 and TEST-F09 suites with the level at 0 and at 1 with no mappings; there are no new diffs and produce partitioning is unchanged.

### [To Do] META-0122 Document key-range records in docs/formats/metadata-records.md
P2 · M3 · Task · deps: META-0114, META-0009 · labels: foundation-for:X-ELASTIC, docs
Done when: the Alluvium record section lists KeyRangeMappingRecord with fields, invariants, gating level and the default mapping rule, linking the ADR and X-ELASTIC.

## [To Do] META-F09 Broker metadata lookups
P0 · M2 · Feature · deps: META-F03
Scope: Indexed lookups for Metadata and DescribeTopicPartitions: topics by name and id, leaders, alive brokers, endpoints per listener.

### [To Do] META-0123 `MetadataView`: a per-request handle over one image version
P0 · M2 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/metadata/KRaftMetadataCache.java @4.4.0
Done when: handlers obtain one `Arc<MetadataImage>`-backed view per request and run every lookup on it; a test publishes a new image mid-request and shows the request still sees one version (architecture invariant).

### [To Do] META-0124 Topic lookups: by name and id, partition count, partition info and internal topics
P0 · M2 · Task · deps: META-0123 · ref: metadata/src/main/java/org/apache/kafka/metadata/KRaftMetadataCache.java @4.4.0
Done when: `topic_id(name)`, `topic_name(id)`, `contains(topic)`, `contains(tp)`, `num_partitions`, `partition(tp)`, `topic_names_to_ids` and `all_topics` behave like KRaftMetadataCache; `__consumer_offsets`, `__transaction_state` and `__share_group_state` are flagged internal; unit tests.

### [To Do] META-0125 Broker lookups: alive brokers and endpoints per listener, fencing, shutdown and racks
P0 · M2 · Task · deps: META-0123 · ref: metadata/src/main/java/org/apache/kafka/metadata/KRaftMetadataCache.java @4.4.0
Done when: alive means registered and not fenced; `alive_broker_nodes(listener)` sorted by id, `alive_broker_node(id, listener)`, `broker_nodes(listener)`, `is_broker_fenced`, `is_broker_shutting_down`, `alive_broker_epoch` and racks; `random_alive_broker_id` uses an injected seeded RNG; unit tests.

### [To Do] META-0126 Topic metadata for the Metadata API: endpoints, ISR, offline replicas and per-partition errors
P0 · M2 · Task · deps: META-0124, META-0125 · ref: metadata/src/main/java/org/apache/kafka/metadata/KRaftMetadataCache.java getTopicMetadata @4.4.0
Done when: results follow KRaftMetadataCache.getTopicMetadata: LEADER_NOT_AVAILABLE without a leader, LISTENER_NOT_FOUND when the leader lacks the listener, REPLICA_NOT_AVAILABLE when unavailable endpoints are reported as errors, offline replicas from directory state, and leader epochs; ported KRaftMetadataCacheTest cases pass.

### [To Do] META-0127 DescribeTopicPartitions paging: cursor and response partition limit
P0 · M2 · Task · deps: META-0126 · ref: metadata/src/main/java/org/apache/kafka/metadata/KRaftMetadataCache.java describeTopicResponse @4.4.0
Done when: topics are sorted by name, iteration starts at the cursor's topic and partition, stops at the response partition limit and returns `next_cursor`; unknown topics get UNKNOWN_TOPIC_OR_PARTITION; ported describeTopicResponse test cases pass.

### [To Do] META-0128 Partition leader and replica endpoint lookups for current-leader hints (KIP-951)
P0 · M2 · Task · deps: META-0124, META-0125 · ref: metadata/src/main/java/org/apache/kafka/metadata/KRaftMetadataCache.java getPartitionLeaderEndpoint, KIP-951 @4.4.0
Done when: `leader_and_isr(tp)`, `partition_leader_endpoint(tp, listener)` and `partition_replica_endpoints(tp, listener)` return nodes or none as in KRaftMetadataCache; Produce and Fetch use them for NOT_LEADER_OR_FOLLOWER hints; unit tests.

### [To Do] META-0129 Cluster-level lookups: cluster id, controller id for clients, finalized features
P0 · M2 · Task · deps: META-0125 · ref: core/src/main/scala/kafka/server/KafkaApis.scala handleTopicMetadataRequest @4.4.0
Done when: the controller id returned to clients is a random alive broker id or -1, as KafkaApis does in KRaft; finalized features with their epoch feed ApiVersions; controller endpoints come from ControllerRegistration when present (DescribeCluster endpoint type 2); unit tests.

### [To Do] META-0130 Derived indexes built on image apply: per-listener endpoints and partitions per broker
P1 · M2 · Task · deps: META-0126, TEST-F11 · labels: perf
Done when: indexes are rebuilt only for changed brokers and topics; lookups are O(log n); a benchmark of a Metadata response for 10k topics with 10 partitions each meets the budget recorded with NFR-F01.

### [To Do] META-0131 Property test: lookups equal a brute-force scan of the image
P0 · M2 · Task · deps: META-0126, META-0127, META-0128, TEST-F16 · labels: test
Done when: for random images, listeners and requests, every lookup in this feature returns the same result as a straightforward scan of the image components.
