// Portions of this file are ported from Apache Kafka 4.4.0 and modified by Alluvium;
// each is marked `// Ported from:`. Apache Kafka is Copyright The Apache Software
// Foundation and licensed under the Apache License 2.0 (LICENSES/Apache-2.0.txt).
// See NOTICE.

//! Variable-length integers: the unsigned varint (at most 5 bytes), and the zigzag
//! signed varint and varlong (at most 10 bytes) of flexible versions and records.
//!
//! Decoding matches Kafka bit for bit, including inputs Kafka accepts that a canonical
//! decoder would reject: an overlong encoding such as `0x80 0x00` is read, and the bits
//! of an unsigned varint's 5th byte above bit 3 are dropped. Encoding is canonical.

use crate::error::{DecodeError, DecodeErrorKind};
use crate::reader::Reader;
use crate::writer::Writer;

impl Reader {
    /// An unsigned varint: 7 bits per byte, least significant first, up to 5 bytes.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if the input ends inside it, and
    /// [`DecodeErrorKind::VarintTooLong`] if its 5th byte has the continuation bit set.
    // Parity: clients/src/main/java/org/apache/kafka/common/utils/internals/ByteUtils.java@4.4.0 ByteUtils.readUnsignedVarint
    pub fn read_unsigned_varint(&mut self) -> Result<u32, DecodeError> {
        let start = self.offset();
        let mut value = 0_u32;
        for shift in [0, 7, 14, 21] {
            let [byte] = self.array()?;
            value |= u32::from(byte & 0x7f).wrapping_shl(shift);
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        let [byte] = self.array()?;
        if byte & 0x80 != 0 {
            return Err(Reader::error_at(start, DecodeErrorKind::VarintTooLong));
        }
        // Bits of the 5th byte above bit 3 shift out of the u32, as in Kafka.
        Ok(value | u32::from(byte).wrapping_shl(28))
    }

    /// A zigzag-encoded `i32` in an unsigned varint.
    ///
    /// # Errors
    ///
    /// As [`Reader::read_unsigned_varint`].
    // Parity: clients/src/main/java/org/apache/kafka/common/utils/internals/ByteUtils.java@4.4.0 ByteUtils.readVarint
    pub fn read_varint(&mut self) -> Result<i32, DecodeError> {
        let raw = self.read_unsigned_varint()?;
        Ok(raw.wrapping_shr(1).cast_signed() ^ (raw & 1).cast_signed().wrapping_neg())
    }

    /// A zigzag-encoded `i64`: 7 bits per byte, up to 10 bytes.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if the input ends inside it, and
    /// [`DecodeErrorKind::VarlongTooLong`] if its 10th byte has the continuation bit set.
    // Parity: clients/src/main/java/org/apache/kafka/common/utils/internals/ByteUtils.java@4.4.0 ByteUtils.readVarlong, ByteUtils.readUnsignedVarlong
    pub fn read_varlong(&mut self) -> Result<i64, DecodeError> {
        let start = self.offset();
        let mut raw = 0_u64;
        let mut shift = 0_u32;
        loop {
            let [byte] = self.array()?;
            if byte & 0x80 == 0 {
                raw |= u64::from(byte).wrapping_shl(shift);
                break;
            }
            raw |= u64::from(byte & 0x7f).wrapping_shl(shift);
            shift = shift.wrapping_add(7);
            if shift > 63 {
                return Err(Reader::error_at(start, DecodeErrorKind::VarlongTooLong));
            }
        }
        Ok(raw.wrapping_shr(1).cast_signed() ^ (raw & 1).cast_signed().wrapping_neg())
    }
}

impl Writer {
    /// An unsigned varint, in the fewest bytes.
    // Parity: clients/src/main/java/org/apache/kafka/common/utils/internals/ByteUtils.java@4.4.0 ByteUtils.writeUnsignedVarint
    pub fn write_unsigned_varint(&mut self, value: u32) {
        let mut rest = value;
        while rest >= 0x80 {
            let [low, ..] = rest.to_le_bytes();
            self.put_u8(low | 0x80);
            rest = rest.wrapping_shr(7);
        }
        let [low, ..] = rest.to_le_bytes();
        self.put_u8(low);
    }

    /// A zigzag-encoded `i32`.
    // Parity: clients/src/main/java/org/apache/kafka/common/utils/internals/ByteUtils.java@4.4.0 ByteUtils.writeVarint
    pub fn write_varint(&mut self, value: i32) {
        self.write_unsigned_varint(zigzag32(value));
    }

    /// A zigzag-encoded `i64`, in the fewest bytes.
    // Parity: clients/src/main/java/org/apache/kafka/common/utils/internals/ByteUtils.java@4.4.0 ByteUtils.writeVarlong
    pub fn write_varlong(&mut self, value: i64) {
        let mut rest = zigzag64(value);
        while rest >= 0x80 {
            let [low, ..] = rest.to_le_bytes();
            self.put_u8(low | 0x80);
            rest = rest.wrapping_shr(7);
        }
        let [low, ..] = rest.to_le_bytes();
        self.put_u8(low);
    }
}

fn zigzag32(value: i32) -> u32 {
    (value.wrapping_shl(1) ^ value.wrapping_shr(31)).cast_unsigned()
}

fn zigzag64(value: i64) -> u64 {
    (value.wrapping_shl(1) ^ value.wrapping_shr(63)).cast_unsigned()
}

/// The bytes [`Writer::write_unsigned_varint`] writes for `value`: 1 to 5.
// Parity: clients/src/main/java/org/apache/kafka/common/utils/internals/ByteUtils.java@4.4.0 ByteUtils.sizeOfUnsignedVarint
pub fn size_of_unsigned_varint(value: u32) -> usize {
    bytes_for(u32::BITS.saturating_sub(value.leading_zeros()))
}

/// The bytes [`Writer::write_varint`] writes for `value`: 1 to 5.
pub fn size_of_varint(value: i32) -> usize {
    size_of_unsigned_varint(zigzag32(value))
}

/// The bytes [`Writer::write_varlong`] writes for `value`: 1 to 10.
// Parity: clients/src/main/java/org/apache/kafka/common/utils/internals/ByteUtils.java@4.4.0 ByteUtils.sizeOfVarlong
pub fn size_of_varlong(value: i64) -> usize {
    bytes_for(u64::BITS.saturating_sub(zigzag64(value).leading_zeros()))
}

/// Bytes of 7 bits each for `bits` significant bits, at least one.
fn bytes_for(bits: u32) -> usize {
    usize::try_from(bits.div_ceil(7).max(1)).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use alluvium_types::strategies::adversarial::{VarintShape, bad_varint};
    use bytes::Bytes;
    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};

    use super::{size_of_unsigned_varint, size_of_varint, size_of_varlong};
    use crate::error::{DecodeError, DecodeErrorKind};
    use crate::reader::Reader;
    use crate::writer::Writer;

    fn reader(bytes: &[u8]) -> Reader {
        Reader::new(Bytes::copy_from_slice(bytes))
    }

    fn written(write: impl FnOnce(&mut Writer)) -> Vec<u8> {
        let mut writer = Writer::new();
        write(&mut writer);
        writer.into_bytes().to_vec()
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/utils/internals/ByteUtilsTest.java@4.4.0
    // ByteUtilsTest.testUnsignedVarintSerde, testVarintSerde, testVarlongSerde,
    // testInvalidVarint and testInvalidVarlong. Kafka's unsigned varints are Java
    // ints, so negative values appear here as their u32 bits.

    fn unsigned_varint_serde(value: i32, expected: &[u8]) {
        let value = value.cast_unsigned();
        assert_eq!(
            written(|w| w.write_unsigned_varint(value)),
            expected,
            "{value}"
        );
        assert_eq!(reader(expected).read_unsigned_varint(), Ok(value));
        assert_eq!(size_of_unsigned_varint(value), expected.len());
    }

    fn varint_serde(value: i32, expected: &[u8]) {
        assert_eq!(written(|w| w.write_varint(value)), expected, "{value}");
        assert_eq!(reader(expected).read_varint(), Ok(value));
        assert_eq!(size_of_varint(value), expected.len());
    }

    fn varlong_serde(value: i64, expected: &[u8]) {
        assert_eq!(written(|w| w.write_varlong(value)), expected, "{value}");
        assert_eq!(reader(expected).read_varlong(), Ok(value));
        assert_eq!(size_of_varlong(value), expected.len());
    }

    #[test]
    fn unsigned_varint_serde_vectors() {
        unsigned_varint_serde(0, &[0x00]);
        unsigned_varint_serde(-1, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        unsigned_varint_serde(1, &[0x01]);
        unsigned_varint_serde(63, &[0x3F]);
        unsigned_varint_serde(-64, &[0xC0, 0xFF, 0xFF, 0xFF, 0x0F]);
        unsigned_varint_serde(64, &[0x40]);
        unsigned_varint_serde(8191, &[0xFF, 0x3F]);
        unsigned_varint_serde(-8192, &[0x80, 0xC0, 0xFF, 0xFF, 0x0F]);
        unsigned_varint_serde(8192, &[0x80, 0x40]);
        unsigned_varint_serde(-8193, &[0xFF, 0xBF, 0xFF, 0xFF, 0x0F]);
        unsigned_varint_serde(1_048_575, &[0xFF, 0xFF, 0x3F]);
        unsigned_varint_serde(1_048_576, &[0x80, 0x80, 0x40]);
        unsigned_varint_serde(i32::MAX, &[0xFF, 0xFF, 0xFF, 0xFF, 0x07]);
        unsigned_varint_serde(i32::MIN, &[0x80, 0x80, 0x80, 0x80, 0x08]);
    }

    #[test]
    fn varint_serde_vectors() {
        varint_serde(0, &[0x00]);
        varint_serde(-1, &[0x01]);
        varint_serde(1, &[0x02]);
        varint_serde(63, &[0x7E]);
        varint_serde(-64, &[0x7F]);
        varint_serde(64, &[0x80, 0x01]);
        varint_serde(-65, &[0x81, 0x01]);
        varint_serde(8191, &[0xFE, 0x7F]);
        varint_serde(-8192, &[0xFF, 0x7F]);
        varint_serde(8192, &[0x80, 0x80, 0x01]);
        varint_serde(-8193, &[0x81, 0x80, 0x01]);
        varint_serde(1_048_575, &[0xFE, 0xFF, 0x7F]);
        varint_serde(-1_048_576, &[0xFF, 0xFF, 0x7F]);
        varint_serde(1_048_576, &[0x80, 0x80, 0x80, 0x01]);
        varint_serde(-1_048_577, &[0x81, 0x80, 0x80, 0x01]);
        varint_serde(134_217_727, &[0xFE, 0xFF, 0xFF, 0x7F]);
        varint_serde(-134_217_728, &[0xFF, 0xFF, 0xFF, 0x7F]);
        varint_serde(134_217_728, &[0x80, 0x80, 0x80, 0x80, 0x01]);
        varint_serde(-134_217_729, &[0x81, 0x80, 0x80, 0x80, 0x01]);
        varint_serde(i32::MAX, &[0xFE, 0xFF, 0xFF, 0xFF, 0x0F]);
        varint_serde(i32::MIN, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    }

    #[test]
    fn varlong_serde_vectors() {
        varlong_serde(0, &[0x00]);
        varlong_serde(-1, &[0x01]);
        varlong_serde(1, &[0x02]);
        varlong_serde(63, &[0x7E]);
        varlong_serde(-64, &[0x7F]);
        varlong_serde(64, &[0x80, 0x01]);
        varlong_serde(-65, &[0x81, 0x01]);
        varlong_serde(8191, &[0xFE, 0x7F]);
        varlong_serde(-8192, &[0xFF, 0x7F]);
        varlong_serde(8192, &[0x80, 0x80, 0x01]);
        varlong_serde(-8193, &[0x81, 0x80, 0x01]);
        varlong_serde(1_048_575, &[0xFE, 0xFF, 0x7F]);
        varlong_serde(-1_048_576, &[0xFF, 0xFF, 0x7F]);
        varlong_serde(1_048_576, &[0x80, 0x80, 0x80, 0x01]);
        varlong_serde(-1_048_577, &[0x81, 0x80, 0x80, 0x01]);
        varlong_serde(134_217_727, &[0xFE, 0xFF, 0xFF, 0x7F]);
        varlong_serde(-134_217_728, &[0xFF, 0xFF, 0xFF, 0x7F]);
        varlong_serde(134_217_728, &[0x80, 0x80, 0x80, 0x80, 0x01]);
        varlong_serde(-134_217_729, &[0x81, 0x80, 0x80, 0x80, 0x01]);
        varlong_serde(i64::from(i32::MAX), &[0xFE, 0xFF, 0xFF, 0xFF, 0x0F]);
        varlong_serde(i64::from(i32::MIN), &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        varlong_serde(17_179_869_183, &[0xFE, 0xFF, 0xFF, 0xFF, 0x7F]);
        varlong_serde(-17_179_869_184, &[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
        varlong_serde(17_179_869_184, &[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);
        varlong_serde(-17_179_869_185, &[0x81, 0x80, 0x80, 0x80, 0x80, 0x01]);
        varlong_serde(2_199_023_255_551, &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
        varlong_serde(-2_199_023_255_552, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
        varlong_serde(
            2_199_023_255_552,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );
        varlong_serde(
            -2_199_023_255_553,
            &[0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );
        varlong_serde(
            281_474_976_710_655,
            &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        );
        varlong_serde(
            -281_474_976_710_656,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        );
        varlong_serde(
            281_474_976_710_656,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );
        varlong_serde(
            -281_474_976_710_657,
            &[0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );
        varlong_serde(
            36_028_797_018_963_967,
            &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        );
        varlong_serde(
            -36_028_797_018_963_968,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        );
        varlong_serde(
            36_028_797_018_963_968,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );
        varlong_serde(
            -36_028_797_018_963_969,
            &[0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );
        varlong_serde(
            4_611_686_018_427_387_903,
            &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        );
        varlong_serde(
            -4_611_686_018_427_387_904,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        );
        varlong_serde(
            4_611_686_018_427_387_904,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );
        varlong_serde(
            -4_611_686_018_427_387_905,
            &[0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );
        varlong_serde(
            i64::MAX,
            &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
        );
        varlong_serde(
            i64::MIN,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
        );
    }

    #[test]
    fn invalid_varint() {
        // One overflow byte.
        assert_eq!(
            reader(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]).read_varint(),
            Err(DecodeError::new(0, DecodeErrorKind::VarintTooLong))
        );
    }

    #[test]
    fn invalid_varlong() {
        // One overflow byte.
        let bytes = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
        ];
        assert_eq!(
            reader(&bytes).read_varlong(),
            Err(DecodeError::new(0, DecodeErrorKind::VarlongTooLong))
        );
    }

    // End of the ported tests.

    #[test]
    fn kafka_accepts_overlong_varints_and_drops_the_5th_bytes_high_bits() {
        assert_eq!(reader(&[0x80, 0x00]).read_unsigned_varint(), Ok(0));
        assert_eq!(
            reader(&[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]).read_unsigned_varint(),
            reader(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]).read_unsigned_varint()
        );
        let mut input = reader(&[0x7F, 0x80]);
        assert_eq!(input.read_unsigned_varint(), Ok(0x7F));
        assert_eq!(
            input.read_unsigned_varint(),
            Err(DecodeError::new(
                2,
                DecodeErrorKind::Truncated {
                    needed: 1,
                    remaining: 0
                }
            ))
        );
    }

    proptest! {
        #[test]
        fn bad_varints_are_read_as_kafka_reads_them((bytes, shape) in bad_varint(5)) {
            let result = Reader::new(Bytes::from(bytes.clone())).read_unsigned_varint();
            match shape {
                // Padding adds no bits, so the value is the first byte's seven.
                VarintShape::Overlong => prop_assert_eq!(result, Ok(u32::from(bytes[0] & 0x7f))),
                VarintShape::TooLong => prop_assert_eq!(
                    result,
                    Err(DecodeError::new(0, DecodeErrorKind::VarintTooLong))
                ),
                VarintShape::Unterminated => {
                    let truncated = matches!(
                        result,
                        Err(DecodeError { kind: DecodeErrorKind::Truncated { .. }, .. })
                    );
                    prop_assert!(truncated, "{result:?}");
                }
            }
        }

        #[test]
        fn bad_varlongs_are_read_as_kafka_reads_them((bytes, shape) in bad_varint(10)) {
            let result = Reader::new(Bytes::from(bytes.clone())).read_varlong();
            match shape {
                VarintShape::Overlong => {
                    let raw = i64::from(bytes[0] & 0x7f);
                    prop_assert_eq!(result, Ok((raw >> 1) ^ (raw & 1).wrapping_neg()));
                }
                VarintShape::TooLong => prop_assert_eq!(
                    result,
                    Err(DecodeError::new(0, DecodeErrorKind::VarlongTooLong))
                ),
                VarintShape::Unterminated => {
                    let truncated = matches!(
                        result,
                        Err(DecodeError { kind: DecodeErrorKind::Truncated { .. }, .. })
                    );
                    prop_assert!(truncated, "{result:?}");
                }
            }
        }
    }

    proptest! {
        #[test]
        fn varints_round_trip_in_their_computed_size(a in any::<u32>(), b in any::<i32>(), c in any::<i64>()) {
            let bytes = written(|w| w.write_unsigned_varint(a));
            prop_assert_eq!(bytes.len(), size_of_unsigned_varint(a));
            prop_assert_eq!(reader(&bytes).read_unsigned_varint(), Ok(a));
            let bytes = written(|w| w.write_varint(b));
            prop_assert_eq!(bytes.len(), size_of_varint(b));
            prop_assert_eq!(reader(&bytes).read_varint(), Ok(b));
            let bytes = written(|w| w.write_varlong(c));
            prop_assert_eq!(bytes.len(), size_of_varlong(c));
            prop_assert_eq!(reader(&bytes).read_varlong(), Ok(c));
        }
    }
}
