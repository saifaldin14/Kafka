//! Proptest strategies for this crate's types, shared with other crates' property tests.
//!
//! Enabled by the `proptest-strategies` feature; other crates list it on their
//! `alluvium-types` dev-dependency. The strategies live here, beside the types, rather
//! than in `alluvium-testkit`: testkit depends on this crate, so a dev-dependency from
//! here to testkit would compile two copies of these types
//! ([testing standard](../../../docs/standards/testing.md#shared-strategies)).
//!
//! Each type strategy produces valid values only, and favours the boundaries where bugs
//! cluster: 0, 1 and the maximum.
//!
//! The boundary-bias helpers at the top are the building blocks every crate's
//! strategies use for plain wire values: integers biased towards Kafka's -1 sentinel, 0,
//! 1 and both extremes, and strings and byte arrays biased towards empty and the
//! maximum length. Generated message strategies (codegen) and record batch strategies
//! are built from them, so a boundary bug in any field is found the same way.

pub mod adversarial;
pub mod composite;

use proptest::collection::{btree_set, vec};
use proptest::option;
use proptest::prelude::{Just, Strategy, any, prop_oneof};
use proptest::sample::{Index, select};

use crate::{
    BrokerEpoch, BrokerId, ClientId, CorrelationId, GenerationId, GroupId, GroupInstanceId,
    KeyHash, KeyRange, LeaderEpoch, MemberEpoch, MonotonicInstant, NamespaceId, NodeId, Offset,
    OffsetRange, OffsetSpec, PartitionIndex, ProducerEpoch, ProducerId, Sequence, TopicId,
    TopicIdPartition, TopicName, TopicPartition, TransactionalId, Uuid, WallTime,
};

/// A strategy for any value of an integer type, with extra weight on the listed
/// boundary values.
macro_rules! boundary_strategy {
    ($(#[$doc:meta])* $name:ident, $int:ty, [$($boundary:expr),+]) => {
        $(#[$doc])*
        pub fn $name() -> impl Strategy<Value = $int> {
            prop_oneof![
                1 => select(vec![$($boundary),+]),
                1 => any::<$int>(),
            ]
        }
    };
}

boundary_strategy!(
    /// Any `i8`, half the time one of -1, 0, 1, `i8::MIN` and `i8::MAX`.
    boundary_i8,
    i8,
    [-1, 0, 1, i8::MIN, i8::MAX]
);
boundary_strategy!(
    /// Any `i16`, half the time one of -1, 0, 1, `i16::MIN` and `i16::MAX`.
    boundary_i16,
    i16,
    [-1, 0, 1, i16::MIN, i16::MAX]
);
boundary_strategy!(
    /// Any `i32`, half the time one of -1, 0, 1, `i32::MIN` and `i32::MAX`.
    boundary_i32,
    i32,
    [-1, 0, 1, i32::MIN, i32::MAX]
);
boundary_strategy!(
    /// Any `i64`, half the time one of -1, 0, 1, `i32::MAX`, `i64::MIN` and `i64::MAX`.
    boundary_i64,
    i64,
    [-1, 0, 1, i64::from(i32::MAX), i64::MIN, i64::MAX]
);
boundary_strategy!(
    /// Any `u16`, half the time one of 0, 1, `i16::MAX` and `u16::MAX`.
    boundary_u16,
    u16,
    [0, 1, i16::MAX.cast_unsigned(), u16::MAX]
);
boundary_strategy!(
    /// Any `u32`, half the time one of 0, 1, `i32::MAX` and `u32::MAX`.
    boundary_u32,
    u32,
    [0, 1, i32::MAX.cast_unsigned(), u32::MAX]
);

/// Non-negative `i32` values, biased towards 0, 1 and `i32::MAX`.
pub fn non_negative_i32() -> impl Strategy<Value = i32> {
    prop_oneof![
        1 => Just(0),
        1 => Just(1),
        1 => Just(i32::MAX),
        7 => 0..=i32::MAX,
    ]
}

/// Non-negative `i64` values, biased towards 0, 1, `i32::MAX` (where 32-bit fields
/// overflow) and `i64::MAX`.
pub fn non_negative_i64() -> impl Strategy<Value = i64> {
    prop_oneof![
        1 => Just(0),
        1 => Just(1),
        1 => Just(i64::from(i32::MAX)),
        1 => Just(i64::MAX),
        6 => 0..=i64::MAX,
    ]
}

/// Strings of at most `max_bytes` UTF-8 bytes, biased towards empty and exactly
/// `max_bytes` (once all ASCII, once ending in a two-byte character), with any
/// characters in between.
pub fn string_up_to(max_bytes: usize) -> impl Strategy<Value = String> {
    let ending_multibyte = max_bytes
        .checked_sub(2)
        .map_or_else(String::new, |ascii| "x".repeat(ascii) + "é");
    prop_oneof![
        1 => Just(String::new()),
        1 => Just("x".repeat(max_bytes)),
        1 => Just(ending_multibyte),
        7 => vec(any::<char>(), 0..=max_bytes.min(32)).prop_map(move |characters| {
            let mut text = String::new();
            for character in characters {
                if text.len() + character.len_utf8() > max_bytes {
                    break;
                }
                text.push(character);
            }
            text
        }),
    ]
}

/// Byte arrays of at most `max_len` bytes, biased towards empty and exactly `max_len`.
pub fn bytes_up_to(max_len: usize) -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        1 => Just(Vec::new()),
        1 => Just(vec![0xff; max_len]),
        8 => vec(any::<u8>(), 0..=max_len.min(64)),
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
    boundary_i32().prop_map(CorrelationId::from)
}

/// Valid [`ProducerId`]s, biased towards 0, 1, `i32::MAX` and `i64::MAX`.
pub fn producer_id() -> impl Strategy<Value = ProducerId> {
    non_negative_i64().prop_filter_map("non-negative", |value| ProducerId::try_from(value).ok())
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

/// Valid [`BrokerEpoch`]s, biased towards 0, 1, `i32::MAX` and `i64::MAX`.
pub fn broker_epoch() -> impl Strategy<Value = BrokerEpoch> {
    non_negative_i64().prop_filter_map("non-negative", |value| BrokerEpoch::try_from(value).ok())
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
    non_negative_i64().prop_filter_map("non-negative", |value| Offset::try_from(value).ok())
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

/// Valid [`WallTime`]s, biased towards the epoch, 1 ms, `i32::MAX` and `i64::MAX`.
pub fn wall_time() -> impl Strategy<Value = WallTime> {
    non_negative_i64().prop_filter_map("non-negative", |value| WallTime::try_from(value).ok())
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

/// Valid [`TopicId`]s: any non-zero [`Uuid`], biased towards the metadata topic's id.
pub fn topic_id() -> impl Strategy<Value = TopicId> {
    prop_oneof![
        1 => Just(TopicId::METADATA),
        9 => uuid().prop_filter_map("non-zero", |id| TopicId::try_from(id).ok()),
    ]
}

const TOPIC_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-";

/// Valid [`TopicName`]s of 1 to 249 characters, biased towards 1, 248 and 249.
pub fn topic_name() -> impl Strategy<Value = TopicName> {
    prop_oneof![
        1 => Just(1_usize),
        1 => Just(248),
        1 => Just(249),
        7 => 1..=249_usize,
    ]
    .prop_flat_map(|length| vec(select(TOPIC_ALPHABET), length))
    .prop_filter_map("not `.` or `..`", |bytes| {
        TopicName::try_from(String::from_utf8(bytes).ok()?).ok()
    })
}

/// Strings that break each of Kafka's topic name rules: empty, `.`, `..`, 250 or more
/// characters (legal or not), and an illegal character anywhere in a valid name.
pub fn invalid_topic_name() -> impl Strategy<Value = String> {
    let illegal = any::<char>().prop_filter("illegal in topic names", |character| {
        !(character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    });
    prop_oneof![
        1 => Just(String::new()),
        1 => Just(".".to_owned()),
        1 => Just("..".to_owned()),
        2 => (250..=300_usize, select(&["a", " ", "é"][..]))
            .prop_map(|(length, unit)| unit.repeat(length)),
        5 => (topic_name(), illegal, any::<Index>()).prop_map(|(name, character, at)| {
            let mut characters: Vec<char> = name.as_str().chars().collect();
            characters.insert(at.index(characters.len() + 1), character);
            characters.into_iter().collect()
        }),
    ]
}

/// Pairs of valid [`TopicName`]s that collide: the second swaps some of the first's
/// `.` and `_` for each other, and both contain at least one.
pub fn colliding_topic_names() -> impl Strategy<Value = (TopicName, TopicName)> {
    (topic_name(), any::<Index>(), any::<u64>()).prop_filter_map(
        "a collision character and two valid names",
        |(name, at, flips)| {
            // Make sure there is at least one collision character to swap.
            let mut first: Vec<u8> = name.as_str().bytes().collect();
            let position = at.index(first.len());
            if !first.iter().any(|byte| matches!(byte, b'.' | b'_')) {
                first[position] = b'_';
            }
            let second: Vec<u8> = first
                .iter()
                .enumerate()
                .map(|(index, &byte)| match byte {
                    b'.' | b'_' if flips >> (index % 64) & 1 == 1 => {
                        if byte == b'.' {
                            b'_'
                        } else {
                            b'.'
                        }
                    }
                    other => other,
                })
                .collect();
            let first = TopicName::try_from(String::from_utf8(first).ok()?).ok()?;
            let second = TopicName::try_from(String::from_utf8(second).ok()?).ok()?;
            Some((first, second))
        },
    )
}

/// [`GroupId`]s: any string, biased towards the empty id the coordinator rejects and
/// towards ids with characters that need escaping in logs.
pub fn group_id() -> impl Strategy<Value = GroupId> {
    prop_oneof![
        1 => Just(String::new()),
        1 => Just("group\nwith\u{202e}controls".to_owned()),
        8 => any::<String>(),
    ]
    .prop_map(GroupId::from)
}

/// Valid [`GroupInstanceId`]s, which follow the topic-name rules.
pub fn group_instance_id() -> impl Strategy<Value = GroupInstanceId> {
    topic_name().prop_filter_map("valid", |name| {
        GroupInstanceId::try_from(name.as_str()).ok()
    })
}

/// [`TransactionalId`]s: any non-empty string, biased towards characters that need
/// escaping in logs.
pub fn transactional_id() -> impl Strategy<Value = TransactionalId> {
    prop_oneof![
        1 => Just("tx\r\nforged".to_owned()),
        9 => any::<String>(),
    ]
    .prop_filter_map("non-empty", |id| TransactionalId::try_from(id).ok())
}

/// [`ClientId`]s: any string, biased towards the empty one and control characters.
pub fn client_id() -> impl Strategy<Value = ClientId> {
    prop_oneof![
        1 => Just(String::new()),
        1 => Just("client\u{0}\n".to_owned()),
        8 => any::<String>(),
    ]
    .prop_map(ClientId::from)
}

/// Valid [`TopicPartition`]s.
pub fn topic_partition() -> impl Strategy<Value = TopicPartition> {
    (topic_name(), partition_index())
        .prop_map(|(topic, partition)| TopicPartition { topic, partition })
}

/// Sets of up to 32 [`TopicPartition`]s over at most four topics and partitions 0 to 7,
/// so topics repeat and partitions collide.
pub fn topic_partitions() -> impl Strategy<Value = std::collections::BTreeSet<TopicPartition>> {
    vec(topic_name(), 1..=4).prop_flat_map(|topics| {
        btree_set(
            (select(topics), 0..8_i32).prop_filter_map("non-negative", |(topic, index)| {
                Some(TopicPartition {
                    topic,
                    partition: PartitionIndex::try_from(index).ok()?,
                })
            }),
            0..=32,
        )
    })
}

/// [`TopicIdPartition`]s, with and without a known topic name.
pub fn topic_id_partition() -> impl Strategy<Value = TopicIdPartition> {
    (topic_id(), partition_index(), option::of(topic_name())).prop_map(
        |(topic_id, partition, topic)| TopicIdPartition {
            topic_id,
            partition,
            topic,
        },
    )
}

/// `isolation_level` wire values: the valid 0 and 1, their neighbours, the extremes and
/// any other `i8`.
pub fn isolation_level_wire_value() -> impl Strategy<Value = i8> {
    prop_oneof![
        4 => 0..=1_i8,
        1 => Just(2),
        5 => boundary_i8(),
    ]
}

/// `acks` wire values: the valid -1, 0 and 1, their neighbours, the extremes and any
/// other `i16`.
pub fn acks_wire_value() -> impl Strategy<Value = i16> {
    prop_oneof![
        4 => -1..=1_i16,
        1 => Just(-2),
        1 => Just(2),
        4 => boundary_i16(),
    ]
}

/// Boundary values of the key hash space, `[0, 2^31]`, biased towards both ends.
fn key_space_point() -> impl Strategy<Value = u32> {
    prop_oneof![
        1 => Just(0_u32),
        1 => Just(1),
        1 => Just((1 << 31) - 1),
        1 => Just(1 << 31),
        6 => 0..=(1_u32 << 31),
    ]
}

/// [`KeyHash`]es over the whole space, biased towards 0, 1 and `2^31 - 1`.
pub fn key_hash() -> impl Strategy<Value = KeyHash> {
    key_space_point().prop_filter_map("inside the space", |value| KeyHash::try_from(value).ok())
}

/// [`KeyRange`]s: empty, small, large and the full space.
pub fn key_range() -> impl Strategy<Value = KeyRange> {
    prop_oneof![
        1 => Just(KeyRange::FULL),
        2 => (0..(1_u32 << 31), 0..64_u32).prop_filter_map("inside the space", |(start, length)| {
            KeyRange::new(start, start.checked_add(length)?).ok()
        }),
        7 => (key_space_point(), key_space_point())
            .prop_filter_map("ordered", |(a, b)| KeyRange::new(a.min(b), a.max(b)).ok()),
    ]
}

/// Sets of up to 17 non-empty [`KeyRange`]s that partition the hash space exactly, in
/// random order: [`KeyRange::FULL`] split at random interior points.
pub fn key_space_partition() -> impl Strategy<Value = Vec<KeyRange>> {
    vec((any::<Index>(), any::<Index>()), 0..16)
        .prop_map(|splits| {
            let mut ranges = vec![KeyRange::FULL];
            for (pick, point) in splits {
                let index = pick.index(ranges.len());
                let range = ranges[index];
                let interior = usize::try_from(range.end() - range.start() - 1).unwrap_or(0);
                if interior == 0 {
                    continue;
                }
                let offset = u32::try_from(point.index(interior)).unwrap_or(0) + 1;
                let at = KeyHash::try_from(range.start() + offset);
                if let Ok((low, high)) = at.and_then(|at| range.split_at(at)) {
                    ranges[index] = low;
                    ranges.push(high);
                }
            }
            ranges
        })
        .prop_shuffle()
}

/// Valid [`NamespaceId`]s, biased towards the default namespace and the 1- and
/// 63-character limits.
pub fn namespace_id() -> impl Strategy<Value = NamespaceId> {
    prop_oneof![
        2 => Just("default".to_owned()),
        1 => Just("a".to_owned()),
        1 => "[a-z][a-z0-9-]{61}[a-z0-9]",
        6 => "[a-z]([a-z0-9-]{0,61}[a-z0-9])?",
    ]
    .prop_filter_map("valid", |id| id.parse().ok())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{Just, Strategy, prop_assert, proptest};
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;

    use super::{bytes_up_to, string_up_to};

    proptest! {
        #[test]
        fn strings_and_bytes_never_exceed_their_maximum(
            (max, text, bytes) in (0..=40_usize)
                .prop_flat_map(|max| (Just(max), string_up_to(max), bytes_up_to(max)))
        ) {
            prop_assert!(text.len() <= max, "{} bytes over {max}", text.len());
            prop_assert!(bytes.len() <= max, "{} bytes over {max}", bytes.len());
        }
    }

    #[test]
    fn the_maximum_length_is_reached() {
        let mut runner = TestRunner::deterministic();
        let lengths: Vec<usize> = (0..200)
            .map(|_| {
                string_up_to(32_767)
                    .new_tree(&mut runner)
                    .unwrap()
                    .current()
                    .len()
            })
            .collect();
        assert!(lengths.contains(&0), "{lengths:?}");
        assert!(lengths.contains(&32_767), "{lengths:?}");
    }
}
