//! [`RecordsField`]: a `records` field's batches, in memory and never copied out of the
//! frame they arrived in, or deferred to the network layer, which sends them from where
//! they are stored.

use std::fmt;

use alluvium_types::PayloadSummary;
use bytes::Bytes;

/// The record batches in a `records` field.
///
/// Decoding gives [`RecordsField::Memory`], a slice of the frame that carried the
/// batches: it never copies record payloads, so holding one keeps the whole frame alive
/// ([`Reader::retained_bytes`](crate::Reader::retained_bytes)).
///
/// A response the broker builds may instead hold [`RecordsField::Deferred`] batches,
/// such as a region of a segment file, which it never reads into memory. The encoder
/// writes their length and leaves a placeholder of that length in the frame for the
/// network layer to fill ([`Segment::Deferred`](crate::Segment::Deferred)). The protocol
/// crate does no I/O: it only carries the length and the caller's token.
///
/// `Debug` prints a [`PayloadSummary`], never the records themselves, which may hold
/// personal or secret data.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java@4.4.0 Readable.readRecords
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/SendBuilder.java@4.4.0 SendBuilder.writeRecords
#[derive(Clone, PartialEq, Eq)]
pub enum RecordsField {
    /// Batches in memory, as Kafka's `MemoryRecords`.
    Memory(Bytes),
    /// `len` bytes of batches the network layer sends from where they are stored, as
    /// Kafka sends `FileRecords` with their own `Send`. `token` is the caller's: it says
    /// which batches these are, such as an index into the file regions a fetch read.
    Deferred {
        /// The batches' size, in bytes.
        len: usize,
        /// Which batches, as the caller numbers them.
        token: u64,
    },
}

impl RecordsField {
    /// Record batches held in `bytes`.
    pub fn new(bytes: Bytes) -> RecordsField {
        RecordsField::Memory(bytes)
    }

    /// The record batches' bytes, if they are in memory.
    pub fn as_bytes(&self) -> Option<&Bytes> {
        match self {
            RecordsField::Memory(bytes) => Some(bytes),
            RecordsField::Deferred { .. } => None,
        }
    }

    /// The size of the record batches, in bytes.
    pub fn len(&self) -> usize {
        match self {
            RecordsField::Memory(bytes) => bytes.len(),
            RecordsField::Deferred { len, .. } => *len,
        }
    }

    /// Whether the field holds no bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl fmt::Debug for RecordsField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordsField::Memory(bytes) => write!(f, "RecordsField({:?})", PayloadSummary(bytes)),
            RecordsField::Deferred { len, token } => {
                write!(f, "RecordsField(Deferred {{ len: {len}, token: {token} }})")
            }
        }
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
        let deferred = RecordsField::Deferred { len: 10, token: 3 };
        assert_eq!(
            format!("{deferred:?}"),
            "RecordsField(Deferred { len: 10, token: 3 })"
        );
        assert_eq!((deferred.len(), deferred.as_bytes()), (10, None));
    }
}
