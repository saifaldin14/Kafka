//! Epochs and generations: the counters Kafka uses to fence stale leaders, brokers and
//! group members.

use std::fmt;

use crate::id::{IdError, IdKind, non_negative, nullable};

/// A partition leader epoch. It increases with each leader change, and a replica
/// rejects requests carrying an older one, so comparisons are by numeric value.
/// Non-negative; `NO_PARTITION_LEADER_EPOCH` (`-1`) reads as `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LeaderEpoch(i32);

/// A broker epoch, assigned at each broker registration, which fences requests from an
/// earlier incarnation of the broker. Non-negative; `-1` (unknown) reads as `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BrokerEpoch(i64);

/// The member epoch of a consumer group heartbeat (KIP-848): a request to join or
/// leave, or the member's current epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemberEpoch {
    /// `0`: the member is joining.
    Join,
    /// `-1`: the member is leaving.
    Leave,
    /// `-2`: a static member is leaving temporarily and keeps its assignment.
    LeaveStatic,
    /// A positive epoch.
    Epoch(PositiveEpoch),
}

/// A positive epoch value, the payload of [`MemberEpoch::Epoch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PositiveEpoch(i32);

/// A classic group's generation id. Non-negative; `UNKNOWN_GENERATION_ID` (`-1`) reads as
/// `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenerationId(i32);

impl LeaderEpoch {
    /// The epoch as Kafka writes it.
    pub fn get(self) -> i32 {
        self.0
    }

    /// Reads a nullable leader epoch field.
    ///
    /// # Errors
    ///
    /// [`IdError::Negative`] for a negative value other than `-1`.
    // Parity: clients/src/main/java/org/apache/kafka/common/record/internal/RecordBatch.java@4.4.0 NO_PARTITION_LEADER_EPOCH
    pub fn from_wire_nullable(value: i32) -> Result<Option<LeaderEpoch>, IdError> {
        nullable(value, LeaderEpoch::try_from)
    }
}

impl TryFrom<i32> for LeaderEpoch {
    type Error = IdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        non_negative(IdKind::LeaderEpoch, value.into()).map(|()| LeaderEpoch(value))
    }
}

impl fmt::Display for LeaderEpoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl BrokerEpoch {
    /// The epoch as Kafka writes it.
    pub fn get(self) -> i64 {
        self.0
    }

    /// Reads a nullable broker epoch field, where `-1` means unknown.
    ///
    /// # Errors
    ///
    /// [`IdError::Negative`] for a negative value other than `-1`.
    pub fn from_wire_nullable(value: i64) -> Result<Option<BrokerEpoch>, IdError> {
        nullable(value, BrokerEpoch::try_from)
    }
}

impl TryFrom<i64> for BrokerEpoch {
    type Error = IdError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        non_negative(IdKind::BrokerEpoch, value).map(|()| BrokerEpoch(value))
    }
}

impl fmt::Display for BrokerEpoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Parity: clients/src/main/java/org/apache/kafka/common/requests/ConsumerGroupHeartbeatRequest.java@4.4.0
// JOIN_GROUP_MEMBER_EPOCH (0), LEAVE_GROUP_MEMBER_EPOCH (-1), LEAVE_GROUP_STATIC_MEMBER_EPOCH (-2)
impl MemberEpoch {
    /// Reads the wire value: `0`, `-1`, `-2` or a positive epoch.
    ///
    /// # Errors
    ///
    /// [`IdError::Negative`] for any value below `-2`.
    pub fn from_wire(value: i32) -> Result<MemberEpoch, IdError> {
        match value {
            0 => Ok(MemberEpoch::Join),
            -1 => Ok(MemberEpoch::Leave),
            -2 => Ok(MemberEpoch::LeaveStatic),
            _ if value > 0 => Ok(MemberEpoch::Epoch(PositiveEpoch(value))),
            _ => Err(IdError::Negative {
                kind: IdKind::MemberEpoch,
                value: value.into(),
            }),
        }
    }

    /// The wire value.
    pub fn to_wire(self) -> i32 {
        match self {
            MemberEpoch::Join => 0,
            MemberEpoch::Leave => -1,
            MemberEpoch::LeaveStatic => -2,
            MemberEpoch::Epoch(epoch) => epoch.0,
        }
    }
}

impl fmt::Display for MemberEpoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_wire())
    }
}

impl PositiveEpoch {
    /// The epoch value, always at least 1.
    pub fn get(self) -> i32 {
        self.0
    }
}

impl TryFrom<i32> for PositiveEpoch {
    type Error = IdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > 0 {
            Ok(PositiveEpoch(value))
        } else {
            Err(IdError::Negative {
                kind: IdKind::MemberEpoch,
                value: value.into(),
            })
        }
    }
}

impl GenerationId {
    /// The generation as Kafka writes it.
    pub fn get(self) -> i32 {
        self.0
    }

    /// Reads a nullable generation id field.
    ///
    /// # Errors
    ///
    /// [`IdError::Negative`] for a negative value other than `-1`.
    // Parity: clients/src/main/java/org/apache/kafka/common/requests/JoinGroupRequest.java@4.4.0 UNKNOWN_GENERATION_ID
    pub fn from_wire_nullable(value: i32) -> Result<Option<GenerationId>, IdError> {
        nullable(value, GenerationId::try_from)
    }
}

impl TryFrom<i32> for GenerationId {
    type Error = IdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        non_negative(IdKind::Generation, value.into()).map(|()| GenerationId(value))
    }
}

impl fmt::Display for GenerationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};

    use super::{BrokerEpoch, GenerationId, LeaderEpoch, MemberEpoch, PositiveEpoch};
    use crate::id::{IdError, IdKind};

    proptest! {
        #[test]
        fn member_epochs_round_trip_every_accepted_value(value in any::<i32>()) {
            match MemberEpoch::from_wire(value) {
                Ok(epoch) => {
                    prop_assert!(value >= -2);
                    prop_assert_eq!(epoch.to_wire(), value);
                }
                Err(error) => {
                    prop_assert!(value < -2);
                    prop_assert_eq!(error, IdError::Negative { kind: IdKind::MemberEpoch, value: value.into() });
                }
            }
        }

        #[test]
        fn leader_epochs_order_by_value(a in 0..=i32::MAX, b in 0..=i32::MAX) {
            let (ea, eb) = (LeaderEpoch::try_from(a), LeaderEpoch::try_from(b));
            prop_assert_eq!(ea.ok().cmp(&eb.ok()), a.cmp(&b));
        }

        #[test]
        fn only_non_negative_values_are_accepted(l in any::<i32>(), b in any::<i64>(), g in any::<i32>()) {
            prop_assert_eq!(LeaderEpoch::try_from(l).ok().map(LeaderEpoch::get), (l >= 0).then_some(l));
            prop_assert_eq!(BrokerEpoch::try_from(b).ok().map(BrokerEpoch::get), (b >= 0).then_some(b));
            prop_assert_eq!(GenerationId::try_from(g).ok().map(GenerationId::get), (g >= 0).then_some(g));
        }
    }

    #[test]
    fn member_epoch_sentinels_match_kafka() {
        // Parity: clients/src/main/java/org/apache/kafka/common/requests/ConsumerGroupHeartbeatRequest.java@4.4.0
        assert_eq!(MemberEpoch::from_wire(0), Ok(MemberEpoch::Join));
        assert_eq!(MemberEpoch::from_wire(-1), Ok(MemberEpoch::Leave));
        assert_eq!(MemberEpoch::from_wire(-2), Ok(MemberEpoch::LeaveStatic));
        assert_eq!(MemberEpoch::from_wire(5).map(MemberEpoch::to_wire), Ok(5));
        assert_eq!(
            MemberEpoch::from_wire(-3).map_err(|error| error.to_string()),
            Err("negative member epoch -3".to_owned())
        );
        assert!(PositiveEpoch::try_from(0).is_err());
        assert_eq!(MemberEpoch::LeaveStatic.to_string(), "-2");
    }

    #[test]
    fn minus_one_is_none_for_nullable_fields() {
        assert_eq!(LeaderEpoch::from_wire_nullable(-1), Ok(None));
        assert_eq!(BrokerEpoch::from_wire_nullable(-1), Ok(None));
        assert_eq!(GenerationId::from_wire_nullable(-1), Ok(None));
        for value in [-2, i32::MIN] {
            assert!(LeaderEpoch::from_wire_nullable(value).is_err());
            assert!(GenerationId::from_wire_nullable(value).is_err());
        }
        assert!(BrokerEpoch::from_wire_nullable(i64::MIN).is_err());
        assert_eq!(
            LeaderEpoch::from_wire_nullable(i32::MAX).map(|e| e.map(LeaderEpoch::get)),
            Ok(Some(i32::MAX))
        );
    }
}
