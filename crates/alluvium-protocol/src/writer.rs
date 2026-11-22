//! [`Writer`]: encodes Kafka's primitive types into a growable buffer.

use alluvium_types::Uuid;
use bytes::{BufMut, Bytes, BytesMut};

/// Appends Kafka primitives to a [`BytesMut`].
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/ByteBufferAccessor.java@4.4.0 ByteBufferAccessor
#[derive(Debug, Clone, Default)]
pub struct Writer {
    buf: BytesMut,
}

impl Writer {
    /// Appends one byte.
    pub(crate) fn put_u8(&mut self, byte: u8) {
        self.buf.put_u8(byte);
    }

    /// An empty writer.
    pub fn new() -> Writer {
        Writer::default()
    }

    /// The bytes written so far.
    pub fn into_bytes(self) -> Bytes {
        self.buf.freeze()
    }

    /// An `int8`.
    pub fn write_i8(&mut self, value: i8) {
        self.buf.put_i8(value);
    }

    /// An `int16`, big-endian.
    pub fn write_i16(&mut self, value: i16) {
        self.buf.put_i16(value);
    }

    /// An `int32`, big-endian.
    pub fn write_i32(&mut self, value: i32) {
        self.buf.put_i32(value);
    }

    /// An `int64`, big-endian.
    pub fn write_i64(&mut self, value: i64) {
        self.buf.put_i64(value);
    }

    /// A `uint16`, big-endian.
    pub fn write_u16(&mut self, value: u16) {
        self.buf.put_u16(value);
    }

    /// A `uint32`, big-endian.
    pub fn write_u32(&mut self, value: u32) {
        self.buf.put_u32(value);
    }

    /// A `float64`, bit for bit.
    pub fn write_f64(&mut self, value: f64) {
        self.buf.put_u64(value.to_bits());
    }

    /// A `bool` as `0x01` or `0x00`, so a non-zero byte other than 1 read from the wire
    /// is written back as `0x01`.
    pub fn write_bool(&mut self, value: bool) {
        self.buf.put_u8(u8::from(value));
    }

    /// A `uuid`: the most significant 64 bits, then the least.
    pub fn write_uuid(&mut self, value: Uuid) {
        self.buf.put_i64(value.most_significant_bits());
        self.buf.put_i64(value.least_significant_bits());
    }
}
