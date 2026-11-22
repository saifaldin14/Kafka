//! Kafka's 128-bit UUIDs: topic ids, directory ids, incarnation ids and the like.
//!
//! Kafka writes a UUID on the wire as two big-endian `int64`s and prints it as 22
//! characters of unpadded URL-safe base64, not in the familiar hyphenated hex form.
//! Ordering compares the two halves as *signed* numbers, as Kafka's `compareTo` does, so
//! sorted lists of ids come out in the same order as on a Kafka broker.

use std::fmt;
use std::str::FromStr;

use crate::{IdError, IdKind, SeededRng};

/// A Kafka UUID: the most significant 64 bits, then the least significant 64 bits.
///
/// `Display` and `FromStr` use Kafka's base64url form (`AAAAAAAAAAAAAAAAAAAAAA` is the
/// zero UUID). `Debug` prints the same form, so ids in logs match what Kafka tools
/// show.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uuid {
    // Field order makes the derived `Ord` compare `most` first, both signed, as
    // Kafka's `compareTo`.
    most: i64,
    least: i64,
}

/// Why a string is not a Kafka UUID.
///
/// Every variant echoes at most 24 characters of the input, Kafka's length limit, so a
/// huge or hostile input never ends up whole in a log line.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UuidError {
    /// The input is longer than 24 characters, the longest base64 UUID (22 characters
    /// plus `==`).
    #[error("input with prefix {prefix:?} is too long to be decoded as a base64 UUID")]
    TooLong {
        /// The first 24 characters of the input.
        prefix: String,
    },
    /// The input is not valid URL-safe base64.
    #[error("input {input:?} is not valid base64url: {problem}")]
    InvalidBase64 {
        /// The input, at most 24 bytes.
        input: String,
        /// What is wrong with it.
        problem: Base64Problem,
    },
    /// The input is valid base64url but doesn't decode to 16 bytes.
    #[error("input {input:?} decoded as {decoded} bytes, not the 16 bytes of a base64 UUID")]
    WrongLength {
        /// The input, at most 24 bytes.
        input: String,
        /// How many bytes it decoded to.
        decoded: usize,
    },
}

/// What makes a string invalid base64url, in the cases Java's `Base64` decoder rejects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Base64Problem {
    /// A byte outside `A-Z`, `a-z`, `0-9`, `-`, `_` and `=`.
    #[error("illegal character 0x{byte:02x} at byte {index}")]
    IllegalCharacter {
        /// The byte.
        byte: u8,
        /// Its position in the input.
        index: usize,
    },
    /// A single character, which encodes fewer than 8 bits.
    #[error("a single character is not base64")]
    TooShort,
    /// The last 4-character unit is padded wrongly, as `=` alone, or as `xx=` without
    /// the second `=`.
    #[error("the last 4-character unit has wrong padding")]
    WrongPadding,
    /// The last unit has one character, too few for a byte.
    #[error("the last unit does not have enough bits for a byte")]
    DanglingCharacter,
    /// Something follows the padding.
    #[error("unexpected byte at {index} after the padding")]
    AfterPadding {
        /// Its position in the input.
        index: usize,
    },
}

/// Kafka's limit on the input to `Uuid.fromString`.
const MAX_INPUT_LENGTH: usize = 24;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// A topic's id. Never [`Uuid::ZERO`], which Kafka uses for "no topic id".
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TopicId(Uuid);

// Parity: clients/src/main/java/org/apache/kafka/common/Uuid.java@4.4.0 Uuid.ZERO_UUID, Uuid.ONE_UUID, Uuid.METADATA_TOPIC_ID, Uuid.RESERVED
impl Uuid {
    /// Kafka's `ZERO_UUID`: a null or unset id. Reserved: never generated.
    pub const ZERO: Uuid = Uuid::from_bits(0, 0);

    /// Kafka's `ONE_UUID`, the metadata topic's id. Reserved: never generated.
    pub const ONE: Uuid = Uuid::from_bits(0, 1);

    /// The UUID with these halves, as Kafka's `new Uuid(mostSigBits, leastSigBits)`.
    pub const fn from_bits(most_significant: i64, least_significant: i64) -> Uuid {
        Uuid {
            most: most_significant,
            least: least_significant,
        }
    }

    /// A random version 4 UUID that is not reserved and whose text has no `-`.
    ///
    /// Kafka avoids `-` so that an id can never be mistaken for a command-line option.
    /// The bits come from `rng`, so a seeded simulation generates the same ids every
    /// run, and production ids are as unpredictable as the root generator.
    // Parity: clients/src/main/java/org/apache/kafka/common/Uuid.java@4.4.0 Uuid.randomUuid
    pub fn random(rng: &mut SeededRng) -> Uuid {
        loop {
            let candidate = Uuid::random_version_4(rng);
            // ZERO and ONE have version 0, so they can't occur; Kafka checks anyway, and
            // so does this, so the rule doesn't depend on that reasoning.
            if candidate != Uuid::ZERO
                && candidate != Uuid::ONE
                && !candidate.to_string().contains('-')
            {
                return candidate;
            }
        }
    }

    /// 16 random bytes with the version 4 and IETF variant bits set, as
    /// `java.util.UUID.randomUUID` does.
    fn random_version_4(rng: &mut SeededRng) -> Uuid {
        let mut bytes = [0; 16];
        rng.fill_bytes(&mut bytes);
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Uuid::from_bytes(bytes)
    }

    /// The most significant 64 bits, the first `int64` on the wire.
    pub const fn most_significant_bits(self) -> i64 {
        self.most
    }

    /// The least significant 64 bits, the second `int64` on the wire.
    pub const fn least_significant_bits(self) -> i64 {
        self.least
    }

    fn to_bytes(self) -> [u8; 16] {
        let mut bytes = [0; 16];
        bytes[..8].copy_from_slice(&self.most.to_be_bytes());
        bytes[8..].copy_from_slice(&self.least.to_be_bytes());
        bytes
    }

    fn from_bytes(bytes: [u8; 16]) -> Uuid {
        let mut most = [0; 8];
        let mut least = [0; 8];
        most.copy_from_slice(&bytes[..8]);
        least.copy_from_slice(&bytes[8..]);
        Uuid {
            most: i64::from_be_bytes(most),
            least: i64::from_be_bytes(least),
        }
    }
}

impl TopicId {
    /// The id of the cluster metadata topic, `__cluster_metadata` (Kafka's
    /// `METADATA_TOPIC_ID`, equal to [`Uuid::ONE`]).
    pub const METADATA: TopicId = TopicId(Uuid::ONE);

    /// The UUID.
    pub fn get(self) -> Uuid {
        self.0
    }

    /// Reads a nullable topic id field, where [`Uuid::ZERO`] means "no topic id".
    pub fn from_wire_nullable(id: Uuid) -> Option<TopicId> {
        TopicId::try_from(id).ok()
    }
}

impl TryFrom<Uuid> for TopicId {
    type Error = IdError;

    fn try_from(id: Uuid) -> Result<TopicId, IdError> {
        if id == Uuid::ZERO {
            Err(IdError::Zero {
                kind: IdKind::Topic,
            })
        } else {
            Ok(TopicId(id))
        }
    }
}

impl fmt::Display for TopicId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for TopicId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TopicId({})", self.0)
    }
}

// Parity: clients/src/main/java/org/apache/kafka/common/Uuid.java@4.4.0 Uuid.toString
impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.to_bytes();
        let mut text = [0; 22];
        for (chunk, out) in bytes.chunks(3).zip(text.chunks_mut(4)) {
            // Up to three bytes as a 24-bit group, then 6 bits per character, highest
            // first. A final short chunk fills only the characters it needs.
            let group = chunk
                .iter()
                .enumerate()
                .fold(0_usize, |group, (index, byte)| {
                    group | usize::from(*byte) << (16 - 8 * index)
                });
            for (index, slot) in out.iter_mut().enumerate() {
                *slot = ALPHABET[(group >> (18 - 6 * index)) & 0x3f];
            }
        }
        // Every byte comes from ALPHABET, which is ASCII.
        f.write_str(std::str::from_utf8(&text).map_err(|_| fmt::Error)?)
    }
}

impl fmt::Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uuid({self})")
    }
}

// Parity: clients/src/main/java/org/apache/kafka/common/Uuid.java@4.4.0 Uuid.fromString
impl FromStr for Uuid {
    type Err = UuidError;

    fn from_str(input: &str) -> Result<Uuid, UuidError> {
        // Kafka counts UTF-16 units and this counts bytes. They agree for ASCII, and any
        // other input is rejected either way: as too long here, as an illegal
        // character by Kafka.
        if input.len() > MAX_INPUT_LENGTH {
            return Err(UuidError::TooLong {
                prefix: input.chars().take(MAX_INPUT_LENGTH).collect(),
            });
        }
        let (decoded, length) =
            decode_base64url(input.as_bytes()).map_err(|problem| UuidError::InvalidBase64 {
                input: input.to_owned(),
                problem,
            })?;
        let bytes: [u8; 16] = decoded
            .get(..length)
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or_else(|| UuidError::WrongLength {
                input: input.to_owned(),
                decoded: length,
            })?;
        Ok(Uuid::from_bytes(bytes))
    }
}

/// Decodes URL-safe base64 as Java's `Base64.getUrlDecoder().decode` does: padding is
/// optional, but if present it must be complete; bits below the last whole byte are
/// ignored, not checked. Returns the bytes and how many there are; at most 18 for the
/// 24-byte inputs this is called with.
// Parity: Java's java.util.Base64.Decoder.decode0 and decodedOutLength (JDK 17+), which Uuid.fromString calls.
fn decode_base64url(input: &[u8]) -> Result<([u8; 18], usize), Base64Problem> {
    if input.len() == 1 {
        return Err(Base64Problem::TooShort);
    }
    let mut out = [0; 18];
    let mut length = 0;
    let mut push = |byte: u32| {
        // Masked to 8 bits, and at most 18 bytes come from a 24-byte input.
        if let (Some(slot), Ok(byte)) = (out.get_mut(length), u8::try_from(byte & 0xff)) {
            *slot = byte;
            length += 1;
        }
    };
    let mut bits = 0_u32;
    let mut shift = 18_i32;
    let mut index = 0;
    while let Some(&byte) = input.get(index) {
        index += 1;
        if byte == b'=' {
            // `=` with no character before it in its unit, or `xx=` not followed by a
            // second `=`.
            let second_pad = input.get(index) == Some(&b'=');
            if shift == 18 || (shift == 6 && !second_pad) {
                return Err(Base64Problem::WrongPadding);
            }
            if shift == 6 {
                index += 1;
            }
            break;
        }
        let value = ALPHABET
            .iter()
            .position(|&symbol| symbol == byte)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(Base64Problem::IllegalCharacter {
                byte,
                index: index - 1,
            })?;
        bits |= value << shift;
        shift -= 6;
        if shift < 0 {
            push(bits >> 16);
            push(bits >> 8);
            push(bits);
            shift = 18;
            bits = 0;
        }
    }
    match shift {
        6 => push(bits >> 16),
        0 => {
            push(bits >> 16);
            push(bits >> 8);
        }
        12 => return Err(Base64Problem::DanglingCharacter),
        _ => {}
    }
    if index < input.len() {
        return Err(Base64Problem::AfterPadding { index });
    }
    Ok((out, length))
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
    use proptest::sample::select;

    use super::{Base64Problem, TopicId, Uuid, UuidError};
    use crate::strategies::{topic_id, uuid};
    use crate::{IdError, IdKind, SeededRng};

    // Ported from clients/src/test/java/org/apache/kafka/common/UuidTest.java@4.4.0
    // (Apache License 2.0). testHashCode and the array helpers are not ported: Java's
    // hashCode and array conversions have no counterpart here.

    #[test]
    fn significant_bits() {
        let id = Uuid::from_bits(34, 98);
        assert_eq!(id.most_significant_bits(), 34);
        assert_eq!(id.least_significant_bits(), 98);
    }

    #[test]
    fn uuid_equality() {
        assert_eq!(Uuid::from_bits(0, 0), Uuid::from_bits(0, 0));
        assert_eq!(Uuid::from_bits(12, 13), Uuid::from_bits(12, 13));
        assert_ne!(Uuid::from_bits(12, 13), Uuid::from_bits(24, 38));
    }

    #[test]
    fn string_conversion() {
        let zero = Uuid::from_bits(0, 0);
        assert_eq!(zero.to_string(), "AAAAAAAAAAAAAAAAAAAAAA");
        assert_eq!("AAAAAAAAAAAAAAAAAAAAAA".parse(), Ok(zero));
        let id: Uuid = "UXyU9i5ARn6W00ON2taeWA".parse().unwrap();
        assert_eq!(id.to_string(), "UXyU9i5ARn6W00ON2taeWA");
        assert_eq!(format!("{id:?}"), "Uuid(UXyU9i5ARn6W00ON2taeWA)");
    }

    #[test]
    fn compare_uuids() {
        let id00 = Uuid::from_bits(0, 0);
        let id01 = Uuid::from_bits(0, 1);
        let id10 = Uuid::from_bits(1, 0);
        assert_eq!(id00.cmp(&id00), Ordering::Equal);
        assert_eq!(id00.cmp(&id01), Ordering::Less);
        assert_eq!(id00.cmp(&id10), Ordering::Less);
        assert_eq!(id01.cmp(&id00), Ordering::Greater);
        assert_eq!(id10.cmp(&id00), Ordering::Greater);
        assert_eq!(id01.cmp(&id10), Ordering::Less);
        assert_eq!(id10.cmp(&id01), Ordering::Greater);
        // Signed: a negative most significant half sorts first.
        assert!(Uuid::from_bits(-1, 0) < Uuid::from_bits(0, 0));
    }

    #[test]
    fn from_string_with_invalid_input() {
        // 32 zero bytes encode as 43 characters; 4 as 6.
        let oversize = "A".repeat(43);
        assert_eq!(
            oversize.parse::<Uuid>(),
            Err(UuidError::TooLong {
                prefix: "A".repeat(24)
            })
        );
        assert_eq!(
            "AAAAAA".parse::<Uuid>(),
            Err(UuidError::WrongLength {
                input: "AAAAAA".to_owned(),
                decoded: 4
            })
        );
    }

    // End of the ported cases.

    #[test]
    fn reserved_uuids_and_the_metadata_topic_id() {
        assert_eq!(Uuid::ZERO.to_string(), "AAAAAAAAAAAAAAAAAAAAAA");
        assert_eq!(Uuid::ONE.to_string(), "AAAAAAAAAAAAAAAAAAAAAQ");
        assert_eq!(TopicId::METADATA.to_string(), "AAAAAAAAAAAAAAAAAAAAAQ");
        assert_eq!(TopicId::METADATA.get(), Uuid::ONE);
        assert_eq!(
            format!("{:?}", TopicId::METADATA),
            "TopicId(AAAAAAAAAAAAAAAAAAAAAQ)"
        );
    }

    #[test]
    fn random_uuids_from_a_fixed_seed_are_fixed() {
        // From the independent Python reference behind the SeededRng goldens, with
        // Python's own base64. The seventh draw, EvqS83l7SyiHHZO8g--tTQ, contains `-`
        // and is skipped, as Kafka skips it.
        let mut rng = SeededRng::from_seed(42);
        let ids: Vec<String> = (0..6).map(|_| Uuid::random(&mut rng).to_string()).collect();
        assert_eq!(
            ids,
            [
                "X143pZvJRcG9O9xrG39lfg",
                "L7gUvESiQP6BXKZ7Y1vXPQ",
                "StL8bPnfSJGjEjx5arVhyw",
                "F5KnWwxfQ0myX3ohSoFAxg",
                "X3ekr8rUS2amIAZ3TAcQxg",
                "TBFSFPtkTXu3Gc0atLOXSw",
            ]
        );
    }

    #[test]
    fn a_topic_id_is_never_zero() {
        assert_eq!(
            TopicId::try_from(Uuid::ZERO),
            Err(IdError::Zero {
                kind: IdKind::Topic
            })
        );
        assert_eq!(TopicId::from_wire_nullable(Uuid::ZERO), None);
        assert_eq!(
            TopicId::from_wire_nullable(Uuid::ONE),
            Some(TopicId::METADATA)
        );
        assert_eq!(
            IdError::Zero {
                kind: IdKind::Topic
            }
            .to_string(),
            "the zero UUID is not a topic id"
        );
    }

    #[test]
    fn padding_follows_javas_decoder() {
        let zero = Ok(Uuid::from_bits(0, 0));
        assert_eq!("AAAAAAAAAAAAAAAAAAAAAA==".parse(), zero);
        // Java ignores the low bits of the last character: `B` is 000001, and only its
        // top two bits are part of the 16th byte.
        assert_eq!("AAAAAAAAAAAAAAAAAAAAAB".parse(), zero);
        let invalid = |input: &str, problem| {
            assert_eq!(
                input.parse::<Uuid>(),
                Err(UuidError::InvalidBase64 {
                    input: input.to_owned(),
                    problem
                }),
                "{input}"
            );
        };
        invalid("AAAAAAAAAAAAAAAAAAAAAA=", Base64Problem::WrongPadding);
        invalid("AAAAAAAAAAAAAAAAAAAAAA=A", Base64Problem::WrongPadding);
        invalid("AAAAAAAAAAAAAAAAAAAAA=", Base64Problem::DanglingCharacter);
        invalid("AAAAAAAAAAAAAAAAAAAA====", Base64Problem::WrongPadding);
        invalid(
            "AAAAAAAAAAAAAAAAAA==A",
            Base64Problem::AfterPadding { index: 20 },
        );
        invalid(
            "AAAAAAAAAAAAAAAAAAAAA+",
            Base64Problem::IllegalCharacter {
                byte: b'+',
                index: 21,
            },
        );
        invalid("A", Base64Problem::TooShort);
        // `xxx=` is a complete final unit, so this is valid base64 of the wrong length.
        assert_eq!(
            "AAAAAAAAAAAAAAAAAAAAAAA=".parse::<Uuid>(),
            Err(UuidError::WrongLength {
                input: "AAAAAAAAAAAAAAAAAAAAAAA=".to_owned(),
                decoded: 17
            })
        );
        assert_eq!(
            "".parse::<Uuid>(),
            Err(UuidError::WrongLength {
                input: String::new(),
                decoded: 0
            })
        );
    }

    #[test]
    fn errors_echo_at_most_24_characters() {
        let long = "é".repeat(1_000);
        let Err(UuidError::TooLong { prefix }) = long.parse::<Uuid>() else {
            panic!("expected TooLong");
        };
        assert_eq!(prefix.chars().count(), 24);
    }

    /// An independent model of the strings Java's decoder turns into 16 bytes: exactly
    /// 22 alphabet characters, optionally followed by `==`. The first 21 characters give
    /// 126 bits and the last one its top 2; its low 4 bits are dropped.
    fn model(input: &str) -> Option<Uuid> {
        let body = input.strip_suffix("==").unwrap_or(input);
        if body.len() != 22 {
            return None;
        }
        let digit = |byte: u8| {
            super::ALPHABET
                .iter()
                .position(|&symbol| symbol == byte)
                .and_then(|digit| u128::try_from(digit).ok())
        };
        let (&last, first) = body.as_bytes().split_last()?;
        let mut value = 0_u128;
        for &byte in first {
            value = value << 6 | digit(byte)?;
        }
        value = value << 2 | digit(last)? >> 4;
        Some(Uuid::from_bytes(value.to_be_bytes()))
    }

    proptest! {
        #[test]
        fn strings_round_trip(id in uuid()) {
            let text = id.to_string();
            prop_assert_eq!(text.len(), 22);
            prop_assert_eq!(text.parse::<Uuid>(), Ok(id));
        }

        #[test]
        fn topic_ids_round_trip_through_nullable_fields(id in topic_id()) {
            prop_assert_eq!(TopicId::from_wire_nullable(id.get()), Some(id));
            prop_assert_eq!(id.to_string(), id.get().to_string());
        }

        #[test]
        fn random_uuids_are_version_4_unreserved_and_dash_free(seed in any::<u64>()) {
            let mut rng = SeededRng::from_seed(seed);
            for _ in 0..16 {
                let id = Uuid::random(&mut rng);
                prop_assert!(id != Uuid::ZERO && id != Uuid::ONE);
                prop_assert!(!id.to_string().contains('-'));
                let bytes = id.to_bytes();
                prop_assert_eq!(bytes[6] >> 4, 4, "version");
                prop_assert_eq!(bytes[8] >> 6, 0b10, "IETF variant");
            }
        }

        #[test]
        fn ordering_is_signed_most_then_least(a in uuid(), b in uuid()) {
            let expected = a
                .most_significant_bits()
                .cmp(&b.most_significant_bits())
                .then(a.least_significant_bits().cmp(&b.least_significant_bits()));
            prop_assert_eq!(a.cmp(&b), expected);
        }

        #[test]
        fn parsing_matches_the_model_of_javas_decoder(
            chars in proptest::collection::vec(
                select(&b"AB_-z09=+/ \xc3"[..]),
                0..=26,
            ),
            canonical in uuid(),
        ) {
            // Short random strings over a few alphabet bytes, `=` and some illegal
            // bytes, plus canonical strings with each kind of ending.
            let random = String::from_utf8_lossy(&chars).into_owned();
            let text = canonical.to_string();
            for input in [random, text.clone(), format!("{text}=="), format!("{text}=")] {
                let parsed = input.parse::<Uuid>();
                prop_assert_eq!(parsed.as_ref().ok().copied(), model(&input), "{:?}", input);
            }
        }

        // Any string at all, including long and non-ASCII ones: parsing returns rather
        // than panicking, and accepts only what the model accepts.
        #[test]
        fn any_input_is_parsed_or_rejected_like_the_model(input in any::<String>()) {
            prop_assert_eq!(input.parse::<Uuid>().ok(), model(&input));
        }
    }
}
