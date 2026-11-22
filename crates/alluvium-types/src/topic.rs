// Portions of this file are ported from Apache Kafka 4.4.0 and modified by Alluvium;
// each is marked `// Ported from:`. Apache Kafka is Copyright The Apache Software
// Foundation and licensed under the Apache License 2.0 (LICENSES/Apache-2.0.txt).
// See NOTICE.

//! Topic names, validated by Kafka's rules.

use std::borrow::Cow;
use std::fmt;

/// A valid topic name: 1 to 249 characters from `[a-zA-Z0-9._-]`, and not `.` or `..`.
///
/// Every valid name is ASCII, so ordering by bytes is the same as Kafka's ordering of
/// Java strings, and the name prints as is: it can't contain anything that would need
/// escaping in a log line.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TopicName(Cow<'static, str>);

/// Why a string is not a valid topic name. Each variant is one of Kafka's rules, and a
/// name that breaks several is reported with the first, in Kafka's order.
///
/// The name itself is not included: it is client-supplied and may be up to 32 KiB. The
/// request handler, which has the name, maps this to `INVALID_TOPIC_EXCEPTION` with
/// Kafka's message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TopicNameError {
    /// The empty string.
    #[error("the empty string is not allowed")]
    Empty,
    /// The name `.`.
    #[error("'.' is not allowed")]
    Dot,
    /// The name `..`.
    #[error("'..' is not allowed")]
    DotDot,
    /// More than 249 characters, counted as Kafka counts them, in UTF-16 code units.
    #[error("the name is {length} characters long, longer than the maximum {MAX_LENGTH}")]
    TooLong {
        /// The length in UTF-16 code units.
        length: usize,
    },
    /// A character outside the ASCII alphanumerics, `.`, `_` and `-`.
    #[error(
        "character {character:?} at position {position} is not an ASCII alphanumeric, '.', '_' or '-'"
    )]
    IllegalCharacter {
        /// The first illegal character.
        character: char,
        /// Its position, counted in characters from 0.
        position: usize,
    },
}

/// Kafka's `MAX_NAME_LENGTH`.
pub(crate) const MAX_LENGTH: usize = 249;

// Parity: clients/src/main/java/org/apache/kafka/common/internals/Topic.java@4.4.0 GROUP_METADATA_TOPIC_NAME, TRANSACTION_STATE_TOPIC_NAME, SHARE_GROUP_STATE_TOPIC_NAME, CLUSTER_METADATA_TOPIC_NAME, INTERNAL_TOPICS, Topic.isInternal
impl TopicName {
    /// `__consumer_offsets`: committed offsets and classic group metadata. Internal.
    pub const CONSUMER_OFFSETS: TopicName = TopicName(Cow::Borrowed("__consumer_offsets"));

    /// `__transaction_state`: the transaction coordinator's log. Internal.
    pub const TRANSACTION_STATE: TopicName = TopicName(Cow::Borrowed("__transaction_state"));

    /// `__share_group_state`: share-group delivery state. Internal.
    pub const SHARE_GROUP_STATE: TopicName = TopicName(Cow::Borrowed("__share_group_state"));

    /// `__cluster_metadata`: the controller quorum's metadata log. Not
    /// [internal](Self::is_internal) in Kafka's sense: it is the quorum's own log, not a
    /// topic in the cluster's metadata, so clients never see it listed and the
    /// "internal" flag doesn't apply to it.
    pub const CLUSTER_METADATA: TopicName = TopicName(Cow::Borrowed("__cluster_metadata"));

    /// True for the internal topics Kafka manages and shows to clients as internal:
    /// [`CONSUMER_OFFSETS`](Self::CONSUMER_OFFSETS),
    /// [`TRANSACTION_STATE`](Self::TRANSACTION_STATE) and
    /// [`SHARE_GROUP_STATE`](Self::SHARE_GROUP_STATE). Not
    /// [`CLUSTER_METADATA`](Self::CLUSTER_METADATA), as in Kafka's `Topic.isInternal`.
    pub fn is_internal(&self) -> bool {
        [
            TopicName::CONSUMER_OFFSETS,
            TopicName::TRANSACTION_STATE,
            TopicName::SHARE_GROUP_STATE,
        ]
        .contains(self)
    }

    /// The name.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// True if the name contains `.` or `_`.
    ///
    /// Kafka's metric names can't tell the two apart, so `orders.eu` and `orders_eu`
    /// would report into the same metrics. Kafka therefore refuses to create a topic
    /// that collides with an existing one, and warns about names that could.
    // Parity: clients/src/main/java/org/apache/kafka/common/internals/Topic.java@4.4.0 Topic.hasCollisionChars, Topic.unifyCollisionChars, Topic.hasCollision
    pub fn has_collision_chars(&self) -> bool {
        self.0.contains(['.', '_'])
    }

    /// The name with every `.` replaced by `_`. Two names collide exactly when their
    /// keys are equal, so a set of keys finds collisions among many topics.
    pub fn collision_key(&self) -> String {
        self.0.replace('.', "_")
    }

    /// True if the two names differ only by `.` and `_` in the same positions,
    /// including when they are equal.
    pub fn collides_with(&self, other: &TopicName) -> bool {
        let unify = |byte: &u8| if *byte == b'.' { b'_' } else { *byte };
        self.0.len() == other.0.len()
            && self
                .0
                .bytes()
                .map(|b| unify(&b))
                .eq(other.0.as_bytes().iter().map(unify))
    }
}

// Parity: clients/src/main/java/org/apache/kafka/common/internals/Topic.java@4.4.0 Topic.detectInvalidTopic, Topic.containsValidPattern
pub(crate) fn validate(name: &str) -> Result<(), TopicNameError> {
    if name.is_empty() {
        return Err(TopicNameError::Empty);
    }
    if name == "." {
        return Err(TopicNameError::Dot);
    }
    if name == ".." {
        return Err(TopicNameError::DotDot);
    }
    // Kafka checks the length in UTF-16 code units before checking characters, so a
    // name of 200 non-ASCII characters is reported as an illegal character, not as too
    // long. There are never more code units than bytes, so count only when the bytes
    // exceed the limit.
    if name.len() > MAX_LENGTH {
        let length = name.encode_utf16().count();
        if length > MAX_LENGTH {
            return Err(TopicNameError::TooLong { length });
        }
    }
    match name
        .chars()
        .enumerate()
        .find(|(_, character)| !is_legal(*character))
    {
        Some((position, character)) => Err(TopicNameError::IllegalCharacter {
            character,
            position,
        }),
        None => Ok(()),
    }
}

fn is_legal(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
}

impl TryFrom<&str> for TopicName {
    type Error = TopicNameError;

    fn try_from(name: &str) -> Result<TopicName, TopicNameError> {
        validate(name)?;
        Ok(TopicName(Cow::Owned(name.to_owned())))
    }
}

impl TryFrom<String> for TopicName {
    type Error = TopicNameError;

    fn try_from(name: String) -> Result<TopicName, TopicNameError> {
        validate(&name)?;
        Ok(TopicName(Cow::Owned(name)))
    }
}

impl fmt::Display for TopicName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};

    use super::{MAX_LENGTH, TopicName, TopicNameError};
    use crate::strategies::{colliding_topic_names, invalid_topic_name, topic_name};

    fn reject(name: &str) -> TopicNameError {
        TopicName::try_from(name).unwrap_err()
    }

    fn name(text: &str) -> TopicName {
        TopicName::try_from(text).unwrap()
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/internals/TopicTest.java@4.4.0.

    #[test]
    fn should_accept_valid_topic_names() {
        let max_length = "x".repeat(249);
        for name in [
            "valid",
            "TOPIC",
            "nAmEs",
            "ar6",
            "VaL1d",
            "_0-9_.",
            "...",
            max_length.as_str(),
        ] {
            assert_eq!(TopicName::try_from(name).unwrap().as_str(), name);
        }
    }

    #[test]
    fn should_throw_on_invalid_topic_names() {
        let too_long = "a".repeat(250);
        for name in [
            "",
            "foo bar",
            "..",
            "foo:bar",
            "foo=bar",
            ".",
            too_long.as_str(),
        ] {
            assert!(TopicName::try_from(name).is_err(), "{name:?}");
        }
    }

    #[test]
    fn should_recognize_invalid_characters_in_topic_names() {
        for character in [
            '/', '\\', ',', '\u{0}', ':', '"', '\'', ';', '*', '?', ' ', '\t', '\r', '\n', '=',
        ] {
            // Kafka's test puts the character after "Is ", so the space at position 2
            // is the first illegal character; this checks the character alone too.
            assert!(TopicName::try_from(format!("Is {character}illegal")).is_err());
            assert_eq!(
                reject(&format!("ok{character}")),
                TopicNameError::IllegalCharacter {
                    character,
                    position: 2
                }
            );
        }
    }

    #[test]
    fn topic_has_collision_chars() {
        for topic in ["start", "end", "middle", "many"] {
            assert!(!name(topic).has_collision_chars(), "{topic}");
        }
        for topic in [
            ".start", "end.", "mid.dle", ".ma.ny.", "_start", "end_", "mid_dle", "_ma_ny.",
        ] {
            assert!(name(topic).has_collision_chars(), "{topic}");
        }
    }

    #[test]
    fn unify_collision_chars() {
        assert_eq!(name("topic").collision_key(), "topic");
        assert_eq!(name(".topic").collision_key(), "_topic");
        assert_eq!(name("_topic").collision_key(), "_topic");
        assert_eq!(name("_.topic").collision_key(), "__topic");
    }

    #[test]
    fn topic_has_collision() {
        let periods = [".topic", "to.pic", "topic.", "topic"];
        let underscores = ["_topic", "to_pic", "topic_", "topic"];
        // Self.
        for topic in periods.iter().chain(&underscores) {
            assert!(name(topic).collides_with(&name(topic)), "{topic}");
        }
        // Same position.
        for (period, underscore) in periods.iter().zip(&underscores) {
            assert!(
                name(period).collides_with(&name(underscore)),
                "{period} {underscore}"
            );
        }
        // Different position.
        for (period, underscore) in periods.iter().zip(underscores.iter().rev()) {
            assert!(
                !name(period).collides_with(&name(underscore)),
                "{period} {underscore}"
            );
        }
    }

    // End of the ported cases.

    #[test]
    fn exactly_three_topics_are_internal() {
        for name in [
            "__consumer_offsets",
            "__transaction_state",
            "__share_group_state",
        ] {
            assert!(TopicName::try_from(name).unwrap().is_internal(), "{name}");
        }
        for name in [
            "__cluster_metadata",
            "__consumer_offsets_",
            "__Consumer_offsets",
            "_consumer_offsets",
            "consumer_offsets",
            "orders",
        ] {
            assert!(!TopicName::try_from(name).unwrap().is_internal(), "{name}");
        }
    }

    #[test]
    fn the_constants_are_valid_names() {
        for constant in [
            TopicName::CONSUMER_OFFSETS,
            TopicName::TRANSACTION_STATE,
            TopicName::SHARE_GROUP_STATE,
            TopicName::CLUSTER_METADATA,
        ] {
            assert_eq!(TopicName::try_from(constant.as_str()), Ok(constant));
        }
    }

    #[test]
    fn each_rule_in_kafkas_order() {
        assert_eq!(reject(""), TopicNameError::Empty);
        assert_eq!(reject("."), TopicNameError::Dot);
        assert_eq!(reject(".."), TopicNameError::DotDot);
        // Too long and illegal: too long is checked first.
        assert_eq!(
            reject(&" ".repeat(250)),
            TopicNameError::TooLong { length: 250 }
        );
    }

    #[test]
    fn length_is_counted_in_utf16_code_units_like_java() {
        // 200 two-byte characters: 400 bytes but 200 code units, so the illegal
        // character is reported, as Kafka does.
        assert_eq!(
            reject(&"é".repeat(200)),
            TopicNameError::IllegalCharacter {
                character: 'é',
                position: 0
            }
        );
        // An emoji is two UTF-16 code units: 125 of them are 250 units.
        assert_eq!(
            reject(&"😀".repeat(125)),
            TopicNameError::TooLong { length: 250 }
        );
        assert!(matches!(
            reject(&"😀".repeat(124)),
            TopicNameError::IllegalCharacter { .. }
        ));
    }

    #[test]
    fn errors_print_the_rule_without_the_name() {
        assert_eq!(
            reject("a\nb").to_string(),
            r"character '\n' at position 1 is not an ASCII alphanumeric, '.', '_' or '-'"
        );
        assert_eq!(
            reject(&"a".repeat(300)).to_string(),
            "the name is 300 characters long, longer than the maximum 249"
        );
    }

    /// Kafka's `detectInvalidTopic`, written out again as an independent check.
    fn kafka_rule(name: &str) -> Option<TopicNameError> {
        let utf16_length = name.encode_utf16().count();
        if name.is_empty() {
            Some(TopicNameError::Empty)
        } else if name == "." {
            Some(TopicNameError::Dot)
        } else if name == ".." {
            Some(TopicNameError::DotDot)
        } else if utf16_length > 249 {
            Some(TopicNameError::TooLong {
                length: utf16_length,
            })
        } else {
            name.chars()
                .position(|c| !(c.is_ascii_alphanumeric() || "._-".contains(c)))
                .and_then(|position| {
                    Some(TopicNameError::IllegalCharacter {
                        character: name.chars().nth(position)?,
                        position,
                    })
                })
        }
    }

    proptest! {
        #[test]
        fn names_over_the_legal_alphabet_are_accepted(name in topic_name()) {
            let text = name.as_str().to_owned();
            prop_assert!((1..=MAX_LENGTH).contains(&text.len()));
            prop_assert_eq!(TopicName::try_from(text.as_str()), Ok(name.clone()));
            prop_assert_eq!(TopicName::try_from(text), Ok(name));
        }

        #[test]
        fn invalid_names_fail_the_rule_kafka_reports_first(name in invalid_topic_name()) {
            let expected = kafka_rule(&name);
            prop_assert!(expected.is_some(), "{:?} is valid", name);
            prop_assert_eq!(TopicName::try_from(name.as_str()).err(), expected);
        }

        #[test]
        fn names_collide_exactly_when_their_keys_are_equal(a in topic_name(), b in topic_name()) {
            prop_assert_eq!(a.collides_with(&b), a.collision_key() == b.collision_key());
            prop_assert!(a.collides_with(&a));
        }

        #[test]
        fn generated_colliding_pairs_collide((a, b) in colliding_topic_names()) {
            prop_assert!(a.collides_with(&b) && b.collides_with(&a));
            prop_assert_eq!(a.collision_key(), b.collision_key());
        }

        #[test]
        fn any_string_gets_kafkas_verdict(name in any::<String>()) {
            prop_assert_eq!(TopicName::try_from(name.as_str()).err(), kafka_rule(&name));
        }
    }
}
