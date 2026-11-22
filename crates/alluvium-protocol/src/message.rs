//! [`Message`]: what every generated request, response, header and data type can do.

use crate::error::{DecodeError, DecodeErrorKind};
use crate::reader::Reader;

/// A message generated from one of Kafka's schemas.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/Message.java@4.4.0 Message
pub trait Message: Sized {
    /// Reads the message at `version` from `reader`.
    ///
    /// Fields absent from `version` hold their defaults, and tagged fields this version
    /// doesn't define are kept in `unknown_tagged_fields`.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::UnsupportedVersion`] for a version the message doesn't have,
    /// and any error of the fields' decoders, with the path of the field it is in.
    fn decode(reader: &mut Reader, version: i16) -> Result<Self, DecodeError>;
}

impl Reader {
    /// The error for reading `message` at a `version` it doesn't have, at the current
    /// offset: Kafka's "Can't read version".
    pub fn unsupported_version(&self, message: &'static str, version: i16) -> DecodeError {
        Reader::error_at(
            self.offset(),
            DecodeErrorKind::UnsupportedVersion { message, version },
        )
    }
}
