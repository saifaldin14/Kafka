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

use crate::{
    BrokerEpoch, BrokerId, CorrelationId, GenerationId, LeaderEpoch, MemberEpoch, MonotonicInstant,
    NodeId, Offset, OffsetRange, OffsetSpec, PartitionIndex, ProducerEpoch, ProducerId, Sequence,
    Uuid, WallTime,
};

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

/// Valid [`ProducerId`]s, biased towards 0, 1, `i32::MAX` and `i64::MAX`.
pub fn producer_id() -> impl Strategy<Value = ProducerId> {
    prop_oneof![
        1 => Just(0),
        1 => Just(1),
        1 => Just(i64::from(i32::MAX)),
        1 => Just(i64::MAX),
        6 => 0..=i64::MAX,
    ]
    .prop_filter_map("non-negative", |value| ProducerId::try_from(value).ok())
}

/// Valid [`ProducerEpoch`]s, biased towards 0, 1 and the exhaustion boundary
/// (`i16::MAX - 2` to `i16::MAX`).
pub fn producer_epoch() -> impl Strategy<Value = ProducerEpoch> {
    prop_oneof![
        1 => Just(0),
        1 => Just(1),
        2 => (i16::MAX - 2)..=i16::MAX,
        6 => 0..=i16::MAX,
    ]
    .prop_filter_map("non-negative", |value| ProducerEpoch::try_from(value).ok())
}

/// Valid [`Sequence`]s, biased towards 0, 1 and the wrap point `i32::MAX`.
pub fn sequence() -> impl Strategy<Value = Sequence> {
    non_negative_i32().prop_filter_map("non-negative", |value| Sequence::try_from(value).ok())
}

/// Valid [`LeaderEpoch`]s, biased towards 0, 1 and `i32::MAX`.
pub fn leader_epoch() -> impl Strategy<Value = LeaderEpoch> {
    non_negative_i32().prop_filter_map("non-negative", |value| LeaderEpoch::try_from(value).ok())
}

/// Valid [`BrokerEpoch`]s, biased towards 0, 1 and `i64::MAX`.
pub fn broker_epoch() -> impl Strategy<Value = BrokerEpoch> {
    prop_oneof![
        1 => Just(0),
        1 => Just(1),
        1 => Just(i64::MAX),
        7 => 0..=i64::MAX,
    ]
    .prop_filter_map("non-negative", |value| BrokerEpoch::try_from(value).ok())
}

/// Every [`MemberEpoch`] sentinel, and positive epochs biased towards 1 and `i32::MAX`.
pub fn member_epoch() -> impl Strategy<Value = MemberEpoch> {
    prop_oneof![
        1 => Just(0),
        1 => Just(-1),
        1 => Just(-2),
        1 => Just(1),
        1 => Just(i32::MAX),
        5 => 1..=i32::MAX,
    ]
    .prop_filter_map("accepted wire value", |value| {
        MemberEpoch::from_wire(value).ok()
    })
}

/// Valid [`GenerationId`]s, biased towards 0, 1 and `i32::MAX`.
pub fn generation_id() -> impl Strategy<Value = GenerationId> {
    non_negative_i32().prop_filter_map("non-negative", |value| GenerationId::try_from(value).ok())
}

/// Valid [`Offset`]s, biased towards 0, 1, `i32::MAX` and `i64::MAX`.
pub fn offset() -> impl Strategy<Value = Offset> {
    prop_oneof![
        1 => Just(0),
        1 => Just(1),
        1 => Just(i64::from(i32::MAX)),
        1 => Just(i64::MAX),
        6 => 0..=i64::MAX,
    ]
    .prop_filter_map("non-negative", |value| Offset::try_from(value).ok())
}

/// Valid [`OffsetRange`]s, including empty ones.
pub fn offset_range() -> impl Strategy<Value = OffsetRange> {
    (offset(), offset()).prop_filter_map("ordered", |(a, b)| {
        OffsetRange::new(a.min(b), a.max(b)).ok()
    })
}

/// Every special [`OffsetSpec`], and timestamps biased towards 0 and `i64::MAX`.
pub fn offset_spec() -> impl Strategy<Value = OffsetSpec> {
    prop_oneof![
        3 => -6_i64..=-1,
        1 => Just(0),
        1 => Just(i64::MAX),
        5 => 0..=i64::MAX,
    ]
    .prop_filter_map("accepted wire value", |value| {
        OffsetSpec::from_wire(value).ok()
    })
}

/// [`MonotonicInstant`]s over the whole `u64` range, biased towards 0, 1 and `u64::MAX`.
pub fn monotonic_instant() -> impl Strategy<Value = MonotonicInstant> {
    prop_oneof![
        1 => Just(0),
        1 => Just(1),
        1 => Just(u64::MAX),
        7 => any::<u64>(),
    ]
    .prop_map(MonotonicInstant::from_nanos)
}

/// Valid [`WallTime`]s, biased towards the epoch, 1 ms and `i64::MAX`.
pub fn wall_time() -> impl Strategy<Value = WallTime> {
    prop_oneof![
        1 => Just(0),
        1 => Just(1),
        1 => Just(i64::MAX),
        7 => 0..=i64::MAX,
    ]
    .prop_filter_map("non-negative", |value| WallTime::try_from(value).ok())
}

/// [`Uuid`]s over all 128 bits, biased towards zero, one and the signed extremes.
pub fn uuid() -> impl Strategy<Value = Uuid> {
    let half = || {
        prop_oneof![
            1 => Just(0),
            1 => Just(1),
            1 => Just(-1),
            1 => Just(i64::MIN),
            1 => Just(i64::MAX),
            5 => any::<i64>(),
        ]
    };
    (half(), half()).prop_map(|(most, least)| Uuid::from_bits(most, least))
}
