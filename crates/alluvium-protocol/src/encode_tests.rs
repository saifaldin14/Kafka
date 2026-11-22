//! Tests of the generated encoders, on real messages and on Kafka's generator test
//! messages (`test_messages`).

use std::fmt::Debug;
use std::ops::RangeInclusive;

use bytes::Bytes;

use crate::{EncodeError, Message, Reader, Writer};

/// Encodes `message` at each of `versions`. Where that succeeds, its `encoded_size` is
/// the length written and decoding gives `message` back; where it fails, `encoded_size`
/// fails the same way.
pub(crate) fn round_trip<M: Message + PartialEq + Debug>(
    message: &M,
    versions: RangeInclusive<i16>,
) {
    for version in versions {
        match encode(message, version) {
            Ok(bytes) => {
                let size = message.encoded_size(version).unwrap();
                assert_eq!(size.total, bytes.len(), "version {version}");
                let decoded = M::decode(&mut Reader::new(bytes), version).unwrap();
                assert_eq!(&decoded, message, "version {version}");
            }
            Err(error) => assert_eq!(message.encoded_size(version), Err(error)),
        }
    }
}

fn encode<M: Message>(message: &M, version: i16) -> Result<Bytes, EncodeError> {
    let mut writer = Writer::new();
    message.encode(&mut writer, version)?;
    Ok(writer.into_bytes())
}

#[test]
fn every_message_round_trips_its_default_at_every_version() {
    for check in crate::generated::ROUND_TRIP_DEFAULT_EVERY_VERSION
        .iter()
        .chain(crate::test_messages::ROUND_TRIP_DEFAULT_EVERY_VERSION)
    {
        check();
    }
}

mod versions_and_fields {
    use bytes::Bytes;

    use super::{encode, round_trip};
    use crate::Framing::Classic;
    use crate::generated::produce_request::{
        PartitionProduceData, ProduceRequest, TopicProduceData,
    };
    use crate::generated::produce_response::{
        BatchIndexAndErrorMessage, PartitionProduceResponse, ProduceResponse, TopicProduceResponse,
    };
    use crate::generated::request_header::RequestHeader;
    use crate::test_messages::nullable_struct_message::{MyStruct, NullableStructMessage};
    use crate::test_messages::simple_example_message::SimpleExampleMessage;
    use crate::{EncodeError, Message, RecordsField, Writer};

    #[test]
    fn a_version_the_message_lacks_is_unsupported() {
        let error = encode(&ProduceRequest::default(), 14).unwrap_err();
        assert_eq!(
            error,
            EncodeError::UnsupportedVersion {
                message: "ProduceRequest",
                version: 14
            }
        );
        assert_eq!(
            error.to_string(),
            "can't write version 14 of ProduceRequest"
        );
    }

    #[test]
    fn request_header_v2_writes_client_id_as_a_classic_nullable_string() {
        let header = RequestHeader {
            client_id: Some("app".to_owned()),
            ..RequestHeader::default()
        };
        let mut expected = Writer::new();
        expected.write_i16(0);
        expected.write_i16(0);
        expected.write_i32(0);
        expected
            .write_nullable_string(Classic, Some("app"))
            .unwrap();
        expected.write_unsigned_varint(0);
        assert_eq!(encode(&header, 2).unwrap(), expected.into_bytes());
    }

    #[test]
    fn none_where_the_version_needs_a_value_is_an_error_naming_the_field() {
        let message = NullableStructMessage {
            nullable_struct2: None,
            ..NullableStructMessage::default()
        };
        assert_eq!(
            encode(&message, 0),
            Err(EncodeError::NullForNonNullable {
                field: "NullableStruct2"
            })
        );
        round_trip(&message, 1..=2);
    }

    #[test]
    fn nullable_structs_round_trip_with_and_without_values() {
        let value = MyStruct {
            my_string: "s".to_owned(),
            my_int: 1,
            ..MyStruct::default()
        };
        for nullable_struct in [None, Some(value)] {
            let message = NullableStructMessage {
                nullable_struct,
                ..NullableStructMessage::default()
            };
            round_trip(&message, 0..=2);
        }
    }

    #[test]
    fn an_ignorable_field_the_version_lacks_is_dropped_and_another_fails() {
        let response = ProduceResponse {
            responses: vec![TopicProduceResponse {
                partition_responses: vec![PartitionProduceResponse {
                    record_errors: vec![BatchIndexAndErrorMessage {
                        batch_index: 3,
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..ProduceResponse::default()
        };
        let v7 = encode(&response, 7).unwrap();
        let decoded = ProduceResponse::decode(&mut crate::Reader::new(v7), 7).unwrap();
        assert!(
            decoded.responses[0].partition_responses[0]
                .record_errors
                .is_empty(),
            "dropped at v7"
        );
        round_trip(&response, 8..=13);

        let message = SimpleExampleMessage {
            process_id: alluvium_types::Uuid::from_bits(1, 2),
            ..SimpleExampleMessage::default()
        };
        assert_eq!(
            encode(&message, 0),
            Err(EncodeError::NonDefaultFieldUnsupported {
                field: "processId",
                version: 0
            })
        );
    }

    #[test]
    fn unknown_tagged_fields_in_a_version_without_them_fail() {
        let mut message = SimpleExampleMessage::default();
        message.unknown_tagged_fields.push(crate::RawTaggedField {
            tag: 99,
            data: Bytes::new(),
        });
        assert_eq!(
            encode(&message, 0),
            Err(EncodeError::TaggedFieldsUnsupported { version: 0 })
        );
    }

    #[test]
    fn records_are_counted_as_zero_copy_bytes() {
        let request = ProduceRequest {
            topic_data: vec![TopicProduceData {
                partition_data: vec![PartitionProduceData {
                    records: Some(RecordsField::new(Bytes::from_static(b"record batch"))),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..ProduceRequest::default()
        };
        let size = request.encoded_size(3).unwrap();
        assert_eq!(size.zero_copy, 12);
        assert_eq!(size.total, encode(&request, 3).unwrap().len());
        round_trip(&request, 3..=12);
    }
}

mod tagged {
    use bytes::Bytes;

    use super::encode;
    use crate::Framing::Compact;
    use crate::test_messages::simple_example_message::SimpleExampleMessage;
    use crate::{Message, Reader, Writer};

    fn tagged_field(writer: &mut Writer, tag: u32, data: &[u8]) {
        writer.write_unsigned_varint(tag);
        writer.write_unsigned_varint(u32::try_from(data.len()).unwrap());
        writer.put_slice(data);
    }

    /// `SimpleExampleMessage` v1 bytes: unknown tags in each common struct, and `tags`
    /// in the message's own section.
    fn simple_example_v1(tags: &[(u32, &[u8])]) -> Bytes {
        let mut writer = Writer::new();
        writer.write_uuid(alluvium_types::Uuid::ZERO);
        writer.write_bytes(Compact, b"").unwrap();
        writer.write_nullable_bytes(Compact, None).unwrap();
        for nested_tag in [5, 6] {
            writer.write_i32(123);
            writer.write_i32(123);
            writer.write_unsigned_varint(1);
            tagged_field(&mut writer, nested_tag, b"nested");
        }
        writer.write_u16(33_000);
        writer.write_u32(1_234_567);
        writer.write_unsigned_varint(u32::try_from(tags.len()).unwrap());
        for (tag, data) in tags {
            tagged_field(&mut writer, *tag, data);
        }
        writer.into_bytes()
    }

    #[test]
    fn unknown_tags_round_trip_byte_for_byte_at_every_level() {
        let input = simple_example_v1(&[(2, &[0, 5]), (99, b"a"), (100, b"b")]);
        let message = SimpleExampleMessage::decode(&mut Reader::new(input.clone()), 1).unwrap();
        assert_eq!(message.my_int16, 5);
        assert_eq!(message.unknown_tagged_fields.len(), 2);
        assert_eq!(message.my_common_struct.unknown_tagged_fields[0].tag, 5);
        assert_eq!(encode(&message, 1).unwrap(), input);
    }

    #[test]
    fn tagged_fields_at_their_defaults_are_not_written() {
        let message =
            SimpleExampleMessage::decode(&mut Reader::new(simple_example_v1(&[])), 1).unwrap();
        let input = simple_example_v1(&[]);
        assert_eq!(encode(&message, 1).unwrap(), input);
    }
}

#[test]
fn version_prefixed_data_round_trips_and_a_negative_version_fails() {
    use crate::generated::data::end_txn_marker::EndTxnMarker;
    use crate::{DecodeErrorKind, decode_version_prefixed, encode_version_prefixed};

    let marker = EndTxnMarker {
        coordinator_epoch: 7,
    };
    let bytes = encode_version_prefixed(&marker, 0).unwrap();
    assert_eq!(&bytes[..2], [0, 0]);
    let (version, decoded) =
        decode_version_prefixed::<EndTxnMarker>(&mut Reader::new(bytes)).unwrap();
    assert_eq!((version, decoded), (0, marker));
    let negative = Bytes::from_static(&[0xff, 0xff]);
    let error = decode_version_prefixed::<EndTxnMarker>(&mut Reader::new(negative)).unwrap_err();
    assert!(
        matches!(
            error.kind,
            DecodeErrorKind::UnsupportedVersion { version: -1, .. }
        ),
        "{error}"
    );
}

/// Runs `round_trip` on values from `arbitrary` at each of `versions`: a generated
/// message encodes, its `encoded_size` is the length written, and it decodes back.
pub(crate) fn round_trip_arbitrary<M: Message + PartialEq + Debug>(
    arbitrary: fn(i16) -> proptest::strategy::BoxedStrategy<M>,
    versions: crate::Versions,
) {
    use proptest::test_runner::{Config, TestRunner};

    let (Some(lowest), Some(highest)) = (versions.lowest(), versions.highest()) else {
        return;
    };
    for version in lowest..=highest {
        let mut runner = TestRunner::new(Config {
            cases: 32,
            failure_persistence: None,
            ..Config::default()
        });
        let result = runner.run(&arbitrary(version), |message| {
            let bytes = encode(&message, version).map_err(|error| {
                proptest::test_runner::TestCaseError::fail(format!("version {version}: {error}"))
            })?;
            let size = message
                .encoded_size(version)
                .map_err(|error| proptest::test_runner::TestCaseError::fail(error.to_string()))?;
            proptest::prop_assert_eq!(size.total, bytes.len());
            let decoded = M::decode(&mut Reader::new(bytes), version).map_err(|error| {
                proptest::test_runner::TestCaseError::fail(format!("version {version}: {error}"))
            })?;
            proptest::prop_assert_eq!(&decoded, &message, "version {}", version);
            Ok(())
        });
        if let Err(failure) = result {
            panic!("version {version}: {failure}");
        }
    }
}
