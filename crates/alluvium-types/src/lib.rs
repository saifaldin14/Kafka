//! Alluvium's shared domain vocabulary: identifier newtypes that turn a swapped or
//! out-of-range value into a type error instead of a bug found in production.
//!
//! Layer: 0 ([ADR-0009](../../../docs/adr/0009-crate-layering-and-dependency-rules.md)).
//! It depends on no other Alluvium crate, and every crate may depend on it. Like all
//! layer 0 crates it does no I/O, reads no clock and uses no randomness.
//!
//! Kafka counterparts at 4.4.0: the `int` node ids of
//! `clients/src/main/java/org/apache/kafka/common/Node.java`, including the `-1`
//! "no node" sentinel of `Node.noNode()`.
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
//! - [`IdError`]: why a value was rejected.
//! - `strategies` (feature `proptest-strategies`): proptest strategies for these types,
//!   for property tests in other crates.
//!
//! Errors here carry no Kafka error code: the Kafka error depends on where the value
//! came from, so the decoder or handler that rejects it maps it.

mod id;
#[cfg(any(test, feature = "proptest-strategies"))]
pub mod strategies;

pub use id::{BrokerId, IdError, IdKind, NodeId};
