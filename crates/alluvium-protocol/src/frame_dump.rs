//! Reading one whole Kafka frame for debugging tools such as `alluvium dump frame`
//! (PROTO-F13): its `int32` size prefix, its header, and its body as JSON
//! ([`Message::write_json`]).
//!
//! A request frame says its own API and version in its header. A response frame doesn't,
//! so the caller supplies them, from the request it answers. Nothing here is fatal to the
//! dump:
//! - a frame shorter than its prefix declares is read as far as it goes and reported as
//!   truncated;
//! - a header or body that doesn't decode is reported with the byte offset, counted from
//!   the frame's first byte, and the field path of the failure;
//! - bytes after the body, or after the declared frame, are counted.
//!
//! Only an input too short to hold the size prefix, or a negative size, has no dump.

use bytes::Bytes;

use crate::generated::api_versions_response::ApiVersionsResponse;
use crate::{
    ApiKey, DecodeError, DecodedRequestHeader, HeaderError, Message, Reader, decode_request_header,
    decode_response_header,
};

/// Decodes a body at a version and writes it as JSON.
pub(crate) type BodyJson = fn(&mut Reader, i16) -> Result<String, DecodeError>;

/// [`BodyJson`] for the message `M`.
pub(crate) fn body_json<M: Message>(
    reader: &mut Reader,
    version: i16,
) -> Result<String, DecodeError> {
    M::decode(reader, version).map(|message| message.to_json(version))
}

/// What a frame's header says.
#[derive(Debug, Clone, PartialEq)]
pub enum FrameHeader {
    /// A request header, with the API and header version it selects.
    Request(DecodedRequestHeader),
    /// A response header, for the API and version the caller supplied.
    Response {
        /// The API the response is for.
        api: ApiKey,
        /// The version of the request it answers.
        version: i16,
        /// The correlation id, which matches the request's.
        correlation_id: i32,
    },
}

/// One frame as read (PROTO-0142).
#[derive(Debug, Clone, PartialEq)]
pub struct FrameDump {
    /// The size the `int32` prefix declares: the bytes after it.
    pub declared_size: usize,
    /// The bytes after the prefix that the input holds, up to `declared_size`; fewer means
    /// the frame is truncated.
    pub present: usize,
    /// The header, or why it didn't decode.
    pub header: Result<FrameHeader, HeaderError>,
    /// The body as one line of JSON, or why it didn't decode; `None` when the header
    /// didn't decode, or names an API without message types.
    pub body: Option<Result<String, DecodeError>>,
    /// The bytes of the frame after the body.
    pub trailing: usize,
    /// The bytes of the input after the declared frame.
    pub beyond_frame: usize,
}

impl FrameDump {
    /// Whether the input holds fewer bytes than the prefix declares.
    pub fn is_truncated(&self) -> bool {
        self.present < self.declared_size
    }
}

/// Why an input has no frame dump at all.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FrameError {
    /// Fewer than the four bytes of the size prefix.
    #[error("{length} byte(s) can't hold the 4-byte size prefix of a frame")]
    NoSizePrefix {
        /// The input's length.
        length: usize,
    },
    /// The size prefix is negative.
    #[error("the size prefix is {size}, a negative size")]
    NegativeSize {
        /// The size as read.
        size: i32,
    },
}

/// The request frame at the start of `input`.
///
/// # Errors
///
/// [`FrameError`] if `input` has no size prefix, or a negative one.
pub fn dump_request_frame(input: &Bytes) -> Result<FrameDump, FrameError> {
    dump(input, |reader| {
        let decoded = decode_request_header(reader)?;
        let version = decoded.header.request_api_version;
        let body = decoded
            .api
            .body_json()
            .map(|(request, _)| (request, version));
        Ok((FrameHeader::Request(decoded), body))
    })
}

/// The response frame at the start of `input`, which answers a request for `api` at
/// `version`.
///
/// # Errors
///
/// [`FrameError`] if `input` has no size prefix, or a negative one.
pub fn dump_response_frame(
    input: &Bytes,
    api: ApiKey,
    version: i16,
) -> Result<FrameDump, FrameError> {
    dump(input, |reader| {
        let correlation_id = decode_response_header(api, version, reader)?;
        let body = api.body_json().map(|(_, response)| (response, version));
        Ok((
            FrameHeader::Response {
                api,
                version,
                correlation_id,
            },
            body,
        ))
    })
}

/// The header read by `header`, which also picks the body's decoder and version, and
/// the body after it.
fn dump(
    input: &Bytes,
    header: impl FnOnce(&mut Reader) -> Result<(FrameHeader, Option<(BodyJson, i16)>), HeaderError>,
) -> Result<FrameDump, FrameError> {
    let length = input.len();
    let size = input
        .first_chunk::<4>()
        .map(|prefix| i32::from_be_bytes(*prefix))
        .ok_or(FrameError::NoSizePrefix { length })?;
    let declared_size = usize::try_from(size).map_err(|_| FrameError::NegativeSize { size })?;
    let after_prefix = length.saturating_sub(4);
    let present = after_prefix.min(declared_size);
    let frame = input.slice(..4_usize.saturating_add(present));
    let mut reader = Reader::new(frame.clone());
    // Read past the prefix, so that offsets count from the frame's first byte.
    reader
        .read_i32()
        .map_err(|_| FrameError::NoSizePrefix { length })?;
    let (header, body) = match header(&mut reader) {
        Ok((header, body)) => (Ok(header), body),
        Err(error) => (Err(error), None),
    };
    let start = reader.offset();
    let body = body.map(|(decode, version)| {
        let decoded = decode(&mut reader, version);
        if decoded.is_err() && is_api_versions_response(&header) && version != 0 {
            // A server that doesn't serve the version asked answers at version 0 (KIP-511).
            // Parity: clients/src/main/java/org/apache/kafka/common/requests/ApiVersionsResponse.java@4.4.0 ApiVersionsResponse.parse
            reader = Reader::new(frame.clone());
            // Skips the prefix and header again, so that offsets still count from the frame's start.
            reader.take(start)?;
            return body_json::<ApiVersionsResponse>(&mut reader, 0);
        }
        decoded
    });
    Ok(FrameDump {
        declared_size,
        present,
        header,
        body,
        trailing: reader.remaining(),
        beyond_frame: after_prefix.saturating_sub(declared_size),
    })
}

fn is_api_versions_response(header: &Result<FrameHeader, HeaderError>) -> bool {
    matches!(
        header,
        Ok(FrameHeader::Response {
            api: ApiKey::ApiVersions,
            ..
        })
    )
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::{FrameDump, FrameError, FrameHeader, dump_request_frame, dump_response_frame};
    use crate::generated::api_versions_response::{ApiVersion, ApiVersionsResponse};
    use crate::generated::fetch_response::{FetchResponse, FetchableTopicResponse};
    use crate::generated::produce_request::{ProduceRequest, TopicProduceData};
    use crate::generated::request_header::RequestHeader;
    use crate::{
        ApiKey, DecodeErrorKind, EncodedFrame, HeaderError, Message, Segment, Writer,
        encode_request, encode_response,
    };

    /// `frame` with its size prefix, as on the wire.
    fn prefixed(frame: &EncodedFrame) -> Vec<u8> {
        let mut bytes = i32::try_from(frame.len()).unwrap().to_be_bytes().to_vec();
        for segment in frame.segments() {
            let Segment::Bytes(part) = segment else {
                panic!("no deferred batches here");
            };
            bytes.extend_from_slice(part);
        }
        bytes
    }

    fn produce() -> (ProduceRequest, Vec<u8>) {
        let request = ProduceRequest {
            acks: -1,
            topic_data: vec![TopicProduceData {
                name: "orders".to_owned(),
                ..TopicProduceData::default()
            }],
            ..ProduceRequest::default()
        };
        let header = RequestHeader {
            request_api_key: ApiKey::Produce.id(),
            request_api_version: 9,
            correlation_id: 42,
            client_id: Some("producer-1".to_owned()),
            ..RequestHeader::default()
        };
        let frame = prefixed(&encode_request(&header, &request).unwrap());
        (request, frame)
    }

    #[test]
    fn a_request_frame_reads_as_header_and_json_body() {
        let (request, frame) = produce();
        let dump = dump_request_frame(&Bytes::from(frame.clone())).unwrap();
        let Ok(FrameHeader::Request(header)) = &dump.header else {
            panic!("a request header: {dump:?}");
        };
        assert_eq!(
            (header.api, header.header.correlation_id),
            (ApiKey::Produce, 42)
        );
        assert_eq!(dump.body, Some(Ok(request.to_json(9))));
        assert_eq!(
            (
                dump.declared_size,
                dump.present,
                dump.trailing,
                dump.beyond_frame
            ),
            (frame.len() - 4, frame.len() - 4, 0, 0)
        );
        assert!(!dump.is_truncated());
    }

    #[test]
    fn a_truncated_frame_is_read_as_far_as_it_goes() {
        let (_, frame) = produce();
        let cut = &frame[..frame.len() - 3];
        let dump = dump_request_frame(&Bytes::copy_from_slice(cut)).unwrap();
        assert!(dump.is_truncated());
        assert!(dump.header.is_ok());
        let error = dump.body.unwrap().unwrap_err();
        assert!(
            matches!(error.kind, DecodeErrorKind::Truncated { .. }),
            "{error}"
        );
        assert!(error.offset > 4 && error.offset <= cut.len(), "{error}");
        assert_eq!(error.field_path, ["TopicData", "PartitionData"], "{error}");
    }

    #[test]
    fn bytes_after_the_body_and_after_the_frame_are_counted() {
        let (_, frame) = produce();
        let mut longer = frame.clone();
        let size = i32::try_from(frame.len() - 4 + 2).unwrap();
        longer[..4].copy_from_slice(&size.to_be_bytes());
        longer.extend_from_slice(&[0, 0, 9, 9, 9]);
        let dump = dump_request_frame(&Bytes::from(longer)).unwrap();
        assert_eq!((dump.trailing, dump.beyond_frame), (2, 3));
        assert!(matches!(dump.body, Some(Ok(_))));
    }

    #[test]
    fn only_an_input_without_a_size_has_no_dump() {
        assert_eq!(
            dump_request_frame(&Bytes::from_static(&[0, 0, 1])),
            Err(FrameError::NoSizePrefix { length: 3 })
        );
        assert_eq!(
            dump_request_frame(&Bytes::from_static(&[0xff, 0xff, 0xff, 0xfe])),
            Err(FrameError::NegativeSize { size: -2 })
        );
        let empty = dump_request_frame(&Bytes::from_static(&[0, 0, 0, 0])).unwrap();
        assert!(matches!(
            empty.header,
            Err(HeaderError::Malformed { api_key: -1, .. })
        ));
        let unknown =
            dump_request_frame(&Bytes::from_static(&[0, 0, 0, 4, 0x7f, 0x00, 0, 0])).unwrap();
        assert_eq!(
            (unknown.header, unknown.body),
            (Err(HeaderError::UnknownApiKey { api_key: 0x7f00 }), None)
        );
    }

    #[test]
    fn a_response_frame_reads_with_the_api_and_version_supplied() {
        let response = FetchResponse {
            responses: vec![FetchableTopicResponse::default()],
            ..FetchResponse::default()
        };
        let frame = prefixed(&encode_response(ApiKey::Fetch, 12, 7, &response).unwrap());
        let dump = dump_response_frame(&Bytes::from(frame), ApiKey::Fetch, 12).unwrap();
        assert_eq!(
            dump.header,
            Ok(FrameHeader::Response {
                api: ApiKey::Fetch,
                version: 12,
                correlation_id: 7
            })
        );
        assert_eq!(dump.body, Some(Ok(response.to_json(12))));
    }

    // A server that doesn't serve the ApiVersions version asked answers at version 0
    // (KIP-511), and the dump reads it so, as Kafka's client does.
    #[test]
    fn an_api_versions_response_falls_back_to_version_0() {
        let refusal = ApiVersionsResponse {
            error_code: 35,
            api_keys: vec![ApiVersion {
                api_key: ApiKey::ApiVersions.id(),
                max_version: 2,
                ..ApiVersion::default()
            }],
            ..ApiVersionsResponse::default()
        };
        let mut writer = Writer::new();
        writer.write_i32(9);
        refusal.encode(&mut writer, 0).unwrap();
        let mut frame = Vec::new();
        frame.extend_from_slice(&i32::try_from(writer.len()).unwrap().to_be_bytes());
        frame.extend_from_slice(&writer.into_bytes());
        let dump: FrameDump =
            dump_response_frame(&Bytes::from(frame), ApiKey::ApiVersions, 4).unwrap();
        assert_eq!(dump.body, Some(Ok(refusal.to_json(0))));
    }
}
