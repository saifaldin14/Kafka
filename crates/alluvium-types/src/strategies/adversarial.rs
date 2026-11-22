//! Byte strategies that attack decoders: length prefixes that lie, inputs cut short at
//! any point, varints that are too long or never end, and nesting deeper than a limit
//! ([security standard](../../../../docs/standards/security.md#untrusted-input)).
//!
//! These produce raw bytes in Kafka's encodings, so decoder property tests in any crate
//! can use them without this crate knowing the decoders. A decoder fed them must return
//! an error or a value, never panic, and never allocate from a declared size it hasn't
//! checked against the bytes it has.

use std::ops::RangeInclusive;

use proptest::collection::vec;
use proptest::prelude::{Just, Strategy, any, prop_oneof};

/// A declared length and the body that follows it, as bytes: the length is an `int32`,
/// big-endian, as Kafka's classic encoding writes it.
fn with_int32_length(length: i32, body: Vec<u8>) -> Vec<u8> {
    let mut bytes = length.to_be_bytes().to_vec();
    bytes.extend(body);
    bytes
}

/// An `int32` length prefix that lies about the body after it, then a body of up to
/// `max_body` bytes. The length is more than the body holds (by one, by a lot, or
/// `i32::MAX`), or negative other than Kafka's -1 for null.
pub fn lying_int32_length(max_body: usize) -> impl Strategy<Value = Vec<u8>> {
    vec(any::<u8>(), 0..=max_body).prop_flat_map(|body| {
        let held = i32::try_from(body.len()).unwrap_or(i32::MAX);
        let lie = prop_oneof![
            Just(held.saturating_add(1)),
            held.saturating_add(1)..=i32::MAX,
            Just(i32::MAX),
            Just(i32::MIN),
            i32::MIN..=-2,
        ];
        (lie, Just(body)).prop_map(|(length, body)| with_int32_length(length, body))
    })
}

/// An unsigned varint, as Kafka's compact encoding writes lengths (length + 1).
fn unsigned_varint(mut value: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(5);
    while value >= 0x80 {
        bytes.push(u8::try_from(value & 0x7f).unwrap_or(0) | 0x80);
        value >>= 7;
    }
    bytes.push(u8::try_from(value).unwrap_or(0));
    bytes
}

/// A compact length prefix (an unsigned varint of the length plus one) that lies about
/// the body after it, then a body of up to `max_body` bytes. The prefix declares more
/// than the body holds, `0x80000000` (which wraps to `i32::MAX` in Kafka's arithmetic),
/// or `0xFFFFFFFF`.
pub fn lying_compact_length(max_body: usize) -> impl Strategy<Value = Vec<u8>> {
    vec(any::<u8>(), 0..=max_body).prop_flat_map(|body| {
        let held = u32::try_from(body.len()).unwrap_or(u32::MAX);
        // The prefix is the length plus one, so `held + 2` declares one byte too many.
        let lie = prop_oneof![
            Just(held.saturating_add(2)),
            held.saturating_add(2)..=u32::MAX,
            Just(0x8000_0000),
            Just(u32::MAX),
        ];
        (lie, Just(body)).prop_map(|(prefix, body)| {
            let mut bytes = unsigned_varint(prefix);
            bytes.extend(body);
            bytes
        })
    })
}

/// `valid` cut short: every strict prefix, biased towards empty, one byte and one byte
/// short.
pub fn truncated(valid: Vec<u8>) -> impl Strategy<Value = Vec<u8>> {
    let last = valid.len().saturating_sub(1);
    prop_oneof![
        1 => Just(0),
        1 => Just(last.min(1)),
        1 => Just(last),
        7 => 0..=last,
    ]
    .prop_map(move |length| valid.get(..length).unwrap_or_default().to_vec())
}

/// How a varint from [`bad_varint`] breaks the rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarintShape {
    /// Within the length limit, but padded with continuation bytes that add nothing.
    /// Kafka decodes these to the same value as the short form.
    Overlong,
    /// The continuation bit stays set past the limit: 5 bytes for a varint, 10 for a
    /// varlong. Kafka rejects these.
    TooLong,
    /// The continuation bit is set on the last byte of the input.
    Unterminated,
}

/// A varint of the given shape, for a format whose limit is `max_bytes` (5 for varints,
/// 10 for varlongs), and nothing after it.
pub fn bad_varint(max_bytes: usize) -> impl Strategy<Value = (Vec<u8>, VarintShape)> {
    let max_bytes = max_bytes.max(2);
    prop_oneof![
        // A value of at most 7 bits, then zero groups, all within the limit.
        (0..0x80_u8, 1..max_bytes).prop_map(|(value, padding)| {
            let mut bytes = vec![value | 0x80];
            bytes.extend(std::iter::repeat_n(0x80, padding - 1));
            bytes.push(0x00);
            (bytes, VarintShape::Overlong)
        }),
        (max_bytes..=max_bytes + 5, any::<u8>()).prop_map(|(continued, last)| {
            let mut bytes = vec![0xff; continued];
            bytes.push(last);
            (bytes, VarintShape::TooLong)
        }),
        (1..max_bytes, any::<u8>())
            .prop_map(|(length, byte)| { (vec![byte | 0x80; length], VarintShape::Unterminated) }),
    ]
}

/// `prefix` repeated `depth` times, then up to 16 arbitrary bytes: nesting as deep as a
/// decoder lets it go. For Kafka's struct arrays, `prefix` is a one-element array length
/// (`[0, 0, 0, 1]` classic, `[2]` compact).
pub fn nested(prefix: Vec<u8>, depth: RangeInclusive<usize>) -> impl Strategy<Value = Vec<u8>> {
    (depth, vec(any::<u8>(), 0..=16)).prop_map(move |(depth, tail)| {
        let mut bytes = prefix.repeat(depth);
        bytes.extend(tail);
        bytes
    })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{Just, Strategy, prop_assert, prop_assert_eq, proptest};
    use proptest::test_runner::{Config, TestCaseError, TestRunner};

    use super::{
        VarintShape, bad_varint, lying_compact_length, lying_int32_length, nested, truncated,
    };

    /// A toy decoder of an `int32`-prefixed byte array that checks the declared length
    /// against the bytes it has. Returns the capacity it would reserve.
    fn checked_capacity(input: &[u8]) -> Option<usize> {
        let (prefix, body) = input.split_first_chunk::<4>()?;
        let length = usize::try_from(i32::from_be_bytes(*prefix)).ok()?;
        (length <= body.len()).then_some(length)
    }

    /// The same decoder without the check: it reserves what the input declares.
    fn unchecked_capacity(input: &[u8]) -> Option<usize> {
        let (prefix, _) = input.split_first_chunk::<4>()?;
        Some(i32::from_be_bytes(*prefix).cast_unsigned() as usize)
    }

    /// The property every decoder must have: what it reserves is backed by input bytes.
    fn reserves_only_what_it_has(
        decoder: fn(&[u8]) -> Option<usize>,
    ) -> Result<(), proptest::test_runner::TestError<Vec<u8>>> {
        let mut runner = TestRunner::new(Config {
            failure_persistence: None,
            ..Config::default()
        });
        runner.run(&lying_int32_length(64), |input| {
            let reserved = decoder(&input).unwrap_or(0);
            if reserved > input.len() {
                return Err(TestCaseError::fail(format!(
                    "reserved {reserved} for {} bytes",
                    input.len()
                )));
            }
            Ok(())
        })
    }

    #[test]
    fn a_decoder_that_trusts_a_declared_length_fails_and_a_checked_one_passes() {
        assert!(reserves_only_what_it_has(checked_capacity).is_ok());
        assert!(reserves_only_what_it_has(unchecked_capacity).is_err());
    }

    proptest! {
        #[test]
        fn int32_lengths_lie(input in lying_int32_length(64)) {
            let (prefix, body) = input.split_first_chunk::<4>().unwrap();
            let declared = i32::from_be_bytes(*prefix);
            let held = i64::try_from(body.len()).unwrap();
            prop_assert!(declared < -1 || i64::from(declared) > held, "{declared}");
        }

        #[test]
        fn compact_lengths_lie(input in lying_compact_length(64)) {
            // Decode the unsigned varint prefix.
            let mut prefix = 0_u64;
            let mut used = 0;
            for (index, byte) in input.iter().enumerate() {
                prefix |= u64::from(byte & 0x7f) << (7 * index);
                used = index + 1;
                if byte & 0x80 == 0 {
                    break;
                }
            }
            let declared = prefix.saturating_sub(1);
            let held = u64::try_from(input.len() - used).unwrap();
            prop_assert!(declared > held, "{prefix:#x}");
        }

        #[test]
        fn truncations_are_strict_prefixes(
            (valid, cut) in proptest::collection::vec(proptest::prelude::any::<u8>(), 1..40)
                .prop_flat_map(|valid| (Just(valid.clone()), truncated(valid)))
        ) {
            prop_assert!(cut.len() < valid.len());
            prop_assert_eq!(&valid[..cut.len()], &cut[..]);
        }

        #[test]
        fn bad_varints_have_the_shape_they_claim((bytes, shape) in bad_varint(5)) {
            let last_byte_ends = bytes.last().is_some_and(|byte| byte & 0x80 == 0);
            let ending = bytes.iter().position(|byte| byte & 0x80 == 0);
            match shape {
                VarintShape::Overlong => {
                    prop_assert!(bytes.len() >= 2 && bytes.len() <= 5);
                    prop_assert_eq!(ending, Some(bytes.len() - 1));
                    // Every byte after the first adds no value bits.
                    prop_assert!(bytes[1..].iter().all(|&byte| byte == 0x80 || byte == 0x00));
                }
                VarintShape::TooLong => prop_assert!(ending.is_none_or(|end| end >= 5)),
                VarintShape::Unterminated => {
                    prop_assert!(!last_byte_ends && bytes.len() < 5);
                }
            }
        }

        #[test]
        fn nesting_repeats_the_prefix(bytes in nested(vec![0, 0, 0, 1], 100..=200)) {
            let depth = bytes.chunks(4).take_while(|chunk| *chunk == [0, 0, 0, 1]).count();
            prop_assert!(depth >= 100, "{depth}");
        }
    }
}
