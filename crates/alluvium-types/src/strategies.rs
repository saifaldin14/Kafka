//! Proptest strategies for this crate's types, shared with other crates' property tests.
//!
//! Enabled by the `proptest-strategies` feature; other crates list it on their
//! `alluvium-types` dev-dependency. The strategies live here, beside the types, rather
//! than in `alluvium-testkit`: testkit depends on this crate, so a dev-dependency from
//! here to testkit would compile two copies of these types
//! ([testing standard](../../../docs/standards/testing.md#shared-strategies)).
//!
//! Each strategy produces valid values only, and favours the boundaries where bugs
//! cluster: 0, 1 and the maximum.

use proptest::prelude::{Just, Strategy, any, prop_oneof};

use crate::{BrokerId, CorrelationId, NodeId, PartitionIndex};

/// Non-negative `i32` values, with extra weight on 0, 1 and `i32::MAX`.
pub(crate) fn non_negative_i32() -> impl Strategy<Value = i32> {
    prop_oneof![
        1 => Just(0),
        1 => Just(1),
        1 => Just(i32::MAX),
        7 => 0..=i32::MAX,
    ]
}

/// Valid [`NodeId`]s, biased towards 0, 1 and `i32::MAX`.
pub fn node_id() -> impl Strategy<Value = NodeId> {
    // The values are never negative, so the conversion never fails and nothing is
    // filtered out. The same holds for every strategy below.
    non_negative_i32().prop_filter_map("non-negative", |value| NodeId::try_from(value).ok())
}

/// Valid [`BrokerId`]s, biased towards 0, 1 and `i32::MAX`.
pub fn broker_id() -> impl Strategy<Value = BrokerId> {
    non_negative_i32().prop_filter_map("non-negative", |value| BrokerId::try_from(value).ok())
}

/// Valid [`PartitionIndex`]es, biased towards 0, 1 and `i32::MAX`.
pub fn partition_index() -> impl Strategy<Value = PartitionIndex> {
    non_negative_i32().prop_filter_map("non-negative", |value| PartitionIndex::try_from(value).ok())
}

/// Any [`CorrelationId`], including negative ones.
pub fn correlation_id() -> impl Strategy<Value = CorrelationId> {
    any::<i32>().prop_map(CorrelationId::from)
}
