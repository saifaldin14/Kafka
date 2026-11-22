//! Arrays: an element count, then the elements, decoded within the reader's limits.
//!
//! A declared count can lie, so it is checked before anything is allocated: it must
//! fit in the bytes left (each element takes at least one) and be within
//! [`DecodeLimits::max_array_length`](crate::DecodeLimits::max_array_length). Even
//! then, at most `MAX_PREALLOCATED_ARRAY_CAPACITY` elements are allocated up front,
//! and the rest only as they decode. Generated decoders read every array through
//! [`Reader::read_array`] or [`Reader::read_nullable_array`], so none of them
//! allocates from a declared count on its own.

use crate::error::{DecodeError, DecodeErrorKind};
use crate::framed::{Framing, Width};
use crate::limits::{Limit, MAX_PREALLOCATED_ARRAY_CAPACITY};
use crate::reader::Reader;

impl Reader {
    /// An array: its element count, then each element, read by `read_element`.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::UnexpectedNull`] for a null count,
    /// [`DecodeErrorKind::ArrayLongerThanInput`] or [`DecodeErrorKind::LimitExceeded`]
    /// for a count that can't be real, and any error from `read_element`.
    pub fn read_array<T>(
        &mut self,
        framing: Framing,
        read_element: impl FnMut(&mut Reader) -> Result<T, DecodeError>,
    ) -> Result<Vec<T>, DecodeError> {
        let start = self.offset();
        let length = self.read_present_length(framing, Width::Int)?;
        self.read_elements(start, length, read_element)
    }

    /// A nullable array: any negative count is null.
    ///
    /// # Errors
    ///
    /// As [`Reader::read_array`], except that a null count is not an error.
    pub fn read_nullable_array<T>(
        &mut self,
        framing: Framing,
        read_element: impl FnMut(&mut Reader) -> Result<T, DecodeError>,
    ) -> Result<Option<Vec<T>>, DecodeError> {
        let start = self.offset();
        match self.read_nullable_length(framing, Width::Int)? {
            Some(length) => self.read_elements(start, length, read_element).map(Some),
            None => Ok(None),
        }
    }

    /// `length` elements, after checking that the count can be real.
    // Parity: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java@4.4.0 MessageDataGenerator.generateVariableLengthReader
    fn read_elements<T>(
        &mut self,
        start: usize,
        length: usize,
        mut read_element: impl FnMut(&mut Reader) -> Result<T, DecodeError>,
    ) -> Result<Vec<T>, DecodeError> {
        if length > self.remaining() {
            return Err(Reader::error_at(
                start,
                DecodeErrorKind::ArrayLongerThanInput {
                    length,
                    remaining: self.remaining(),
                },
            ));
        }
        if length > self.limits.max_array_length {
            return Err(Reader::error_at(
                start,
                DecodeErrorKind::LimitExceeded {
                    limit: Limit::ArrayLength,
                    requested: length,
                    allowed: self.limits.max_array_length,
                },
            ));
        }
        let mut elements = Vec::with_capacity(length.min(MAX_PREALLOCATED_ARRAY_CAPACITY));
        while elements.len() < length {
            if elements.len() == elements.capacity() {
                // Double, as `push` would, but never past the declared count.
                let left = length.saturating_sub(elements.len());
                elements.reserve_exact(elements.len().min(left));
            }
            elements.push(read_element(self)?);
        }
        Ok(elements)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use proptest::option;
    use proptest::prelude::{any, prop_assert_eq, proptest};

    use crate::error::{DecodeError, DecodeErrorKind, EncodeError};
    use crate::framed::Framing::{self, Classic, Compact};
    use crate::limits::{DecodeLimits, Limit};
    use crate::reader::Reader;
    use crate::writer::Writer;

    fn reader(bytes: &[u8]) -> Reader {
        Reader::new(Bytes::copy_from_slice(bytes))
    }

    fn error(offset: usize, kind: DecodeErrorKind) -> DecodeError {
        DecodeError::new(offset, kind)
    }

    fn read_i8s(input: &mut Reader, framing: Framing) -> Result<Vec<i8>, DecodeError> {
        input.read_array(framing, Reader::read_i8)
    }

    /// The bytes of an array of `length` int8 elements, each 0.
    fn int8_array(framing: Framing, length: usize) -> Bytes {
        let mut writer = Writer::new();
        writer.write_array_length(framing, length).unwrap();
        writer.put_slice(&vec![0; length]);
        writer.into_bytes()
    }

    #[test]
    fn a_count_beyond_the_input_fails_before_any_element_is_read() {
        let never = |_: &mut Reader| -> Result<(), DecodeError> { unreachable!() };
        let too_long = DecodeErrorKind::ArrayLongerThanInput {
            length: i32::MAX.cast_unsigned() as usize,
            remaining: 4,
        };
        assert_eq!(
            reader(&[0x7f, 0xff, 0xff, 0xff, 1, 2, 3, 4]).read_array(Classic, never),
            Err(error(0, too_long.clone()))
        );
        // 0x80000000 - 1 wraps to i32::MAX, as in Kafka.
        let mut compact = Writer::new();
        compact.write_unsigned_varint(0x8000_0000);
        compact.put_slice(&[1, 2, 3, 4]);
        assert_eq!(
            Reader::new(compact.into_bytes()).read_array(Compact, never),
            Err(error(0, too_long))
        );
        assert_eq!(
            reader(&[0, 0, 0, 3, 1, 2]).read_array(Classic, never),
            Err(error(
                0,
                DecodeErrorKind::ArrayLongerThanInput {
                    length: 3,
                    remaining: 2
                }
            ))
        );
    }

    #[test]
    fn a_count_above_the_limit_fails_before_any_element_is_read() {
        let limits = DecodeLimits {
            max_array_length: 3,
            ..DecodeLimits::default()
        };
        let never = |_: &mut Reader| -> Result<(), DecodeError> { unreachable!() };
        let input = Bytes::from_static(&[0, 0, 0, 4, 1, 2, 3, 4]);
        assert_eq!(
            Reader::with_limits(input, limits).read_array(Classic, never),
            Err(error(
                0,
                DecodeErrorKind::LimitExceeded {
                    limit: Limit::ArrayLength,
                    requested: 4,
                    allowed: 3
                }
            ))
        );
        let at_limit = int8_array(Compact, 3);
        assert_eq!(
            read_i8s(&mut Reader::with_limits(at_limit, limits), Compact),
            Ok(vec![0; 3])
        );
    }

    #[test]
    fn kafkas_limit_is_one_million_elements() {
        let input = int8_array(Classic, 1_000_001);
        assert_eq!(
            read_i8s(&mut Reader::new(input), Classic),
            Err(error(
                0,
                DecodeErrorKind::LimitExceeded {
                    limit: Limit::ArrayLength,
                    requested: 1_000_001,
                    allowed: 1_000_000
                }
            ))
        );
        let at_limit = read_i8s(&mut Reader::new(int8_array(Classic, 1_000_000)), Classic);
        assert_eq!(at_limit.map(|elements| elements.len()), Ok(1_000_000));
    }

    #[test]
    fn arrays_longer_than_the_preallocation_decode_and_never_outgrow_their_count() {
        for length in [0, 1, 999, 1000, 1001, 5000] {
            let elements = read_i8s(&mut Reader::new(int8_array(Compact, length)), Compact);
            let elements = elements.unwrap();
            assert_eq!(
                (elements.len(), elements.capacity()),
                (length, length),
                "{length}"
            );
        }
    }

    #[test]
    fn null_arrays() {
        assert_eq!(
            reader(&[0xff, 0xff, 0xff, 0xff]).read_array(Classic, Reader::read_i8),
            Err(error(0, DecodeErrorKind::UnexpectedNull))
        );
        assert_eq!(
            reader(&[0xff, 0xff, 0xff, 0xff]).read_nullable_array(Classic, Reader::read_i8),
            Ok(None)
        );
        assert_eq!(
            reader(&[0]).read_nullable_array(Compact, Reader::read_i8),
            Ok(None)
        );
    }

    #[test]
    fn an_element_error_stops_the_array() {
        assert_eq!(
            reader(&[3, 0, 0, 1]).read_array(Compact, Reader::read_i16),
            Err(error(
                3,
                DecodeErrorKind::Truncated {
                    needed: 2,
                    remaining: 1
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
        fn arrays_round_trip(
            framing in framing(),
            values in proptest::collection::vec(any::<i32>(), 0..20),
            nullable in option::of(proptest::collection::vec(any::<i32>(), 0..20)),
        ) {
            let mut writer = Writer::new();
            let mut write = || -> Result<(), EncodeError> {
                writer.write_array_length(framing, values.len())?;
                for &value in &values {
                    writer.write_i32(value);
                }
                writer.write_nullable_array_length(framing, nullable.as_ref().map(Vec::len))?;
                for &value in nullable.iter().flatten() {
                    writer.write_i32(value);
                }
                Ok(())
            };
            write().unwrap();
            let mut input = Reader::new(writer.into_bytes());
            prop_assert_eq!(input.read_array(framing, Reader::read_i32), Ok(values));
            prop_assert_eq!(input.read_nullable_array(framing, Reader::read_i32), Ok(nullable));
            prop_assert_eq!(input.remaining(), 0);
        }
    }
}
