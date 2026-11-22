//! Tests of the generated decoders, on real messages and on Kafka's generator test
//! messages (`test_messages`).

use bytes::Bytes;
use proptest::prelude::*;

use crate::Framing::{Classic, Compact};
use crate::test_messages::nullable_struct_message::NullableStructMessage;
use crate::test_messages::simple_example_message::SimpleExampleMessage;
use crate::{DecodeError, Message, Reader, Writer};

fn decode<M: Message>(input: Bytes, version: i16) -> Result<M, DecodeError> {
    M::decode(&mut Reader::new(input), version)
}

proptest! {
    #[test]
    fn no_input_makes_any_decoder_panic(input in proptest::collection::vec(any::<u8>(), 0..256)) {
        let input = Bytes::from(input);
        for decode in crate::generated::DECODE_EVERY_VERSION
            .iter()
            .chain(crate::test_messages::DECODE_EVERY_VERSION)
        {
            decode(&input);
        }
    }
}

fn written(write: impl FnOnce(&mut Writer)) -> Bytes {
    let mut writer = Writer::new();
    write(&mut writer);
    writer.into_bytes()
}

mod versions_and_fields {
    use alluvium_types::Uuid;
    use bytes::Bytes;

    use super::{decode, written};
    use crate::Framing::Classic;
    use crate::generated::produce_request::ProduceRequest;
    use crate::generated::request_header::RequestHeader;
    use crate::{DecodeErrorKind, RecordsField, Writer};

    #[test]
    fn a_version_the_message_lacks_is_unsupported() {
        for version in [2, 14, -1] {
            let error = decode::<ProduceRequest>(Bytes::new(), version).unwrap_err();
            assert_eq!(
                error.kind,
                DecodeErrorKind::UnsupportedVersion {
                    message: "ProduceRequest",
                    version
                }
            );
        }
        let error = decode::<ProduceRequest>(Bytes::new(), 14).unwrap_err();
        assert_eq!(
            error.to_string(),
            "can't read version 14 of ProduceRequest at byte 0"
        );
    }

    #[test]
    fn request_header_v2_reads_client_id_as_a_classic_nullable_string() {
        let header = |client_id: Option<&str>| {
            written(|writer| {
                writer.write_i16(0);
                writer.write_i16(9);
                writer.write_i32(7);
                writer.write_nullable_string(Classic, client_id).unwrap();
                writer.write_unsigned_varint(0);
            })
        };
        let decoded = decode::<RequestHeader>(header(Some("app")), 2).unwrap();
        assert_eq!(decoded.client_id.as_deref(), Some("app"));
        assert_eq!(decoded.correlation_id, 7);
        assert_eq!(
            decode::<RequestHeader>(header(None), 2).unwrap().client_id,
            None
        );
    }

    /// A `ProduceRequest` v3 with one partition, whose topic name `name` writes.
    fn produce_v3(name: impl FnOnce(&mut Writer), records: &[u8]) -> Bytes {
        written(|writer| {
            writer.write_nullable_string(Classic, None).unwrap();
            writer.write_i16(-1);
            writer.write_i32(1000);
            writer.write_array_length(Classic, 1).unwrap();
            name(writer);
            writer.write_array_length(Classic, 1).unwrap();
            writer.write_i32(0);
            let records = RecordsField::new(Bytes::copy_from_slice(records));
            writer.write_records(Classic, Some(&records)).unwrap();
        })
    }

    #[test]
    fn produced_records_are_a_slice_of_the_frame() {
        let frame = produce_v3(
            |writer| writer.write_string(Classic, "orders").unwrap(),
            b"batch",
        );
        let request = decode::<ProduceRequest>(frame.clone(), 3).unwrap();
        let topic = &request.topic_data[0];
        assert_eq!(topic.name, "orders");
        assert_eq!(
            topic.topic_id,
            Uuid::ZERO,
            "absent before v13, so its default"
        );
        let records = topic.partition_data[0].records.as_ref().unwrap().as_bytes();
        assert_eq!(&records[..], b"batch");
        assert!(
            frame.as_ptr_range().contains(&records.as_ptr()),
            "copied, not sliced"
        );
    }

    #[test]
    fn a_null_in_a_version_where_the_field_is_not_nullable_is_an_error_naming_the_field() {
        let frame = produce_v3(
            |writer| writer.write_nullable_string(Classic, None).unwrap(),
            b"",
        );
        let error = decode::<ProduceRequest>(frame, 3).unwrap_err();
        assert_eq!(error.kind, DecodeErrorKind::UnexpectedNull);
        assert_eq!(error.field_path, ["TopicData", "Name"]);
    }

    #[test]
    fn a_struct_array_count_that_lies_fails_before_allocating() {
        let frame = written(|writer| {
            writer.write_nullable_string(Classic, None).unwrap();
            writer.write_i16(1);
            writer.write_i32(1000);
            writer.write_i32(i32::MAX);
        });
        let error = decode::<ProduceRequest>(frame, 3).unwrap_err();
        assert!(
            matches!(error.kind, DecodeErrorKind::ArrayLongerThanInput { .. }),
            "{error}"
        );
        assert_eq!(error.field_path, ["TopicData"]);
    }
}

mod structs_and_tags {
    use alluvium_types::Uuid;
    use bytes::Bytes;

    use super::{NullableStructMessage, SimpleExampleMessage, decode, written};
    use crate::Framing::{Classic, Compact};
    use crate::test_messages::nullable_struct_message::MyStruct4;
    use crate::test_messages::simple_example_message::MyStruct;
    use crate::{DecodeErrorKind, Writer};

    #[test]
    fn nullable_structs_read_an_int8_marker_and_absent_fields_keep_their_defaults() {
        let v0 = written(|writer| {
            writer.write_i8(-1);
            writer.write_string(Classic, "two").unwrap();
            writer.write_i32(2);
        });
        let message = decode::<NullableStructMessage>(v0, 0).unwrap();
        assert_eq!(message.nullable_struct, None);
        let second = message.nullable_struct2.unwrap();
        assert_eq!((second.my_string.as_str(), second.my_int), ("two", 2));
        assert_eq!(
            message.nullable_struct3, None,
            "absent in v0: its default, null"
        );
        assert_eq!(
            message.nullable_struct4,
            Some(MyStruct4::default()),
            "absent: empty struct"
        );
    }

    #[test]
    fn tagged_nullable_structs_read_an_unsigned_varint_marker() {
        let tagged_struct = written(|writer| {
            writer.write_unsigned_varint(1);
            writer.write_string(Compact, "three").unwrap();
            writer.write_i32(3);
            writer.write_unsigned_varint(0);
        });
        let v1 = written(|writer| {
            writer.write_i8(1);
            writer.write_string(Compact, "one").unwrap();
            writer.write_i32(1);
            writer.write_unsigned_varint(0);
            writer.write_i8(-1);
            writer.write_unsigned_varint(2);
            tagged_field(writer, 0, &tagged_struct);
            tagged_field(writer, 1, &[0]);
        });
        let message = decode::<NullableStructMessage>(v1, 1).unwrap();
        assert_eq!(message.nullable_struct.unwrap().my_string, "one");
        assert_eq!(message.nullable_struct2, None);
        assert_eq!(message.nullable_struct3.unwrap().my_string, "three");
        assert_eq!(message.nullable_struct4, None, "a zero marker is null");
    }

    fn tagged_field(writer: &mut Writer, tag: u32, data: &[u8]) {
        writer.write_unsigned_varint(tag);
        writer.write_unsigned_varint(u32::try_from(data.len()).unwrap());
        writer.put_slice(data);
    }

    /// `SimpleExampleMessage` v1 with every mandatory field, then `tags`.
    fn simple_example_v1(tags: &[(u32, &[u8])]) -> Bytes {
        written(|writer| {
            writer.write_uuid(Uuid::ZERO);
            writer.write_bytes(Compact, b"").unwrap();
            writer.write_nullable_bytes(Compact, None).unwrap();
            for _ in 0..2 {
                writer.write_i32(1);
                writer.write_i32(2);
                writer.write_unsigned_varint(0);
            }
            writer.write_u16(7);
            writer.write_u32(9);
            writer.write_unsigned_varint(u32::try_from(tags.len()).unwrap());
            for (tag, data) in tags {
                tagged_field(writer, *tag, data);
            }
        })
    }

    #[test]
    fn untagged_fields_absent_from_a_version_and_unsent_tags_keep_their_defaults() {
        let message = decode::<SimpleExampleMessage>(simple_example_v1(&[]), 1).unwrap();
        assert_eq!((message.my_uint16, message.my_uint32), (7, 9));
        assert_eq!(
            message.my_int16, 123,
            "tagged and not sent: the schema's default"
        );
        assert_eq!(message.my_float64.to_bits(), 12.34_f64.to_bits());
        assert_eq!(message.my_struct, MyStruct::default(), "only in v2+");
    }

    #[test]
    fn a_known_tag_outside_its_tagged_versions_fails_as_in_kafka() {
        let error =
            decode::<SimpleExampleMessage>(simple_example_v1(&[(8, b"\0")]), 1).unwrap_err();
        assert_eq!(
            error.kind,
            DecodeErrorKind::TagNotValidForVersion { tag: 8, version: 1 }
        );
        assert_eq!(error.kind.to_string(), "tag 8 is not valid for version 1");
        assert_eq!(error.field_path, ["myTaggedStruct"]);
    }

    #[test]
    fn a_repeated_known_tag_keeps_the_last_value_and_unknown_tags_are_all_kept() {
        let input = simple_example_v1(&[(2, &[0, 5]), (99, b"a"), (2, &[0, 6]), (99, b"b")]);
        let message = decode::<SimpleExampleMessage>(input, 1).unwrap();
        assert_eq!(message.my_int16, 6);
        let unknown: Vec<(u32, &[u8])> = message
            .unknown_tagged_fields
            .iter()
            .map(|field| (field.tag, &field.data[..]))
            .collect();
        assert_eq!(unknown, [(99, &b"a"[..]), (99, &b"b"[..])]);
    }
}

proptest! {
    // Every element type a vendored schema's primitive array uses, in both framings.
    #[test]
    fn primitive_arrays_decode_in_classic_and_compact_form(
        int8s in proptest::collection::vec(any::<i8>(), 0..8),
        int16s in proptest::collection::vec(any::<i16>(), 0..8),
        int32s in proptest::collection::vec(any::<i32>(), 0..8),
        int64s in proptest::collection::vec(any::<i64>(), 0..8),
        uuids in proptest::collection::vec(any::<(i64, i64)>(), 0..8),
        strings in proptest::collection::vec("[a-z]{0,6}", 0..8),
    ) {
        let uuids: Vec<alluvium_types::Uuid> =
            uuids.into_iter().map(|(most, least)| alluvium_types::Uuid::from_bits(most, least)).collect();
        for framing in [Classic, Compact] {
            let input = written(|writer| {
                writer.write_array_length(framing, int8s.len()).unwrap();
                for value in &int8s {
                    writer.write_i8(*value);
                }
                writer.write_array_length(framing, int16s.len()).unwrap();
                for value in &int16s {
                    writer.write_i16(*value);
                }
                writer.write_array_length(framing, int32s.len()).unwrap();
                for value in &int32s {
                    writer.write_i32(*value);
                }
                writer.write_array_length(framing, int64s.len()).unwrap();
                for value in &int64s {
                    writer.write_i64(*value);
                }
                writer.write_array_length(framing, uuids.len()).unwrap();
                for value in &uuids {
                    writer.write_uuid(*value);
                }
                writer.write_array_length(framing, strings.len()).unwrap();
                for value in &strings {
                    writer.write_string(framing, value).unwrap();
                }
            });
            let mut reader = Reader::new(input);
            prop_assert_eq!(&reader.read_array(framing, Reader::read_i8).unwrap(), &int8s);
            prop_assert_eq!(&reader.read_array(framing, Reader::read_i16).unwrap(), &int16s);
            prop_assert_eq!(&reader.read_array(framing, Reader::read_i32).unwrap(), &int32s);
            prop_assert_eq!(&reader.read_array(framing, Reader::read_i64).unwrap(), &int64s);
            prop_assert_eq!(&reader.read_array(framing, Reader::read_uuid).unwrap(), &uuids);
            let read = reader.read_array(framing, |reader| reader.read_string(framing)).unwrap();
            prop_assert_eq!(read, strings.clone());
            prop_assert_eq!(reader.remaining(), 0);
        }
    }
}
