//! Alluvium's shared domain vocabulary: identifier newtypes that turn a swapped or
//! out-of-range value into a type error instead of a bug found in production, and
//! wrappers that keep secrets and record payloads out of logs.
//!
//! Layer: 0 ([ADR-0009](../../../docs/adr/0009-crate-layering-and-dependency-rules.md)).
//! It depends on no other Alluvium crate, and every crate may depend on it. Like all
//! layer 0 crates it does no I/O, reads no clock and uses no randomness.
//!
//! Kafka counterparts at 4.4.0, each cited with `// Parity:` where it is mirrored:
//!
//! - `clients/.../common/Node.java` (node ids and the `-1` "no node" sentinel);
//! - `clients/.../common/record/internal/RecordBatch.java` and `DefaultRecordBatch.java`
//!   (producer id, epoch and sequence sentinels and arithmetic);
//! - `transaction-coordinator/.../TransactionMetadata.java` (epoch exhaustion);
//! - `clients/.../common/requests/ConsumerGroupHeartbeatRequest.java` and
//!   `JoinGroupRequest.java` (member epoch and generation sentinels);
//! - `clients/.../common/requests/OffsetFetchResponse.java` and `ListOffsetsRequest.java`
//!   (offset sentinels and `ListOffsets` special timestamps);
//! - `clients/.../common/config/types/Password.java` and
//!   `clients/.../common/utils/internals/Crc32C.java`.
//!
//! # Invariants
//!
//! - An id holds a valid value only. Construction goes through `TryFrom` or
//!   `from_wire_nullable`, which reject negative values with [`IdError`].
//! - A wire sentinel such as `-1` ("no node") is never stored as an id: it becomes
//!   `None`.
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
//! - [`IdError`]: why a value was rejected.
//! - [`Secret`]: a value that is never printed and is zeroized on drop.
//! - [`PayloadSummary`]: how record payloads are printed (length and CRC32C only).
//! - `strategies` (feature `proptest-strategies`): proptest strategies for these types,
//!   for property tests in other crates.
//!
//! Errors here carry no Kafka error code: the Kafka error depends on where the value
//! came from, so the decoder or handler that rejects it maps it.

mod epoch;
mod id;
mod offset;
mod payload;
mod producer;
mod secret;
#[cfg(any(test, feature = "proptest-strategies"))]
pub mod strategies;

pub use epoch::{BrokerEpoch, GenerationId, LeaderEpoch, MemberEpoch, PositiveEpoch};
pub use id::{BrokerId, CorrelationId, IdError, IdKind, NodeId, PartitionIndex};
pub use offset::{Offset, OffsetError, OffsetRange, OffsetSpec};
pub use payload::PayloadSummary;
pub use producer::{ProducerEpoch, ProducerId, Sequence};
pub use secret::Secret;
