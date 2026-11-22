//! [`DecodeLimits`]: the bounds a decode checks a declared size against before it
//! allocates anything for it.
//!
//! The limits are plain values chosen by the caller; this crate reads no configuration.
//! The defaults are Kafka's own constants, so a default decode accepts exactly what
//! Kafka accepts.

use std::fmt;

/// The most elements allocated up front for an array, however many it declares. The
/// rest are allocated as they decode, so a false count costs little.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java@4.4.0 MessageUtil.MAX_PREALLOCATED_ARRAY_CAPACITY
pub(crate) const MAX_PREALLOCATED_ARRAY_CAPACITY: usize = 1000;

/// The bounds one decode enforces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeLimits {
    /// The most elements one array may declare.
    pub max_array_length: usize,
}

impl Default for DecodeLimits {
    /// Kafka's limits.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java@4.4.0 MessageUtil.MAX_ARRAY_LENGTH
    fn default() -> DecodeLimits {
        DecodeLimits {
            max_array_length: 1_000_000,
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
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Limit::ArrayLength => "array length",
            Limit::StringLength => "string length",
        })
    }
}
