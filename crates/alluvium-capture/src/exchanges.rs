//! A connection's requests paired with their responses by correlation id (PROTO-0145),
//! so that each response decodes at its request's API and version without the caller
//! naming them.
//!
//! Kafka's clients give each request a new correlation id, so an id names one request on
//! a connection; were one to repeat, its responses would pair with its requests in order.
//! A request whose header doesn't decode can't be paired, nor can a response whose
//! correlation id no request has. A request without a response is one the capture ended
//! before, or a `Produce` with `acks=0`, which Kafka doesn't answer.

use std::collections::{BTreeMap, VecDeque};

use alluvium_protocol::{
    FrameDump, FrameError, FrameHeader, dump_request_frame, dump_response_frame,
};

use crate::streams::{Connection, StreamFrame};

/// A request, and its response if the capture has one.
#[derive(Debug, Clone, PartialEq)]
pub struct Exchange {
    /// The request frame.
    pub request: StreamFrame,
    /// The request, read.
    pub request_dump: Result<FrameDump, FrameError>,
    /// The response frame and the response, read at the request's API and version.
    pub response: Option<(StreamFrame, Result<FrameDump, FrameError>)>,
}

/// A connection's exchanges, in request order, and the responses no request has.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Exchanges {
    /// Each request, with its response.
    pub exchanges: Vec<Exchange>,
    /// Responses whose correlation id matches no request read, with the id if the frame
    /// has one.
    pub unmatched: Vec<(StreamFrame, Option<i32>)>,
}

/// The exchanges of `connection`.
pub fn exchanges(connection: &Connection) -> Exchanges {
    let mut exchanges: Vec<Exchange> = connection
        .requests
        .iter()
        .map(|request| Exchange {
            request: request.clone(),
            request_dump: dump_request_frame(&request.bytes),
            response: None,
        })
        .collect();
    // The requests waiting for a response, by correlation id, in request order.
    let mut waiting: BTreeMap<i32, VecDeque<usize>> = BTreeMap::new();
    for (index, exchange) in exchanges.iter().enumerate() {
        if let Some((_, _, correlation_id)) = request_header(exchange) {
            waiting.entry(correlation_id).or_default().push_back(index);
        }
    }
    let mut unmatched = Vec::new();
    for response in &connection.responses {
        // Every response header starts with the correlation id, after the size prefix.
        let correlation_id = response
            .bytes
            .get(4..8)
            .and_then(|bytes| bytes.try_into().ok())
            .map(i32::from_be_bytes);
        let exchange = correlation_id
            .and_then(|id| waiting.get_mut(&id)?.pop_front())
            .and_then(|index| exchanges.get_mut(index));
        match exchange.and_then(|exchange| Some((request_header(exchange)?, exchange))) {
            Some(((api, version, _), exchange)) => {
                let dump = dump_response_frame(&response.bytes, api, version);
                exchange.response = Some((response.clone(), dump));
            }
            None => unmatched.push((response.clone(), correlation_id)),
        }
    }
    Exchanges {
        exchanges,
        unmatched,
    }
}

/// The API, version and correlation id of an exchange's request, if its header decoded.
fn request_header(exchange: &Exchange) -> Option<(alluvium_protocol::ApiKey, i16, i32)> {
    match &exchange.request_dump {
        Ok(FrameDump {
            header: Ok(FrameHeader::Request(decoded)),
            ..
        }) => Some((
            decoded.api,
            decoded.header.request_api_version,
            decoded.header.correlation_id,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use alluvium_protocol::generated::api_versions_request::ApiVersionsRequest;
    use alluvium_protocol::generated::api_versions_response::ApiVersionsResponse;
    use alluvium_protocol::generated::request_header::RequestHeader;
    use alluvium_protocol::{
        ApiKey, EncodedFrame, Message, Segment, encode_request, encode_response,
    };
    use bytes::Bytes;

    use super::exchanges;
    use crate::streams::{Connection, StreamFrame};

    fn wire(frame: &EncodedFrame) -> StreamFrame {
        let mut bytes = u32::try_from(frame.len()).unwrap().to_be_bytes().to_vec();
        for segment in frame.segments() {
            let Segment::Bytes(part) = segment else {
                panic!("no deferred batches here");
            };
            bytes.extend_from_slice(part);
        }
        StreamFrame {
            offset: 0,
            packet: 1,
            bytes: Bytes::from(bytes),
        }
    }

    fn request(correlation_id: i32) -> StreamFrame {
        let header = RequestHeader {
            request_api_key: ApiKey::ApiVersions.id(),
            request_api_version: 3,
            correlation_id,
            ..RequestHeader::default()
        };
        wire(&encode_request(&header, &ApiVersionsRequest::default()).unwrap())
    }

    fn response(correlation_id: i32) -> StreamFrame {
        wire(
            &encode_response(
                ApiKey::ApiVersions,
                3,
                correlation_id,
                &ApiVersionsResponse::default(),
            )
            .unwrap(),
        )
    }

    #[test]
    fn responses_pair_by_correlation_id_and_decode_at_the_requests_version() {
        let connection = Connection {
            client: SocketAddr::from(([10, 0, 0, 1], 40000)),
            server: SocketAddr::from(([10, 0, 0, 2], 9092)),
            roles_from_handshake: true,
            requests: vec![request(1), request(2), request(3)],
            responses: vec![response(2), response(1), response(9)],
            problems: Vec::new(),
        };
        let paired = exchanges(&connection);
        let answered: Vec<Option<String>> = paired
            .exchanges
            .iter()
            .map(|exchange| {
                exchange
                    .response
                    .as_ref()
                    .map(|(_, dump)| dump.as_ref().unwrap().body.clone().unwrap().unwrap())
            })
            .collect();
        let json = ApiVersionsResponse::default().to_json(3);
        assert_eq!(answered, [Some(json.clone()), Some(json), None]);
        assert_eq!(paired.unmatched.len(), 1);
        assert_eq!(paired.unmatched[0].1, Some(9));
    }
}
