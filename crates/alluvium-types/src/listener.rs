//! Listener names: the names of the endpoints a node listens on, such as `PLAINTEXT`
//! or `CONTROLLER`, as used in `listeners` and `listener.security.protocol.map`.

use std::fmt;

/// A listener name, normalised to upper case.
///
/// Built only by [`ListenerName::normalised`], so names that differ only in case are
/// equal: `plaintext` and `PLAINTEXT` are one listener. After normalising, equality is
/// exact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListenerName(String);

/// The listener name was empty or blank.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the listener name is empty or blank")]
pub struct BlankListenerName;

// Parity: clients/src/main/java/org/apache/kafka/common/network/ListenerName.java@4.4.0 ListenerName.normalised
impl ListenerName {
    /// Upper-cases `value` as Java's `toUpperCase(Locale.ROOT)` does: full Unicode case
    /// mapping, the same in every locale, so `ß` becomes `SS`. Surrounding spaces are
    /// kept, as Kafka keeps them.
    ///
    /// # Errors
    ///
    /// [`BlankListenerName`] if `value` is blank in the sense of Kafka's
    /// `Utils.isBlank`: Java's `String.trim()` would leave nothing. `trim()` removes
    /// U+0000 to U+0020 only, so a value of other Unicode spaces is not blank.
    pub fn normalised(value: &str) -> Result<ListenerName, BlankListenerName> {
        // Parity: clients/src/main/java/org/apache/kafka/common/utils/Utils.java@4.4.0 Utils.isBlank
        if value.chars().all(|character| character <= ' ') {
            return Err(BlankListenerName);
        }
        Ok(ListenerName(value.to_uppercase()))
    }

    /// The normalised name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ListenerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{BlankListenerName, ListenerName};

    fn normalised(value: &str) -> String {
        ListenerName::normalised(value).unwrap().as_str().to_owned()
    }

    #[test]
    fn names_are_upper_cased() {
        assert_eq!(normalised("plaintext"), "PLAINTEXT");
        assert_eq!(normalised("Controller_1"), "CONTROLLER_1");
        assert_eq!(
            ListenerName::normalised("Plaintext"),
            ListenerName::normalised("PLAINTEXT")
        );
        assert_ne!(
            ListenerName::normalised("PLAINTEXT"),
            ListenerName::normalised("SSL")
        );
    }

    #[test]
    fn blank_means_what_java_trim_removes() {
        for blank in ["", " ", "\t\r\n", "\u{0}\u{1f} "] {
            assert_eq!(
                ListenerName::normalised(blank),
                Err(BlankListenerName),
                "{blank:?}"
            );
        }
        // U+3000 IDEOGRAPHIC SPACE is above U+0020, so Java's trim() keeps it and
        // Kafka accepts the name.
        assert_eq!(normalised("\u{3000}"), "\u{3000}");
        // Kafka doesn't trim the value it keeps.
        assert_eq!(normalised(" internal "), " INTERNAL ");
    }

    #[test]
    fn non_ascii_case_mapping_matches_java_locale_root() {
        // "straße".toUpperCase(Locale.ROOT) is "STRASSE" in Java: ß has no single upper
        // case letter. The dotless ı upper-cases to I in every locale.
        assert_eq!(normalised("straße"), "STRASSE");
        assert_eq!(normalised("ınternal"), "INTERNAL");
        assert_eq!(normalised("écoute"), "ÉCOUTE");
    }
}
