# Glossary

Kafka has precise terms, and Alluvium uses them exactly, in code, logs and docs. If a term you need isn't here, add it in the same PR.

## Cluster

- **Node**: one Alluvium process. It runs with the broker role, the controller role, or both (combined mode).
- **Broker**: a node role that stores partitions and serves clients.
- **Controller**: a node role that manages cluster metadata. Controllers form the **KRaft quorum**.
- **KRaft**: Kafka's Raft-based metadata quorum, which replaced ZooKeeper. The active controller is the Raft leader.
- **Metadata log**: the replicated Raft log of metadata records (topics, partitions, brokers, configs, ACLs…).
- **Metadata image / delta**: a broker's immutable in-memory view of cluster metadata, built by replaying the metadata log. A delta is the set of changes between two images.
- **Broker epoch**: a number assigned to a broker at each registration. It fences stale broker incarnations.
- **Feature level**: a versioned cluster capability (for example `metadata.version`), upgraded with `UpdateFeatures`.

## Topics and logs

- **Topic**: a named stream of records. It has a name and a **topic id** (UUID).
- **Partition**: an ordered, append-only log. A topic has one or more. Ordering is guaranteed only within a partition.
- **Offset**: the position of a record in a partition, assigned contiguously from 0.
- **Record**: key, value, headers, timestamp. Records are always stored and sent inside a **record batch**.
- **Record batch (v2)**: the unit of storage and transfer. It has a header (base offset, leader epoch, producer id/epoch, base sequence, attributes, CRC) and the records, which may be compressed. Also called "magic 2".
- **Segment**: one file of a partition's log, named by its **base offset**. Old segments are deleted or compacted as a whole.
- **Offset index / time index**: sparse per-segment indexes mapping offset → file position, and timestamp → offset.
- **Log start offset**: the first offset still available. It moves forward through retention, compaction or `DeleteRecords`.
- **Log end offset (LEO)**: the offset the next appended record will get.
- **High watermark (HW)**: the offset up to which records are replicated to all ISR members. Consumers can't read at or beyond it.
- **Last stable offset (LSO)**: the first offset belonging to a still-open transaction. `read_committed` consumers can't read at or beyond it.
- **Retention**: deleting old segments by time (`retention.ms`) or size (`retention.bytes`).
- **Compaction**: keeping only the latest record per key (`cleanup.policy=compact`).
- **Tombstone**: a record with a null value. It marks a key as deleted in a compacted topic.
- **Tiered storage**: moving older segments to remote storage, such as object stores (KIP-405).

## Replication

- **Replica**: a copy of a partition on one broker.
- **Leader**: the replica that handles writes and reads for a partition. **Followers** copy from it by sending Fetch requests.
- **Leader epoch**: a number that increases with every leadership change of a partition. Requests from older epochs are rejected. Followers use it to truncate divergent log suffixes.
- **ISR (in-sync replicas)**: replicas that are caught up with the leader. `acks=all` waits for all of them.
- **Maximal ISR**: the ISR including pending additions not yet committed by the controller. The high watermark is computed over it.
- **ELR (eligible leader replicas)**: replicas that left the ISR but are still safe to elect, because they hold all acknowledged data (KIP-966).
- **`min.insync.replicas`**: the minimum ISR size for accepting `acks=all` writes.
- **Unclean leader election**: electing a replica outside the ISR or ELR. It may lose acknowledged data, and is off by default.
- **Under-replicated / under-min-ISR**: ISR smaller than the replica set / smaller than `min.insync.replicas`.

## Producers

- **acks**: what the leader waits for before replying: `0` (nothing), `1` (its own write), `all` (`-1`; the whole ISR).
- **Idempotent producer**: a producer with a **producer id**, **producer epoch** and per-partition **sequence numbers**, so retries are not appended twice.
- **Transaction**: an atomic set of writes across partitions (and consumer offsets), identified by a **transactional id**.
- **Transaction coordinator**: the broker that manages a transactional id's state in `__transaction_state`.
- **Transaction marker**: a control batch (COMMIT or ABORT) written to each partition in a transaction.
- **Fencing**: rejecting a zombie producer or member because a newer epoch exists.

## Consumers and groups

- **Consumer group**: consumers that share partitions of the topics they subscribe to. Progress is stored as **committed offsets** in `__consumer_offsets`.
- **Group coordinator**: the broker that manages a group.
- **Classic protocol**: the original group protocol (`JoinGroup`/`SyncGroup`/`Heartbeat`). The client computes assignments, and a rebalance pauses the whole group.
- **Consumer protocol**: the newer group protocol (KIP-848, `ConsumerGroupHeartbeat`). The broker computes assignments incrementally.
- **Generation / member epoch**: the version counter of a group's membership (classic) or of a member's assignment (consumer protocol).
- **Rebalance**: redistributing partitions among group members.
- **Share group**: queue-like consumption where members share partitions and acknowledge records individually (KIP-932).
- **Streams group**: the broker-side group protocol for Kafka Streams applications (KIP-1071).
- **Isolation level**: `read_uncommitted` (read up to the high watermark) or `read_committed` (read up to the last stable offset and skip aborted records).

## Protocol

- **API key / API version**: a request type's number and its version. Each request header carries both.
- **ApiVersions**: the request clients use to discover which API versions a broker supports. Alluvium advertises only fully implemented versions.
- **Correlation id**: a client-chosen id echoed in the response, used to match responses to requests.
- **Flexible versions / tagged fields**: newer encodings with compact types and optional fields.
- **Throttle time**: how long the broker tells a client to wait because of quotas.
- **Purgatory / delayed operation**: a request waiting for a condition (for example the high watermark advancing, or enough bytes to fetch) or a timeout.

## Alluvium-specific

- **Sans-I/O**: logic written as a state machine that performs no I/O itself ([ADR-0002](adr/0002-sans-io-state-machines-driven-by-tokio.md)).
- **Driver**: the Tokio task that runs a state machine. It feeds inputs and performs the returned actions.
- **Action**: an output from a state machine asking the driver to do something (send, append, set a timer, respond).
- **Transition**: an output describing a state change and its **reason**. Drivers log it and record it in the flight recorder.
- **Flight recorder**: a bounded ring buffer of a component's recent transitions ([debuggability](standards/debuggability.md#flight-recorders)).
- **Simulation / seed**: deterministic test execution under simulated time, network and disk. The seed reproduces a run exactly ([ADR-0003](adr/0003-deterministic-simulation-testing.md)).
- **Layer**: a crate's position in the dependency order ([architecture §5](architecture.md#5-layering-rules)).
- **Parity citation**: a `// Parity:` comment pointing to the Kafka 4.4.0 source that code mirrors.
