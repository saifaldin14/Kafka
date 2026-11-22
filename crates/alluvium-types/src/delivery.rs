//! Delivery guarantees on the wire: how many replicas a write waits for (`acks`), and
//! which writes a read may see (isolation level).

use std::fmt;

/// Which records a fetch may return.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsolationLevel {
    /// `0`: every record up to the high watermark, including those in open or aborted
    /// transactions.
    ReadUncommitted,
    /// `1`: only up to the last stable offset, and never records of aborted
    /// transactions.
    ReadCommitted,
}

/// How many replicas must have a produced batch before the broker answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Acks {
    /// `0`: no response at all; the producer doesn't wait.
    None,
    /// `1`: the leader has written it.
    Leader,
    /// `-1`: every in-sync replica has it, subject to `min.insync.replicas`.
    All,
}

/// A wire value that is not one of these enums' values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DeliveryError {
    /// An isolation level other than 0 or 1.
    #[error("unknown isolation level {value}")]
    UnknownIsolationLevel {
        /// The rejected value.
        value: i8,
    },
    /// An `acks` value other than -1, 0 or 1. The produce handler answers it with
    /// `INVALID_REQUIRED_ACKS`, as `ReplicaManager.isValidRequiredAcks` does.
    #[error("invalid acks {value}: must be -1, 0 or 1")]
    InvalidAcks {
        /// The rejected value.
        value: i16,
    },
}

// Parity: clients/src/main/java/org/apache/kafka/common/IsolationLevel.java@4.4.0 IsolationLevel.forId, IsolationLevel.toString
impl IsolationLevel {
    /// Reads the `isolation_level` field of a Fetch or `ListOffsets` request.
    ///
    /// # Errors
    ///
    /// [`DeliveryError::UnknownIsolationLevel`] for any value but 0 and 1.
    pub fn from_wire(value: i8) -> Result<IsolationLevel, DeliveryError> {
        match value {
            0 => Ok(IsolationLevel::ReadUncommitted),
            1 => Ok(IsolationLevel::ReadCommitted),
            _ => Err(DeliveryError::UnknownIsolationLevel { value }),
        }
    }

    /// The value Kafka writes.
    pub fn to_wire(self) -> i8 {
        match self {
            IsolationLevel::ReadUncommitted => 0,
            IsolationLevel::ReadCommitted => 1,
        }
    }
}

/// `read_uncommitted` or `read_committed`, the `isolation.level` config values.
impl fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            IsolationLevel::ReadUncommitted => "read_uncommitted",
            IsolationLevel::ReadCommitted => "read_committed",
        })
    }
}

// Parity: core/src/main/scala/kafka/server/ReplicaManager.scala@4.4.0 ReplicaManager.isValidRequiredAcks
impl Acks {
    /// Reads the `acks` field of a Produce request.
    ///
    /// # Errors
    ///
    /// [`DeliveryError::InvalidAcks`] for any value but -1, 0 and 1.
    pub fn from_wire(value: i16) -> Result<Acks, DeliveryError> {
        match value {
            0 => Ok(Acks::None),
            1 => Ok(Acks::Leader),
            -1 => Ok(Acks::All),
            _ => Err(DeliveryError::InvalidAcks { value }),
        }
    }

    /// The value Kafka writes.
    pub fn to_wire(self) -> i16 {
        match self {
            Acks::None => 0,
            Acks::Leader => 1,
            Acks::All => -1,
        }
    }
}

/// `0`, `1` or `all`, as the producer's `acks` config spells them.
impl fmt::Display for Acks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Acks::None => "0",
            Acks::Leader => "1",
            Acks::All => "all",
        })
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{prop_assert_eq, proptest};

    use super::{Acks, DeliveryError, IsolationLevel};
    use crate::strategies::{acks_wire_value, isolation_level_wire_value};

    #[test]
    fn only_0_and_1_are_isolation_levels() {
        for value in i8::MIN..=i8::MAX {
            let decoded = IsolationLevel::from_wire(value);
            match value {
                0 | 1 => assert_eq!(decoded.map(IsolationLevel::to_wire), Ok(value)),
                _ => assert_eq!(decoded, Err(DeliveryError::UnknownIsolationLevel { value })),
            }
        }
        assert_eq!(IsolationLevel::ReadCommitted.to_string(), "read_committed");
        assert_eq!(
            IsolationLevel::ReadUncommitted.to_string(),
            "read_uncommitted"
        );
    }

    #[test]
    fn only_minus_1_0_and_1_are_acks() {
        for value in i16::MIN..=i16::MAX {
            let decoded = Acks::from_wire(value);
            match value {
                -1..=1 => assert_eq!(decoded.map(Acks::to_wire), Ok(value)),
                _ => assert_eq!(decoded, Err(DeliveryError::InvalidAcks { value })),
            }
        }
        assert_eq!(
            [Acks::None, Acks::Leader, Acks::All].map(|acks| acks.to_string()),
            ["0", "1", "all"]
        );
    }

    proptest! {
        // The strategies give other crates' tests valid and invalid wire values.
        #[test]
        fn wire_value_strategies_cover_both_outcomes(level in isolation_level_wire_value(), acks in acks_wire_value()) {
            prop_assert_eq!(IsolationLevel::from_wire(level).is_ok(), (0..=1).contains(&level));
            prop_assert_eq!(Acks::from_wire(acks).is_ok(), (-1..=1).contains(&acks));
        }
    }
}
