//! Identifier newtypes for nodes and brokers.

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

/// Which kind of id a value was meant to be, for error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdKind {
    /// A [`NodeId`].
    Node,
    /// A [`BrokerId`].
    Broker,
}

impl fmt::Display for IdKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            IdKind::Node => "node",
            IdKind::Broker => "broker",
        })
    }
}

/// Why a value is not a valid id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum IdError {
    /// Ids are non-negative; `-1` is accepted only where the wire allows "none".
    #[error("negative {kind} id {value}")]
    Negative {
        /// The id the value was meant to be.
        kind: IdKind,
        /// The rejected value.
        value: i32,
    },
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
        match value {
            -1 => Ok(None),
            _ => NodeId::try_from(value).map(Some),
        }
    }
}

impl TryFrom<i32> for NodeId {
    type Error = IdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(IdError::Negative {
                kind: IdKind::Node,
                value,
            });
        }
        Ok(NodeId(value))
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
        match value {
            -1 => Ok(None),
            _ => BrokerId::try_from(value).map(Some),
        }
    }
}

impl TryFrom<i32> for BrokerId {
    type Error = IdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(IdError::Negative {
                kind: IdKind::Broker,
                value,
            });
        }
        Ok(BrokerId(value))
    }
}

impl fmt::Display for BrokerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{BrokerId, IdError, IdKind, NodeId};

    #[test]
    fn non_negative_values_are_ids() {
        for value in [0, 1, 3, i32::MAX] {
            assert_eq!(NodeId::try_from(value).map(NodeId::get), Ok(value));
            assert_eq!(BrokerId::try_from(value).map(BrokerId::get), Ok(value));
        }
    }

    #[test]
    fn negative_values_are_rejected_with_kind_and_value() {
        for value in [-1, -2, i32::MIN] {
            assert_eq!(
                NodeId::try_from(value),
                Err(IdError::Negative {
                    kind: IdKind::Node,
                    value
                })
            );
            assert_eq!(
                BrokerId::try_from(value),
                Err(IdError::Negative {
                    kind: IdKind::Broker,
                    value
                })
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
            Err(IdError::Negative {
                kind: IdKind::Broker,
                value: -2
            })
        );
        assert_eq!(
            NodeId::from_wire_nullable(i32::MIN),
            Err(IdError::Negative {
                kind: IdKind::Node,
                value: i32::MIN
            })
        );
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
    }

    #[test]
    fn errors_read_as_a_sentence_fragment() {
        let error = IdError::Negative {
            kind: IdKind::Broker,
            value: -2,
        };
        assert_eq!(error.to_string(), "negative broker id -2");
    }
}
