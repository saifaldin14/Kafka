//! Behavior of the generated message types that the generator's own tests can't show:
//! the helpers it writes, run against real messages.

use alluvium_protocol::generated::create_topics_request::{
    CreatableReplicaAssignment, CreatableTopic, CreateTopicsRequest,
};

fn topic(name: &str, num_partitions: i32) -> CreatableTopic {
    CreatableTopic {
        name: name.to_owned(),
        num_partitions,
        ..CreatableTopic::default()
    }
}

#[test]
fn keyed_arrays_find_by_key_in_wire_order_and_keep_duplicates() {
    let request = CreateTopicsRequest {
        topics: vec![topic("orders", 1), topic("payments", 2), topic("orders", 3)],
        ..CreateTopicsRequest::default()
    };
    let found = CreatableTopic::find(&request.topics, "orders").map(|topic| topic.num_partitions);
    assert_eq!(found, Some(1));
    let all: Vec<i32> = CreatableTopic::find_all(&request.topics, "orders")
        .map(|topic| topic.num_partitions)
        .collect();
    assert_eq!(all, [1, 3]);
    assert_eq!(CreatableTopic::find(&request.topics, "refunds"), None);
    assert_eq!(
        CreatableTopic::find_all(&request.topics, "refunds").count(),
        0
    );
}

#[test]
fn keys_of_integer_fields_compare_by_value() {
    let assignments = [
        CreatableReplicaAssignment {
            partition_index: 0,
            broker_ids: vec![1, 2],
            ..CreatableReplicaAssignment::default()
        },
        CreatableReplicaAssignment {
            partition_index: 1,
            broker_ids: vec![2, 3],
            ..CreatableReplicaAssignment::default()
        },
    ];
    let found = CreatableReplicaAssignment::find(&assignments, 1);
    assert_eq!(
        found.map(|assignment| assignment.broker_ids.clone()),
        Some(vec![2, 3])
    );
}

mod redaction {
    use alluvium_protocol::RecordsField;
    use alluvium_protocol::generated::alter_user_scram_credentials_request::{
        AlterUserScramCredentialsRequest, ScramCredentialUpsertion,
    };
    use alluvium_protocol::generated::describe_delegation_token_response::{
        DescribeDelegationTokenResponse, DescribedDelegationToken,
    };
    use alluvium_protocol::generated::produce_request::{
        PartitionProduceData, ProduceRequest, TopicProduceData,
    };
    use alluvium_protocol::generated::sasl_authenticate_request::SaslAuthenticateRequest;
    use bytes::Bytes;

    /// Both `Debug` forms of `value`, which must show none of `secrets`.
    fn assert_hidden(value: &impl std::fmt::Debug, secrets: &[&str]) {
        for printed in [format!("{value:?}"), format!("{value:#?}")] {
            for secret in secrets {
                assert!(!printed.contains(secret), "{secret} is in {printed}");
            }
        }
    }

    #[test]
    fn sasl_auth_bytes_print_as_stars() {
        let request = SaslAuthenticateRequest {
            auth_bytes: Bytes::from_static(b"\0alice\0hunter2"),
            ..SaslAuthenticateRequest::default()
        };
        assert_hidden(&request, &["hunter2", "alice"]);
        assert!(format!("{request:?}").contains("auth_bytes: ***"));
    }

    #[test]
    fn scram_salts_and_delegation_token_hmacs_print_as_stars() {
        let request = AlterUserScramCredentialsRequest {
            upsertions: vec![ScramCredentialUpsertion {
                name: "alice".to_owned(),
                salt: Bytes::from_static(b"salt-secret"),
                salted_password: Bytes::from_static(b"password-secret"),
                ..ScramCredentialUpsertion::default()
            }],
            ..AlterUserScramCredentialsRequest::default()
        };
        assert_hidden(&request, &["salt-secret", "password-secret"]);
        assert!(
            format!("{request:?}").contains("alice"),
            "other fields still print"
        );
        let response = DescribeDelegationTokenResponse {
            tokens: vec![DescribedDelegationToken {
                hmac: Bytes::from_static(b"hmac-secret"),
                ..DescribedDelegationToken::default()
            }],
            ..DescribeDelegationTokenResponse::default()
        };
        assert_hidden(&response, &["hmac-secret"]);
    }

    #[test]
    fn records_print_as_a_payload_summary() {
        let request = ProduceRequest {
            topic_data: vec![TopicProduceData {
                partition_data: vec![PartitionProduceData {
                    records: Some(RecordsField::new(Bytes::from_static(
                        b"card=4111111111111111",
                    ))),
                    ..PartitionProduceData::default()
                }],
                ..TopicProduceData::default()
            }],
            ..ProduceRequest::default()
        };
        assert_hidden(&request, &["card=", "4111111111111111"]);
        assert!(format!("{request:?}").contains("len: 21"));
    }
}

mod known_values {
    use alluvium_protocol::generated::fetch_request::FetchRequest;
    use alluvium_protocol::generated::fetch_response::PartitionData;
    use alluvium_protocol::generated::produce_request::ProduceRequest;
    use alluvium_protocol::{ErrorCode, UnknownErrorCode};
    use alluvium_types::{Acks, DeliveryError, IsolationLevel};

    #[test]
    fn error_codes_read_as_error_code_and_unknown_ones_keep_their_value() {
        let mut partition = PartitionData {
            error_code: 6,
            ..PartitionData::default()
        };
        assert_eq!(partition.error_code(), Ok(ErrorCode::NotLeaderOrFollower));
        partition.error_code = 999;
        assert_eq!(partition.error_code(), Err(UnknownErrorCode { code: 999 }));
        assert_eq!(partition.error_code, 999, "the raw value is kept");
    }

    #[test]
    fn isolation_level_and_acks_read_as_their_enums() {
        let fetch = FetchRequest {
            isolation_level: 1,
            ..FetchRequest::default()
        };
        assert_eq!(fetch.isolation_level(), Ok(IsolationLevel::ReadCommitted));
        let mut produce = ProduceRequest {
            acks: -1,
            ..ProduceRequest::default()
        };
        assert_eq!(produce.acks(), Ok(Acks::All));
        produce.acks = 2;
        assert_eq!(produce.acks(), Err(DeliveryError::InvalidAcks { value: 2 }));
    }
}
