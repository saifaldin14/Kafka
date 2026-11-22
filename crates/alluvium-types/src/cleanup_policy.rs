//! `cleanup.policy`: whether a topic's old segments are deleted, compacted, both or
//! neither.

use std::collections::BTreeSet;
use std::fmt;

use crate::LogSafe;

/// Which cleanups apply to a topic: any combination of delete and compact.
///
/// Built from the config's list of tokens. Neither (the empty list) means infinite
/// retention: segments are never deleted or compacted. Splitting the config string
/// into tokens is the config parser's job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CleanupPolicy {
    delete: bool,
    compact: bool,
}

/// Why a list of tokens is not a cleanup policy, checked in Kafka's order: duplicates
/// first, then each token in turn.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CleanupPolicyError {
    /// The same token appears twice (compared exactly, before tokens are checked).
    #[error("cleanup.policy values must not be duplicated: \"{}\"", LogSafe(.token))]
    Duplicate {
        /// The repeated token.
        token: String,
    },
    /// An empty token, as in `delete,,compact`.
    #[error("cleanup.policy values must not be empty")]
    EmptyToken,
    /// A token other than `delete` or `compact`, compared exactly, so `Compact` is
    /// unknown.
    #[error("cleanup.policy value \"{}\" must be one of: compact, delete", LogSafe(.token))]
    UnknownToken {
        /// The rejected token.
        token: String,
    },
}

impl CleanupPolicy {
    /// `delete`, Kafka's default.
    pub const DELETE: CleanupPolicy = CleanupPolicy {
        delete: true,
        compact: false,
    };

    /// `compact`.
    pub const COMPACT: CleanupPolicy = CleanupPolicy {
        delete: false,
        compact: true,
    };

    /// `delete,compact`: old segments are deleted by retention, and the rest compacted.
    pub const DELETE_AND_COMPACT: CleanupPolicy = CleanupPolicy {
        delete: true,
        compact: true,
    };

    /// The empty list: no cleanup, infinite retention.
    pub const NONE: CleanupPolicy = CleanupPolicy {
        delete: false,
        compact: false,
    };

    /// The policy a list of tokens describes, in any order.
    ///
    /// # Errors
    ///
    /// [`CleanupPolicyError`] for a duplicate, empty or unknown token.
    // Parity: clients/src/main/java/org/apache/kafka/common/config/ConfigDef.java@4.4.0 ConfigDef.ValidList.ensureValid, ConfigDef.ValidString.ensureValid
    // Parity: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java@4.4.0 (cleanup.policy defined with ValidList.in(compact, delete))
    pub fn from_tokens<T: AsRef<str>>(tokens: &[T]) -> Result<CleanupPolicy, CleanupPolicyError> {
        let mut seen = BTreeSet::new();
        for token in tokens {
            let token = token.as_ref();
            if !seen.insert(token) {
                return Err(CleanupPolicyError::Duplicate {
                    token: token.to_owned(),
                });
            }
        }
        let mut policy = CleanupPolicy::NONE;
        for token in tokens {
            match token.as_ref() {
                "delete" => policy.delete = true,
                "compact" => policy.compact = true,
                "" => return Err(CleanupPolicyError::EmptyToken),
                other => {
                    return Err(CleanupPolicyError::UnknownToken {
                        token: other.to_owned(),
                    });
                }
            }
        }
        Ok(policy)
    }

    /// True if retention deletes old segments.
    pub fn deletes(self) -> bool {
        self.delete
    }

    /// True if the log is compacted.
    pub fn compacts(self) -> bool {
        self.compact
    }
}

/// The canonical list: `delete`, `compact`, `delete,compact`, or nothing for the empty
/// list.
impl fmt::Display for CleanupPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match (self.delete, self.compact) {
            (true, true) => "delete,compact",
            (true, false) => "delete",
            (false, true) => "compact",
            (false, false) => "",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{CleanupPolicy, CleanupPolicyError};

    fn parse(tokens: &[&str]) -> Result<CleanupPolicy, CleanupPolicyError> {
        CleanupPolicy::from_tokens(tokens)
    }

    #[test]
    fn each_accepted_list() {
        let cases: [(&[&str], CleanupPolicy, &str); 5] = [
            (&["delete"], CleanupPolicy::DELETE, "delete"),
            (&["compact"], CleanupPolicy::COMPACT, "compact"),
            (
                &["delete", "compact"],
                CleanupPolicy::DELETE_AND_COMPACT,
                "delete,compact",
            ),
            (
                &["compact", "delete"],
                CleanupPolicy::DELETE_AND_COMPACT,
                "delete,compact",
            ),
            // The empty list: infinite retention (TopicConfig.CLEANUP_POLICY_DOC).
            (&[], CleanupPolicy::NONE, ""),
        ];
        for (tokens, policy, shown) in cases {
            assert_eq!(parse(tokens), Ok(policy), "{tokens:?}");
            assert_eq!(policy.to_string(), shown);
        }
        assert!(CleanupPolicy::DELETE_AND_COMPACT.deletes());
        assert!(CleanupPolicy::DELETE_AND_COMPACT.compacts());
        assert!(!CleanupPolicy::NONE.deletes() && !CleanupPolicy::NONE.compacts());
    }

    #[test]
    fn rejections_follow_kafkas_order() {
        let duplicate = |token: &str| {
            Err(CleanupPolicyError::Duplicate {
                token: token.to_owned(),
            })
        };
        let unknown = |token: &str| {
            Err(CleanupPolicyError::UnknownToken {
                token: token.to_owned(),
            })
        };
        assert_eq!(parse(&["delete", "delete"]), duplicate("delete"));
        // Duplicates are found before tokens are checked.
        assert_eq!(parse(&["x", "delete", "x"]), duplicate("x"));
        assert_eq!(parse(&["", ""]), duplicate(""));
        assert_eq!(parse(&["delete", ""]), Err(CleanupPolicyError::EmptyToken));
        assert_eq!(parse(&["Compact"]), unknown("Compact"));
        assert_eq!(parse(&["delete", "retain"]), unknown("retain"));
        assert_eq!(parse(&[" delete"]), unknown(" delete"));
    }

    #[test]
    fn rejected_tokens_print_log_safe() {
        assert_eq!(
            parse(&["delete\nERROR"]).unwrap_err().to_string(),
            r#"cleanup.policy value "delete\nERROR" must be one of: compact, delete"#
        );
    }
}
