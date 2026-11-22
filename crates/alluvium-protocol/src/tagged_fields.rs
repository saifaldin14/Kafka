//! The tagged-field section that ends every struct in a flexible version: a count, then
//! for each field its tag, its size and its bytes, all as unsigned varints.
//!
//! The fields a schema defines are decoded by the generated code. Every other tag is kept
//! as a [`RawTaggedField`] and written back unchanged, so a field added by a newer
//! Kafka version passes through.

use bytes::Bytes;

use crate::error::{DecodeError, DecodeErrorKind, EncodeError};
use crate::reader::Reader;
use crate::writer::Writer;

/// A tagged field kept as its tag and undecoded bytes.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/types/RawTaggedField.java@4.4.0 RawTaggedField
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTaggedField {
    /// The field's tag.
    pub tag: u32,
    /// The field's bytes, a slice of the input it was read from.
    pub data: Bytes,
}

impl Reader {
    /// A whole tagged-field section, every field kept raw.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::NegativeTaggedFieldCount`] or
    /// [`DecodeErrorKind::TaggedFieldCountExceedsInput`] for an impossible count,
    /// [`DecodeErrorKind::NegativeTaggedFieldSize`] for an impossible size, and
    /// [`DecodeErrorKind::Truncated`] if the input ends inside the section.
    // Parity: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java@4.4.0 MessageDataGenerator.generateClassReader
    pub fn read_tagged_fields(&mut self) -> Result<Vec<RawTaggedField>, DecodeError> {
        let start = self.offset();
        let count = self.read_unsigned_varint()?.cast_signed();
        let count = usize::try_from(count).map_err(|_| {
            Reader::error_at(start, DecodeErrorKind::NegativeTaggedFieldCount { count })
        })?;
        if count > self.remaining() {
            return Err(Reader::error_at(
                start,
                DecodeErrorKind::TaggedFieldCountExceedsInput {
                    count,
                    remaining: self.remaining(),
                },
            ));
        }
        let mut fields = Vec::new();
        for _ in 0..count {
            let field_start = self.offset();
            let tag = self.read_unsigned_varint()?;
            let size = self.read_unsigned_varint()?.cast_signed();
            let size = usize::try_from(size).map_err(|_| {
                Reader::error_at(
                    field_start,
                    DecodeErrorKind::NegativeTaggedFieldSize { tag, size },
                )
            })?;
            fields.push(RawTaggedField {
                tag,
                data: self.take(size)?,
            });
        }
        Ok(fields)
    }
}

impl Writer {
    /// A tagged-field section holding `fields`, which must be in ascending tag order
    /// with no tag repeated.
    ///
    /// # Errors
    ///
    /// [`EncodeError::UnsortedRawTags`] for a tag not higher than the one before it, and
    /// [`EncodeError::LengthTooLong`] for a count or size that doesn't fit a varint.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/types/RawTaggedFieldWriter.java@4.4.0 RawTaggedFieldWriter.writeRawTags
    pub fn write_tagged_fields(&mut self, fields: &[RawTaggedField]) -> Result<(), EncodeError> {
        for pair in fields.windows(2) {
            if let [previous, next] = pair
                && next.tag <= previous.tag
            {
                return Err(EncodeError::UnsortedRawTags {
                    tag: next.tag,
                    previous: previous.tag,
                });
            }
        }
        self.write_unsigned_varint(varint_length(fields.len())?);
        for field in fields {
            self.write_unsigned_varint(field.tag);
            self.write_unsigned_varint(varint_length(field.data.len())?);
            self.put_slice(&field.data);
        }
        Ok(())
    }
}

/// A count or size as Kafka writes it: a non-negative Java `int`.
fn varint_length(length: usize) -> Result<u32, EncodeError> {
    i32::try_from(length)
        .map(i32::cast_unsigned)
        .map_err(|_| EncodeError::LengthTooLong { length })
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use proptest::collection::btree_map;
    use proptest::prelude::{any, prop_assert_eq, proptest};

    use super::RawTaggedField;
    use crate::error::{DecodeError, DecodeErrorKind, EncodeError};
    use crate::reader::Reader;
    use crate::writer::Writer;

    fn reader(bytes: &[u8]) -> Reader {
        Reader::new(Bytes::copy_from_slice(bytes))
    }

    fn field(tag: u32, data: &'static [u8]) -> RawTaggedField {
        RawTaggedField {
            tag,
            data: Bytes::from_static(data),
        }
    }

    #[test]
    fn a_section_is_a_count_then_tag_size_and_bytes() {
        let mut writer = Writer::new();
        writer
            .write_tagged_fields(&[field(0, &[7]), field(5, &[1, 2])])
            .unwrap();
        let bytes = writer.into_bytes();
        assert_eq!(&bytes[..], [2, 0, 1, 7, 5, 2, 1, 2]);
        assert_eq!(
            Reader::new(bytes).read_tagged_fields(),
            Ok(vec![field(0, &[7]), field(5, &[1, 2])])
        );
        assert_eq!(reader(&[0]).read_tagged_fields(), Ok(vec![]));
    }

    #[test]
    fn impossible_counts_and_sizes_are_rejected() {
        // 0xFFFFFFFF as an unsigned varint is -1 as a Java int.
        let negative = [0xff, 0xff, 0xff, 0xff, 0x0f];
        assert_eq!(
            reader(&negative).read_tagged_fields(),
            Err(DecodeError::new(
                0,
                DecodeErrorKind::NegativeTaggedFieldCount { count: -1 }
            ))
        );
        assert_eq!(
            reader(&[3, 0, 0]).read_tagged_fields(),
            Err(DecodeError::new(
                0,
                DecodeErrorKind::TaggedFieldCountExceedsInput {
                    count: 3,
                    remaining: 2
                }
            ))
        );
        let mut negative_size = vec![1, 4];
        negative_size.extend(negative);
        assert_eq!(
            reader(&negative_size).read_tagged_fields(),
            Err(DecodeError::new(
                1,
                DecodeErrorKind::NegativeTaggedFieldSize { tag: 4, size: -1 }
            ))
        );
        assert_eq!(
            reader(&[1, 4, 3, 9]).read_tagged_fields(),
            Err(DecodeError::new(
                3,
                DecodeErrorKind::Truncated {
                    needed: 3,
                    remaining: 1
                }
            ))
        );
    }

    #[test]
    fn unsorted_or_repeated_tags_are_not_written() {
        for (tags, previous, tag) in [([2, 1], 2, 1), ([3, 3], 3, 3)] {
            let fields = [field(tags[0], &[]), field(tags[1], &[])];
            assert_eq!(
                Writer::new().write_tagged_fields(&fields),
                Err(EncodeError::UnsortedRawTags { tag, previous })
            );
        }
    }

    proptest! {
        #[test]
        fn sections_round_trip(fields in btree_map(any::<u32>(), proptest::collection::vec(any::<u8>(), 0..20), 0..8)) {
            let fields: Vec<RawTaggedField> = fields
                .into_iter()
                .map(|(tag, data)| RawTaggedField { tag, data: Bytes::from(data) })
                .collect();
            let mut writer = Writer::new();
            writer.write_tagged_fields(&fields).unwrap();
            prop_assert_eq!(Reader::new(writer.into_bytes()).read_tagged_fields(), Ok(fields));
        }
    }
}
