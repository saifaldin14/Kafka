//! Identifier newtypes for nodes, brokers, partitions and requests, and the error every
//! id type reports when a wire value is out of range.

use std::fmt;

/// The id of a node: one Alluvium process, whatever its roles (broker, controller or
/// both). Kafka's `node.id`.
///
/// Use `NodeId` where any node is valid, such as controller quorum voters and the
/// process's own identity. Use [`BrokerId`] where only a broker is valid, such as
/// partition replicas, leaders and the ISR, so a controller-only node can't end up there
/// by mistake. ([glossary](../../../docs/glossary.md))
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(i32);

/// The id of a node in the broker role, where only a broker is valid: partition
/// replicas, leaders and the ISR. Its value is the node's [`NodeId`] value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BrokerId(i32);

/// A partition's index within its topic, from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartitionIndex(i32);

/// The correlation id of a request header: any `i32` the client chose, echoed in the
/// response. Kafka doesn't validate it, so neither do we.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CorrelationId(i32);

/// Which kind of value was rejected, for error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdKind {
    /// A [`NodeId`].
    Node,
    /// A [`BrokerId`].
    Broker,
    /// A [`PartitionIndex`].
    Partition,
    /// A [`ProducerId`](crate::ProducerId).
    Producer,
    /// A [`ProducerEpoch`](crate::ProducerEpoch).
    ProducerEpoch,
    /// A [`Sequence`](crate::Sequence).
    Sequence,
    /// A [`LeaderEpoch`](crate::LeaderEpoch).
    LeaderEpoch,
    /// A [`BrokerEpoch`](crate::BrokerEpoch).
    BrokerEpoch,
    /// A [`MemberEpoch`](crate::MemberEpoch).
    MemberEpoch,
    /// A [`GenerationId`](crate::GenerationId).
    Generation,
    /// A [`TopicId`](crate::TopicId).
    Topic,
}

impl fmt::Display for IdKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            IdKind::Node => "node id",
            IdKind::Broker => "broker id",
            IdKind::Partition => "partition index",
            IdKind::Producer => "producer id",
            IdKind::ProducerEpoch => "producer epoch",
            IdKind::Sequence => "sequence number",
            IdKind::LeaderEpoch => "leader epoch",
            IdKind::BrokerEpoch => "broker epoch",
            IdKind::MemberEpoch => "member epoch",
            IdKind::Generation => "generation id",
            IdKind::Topic => "topic id",
        })
    }
}

/// Why a value is not a valid id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum IdError {
    /// Ids are non-negative. A negative wire value is accepted only where Kafka defines
    /// it as a sentinel (usually `-1`, "none"), and then it becomes `None` or an enum
    /// variant, never an id.
    #[error("negative {kind} {value}")]
    Negative {
        /// The kind of value expected.
        kind: IdKind,
        /// The rejected value, widened to `i64`.
        value: i64,
    },
    /// The zero UUID means "no id" in Kafka, so it is never a UUID id; a nullable
    /// field turns it into `None`.
    #[error("the zero UUID is not a {kind}")]
    Zero {
        /// The kind of id expected.
        kind: IdKind,
    },
}

/// Accepts `value` if it is not negative.
pub(crate) fn non_negative(kind: IdKind, value: i64) -> Result<(), IdError> {
    if value < 0 {
        Err(IdError::Negative { kind, value })
    } else {
        Ok(())
    }
}

/// Reads a nullable wire field: `-1` is `None`, anything else must convert.
pub(crate) fn nullable<T, V: Copy + PartialEq + From<i8>>(
    value: V,
    convert: impl FnOnce(V) -> Result<T, IdError>,
) -> Result<Option<T>, IdError> {
    if value == V::from(-1) {
        Ok(None)
    } else {
        convert(value).map(Some)
    }
}

impl NodeId {
    /// The id as Kafka writes it.
    pub fn get(self) -> i32 {
        self.0
    }

    /// Reads a nullable node id field, where `-1` means "no node".
    ///
    /// # Errors
    ///
    /// [`IdError::Negative`] for a negative value other than `-1`.
    // Parity: clients/src/main/java/org/apache/kafka/common/Node.java@4.4.0 Node.noNode (id -1)
    pub fn from_wire_nullable(value: i32) -> Result<Option<NodeId>, IdError> {
        nullable(value, NodeId::try_from)
    }
}

impl TryFrom<i32> for NodeId {
    type Error = IdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        non_negative(IdKind::Node, value.into()).map(|()| NodeId(value))
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl BrokerId {
    /// The id as Kafka writes it.
    pub fn get(self) -> i32 {
        self.0
    }

    /// Reads a nullable broker id field, where `-1` means "no broker" (for example, no
    /// partition leader).
    ///
    /// # Errors
    ///
    /// [`IdError::Negative`] for a negative value other than `-1`.
    // Parity: clients/src/main/java/org/apache/kafka/common/Node.java@4.4.0 Node.noNode (id -1)
    pub fn from_wire_nullable(value: i32) -> Result<Option<BrokerId>, IdError> {
        nullable(value, BrokerId::try_from)
    }
}

impl TryFrom<i32> for BrokerId {
    type Error = IdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        non_negative(IdKind::Broker, value.into()).map(|()| BrokerId(value))
    }
}

impl fmt::Display for BrokerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartitionIndex {
    /// The index as Kafka writes it.
    pub fn get(self) -> i32 {
        self.0
    }

    /// The next index, or `None` at `i32::MAX`, where there is no next partition.
    pub fn checked_next(self) -> Option<PartitionIndex> {
        self.0.checked_add(1).map(PartitionIndex)
    }
}

impl TryFrom<i32> for PartitionIndex {
    type Error = IdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        non_negative(IdKind::Partition, value.into()).map(|()| PartitionIndex(value))
    }
}

impl fmt::Display for PartitionIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl CorrelationId {
    /// The id as the client sent it.
    pub fn get(self) -> i32 {
        self.0
    }
}

impl From<i32> for CorrelationId {
    fn from(value: i32) -> Self {
        CorrelationId(value)
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, prop_assert_eq, proptest};

    use super::{BrokerId, CorrelationId, IdError, IdKind, NodeId, PartitionIndex};
    use crate::strategies;

    fn negative(kind: IdKind, value: i32) -> IdError {
        IdError::Negative {
            kind,
            value: value.into(),
        }
    }

    proptest! {
        #[test]
        fn try_from_accepts_exactly_the_non_negative_values(value in any::<i32>()) {
            let valid = (value >= 0).then_some(value);
            prop_assert_eq!(NodeId::try_from(value).map(NodeId::get).ok(), valid);
            prop_assert_eq!(BrokerId::try_from(value).map(BrokerId::get).ok(), valid);
            prop_assert_eq!(PartitionIndex::try_from(value).map(PartitionIndex::get).ok(), valid);
        }

        #[test]
        fn nullable_wire_values_round_trip(id in strategies::node_id(), broker in strategies::broker_id()) {
            prop_assert_eq!(NodeId::from_wire_nullable(id.get()), Ok(Some(id)));
            prop_assert_eq!(BrokerId::from_wire_nullable(broker.get()), Ok(Some(broker)));
        }

        #[test]
        fn correlation_ids_round_trip_every_i32(value in any::<i32>()) {
            prop_assert_eq!(CorrelationId::from(value).get(), value);
        }
    }

    #[test]
    fn negative_values_are_rejected_with_kind_and_value() {
        for value in [-1, -2, i32::MIN] {
            assert_eq!(NodeId::try_from(value), Err(negative(IdKind::Node, value)));
            assert_eq!(
                BrokerId::try_from(value),
                Err(negative(IdKind::Broker, value))
            );
            assert_eq!(
                PartitionIndex::try_from(value),
                Err(negative(IdKind::Partition, value))
            );
        }
    }

    #[test]
    fn minus_one_on_the_wire_means_none_and_other_negatives_are_errors() {
        assert_eq!(NodeId::from_wire_nullable(-1), Ok(None));
        assert_eq!(BrokerId::from_wire_nullable(-1), Ok(None));
        assert_eq!(
            NodeId::from_wire_nullable(3).map(|id| id.map(NodeId::get)),
            Ok(Some(3))
        );
        assert_eq!(
            BrokerId::from_wire_nullable(-2),
            Err(negative(IdKind::Broker, -2))
        );
        assert_eq!(
            NodeId::from_wire_nullable(i32::MIN),
            Err(negative(IdKind::Node, i32::MIN))
        );
    }

    #[test]
    fn partition_index_boundaries_and_next() {
        for value in [0, 1, i32::MAX] {
            assert_eq!(
                PartitionIndex::try_from(value).map(PartitionIndex::get),
                Ok(value)
            );
        }
        let next = PartitionIndex::try_from(4).map(PartitionIndex::checked_next);
        assert_eq!(next, Ok(PartitionIndex::try_from(5).ok()));
        let last = PartitionIndex::try_from(i32::MAX).map(PartitionIndex::checked_next);
        assert_eq!(last, Ok(None));
    }

    #[test]
    fn debug_names_the_type_and_display_prints_the_number() {
        let node = NodeId::try_from(3);
        assert_eq!(
            node.map(|id| format!("{id:?} {id}")),
            Ok("NodeId(3) 3".to_owned())
        );
        let broker = BrokerId::try_from(7);
        assert_eq!(
            broker.map(|id| format!("{id:?} {id}")),
            Ok("BrokerId(7) 7".to_owned())
        );
        assert_eq!(
            PartitionIndex::try_from(2).map(|index| index.to_string()),
            Ok("2".to_owned())
        );
        assert_eq!(CorrelationId::from(-5).to_string(), "-5");
    }

    #[test]
    fn errors_read_as_a_sentence_fragment() {
        assert_eq!(
            negative(IdKind::Broker, -2).to_string(),
            "negative broker id -2"
        );
        assert_eq!(
            negative(IdKind::Partition, -1).to_string(),
            "negative partition index -1"
        );
    }
}
