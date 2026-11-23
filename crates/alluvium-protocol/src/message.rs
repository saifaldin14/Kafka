//! [`Message`]: what every generated request, response, header and data type can do.

use bytes::Bytes;

use crate::error::{DecodeError, DecodeErrorKind, EncodeError};
use crate::json::JsonWriter;
use crate::reader::Reader;
use crate::writer::Writer;

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

    /// Writes the message at `version` to `writer`.
    ///
    /// Fields absent from `version` must hold their defaults unless they are
    /// `ignorable`, and tagged fields are written only when they differ from their
    /// defaults, with the unknown ones in tag order.
    ///
    /// # Errors
    ///
    /// [`EncodeError::UnsupportedVersion`] for a version the message doesn't have, and
    /// the other [`EncodeError`]s for a value the version can't carry.
    fn encode(&self, writer: &mut Writer, version: i16) -> Result<(), EncodeError>;

    /// The size of the message at `version`: what [`Message::encode`] would write.
    ///
    /// # Errors
    ///
    /// As [`Message::encode`].
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/Message.java@4.4.0 Message.size
    fn encoded_size(&self, version: i16) -> Result<EncodedSize, EncodeError> {
        let mut writer = Writer::counting();
        self.encode(&mut writer, version)?;
        Ok(EncodedSize {
            total: writer.len(),
            zero_copy: writer.zero_copy_len(),
        })
    }

    /// Writes the message at `version`, the version it was read at, as JSON in Kafka's
    /// `JsonConverter` form ([`JsonWriter`]).
    // Parity: generator/src/main/java/org/apache/kafka/message/JsonConverterGenerator.java@4.4.0 JsonConverterGenerator.generateWrite
    fn write_json(&self, json: &mut JsonWriter, version: i16);

    /// The message at `version` as one line of JSON ([`Message::write_json`]).
    fn to_json(&self, version: i16) -> String {
        let mut json = JsonWriter::new();
        self.write_json(&mut json, version);
        json.finish()
    }
}

/// The size of an encoded message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodedSize {
    /// Every byte the message takes.
    pub total: usize,
    /// The bytes among them of record batches and zero-copy byte fields, which a frame
    /// can share instead of copying (PROTO-F08).
    pub zero_copy: usize,
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

/// `version` as an `int16`, then `message` at that version: how Kafka stores data types
/// such as control record keys and consumer protocol payloads.
///
/// # Errors
///
/// As [`Message::encode`].
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java@4.4.0 MessageUtil.toVersionPrefixedBytes
pub fn encode_version_prefixed<M: Message>(
    message: &M,
    version: i16,
) -> Result<Bytes, EncodeError> {
    let mut writer = Writer::new();
    writer.write_i16(version);
    message.encode(&mut writer, version)?;
    Ok(writer.into_bytes())
}

/// An `int16` version, then a message at that version, as
/// [`encode_version_prefixed`] writes them.
///
/// # Errors
///
/// As [`Message::decode`]: a version the message doesn't have, negative ones included,
/// is [`DecodeErrorKind::UnsupportedVersion`].
pub fn decode_version_prefixed<M: Message>(reader: &mut Reader) -> Result<(i16, M), DecodeError> {
    let version = reader.read_i16()?;
    let message = M::decode(reader, version)?;
    Ok((version, message))
}
