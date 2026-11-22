//! Producer identity and ordering: the producer id, its epoch, and record sequence
//! numbers, which together make produce requests idempotent.

use std::fmt;

use crate::id::{IdError, IdKind, non_negative, nullable};

/// A producer id, assigned by the transaction coordinator. Non-negative; Kafka writes
/// `NO_PRODUCER_ID` (`-1`) for non-idempotent batches, which reads as `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProducerId(i64);

/// A producer's epoch. A bump fences older instances of the same producer id.
/// Non-negative; `NO_PRODUCER_EPOCH` (`-1`) reads as `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProducerEpoch(i16);

/// The sequence number of a record within a producer's partition stream. Non-negative;
/// `NO_SEQUENCE` (`-1`) reads as `None`. It wraps from `i32::MAX` to 0, as Kafka's does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sequence(i32);

impl ProducerId {
    /// The id as Kafka writes it.
    pub fn get(self) -> i64 {
        self.0
    }

    /// Reads a nullable producer id field.
    ///
    /// # Errors
    ///
    /// [`IdError::Negative`] for a negative value other than `-1`.
    // Parity: clients/src/main/java/org/apache/kafka/common/record/internal/RecordBatch.java@4.4.0 NO_PRODUCER_ID
    pub fn from_wire_nullable(value: i64) -> Result<Option<ProducerId>, IdError> {
        nullable(value, ProducerId::try_from)
    }
}

impl TryFrom<i64> for ProducerId {
    type Error = IdError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        non_negative(IdKind::Producer, value).map(|()| ProducerId(value))
    }
}

impl fmt::Display for ProducerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ProducerEpoch {
    /// The epoch as Kafka writes it.
    pub fn get(self) -> i16 {
        self.0
    }

    /// Reads a nullable producer epoch field.
    ///
    /// # Errors
    ///
    /// [`IdError::Negative`] for a negative value other than `-1`.
    // Parity: clients/src/main/java/org/apache/kafka/common/record/internal/RecordBatch.java@4.4.0 NO_PRODUCER_EPOCH
    pub fn from_wire_nullable(value: i16) -> Result<Option<ProducerEpoch>, IdError> {
        nullable(value, ProducerEpoch::try_from)
    }

    /// True when the epoch can't be bumped safely any more (`i16::MAX - 1` or more). The
    /// coordinator then allocates a new producer id instead: one step is kept in reserve
    /// because aborting a transaction bumps the epoch once more.
    // Parity: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java@4.4.0 TransactionMetadata.isEpochExhausted
    pub fn is_exhausted(self) -> bool {
        self.0 >= i16::MAX - 1
    }

    /// The next epoch, or `None` at `i16::MAX`.
    pub fn checked_bump(self) -> Option<ProducerEpoch> {
        self.0.checked_add(1).map(ProducerEpoch)
    }
}

impl TryFrom<i16> for ProducerEpoch {
    type Error = IdError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        non_negative(IdKind::ProducerEpoch, value.into()).map(|()| ProducerEpoch(value))
    }
}

impl fmt::Display for ProducerEpoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Sequence numbers live in `[0, 2^31)`. 2^31 divides 2^32, so wrapping `u32`
/// arithmetic masked to 31 bits is exactly arithmetic modulo 2^31.
const SEQUENCE_MASK: u32 = 0x7fff_ffff;

impl Sequence {
    /// The sequence number as Kafka writes it.
    pub fn get(self) -> i32 {
        self.0
    }

    /// Reads a nullable sequence field.
    ///
    /// # Errors
    ///
    /// [`IdError::Negative`] for a negative value other than `-1`.
    // Parity: clients/src/main/java/org/apache/kafka/common/record/internal/RecordBatch.java@4.4.0 NO_SEQUENCE
    pub fn from_wire_nullable(value: i32) -> Result<Option<Sequence>, IdError> {
        nullable(value, Sequence::try_from)
    }

    /// The sequence `n` records later, wrapping from `i32::MAX` to 0. For `n` up to
    /// `i32::MAX` this equals Kafka's `incrementSequence`; larger `n` keep wrapping
    /// modulo 2^31.
    // Parity: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java@4.4.0 DefaultRecordBatch.incrementSequence
    #[must_use]
    pub fn wrapping_add(self, n: u32) -> Sequence {
        Sequence((self.0.cast_unsigned().wrapping_add(n) & SEQUENCE_MASK).cast_signed())
    }

    /// The sequence `n` records earlier, wrapping from 0 to `i32::MAX`. For `n` up to
    /// `i32::MAX` this equals Kafka's `decrementSequence`.
    // Parity: clients/src/main/java/org/apache/kafka/common/record/internal/DefaultRecordBatch.java@4.4.0 DefaultRecordBatch.decrementSequence
    #[must_use]
    pub fn wrapping_sub(self, n: u32) -> Sequence {
        Sequence((self.0.cast_unsigned().wrapping_sub(n) & SEQUENCE_MASK).cast_signed())
    }
}

impl TryFrom<i32> for Sequence {
    type Error = IdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        non_negative(IdKind::Sequence, value.into()).map(|()| Sequence(value))
    }
}

impl fmt::Display for Sequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, prop_assert_eq, proptest};

    use super::{ProducerEpoch, ProducerId, Sequence};
    use crate::id::{IdError, IdKind};
    use crate::strategies;

    fn epoch(value: i16) -> ProducerEpoch {
        ProducerEpoch::try_from(value).unwrap_or_else(|error| panic!("{error}"))
    }

    fn seq(value: i32) -> Sequence {
        Sequence::try_from(value).unwrap_or_else(|error| panic!("{error}"))
    }

    /// Kafka's `DefaultRecordBatch.incrementSequence`, transcribed for comparison.
    fn kafka_increment(sequence: i32, increment: i32) -> i32 {
        if sequence > i32::MAX - increment {
            increment - (i32::MAX - sequence) - 1
        } else {
            sequence + increment
        }
    }

    /// Kafka's `DefaultRecordBatch.decrementSequence`, transcribed for comparison.
    fn kafka_decrement(sequence: i32, decrement: i32) -> i32 {
        if sequence < decrement {
            i32::MAX - (decrement - sequence) + 1
        } else {
            sequence - decrement
        }
    }

    proptest! {
        #[test]
        fn sequence_arithmetic_matches_kafka(s in strategies::sequence(), n in 0..=i32::MAX) {
            let n_u32 = n.unsigned_abs();
            prop_assert_eq!(s.wrapping_add(n_u32).get(), kafka_increment(s.get(), n));
            prop_assert_eq!(s.wrapping_sub(n_u32).get(), kafka_decrement(s.get(), n));
        }

        #[test]
        fn subtracting_undoes_adding(s in strategies::sequence(), n in any::<u32>()) {
            prop_assert_eq!(s.wrapping_add(n).wrapping_sub(n), s);
        }

        #[test]
        fn only_non_negative_values_are_accepted(id in any::<i64>(), e in any::<i16>(), s in any::<i32>()) {
            prop_assert_eq!(ProducerId::try_from(id).ok().map(ProducerId::get), (id >= 0).then_some(id));
            prop_assert_eq!(ProducerEpoch::try_from(e).ok().map(ProducerEpoch::get), (e >= 0).then_some(e));
            prop_assert_eq!(Sequence::try_from(s).ok().map(Sequence::get), (s >= 0).then_some(s));
        }
    }

    #[test]
    fn minus_one_is_none_and_other_negatives_are_errors() {
        assert_eq!(ProducerId::from_wire_nullable(-1), Ok(None));
        assert_eq!(ProducerEpoch::from_wire_nullable(-1), Ok(None));
        assert_eq!(Sequence::from_wire_nullable(-1), Ok(None));
        assert_eq!(
            ProducerId::from_wire_nullable(-2),
            Err(IdError::Negative {
                kind: IdKind::Producer,
                value: -2
            })
        );
        assert_eq!(
            ProducerEpoch::from_wire_nullable(i16::MIN).map_err(|error| error.to_string()),
            Err("negative producer epoch -32768".to_owned())
        );
        assert_eq!(
            Sequence::from_wire_nullable(7).map(|s| s.map(Sequence::get)),
            Ok(Some(7))
        );
    }

    #[test]
    fn epochs_near_the_top_are_exhausted_and_stop_bumping() {
        // Parity: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionMetadata.java@4.4.0 TransactionMetadata.isEpochExhausted
        assert!(!epoch(32765).is_exhausted());
        assert!(epoch(32766).is_exhausted());
        assert!(epoch(32767).is_exhausted());
        assert_eq!(epoch(32765).checked_bump(), Some(epoch(32766)));
        assert_eq!(epoch(32766).checked_bump(), Some(epoch(32767)));
        assert_eq!(epoch(32767).checked_bump(), None);
    }

    #[test]
    fn sequences_wrap_at_the_top() {
        assert_eq!(seq(i32::MAX).wrapping_add(1), seq(0));
        assert_eq!(seq(i32::MAX - 1).wrapping_add(5), seq(3));
        assert_eq!(seq(0).wrapping_sub(1), seq(i32::MAX));
        assert_eq!(seq(2).wrapping_sub(5), seq(i32::MAX - 2));
        assert_eq!(seq(10).wrapping_add(0), seq(10));
    }
}
