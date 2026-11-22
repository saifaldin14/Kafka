// Portions of this file are ported from Apache Kafka 4.4.0 and modified by Alluvium;
// each is marked `// Ported from:`. Apache Kafka is Copyright The Apache Software
// Foundation and licensed under the Apache License 2.0 (LICENSES/Apache-2.0.txt).
// See NOTICE.

//! Kafka's tests of its generated messages, run against Alluvium's: the tricky cases of
//! tagged defaults, nullable structs, unknown tags, version downgrades and array bounds.

use bytes::Bytes;

use crate::encode_tests::round_trip;
use crate::{EncodeError, Message, RawTaggedField, Reader, Writer};

/// `message` written at `version` and read back.
fn round_trip_serde<M: Message>(message: &M, version: i16) -> M {
    let mut writer = Writer::new();
    message.encode(&mut writer, version).unwrap();
    M::decode(&mut Reader::new(writer.into_bytes()), version).unwrap()
}

fn encode<M: Message>(message: &M, version: i16) -> Result<(), EncodeError> {
    message.encode(&mut Writer::new(), version)
}

mod simple_example_message {
    use super::{encode, round_trip, round_trip_serde};
    use crate::EncodeError;
    use crate::test_messages::simple_example_message::{MyStruct, SimpleExampleMessage};

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/SimpleExampleMessageTest.java@4.4.0 (testMyStructUnsupportedVersion)
    #[test]
    fn a_struct_set_in_a_version_without_it_fails() {
        let message = SimpleExampleMessage {
            my_struct: MyStruct {
                struct_id: 10,
                ..MyStruct::default()
            },
            ..SimpleExampleMessage::default()
        };
        assert_eq!(
            encode(&message, 1),
            Err(EncodeError::NonDefaultFieldUnsupported {
                field: "myStruct",
                version: 1
            })
        );
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/SimpleExampleMessageTest.java@4.4.0 (testTaggedFieldsShouldSupportFlexibleVersionSubset)
    #[test]
    fn an_ignorable_tagged_field_is_dropped_below_its_tagged_versions() {
        let message = SimpleExampleMessage {
            tagged_long_flexible_version_subset: 15,
            ..SimpleExampleMessage::default()
        };
        round_trip(&message, 2..=2);
        assert_eq!(
            round_trip_serde(&message, 1),
            SimpleExampleMessage::default()
        );
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/MessageTest.java@4.4.0 (testLongTaggedString)
    #[test]
    fn a_tagged_string_longer_than_a_one_byte_varint_size_round_trips() {
        let message = SimpleExampleMessage {
            my_string: "a".repeat(1024),
            ..SimpleExampleMessage::default()
        };
        round_trip(&message, 1..=2);
    }
}

mod nullable_struct_message {
    use super::{round_trip, round_trip_serde};
    use crate::test_messages::nullable_struct_message::{
        MyStruct2, MyStruct4, NullableStructMessage,
    };
    use crate::{EncodeError, Message};

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/NullableStructMessageTest.java@4.4.0 (testDefaultValues)
    #[test]
    fn defaults_are_null_where_the_schema_says_so_and_empty_otherwise() {
        let message = NullableStructMessage::default();
        assert_eq!(message.nullable_struct, None);
        assert_eq!(message.nullable_struct2, Some(MyStruct2::default()));
        assert_eq!(message.nullable_struct3, None);
        assert_eq!(message.nullable_struct4, Some(MyStruct4::default()));
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/NullableStructMessageTest.java@4.4.0 (testNullForAllFields)
    #[test]
    fn every_struct_can_be_null_in_version_2() {
        let message = NullableStructMessage {
            nullable_struct: None,
            nullable_struct2: None,
            nullable_struct3: None,
            nullable_struct4: None,
            ..NullableStructMessage::default()
        };
        assert_eq!(round_trip_serde(&message, 2), message);
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/NullableStructMessageTest.java@4.4.0 (testNullableStruct2CanNotBeNullInVersion0)
    #[test]
    fn nullable_struct2_cannot_be_null_in_version_0() {
        let message = NullableStructMessage {
            nullable_struct2: None,
            ..NullableStructMessage::default()
        };
        assert_eq!(
            message.encode(&mut crate::Writer::new(), 0),
            Err(EncodeError::NullForNonNullable {
                field: "NullableStruct2"
            })
        );
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/NullableStructMessageTest.java@4.4.0 (testTaggedStructSize)
    #[test]
    fn a_tagged_struct_at_the_one_byte_varint_size_boundary_round_trips() {
        // The struct takes 127 bytes, the most a one-byte varint size holds; its non-null
        // marker makes the tagged value 128 bytes, so the size takes two.
        let nullable_struct4 = MyStruct4 {
            my_int: 4,
            my_string: "\0".repeat(121),
            ..MyStruct4::default()
        };
        let message = NullableStructMessage {
            nullable_struct: None,
            nullable_struct2: None,
            nullable_struct3: None,
            nullable_struct4: Some(nullable_struct4),
            ..NullableStructMessage::default()
        };
        round_trip(&message, 2..=2);
    }
}

mod message {
    use super::{Bytes, RawTaggedField, encode};
    use crate::EncodeError;
    use crate::generated::create_topics_request::CreateTopicsRequest;
    use crate::generated::heartbeat_request::HeartbeatRequest;

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/MessageTest.java@4.4.0 (testNonIgnorableFieldWithDefaultNull)
    #[test]
    fn a_non_ignorable_field_whose_default_is_null_must_be_null_where_absent() {
        let request = |group_instance_id: Option<&str>| HeartbeatRequest {
            group_id: "groupId".to_owned(),
            generation_id: 15,
            member_id: "memberId".to_owned(),
            group_instance_id: group_instance_id.map(str::to_owned),
            ..HeartbeatRequest::default()
        };
        assert_eq!(
            encode(&request(Some("instanceId")), 0),
            Err(EncodeError::NonDefaultFieldUnsupported {
                field: "GroupInstanceId",
                version: 0
            })
        );
        assert_eq!(encode(&request(None), 0), Ok(()));
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/MessageTest.java@4.4.0 (testUnknownTaggedFields)
    // Kafka's test writes version 0; CreateTopics starts at version 2 since Kafka 4.0.
    #[test]
    fn unknown_tagged_fields_need_a_flexible_version() {
        let mut request = CreateTopicsRequest::default();
        assert_eq!(encode(&request, 6), Ok(()));
        request.unknown_tagged_fields.push(RawTaggedField {
            tag: 1000,
            data: Bytes::from_static(&[1, 2, 3]),
        });
        assert_eq!(
            encode(&request, 2),
            Err(EncodeError::TaggedFieldsUnsupported { version: 2 })
        );
        assert_eq!(encode(&request, 6), Ok(()));
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/MessageTest.java@4.4.0 (testCompareWithUnknownTaggedFields)
    #[test]
    fn unknown_tagged_fields_take_part_in_equality() {
        let mut first = CreateTopicsRequest {
            timeout_ms: 123,
            ..CreateTopicsRequest::default()
        };
        let mut second = first.clone();
        assert_eq!(first, second);
        let field = RawTaggedField {
            tag: 0,
            data: Bytes::from_static(&[0]),
        };
        first.unknown_tagged_fields.push(field.clone());
        assert_ne!(first, second);
        second.unknown_tagged_fields.push(field);
        assert_eq!(first, second);
    }
}

mod simple_arrays_message {
    use bytes::{BufMut, Bytes, BytesMut};

    use crate::test_messages::simple_arrays_message::SimpleArraysMessage;
    use crate::test_messages::simple_keyed_arrays_message::SimpleKeyedArraysMessage;
    use crate::{DecodeError, DecodeErrorKind, Limit, Message, Reader};

    fn decode<M: Message>(input: impl Into<Bytes>, version: i16) -> Result<M, DecodeError> {
        M::decode(&mut Reader::new(input.into()), version)
    }

    /// An `int32` count, then `count` elements, each written by `put`.
    fn classic_array(count: usize, put: impl Fn(&mut BytesMut, usize)) -> Bytes {
        let mut buf = BytesMut::new();
        buf.put_i32(i32::try_from(count).unwrap());
        for index in 0..count {
            put(&mut buf, index);
        }
        buf.freeze()
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/SimpleArraysMessageTest.java@4.4.0 (testArrayBoundsChecking)
    #[test]
    fn a_first_array_longer_than_the_input_fails() {
        let input = [0x7f, 0, 0, 0, 0, 0, 0, 0];
        let error = decode::<SimpleArraysMessage>(input.to_vec(), 2).unwrap_err();
        assert_eq!(
            error.kind,
            DecodeErrorKind::ArrayLongerThanInput {
                length: 126,
                remaining: 7
            }
        );
        assert_eq!(error.field_path, ["Goats"]);
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/SimpleArraysMessageTest.java@4.4.0 (testArrayBoundsCheckingOtherArray)
    #[test]
    fn a_second_array_longer_than_the_input_fails() {
        let input = [0x01, 0x7e, 0, 0, 0, 0, 0, 0];
        let error = decode::<SimpleArraysMessage>(input.to_vec(), 2).unwrap_err();
        assert_eq!(
            error.kind,
            DecodeErrorKind::ArrayLongerThanInput {
                length: 125,
                remaining: 6
            }
        );
        assert_eq!(error.field_path, ["Sheep"]);
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/SimpleArraysMessageTest.java@4.4.0 (testDeclaredLengthAboveInitialCapacityStillParses)
    #[test]
    fn an_array_longer_than_the_preallocated_capacity_decodes() {
        let input = classic_array(1010, |buf, index| {
            buf.put_i32(i32::try_from(index).unwrap());
        });
        let message = decode::<SimpleArraysMessage>(input, 0).unwrap();
        assert_eq!(message.sheep.len(), 1010);
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/SimpleArraysMessageTest.java@4.4.0 (testKeyedDeclaredLengthAboveInitialCapacityStillParses)
    #[test]
    fn a_keyed_array_longer_than_the_preallocated_capacity_decodes() {
        let input = classic_array(1010, |buf, index| {
            let key = i32::try_from(index).unwrap();
            buf.put_i32(key);
            buf.put_i32(key.checked_mul(2).unwrap());
        });
        let message = decode::<SimpleKeyedArraysMessage>(input, 0).unwrap();
        assert_eq!(message.keyed_structs.len(), 1010);
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/SimpleArraysMessageTest.java@4.4.0 (testArrayLengthAboveMaxIsRejected)
    #[test]
    fn an_array_above_the_maximum_length_fails() {
        let count = 1_000_001;
        let error = decode::<SimpleArraysMessage>(classic_array(count, |buf, _| buf.put_u8(0)), 0)
            .unwrap_err();
        assert_eq!(
            error.kind,
            DecodeErrorKind::LimitExceeded {
                limit: Limit::ArrayLength,
                requested: count,
                allowed: 1_000_000
            }
        );
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/message/SimpleArraysMessageTest.java@4.4.0 (testKeyedArrayLengthAboveMaxIsRejected)
    #[test]
    fn a_keyed_array_above_the_maximum_length_fails() {
        let count = 1_000_001;
        let error =
            decode::<SimpleKeyedArraysMessage>(classic_array(count, |buf, _| buf.put_u8(0)), 0)
                .unwrap_err();
        assert_eq!(
            error.kind,
            DecodeErrorKind::LimitExceeded {
                limit: Limit::ArrayLength,
                requested: count,
                allowed: 1_000_000
            }
        );
    }
}
