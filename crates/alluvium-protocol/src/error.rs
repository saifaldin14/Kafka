//! Why bytes could not be decoded.

use std::fmt;

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
