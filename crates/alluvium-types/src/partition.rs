//! Partition identities: a topic partition named by topic name, or by topic id.

use std::fmt;

use crate::{PartitionIndex, TopicId, TopicName};

/// A partition of a topic, named by the topic's name.
///
/// Ordered by topic name, then partition, so a `BTreeSet` or `BTreeMap` of partitions
/// groups each topic's partitions together in index order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TopicPartition {
    /// The topic.
    pub topic: TopicName,
    /// The partition within it.
    pub partition: PartitionIndex,
}

/// A partition named by topic id, with the topic name when it is known.
///
/// Requests that carry topic ids, such as Fetch from version 13, don't carry names, so
/// the name is `None` until it is looked up; resolving it is the metadata image's job,
/// not this type's. Equality compares all three fields, as Kafka's does, so the same
/// partition with and without a resolved name is not equal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TopicIdPartition {
    /// The topic's id.
    pub topic_id: TopicId,
    /// The partition within it.
    pub partition: PartitionIndex,
    /// The topic's name, if known.
    pub topic: Option<TopicName>,
}

impl TopicPartition {
    /// `__cluster_metadata-0`, the controller quorum's metadata log (Kafka's
    /// `Topic.CLUSTER_METADATA_TOPIC_PARTITION`).
    // Parity: clients/src/main/java/org/apache/kafka/common/internals/Topic.java@4.4.0 CLUSTER_METADATA_TOPIC_PARTITION
    pub const CLUSTER_METADATA: TopicPartition = TopicPartition {
        topic: TopicName::CLUSTER_METADATA,
        partition: PartitionIndex::ZERO,
    };
}

/// `<topic>-<partition>`, for example `orders-3`, as Kafka prints it.
// Parity: clients/src/main/java/org/apache/kafka/common/TopicPartition.java@4.4.0 TopicPartition.toString
impl fmt::Display for TopicPartition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.topic, self.partition)
    }
}

/// `<topic id>:<topic>-<partition>`, with `null` for an unknown name, as Kafka prints
/// it.
// Parity: clients/src/main/java/org/apache/kafka/common/TopicIdPartition.java@4.4.0 TopicIdPartition.toString
impl fmt::Display for TopicIdPartition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.topic_id)?;
        match &self.topic {
            Some(topic) => write!(f, "{topic}")?,
            None => f.write_str("null")?,
        }
        write!(f, "-{}", self.partition)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use proptest::prelude::{prop_assert, prop_assert_eq, proptest};

    use super::{TopicIdPartition, TopicPartition};
    use crate::strategies::{topic_id_partition, topic_partitions};
    use crate::{PartitionIndex, TopicId, TopicName};

    fn partition(topic: &str, index: i32) -> TopicPartition {
        TopicPartition {
            topic: TopicName::try_from(topic).unwrap(),
            partition: PartitionIndex::try_from(index).unwrap(),
        }
    }

    #[test]
    fn displays_as_topic_dash_partition() {
        assert_eq!(partition("orders", 3).to_string(), "orders-3");
        // A topic name may itself end in a dash and digits; Kafka prints it the same way.
        assert_eq!(partition("orders-1", 0).to_string(), "orders-1-0");
        assert_eq!(
            TopicPartition::CLUSTER_METADATA.to_string(),
            "__cluster_metadata-0"
        );
    }

    #[test]
    fn sets_iterate_by_topic_then_partition() {
        let set: BTreeSet<TopicPartition> = [
            partition("b", 0),
            partition("a", 10),
            partition("a", 2),
            partition("b", 1),
            partition("a", 0),
        ]
        .into_iter()
        .collect();
        let order: Vec<String> = set.iter().map(ToString::to_string).collect();
        // Partitions sort as numbers: a-10 after a-2.
        assert_eq!(order, ["a-0", "a-2", "a-10", "b-0", "b-1"]);
    }

    #[test]
    fn topic_id_partitions_display_and_compare_like_kafka() {
        let known = TopicIdPartition {
            topic_id: TopicId::METADATA,
            partition: PartitionIndex::ZERO,
            topic: Some(TopicName::CLUSTER_METADATA),
        };
        assert_eq!(
            known.to_string(),
            "AAAAAAAAAAAAAAAAAAAAAQ:__cluster_metadata-0"
        );
        let unknown = TopicIdPartition {
            topic: None,
            ..known.clone()
        };
        assert_eq!(unknown.to_string(), "AAAAAAAAAAAAAAAAAAAAAQ:null-0");
        assert_ne!(known, unknown);
    }

    proptest! {
        #[test]
        fn generated_sets_are_sorted_by_topic_then_partition(set in topic_partitions()) {
            let keys: Vec<(String, i32)> = set
                .iter()
                .map(|tp| (tp.topic.as_str().to_owned(), tp.partition.get()))
                .collect();
            let mut sorted = keys.clone();
            sorted.sort();
            prop_assert_eq!(keys, sorted);
        }

        #[test]
        fn topic_id_partition_display_names_every_field(tip in topic_id_partition()) {
            let shown = tip.to_string();
            let id = tip.topic_id.to_string();
            prop_assert!(shown.starts_with(&id), "{}", shown);
            let suffix = format!("-{}", tip.partition);
            prop_assert!(shown.ends_with(&suffix), "{}", shown);
        }
    }
}
