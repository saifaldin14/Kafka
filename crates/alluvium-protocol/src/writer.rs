//! [`Writer`]: encodes Kafka's primitive types into a growable buffer.

use alluvium_types::Uuid;
use bytes::{BufMut, Bytes, BytesMut};

/// Appends Kafka primitives to a [`BytesMut`], or only counts them.
///
/// A counting writer, from [`Writer::counting`], stores nothing: it adds up the bytes
/// each write would append. `Message::encoded_size` encodes into one, so the size it
/// reports is the length `encode` writes by construction. Both kinds also count the
/// bytes of record batches and zero-copy byte fields, which a frame can share rather
/// than copy.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/ByteBufferAccessor.java@4.4.0 ByteBufferAccessor
#[derive(Debug, Clone, Default)]
pub struct Writer {
    buf: BytesMut,
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

    /// Appends `bytes`, a value a frame can share rather than copy.
    pub(crate) fn put_shared(&mut self, bytes: &[u8]) {
        self.zero_copy = self.zero_copy.saturating_add(bytes.len());
        self.put_slice(bytes);
    }

    /// An empty writer.
    pub fn new() -> Writer {
        Writer::default()
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
            ..Writer::default()
        }
    }

    /// Appends everything `other` wrote.
    pub(crate) fn append(&mut self, other: &Writer) {
        self.zero_copy = self.zero_copy.saturating_add(other.zero_copy);
        if self.counting {
            self.counted = self.counted.saturating_add(other.len());
        } else {
            self.buf.put_slice(&other.buf);
        }
    }

    /// The bytes written, or counted, so far.
    pub fn len(&self) -> usize {
        if self.counting {
            self.counted
        } else {
            self.buf.len()
        }
    }

    /// Whether nothing has been written.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The bytes of record batches and zero-copy byte fields among them.
    pub fn zero_copy_len(&self) -> usize {
        self.zero_copy
    }

    /// The bytes written so far; none for a counting writer.
    pub fn into_bytes(self) -> Bytes {
        self.buf.freeze()
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
