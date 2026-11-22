//! Rust identifiers for Kafka's names: fields and modules in `snake_case`, with Rust
//! keywords suffixed, and the checks that keep two names from becoming one.

use std::collections::BTreeMap;

/// Every Rust keyword, strict and reserved, in the 2024 edition. A Kafka name that maps
/// to one gets [`KEYWORD_SUFFIX`].
const KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl",
    "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// What a keyword gets so that it can be a field name: `Type` becomes `type_`.
const KEYWORD_SUFFIX: &str = "_";

/// Names the generated code uses without a path, which a schema's struct can't take.
const RESERVED_STRUCT_NAMES: &[&str] = &[
    "Option",
    "Some",
    "None",
    "Vec",
    "String",
    "Box",
    "Result",
    "Ok",
    "Err",
    "Default",
    "Clone",
    "Copy",
    "Debug",
    "PartialEq",
    "Eq",
    "Self",
];

/// The field every struct in a flexible version gets, which a schema field can't take.
pub(crate) const UNKNOWN_TAGGED_FIELDS: &str = "unknown_tagged_fields";

/// `name` in `snake_case`. An underscore goes before an upper-case letter that follows
/// a lower-case letter or digit, or that starts a word after an acronym:
/// `TransactionalId` → `transactional_id`, `KRaftVersion` → `k_raft_version`.
pub(crate) fn snake_case(name: &str) -> String {
    let characters: Vec<char> = name.chars().collect();
    let mut snake = String::with_capacity(name.len() + 4);
    for (index, &character) in characters.iter().enumerate() {
        let previous = index.checked_sub(1).and_then(|at| characters.get(at));
        if let (true, Some(previous)) = (character.is_ascii_uppercase(), previous) {
            let next_is_lower = characters
                .get(index + 1)
                .is_some_and(char::is_ascii_lowercase);
            if previous.is_ascii_lowercase()
                || previous.is_ascii_digit()
                || (previous.is_ascii_uppercase() && next_is_lower)
            {
                snake.push('_');
            }
        }
        snake.push(character.to_ascii_lowercase());
    }
    snake
}

/// A field's or module's Rust name: `name` in `snake_case`, suffixed if a keyword.
pub(crate) fn identifier(name: &str) -> String {
    let snake = snake_case(name);
    if KEYWORDS.contains(&snake.as_str()) {
        snake + KEYWORD_SUFFIX
    } else {
        snake
    }
}

/// Fails if a struct name would shadow a name the generated code uses unqualified.
pub(crate) fn check_struct_name(name: &str) -> Result<(), String> {
    if RESERVED_STRUCT_NAMES.contains(&name) {
        return Err(format!(
            "struct {name} would shadow `{name}` in the generated code; the generator can't use \
             that name"
        ));
    }
    Ok(())
}

/// Fails if two of `names` (Kafka field names of one struct) map to the same Rust
/// identifier, or one maps to the field the generator adds.
pub(crate) fn check_collisions<'a>(names: impl IntoIterator<Item = &'a str>) -> Result<(), String> {
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for name in names {
        let rust = identifier(name);
        if rust == UNKNOWN_TAGGED_FIELDS {
            return Err(format!(
                "field {name} maps to `{rust}`, which the generator adds to every flexible struct"
            ));
        }
        if let Some(other) = seen.insert(rust.clone(), name) {
            return Err(format!("fields {other} and {name} both map to `{rust}`"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_collisions, check_struct_name, identifier, snake_case};

    #[test]
    fn kafka_names_become_snake_case() {
        for (kafka, rust) in [
            ("TransactionalId", "transactional_id"),
            ("ThrottleTimeMs", "throttle_time_ms"),
            ("KRaftVersion", "k_raft_version"),
            ("TopicID", "topic_id"),
            ("V1Field", "v1_field"),
            ("Ipv4Address", "ipv4_address"),
            ("A", "a"),
        ] {
            assert_eq!(snake_case(kafka), rust, "{kafka}");
        }
    }

    #[test]
    fn keywords_get_a_suffix() {
        assert_eq!(identifier("Type"), "type_");
        assert_eq!(identifier("Match"), "match_");
        assert_eq!(identifier("Matches"), "matches");
    }

    #[test]
    fn colliding_and_reserved_names_are_rejected() {
        assert!(check_collisions(["TopicId", "TopicID"]).is_err());
        assert!(check_collisions(["UnknownTaggedFields"]).is_err());
        assert!(check_collisions(["TopicId", "TopicName"]).is_ok());
        assert!(check_struct_name("Option").is_err());
        assert!(check_struct_name("TopicData").is_ok());
    }
}
