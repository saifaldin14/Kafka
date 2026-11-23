//! [`Writer`]: encodes Kafka's primitive types into a growable buffer, and
//! [`EncodedFrame`], what it wrote as segments for a vectored write.

use alluvium_types::Uuid;
use bytes::{BufMut, Bytes, BytesMut};

use crate::error::EncodeError;

/// Appends Kafka primitives to a [`BytesMut`], or only counts them.
///
/// A writer from [`Writer::sharing`] shares record batches and zero-copy byte fields
/// rather than copying them: each becomes a segment of its own that holds the caller's
/// [`Bytes`], and [`Writer::into_frame`] returns the segments for a vectored write, as
/// Kafka's `SendBuilder` does. A writer from [`Writer::new`] copies them, which is
/// cheaper when the caller wants one buffer ([`Writer::into_bytes`]).
///
/// A counting writer, from [`Writer::counting`], stores nothing: it adds up the bytes
/// each write would append. `Message::encoded_size` encodes into one, so the size it
/// reports is the length `encode` writes by construction. Both kinds also count the
/// bytes of record batches and zero-copy byte fields.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/ByteBufferAccessor.java@4.4.0 ByteBufferAccessor
#[derive(Debug, Clone, Default)]
pub struct Writer {
    /// The bytes the writer encoded.
    buf: BytesMut,
    /// The values shared so far, for a [`Writer::sharing`] writer. Boxed so that a
    /// copying writer, which most encodes use, stays small.
    shared: Option<Box<Shared>>,
    /// Whether bytes are only counted.
    counting: bool,
    /// Whether this is a [`Writer::sizing`] writer, which skips the checks Kafka makes
    /// only while writing.
    sizing: bool,
    /// The bytes counted, when counting.
    counted: usize,
    /// The bytes of record batches and zero-copy byte fields written.
    zero_copy: usize,
}

/// The record batches and zero-copy values a sharing writer holds.
#[derive(Debug, Clone, Default)]
struct Shared {
    /// Each value, with the length `buf` had when it was written: the frame is `buf` with
    /// each value inserted at its offset.
    values: Vec<(usize, Segment)>,
    /// Their total length.
    len: usize,
}

impl Writer {
    /// Appends one byte.
    pub(crate) fn put_u8(&mut self, byte: u8) {
        if self.counting {
            self.counted = self.counted.saturating_add(1);
        } else {
            self.buf.put_u8(byte);
        }
    }

    /// Appends bytes as they are.
    pub(crate) fn put_slice(&mut self, bytes: &[u8]) {
        if self.counting {
            self.counted = self.counted.saturating_add(bytes.len());
        } else {
            self.buf.put_slice(bytes);
        }
    }

    /// Appends what `write` puts in the buffer, or, when counting, adds `size()`: for a
    /// value written a byte at a time, so the mode is checked once per value rather than
    /// once per byte.
    pub(crate) fn put_with(
        &mut self,
        size: impl FnOnce() -> usize,
        write: impl FnOnce(&mut BytesMut),
    ) {
        if self.counting {
            self.counted = self.counted.saturating_add(size());
        } else {
            write(&mut self.buf);
        }
    }

    /// Appends `bytes`, a record batch or zero-copy value: shared rather than copied by a
    /// [`Writer::sharing`] writer.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/SendBuilder.java@4.4.0 SendBuilder.writeByteBuffer, SendBuilder.writeRecords
    pub(crate) fn put_shared(&mut self, bytes: &Bytes) {
        self.zero_copy = self.zero_copy.saturating_add(bytes.len());
        match &mut self.shared {
            Some(shared) if !bytes.is_empty() => {
                shared.len = shared.len.saturating_add(bytes.len());
                shared
                    .values
                    .push((self.buf.len(), Segment::Bytes(bytes.clone())));
            }
            _ => self.put_slice(bytes),
        }
    }

    /// Appends a placeholder for `len` bytes of record batches the network layer sends
    /// from where they are stored, identified by the caller's `token`.
    ///
    /// # Errors
    ///
    /// [`EncodeError::DeferredRecordsNotInFrame`] unless this is a [`Writer::sharing`]
    /// or counting writer: the batches aren't in memory to copy, as Kafka's `Writable`
    /// refuses records that aren't `MemoryRecords`.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/Writable.java@4.4.0 Writable.writeRecords
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/SendBuilder.java@4.4.0 SendBuilder.writeRecords
    pub(crate) fn put_deferred(&mut self, len: usize, token: u64) -> Result<(), EncodeError> {
        if self.counting {
            self.counted = self.counted.saturating_add(len);
        } else {
            let shared = self
                .shared
                .as_mut()
                .ok_or(EncodeError::DeferredRecordsNotInFrame { len })?;
            shared.len = shared.len.saturating_add(len);
            shared
                .values
                .push((self.buf.len(), Segment::Deferred { len, token }));
        }
        self.zero_copy = self.zero_copy.saturating_add(len);
        Ok(())
    }

    /// An empty writer.
    pub fn new() -> Writer {
        Writer::default()
    }

    /// An empty writer that shares record batches and zero-copy byte values with the
    /// message instead of copying them, for a frame sent with a vectored write
    /// ([`Writer::into_frame`]). It is the only kind that writes deferred record batches.
    pub fn sharing() -> Writer {
        Writer {
            shared: Some(Box::default()),
            ..Writer::default()
        }
    }

    /// A writer that counts the bytes written instead of storing them.
    pub fn counting() -> Writer {
        Writer {
            counting: true,
            ..Writer::default()
        }
    }

    /// A counting writer for Kafka's sizing pass, which a failed encode reruns to report
    /// the error Kafka would.
    ///
    /// Kafka computes a message's size before writing it, and the sizing pass finds some
    /// problems (unknown tagged fields where the version has none, a null where it is not
    /// nullable, a string too long) while the write finds others (a non-default field the
    /// version lacks). Where a message has both, Kafka reports the sizing problem.
    /// Generated encoders check only what the sizing pass checks when writing into this
    /// writer.
    pub fn sizing() -> Writer {
        Writer {
            counting: true,
            sizing: true,
            ..Writer::default()
        }
    }

    /// Whether this is a [`Writer::sizing`] writer.
    pub fn is_sizing(&self) -> bool {
        self.sizing
    }

    /// An empty writer of the same kind, for a value whose size must be known before it
    /// is written.
    pub(crate) fn nested(&self) -> Writer {
        Writer {
            counting: self.counting,
            sizing: self.sizing,
            shared: self.shared.as_ref().map(|_| Box::default()),
            ..Writer::default()
        }
    }

    /// Appends everything `other` wrote, sharing what it shared.
    pub(crate) fn append(&mut self, other: &Writer) {
        self.zero_copy = self.zero_copy.saturating_add(other.zero_copy);
        if self.counting {
            self.counted = self.counted.saturating_add(other.len());
            return;
        }
        let base = self.buf.len();
        self.buf.put_slice(&other.buf);
        if let Some(theirs) = other
            .shared
            .as_deref()
            .filter(|theirs| !theirs.values.is_empty())
        {
            let ours = self.shared.get_or_insert_with(Box::default);
            ours.len = ours.len.saturating_add(theirs.len);
            ours.values.extend(
                theirs
                    .values
                    .iter()
                    .map(|(offset, bytes)| (base.saturating_add(*offset), bytes.clone())),
            );
        }
    }

    /// The bytes written, or counted, so far.
    pub fn len(&self) -> usize {
        if self.counting {
            self.counted
        } else {
            self.buf.len().saturating_add(self.shared_len())
        }
    }

    /// The bytes of the values shared so far.
    fn shared_len(&self) -> usize {
        self.shared.as_ref().map_or(0, |shared| shared.len)
    }

    /// Whether nothing has been written.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The bytes of record batches and zero-copy byte fields among them.
    pub fn zero_copy_len(&self) -> usize {
        self.zero_copy
    }

    /// The bytes written so far, in one buffer; none for a counting writer. Shared values
    /// are copied into it; [`Writer::into_frame`] keeps them shared.
    ///
    /// # Panics
    ///
    /// If a sharing writer holds deferred record batches, which only the network layer
    /// can read: send such a message with [`Writer::into_frame`]. Decoding never gives
    /// deferred batches, so only the caller's own message can hold them.
    #[expect(
        clippy::panic,
        reason = "deferred batches are a caller's own value, never input; their bytes don't exist here"
    )]
    pub fn into_bytes(self) -> Bytes {
        let len = self.len();
        let Some(shared) = self.shared.filter(|shared| !shared.values.is_empty()) else {
            return self.buf.freeze();
        };
        // Checked before allocating the whole length, deferred batches included.
        let values: Vec<(usize, &Bytes)> = shared
            .values
            .iter()
            .map(|(offset, value)| match value {
                Segment::Bytes(value) => (*offset, value),
                Segment::Deferred { .. } => {
                    panic!("into_bytes of a writer holding deferred record batches; use into_frame")
                }
            })
            .collect();
        let mut bytes = BytesMut::with_capacity(len);
        let mut owned = 0;
        for (offset, value) in values {
            bytes.put_slice(self.buf.get(owned..offset).unwrap_or_default());
            bytes.put_slice(value);
            owned = offset;
        }
        bytes.put_slice(self.buf.get(owned..).unwrap_or_default());
        bytes.freeze()
    }

    /// The bytes written so far as segments: one for a copying writer, and for a
    /// [`Writer::sharing`] one, each shared value with the owned bytes between them; none
    /// for a counting writer.
    pub fn into_frame(self) -> EncodedFrame {
        let len = self.buf.len().saturating_add(self.shared_len());
        let buf = self.buf.freeze();
        let values = self.shared.map(|shared| shared.values).unwrap_or_default();
        let mut segments = Vec::with_capacity(values.len().saturating_mul(2).saturating_add(1));
        let mut owned = 0;
        for (offset, value) in values {
            if offset > owned {
                segments.push(Segment::Bytes(buf.slice(owned..offset)));
            }
            segments.push(value);
            owned = offset;
        }
        if buf.len() > owned {
            segments.push(Segment::Bytes(buf.slice(owned..)));
        }
        EncodedFrame { segments, len }
    }

    /// An `int8`.
    pub fn write_i8(&mut self, value: i8) {
        self.put_slice(&value.to_be_bytes());
    }

    /// An `int16`, big-endian.
    pub fn write_i16(&mut self, value: i16) {
        self.put_slice(&value.to_be_bytes());
    }

    /// An `int32`, big-endian.
    pub fn write_i32(&mut self, value: i32) {
        self.put_slice(&value.to_be_bytes());
    }

    /// An `int64`, big-endian.
    pub fn write_i64(&mut self, value: i64) {
        self.put_slice(&value.to_be_bytes());
    }

    /// A `uint16`, big-endian.
    pub fn write_u16(&mut self, value: u16) {
        self.put_slice(&value.to_be_bytes());
    }

    /// A `uint32`, big-endian.
    pub fn write_u32(&mut self, value: u32) {
        self.put_slice(&value.to_be_bytes());
    }

    /// A `float64`, bit for bit.
    pub fn write_f64(&mut self, value: f64) {
        self.put_slice(&value.to_bits().to_be_bytes());
    }

    /// A `bool` as `0x01` or `0x00`, so a non-zero byte other than 1 read from the wire
    /// is written back as `0x01`.
    pub fn write_bool(&mut self, value: bool) {
        self.put_u8(u8::from(value));
    }

    /// A `uuid`: the most significant 64 bits, then the least.
    pub fn write_uuid(&mut self, value: Uuid) {
        self.write_i64(value.most_significant_bits());
        self.write_i64(value.least_significant_bits());
    }
}

/// An encoded message as segments to send in order, for a vectored write: the bytes the
/// encoder wrote, and between them each record batch and zero-copy byte value, which
/// shares the message's buffer rather than copying it, or a placeholder for deferred
/// record batches.
///
/// Owned bytes between two shared values are one segment, so a frame has at most one
/// more owned segment than shared ones.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/SendBuilder.java@4.4.0 SendBuilder
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EncodedFrame {
    segments: Vec<Segment>,
    len: usize,
}

/// One part of an [`EncodedFrame`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// Bytes to send as they are.
    Bytes(Bytes),
    /// A placeholder for `len` bytes of [`RecordsField::Deferred`](crate::RecordsField::Deferred)
    /// batches: the network layer sends the batches `token` names in its place, and
    /// ends the connection if it can't send exactly `len` bytes of them, since the
    /// frame's length prefix promised that many.
    Deferred {
        /// The batches' size, in bytes.
        len: usize,
        /// The caller's token for the batches.
        token: u64,
    },
}

impl Segment {
    /// The segment's length in bytes.
    pub fn len(&self) -> usize {
        match self {
            Segment::Bytes(bytes) => bytes.len(),
            Segment::Deferred { len, .. } => *len,
        }
    }

    /// Whether the segment has no bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl EncodedFrame {
    /// The segments, in order.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// The frame's length in bytes: the sum of its segments.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the frame has no bytes.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::{EncodedFrame, Segment, Writer};
    use crate::error::EncodeError;

    /// The frame's segments as bytes, failing on a deferred one.
    fn bytes(frame: &EncodedFrame) -> Vec<&Bytes> {
        frame
            .segments()
            .iter()
            .map(|segment| match segment {
                Segment::Bytes(bytes) => bytes,
                Segment::Deferred { .. } => panic!("a deferred segment"),
            })
            .collect()
    }

    #[test]
    fn shared_values_are_segments_of_their_own_between_coalesced_owned_bytes() {
        let shared = Bytes::from_static(b"records");
        let mut writer = Writer::sharing();
        writer.write_i16(1);
        writer.write_i32(2);
        writer.put_shared(&shared);
        writer.put_shared(&Bytes::new());
        writer.put_shared(&shared);
        writer.write_i8(3);
        assert_eq!(writer.len(), 2 + 4 + 7 + 7 + 1);
        assert_eq!(writer.zero_copy_len(), 14);
        let frame = writer.clone().into_frame();
        let segments: Vec<&[u8]> = bytes(&frame).into_iter().map(|s| &s[..]).collect();
        assert_eq!(
            segments,
            [&[0, 1, 0, 0, 0, 2][..], b"records", b"records", &[3][..]]
        );
        assert_eq!(bytes(&frame)[1].as_ptr(), shared.as_ptr());
        assert_eq!(frame.len(), writer.len());
        let concatenated: Vec<u8> = bytes(&frame).into_iter().flatten().copied().collect();
        assert_eq!(writer.into_bytes(), concatenated);
    }

    #[test]
    fn appending_a_nested_writer_keeps_its_values_shared() {
        let shared = Bytes::from_static(b"value");
        let mut outer = Writer::sharing();
        outer.write_i8(1);
        let mut nested = outer.nested();
        nested.write_i8(2);
        nested.put_shared(&shared);
        nested.write_i8(3);
        outer.append(&nested);
        outer.write_i8(4);
        assert_eq!(outer.zero_copy_len(), 5);
        let frame = outer.into_frame();
        let segments: Vec<&[u8]> = bytes(&frame).into_iter().map(|s| &s[..]).collect();
        assert_eq!(segments, [&[1, 2][..], b"value", &[3, 4][..]]);
        assert_eq!(bytes(&frame)[1].as_ptr(), shared.as_ptr());
    }

    #[test]
    fn a_copying_writer_writes_one_segment() {
        let mut writer = Writer::new();
        writer.write_i8(1);
        writer.put_shared(&Bytes::from_static(b"value"));
        assert_eq!(writer.zero_copy_len(), 5);
        let frame = writer.into_frame();
        assert_eq!(
            frame.segments(),
            [Segment::Bytes(Bytes::from_static(b"\x01value"))]
        );
    }

    #[test]
    fn a_counting_writer_counts_shared_values_and_stores_nothing() {
        let mut writer = Writer::counting();
        writer.write_i32(1);
        writer.put_shared(&Bytes::from_static(b"abc"));
        assert_eq!((writer.len(), writer.zero_copy_len()), (7, 3));
        assert!(writer.into_frame().is_empty());
    }

    // PROTO-0103: deferred batches are a placeholder of their length in a frame, are
    // counted by a counting writer, and are refused by a copying one.
    #[test]
    fn deferred_records_are_a_placeholder_only_a_frame_holds() {
        let mut writer = Writer::sharing();
        writer.write_i8(1);
        writer.put_deferred(1000, 7).unwrap();
        writer.write_i8(2);
        assert_eq!((writer.len(), writer.zero_copy_len()), (1002, 1000));
        let frame = writer.into_frame();
        assert_eq!(
            frame.segments(),
            [
                Segment::Bytes(Bytes::from_static(&[1])),
                Segment::Deferred {
                    len: 1000,
                    token: 7
                },
                Segment::Bytes(Bytes::from_static(&[2])),
            ]
        );
        assert_eq!(frame.len(), 1002);
        let mut counting = Writer::counting();
        counting.put_deferred(1000, 7).unwrap();
        assert_eq!((counting.len(), counting.zero_copy_len()), (1000, 1000));
        let mut copying = Writer::new();
        assert_eq!(
            copying.put_deferred(1000, 7),
            Err(EncodeError::DeferredRecordsNotInFrame { len: 1000 })
        );
        assert_eq!((copying.len(), copying.zero_copy_len()), (0, 0));
    }

    #[test]
    #[should_panic(expected = "into_bytes of a writer holding deferred record batches")]
    fn deferred_records_have_no_bytes_to_copy() {
        let mut writer = Writer::sharing();
        writer.put_deferred(1, 0).unwrap();
        let _ = writer.into_bytes();
    }
}
