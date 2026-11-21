# X-GEO: Geo-replication

> Crates: `alluvium-link` (new, layer 4) · Kafka counterpart: none (Alluvium enhancement) · Milestones: M15

Built-in cross-cluster replication with identical offsets, offset sync, failover and failback, active-active patterns and remote observer replicas.

## [To Do] X-GEO-F01 Geo-replication design
P0 · M15 · Feature · deps: REPL-F06
Scope: Spike and ADR: cluster links and offset-preserving mirroring.

### [To Do] X-GEO-0001 Spike: geo-replication architecture ADR (cluster links, offset-preserving mirror topics, failover model)
P0 · M15 · Spike · deps: none · ref: new ADR "Cluster links and offset-preserving mirroring"
Done when: ADR accepted that decides: destination-pull links using the Kafka Fetch protocol (so Kafka 4.x sources work) vs source push; byte-for-byte batch copy with identical offsets; mirror topic and mirror partition state machines; the read-only error code for mirrors; offset sync without translation; failover, failback and divergence handling; active-active naming and loop prevention; placement of sans-I/O link logic (layer 2) and the driver (layer 4); comparison with MirrorMaker 2 (KIP-382) and its offset translation; rejected alternatives recorded.

### [To Do] X-GEO-0002 Mirroring invariants specification for simulation checkers
P0 · M15 · Task · deps: X-GEO-0001 · labels: docs, test
Done when: docs/geo/invariants.md states the checkable invariants (mirror record at offset o equals the source record at o; mirror LEO never exceeds the source HW observed at fetch time; a non-promoted mirror never accepts client writes; promotion offsets are immutable; no record is mirrored back to its origin cluster) and each is implemented as a TEST-F05 invariant checker with a unit test that triggers it.

### [To Do] X-GEO-0003 Geo-replication threat model
P0 · M15 · Task · deps: X-GEO-0001, SEC-F14 · labels: security, docs
Done when: docs/security/threat-model-geo.md covers stolen link credentials, MITM on WAN links, a malicious or compromised source cluster sending crafted batches, over-privileged link principals, data residency violations through auto-mirroring and failover misuse; every threat maps to mitigations and X-GEO test task IDs.

### [To Do] X-GEO-0004 `alluvium.link.version` finalized feature gating link and mirror metadata records
P0 · M15 · Task · deps: X-GEO-0001, META-F06, CTRL-F17
Done when: at level 0 (default) the controller rejects every X-GEO record type with UNSUPPORTED_VERSION; level 1 enables them; downgrade to 0 is refused while links or mirror topics exist; unit tests per record type.

### [To Do] X-GEO-0005 Register the `alluvium.link.*` and `alluvium.observer.*` config namespaces
P0 · M15 · Task · deps: X-GEO-0001, CFG-F11
Done when: both prefixes are in the CFG-F11 registry linked to the geo ADR; the registry check rejects unregistered keys under them; docs/config.md lists the namespaces.

## [To Do] X-GEO-F02 Cluster links
P0 · M15 · Feature · deps: X-GEO-F01, NET-F10
Scope: Connections to remote Kafka or Alluvium clusters with credentials and config.

### [To Do] X-GEO-0006 Cluster link data model and metadata records
P0 · M15 · Task · deps: X-GEO-0004, META-F02
Done when: `ClusterLinkRecord` (link id, name, remote bootstrap servers, expected remote cluster id, security config reference, direction, state) and `RemoveClusterLinkRecord` are validated by the controller (unique name, valid bootstrap list, max links `alluvium.link.max.links`), replayed into the image and survive snapshots; unit tests cover each validation error.

### [To Do] X-GEO-0007 Link credentials stored encrypted and never returned
P0 · M15 · Task · deps: X-GEO-0006, SEC-F11
Done when: link security settings (`security.protocol`, SASL mechanism, JAAS-equivalent credentials, TLS key store) are stored as password-type values encrypted with `password.encoder.secret` (or the X-SECX KMS seam when enabled); describe responses show them as redacted; credential update rotates without recreating the link; a redaction test covers logs, events and describe output.

### [To Do] X-GEO-0008 Link state machine: Pending → Connecting → Authenticating → Active ↔ Degraded, Paused, Failed
P0 · M15 · Task · deps: X-GEO-0006
Done when: the sans-I/O machine returns transitions with reasons (`remote_unreachable`, `authentication_failed`, `cluster_id_mismatch`, `paused_by_admin`, `metadata_refresh_failed`, `recovered`); every transition is recorded in the link's flight recorder; a property test over random input sequences shows Failed is left only by admin action.

### [To Do] X-GEO-0009 Outbound link connections over the inter-node client with version negotiation
P0 · M15 · Task · deps: X-GEO-0008, NET-F10, PROTO-F14
Done when: the link driver resolves bootstrap servers, connects with timeouts and backoff, and negotiates API versions with the remote; remotes older than the minimum in the ADR fail with reason `unsupported_remote_version`; connection count per link is bounded by `alluvium.link.max.connections.per.broker`; tests run against Kafka 4.4.0 and Alluvium remotes.

### [To Do] X-GEO-0010 Link TLS: rustls client config, hostname verification, client certificates
P0 · M15 · Task · deps: X-GEO-0009, NET-F07 · labels: security
Done when: links with `security.protocol=SSL` or `SASL_SSL` verify the remote certificate chain and hostname, present client certificates when configured, and refuse plaintext to non-loopback remotes unless `alluvium.link.allow.plaintext=true`; tests cover hostname mismatch, untrusted CA and expired certificates.

### [To Do] X-GEO-0011 Link SASL client authentication: PLAIN and SCRAM-SHA-256/512
P0 · M15 · Task · deps: X-GEO-0009, SEC-F16 · labels: security
Done when: links authenticate to remotes with PLAIN and SCRAM using the client-side mechanisms from SEC-F16; KIP-368 re-authentication is honored when the remote requires it; failures move the link to Failed with reason `authentication_failed` and the remote error message; compat tests run against Kafka 4.4.0 with each mechanism.

### [To Do] X-GEO-0012 Link SASL/OAUTHBEARER client: client-credentials token retrieval and refresh
P1 · M15 · Task · deps: X-GEO-0011 · labels: security
Done when: the link obtains tokens from the configured token endpoint (client id and secret as `Secret`), refreshes them before expiry, re-authenticates the session, and bounds token response size; tests use a fake token endpoint with expiry and failure cases.

### [To Do] X-GEO-0013 Remote cluster identity pinning via DescribeCluster
P0 · M15 · Task · deps: X-GEO-0009
Done when: after authentication the link calls DescribeCluster and compares the remote cluster id with the configured one; a mismatch moves the link to Failed with reason `cluster_id_mismatch` and no data is fetched; the id is re-checked on every reconnect; tests cover mismatch after a remote is re-pointed.

### [To Do] X-GEO-0014 Remote metadata discovery and bounded remote metadata cache
P0 · M15 · Task · deps: X-GEO-0013
Done when: the link refreshes remote topic metadata (topic ids, partition counts, leaders, leader epochs) via Metadata or DescribeTopicPartitions every `alluvium.link.metadata.refresh.ms` and on leader errors; the cache is bounded by mirrored topics only; tests cover leader moves and NOT_LEADER_OR_FOLLOWER responses.

### [To Do] X-GEO-0015 Mirror fetcher placement: destination partition leaders run the fetchers
P0 · M15 · Task · deps: X-GEO-0014, REPL-F12
Done when: each mirror partition's destination leader owns its link fetcher; leadership changes start and stop fetchers through the replica manager's delta handling; a simulation test moves leadership repeatedly and asserts exactly one active fetcher per mirror partition.

### [To Do] X-GEO-0016 Link administration: native admin API and `alluvium link` CLI
P0 · M15 · Task · deps: X-GEO-0006, X-GEO-0008, X-NATIVE-F08, CLI-F01
Done when: create (with dry-run validation that connects and checks the cluster id), describe, list, alter (bootstrap, credentials, settings), pause, resume and delete work through native endpoints and `alluvium link create|describe|list|alter|pause|resume|delete`; delete is refused while mirror topics use the link unless `--force`; tests per operation.

### [To Do] X-GEO-0017 Authorization for links: new `ClusterLink` resource type with ACL fallback
P0 · M15 · Task · deps: X-GEO-0016, SEC-F08 · labels: security
Done when: CREATE, ALTER, DESCRIBE and DELETE on `ClusterLink` resources are enforced; without policies, cluster ALTER/DESCRIBE ACLs are required as documented; tests cover allowed and denied principals.

### [To Do] X-GEO-0018 Source-initiated links for destinations behind firewalls
P2 · M15 · Task · deps: X-GEO-0009, X-GEO-0010
Done when: with `link.mode=source-initiated` the source cluster opens an authenticated connection to the destination, which then sends Fetch requests over it; the connection is supervised and reconnected with backoff; a compat test mirrors across a one-way firewall rule.

### [To Do] X-GEO-0019 Link configuration keys and validation
P0 · M15 · Task · deps: X-GEO-0006, X-GEO-0005
Done when: per-link settings (`link.mode`, `metadata.refresh.ms`, `fetch.max.bytes`, `fetch.max.wait.ms`, `connections.max.idle.ms`, security settings) and broker defaults under `alluvium.link.*` are defined with types, defaults, validators and docs; invalid values fail link creation with INVALID_CONFIG naming the key.

### [To Do] X-GEO-0020 Link metrics, events and diagnostics
P0 · M15 · Task · deps: X-GEO-0008, OBS-F04, OBS-F07, OBS-F08
Done when: `alluvium_link_state{link}`, `alluvium_link_connection_failures_total{link,reason}` and `alluvium_link_metadata_refresh_seconds{link}` exist; events `link_state_changed` carry old and new state and reason; `/debug/links` shows state and the flight recorder; tests assert metric and event output.

### [To Do] X-GEO-0021 Negative tests: wrong credentials, wrong cluster, TLS failures, malformed remote responses
P0 · M15 · Task · deps: X-GEO-0011, X-GEO-0010, X-GEO-0013 · labels: security, test
Done when: tests cover wrong password, revoked remote ACLs, cluster id mismatch, TLS hostname mismatch and a fake remote returning inconsistent metadata (unknown leaders, negative partition counts, oversized responses); each ends in the expected link state with no panic and bounded memory.

## [To Do] X-GEO-F03 Mirror topics
P0 · M15 · Feature · deps: X-GEO-F02
Scope: Byte-for-byte batch copy with identical offsets; read-only mirrors.

### [To Do] X-GEO-0022 Mirror topic data model and metadata records
P0 · M15 · Task · deps: X-GEO-0006
Done when: `MirrorTopicRecord` (topic id, link id, source topic name, source topic id, state, promotion info) is validated (link exists, source topic exists, partition count equals the source), replayed and snapshotted; mirror topics are ordinary topics with an attached mirror record; unit tests cover validation errors.

### [To Do] X-GEO-0023 Mirror topic and mirror partition state machines
P0 · M15 · Task · deps: X-GEO-0022
Done when: topic states Pending → Mirroring ↔ Paused → Promoting → Promoted, Failed (reasons `source_deleted`, `source_recreated`, `diverged`, `link_failed`) and partition states Fetching, CaughtUp, Lagging, Paused, Stopped, Failed are sans-I/O machines whose transitions carry reasons and go to the partition flight recorder; property tests check legal transitions only.

### [To Do] X-GEO-0024 Mirror topic creation: validation, partition count and required configs
P0 · M15 · Task · deps: X-GEO-0023, X-GEO-0014
Done when: creating a mirror reads the source topic's partition count and configs, creates the destination topic with the same partition count, `compression.type=producer`, `message.timestamp.type` equal to the source and `max.message.bytes` at least the source value; creation fails clearly when the source is missing or the link is not Active; tests per validation.

### [To Do] X-GEO-0025 Read-only enforcement for mirror topics
P0 · M15 · Task · deps: X-GEO-0023, REPL-F03
Done when: Produce, transactional produce, DeleteRecords, CreatePartitions and alteration of mirrored configs on a non-promoted mirror fail with the error code chosen in the ADR for every API version; consumers and groups read normally; tests per API and version.

### [To Do] X-GEO-0026 Mirror fetcher: Fetch from the source leader with sessions, backoff and bounded buffers
P0 · M15 · Task · deps: X-GEO-0023, X-GEO-0015
Done when: the destination leader fetches from the source partition leader at offset = mirror LEO with read_uncommitted isolation, using KIP-227 fetch sessions, `fetch.max.bytes`, `fetch.max.wait.ms` and backoff on errors; in-flight bytes per link are bounded by `alluvium.link.buffer.memory.bytes`; handling of NOT_LEADER, FENCED_LEADER_EPOCH, OFFSET_OUT_OF_RANGE and UNKNOWN_TOPIC_ID is tested.

### [To Do] X-GEO-0027 Byte-for-byte mirror append with contiguity and integrity validation
P0 · M15 · Task · deps: X-GEO-0026, STOR-F08, REC-F12 · labels: security
Done when: fetched batches are appended unmodified in the CRC-covered region with their source base offsets; each batch is validated (CRC32C, size limits, magic v2, base offset ≥ LEO, last offset ≥ base offset) before append; records are never recompressed; a test compares mirror segment batches byte-for-byte with the source in the CRC-covered region.

### [To Do] X-GEO-0028 Offset gaps from compacted or deleted source ranges
P0 · M15 · Task · deps: X-GEO-0027
Done when: a batch whose base offset is greater than the mirror LEO advances the LEO across the gap; mirror log start offset follows the source log start offset (from fetch responses) via a local log-start-offset increment with reason `mirror_source_log_start`; tests cover compacted sources and source DeleteRecords.

### [To Do] X-GEO-0029 Leader epoch stamping and the source epoch cache
P0 · M15 · Task · deps: X-GEO-0027, STOR-F11
Done when: `partitionLeaderEpoch` (outside the CRC) is rewritten to the destination leader epoch on append so destination truncation and OffsetsForLeaderEpoch work; the source epoch of each offset range is recorded in a separate bounded mirror epoch cache persisted with checkpoint files; recovery and truncation tests cover both caches.

### [To Do] X-GEO-0030 Source divergence detection and mirror truncation policy
P0 · M15 · Task · deps: X-GEO-0029
Done when: after a source leader change the fetcher validates its position with OffsetsForLeaderEpoch using the source epoch cache; on divergence (unclean election on the source) the mirror truncates to the divergence point and emits `mirror_truncated` with offsets when `alluvium.link.mirror.divergence.policy=truncate`, or moves to Failed(`diverged`) when `fail`; a promoted mirror is never truncated; simulation tests cover both policies.

### [To Do] X-GEO-0031 Producer state on mirror append
P0 · M15 · Task · deps: X-GEO-0027, STOR-F15
Done when: mirrored batches update producer state (producer id, epoch, last sequences) without enforcing sequence validation against local producers; producer-state snapshots include mirrored producers; after promotion a mirrored idempotent producer continuing on the destination gets the same duplicate detection it would on the source; tests cover duplicates across promotion.

### [To Do] X-GEO-0032 Timestamp preservation and time index on mirrors
P0 · M15 · Task · deps: X-GEO-0027, STOR-F05
Done when: CreateTime and LogAppendTime batches keep their source timestamps (no restamping); the time index is built from them; ListOffsets by timestamp on the mirror returns the same offsets as on the source; tests cover both timestamp types.

### [To Do] X-GEO-0033 Mirror partitions replicate inside the destination cluster
P0 · M15 · Task · deps: X-GEO-0027, REPL-F04
Done when: destination followers replicate mirror partitions with normal ISR and HW rules; consumers on the destination read up to the destination HW; a simulation asserts destination HW ≤ source HW observed and no acknowledged mirror data is lost under destination broker crashes.

### [To Do] X-GEO-0034 Source topic deletion and recreation handling
P0 · M15 · Task · deps: X-GEO-0023, X-GEO-0014
Done when: a deleted source topic moves the mirror to Failed(`source_deleted`) while staying readable; a source topic recreated with the same name but a new topic id moves it to Failed(`source_recreated`) without fetching from the new topic; events carry both topic ids; tests cover both.

### [To Do] X-GEO-0035 Backpressure and limits for mirror fetchers
P0 · M15 · Task · deps: X-GEO-0026, STOR-F25
Done when: fetchers pause when the destination log directory hits disk-full protection, when the per-link buffer is full, or when the destination partition is offline, and resume automatically; in-flight fetches per link are bounded by `alluvium.link.max.in.flight.fetches`; metrics expose paused partitions by reason.

### [To Do] X-GEO-0036 Fetch from source followers in the nearest rack
P2 · M15 · Task · deps: X-GEO-0026, REPL-F11
Done when: links set `client.rack` from `alluvium.link.client.rack`, so KIP-392 sources can redirect mirror fetches to a closer follower; the fetcher follows `preferred_read_replica`; a compat test against Kafka 4.4.0 with rack-aware selection shows fetches served by the follower.

### [To Do] X-GEO-0037 Fuzz mirror append validation with batches from a malicious source
P0 · M15 · Task · deps: X-GEO-0027, TEST-F07 · labels: security, test, decoder
Done when: `fuzz_mirror_append` feeds arbitrary fetch-response record sets through validation and append on the in-memory filesystem; it runs clean with no panic, bounded allocation and no corrupt segment (recovery succeeds afterwards).

### [To Do] X-GEO-0038 Compatibility: Kafka 4.4.0 source mirrored to Alluvium with identical offsets
P0 · M15 · Task · deps: X-GEO-0027, X-GEO-0028, TEST-F08 · labels: test
Done when: the compat harness mirrors topics from Kafka 4.4.0 (plain, compressed with each codec, compacted, with headers, idempotent producer data) to Alluvium; the Java consumer reads identical (offset, key, value, headers, timestamp) sequences from both clusters.

### [To Do] X-GEO-0039 Compatibility: Alluvium source to Alluvium destination
P0 · M15 · Task · deps: X-GEO-0027, TEST-F08 · labels: test
Done when: the same scenarios as the Kafka-source test pass with an Alluvium source, including encrypted-at-rest source topics when X-SECX is enabled on the source.

### [To Do] X-GEO-0040 Simulation: mirroring under WAN faults and leader changes on both sides
P0 · M15 · Task · deps: X-GEO-0030, X-GEO-0033, TEST-F06 · labels: test
Done when: a seeded two-cluster simulation with delays, drops, partitions and leader changes on both clusters checks the geo invariants at every step for thousands of seeds in nightly runs; failures print the seed and both clusters' flight recorders.

### [To Do] X-GEO-0041 Mirror topic metrics and events
P0 · M15 · Task · deps: X-GEO-0023, OBS-F04, OBS-F07
Done when: `alluvium_link_mirror_bytes_total{link,topic}`, `_records_total`, `_fetch_errors_total{link,reason}` and `alluvium_link_mirror_partitions{state}` exist with bounded labels; events `mirror_state_changed` and `mirror_truncated` are in the catalog with reasons.

### [To Do] X-GEO-0042 Mirror throughput and CPU benchmarks
P0 · M15 · Task · deps: X-GEO-0027, TEST-F11 · labels: perf
Done when: macro benchmarks report MB/s per link and CPU per GB mirrored at 0, 50 and 150 ms RTT with fetch session tuning; criterion benchmarks cover batch validation; results and budgets are in docs/benchmarks/geo.md.

## [To Do] X-GEO-F04 Consumer offset sync
P0 · M15 · Feature · deps: X-GEO-F03, GRP-F07
Scope: Group offsets mirrored to the destination.

### [To Do] X-GEO-0043 Group selection for offset sync
P0 · M15 · Task · deps: X-GEO-0005 · ref: connect/mirror/src/main/java/org/apache/kafka/connect/mirror/DefaultGroupFilter.java @4.4.0
Done when: per-link `consumer.offset.sync.groups.include` and `.exclude` regex lists select groups; only offsets for mirrored topics are synced; unit tests cover include/exclude precedence and invalid regexes rejected at config time.

### [To Do] X-GEO-0044 Reading committed offsets from the source
P0 · M15 · Task · deps: X-GEO-0043, X-GEO-0014
Done when: every `consumer.offset.sync.ms` the link lists groups and fetches committed offsets with batched OffsetFetch and `require_stable=true`, bounded by `consumer.offset.sync.max.groups`; FindCoordinator lookups are batched; tests cover coordinator moves and COORDINATOR_LOAD_IN_PROGRESS.

### [To Do] X-GEO-0045 Applying offsets on the destination without translation, clamped to mirrored data
P0 · M15 · Task · deps: X-GEO-0044
Done when: offsets are written to destination `__consumer_offsets` with the same value, metadata and leader epoch as the source, but clamped to the mirror partition's HW when the source offset is ahead; clamped commits are counted; tests cover equal, behind and ahead cases.

### [To Do] X-GEO-0046 Protection of groups active on the destination
P0 · M15 · Task · deps: X-GEO-0045 · ref: connect/mirror/src/main/java/org/apache/kafka/connect/mirror/MirrorCheckpointTask.java refreshIdleConsumerGroupOffset @4.4.0
Done when: offsets are never written for a destination group that has members; a synced offset never moves a destination commit backwards unless `consumer.offset.sync.allow.rewind=true`; tests cover active groups, rewinds and groups that become active mid-sync.

### [To Do] X-GEO-0047 Group type creation on the destination for classic and consumer groups
P0 · M15 · Task · deps: X-GEO-0045, GRP-F05
Done when: a group missing on the destination is created empty with the source group type (classic or KIP-848 consumer); share and streams groups are excluded and documented; tests cover both types and a later join by real consumers.

### [To Do] X-GEO-0048 Propagation of source group and offset deletions
P2 · M15 · Task · deps: X-GEO-0045
Done when: with `consumer.offset.sync.propagate.deletes=true`, source DeleteGroups and OffsetDelete are applied to inactive destination groups; otherwise they are ignored and counted; tests cover both settings.

### [To Do] X-GEO-0049 Offset sync visible to Kafka tools
P0 · M15 · Task · deps: X-GEO-0045, TEST-F08 · labels: test
Done when: `kafka-consumer-groups.sh --describe` on the destination shows synced offsets equal to the source's; a Java consumer switching clusters resumes at the synced position.

### [To Do] X-GEO-0050 Simulation: consumer failover with synced offsets
P0 · M15 · Task · deps: X-GEO-0045, TEST-F06 · labels: test
Done when: a seeded two-cluster simulation fails consumers over at random times and asserts no record is skipped and duplicates are bounded by records produced within one sync interval plus mirror lag.

### [To Do] X-GEO-0051 MirrorMaker 2-compatible checkpoint emission
P3 · M15 · Task · deps: X-GEO-0045 · ref: connect/mirror/src/main/java/org/apache/kafka/connect/mirror/Checkpoint.java @4.4.0
Done when: with `consumer.offset.sync.emit.mm2.checkpoints=true` the destination writes `<alias>.checkpoints.internal` records in MM2's Checkpoint format so `RemoteClusterUtils.translateOffsets` returns correct offsets; a compat test calls it from the Java client.

### [To Do] X-GEO-0052 Offset sync metrics and events
P0 · M15 · Task · deps: X-GEO-0045, OBS-F04
Done when: `alluvium_link_offset_sync_age_seconds{link}`, `_groups_synced{link}`, `_commits_clamped_total{link}` and `_skipped_active_groups_total{link}` exist; event `offset_sync_failed` carries the reason.

## [To Do] X-GEO-F05 Metadata sync
P1 · M15 · Feature · deps: X-GEO-F03
Scope: Topic configs, ACLs and partitions kept in sync.

### [To Do] X-GEO-0053 Topic config sync with exclusion rules
P1 · M15 · Task · deps: X-GEO-0014 · ref: connect/mirror/src/main/java/org/apache/kafka/connect/mirror/DefaultConfigPropertyFilter.java @4.4.0
Done when: every `config.sync.ms` mirrored topic configs are read with DescribeConfigs and applied to the mirror, excluding MM2's default exclusions (throttled replicas, `min.insync.replicas`, `unclean.leader.election.enable`, timestamp difference keys) plus `config.sync.exclude`; a property test shows excluded keys are never written.

### [To Do] X-GEO-0054 Local config overrides on mirrors are preserved
P1 · M15 · Task · deps: X-GEO-0053
Done when: keys listed in the mirror's `alluvium.link.config.local.overrides` are never overwritten by sync; changing a synced key locally is rejected unless it is added to the overrides; tests cover both.

### [To Do] X-GEO-0055 Partition count sync
P1 · M15 · Task · deps: X-GEO-0014, CTRL-F06
Done when: a source partition increase triggers CreatePartitions on the mirror with the same count within one metadata refresh; mirror partitions never outnumber source partitions; fetchers start for the new partitions from offset 0; tests cover increases during active mirroring.

### [To Do] X-GEO-0056 ACL sync for mirrored topics with privilege downgrade
P2 · M15 · Task · deps: X-GEO-0014, SEC-F09 · labels: security · ref: connect/mirror/src/main/java/org/apache/kafka/connect/mirror/MirrorSourceConnector.java downgradeAllowAllACL @4.4.0
Done when: with `acl.sync.enabled=true`, source topic ACLs are copied to the mirror with ALL downgraded to READ and WRITE grants dropped (mirrors are read-only); principals are mapped by `acl.sync.principal.mapping`; tests cover downgrade and mapping.

### [To Do] X-GEO-0057 Automatic mirror creation from topic filters
P1 · M15 · Task · deps: X-GEO-0053, X-GEO-0024 · ref: connect/mirror/src/main/java/org/apache/kafka/connect/mirror/DefaultTopicFilter.java @4.4.0
Done when: `auto.mirror.topics.include` and `.exclude` regexes create mirror topics for new matching source topics within one refresh; internal topics (`__*`) are never matched; creation respects destination topic limits; tests cover new, excluded and internal topics.

### [To Do] X-GEO-0058 Configurable action on source topic deletion
P1 · M15 · Task · deps: X-GEO-0034
Done when: `source.topic.deleted.action` (`keep`, default, or `delete`) is applied when a source topic disappears; `delete` requires the link principal to have DELETE on the mirror; events record the action; tests cover both.

### [To Do] X-GEO-0059 Metadata sync metrics and events
P1 · M15 · Task · deps: X-GEO-0053, X-GEO-0055, OBS-F04
Done when: `alluvium_link_config_sync_changes_total{link}`, `_partition_sync_total{link}` and `_metadata_sync_errors_total{link,reason}` exist; events `mirror_config_synced` and `mirror_partitions_added` list changed keys (values redacted for sensitive keys).

### [To Do] X-GEO-0060 Compatibility: config and partition changes on a Kafka source propagate
P1 · M15 · Task · deps: X-GEO-0053, X-GEO-0055, TEST-F08 · labels: test
Done when: `kafka-configs.sh` and `kafka-topics.sh --alter --partitions` on a Kafka 4.4.0 source are reflected on the Alluvium mirror within the sync interval, verified with the same tools on the destination.

## [To Do] X-GEO-F06 Failover and failback
P0 · M15 · Feature · deps: X-GEO-F03
Scope: Promote mirrors to writable; reverse direction.

### [To Do] X-GEO-0061 Promotion state machine: Mirroring → Promoting → Promoted, resumable
P0 · M15 · Task · deps: X-GEO-0023
Done when: promotion stops fetchers, drains in-flight appends, records per-partition promotion offsets and completes when every partition is promoted; it resumes after controller or leader failover from records; transitions carry reasons `planned_switchover` or `forced_failover`; a property test shows each partition is promoted exactly once.

### [To Do] X-GEO-0062 `MirrorPromotionRecord` with immutable promotion offsets
P0 · M15 · Task · deps: X-GEO-0061
Done when: the record holds topic id, partition, promotion offset (mirror LEO at stop), last source epoch and time; the controller rejects any later change; replay and snapshot round-trip tests pass.

### [To Do] X-GEO-0063 Planned switchover safety checks
P0 · M15 · Task · deps: X-GEO-0061, X-GEO-F04
Done when: planned promotion is refused unless every partition's mirror LEO equals the last observed source HW, the source topic is not receiving writes (no LEO change for `switchover.quiesce.ms`) and the last offset sync is newer than the last source commit; the error lists failing partitions and groups; tests cover each failing check.

### [To Do] X-GEO-0064 Forced failover with a data-loss estimate
P0 · M15 · Task · deps: X-GEO-0061
Done when: `--force` promotes immediately regardless of lag or link state; the response and event `mirror_promoted` report, per partition, the promotion offset and the last known source HW so the possible loss is visible; tests cover a down link.

### [To Do] X-GEO-0065 Promote API and `alluvium link promote` CLI with authorization
P0 · M15 · Task · deps: X-GEO-0063, X-GEO-0064, X-GEO-0016
Done when: native admin and CLI promote one topic, a list or all mirrors of a link, in planned or forced mode; ALTER on the link and the topic is required; tests cover authorization and partial failures.

### [To Do] X-GEO-0066 Writes after promotion
P0 · M15 · Task · deps: X-GEO-0062, X-GEO-0025
Done when: after a partition is Promoted, produce is accepted with offsets continuing from the promotion offset; groups and transactions work as on any topic; a test produces with the Java client immediately after promotion.

### [To Do] X-GEO-0067 Producer id collision safety after promotion
P0 · M15 · Task · deps: X-GEO-0066, X-GEO-0031, TXN-F02
Done when: on promotion the destination's producer id allocation is advanced past the highest producer id seen in mirrored data (or pid ranges are partitioned per the ADR) so new destination producers never reuse a mirrored producer id; a test with overlapping pid spaces proves no OUT_OF_ORDER_SEQUENCE or false duplicate.

### [To Do] X-GEO-0068 Final consumer offset sync at promotion
P0 · M15 · Task · deps: X-GEO-0061, X-GEO-F04
Done when: planned promotion performs one last offset sync after fetchers stop and before writes are enabled; offset sync stops for promoted topics; tests show groups resume exactly at the source position after a planned switchover.

### [To Do] X-GEO-0069 Reverse direction: making the old primary a mirror of the new primary
P0 · M15 · Task · deps: X-GEO-0065, X-GEO-0016
Done when: `alluvium link reverse` converts the old source topic into a mirror of the promoted topic over a reverse link when its log ends at or before the promotion offset; otherwise it refuses and points to the failback truncation flow; tests cover both.

### [To Do] X-GEO-0070 Failback with divergence: truncation of the old primary to the promotion offset
P0 · M15 · Task · deps: X-GEO-0069, X-GEO-0062
Done when: with `--allow-truncate`, records of the old primary beyond the promotion offset are exported to `<topic>.diverged` (or a file) with source offsets in headers, then the log is truncated and mirroring from the new primary starts; event `diverged_records_exported` carries the count; tests verify exported records equal the truncated ones.

### [To Do] X-GEO-0071 Simulation: planned switchover, forced failover and failback round trips
P0 · M15 · Task · deps: X-GEO-0068, X-GEO-0070, X-GEO-0067, TEST-F06 · labels: test
Done when: seeded simulations assert: planned switchover loses and duplicates nothing; forced failover loses at most the records above the promotion offsets; failback round trip ends with both clusters identical up to the new primary's HW; controller failover mid-promotion is safe.

### [To Do] X-GEO-0072 Failover metrics and events
P0 · M15 · Task · deps: X-GEO-0061, OBS-F04, OBS-F07
Done when: `alluvium_link_promotions_total{mode}`, `alluvium_link_promotion_seconds` and `alluvium_link_promotion_estimated_loss_records` exist; events `mirror_promoted`, `mirror_reversed` and `mirror_failback_truncated` carry reasons and offsets.

## [To Do] X-GEO-F07 Active-active
P1 · M15 · Feature · deps: X-GEO-F03
Scope: Ownership or prefixing patterns; loop prevention.

### [To Do] X-GEO-0073 Mirror topic renaming with a prefix pattern
P1 · M15 · Task · deps: X-GEO-0024 · ref: connect/mirror-client/src/main/java/org/apache/kafka/connect/mirror/DefaultReplicationPolicy.java formatRemoteTopic @4.4.0
Done when: per-link `mirror.topic.name.pattern` (default `${topic}`, typical `${link}.${topic}`) names mirror topics; offsets stay identical under renaming; collisions with existing topics fail creation; tests cover patterns and collisions.

### [To Do] X-GEO-0074 Provenance chain metadata for mirrored topics
P1 · M15 · Task · deps: X-GEO-0073
Done when: each mirror topic records the ordered list of origin cluster ids (source's chain plus the source cluster); Alluvium sources expose their chain through DescribeConfigs as a read-only `alluvium.link.provenance` config; Kafka sources yield a chain of one; tests cover multi-hop chains.

### [To Do] X-GEO-0075 Loop prevention by provenance and by name
P1 · M15 · Task · deps: X-GEO-0074 · ref: connect/mirror/src/main/java/org/apache/kafka/connect/mirror/MirrorSourceConnector.java isCycle @4.4.0
Done when: a link refuses to mirror a topic whose provenance contains the local cluster id, and, for Kafka sources, a topic whose name carries the local cluster's prefix per the naming pattern; refusals emit `mirror_loop_prevented`; auto-mirroring never creates such topics; tests cover two- and three-cluster rings.

### [To Do] X-GEO-0076 Topic ownership model: one writable owner, mirrors elsewhere
P1 · M15 · Task · deps: X-GEO-0075, X-GEO-F06
Done when: a topic name is writable in exactly one cluster of a link group; ownership transfer is a planned switchover; describe shows the owner cluster id; a test shows writes to a non-owner copy are rejected.

### [To Do] X-GEO-0077 Offset sync for renamed and aggregated topics
P1 · M15 · Task · deps: X-GEO-0073, X-GEO-F04
Done when: offsets for source topic `t` are written to the renamed mirror `<prefix>.t` on the destination; aggregate consumers subscribing with a regex across local and remote topics keep independent offsets per topic; tests cover renamed topics.

### [To Do] X-GEO-0078 MirrorMaker 2-compatible heartbeats for replication-hop tooling
P3 · M15 · Task · deps: X-GEO-0073 · ref: connect/mirror/src/main/java/org/apache/kafka/connect/mirror/MirrorHeartbeatTask.java @4.4.0
Done when: with `emit.mm2.heartbeats=true` the link writes heartbeats in MM2's format so `RemoteClusterUtils.replicationHops` works; a compat test calls it.

### [To Do] X-GEO-0079 Simulation: bidirectional links among three clusters
P1 · M15 · Task · deps: X-GEO-0075, X-GEO-0077, TEST-F06 · labels: test
Done when: a seeded simulation with bidirectional links among three clusters, auto-mirroring and concurrent writes asserts no record returns to its origin cluster, topic count stays bounded, and every acknowledged record reaches every other cluster after the faults heal.

### [To Do] X-GEO-0080 Active-active patterns guide
P1 · M15 · Task · deps: X-GEO-0076, X-GEO-0077 · labels: docs
Done when: docs/geo/active-active.md covers the prefix and ownership patterns, aggregate consumption with regex subscriptions, loop prevention guarantees and limits (no conflict resolution for the same key written in two regions).

## [To Do] X-GEO-F08 Remote observer replicas
P2 · M15 · Feature · deps: X-GEO-F01, REPL-F06
Scope: Asynchronous replicas in other regions outside the ISR.

### [To Do] X-GEO-0081 Observer replica model and placement constraints
P2 · M15 · Task · deps: X-GEO-0004, META-F02
Done when: an Alluvium-specific record lists observer replicas per partition, separate from replicas used for ISR and ELR; `alluvium.observer.placement` (per-topic JSON: rack or region and count) is validated; replay and snapshot tests pass.

### [To Do] X-GEO-0082 Controller placement and reassignment with observers
P2 · M15 · Task · deps: X-GEO-0081, CTRL-F07, CTRL-F10
Done when: topic creation and reassignment place observers per constraints; observers are never added to ISR or ELR and are never elected leader by preferred, unclean or ELR elections; controller tests cover each election type.

### [To Do] X-GEO-0083 Leader handling of observer fetches outside the ISR
P2 · M15 · Task · deps: X-GEO-0082, REPL-F07
Done when: observers fetch like followers, their LEO is tracked, but they never affect HW, ISR expansion, min ISR or acks=all completion; a test with a stalled observer shows produce latency unaffected.

### [To Do] X-GEO-0084 Consumer reads from local observers
P2 · M15 · Task · deps: X-GEO-0083, REPL-F11
Done when: the KIP-392 replica selector can return an observer in the consumer's rack as preferred read replica; observers serve fetches up to the HW they have received; a compat test reads from an observer with the Java client using `client.rack`.

### [To Do] X-GEO-0085 Exposure of observers in Metadata and DescribeTopicPartitions
P2 · M15 · Task · deps: X-GEO-0082
Done when: observers appear in replica lists but never in ISR or ELR, as specified in the ADR; with no observers configured responses are identical to Kafka (differential test); tests cover each API version.

### [To Do] X-GEO-0086 Optional observer promotion into the ISR under min-ISR pressure
P3 · M15 · Task · deps: X-GEO-0083, CTRL-F09
Done when: with `alluvium.observer.promotion.policy=under-min-isr` a caught-up observer is temporarily added to the ISR when the ISR drops below `min.insync.replicas` and demoted when replicas recover; events carry reasons; simulation tests check no acknowledged write is lost.

### [To Do] X-GEO-0087 Observer replica simulation and lag metrics
P2 · M15 · Task · deps: X-GEO-0083, TEST-F06, OBS-F04
Done when: `alluvium_observer_lag_records{topic}` and `_lag_seconds{topic}` exist; a seeded simulation with WAN delay to observers asserts HW progress is independent of observers and observers converge after faults heal.

### [To Do] X-GEO-0088 Observer replicas guide
P3 · M15 · Task · deps: X-GEO-0084 · labels: docs
Done when: docs/geo/observers.md covers placement, read-locality, promotion policy, the difference from cluster links, and costs.

## [To Do] X-GEO-F09 Link throttling and compression
P1 · M15 · Feature · deps: X-GEO-F03
Scope: Bandwidth limits and compression for links.

### [To Do] X-GEO-0089 Sans-I/O link byte-rate limiter
P1 · M15 · Task · deps: QUOTA-F10
Done when: a token-bucket limiter driven by the injected clock bounds fetched bytes per link per broker to `link.fetch.max.bytes.per.second`; fetch sizes are reduced instead of over-fetching; property tests show the long-run rate never exceeds the limit by more than one maximum fetch.

### [To Do] X-GEO-0090 Cluster-wide link bandwidth budget split across brokers
P1 · M15 · Task · deps: X-GEO-0089
Done when: a per-link cluster budget is divided among brokers in proportion to their mirror partition leadership and recomputed on leadership changes; dynamic updates apply without restart; tests cover rebalancing of the share.

### [To Do] X-GEO-0091 Fairness among mirror partitions on a link
P1 · M15 · Task · deps: X-GEO-0089, X-GEO-0026
Done when: fetch sessions rotate partition order so no partition starves; a test with one hot and many cold partitions shows every cold partition progresses within a bounded number of fetches.

### [To Do] X-GEO-0092 Honoring source-side throttling
P1 · M15 · Task · deps: X-GEO-0026
Done when: non-zero `throttle_time_ms` from the source delays the next fetch per KIP-219 client-side semantics; a compat test with a Kafka 4.4.0 source quota on the link principal shows no quota violations.

### [To Do] X-GEO-0093 Link transport compression between Alluvium clusters
P2 · M15 · Task · deps: X-GEO-0026, REC-F08, REC-F10
Done when: when both ends advertise the Alluvium link compression feature, fetch responses are zstd-compressed as a whole frame and decompressed with a bound equal to `fetch.max.bytes`; Kafka sources fall back automatically; tests show identical appended bytes with and without compression and the compression ratio metric.

### [To Do] X-GEO-0094 Throttling and compression metrics
P1 · M15 · Task · deps: X-GEO-0089, OBS-F04
Done when: `alluvium_link_throttle_seconds_total{link}`, `alluvium_link_fetch_bytes_per_second{link}` and `alluvium_link_compression_ratio{link}` exist and are asserted in tests.

### [To Do] X-GEO-0095 Link throttling benchmarks
P2 · M15 · Task · deps: X-GEO-0089, TEST-F11 · labels: perf
Done when: benchmarks show achieved throughput within 5% of the configured limit across RTTs and the CPU cost of transport compression; results are in docs/benchmarks/geo.md.

## [To Do] X-GEO-F10 Link lag monitoring
P0 · M15 · Feature · deps: X-GEO-F03, OBS-F04
Scope: Lag metrics and alerts.

### [To Do] X-GEO-0096 Mirror lag computation in records and time
P0 · M15 · Task · deps: X-GEO-0026
Done when: per partition, record lag = source HW from the latest fetch response − mirror LEO, and time lag = now − source timestamp of the newest record not yet mirrored (from the source time index via ListOffsets when needed); both are zero exactly when caught up; property tests cover the edge cases.

### [To Do] X-GEO-0097 Lag metrics with bounded cardinality
P0 · M15 · Task · deps: X-GEO-0096
Done when: `alluvium_link_mirror_lag_records{link,topic}` and `alluvium_link_mirror_lag_seconds{link,topic}` (max over partitions) exist; per-partition series only appear with `alluvium.metrics.per.partition.enabled`; tests assert values.

### [To Do] X-GEO-0098 RPO gauge per link
P0 · M15 · Task · deps: X-GEO-0096
Done when: `alluvium_link_rpo_seconds{link}` reports the maximum time lag across all mirror partitions of the link and offset-sync age; a test with one lagging partition shows the gauge follows it.

### [To Do] X-GEO-0099 Lag in describe APIs and CLI
P0 · M15 · Task · deps: X-GEO-0096, X-GEO-0016
Done when: `alluvium link describe --mirrors` and the native describe endpoint return per-partition source HW, mirror LEO, record lag, time lag and state; tests cover a paused link.

### [To Do] X-GEO-0100 Alert rules and dashboards for links
P0 · M15 · Task · deps: X-GEO-0097, X-GEO-0098, OPS-F11
Done when: Prometheus alert rules for link down, lag above threshold, RPO above target and offset sync stale, plus a Grafana dashboard, are added; each alert links to a runbook from X-GEO-F12.

### [To Do] X-GEO-0101 Lag threshold events
P0 · M15 · Task · deps: X-GEO-0096, OBS-F07
Done when: events `mirror_lag_threshold_exceeded` and `mirror_lag_recovered` fire once per crossing of `alluvium.link.lag.alert.records` or `.lag.alert.ms` with hysteresis; tests use the simulated clock.

### [To Do] X-GEO-0102 Lag accuracy simulation
P0 · M15 · Task · deps: X-GEO-0096, TEST-F06 · labels: test
Done when: a seeded simulation compares reported lag with ground truth from both clusters at every step and asserts lag is zero if and only if the mirror is caught up.

## [To Do] X-GEO-F11 Transactional mirroring
P1 · M15 · Feature · deps: X-GEO-F03, TXN-F08
Scope: Mirroring transaction markers for exactly-once consumers.

### [To Do] X-GEO-0103 Control batches mirrored byte-for-byte at the same offsets
P1 · M15 · Task · deps: X-GEO-0027, REC-F11
Done when: COMMIT and ABORT markers and aborted data are fetched with read_uncommitted and appended at their source offsets; a test compares control batches on both clusters.

### [To Do] X-GEO-0104 Transaction index and LSO on mirrors
P1 · M15 · Task · deps: X-GEO-0103, TXN-F11
Done when: mirror appends maintain producer transaction state, the aborted transaction index and the LSO from mirrored markers; read_committed fetches on the mirror return exactly the committed records; mirror LSO never exceeds the source LSO observed.

### [To Do] X-GEO-0105 Open transactions at promotion are aborted
P1 · M15 · Task · deps: X-GEO-0104, X-GEO-0061
Done when: promotion writes ABORT markers for transactions without a marker at the promotion offset (or waits up to `promotion.transaction.wait.ms` in planned mode), so the LSO advances; tests show no stuck LSO after forced failover mid-transaction.

### [To Do] X-GEO-0106 Transaction coordinator state is not mirrored; producers re-initialize
P1 · M15 · Task · deps: X-GEO-0105
Done when: `__transaction_state` is never mirrored; after promotion transactional producers must call InitProducerId on the destination, which fences correctly; documented and tested with the Java transactional producer.

### [To Do] X-GEO-0107 Offset sync reads only committed transactional offsets
P1 · M15 · Task · deps: X-GEO-0104, X-GEO-F04
Done when: offset sync uses `require_stable=true` so offsets pending in open transactions (TxnOffsetCommit) are not synced until committed; tests cover committed and aborted transactional offset commits.

### [To Do] X-GEO-0108 Exactly-once verification across a link
P1 · M15 · Task · deps: X-GEO-0104, TEST-F06 · labels: test
Done when: a seeded simulation with transactional producers on the source, aborts and crashes asserts a read_committed consumer on the mirror sees exactly the committed records in order; a compat test runs a Kafka Streams EOS application against the source and verifies the mirror output.

## [To Do] X-GEO-F12 Geo testing, runbooks and documentation
P0 · M15 · Feature · deps: X-GEO-F03, TEST-F06
Scope: WAN partition simulation, DR runbooks, guide.

### [To Do] X-GEO-0109 Geo-replication is off by default
P0 · M15 · Task · deps: X-GEO-F02, X-GEO-F04, X-GEO-F05, X-GEO-F06, X-GEO-F07, X-GEO-F08 · labels: test
Done when: test `geo_replication_off_by_default` starts a node with default config and asserts `alluvium.link.version` is 0, no link drivers or fetchers run, no observer replicas are placed, and Metadata and DescribeTopicPartitions responses equal Kafka's.

### [To Do] X-GEO-0110 Kafka compatibility and differential suites pass with links enabled but unused
P0 · M15 · Task · deps: X-GEO-0109, TEST-F08, TEST-F09 · labels: test
Done when: the full compat and differential suites run in CI with feature level 1 and one paused link with no mirror topics; all pass with no new ignore rules.

### [To Do] X-GEO-0111 Two- and three-cluster simulation builder with WAN fault plans
P0 · M15 · Task · deps: TEST-F04 · labels: test
Done when: the testkit builds several clusters sharing one simulated network; fault plans add WAN latency (50–300 ms), bandwidth caps, asymmetric partitions and flapping links between clusters; a seeded test reproduces identically from its printed seed.

### [To Do] X-GEO-0112 Real-cluster WAN tests with injected latency and loss
P0 · M15 · Task · deps: X-GEO-F06, TEST-F12 · labels: test
Done when: the chaos harness runs two multi-process clusters with network latency and loss injection, performs mirroring, planned switchover and failback, and checks no-loss and ordering with the Jepsen-style checker.

### [To Do] X-GEO-0113 DR runbooks: switchover, forced failover, failback, link credential rotation, adding topics
P0 · M15 · Task · deps: X-GEO-F06, OPS-F10 · labels: docs
Done when: docs/runbooks/geo-*.md has one runbook per procedure with preconditions, commands, verification and rollback; each runbook is executed step by step by an automated test in the compat harness.

### [To Do] X-GEO-0114 Migration from Kafka using cluster links with identical offsets
P0 · M15 · Task · deps: X-GEO-F04, OPS-F08 · labels: docs
Done when: docs/geo/migrate-from-kafka.md describes linking an Alluvium cluster to a Kafka 4.x cluster, mirroring topics and offsets, cutting producers and consumers over and promoting; an end-to-end compat test follows the guide and verifies no loss and no offset translation.

### [To Do] X-GEO-0115 Migration from MirrorMaker 2 to cluster links
P1 · M15 · Task · deps: X-GEO-F07 · labels: docs
Done when: the guide maps MM2 settings (replication policy, topic and group filters, sync intervals) to link settings and explains moving consumers off translated offsets; a compat test replaces a running MM2 pipeline with a link.

### [To Do] X-GEO-0116 Geo-replication guide and opt-in documentation
P0 · M15 · Task · deps: X-GEO-F06, X-GEO-F10 · labels: docs
Done when: docs/geo/guide.md covers concepts (links, mirror topics and states, offsets, RPO and RTO), enabling the feature level, configuration reference links, monitoring and limits.

### [To Do] X-GEO-0117 Geo security negative tests and redaction
P0 · M15 · Task · deps: X-GEO-F02, OBS-F11 · labels: security, test
Done when: tests cover a link principal lacking source READ, mirror writes by clients, link credential leakage in logs, events, metrics and `/debug/links`, and a source sending oversized or corrupt batches; all are rejected or redacted.

### [To Do] X-GEO-0118 Geo-replication benchmark report
P1 · M15 · Task · deps: X-GEO-0042, X-GEO-F09 · labels: perf, docs
Done when: docs/benchmarks/geo.md reports mirror throughput, CPU, end-to-end replication latency and failover RTO at several RTTs with hardware and versions, compared with MirrorMaker 2 on the same setup.

## [To Do] X-GEO-F13 Erasure propagation across cluster links
P1 · M15 · Feature · deps: X-GEO-F03, X-SECX-F10
Scope: Deletes, tombstones and crypto-shredding performed for right-to-erasure reach every mirror, so erased data doesn't survive on another cluster.

### [To Do] X-GEO-0119 Spike: erasure guarantees across cluster links
P1 · M15 · Spike · deps: none · labels: security
Done when: an accepted ADR defines which erasure operations propagate (DeleteRecords, tombstones, key erasure, key destruction), ordering relative to mirrored data, and the guarantee when a link is down.

### [To Do] X-GEO-0120 Mirror DeleteRecords and log start offset movement
P1 · M15 · Task · deps: X-GEO-0119
Done when: DeleteRecords on a source topic advances the mirror's log start offset to the same offset; tests cover links that were offline during the delete.

### [To Do] X-GEO-0121 Propagate key erasure and crypto-shredding to mirrors
P1 · M15 · Task · deps: X-GEO-0119 · labels: security
Done when: an erasure request recorded on the source is replayed on every linked cluster; key destruction for a data subject is mirrored so mirrored ciphertext becomes unreadable; an audit event is written on each cluster.

### [To Do] X-GEO-0122 Erasure status and lag reporting
P1 · M15 · Task · deps: X-GEO-0120, X-GEO-0121
Done when: each link reports pending erasure operations and their age as metrics and in describe output; an alert rule fires when erasure lag exceeds a configured bound.

### [To Do] X-GEO-0123 Simulation: erased data never survives on a mirror
P1 · M15 · Task · deps: X-GEO-0122, TEST-F06 · labels: test
Done when: a simulation with link outages and failover checks that, after links recover, no mirror serves records erased on the source.
