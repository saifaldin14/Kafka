//! The tagged-field section that ends every struct in a flexible version: a count, then
//! for each field its tag, its size and its bytes, all as unsigned varints.
//!
//! The fields a schema defines are decoded by the generated code. Every other tag is kept
//! as a [`RawTaggedField`] and written back unchanged, so a field added by a newer
//! Kafka version passes through.

use bytes::Bytes;

use crate::error::{DecodeError, DecodeErrorKind, EncodeError};
use crate::limits::Limit;
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
    /// A whole tagged-field section, every field kept raw, in the order read.
    ///
    /// The order isn't checked: like Kafka's `Readable.readUnknownTaggedField`, this
    /// accepts tags out of order or repeated. [`Writer::write_tagged_fields`] writes only
    /// ascending tags, as Kafka's `RawTaggedFieldWriter` does, so such a section can be
    /// read but not written back.
    ///
    /// # Errors
    ///
    /// As [`Reader::read_tagged_field_count`] and [`Reader::read_tagged_field_header`],
    /// and [`DecodeErrorKind::Truncated`] if the input ends inside the section.
    pub fn read_tagged_fields(&mut self) -> Result<Vec<RawTaggedField>, DecodeError> {
        let start = self.offset();
        let count = self.read_tagged_field_count()?;
        self.read_elements(start, count, |reader| {
            let (tag, size) = reader.read_tagged_field_header()?;
            Ok(RawTaggedField {
                tag,
                data: reader.take_kept(size)?,
            })
        })
    }

    /// The field count that starts a tagged-field section.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::NegativeTaggedFieldCount`],
    /// [`DecodeErrorKind::TaggedFieldCountExceedsInput`] or
    /// [`DecodeErrorKind::LimitExceeded`] for a count that can't be real.
    // Parity: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java@4.4.0 MessageDataGenerator.generateClassReader
    pub fn read_tagged_field_count(&mut self) -> Result<usize, DecodeError> {
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
        if count > self.limits.max_tagged_field_count {
            return Err(Reader::error_at(
                start,
                DecodeErrorKind::LimitExceeded {
                    limit: Limit::TaggedFieldCount,
                    requested: count,
                    allowed: self.limits.max_tagged_field_count,
                },
            ));
        }
        Ok(count)
    }

    /// One tagged field's tag and size; its value follows.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::NegativeTaggedFieldSize`] for an impossible size.
    pub fn read_tagged_field_header(&mut self) -> Result<(u32, usize), DecodeError> {
        let start = self.offset();
        let tag = self.read_unsigned_varint()?;
        let size = self.read_unsigned_varint()?.cast_signed();
        let size = usize::try_from(size).map_err(|_| {
            Reader::error_at(
                start,
                DecodeErrorKind::NegativeTaggedFieldSize { tag, size },
            )
        })?;
        Ok((tag, size))
    }

    /// The value of the known tagged field `tag`, read by `read` from exactly the `size`
    /// bytes its header declared.
    ///
    /// `read` gets a reader over just those bytes, so it can't read into the next field,
    /// and it shares this reader's offsets and allocation budget.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if fewer than `size` bytes remain, any error of
    /// `read`, and [`DecodeErrorKind::TaggedFieldSizeMismatch`] if `read` leaves bytes
    /// unread.
    pub fn read_tagged_value<T>(
        &mut self,
        tag: u32,
        size: usize,
        read: impl FnOnce(&mut Reader) -> Result<T, DecodeError>,
    ) -> Result<T, DecodeError> {
        let start = self.offset();
        let mut value_reader = Reader {
            buf: self.take(size)?,
            total: start.saturating_add(size),
            limits: self.limits,
            budget_left: self.budget_left,
            kept: 0,
        };
        let value = read(&mut value_reader);
        self.budget_left = value_reader.budget_left;
        self.kept |= value_reader.kept;
        let value = value?;
        if value_reader.remaining() > 0 {
            return Err(Reader::error_at(
                start,
                DecodeErrorKind::TaggedFieldSizeMismatch {
                    tag,
                    declared: size,
                    read: size.saturating_sub(value_reader.remaining()),
                },
            ));
        }
        Ok(value)
    }

    /// Keeps the tagged field `tag`, whose `size` bytes come next, raw in `fields`.
    ///
    /// # Errors
    ///
    /// [`DecodeErrorKind::Truncated`] if fewer than `size` bytes remain, and
    /// [`DecodeErrorKind::LimitExceeded`] if keeping it would exceed the allocation
    /// budget.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java@4.4.0 Readable.readUnknownTaggedField
    pub fn read_unknown_tagged_field(
        &mut self,
        tag: u32,
        size: usize,
        fields: &mut Vec<RawTaggedField>,
    ) -> Result<(), DecodeError> {
        let start = self.offset();
        let data = self.take_kept(size)?;
        if fields.len() == fields.capacity() {
            // Grow as `Vec` would, doubling, but charge the whole new block before it is
            // allocated: while the fields move into it, the old block is live too.
            let additional = fields.capacity().max(1);
            let capacity = fields.capacity().saturating_add(additional);
            self.charge(start, capacity.saturating_mul(size_of::<RawTaggedField>()))?;
            fields.reserve_exact(additional);
        }
        fields.push(RawTaggedField { tag, data });
        Ok(())
    }

    /// The error for a known tag in a version where its field isn't tagged, at the
    /// current offset: Kafka's "Tag N is not valid for version V".
    pub fn tag_not_valid(&self, tag: u32, version: i16) -> DecodeError {
        Reader::error_at(
            self.offset(),
            DecodeErrorKind::TagNotValidForVersion { tag, version },
        )
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
    pub fn write_tagged_fields(&mut self, fields: &[RawTaggedField]) -> Result<(), EncodeError> {
        let mut raw = RawTaggedFieldWriter::new(fields)?;
        self.write_tagged_field_count(raw.len())?;
        raw.write_rest(self)
    }

    /// The field count that starts a tagged-field section.
    ///
    /// # Errors
    ///
    /// [`EncodeError::LengthTooLong`] for a count that doesn't fit a varint.
    pub fn write_tagged_field_count(&mut self, count: usize) -> Result<(), EncodeError> {
        self.write_unsigned_varint(varint_length(count)?);
        Ok(())
    }

    /// A known tagged field: its tag, the size of its value, then the value `write`
    /// writes. The value is written to a nested writer first, to learn its size.
    ///
    /// # Errors
    ///
    /// Any error of `write`, and [`EncodeError::LengthTooLong`] for a value whose size
    /// doesn't fit a varint.
    pub fn write_tagged_value(
        &mut self,
        tag: u32,
        write: impl FnOnce(&mut Writer) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError> {
        let mut value = self.nested();
        write(&mut value)?;
        self.write_unsigned_varint(tag);
        self.write_unsigned_varint(varint_length(value.len())?);
        self.append(&value);
        Ok(())
    }

    /// One raw tagged field.
    fn write_raw_tagged_field(&mut self, field: &RawTaggedField) -> Result<(), EncodeError> {
        self.write_unsigned_varint(field.tag);
        self.write_unsigned_varint(varint_length(field.data.len())?);
        self.put_slice(&field.data);
        Ok(())
    }
}

/// Writes a struct's unknown tagged fields between its known ones, so that the whole
/// section is in ascending tag order.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/types/RawTaggedFieldWriter.java@4.4.0 RawTaggedFieldWriter
#[derive(Debug)]
pub struct RawTaggedFieldWriter<'a> {
    /// The fields not yet written.
    fields: &'a [RawTaggedField],
}

impl<'a> RawTaggedFieldWriter<'a> {
    /// A writer of `fields`, which must be in ascending tag order with no tag repeated.
    ///
    /// # Errors
    ///
    /// [`EncodeError::UnsortedRawTags`] for a tag not higher than the one before it.
    pub fn new(fields: &'a [RawTaggedField]) -> Result<RawTaggedFieldWriter<'a>, EncodeError> {
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
        Ok(RawTaggedFieldWriter { fields })
    }

    /// The fields not yet written.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Whether every field has been written.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Writes the fields whose tags are below `next_defined`, the next tag the struct
    /// defines.
    ///
    /// # Errors
    ///
    /// [`EncodeError::RawTagIsDefined`] for a field whose tag is `next_defined`, and
    /// [`EncodeError::LengthTooLong`] for a size that doesn't fit a varint.
    #[inline]
    pub fn write_below(
        &mut self,
        writer: &mut Writer,
        next_defined: u32,
    ) -> Result<(), EncodeError> {
        while let Some((field, rest)) = self.fields.split_first() {
            if field.tag == next_defined {
                return Err(EncodeError::RawTagIsDefined { tag: field.tag });
            }
            if field.tag > next_defined {
                break;
            }
            writer.write_raw_tagged_field(field)?;
            self.fields = rest;
        }
        Ok(())
    }

    /// Writes the fields whose tags are below `tag`, a tag the struct defines, then the
    /// known field `tag` if it is `wanted`: its size, and the value `write` writes.
    ///
    /// # Errors
    ///
    /// As [`RawTaggedFieldWriter::write_below`] and [`Writer::write_tagged_value`].
    // Inlined so that a struct whose tagged fields are all defaults, the usual case, pays
    // for a few comparisons rather than a call per field; without the hint, whether it is
    // inlined depends on unrelated code in the same codegen unit.
    #[inline]
    pub fn write_known(
        &mut self,
        writer: &mut Writer,
        tag: u32,
        wanted: bool,
        write: impl FnOnce(&mut Writer) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError> {
        self.write_below(writer, tag)?;
        if wanted {
            writer.write_tagged_value(tag, write)?;
        }
        Ok(())
    }

    /// Writes every field not yet written.
    ///
    /// # Errors
    ///
    /// [`EncodeError::LengthTooLong`] for a size that doesn't fit a varint.
    pub fn write_rest(&mut self, writer: &mut Writer) -> Result<(), EncodeError> {
        for field in self.fields {
            writer.write_raw_tagged_field(field)?;
        }
        self.fields = &[];
        Ok(())
    }
}

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

    use super::{RawTaggedField, RawTaggedFieldWriter};
    use crate::error::{DecodeError, DecodeErrorKind, EncodeError};
    use crate::limits::{DecodeLimits, Limit};
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

    // PROTO-0104: kept unknown fields are slices of the input, so they retain it; a
    // field with no data holds nothing.
    #[test]
    fn unknown_fields_with_data_retain_the_input() {
        for (section, retained) in [(&[1, 0, 0][..], 0), (&[1, 0, 1, 7][..], 4), (&[0][..], 0)] {
            let mut input = reader(section);
            input.read_tagged_fields().unwrap();
            assert_eq!(input.retained_bytes(), retained, "{section:?}");
        }
        let mut input = reader(&[9, 9, 7]);
        let mut fields = Vec::new();
        input.read_unknown_tagged_field(4, 1, &mut fields).unwrap();
        assert_eq!(input.retained_bytes(), 3);
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
    fn kafkas_limit_is_ten_thousand_tagged_fields() {
        let section = |count: u32| {
            let mut writer = Writer::new();
            writer.write_unsigned_varint(count);
            // Tag 0 with size 0, two bytes per field: enough to pass the remaining check.
            writer.put_slice(&vec![0; 2 * usize::try_from(count).unwrap()]);
            writer.into_bytes()
        };
        // Rejected from the count, before any entry is read: tag 0 repeated would
        // otherwise decode.
        assert_eq!(
            Reader::new(section(10_001)).read_tagged_fields(),
            Err(DecodeError::new(
                0,
                DecodeErrorKind::LimitExceeded {
                    limit: Limit::TaggedFieldCount,
                    requested: 10_001,
                    allowed: 10_000
                }
            ))
        );
        let at_limit = Reader::new(section(10_000)).read_tagged_fields();
        assert_eq!(at_limit.map(|fields| fields.len()), Ok(10_000));
        let limits = DecodeLimits {
            max_tagged_field_count: 1,
            ..DecodeLimits::default()
        };
        assert_eq!(
            Reader::with_limits(section(2), limits).read_tagged_fields(),
            Err(DecodeError::new(
                0,
                DecodeErrorKind::LimitExceeded {
                    limit: Limit::TaggedFieldCount,
                    requested: 2,
                    allowed: 1
                }
            ))
        );
    }

    #[test]
    fn a_section_is_charged_to_the_budget_before_it_is_read() {
        let needed = 2 * size_of::<RawTaggedField>();
        let limits = DecodeLimits {
            allocation_budget: needed - 1,
            ..DecodeLimits::default()
        };
        assert_eq!(
            Reader::with_limits(Bytes::from_static(&[2, 0, 0, 1, 0]), limits).read_tagged_fields(),
            Err(DecodeError::new(
                0,
                DecodeErrorKind::LimitExceeded {
                    limit: Limit::AllocationBudget,
                    requested: needed,
                    allowed: needed - 1
                }
            ))
        );
    }

    #[test]
    fn tags_are_read_in_any_order_as_kafka_reads_them() {
        assert_eq!(
            reader(&[3, 5, 0, 1, 0, 1, 0]).read_tagged_fields(),
            Ok(vec![field(5, &[]), field(1, &[]), field(1, &[])])
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

    #[test]
    fn unknown_tags_are_written_between_known_ones_in_tag_order() {
        let unknown = [RawTaggedField {
            tag: 1,
            data: Bytes::from_static(b"u"),
        }];
        let mut writer = Writer::new();
        let mut raw = RawTaggedFieldWriter::new(&unknown).unwrap();
        writer.write_tagged_field_count(3).unwrap();
        for tag in [0, 2] {
            raw.write_known(&mut writer, tag, true, |writer| {
                writer.write_i8(7);
                Ok(())
            })
            .unwrap();
        }
        raw.write_rest(&mut writer).unwrap();
        let written = writer.into_bytes();
        assert_eq!(&written[..], [3, 0, 1, 7, 1, 1, b'u', 2, 1, 7]);
        let tags: Vec<u32> = Reader::new(written)
            .read_tagged_fields()
            .unwrap()
            .iter()
            .map(|field| field.tag)
            .collect();
        assert_eq!(tags, [0, 1, 2]);
    }

    #[test]
    fn an_unknown_field_cannot_use_a_defined_tag() {
        let unknown = [RawTaggedField {
            tag: 2,
            data: Bytes::new(),
        }];
        let mut raw = RawTaggedFieldWriter::new(&unknown).unwrap();
        let result = raw.write_known(&mut Writer::new(), 2, false, |_| Ok(()));
        assert_eq!(result, Err(EncodeError::RawTagIsDefined { tag: 2 }));
    }
}
