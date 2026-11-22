//! Length-prefixed values: strings, byte arrays, records and array lengths.
//!
//! Each comes in two framings. Classic versions prefix the length as an `int16` (strings)
//! or `int32` (everything else), with -1 for null. Flexible versions use the compact
//! form: an unsigned varint holding the length plus one, with 0 for null.
//!
//! Lengths are computed as Kafka's generated decoders compute them, with Java `int`
//! arithmetic: any negative length is null, and a compact prefix of `0x80000000` wraps to
//! `i32::MAX` and then fails as truncated. The limits on string and array sizes are
//! the caller's, in `DecodeLimits`.

use bytes::Bytes;

use crate::error::{DecodeError, DecodeErrorKind, EncodeError};
use crate::reader::Reader;
use crate::writer::Writer;

/// How a length prefix is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    /// Classic versions: an `int16` (strings) or `int32` length, -1 for null.
    Classic,
    /// Flexible versions: an unsigned varint of the length plus one, 0 for null.
    Compact,
}

/// The width of a classic length prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Width {
    /// `int16`, for strings.
    Short,
    /// `int32`, for byte arrays, records and arrays.
    Int,
}

/// The longest string Kafka encodes or decodes, in UTF-8 bytes.
const MAX_STRING_LENGTH: usize = 0x7fff;

// Parity: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java@4.4.0 MessageDataGenerator.generateVariableLengthReader
impl Reader {
    /// A length prefix, with Java `int` semantics: negative means null.
    fn read_length(&mut self, framing: Framing, width: Width) -> Result<i32, DecodeError> {
        match (framing, width) {
            (Framing::Classic, Width::Short) => self.read_i16().map(i32::from),
            (Framing::Classic, Width::Int) => self.read_i32(),
            (Framing::Compact, _) => Ok(self.read_unsigned_varint()?.cast_signed().wrapping_sub(1)),
        }
    }

    /// A length prefix that may be null.
    fn read_nullable_length(
        &mut self,
        framing: Framing,
        width: Width,
    ) -> Result<Option<usize>, DecodeError> {
        let length = self.read_length(framing, width)?;
        Ok(usize::try_from(length).ok())
    }

    /// A length prefix that must not be null.
    fn read_present_length(
        &mut self,
        framing: Framing,
        width: Width,
    ) -> Result<usize, DecodeError> {
        let start = self.offset();
        self.read_nullable_length(framing, width)?
            .ok_or_else(|| Reader::error_at(start, DecodeErrorKind::UnexpectedNull))
    }

    /// The UTF-8 text of `length` bytes. Invalid sequences become U+FFFD, as Java's
    /// `new String(bytes, UTF_8)` does.
    fn read_utf8(&mut self, length: usize) -> Result<String, DecodeError> {
        Ok(String::from_utf8_lossy(&self.take(length)?).into_owned())
    }

    /// A `string`.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::UnexpectedNull`] for a null length, and
    /// [`DecodeErrorKind::Truncated`] if the input ends inside it.
    pub fn read_string(&mut self, framing: Framing) -> Result<String, DecodeError> {
        let length = self.read_present_length(framing, Width::Short)?;
        self.read_utf8(length)
    }

    /// A nullable `string`: any negative length is null.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if the input ends inside it.
    pub fn read_nullable_string(
        &mut self,
        framing: Framing,
    ) -> Result<Option<String>, DecodeError> {
        match self.read_nullable_length(framing, Width::Short)? {
            Some(length) => self.read_utf8(length).map(Some),
            None => Ok(None),
        }
    }

    /// A `bytes` field, as a slice of the input.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::UnexpectedNull`] for a null length, and
    /// [`DecodeErrorKind::Truncated`] if the input ends inside it.
    pub fn read_bytes(&mut self, framing: Framing) -> Result<Bytes, DecodeError> {
        let length = self.read_present_length(framing, Width::Int)?;
        self.take(length)
    }

    /// A nullable `bytes` field, as a slice of the input: any negative length is null.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if the input ends inside it.
    pub fn read_nullable_bytes(&mut self, framing: Framing) -> Result<Option<Bytes>, DecodeError> {
        match self.read_nullable_length(framing, Width::Int)? {
            Some(length) => self.take(length).map(Some),
            None => Ok(None),
        }
    }

    /// A `records` field, as an unparsed slice of the input: any negative length is null.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if the input ends inside it.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java@4.4.0 Readable.readRecords
    pub fn read_records(&mut self, framing: Framing) -> Result<Option<Bytes>, DecodeError> {
        self.read_nullable_bytes(framing)
    }

    /// An array's element count. Each element takes at least one byte, so a count larger
    /// than the bytes left is rejected before the caller allocates anything.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::UnexpectedNull`] for a null length, and
    /// [`DecodeErrorKind::ArrayLongerThanInput`] if it exceeds the bytes remaining.
    pub fn read_array_length(&mut self, framing: Framing) -> Result<usize, DecodeError> {
        let start = self.offset();
        let length = self.read_present_length(framing, Width::Int)?;
        self.check_array_length(start, length)
    }

    /// A nullable array's element count: any negative length is null.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::ArrayLongerThanInput`] if it exceeds the bytes remaining.
    pub fn read_nullable_array_length(
        &mut self,
        framing: Framing,
    ) -> Result<Option<usize>, DecodeError> {
        let start = self.offset();
        match self.read_nullable_length(framing, Width::Int)? {
            Some(length) => self.check_array_length(start, length).map(Some),
            None => Ok(None),
        }
    }

    fn check_array_length(&self, start: usize, length: usize) -> Result<usize, DecodeError> {
        if length > self.remaining() {
            return Err(Reader::error_at(
                start,
                DecodeErrorKind::ArrayLongerThanInput {
                    length,
                    remaining: self.remaining(),
                },
            ));
        }
        Ok(length)
    }
}

// Parity: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java@4.4.0 MessageDataGenerator.generateVariableLengthWriter
impl Writer {
    /// A length prefix: `None` writes null.
    fn write_length(
        &mut self,
        framing: Framing,
        width: Width,
        length: Option<usize>,
    ) -> Result<(), EncodeError> {
        let too_long = |length| EncodeError::LengthTooLong { length };
        match (framing, width, length) {
            (Framing::Classic, Width::Short, None) => self.write_i16(-1),
            (Framing::Classic, Width::Int, None) => self.write_i32(-1),
            (Framing::Compact, _, None) => self.write_unsigned_varint(0),
            (Framing::Classic, Width::Short, Some(length)) => {
                self.write_i16(i16::try_from(length).map_err(|_| too_long(length))?);
            }
            (Framing::Classic, Width::Int, Some(length)) => {
                self.write_i32(i32::try_from(length).map_err(|_| too_long(length))?);
            }
            (Framing::Compact, _, Some(length)) => {
                // Kafka adds one in a Java int, so the length must stay below i32::MAX.
                let prefix = i32::try_from(length)
                    .ok()
                    .and_then(|length| length.checked_add(1))
                    .ok_or_else(|| too_long(length))?;
                self.write_unsigned_varint(prefix.cast_unsigned());
            }
        }
        Ok(())
    }

    /// A `string`.
    ///
    /// # Errors
    ///
    /// [`EncodeError::StringTooLong`] above 32767 UTF-8 bytes, as Kafka refuses.
    pub fn write_string(&mut self, framing: Framing, value: &str) -> Result<(), EncodeError> {
        if value.len() > MAX_STRING_LENGTH {
            return Err(EncodeError::StringTooLong {
                length: value.len(),
            });
        }
        self.write_length(framing, Width::Short, Some(value.len()))?;
        self.put_slice(value.as_bytes());
        Ok(())
    }

    /// A nullable `string`.
    ///
    /// # Errors
    ///
    /// As [`Writer::write_string`].
    pub fn write_nullable_string(
        &mut self,
        framing: Framing,
        value: Option<&str>,
    ) -> Result<(), EncodeError> {
        match value {
            Some(value) => self.write_string(framing, value),
            None => self.write_length(framing, Width::Short, None),
        }
    }

    /// A `bytes` field.
    ///
    /// # Errors
    ///
    /// [`EncodeError::LengthTooLong`] if its length doesn't fit the prefix.
    pub fn write_bytes(&mut self, framing: Framing, value: &[u8]) -> Result<(), EncodeError> {
        self.write_length(framing, Width::Int, Some(value.len()))?;
        self.put_slice(value);
        Ok(())
    }

    /// A nullable `bytes` field.
    ///
    /// # Errors
    ///
    /// As [`Writer::write_bytes`].
    pub fn write_nullable_bytes(
        &mut self,
        framing: Framing,
        value: Option<&[u8]>,
    ) -> Result<(), EncodeError> {
        match value {
            Some(value) => self.write_bytes(framing, value),
            None => self.write_length(framing, Width::Int, None),
        }
    }

    /// A `records` field, written as it is.
    ///
    /// # Errors
    ///
    /// As [`Writer::write_bytes`].
    pub fn write_records(
        &mut self,
        framing: Framing,
        value: Option<&[u8]>,
    ) -> Result<(), EncodeError> {
        self.write_nullable_bytes(framing, value)
    }

    /// An array's element count.
    ///
    /// # Errors
    ///
    /// [`EncodeError::LengthTooLong`] if it doesn't fit the prefix.
    pub fn write_array_length(
        &mut self,
        framing: Framing,
        length: usize,
    ) -> Result<(), EncodeError> {
        self.write_length(framing, Width::Int, Some(length))
    }

    /// A nullable array's element count.
    ///
    /// # Errors
    ///
    /// As [`Writer::write_array_length`].
    pub fn write_nullable_array_length(
        &mut self,
        framing: Framing,
        length: Option<usize>,
    ) -> Result<(), EncodeError> {
        self.write_length(framing, Width::Int, length)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use proptest::option;
    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};

    use super::Framing::{self, Classic, Compact};
    use crate::error::{DecodeError, DecodeErrorKind, EncodeError};
    use crate::reader::Reader;
    use crate::writer::Writer;

    fn reader(bytes: &[u8]) -> Reader {
        Reader::new(Bytes::copy_from_slice(bytes))
    }

    fn written(write: impl FnOnce(&mut Writer) -> Result<(), EncodeError>) -> Vec<u8> {
        let mut writer = Writer::new();
        write(&mut writer).unwrap();
        writer.into_bytes().to_vec()
    }

    fn error(offset: usize, kind: DecodeErrorKind) -> DecodeError {
        DecodeError::new(offset, kind)
    }

    fn truncated(offset: usize, needed: usize, remaining: usize) -> DecodeError {
        error(offset, DecodeErrorKind::Truncated { needed, remaining })
    }

    /// A compact length prefix: the unsigned varint `prefix`.
    fn compact_prefix(prefix: u32) -> Vec<u8> {
        let mut writer = Writer::new();
        writer.write_unsigned_varint(prefix);
        writer.into_bytes().to_vec()
    }

    #[test]
    fn strings_are_length_prefixed_utf8() {
        assert_eq!(
            written(|w| w.write_string(Classic, "hé")),
            [0, 3, b'h', 0xc3, 0xa9]
        );
        assert_eq!(
            written(|w| w.write_string(Compact, "hé")),
            [4, b'h', 0xc3, 0xa9]
        );
        assert_eq!(
            written(|w| w.write_nullable_string(Classic, None)),
            [0xff, 0xff]
        );
        assert_eq!(written(|w| w.write_nullable_string(Compact, None)), [0]);
        assert_eq!(
            reader(&[0, 1, 0xff]).read_string(Classic),
            Ok("\u{fffd}".to_owned())
        );
    }

    #[test]
    fn a_null_string_is_rejected_where_it_isnt_allowed() {
        assert_eq!(
            reader(&[0xff, 0xfe]).read_string(Classic),
            Err(error(0, DecodeErrorKind::UnexpectedNull))
        );
        assert_eq!(
            reader(&[0x80, 0x00]).read_nullable_string(Classic),
            Ok(None)
        );
        assert_eq!(
            reader(&[0]).read_string(Compact),
            Err(error(0, DecodeErrorKind::UnexpectedNull))
        );
        assert_eq!(
            reader(&[0, 3, b'a']).read_string(Classic),
            Err(truncated(2, 3, 1))
        );
    }

    #[test]
    fn compact_lengths_use_java_int_arithmetic() {
        // 0 and 0x80000001..=0xFFFFFFFF are negative after Kafka's `- 1`: null.
        for prefix in [0, 0x8000_0001, 0xffff_ffff] {
            let bytes = compact_prefix(prefix);
            assert_eq!(
                reader(&bytes).read_nullable_string(Compact),
                Ok(None),
                "{prefix:#x}"
            );
            assert_eq!(
                reader(&bytes).read_nullable_bytes(Compact),
                Ok(None),
                "{prefix:#x}"
            );
            assert_eq!(
                reader(&bytes).read_records(Compact),
                Ok(None),
                "{prefix:#x}"
            );
            assert_eq!(
                reader(&bytes).read_bytes(Compact),
                Err(error(0, DecodeErrorKind::UnexpectedNull)),
                "{prefix:#x}"
            );
        }
        // 0x80000000 - 1 wraps to i32::MAX: not null, and far too long.
        let bytes = compact_prefix(0x8000_0000);
        assert_eq!(
            reader(&bytes).read_string(Compact),
            Err(truncated(5, i32::MAX.cast_unsigned() as usize, 0))
        );
    }

    #[test]
    fn strings_over_32767_bytes_are_not_encoded() {
        let long = "x".repeat(0x8000);
        for framing in [Classic, Compact] {
            assert_eq!(
                Writer::new().write_string(framing, &long),
                Err(EncodeError::StringTooLong { length: 0x8000 })
            );
        }
        assert!(
            Writer::new()
                .write_string(Classic, &"x".repeat(0x7fff))
                .is_ok()
        );
    }

    #[test]
    fn bytes_and_records_are_slices_of_the_input() {
        let input = Bytes::from_static(&[0, 0, 0, 2, 7, 8, 3, 9, 9]);
        let mut input_reader = Reader::new(input.clone());
        let classic = input_reader.read_bytes(Classic).unwrap();
        let compact = input_reader.read_records(Compact).unwrap().unwrap();
        assert_eq!((&classic[..], &compact[..]), (&[7, 8][..], &[9, 9][..]));
        let input_range = input.as_ptr_range();
        for slice in [&classic, &compact] {
            assert!(
                input_range.contains(&slice.as_ptr()),
                "copied instead of sliced"
            );
        }
        assert_eq!(
            Reader::new(Bytes::from_static(&[0xff, 0xff, 0xff, 0xff])).read_bytes(Classic),
            Err(error(0, DecodeErrorKind::UnexpectedNull))
        );
        assert_eq!(reader(&[0x80, 0, 0, 0]).read_records(Classic), Ok(None));
        assert_eq!(
            reader(&[0, 0, 0, 5, 1]).read_bytes(Classic),
            Err(truncated(4, 5, 1))
        );
    }

    #[test]
    fn empty_and_null_bytes_differ() {
        assert_eq!(written(|w| w.write_nullable_bytes(Compact, Some(&[]))), [1]);
        assert_eq!(written(|w| w.write_nullable_bytes(Compact, None)), [0]);
        assert_eq!(
            written(|w| w.write_nullable_bytes(Classic, Some(&[]))),
            [0, 0, 0, 0]
        );
        assert_eq!(
            written(|w| w.write_nullable_bytes(Classic, None)),
            [0xff; 4]
        );
    }

    #[test]
    fn array_lengths_beyond_the_input_fail_before_any_allocation() {
        assert_eq!(
            reader(&[0x7f, 0xff, 0xff, 0xff]).read_array_length(Classic),
            Err(error(
                0,
                DecodeErrorKind::ArrayLongerThanInput {
                    length: i32::MAX.cast_unsigned() as usize,
                    remaining: 0
                }
            ))
        );
        assert_eq!(
            reader(&[0xff, 0xff, 0xff, 0xff]).read_nullable_array_length(Classic),
            Ok(None)
        );
        assert_eq!(
            reader(&[0xff, 0xff, 0xff, 0xff]).read_array_length(Classic),
            Err(error(0, DecodeErrorKind::UnexpectedNull))
        );
        assert_eq!(reader(&[3, 1, 2]).read_array_length(Compact), Ok(2));
        assert_eq!(reader(&[0]).read_nullable_array_length(Compact), Ok(None));
        assert_eq!(
            reader(&compact_prefix(0x8000_0000)).read_array_length(Compact),
            Err(error(
                0,
                DecodeErrorKind::ArrayLongerThanInput {
                    length: i32::MAX.cast_unsigned() as usize,
                    remaining: 0
                }
            ))
        );
    }

    fn framing() -> impl proptest::strategy::Strategy<Value = Framing> {
        proptest::prop_oneof![
            proptest::strategy::Just(Classic),
            proptest::strategy::Just(Compact)
        ]
    }

    proptest! {
        #[test]
        fn strings_round_trip(framing in framing(), value in ".{0,40}", nullable in option::of(".{0,40}")) {
            let bytes = written(|w| {
                w.write_string(framing, &value)?;
                w.write_nullable_string(framing, nullable.as_deref())
            });
            let mut input = reader(&bytes);
            prop_assert_eq!(input.read_string(framing), Ok(value));
            prop_assert_eq!(input.read_nullable_string(framing), Ok(nullable));
            prop_assert_eq!(input.remaining(), 0);
        }

        #[test]
        fn bytes_records_and_array_lengths_round_trip(
            framing in framing(),
            value in proptest::collection::vec(any::<u8>(), 0..40),
            nullable in option::of(proptest::collection::vec(any::<u8>(), 0..40)),
            length in 0_usize..10,
            nullable_length in option::of(0_usize..10),
        ) {
            let bytes = written(|w| {
                w.write_bytes(framing, &value)?;
                w.write_nullable_bytes(framing, nullable.as_deref())?;
                w.write_records(framing, nullable.as_deref())?;
                w.write_array_length(framing, length)?;
                w.write_nullable_array_length(framing, nullable_length)?;
                w.write_bytes(framing, &[0; 20])
            });
            let mut input = reader(&bytes);
            prop_assert_eq!(input.read_bytes(framing).map(|b| b.to_vec()), Ok(value));
            prop_assert_eq!(input.read_nullable_bytes(framing).map(|b| b.map(|b| b.to_vec())), Ok(nullable.clone()));
            prop_assert_eq!(input.read_records(framing).map(|b| b.map(|b| b.to_vec())), Ok(nullable));
            prop_assert_eq!(input.read_array_length(framing), Ok(length));
            prop_assert_eq!(input.read_nullable_array_length(framing), Ok(nullable_length));
            prop_assert!(input.read_bytes(framing).is_ok());
        }
    }
}
