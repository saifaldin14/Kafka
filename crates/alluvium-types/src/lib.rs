//! Alluvium's shared domain vocabulary: identifier newtypes that turn a swapped or
//! out-of-range value into a type error instead of a bug found in production, and
//! wrappers that keep secrets and record payloads out of logs.
//!
//! Layer: 0 ([ADR-0009](../../../docs/adr/0009-crate-layering-and-dependency-rules.md)).
//! It depends on no other Alluvium crate, and every crate may depend on it. Like all
//! layer 0 crates it does no I/O. It defines the only readers of the operating system's
//! clocks and entropy, [`SystemClock`] and [`SeededRng::from_os_entropy`], but never
//! calls them: only layer 3–5 code does
//! ([ADR-0011](../../../docs/adr/0011-time-model-and-clock-injection.md),
//! [ADR-0012](../../../docs/adr/0012-seeded-randomness-and-hashing.md)).
//!
//! Kafka counterparts at 4.4.0, each cited with `// Parity:` where it is mirrored:
//!
//! - `clients/.../common/Node.java` (node ids and the `-1` "no node" sentinel);
//! - `clients/.../common/Uuid.java` (base64 form, signed ordering, reserved ids);
//! - `clients/.../common/record/TimestampType.java` and
//!   `clients/.../common/record/internal/CompressionType.java` (ids and names);
//! - `clients/.../common/IsolationLevel.java` and `core/.../server/ReplicaManager.scala`
//!   (isolation level and `acks` wire values);
//! - `clients/.../common/internals/Topic.java` (topic name rules, also applied to group
//!   instance ids by `JoinGroupRequest.java`);
//! - `clients/.../common/record/internal/RecordBatch.java` and `DefaultRecordBatch.java`
//!   (producer id, epoch and sequence sentinels and arithmetic);
//! - `transaction-coordinator/.../TransactionMetadata.java` (epoch exhaustion);
//! - `clients/.../common/requests/ConsumerGroupHeartbeatRequest.java` and
//!   `JoinGroupRequest.java` (member epoch and generation sentinels);
//! - `clients/.../common/requests/OffsetFetchResponse.java` and `ListOffsetsRequest.java`
//!   (offset sentinels and `ListOffsets` special timestamps);
//! - `clients/.../common/utils/Time.java`, `SystemTime.java` and `Timer.java` (wall-clock
//!   and monotonic readings, saturating deadlines);
//! - `clients/.../common/network/ListenerName.java` (listener name normalisation);
//! - `clients/.../common/TopicPartition.java` and `TopicIdPartition.java` (display);
//! - `clients/.../common/config/ConfigDef.java` `ValidList` (the `cleanup.policy` list
//!   rules);
//! - `clients/.../common/config/types/Password.java` and
//!   `clients/.../common/utils/internals/Crc32C.java`.
//!
//! # Invariants
//!
//! - An id holds a valid value only. Construction goes through `TryFrom` or
//!   `from_wire_nullable`, which reject negative values with [`IdError`].
//! - A wire sentinel such as `-1` ("no node") or the zero UUID ("no topic id") is
//!   never stored as an id: it becomes `None`.
//!
//! # Entry points
//!
//! - [`NodeId`] and [`BrokerId`]: which process, and which broker.
//! - [`PartitionIndex`] and [`CorrelationId`]: which partition of a topic, and which
//!   request a response answers.
//! - [`ProducerId`], [`ProducerEpoch`] and [`Sequence`]: idempotent and transactional
//!   producers.
//! - [`LeaderEpoch`], [`BrokerEpoch`], [`MemberEpoch`] and [`GenerationId`]: fencing
//!   counters for leaders, brokers and group members.
//! - [`Offset`], [`OffsetRange`] and [`OffsetSpec`]: log positions, half-open ranges of
//!   them, and what `ListOffsets` asks for.
//! - [`MonotonicInstant`] and [`WallTime`]: points in time for deadlines and for
//!   timestamps, never converted into each other ([`TimeError`] when out of range).
//! - [`Clock`]: reads both, passed as `Arc<dyn Clock>`. [`SystemClock`] reads the
//!   operating system ([`WallClockError`] when its wall clock is out of range);
//!   [`ManualClock`] moves only when told to, for tests and simulation.
//! - [`Deadline`]: when a wait must end; a timeout too long to represent saturates.
//! - [`SeededRng`]: the only random number generator, replayable from a `u64` seed,
//!   with independent `fork`s per component ([`EntropyError`] if the OS has none).
//! - [`collections`]: `SeededHashMap` and `SeededHashSet`, hashed under a `HashSeed`
//!   drawn from a `SeededRng`, replacing `std`'s randomly seeded maps.
//! - [`Uuid`]: Kafka's 128-bit ids, printed and parsed in Kafka's base64url form
//!   ([`UuidError`] when a string isn't one), generated from a [`SeededRng`] with
//!   [`Uuid::random`]; and [`TopicId`], which is never zero.
//! - [`TopicName`]: a name that passes Kafka's rules ([`TopicNameError`] names the
//!   rule a string breaks).
//! - [`GroupId`], [`GroupInstanceId`], [`TransactionalId`] and [`ClientId`]: ids
//!   clients choose, printed with [`LogSafe`].
//! - [`ListenerName`]: a listener's name, upper-cased as Kafka normalises it.
//! - [`TimestampType`] and [`CompressionType`]: record batch attributes, by Kafka id
//!   and name ([`AttributeError`] for others).
//! - [`CleanupPolicy`]: delete, compact, both or neither ([`CleanupPolicyError`]).
//! - [`ByteSize`]: a non-negative size, printed with an IEC approximation
//!   ([`ByteSizeError`]).
//! - [`Acks`] and [`IsolationLevel`]: what a write waits for and what a read may see
//!   ([`DeliveryError`] for other wire values).
//! - [`TopicPartition`] and [`TopicIdPartition`]: a partition named by topic name, or by
//!   topic id with the name when known.
//! - [`IdError`]: why a value was rejected.
//! - [`Secret`]: a value that is never printed and is zeroized on drop.
//! - [`PayloadSummary`]: how record payloads are printed (length and CRC32C only).
//! - [`LogSafe`]: how client-supplied strings are printed, with characters that could
//!   forge a log line escaped.
//! - `strategies` (feature `proptest-strategies`): proptest strategies for these types,
//!   for property tests in other crates.
//!
//! Errors here carry no Kafka error code: the Kafka error depends on where the value
//! came from, so the decoder or handler that rejects it maps it.

mod byte_size;
mod cleanup_policy;
mod client_ids;
mod clock;
pub mod collections;
mod deadline;
mod delivery;
mod epoch;
mod id;
mod listener;
mod log_safe;
mod offset;
mod partition;
mod payload;
mod producer;
mod record_attributes;
mod rng;
mod secret;
#[cfg(any(test, feature = "proptest-strategies"))]
pub mod strategies;
mod time;
mod topic;
mod uuid;

pub use byte_size::{ByteSize, ByteSizeError};
pub use cleanup_policy::{CleanupPolicy, CleanupPolicyError};
pub use client_ids::{ClientId, GroupId, GroupInstanceId, TransactionalId};
pub use clock::{Clock, ManualClock, SystemClock, WallClockError};
pub use deadline::Deadline;
pub use delivery::{Acks, DeliveryError, IsolationLevel};
pub use epoch::{BrokerEpoch, GenerationId, LeaderEpoch, MemberEpoch, PositiveEpoch};
pub use id::{BrokerId, CorrelationId, IdError, IdKind, NodeId, PartitionIndex};
pub use listener::{BlankListenerName, ListenerName};
pub use log_safe::LogSafe;
pub use offset::{Offset, OffsetError, OffsetRange, OffsetSpec};
pub use partition::{TopicIdPartition, TopicPartition};
pub use payload::PayloadSummary;
pub use producer::{ProducerEpoch, ProducerId, Sequence};
pub use record_attributes::{AttributeError, CompressionType, TimestampType};
pub use rng::{EntropyError, SeededRng};
pub use secret::Secret;
pub use time::{MonotonicInstant, TimeError, WallTime};
pub use topic::{TopicName, TopicNameError};
pub use uuid::{Base64Problem, TopicId, Uuid, UuidError};
