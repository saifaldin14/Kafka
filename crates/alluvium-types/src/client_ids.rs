//! Identifiers that clients choose: group ids and group instance ids.
//!
//! Clients put any text they like in these, so every type here prints through
//! [`LogSafe`]: a value can't start a fake log line or reorder the rest of one.
//!
//! Ordering is by UTF-8 bytes. It differs from Java's `String` order only between
//! characters above U+FFFF and those from U+E000 to U+FFFF. Nothing Kafka returns is
//! sorted by these ids.

use std::fmt;

use crate::{LogSafe, TopicNameError, topic};

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

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};

    use super::{GroupId, GroupInstanceId};
    use crate::TopicName;
    use crate::strategies::{group_id, group_instance_id};

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

    proptest! {
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
