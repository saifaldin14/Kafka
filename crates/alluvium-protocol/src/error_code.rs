//! [`ErrorCode`]'s hand-written half: reading a code from the wire, and printing it.
//! The enum and its per-code tables are generated from Kafka's `Errors.java` into
//! `generated::error_code`.

use std::fmt;

pub use crate::generated::error_code::ErrorCode;

/// An error code Kafka 4.4.0 doesn't define, as read from the wire.
///
/// Kafka's `Errors.forCode` logs such a code and treats it as `UNKNOWN_SERVER_ERROR`.
/// [`ErrorCode::from_code`] returns the raw value instead, so the caller logs it where
/// it has the context, and [`UnknownErrorCode::error_code`] gives Kafka's fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unknown error code {code}")]
pub struct UnknownErrorCode {
    /// The code as read.
    pub code: i16,
}

impl UnknownErrorCode {
    /// The error Kafka reads an unknown code as: `UNKNOWN_SERVER_ERROR`.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/Errors.java@4.4.0 Errors.forCode
    pub const fn error_code(self) -> ErrorCode {
        ErrorCode::UnknownServerError
    }
}

impl ErrorCode {
    /// The error with wire code `code`.
    ///
    /// # Errors
    ///
    /// [`UnknownErrorCode`], carrying `code`, if Kafka 4.4.0 defines no such code.
    pub fn from_code(code: i16) -> Result<ErrorCode, UnknownErrorCode> {
        ErrorCode::ALL
            .binary_search_by_key(&code, |error| error.code())
            .ok()
            .and_then(|index| ErrorCode::ALL.get(index))
            .copied()
            .ok_or(UnknownErrorCode { code })
    }
}

/// `NOT_LEADER_OR_FOLLOWER (6)`. In logs and events, record a code as two fields,
/// `error_code = error.code()` and `error_name = error.name()`
/// (docs/standards/debuggability.md).
impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name(), self.code())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{ErrorCode, UnknownErrorCode};

    #[test]
    fn every_code_round_trips_and_is_listed_in_code_order() {
        assert_eq!(ErrorCode::ALL.len(), 138);
        for (error, code) in ErrorCode::ALL.into_iter().zip(-1_i16..) {
            assert_eq!(error.code(), code);
            assert_eq!(ErrorCode::from_code(code), Ok(error));
        }
    }

    proptest! {
        #[test]
        fn a_code_is_known_exactly_when_kafka_defines_it(code in any::<i16>()) {
            let decoded = ErrorCode::from_code(code);
            if (-1..=136).contains(&code) {
                prop_assert_eq!(decoded.map(ErrorCode::code), Ok(code));
            } else {
                prop_assert_eq!(decoded, Err(UnknownErrorCode { code }));
            }
        }
    }

    #[test]
    fn an_unknown_code_falls_back_to_unknown_server_error_as_in_kafka() {
        let unknown = ErrorCode::from_code(137).unwrap_err();
        assert_eq!(unknown.code, 137);
        assert_eq!(unknown.error_code(), ErrorCode::UnknownServerError);
        assert_eq!(unknown.to_string(), "unknown error code 137");
    }

    #[test]
    fn display_is_the_name_and_code() {
        assert_eq!(
            ErrorCode::NotLeaderOrFollower.to_string(),
            "NOT_LEADER_OR_FOLLOWER (6)"
        );
        assert_eq!(ErrorCode::None.to_string(), "NONE (0)");
    }

    #[test]
    fn the_tables_agree_with_kafka_on_well_known_codes() {
        let not_leader = ErrorCode::NotLeaderOrFollower;
        assert!(not_leader.is_retriable() && not_leader.invalidates_metadata());
        assert!(!ErrorCode::TopicAuthorizationFailed.is_retriable());
        assert_eq!(ErrorCode::None.default_message(), None);
        assert_eq!(
            ErrorCode::RequestTimedOut.default_message(),
            Some("The request timed out.")
        );
    }
}
