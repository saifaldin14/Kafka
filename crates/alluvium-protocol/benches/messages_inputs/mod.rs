//! The messages both `messages` benchmark targets measure, so wall time (criterion) and
//! instruction counts (gungraun) describe the same work.

use alluvium_protocol::generated::fetch_response::{
    FetchResponse, FetchableTopicResponse, PartitionData,
};
use alluvium_protocol::generated::produce_request::{
    PartitionProduceData, ProduceRequest, TopicProduceData,
};
use alluvium_protocol::{Message, Reader, RecordsField, Writer};
use alluvium_types::{SeededRng, Uuid};
use bytes::Bytes;

/// The `ProduceRequest` version measured: the highest in Kafka 4.4.0.
pub(crate) const PRODUCE_REQUEST_VERSION: i16 = 13;

/// The `FetchResponse` version measured: the highest in Kafka 4.4.0.
pub(crate) const FETCH_RESPONSE_VERSION: i16 = 18;

/// The bytes of one partition's record batch. The codecs carry records as an opaque
/// slice, so their size changes only the copy into an encoded message.
const RECORDS_SIZE: usize = 128;

/// A record batch's worth of bytes, from a fixed seed.
fn records() -> RecordsField {
    let mut rng = SeededRng::from_seed(0x7265_636f_7264_7300);
    let mut bytes = vec![0; RECORDS_SIZE];
    rng.fill_bytes(&mut bytes);
    RecordsField::new(Bytes::from(bytes))
}

/// An index for each of `partitions` partitions.
fn indexes(partitions: usize) -> impl Iterator<Item = i32> {
    0..i32::try_from(partitions).unwrap_or(i32::MAX)
}

/// A `ProduceRequest` with one batch for each of `partitions` partitions of one topic.
pub(crate) fn produce_request(partitions: usize) -> ProduceRequest {
    let records = records();
    ProduceRequest {
        acks: -1,
        timeout_ms: 30_000,
        topic_data: vec![TopicProduceData {
            topic_id: Uuid::from_bits(0x0123_4567_89ab_cdef, 0x0fed_cba9_8765_4321),
            partition_data: indexes(partitions)
                .map(|index| PartitionProduceData {
                    index,
                    records: Some(records.clone()),
                    ..PartitionProduceData::default()
                })
                .collect(),
            ..TopicProduceData::default()
        }],
        ..ProduceRequest::default()
    }
}

/// A `FetchResponse` with one batch for each of `partitions` partitions of one topic.
pub(crate) fn fetch_response(partitions: usize) -> FetchResponse {
    let records = records();
    FetchResponse {
        responses: vec![FetchableTopicResponse {
            topic_id: Uuid::from_bits(0x0123_4567_89ab_cdef, 0x0fed_cba9_8765_4321),
            partitions: indexes(partitions)
                .map(|partition_index| PartitionData {
                    partition_index,
                    high_watermark: 1_000_000,
                    last_stable_offset: 1_000_000,
                    log_start_offset: 0,
                    records: Some(records.clone()),
                    ..PartitionData::default()
                })
                .collect(),
            ..FetchableTopicResponse::default()
        }],
        ..FetchResponse::default()
    }
}

/// `message` encoded at `version`.
///
/// # Panics
///
/// If `message` doesn't encode at `version` or decode back unchanged: the benchmarks
/// would then measure a failure path instead of the codec.
pub(crate) fn encoded<M: Message + PartialEq>(message: &M, version: i16) -> Bytes {
    let mut writer = Writer::new();
    assert!(
        message.encode(&mut writer, version).is_ok(),
        "the benchmark input encodes"
    );
    let bytes = writer.into_bytes();
    assert!(
        M::decode(&mut Reader::new(bytes.clone()), version).as_ref() == Ok(message),
        "the benchmark input decodes back unchanged"
    );
    bytes
}
