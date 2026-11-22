//! Identifiers that clients choose: group ids, group instance ids, transactional ids
//! and client ids.
//!
//! Clients put any text they like in these, so every type here prints through
//! [`LogSafe`]: a value can't start a fake log line or reorder the rest of one.
//!
//! Ordering is by UTF-8 bytes. It differs from Java's `String` order only between
//! characters above U+FFFF and those from U+E000 to U+FFFF. Nothing Kafka returns is
//! sorted by these ids.

use std::fmt;

use crate::{IdError, IdKind, LogSafe, TopicNameError, topic};

/// A consumer, share or streams group's id, as the client sent it: any string,
/// including empty.
///
/// The group coordinator rejects an empty id with `INVALID_GROUP_ID`, so an empty
/// `GroupId` can exist here and is refused there, where Kafka refuses it.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupId(String);

/// A static group member's id: the topic-name rules apply, as
/// `JoinGroupRequest.validateGroupInstanceId` checks them, and the error is a
/// [`TopicNameError`]. The request handler maps it to Kafka's
/// `INVALID_CONFIGURATION` response.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupInstanceId(String);

/// A transactional producer's id. Never empty.
///
/// The field is nullable on the wire: null means the producer is not transactional (an
/// idempotent producer gets a fresh producer id), and the empty string is rejected,
/// which the request handler maps to Kafka's response for its API.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionalId(String);

/// The `client.id` a client sends in every request header: any string, including empty
/// and including control characters, which print escaped.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientId(String);

impl GroupId {
    /// The id as the client sent it.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// True for the empty id, which the coordinator rejects.
    // Parity: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorService.java@4.4.0 GroupCoordinatorService.isGroupIdNotEmpty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for GroupId {
    fn from(id: String) -> GroupId {
        GroupId(id)
    }
}

impl From<&str> for GroupId {
    fn from(id: &str) -> GroupId {
        GroupId(id.to_owned())
    }
}

impl GroupInstanceId {
    /// The id.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Parity: clients/src/main/java/org/apache/kafka/common/requests/JoinGroupRequest.java@4.4.0 JoinGroupRequest.validateGroupInstanceId
impl TryFrom<String> for GroupInstanceId {
    type Error = TopicNameError;

    fn try_from(id: String) -> Result<GroupInstanceId, TopicNameError> {
        topic::validate(&id)?;
        Ok(GroupInstanceId(id))
    }
}

impl TryFrom<&str> for GroupInstanceId {
    type Error = TopicNameError;

    fn try_from(id: &str) -> Result<GroupInstanceId, TopicNameError> {
        topic::validate(id)?;
        Ok(GroupInstanceId(id.to_owned()))
    }
}

impl TransactionalId {
    /// The id.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Reads a nullable transactional id field: null is `None`, a non-transactional
    /// producer.
    ///
    /// # Errors
    ///
    /// [`IdError::Empty`] for the empty string.
    // Parity: core/src/main/scala/kafka/coordinator/transaction/TransactionCoordinator.scala@4.4.0 TransactionCoordinator.handleInitProducerId (the null and isEmpty branches)
    pub fn from_wire_nullable(id: Option<String>) -> Result<Option<TransactionalId>, IdError> {
        id.map(TransactionalId::try_from).transpose()
    }
}

impl TryFrom<String> for TransactionalId {
    type Error = IdError;

    fn try_from(id: String) -> Result<TransactionalId, IdError> {
        if id.is_empty() {
            Err(IdError::Empty {
                kind: IdKind::Transactional,
            })
        } else {
            Ok(TransactionalId(id))
        }
    }
}

impl TryFrom<&str> for TransactionalId {
    type Error = IdError;

    fn try_from(id: &str) -> Result<TransactionalId, IdError> {
        TransactionalId::try_from(id.to_owned())
    }
}

impl ClientId {
    /// The id as the client sent it.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ClientId {
    fn from(id: String) -> ClientId {
        ClientId(id)
    }
}

impl From<&str> for ClientId {
    fn from(id: &str) -> ClientId {
        ClientId(id.to_owned())
    }
}

macro_rules! log_safe_fmt {
    ($type:ident) => {
        impl fmt::Display for $type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                LogSafe(&self.0).fmt(f)
            }
        }

        impl fmt::Debug for $type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}(\"{}\")", stringify!($type), LogSafe(&self.0))
            }
        }
    };
}

log_safe_fmt!(GroupId);
log_safe_fmt!(GroupInstanceId);
log_safe_fmt!(TransactionalId);
log_safe_fmt!(ClientId);

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};

    use super::{ClientId, GroupId, GroupInstanceId, TransactionalId};
    use crate::strategies::{client_id, group_id, group_instance_id, transactional_id};
    use crate::{IdError, IdKind, TopicName};

    #[test]
    fn an_empty_group_id_exists_and_says_so() {
        assert!(GroupId::from("").is_empty());
        assert!(!GroupId::from("payments").is_empty());
    }

    #[test]
    fn group_ids_print_log_safe() {
        let id = GroupId::from("payments\nERROR forged");
        assert_eq!(id.to_string(), r"payments\nERROR forged");
        assert_eq!(format!("{id:?}"), r#"GroupId("payments\nERROR forged")"#);
        assert_eq!(id.as_str(), "payments\nERROR forged");
    }

    #[test]
    fn group_instance_ids_follow_the_topic_rules() {
        assert_eq!(
            GroupInstanceId::try_from("consumer-1").unwrap().as_str(),
            "consumer-1"
        );
        assert_eq!(
            format!("{:?}", GroupInstanceId::try_from("consumer-1").unwrap()),
            r#"GroupInstanceId("consumer-1")"#
        );
        for invalid in ["", ".", "..", "has space", "colon:"] {
            assert_eq!(
                GroupInstanceId::try_from(invalid).err(),
                TopicName::try_from(invalid).err(),
                "{invalid:?}"
            );
        }
    }

    #[test]
    fn transactional_ids_are_null_empty_or_valid() {
        assert_eq!(TransactionalId::from_wire_nullable(None), Ok(None));
        let empty = Some(IdError::Empty {
            kind: IdKind::Transactional,
        });
        assert_eq!(
            TransactionalId::from_wire_nullable(Some(String::new())).err(),
            empty
        );
        assert_eq!(TransactionalId::try_from("").err(), empty);
        assert_eq!(
            TransactionalId::from_wire_nullable(Some("orders-tx".to_owned()))
                .unwrap()
                .unwrap()
                .as_str(),
            "orders-tx"
        );
        assert_eq!(
            IdError::Empty {
                kind: IdKind::Transactional
            }
            .to_string(),
            "an empty transactional id is not allowed"
        );
    }

    #[test]
    fn client_ids_accept_anything_and_print_it_escaped() {
        for text in ["", "producer-1", "bell\u{7}", "line\nbreak"] {
            assert_eq!(ClientId::from(text).as_str(), text);
        }
        let id = ClientId::from("line\nbreak\u{7}");
        assert_eq!(id.to_string(), r"line\nbreak\u{7}");
        assert_eq!(format!("{id:?}"), r#"ClientId("line\nbreak\u{7}")"#);
        assert_eq!(
            format!("{:?}", TransactionalId::try_from("tx\r").unwrap()),
            r#"TransactionalId("tx\r")"#
        );
    }

    proptest! {
        #[test]
        fn any_non_empty_transactional_id_is_accepted(id in any::<String>()) {
            let parsed = TransactionalId::from_wire_nullable(Some(id.clone()));
            prop_assert_eq!(parsed.is_err(), id.is_empty());
            if let Ok(Some(parsed)) = parsed {
                prop_assert_eq!(parsed.as_str(), id.as_str());
            }
        }

        #[test]
        fn generated_transactional_and_client_ids_print_on_one_line(tx in transactional_id(), client in client_id()) {
            prop_assert!(!tx.as_str().is_empty());
            prop_assert!(!tx.to_string().contains('\n') && !client.to_string().contains('\n'));
        }

        #[test]
        fn group_instance_ids_reject_exactly_what_topic_names_reject(id in any::<String>()) {
            prop_assert_eq!(
                GroupInstanceId::try_from(id.as_str()).err(),
                TopicName::try_from(id.as_str()).err()
            );
            prop_assert_eq!(
                GroupInstanceId::try_from(id.clone()).err(),
                TopicName::try_from(id).err()
            );
        }

        #[test]
        fn generated_ids_are_valid_and_print_on_one_line(group in group_id(), instance in group_instance_id()) {
            prop_assert!(!group.to_string().contains('\n'));
            prop_assert_eq!(GroupInstanceId::try_from(instance.as_str()), Ok(instance));
        }
    }
}
