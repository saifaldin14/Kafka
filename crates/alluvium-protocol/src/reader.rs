//! [`Reader`]: decodes Kafka's primitive types from a buffer of untrusted bytes.

use alluvium_types::Uuid;
use bytes::{Buf, Bytes};

use crate::error::{DecodeError, DecodeErrorKind};
use crate::limits::{DecodeLimits, Limit};

/// Reads Kafka primitives from the front of a [`Bytes`] buffer.
///
/// Every read checks the bytes remaining before taking any, so a short input is a
/// [`DecodeErrorKind::Truncated`] at the offset where the value started, never a panic.
/// Values sliced out of the input, such as byte arrays, share its allocation. Declared
/// sizes are checked against the reader's [`DecodeLimits`] before anything is
/// allocated for them, and every allocation is charged to its budget.
///
/// It is not `Clone`: a clone would have its own copy of the allocation budget left.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/ByteBufferAccessor.java@4.4.0 ByteBufferAccessor
#[derive(Debug)]
pub struct Reader {
    pub(crate) buf: Bytes,
    /// The input's length, to report offsets from its start.
    pub(crate) total: usize,
    pub(crate) limits: DecodeLimits,
    /// The bytes of [`DecodeLimits::allocation_budget`] not yet charged.
    pub(crate) budget_left: usize,
}

impl Reader {
    /// A reader over all of `buf`, with Kafka's limits.
    pub fn new(buf: Bytes) -> Reader {
        Reader::with_limits(buf, DecodeLimits::default())
    }

    /// A reader over all of `buf`, with the caller's limits.
    pub fn with_limits(buf: Bytes, limits: DecodeLimits) -> Reader {
        let total = buf.len();
        Reader {
            buf,
            total,
            limits,
            budget_left: limits.allocation_budget,
        }
    }

    /// The offset of the next byte to read, from the start of the input.
    pub fn offset(&self) -> usize {
        self.total.saturating_sub(self.buf.len())
    }

    /// The number of bytes not yet read.
    pub fn remaining(&self) -> usize {
        self.buf.len()
    }

    /// An error of `kind` at `offset`.
    pub(crate) fn error_at(offset: usize, kind: DecodeErrorKind) -> DecodeError {
        DecodeError::new(offset, kind)
    }

    /// Charges an allocation of `bytes` to the budget, before it is made. A failure is
    /// reported at `start`, the offset of the value that needs it.
    pub(crate) fn charge(&mut self, start: usize, bytes: usize) -> Result<(), DecodeError> {
        self.budget_left = self.budget_left.checked_sub(bytes).ok_or_else(|| {
            Reader::error_at(
                start,
                DecodeErrorKind::LimitExceeded {
                    limit: Limit::AllocationBudget,
                    requested: bytes,
                    allowed: self.budget_left,
                },
            )
        })?;
        Ok(())
    }

    /// The next `len` bytes, sharing the input's allocation.
    pub(crate) fn take(&mut self, len: usize) -> Result<Bytes, DecodeError> {
        if len > self.buf.len() {
            return Err(Self::error_at(
                self.offset(),
                DecodeErrorKind::Truncated {
                    needed: len,
                    remaining: self.buf.len(),
                },
            ));
        }
        Ok(self.buf.split_to(len))
    }

    /// The next `N` bytes as an array, copied out of the input. Unlike [`Reader::take`],
    /// this makes no new handle on the input, so it costs no reference-count update.
    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let array: [u8; N] = self
            .buf
            .get(..N)
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or_else(|| {
                Self::error_at(
                    self.offset(),
                    DecodeErrorKind::Truncated {
                        needed: N,
                        remaining: self.buf.len(),
                    },
                )
            })?;
        // At least `N` bytes remain: `get` just returned them.
        self.buf.advance(N);
        Ok(array)
    }

    /// An `int8`.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if fewer than 1 byte remain.
    pub fn read_i8(&mut self) -> Result<i8, DecodeError> {
        self.array().map(i8::from_be_bytes)
    }

    /// An `int16`, big-endian.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if fewer than 2 bytes remain.
    pub fn read_i16(&mut self) -> Result<i16, DecodeError> {
        self.array().map(i16::from_be_bytes)
    }

    /// An `int32`, big-endian.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if fewer than 4 bytes remain.
    pub fn read_i32(&mut self) -> Result<i32, DecodeError> {
        self.array().map(i32::from_be_bytes)
    }

    /// An `int64`, big-endian.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if fewer than 8 bytes remain.
    pub fn read_i64(&mut self) -> Result<i64, DecodeError> {
        self.array().map(i64::from_be_bytes)
    }

    /// A `uint16`, big-endian.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if fewer than 2 bytes remain.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java@4.4.0 Readable.readUnsignedShort
    pub fn read_u16(&mut self) -> Result<u16, DecodeError> {
        self.array().map(u16::from_be_bytes)
    }

    /// A `uint32`, big-endian.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if fewer than 4 bytes remain.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java@4.4.0 Readable.readUnsignedInt
    pub fn read_u32(&mut self) -> Result<u32, DecodeError> {
        self.array().map(u32::from_be_bytes)
    }

    /// A `float64`: IEEE 754, big-endian, bit for bit (NaN payloads included).
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if fewer than 8 bytes remain.
    pub fn read_f64(&mut self) -> Result<f64, DecodeError> {
        self.array().map(f64::from_be_bytes)
    }

    /// A `bool`: any non-zero byte is true, as Kafka's generated `readByte() != 0` reads
    /// it.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if fewer than 1 byte remain.
    pub fn read_bool(&mut self) -> Result<bool, DecodeError> {
        self.array::<1>().map(|[byte]| byte != 0)
    }

    /// A `uuid`: the most significant 64 bits, then the least, each big-endian.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if fewer than 16 bytes remain.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java@4.4.0 Readable.readUuid
    pub fn read_uuid(&mut self) -> Result<Uuid, DecodeError> {
        let most = self.read_i64()?;
        let least = self.read_i64()?;
        Ok(Uuid::from_bits(most, least))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alluvium_types::Uuid;
    use bytes::Bytes;
    use proptest::prelude::{any, prop_assert_eq, proptest};

    use super::Reader;
    use crate::error::{DecodeError, DecodeErrorKind};
    use crate::writer::Writer;

    fn reader(bytes: &[u8]) -> Reader {
        Reader::new(Bytes::copy_from_slice(bytes))
    }

    fn written(write: impl FnOnce(&mut Writer)) -> Vec<u8> {
        let mut writer = Writer::new();
        write(&mut writer);
        writer.into_bytes().to_vec()
    }

    fn truncated(offset: usize, needed: usize, remaining: usize) -> DecodeError {
        DecodeError::new(offset, DecodeErrorKind::Truncated { needed, remaining })
    }

    #[test]
    fn every_int8_round_trips() {
        for value in i8::MIN..=i8::MAX {
            let bytes = written(|w| w.write_i8(value));
            assert_eq!(reader(&bytes).read_i8(), Ok(value));
        }
    }

    #[test]
    fn fixed_width_integers_are_big_endian() {
        assert_eq!(written(|w| w.write_i16(0x0102)), [1, 2]);
        assert_eq!(written(|w| w.write_i32(0x0102_0304)), [1, 2, 3, 4]);
        assert_eq!(
            written(|w| w.write_i64(0x0102_0304_0506_0708)),
            [1, 2, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(written(|w| w.write_u16(0xfffe)), [0xff, 0xfe]);
        assert_eq!(
            written(|w| w.write_u32(0xffff_fffe)),
            [0xff, 0xff, 0xff, 0xfe]
        );
        assert_eq!(reader(&[0xff, 0xfe]).read_i16(), Ok(-2));
    }

    #[test]
    fn a_short_input_is_truncated_at_the_value_that_started() {
        let mut input = reader(&[0, 1, 2, 3, 4]);
        assert_eq!(input.read_i16(), Ok(1));
        assert_eq!(input.read_i32(), Err(truncated(2, 4, 3)));
        assert_eq!(reader(&[]).read_i8(), Err(truncated(0, 1, 0)));
        assert_eq!(reader(&[0; 7]).read_i64(), Err(truncated(0, 8, 7)));
        assert_eq!(reader(&[0]).read_u16(), Err(truncated(0, 2, 1)));
        assert_eq!(reader(&[0; 3]).read_u32(), Err(truncated(0, 4, 3)));
        assert_eq!(reader(&[0; 7]).read_f64(), Err(truncated(0, 8, 7)));
        assert_eq!(reader(&[]).read_bool(), Err(truncated(0, 1, 0)));
        assert_eq!(reader(&[0; 15]).read_uuid(), Err(truncated(8, 8, 7)));
    }

    #[test]
    fn any_non_zero_byte_is_true_and_true_is_written_as_one() {
        for byte in [1, 2, 0x7f, 0xff] {
            assert_eq!(reader(&[byte]).read_bool(), Ok(true), "{byte}");
        }
        assert_eq!(reader(&[0]).read_bool(), Ok(false));
        assert_eq!(written(|w| w.write_bool(true)), [1]);
        assert_eq!(written(|w| w.write_bool(false)), [0]);
    }

    #[test]
    fn uuids_are_written_as_kafka_writes_them() {
        assert_eq!(written(|w| w.write_uuid(Uuid::ZERO)), [0; 16]);
        // Kafka 4.4.0's ByteBufferAccessor.writeUuid(Uuid.fromString(...)), in JShell.
        let id = Uuid::from_str("UXyU9i5ARn6W00ON2taeWA").unwrap();
        let kafka = [
            0x51, 0x7c, 0x94, 0xf6, 0x2e, 0x40, 0x46, 0x7e, 0x96, 0xd3, 0x43, 0x8d, 0xda, 0xd6,
            0x9e, 0x58,
        ];
        assert_eq!(written(|w| w.write_uuid(id)), kafka);
        assert_eq!(reader(&kafka).read_uuid(), Ok(id));
    }

    proptest! {
        #[test]
        fn integers_round_trip(a in any::<i16>(), b in any::<i32>(), c in any::<i64>(),
                               d in any::<u16>(), e in any::<u32>()) {
            let bytes = written(|w| {
                w.write_i16(a);
                w.write_i32(b);
                w.write_i64(c);
                w.write_u16(d);
                w.write_u32(e);
            });
            let mut input = reader(&bytes);
            prop_assert_eq!(input.read_i16(), Ok(a));
            prop_assert_eq!(input.read_i32(), Ok(b));
            prop_assert_eq!(input.read_i64(), Ok(c));
            prop_assert_eq!(input.read_u16(), Ok(d));
            prop_assert_eq!(input.read_u32(), Ok(e));
            prop_assert_eq!(input.remaining(), 0);
        }

        #[test]
        fn doubles_round_trip_bit_for_bit(bits in any::<u64>()) {
            let bytes = written(|w| w.write_f64(f64::from_bits(bits)));
            prop_assert_eq!(reader(&bytes).read_f64().map(f64::to_bits), Ok(bits));
        }

        #[test]
        fn uuids_round_trip(most in any::<i64>(), least in any::<i64>()) {
            let id = Uuid::from_bits(most, least);
            prop_assert_eq!(reader(&written(|w| w.write_uuid(id))).read_uuid(), Ok(id));
        }
    }

    #[test]
    fn special_doubles_round_trip_bit_for_bit() {
        for value in [
            0.0,
            -0.0,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MIN_POSITIVE / 2.0,
        ] {
            let bytes = written(|w| w.write_f64(value));
            assert_eq!(
                reader(&bytes).read_f64().map(f64::to_bits),
                Ok(value.to_bits())
            );
        }
    }
}
