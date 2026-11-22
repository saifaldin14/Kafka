//! Why bytes could not be decoded, or a value could not be encoded.

use std::fmt;

use crate::limits::Limit;

/// Why decoding failed, where in the input, and in which field.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{kind} at byte {offset}{}", FieldPath(.field_path))]
pub struct DecodeError {
    /// The offset in the input of the value that could not be decoded.
    pub offset: usize,
    /// The fields being decoded, outermost first. Empty for a value read on its own.
    pub field_path: Vec<&'static str>,
    /// What went wrong.
    pub kind: DecodeErrorKind,
}

impl DecodeError {
    pub(crate) fn new(offset: usize, kind: DecodeErrorKind) -> DecodeError {
        DecodeError {
            offset,
            field_path: Vec::new(),
            kind,
        }
    }

    /// The same error, inside the field `name`. Decoders call this as the error passes
    /// out through each field, so the path ends up outermost first.
    #[must_use]
    pub fn within(mut self, name: &'static str) -> DecodeError {
        self.field_path.insert(0, name);
        self
    }
}

/// Names the field a decode error happened in, as generated decoders do after each
/// read: `reader.read_i32().within("PartitionIndex")?`.
pub trait WithinField {
    /// The same result, with `field` added to the front of an error's field path.
    #[must_use]
    fn within(self, field: &'static str) -> Self;
}

impl<T> WithinField for Result<T, DecodeError> {
    fn within(self, field: &'static str) -> Self {
        self.map_err(|error| error.within(field))
    }
}

/// What went wrong while decoding.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum DecodeErrorKind {
    /// The input ended before the value did.
    #[error("needed {needed} byte(s), but only {remaining} remain")]
    Truncated {
        /// The bytes the value needs.
        needed: usize,
        /// The bytes left in the input.
        remaining: usize,
    },
    /// An unsigned varint whose 5th byte has its continuation bit set.
    #[error("varint is too long: the 5th byte has its continuation bit set")]
    VarintTooLong,
    /// A varlong whose 10th byte has its continuation bit set.
    #[error("varlong is too long: the 10th byte has its continuation bit set")]
    VarlongTooLong,
    /// A field that can't be null in this version has a null (negative) length.
    #[error("a non-nullable field was serialized as null")]
    UnexpectedNull,
    /// An array declares more elements than there are bytes left, so it can't be real.
    #[error("an array of {length} element(s) can't fit in the {remaining} byte(s) remaining")]
    ArrayLongerThanInput {
        /// The declared element count.
        length: usize,
        /// The bytes left in the input.
        remaining: usize,
    },
    /// A declared size above a [`Limit`]: one of the caller's
    /// [`DecodeLimits`](crate::DecodeLimits) or a fixed rule of the format. It is found
    /// before anything is allocated for it.
    #[error("{limit} {requested} exceeds the limit {allowed}")]
    LimitExceeded {
        /// The limit exceeded.
        limit: Limit,
        /// The size the input declared.
        requested: usize,
        /// The most the limit allows.
        allowed: usize,
    },
    /// A tagged-field count that is negative as a Java `int`.
    #[error("invalid negative number of tagged fields {count}")]
    NegativeTaggedFieldCount {
        /// The count, as Kafka reads it.
        count: i32,
    },
    /// More tagged fields than there are bytes left: each takes at least two.
    #[error("{count} tagged field(s) can't fit in the {remaining} byte(s) remaining")]
    TaggedFieldCountExceedsInput {
        /// The declared count.
        count: usize,
        /// The bytes left in the input.
        remaining: usize,
    },
    /// A tagged field whose size is negative as a Java `int`.
    #[error("tagged field {tag} has a negative size {size}")]
    NegativeTaggedFieldSize {
        /// The field's tag.
        tag: u32,
        /// The size, as Kafka reads it.
        size: i32,
    },
    /// A version the message doesn't have.
    #[error("can't read version {version} of {message}")]
    UnsupportedVersion {
        /// The message, as Kafka's schemas name it.
        message: &'static str,
        /// The version asked for.
        version: i16,
    },
    /// A tag the struct defines, in a version where that field isn't tagged.
    #[error("tag {tag} is not valid for version {version}")]
    TagNotValidForVersion {
        /// The tag.
        tag: u32,
        /// The version being read.
        version: i16,
    },
    /// A known tagged field whose value didn't take exactly its declared size. Kafka
    /// reads past or short of the declared size and fails later or not at all; this
    /// fails where the field is, and only for malformed input.
    #[error("tagged field {tag} declares {declared} byte(s), but its value took {read}")]
    TaggedFieldSizeMismatch {
        /// The field's tag.
        tag: u32,
        /// The size its header declared.
        declared: usize,
        /// The bytes its value took.
        read: usize,
    },
}

/// Why a value can't be encoded.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum EncodeError {
    /// A string longer than Kafka's 32767-byte limit.
    #[error("a string of {length} bytes is longer than the 32767 bytes Kafka allows")]
    StringTooLong {
        /// The string's length in UTF-8 bytes.
        length: usize,
    },
    /// Raw tagged fields out of order: each tag must be higher than the one before.
    #[error(
        "invalid raw tag field list: tag {tag} comes after tag {previous}, but is not higher than it"
    )]
    UnsortedRawTags {
        /// The tag out of order.
        tag: u32,
        /// The tag before it.
        previous: u32,
    },
    /// A byte array, records or array too long for its length prefix.
    #[error("a length of {length} doesn't fit in the length prefix")]
    LengthTooLong {
        /// The length.
        length: usize,
    },
}

/// Formats a field path as ` in A.B.C`, or nothing when empty.
struct FieldPath<'a>(&'a [&'static str]);

impl fmt::Display for FieldPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.0.is_empty() {
            write!(f, " in {}", self.0.join("."))?;
        }
        Ok(())
    }
}
