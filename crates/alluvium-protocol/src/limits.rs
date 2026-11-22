//! [`DecodeLimits`]: the bounds a decode checks a declared size against before it
//! allocates anything for it, and the budget every allocation is charged to.
//!
//! The limits are plain values chosen by the caller; this crate reads no configuration.
//! The count limits default to Kafka's own constants. The allocation budget has no Kafka
//! counterpart: it bounds the memory one decode takes in total
//! ([ADR-0017](../../../docs/adr/0017-per-decode-allocation-budget.md)).

use std::fmt;

/// The most elements allocated up front for an array, however many it declares. The
/// rest are allocated only once that many have decoded, so a false count costs little.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java@4.4.0 MessageUtil.MAX_PREALLOCATED_ARRAY_CAPACITY
pub(crate) const MAX_PREALLOCATED_ARRAY_CAPACITY: usize = 1000;

/// The bounds one decode enforces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeLimits {
    /// The most elements one array may declare.
    pub max_array_length: usize,
    /// The most fields one tagged-field section may declare.
    pub max_tagged_field_count: usize,
    /// The most bytes one decode may allocate, charged before each allocation
    /// ([ADR-0017](../../../docs/adr/0017-per-decode-allocation-budget.md)). Slices of
    /// the input cost nothing.
    pub allocation_budget: usize,
}

impl Default for DecodeLimits {
    /// Kafka's limits, and a budget of Kafka's default `socket.request.max.bytes`.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java@4.4.0 MessageUtil.MAX_ARRAY_LENGTH, MessageUtil.MAX_TAGGED_FIELD_COUNT
    fn default() -> DecodeLimits {
        DecodeLimits {
            max_array_length: 1_000_000,
            max_tagged_field_count: 10_000,
            allocation_budget: 100 * 1024 * 1024,
        }
    }
}

/// A limit a decode can exceed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limit {
    /// [`DecodeLimits::max_array_length`].
    ArrayLength,
    /// Kafka's 32767-byte string length. It is a rule of the format, which no
    /// encoder exceeds, so it is not one of the caller's limits.
    StringLength,
    /// [`DecodeLimits::max_tagged_field_count`].
    TaggedFieldCount,
    /// [`DecodeLimits::allocation_budget`]: the bytes an allocation needs, against
    /// those left in the budget.
    AllocationBudget,
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Limit::ArrayLength => "array length",
            Limit::StringLength => "string length",
            Limit::TaggedFieldCount => "tagged-field count",
            Limit::AllocationBudget => "allocation in bytes",
        })
    }
}
