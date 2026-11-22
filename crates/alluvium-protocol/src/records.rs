//! [`RecordsField`]: a `records` field's bytes, never copied out of the frame they
//! arrived in.

use std::fmt;

use alluvium_types::PayloadSummary;
use bytes::Bytes;

/// The record batches in a `records` field, as a slice of the frame that carried them.
///
/// Decoding never copies record payloads: the slice shares the frame's allocation, so
/// holding a `RecordsField` keeps the whole frame alive. The network layer counts that
/// memory against the connection until the request is done.
///
/// `Debug` prints a [`PayloadSummary`], never the records themselves, which may hold
/// personal or secret data.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java@4.4.0 Readable.readRecords
#[derive(Clone, PartialEq, Eq)]
pub struct RecordsField(Bytes);

impl RecordsField {
    /// Record batches held in `bytes`.
    pub fn new(bytes: Bytes) -> RecordsField {
        RecordsField(bytes)
    }

    /// The record batches' bytes.
    pub fn as_bytes(&self) -> &Bytes {
        &self.0
    }

    /// The size of the record batches, in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the field holds no bytes.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for RecordsField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RecordsField({:?})", PayloadSummary(&self.0))
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::RecordsField;

    #[test]
    fn debug_prints_a_summary_not_the_records() {
        let records = RecordsField::new(Bytes::from_static(b"secret key=value"));
        let printed = format!("{records:?}");
        assert!(
            printed.starts_with("RecordsField(Bytes { len: 16, crc32c: "),
            "{printed}"
        );
        assert!(!printed.contains("secret"), "{printed}");
    }
}
