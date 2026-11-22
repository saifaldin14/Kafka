//! Alluvium's shared domain vocabulary: identifier newtypes that turn a swapped or
//! out-of-range value into a type error instead of a bug found in production, and
//! wrappers that keep secrets and record payloads out of logs.
//!
//! Layer: 0 ([ADR-0009](../../../docs/adr/0009-crate-layering-and-dependency-rules.md)).
//! It depends on no other Alluvium crate, and every crate may depend on it. Like all
//! layer 0 crates it does no I/O, reads no clock and uses no randomness.
//!
//! Kafka counterparts at 4.4.0: the `int` node ids of
//! `clients/src/main/java/org/apache/kafka/common/Node.java`, including the `-1`
//! "no node" sentinel of `Node.noNode()`, and
//! `clients/src/main/java/org/apache/kafka/common/config/types/Password.java` and
//! `clients/src/main/java/org/apache/kafka/common/utils/internals/Crc32C.java`.
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
//! - [`IdError`]: why a value was rejected.
//! - [`Secret`]: a value that is never printed and is zeroized on drop.
//! - [`PayloadSummary`]: how record payloads are printed (length and CRC32C only).
//! - `strategies` (feature `proptest-strategies`): proptest strategies for these types,
//!   for property tests in other crates.
//!
//! Errors here carry no Kafka error code: the Kafka error depends on where the value
//! came from, so the decoder or handler that rejects it maps it.

mod id;
mod payload;
mod secret;
#[cfg(any(test, feature = "proptest-strategies"))]
pub mod strategies;

pub use id::{BrokerId, CorrelationId, IdError, IdKind, NodeId, PartitionIndex};
pub use payload::PayloadSummary;
pub use secret::Secret;
