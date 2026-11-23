// Portions of this file are ported from Apache Kafka 4.4.0 and modified by Alluvium;
// each is marked `// Ported from:`. Apache Kafka is Copyright The Apache Software
// Foundation and licensed under the Apache License 2.0 (LICENSES/Apache-2.0.txt).
// See NOTICE.

//! Request and response headers: choosing their version from the API and its version,
//! and reading and writing them.
//!
//! A request header is read before anything is known about the request, so
//! [`decode_request_header`] peeks at its API key and version, chooses the header
//! version from them ([`ApiKey::request_header_version`]), and reads the header. A
//! response header carries only the correlation id; its version comes from the request
//! it answers ([`ApiKey::response_header_version`]).
//!
//! [`encode_response`] and [`encode_request`] put a header and a body into one
//! [`EncodedFrame`] that shares the body's record batches; its length is exact, so the
//! network layer writes it as the frame's length prefix.

use crate::api_key::ApiKey;
use crate::error::{DecodeError, EncodeError};
use crate::generated::request_header::RequestHeader;
use crate::generated::response_header::ResponseHeader;
use crate::message::Message;
use crate::reader::Reader;
use crate::writer::EncodedFrame;
use crate::writer::Writer;

/// Why a header couldn't be read or written.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HeaderError {
    /// An API key Kafka 4.4.0 doesn't define.
    #[error("unknown API key {api_key}")]
    UnknownApiKey {
        /// The key as read.
        api_key: i16,
    },
    /// An API Kafka has removed, whose key has no versions.
    #[error("unsupported API {} (key {}) at version {version}", api.name(), api.id())]
    UnsupportedApi {
        /// The API.
        api: ApiKey,
        /// The version as read.
        version: i16,
    },
    /// The header didn't decode.
    #[error("malformed request header, API key {api_key} as far as read: {error}")]
    Malformed {
        /// The API key, or -1 if the input ended before it: Kafka's "best guess of the
        /// apiKeyId", for the log. Kafka sends no response to a header it can't read; it
        /// closes the connection.
        api_key: i16,
        /// What went wrong.
        error: DecodeError,
    },
    /// The header didn't encode.
    #[error(transparent)]
    Encode(#[from] EncodeError),
}

/// A request header as read, with what was chosen to read it.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedRequestHeader {
    /// The API the request is for.
    pub api: ApiKey,
    /// The header version read: 1 or 2.
    pub header_version: i16,
    /// The header. A null client id reads as empty, as in Kafka.
    pub header: RequestHeader,
    /// The header's size in bytes, after which the request body starts.
    pub size: usize,
}

/// Reads a request header: its API key and version, then the header at the version
/// they select.
///
/// # Errors
///
/// [`HeaderError::UnknownApiKey`] for an API key Kafka doesn't define,
/// [`HeaderError::UnsupportedApi`] for a removed API, and [`HeaderError::Malformed`],
/// with the API key if it was read, for a header that doesn't decode.
// Parity: clients/src/main/java/org/apache/kafka/common/requests/RequestHeader.java@4.4.0 RequestHeader.parse
pub fn decode_request_header(reader: &mut Reader) -> Result<DecodedRequestHeader, HeaderError> {
    let start = reader.offset();
    let mut peek = Reader::new(reader.buf.slice(..reader.buf.len().min(4)));
    let malformed = |api_key| move |error| HeaderError::Malformed { api_key, error };
    // The peek reads from offset 0; its errors name the offset in the input.
    let peeked = |api_key| {
        move |mut error: DecodeError| {
            error.offset = error.offset.saturating_add(start);
            HeaderError::Malformed { api_key, error }
        }
    };
    let api_key = peek.read_i16().map_err(peeked(-1))?;
    let version = peek.read_i16().map_err(peeked(api_key))?;
    let api = ApiKey::from_id(api_key).ok_or(HeaderError::UnknownApiKey { api_key })?;
    let header_version = api
        .request_header_version(version)
        .ok_or(HeaderError::UnsupportedApi { api, version })?;
    let mut header = RequestHeader::decode(reader, header_version).map_err(malformed(api_key))?;
    // Kafka's schema makes the client id nullable, but Kafka reads a null as empty.
    header.client_id.get_or_insert_with(String::new);
    Ok(DecodedRequestHeader {
        api,
        header_version,
        header,
        size: reader.offset().saturating_sub(start),
    })
}

/// Writes a request header, at the version its API key and version select, as a client
/// of another node does.
///
/// # Errors
///
/// [`HeaderError::UnknownApiKey`] and [`HeaderError::UnsupportedApi`] as in
/// [`decode_request_header`], and [`HeaderError::Encode`] if the header doesn't encode.
// Parity: clients/src/main/java/org/apache/kafka/common/requests/RequestHeader.java@4.4.0 RequestHeader.write
pub fn encode_request_header(
    header: &RequestHeader,
    writer: &mut Writer,
) -> Result<(), HeaderError> {
    let (api_key, version) = (header.request_api_key, header.request_api_version);
    let api = ApiKey::from_id(api_key).ok_or(HeaderError::UnknownApiKey { api_key })?;
    let header_version = api
        .request_header_version(version)
        .ok_or(HeaderError::UnsupportedApi { api, version })?;
    Ok(header.encode(writer, header_version)?)
}

/// Writes the header of a response to a request for `api` at `version`, carrying the
/// request's `correlation_id`.
///
/// # Errors
///
/// [`HeaderError::UnsupportedApi`] for a removed API, and [`HeaderError::Encode`] if the
/// header doesn't encode.
// Parity: clients/src/main/java/org/apache/kafka/common/requests/RequestHeader.java@4.4.0 RequestHeader.toResponseHeader
pub fn encode_response_header(
    api: ApiKey,
    version: i16,
    correlation_id: i32,
    writer: &mut Writer,
) -> Result<(), HeaderError> {
    let header_version = api
        .response_header_version(version)
        .ok_or(HeaderError::UnsupportedApi { api, version })?;
    let header = ResponseHeader {
        correlation_id,
        ..ResponseHeader::default()
    };
    Ok(header.encode(writer, header_version)?)
}

/// A response: its header, for a request for `api` at `version` with `correlation_id`,
/// then `body`, as one frame that shares the body's record batches. The frame's length is
/// exact, so it is the length prefix the network layer writes before it.
///
/// # Errors
///
/// As [`encode_response_header`], and [`HeaderError::Encode`] if the body doesn't encode.
// Parity: clients/src/main/java/org/apache/kafka/common/requests/AbstractResponse.java@4.4.0 AbstractResponse.toSend
pub fn encode_response<M: Message>(
    api: ApiKey,
    version: i16,
    correlation_id: i32,
    body: &M,
) -> Result<EncodedFrame, HeaderError> {
    let mut writer = Writer::sharing();
    encode_response_header(api, version, correlation_id, &mut writer)?;
    body.encode(&mut writer, version)?;
    Ok(writer.into_frame())
}

/// A request to another node: `header`, then `body` at the header's API version, as one
/// frame that shares the body's record batches. The frame's length is exact.
///
/// # Errors
///
/// As [`encode_request_header`], and [`HeaderError::Encode`] if the body doesn't encode.
// Parity: clients/src/main/java/org/apache/kafka/common/requests/AbstractRequest.java@4.4.0 AbstractRequest.toSend
pub fn encode_request<M: Message>(
    header: &RequestHeader,
    body: &M,
) -> Result<EncodedFrame, HeaderError> {
    let mut writer = Writer::sharing();
    encode_request_header(header, &mut writer)?;
    body.encode(&mut writer, header.request_api_version)?;
    Ok(writer.into_frame())
}

/// Reads the header of a response to a request for `api` at `version`, and returns its
/// correlation id, for the caller to match with the request.
///
/// # Errors
///
/// [`HeaderError::UnsupportedApi`] for a removed API, and [`HeaderError::Malformed`] for
/// a header that doesn't decode.
// Parity: clients/src/main/java/org/apache/kafka/common/requests/ResponseHeader.java@4.4.0 ResponseHeader.parse
pub fn decode_response_header(
    api: ApiKey,
    version: i16,
    reader: &mut Reader,
) -> Result<i32, HeaderError> {
    let header_version = api
        .response_header_version(version)
        .ok_or(HeaderError::UnsupportedApi { api, version })?;
    ResponseHeader::decode(reader, header_version)
        .map(|header| header.correlation_id)
        .map_err(|error| HeaderError::Malformed {
            api_key: api.id(),
            error,
        })
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::{
        HeaderError, decode_request_header, decode_response_header, encode_request_header,
        encode_response_header,
    };
    use crate::generated::request_header::RequestHeader;
    use crate::{ApiKey, DecodeErrorKind, Limit, Message, Reader, Writer};

    fn header(
        api: ApiKey,
        version: i16,
        client_id: Option<&str>,
        correlation_id: i32,
    ) -> RequestHeader {
        RequestHeader {
            request_api_key: api.id(),
            request_api_version: version,
            correlation_id,
            client_id: client_id.map(str::to_owned),
            ..RequestHeader::default()
        }
    }

    fn encoded(header: &RequestHeader) -> Bytes {
        let mut writer = Writer::new();
        encode_request_header(header, &mut writer).unwrap();
        writer.into_bytes()
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/requests/RequestHeaderTest.java@4.4.0 (testRequestHeaderV1)
    #[test]
    fn a_v1_header_round_trips_in_10_bytes() {
        let header = header(ApiKey::FindCoordinator, 1, Some(""), 10);
        let bytes = encoded(&header);
        assert_eq!(bytes.len(), 10);
        let decoded = decode_request_header(&mut Reader::new(bytes)).unwrap();
        assert_eq!((decoded.header_version, decoded.size), (1, 10));
        assert_eq!(decoded.header, header);
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/requests/RequestHeaderTest.java@4.4.0 (testRequestHeaderV2)
    #[test]
    fn a_v2_header_round_trips_in_11_bytes() {
        let header = header(ApiKey::CreateDelegationToken, 2, Some(""), 10);
        let bytes = encoded(&header);
        assert_eq!(bytes.len(), 11);
        let decoded = decode_request_header(&mut Reader::new(bytes)).unwrap();
        assert_eq!((decoded.header_version, decoded.size), (2, 11));
        assert_eq!(decoded.header, header);
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/requests/RequestHeaderTest.java@4.4.0 (parseHeaderFromBufferWithNonZeroPosition)
    #[test]
    fn a_header_after_other_bytes_decodes() {
        let header = header(ApiKey::FindCoordinator, 1, Some(""), 10);
        let mut input = vec![0xaa; 10];
        input.extend_from_slice(&encoded(&header));
        let mut reader = Reader::new(Bytes::from(input));
        reader.take(10).unwrap();
        assert_eq!(decode_request_header(&mut reader).unwrap().header, header);
        assert_eq!(reader.remaining(), 0);
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/requests/RequestHeaderTest.java@4.4.0 (parseHeaderWithNullClientId)
    #[test]
    fn a_null_client_id_reads_as_empty() {
        let mut writer = Writer::new();
        header(ApiKey::FindCoordinator, 10, None, 123)
            .encode(&mut writer, 2)
            .unwrap();
        let decoded = decode_request_header(&mut Reader::new(writer.into_bytes())).unwrap();
        assert_eq!(
            decoded.header,
            header(ApiKey::FindCoordinator, 10, Some(""), 123)
        );
        assert_eq!(decoded.api, ApiKey::FindCoordinator);
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/requests/RequestHeaderTest.java@4.4.0 (testHugeDeclaredTaggedFieldCountIsRejected)
    #[test]
    fn a_huge_tagged_field_count_is_rejected() {
        let bytes = encoded(&header(ApiKey::FindCoordinator, 10, Some("client"), 123));
        let (prefix, tags) = bytes.split_at(bytes.len() - 1);
        assert_eq!(tags, [0], "an empty tagged-field section to replace");
        let declared = 20_000_000;
        let mut writer = Writer::new();
        writer.write_unsigned_varint(declared);
        let mut input = prefix.to_vec();
        input.extend_from_slice(&writer.into_bytes());
        // As many padding bytes as tags declared, so only the count limit can refuse it.
        input.resize(input.len() + declared as usize, 0);
        let error = decode_request_header(&mut Reader::new(Bytes::from(input))).unwrap_err();
        let HeaderError::Malformed { api_key, error } = error else {
            panic!("{error:?}");
        };
        assert_eq!(api_key, ApiKey::FindCoordinator.id());
        assert!(matches!(
            error.kind,
            DecodeErrorKind::LimitExceeded {
                limit: Limit::TaggedFieldCount,
                ..
            }
        ));
    }

    // Ported from: clients/src/test/java/org/apache/kafka/common/requests/RequestHeaderTest.java@4.4.0 (verifySizeMethodsReturnSameValue)
    #[test]
    fn the_size_is_the_headers_encoded_size() {
        let header = header(ApiKey::FindCoordinator, 10, Some("hakuna-matata"), 123);
        let decoded = decode_request_header(&mut Reader::new(encoded(&header))).unwrap();
        assert_eq!(decoded.size, header.encoded_size(2).unwrap().total);
    }

    #[test]
    fn an_api_versions_request_above_every_known_version_has_a_v2_header() {
        let header = header(ApiKey::ApiVersions, 127, Some("newer-client"), 1);
        let decoded = decode_request_header(&mut Reader::new(encoded(&header))).unwrap();
        assert_eq!(
            (decoded.api, decoded.header_version),
            (ApiKey::ApiVersions, 2)
        );
    }

    #[test]
    fn unknown_and_removed_apis_and_truncated_headers_are_errors() {
        let decode =
            |bytes: &[u8]| decode_request_header(&mut Reader::new(Bytes::copy_from_slice(bytes)));
        assert_eq!(
            decode(&[0x7f, 0x00, 0, 0]),
            Err(HeaderError::UnknownApiKey { api_key: 0x7f00 })
        );
        assert_eq!(
            decode(&[0, 4, 0, 1]),
            Err(HeaderError::UnsupportedApi {
                api: ApiKey::LeaderAndIsr,
                version: 1
            })
        );
        assert!(matches!(
            decode(&[]),
            Err(HeaderError::Malformed { api_key: -1, .. })
        ));
        assert!(matches!(
            decode(&[0, 3]),
            Err(HeaderError::Malformed { api_key: 3, .. })
        ));
        let mut reader = Reader::new(Bytes::from_static(&[9, 9, 9, 0, 3, 0]));
        reader.take(3).unwrap();
        let Err(HeaderError::Malformed { error, .. }) = decode_request_header(&mut reader) else {
            panic!("a truncated version");
        };
        assert_eq!(error.offset, 5, "offsets count from the start of the input");
        assert!(matches!(
            decode(&[0, 3, 0, 12, 0]),
            Err(HeaderError::Malformed { api_key: 3, .. })
        ));
    }

    #[test]
    fn a_null_client_id_and_an_empty_one_differ_on_the_wire() {
        let null = encoded(&header(ApiKey::Metadata, 1, None, 7));
        let empty = encoded(&header(ApiKey::Metadata, 1, Some(""), 7));
        assert_eq!(null.get(8..), Some(&[0xff, 0xff][..]));
        assert_eq!(empty.get(8..), Some(&[0, 0][..]));
    }

    #[test]
    fn response_headers_carry_the_correlation_id_at_the_requests_header_version() {
        for (api, version, length) in [
            (ApiKey::Fetch, 11, 4),
            (ApiKey::Fetch, 12, 5),
            // KIP-511: always v0, even where ApiVersions is flexible.
            (ApiKey::ApiVersions, 3, 4),
        ] {
            let mut writer = Writer::new();
            encode_response_header(api, version, 42, &mut writer).unwrap();
            let bytes = writer.into_bytes();
            assert_eq!(bytes.len(), length, "{api:?} v{version}");
            let mut reader = Reader::new(bytes);
            assert_eq!(decode_response_header(api, version, &mut reader), Ok(42));
            assert_eq!(reader.remaining(), 0);
        }
        let truncated = decode_response_header(
            ApiKey::Fetch,
            12,
            &mut Reader::new(Bytes::from_static(&[0, 0])),
        );
        assert!(matches!(
            truncated,
            Err(HeaderError::Malformed { api_key: 1, .. })
        ));
    }
}
