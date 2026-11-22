//! Namespaces: the tenant scope of topics, groups and transactional ids
//! ([ADR-0014](../../../docs/adr/0014-namespace-identity.md)).
//!
//! Every resource of a cluster without tenancy, and every name a Kafka client sends
//! without a namespace, is in [`NamespaceId::DEFAULT`]. A qualified name is a pair of a
//! namespace and a Kafka name, never a prefix, so Kafka's name rules and lengths apply
//! unchanged inside every namespace.

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use crate::LogSafe;

/// A namespace's id: 1 to 63 characters of `[a-z0-9-]`, starting with a letter and not
/// ending with `-`.
///
/// The alphabet can't forge a log line, so it prints as is.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamespaceId(Cow<'static, str>);

/// Why a string is not a namespace id: the first rule it breaks, in this order.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NamespaceIdError {
    /// The empty string.
    #[error("a namespace id can't be empty")]
    Empty,
    /// More than 63 characters.
    #[error("namespace id \"{}…\" is {length} characters long, longer than 63", LogSafe(.prefix))]
    TooLong {
        /// The first 63 characters.
        prefix: String,
        /// The length in characters.
        length: usize,
    },
    /// A character outside `[a-z0-9-]`.
    #[error(
        "character {character:?} at position {position} is not a lower-case ASCII letter, digit or '-'"
    )]
    IllegalCharacter {
        /// The first illegal character.
        character: char,
        /// Its position, counted in characters from 0.
        position: usize,
    },
    /// The first character is a digit or `-`.
    #[error("namespace id \"{}\" must start with a letter", LogSafe(.id))]
    NotStartingWithLetter {
        /// The rejected id.
        id: String,
    },
    /// The last character is `-`.
    #[error("namespace id \"{}\" must not end with '-'", LogSafe(.id))]
    EndingWithHyphen {
        /// The rejected id.
        id: String,
    },
}

const MAX_LENGTH: usize = 63;

impl NamespaceId {
    /// `default`: the namespace of every resource named without one. Never stored in
    /// metadata.
    pub const DEFAULT: NamespaceId = NamespaceId(Cow::Borrowed("default"));

    /// The id.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// True for [`NamespaceId::DEFAULT`].
    pub fn is_default(&self) -> bool {
        *self == NamespaceId::DEFAULT
    }
}

fn validate(id: &str) -> Result<(), NamespaceIdError> {
    let length = id.chars().count();
    if length == 0 {
        return Err(NamespaceIdError::Empty);
    }
    if length > MAX_LENGTH {
        return Err(NamespaceIdError::TooLong {
            prefix: id.chars().take(MAX_LENGTH).collect(),
            length,
        });
    }
    if let Some((position, character)) = id
        .chars()
        .enumerate()
        .find(|(_, c)| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-'))
    {
        return Err(NamespaceIdError::IllegalCharacter {
            character,
            position,
        });
    }
    if !id.starts_with(|c: char| c.is_ascii_lowercase()) {
        return Err(NamespaceIdError::NotStartingWithLetter { id: id.to_owned() });
    }
    if id.ends_with('-') {
        return Err(NamespaceIdError::EndingWithHyphen { id: id.to_owned() });
    }
    Ok(())
}

impl FromStr for NamespaceId {
    type Err = NamespaceIdError;

    fn from_str(id: &str) -> Result<NamespaceId, NamespaceIdError> {
        validate(id)?;
        Ok(NamespaceId(Cow::Owned(id.to_owned())))
    }
}

impl fmt::Display for NamespaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};

    use super::{NamespaceId, NamespaceIdError};
    use crate::strategies::namespace_id;

    #[test]
    fn default_round_trips_through_display_and_from_str() {
        assert_eq!(NamespaceId::DEFAULT.to_string(), "default");
        assert_eq!("default".parse(), Ok(NamespaceId::DEFAULT));
        assert!("default".parse::<NamespaceId>().unwrap().is_default());
        assert!(!"payments".parse::<NamespaceId>().unwrap().is_default());
    }

    #[test]
    fn each_rule_in_order() {
        let parse = |id: &str| id.parse::<NamespaceId>();
        assert!(parse("a").is_ok());
        assert!(parse("team-7").is_ok());
        assert!(parse(&format!("a{}", "b".repeat(62))).is_ok());
        assert_eq!(parse(""), Err(NamespaceIdError::Empty));
        assert!(matches!(
            parse(&"a".repeat(64)),
            Err(NamespaceIdError::TooLong { length: 64, .. })
        ));
        assert_eq!(
            parse("Team"),
            Err(NamespaceIdError::IllegalCharacter {
                character: 'T',
                position: 0
            })
        );
        assert_eq!(
            parse("a.b"),
            Err(NamespaceIdError::IllegalCharacter {
                character: '.',
                position: 1
            })
        );
        assert_eq!(
            parse("7team"),
            Err(NamespaceIdError::NotStartingWithLetter {
                id: "7team".to_owned()
            })
        );
        assert_eq!(
            parse("-team"),
            Err(NamespaceIdError::NotStartingWithLetter {
                id: "-team".to_owned()
            })
        );
        assert_eq!(
            parse("team-"),
            Err(NamespaceIdError::EndingWithHyphen {
                id: "team-".to_owned()
            })
        );
    }

    #[test]
    fn errors_print_log_safe_and_bounded() {
        let error = "x".repeat(10_000).parse::<NamespaceId>().unwrap_err();
        assert!(error.to_string().len() < 200, "{error}");
        assert!(
            !"a\nb"
                .parse::<NamespaceId>()
                .unwrap_err()
                .to_string()
                .contains('\n')
        );
    }

    /// The rules written out again as an independent check.
    fn valid(id: &str) -> bool {
        let bytes = id.as_bytes();
        (1..=63).contains(&id.chars().count())
            && bytes
                .iter()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
            && bytes.first().is_some_and(u8::is_ascii_lowercase)
            && bytes.last() != Some(&b'-')
    }

    proptest! {
        #[test]
        fn any_string_is_accepted_exactly_when_the_rules_hold(id in any::<String>()) {
            prop_assert_eq!(id.parse::<NamespaceId>().is_ok(), valid(&id));
        }

        #[test]
        fn short_strings_over_the_alphabet_match_the_rules(id in "[a-z0-9-]{0,66}") {
            prop_assert_eq!(id.parse::<NamespaceId>().is_ok(), valid(&id));
        }

        #[test]
        fn valid_ids_round_trip(id in namespace_id()) {
            prop_assert!(valid(id.as_str()));
            prop_assert_eq!(id.to_string().parse::<NamespaceId>(), Ok(id));
        }
    }
}
