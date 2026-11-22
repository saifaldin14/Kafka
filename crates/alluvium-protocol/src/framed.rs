//! Length-prefixed values: strings, byte arrays and records, and the length prefix of
//! arrays, whose elements `array.rs` decodes.
//!
//! Each comes in two framings. Classic versions prefix the length as an `int16` (strings)
//! or `int32` (everything else), with -1 for null. Flexible versions use the compact
//! form: an unsigned varint holding the length plus one, with 0 for null.
//!
//! Lengths are computed as Kafka's generated decoders compute them, with Java `int`
//! arithmetic: any negative length is null, and a compact prefix of `0x80000000` wraps to
//! `i32::MAX`, which then fails as too long for a string or as truncated otherwise.

use std::str::Utf8Chunk;

use bytes::Bytes;

use crate::error::{DecodeError, DecodeErrorKind, EncodeError};
use crate::limits::Limit;
use crate::reader::Reader;
use crate::records::RecordsField;
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
pub(crate) enum Width {
    /// `int16`, for strings.
    Short,
    /// `int32`, for byte arrays, records and arrays.
    Int,
}

/// The UTF-8 length of U+FFFD, which replaces each invalid sequence in a string.
const REPLACEMENT_LENGTH: usize = char::REPLACEMENT_CHARACTER.len_utf8();

/// The longest string Kafka encodes or decodes, in UTF-8 bytes. A classic `int16`
/// prefix can't exceed it; a compact one can, and is rejected before it is read.
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
    pub(crate) fn read_nullable_length(
        &mut self,
        framing: Framing,
        width: Width,
    ) -> Result<Option<usize>, DecodeError> {
        let length = self.read_length(framing, width)?;
        Ok(usize::try_from(length).ok())
    }

    /// A length prefix that must not be null.
    pub(crate) fn read_present_length(
        &mut self,
        framing: Framing,
        width: Width,
    ) -> Result<usize, DecodeError> {
        let start = self.offset();
        self.read_nullable_length(framing, width)?
            .ok_or_else(|| Reader::error_at(start, DecodeErrorKind::UnexpectedNull))
    }

    /// The UTF-8 text of `length` bytes, whose length prefix started at `start`.
    /// Invalid sequences become U+FFFD, as Java's `new String(bytes, UTF_8)` does.
    fn read_utf8(&mut self, start: usize, length: usize) -> Result<String, DecodeError> {
        if length > MAX_STRING_LENGTH {
            return Err(Reader::error_at(
                start,
                DecodeErrorKind::LimitExceeded {
                    limit: Limit::StringLength,
                    requested: length,
                    allowed: MAX_STRING_LENGTH,
                },
            ));
        }
        let bytes = self.take(length)?;
        // Valid text, the usual case, is checked once and copied once.
        if let Ok(text) = std::str::from_utf8(&bytes) {
            self.charge(start, length)?;
            return Ok(text.to_owned());
        }
        // Each invalid sequence becomes one U+FFFD, as in `String::from_utf8_lossy`. The
        // text is built here so the exact length can be charged before it is allocated.
        let replaced = |chunk: &Utf8Chunk<'_>| !chunk.invalid().is_empty();
        let decoded_length = bytes.utf8_chunks().fold(0_usize, |total, chunk| {
            let replacement = if replaced(&chunk) {
                REPLACEMENT_LENGTH
            } else {
                0
            };
            total
                .saturating_add(chunk.valid().len())
                .saturating_add(replacement)
        });
        self.charge(start, decoded_length)?;
        let mut text = String::with_capacity(decoded_length);
        for chunk in bytes.utf8_chunks() {
            text.push_str(chunk.valid());
            if replaced(&chunk) {
                text.push(char::REPLACEMENT_CHARACTER);
            }
        }
        Ok(text)
    }

    /// A `string`.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::UnexpectedNull`] for a null length,
    /// [`DecodeErrorKind::LimitExceeded`] for a length over 32767 bytes, and
    /// [`DecodeErrorKind::Truncated`] if the input ends inside it.
    pub fn read_string(&mut self, framing: Framing) -> Result<String, DecodeError> {
        let start = self.offset();
        let length = self.read_present_length(framing, Width::Short)?;
        self.read_utf8(start, length)
    }

    /// A nullable `string`: any negative length is null.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::LimitExceeded`] for a length over 32767 bytes, and
    /// [`DecodeErrorKind::Truncated`] if the input ends inside it.
    pub fn read_nullable_string(
        &mut self,
        framing: Framing,
    ) -> Result<Option<String>, DecodeError> {
        let start = self.offset();
        match self.read_nullable_length(framing, Width::Short)? {
            Some(length) => self.read_utf8(start, length).map(Some),
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
    pub fn read_records(&mut self, framing: Framing) -> Result<Option<RecordsField>, DecodeError> {
        self.read_nullable_bytes(framing)
            .map(|bytes| bytes.map(RecordsField::new))
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
        value: Option<&RecordsField>,
    ) -> Result<(), EncodeError> {
        self.write_nullable_bytes(framing, value.map(|records| &records.as_bytes()[..]))
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
    use alluvium_types::strategies::adversarial::{
        lying_compact_length, lying_int32_length, truncated as cut_short,
    };
    use bytes::Bytes;
    use proptest::option;
    use proptest::prelude::{Just, Strategy, any, prop_assert, prop_assert_eq, proptest};

    use super::Framing::{self, Classic, Compact};
    use crate::error::{DecodeError, DecodeErrorKind, EncodeError};
    use crate::limits::{DecodeLimits, Limit};
    use crate::reader::Reader;
    use crate::records::RecordsField;
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
        let wrapped = i32::MAX.cast_unsigned() as usize;
        assert_eq!(
            reader(&bytes).read_bytes(Compact),
            Err(truncated(5, wrapped, 0))
        );
        assert_eq!(
            reader(&bytes).read_string(Compact),
            Err(error(
                0,
                DecodeErrorKind::LimitExceeded {
                    limit: Limit::StringLength,
                    requested: wrapped,
                    allowed: 0x7fff
                }
            ))
        );
    }

    #[test]
    fn strings_over_32767_bytes_fail_before_they_are_read() {
        let too_long = error(
            0,
            DecodeErrorKind::LimitExceeded {
                limit: Limit::StringLength,
                requested: 40_000,
                allowed: 0x7fff,
            },
        );
        // Only the prefix is present: the length is rejected, not found truncated.
        let declared = compact_prefix(40_001);
        assert_eq!(
            reader(&declared).read_string(Compact),
            Err(too_long.clone())
        );
        assert_eq!(
            reader(&declared).read_nullable_string(Compact),
            Err(too_long)
        );
        let mut longest = compact_prefix(0x8000);
        longest.extend(std::iter::repeat_n(b'x', 0x7fff));
        assert_eq!(
            reader(&longest).read_string(Compact).map(|s| s.len()),
            Ok(0x7fff)
        );
    }

    #[test]
    fn a_string_is_charged_its_decoded_length() {
        let budget = |allocation_budget| DecodeLimits {
            allocation_budget,
            ..DecodeLimits::default()
        };
        // One invalid byte decodes to U+FFFD, three bytes.
        let input = || Bytes::from_static(&[2, 0xff]);
        let mut exact = Reader::with_limits(input(), budget(3));
        assert_eq!(exact.read_string(Compact), Ok("\u{fffd}".to_owned()));
        assert_eq!(
            Reader::with_limits(input(), budget(2)).read_string(Compact),
            Err(error(
                0,
                DecodeErrorKind::LimitExceeded {
                    limit: Limit::AllocationBudget,
                    requested: 3,
                    allowed: 2
                }
            ))
        );
        // The budget is shared by every read.
        let mut two = Reader::with_limits(Bytes::from_static(&[3, b'a', b'b', 2, b'c']), budget(2));
        assert_eq!(two.read_string(Compact), Ok("ab".to_owned()));
        assert_eq!(
            two.read_string(Compact),
            Err(error(
                3,
                DecodeErrorKind::LimitExceeded {
                    limit: Limit::AllocationBudget,
                    requested: 1,
                    allowed: 0
                }
            ))
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
        let compact = compact.as_bytes().clone();
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

    fn framing() -> impl Strategy<Value = Framing> {
        proptest::prop_oneof![Just(Classic), Just(Compact)]
    }

    proptest! {
        #[test]
        fn a_lying_length_is_an_error_before_anything_is_read(
            classic in lying_int32_length(64),
            compact in lying_compact_length(64),
        ) {
            prop_assert!(reader(&classic).read_bytes(Classic).is_err());
            prop_assert!(reader(&compact).read_bytes(Compact).is_err());
            prop_assert!(reader(&compact).read_string(Compact).is_err());
        }

        #[test]
        fn a_string_cut_short_is_an_error(
            (framing, cut) in (framing(), ".{1,40}").prop_flat_map(|(framing, text)| {
                let bytes = written(|w| w.write_string(framing, &text));
                (Just(framing), cut_short(bytes))
            })
        ) {
            prop_assert!(reader(&cut).read_string(framing).is_err());
        }
    }

    proptest! {
        #[test]
        fn strings_decode_as_from_utf8_lossy_does(bytes in proptest::collection::vec(any::<u8>(), 0..40)) {
            let mut input = Writer::new();
            input.write_bytes(Compact, &bytes).unwrap();
            let expected = String::from_utf8_lossy(&bytes).into_owned();
            let decoded = Reader::new(input.into_bytes()).read_string(Compact).unwrap();
            prop_assert_eq!((decoded.capacity(), decoded), (expected.len(), expected));
        }

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
        fn bytes_and_records_round_trip(
            framing in framing(),
            value in proptest::collection::vec(any::<u8>(), 0..40),
            nullable in option::of(proptest::collection::vec(any::<u8>(), 0..40)),
        ) {
            let bytes = written(|w| {
                w.write_bytes(framing, &value)?;
                w.write_nullable_bytes(framing, nullable.as_deref())?;
                w.write_records(framing, nullable.clone().map(|bytes| RecordsField::new(Bytes::from(bytes))).as_ref())
            });
            let mut input = reader(&bytes);
            prop_assert_eq!(input.read_bytes(framing).map(|b| b.to_vec()), Ok(value));
            prop_assert_eq!(input.read_nullable_bytes(framing).map(|b| b.map(|b| b.to_vec())), Ok(nullable.clone()));
            prop_assert_eq!(input.read_records(framing).map(|b| b.map(|b| b.as_bytes().to_vec())), Ok(nullable));
            prop_assert_eq!(input.remaining(), 0);
        }
    }
}
