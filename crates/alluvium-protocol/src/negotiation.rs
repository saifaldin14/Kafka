// Portions of this file are ported from Apache Kafka 4.4.0 and modified by Alluvium;
// each is marked `// Ported from:`. Apache Kafka is Copyright The Apache Software
// Foundation and licensed under the Apache License 2.0 (LICENSES/Apache-2.0.txt).
// See NOTICE.

//! Version negotiation with a peer: what versions of each API it serves, from its
//! `ApiVersions` response ([`NodeApiVersions`]), the highest version both sides can use
//! ([`NodeApiVersions::latest_usable_version`]), and the `ApiVersions` exchange that
//! learns them ([`ApiVersionsHandshake`]), for Alluvium's clients of other nodes.
//!
//! Everything here is a pure function of its inputs: the network layer sends the
//! requests, decodes the response header, and logs the [`Negotiation`] (NET-F10).

use std::collections::BTreeMap;
use std::fmt;

use bytes::Bytes;

use crate::generated::api_versions_response::ApiVersionsResponse;
use crate::{ApiKey, DecodeError, ErrorCode, Listener, Message, Reader, Versions};

/// The versions of one API a peer reported: Kafka's `ApiVersion`. `min_version` can be
/// above `max_version`, as Kafka reports an API it has no versions of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerApiVersion {
    /// The API's key.
    pub api_key: i16,
    /// The lowest version the peer serves.
    pub min_version: i16,
    /// The highest version the peer serves.
    pub max_version: i16,
}

/// The API versions a peer serves, from its `ApiVersions` response (PROTO-0148).
///
/// APIs Kafka 4.4.0 doesn't define are kept apart, for diagnostics: a newer peer may
/// serve APIs this node doesn't know.
// Parity: clients/src/main/java/org/apache/kafka/clients/NodeApiVersions.java@4.4.0 NodeApiVersions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeApiVersions {
    /// The known APIs' versions, by key; a key listed twice keeps its last entry, as
    /// Kafka's map does.
    known: BTreeMap<i16, PeerApiVersion>,
    /// The entries of APIs Kafka 4.4.0 doesn't define, in the order received.
    unknown: Vec<PeerApiVersion>,
}

impl NodeApiVersions {
    /// The versions `response` lists.
    pub fn new(response: &ApiVersionsResponse) -> NodeApiVersions {
        let mut known = BTreeMap::new();
        let mut unknown = Vec::new();
        for entry in &response.api_keys {
            let entry = PeerApiVersion {
                api_key: entry.api_key,
                min_version: entry.min_version,
                max_version: entry.max_version,
            };
            if ApiKey::from_id(entry.api_key).is_some() {
                known.insert(entry.api_key, entry);
            } else {
                unknown.push(entry);
            }
        }
        NodeApiVersions { known, unknown }
    }

    /// The versions of `api` the peer serves, or `None` if it didn't list it.
    pub fn api_version(&self, api: ApiKey) -> Option<PeerApiVersion> {
        self.known.get(&api.id()).copied()
    }

    /// The APIs the peer listed that Kafka 4.4.0 doesn't define.
    pub fn unknown_apis(&self) -> &[PeerApiVersion] {
        &self.unknown
    }

    /// The highest version of `api` that this node has, the peer serves, and `allowed`
    /// includes (PROTO-0149). Pass `api.versions()` to allow any.
    ///
    /// # Errors
    ///
    /// [`UnsupportedVersion`] if no version is in all three.
    // Parity: clients/src/main/java/org/apache/kafka/clients/NodeApiVersions.java@4.4.0 NodeApiVersions.latestUsableVersion
    pub fn latest_usable_version(
        &self,
        api: ApiKey,
        allowed: Versions,
    ) -> Result<i16, UnsupportedVersion> {
        let local = api.versions().intersect(allowed);
        let peer = self.api_version(api);
        let unsupported = UnsupportedVersion { api, local, peer };
        let peer = peer.ok_or(unsupported)?;
        local
            .intersect(Versions::new(peer.min_version, peer.max_version))
            .highest()
            .ok_or(unsupported)
    }
}

/// The lowest and highest of `versions` as Kafka writes a range it has none of: 0 to -1.
fn bounds(versions: Versions) -> (i16, i16) {
    (
        versions.lowest().unwrap_or(0),
        versions.highest().unwrap_or(-1),
    )
}

/// Each API the peer listed, with its range and the version this node would use, then
/// each broker API it didn't list, as `UNSUPPORTED`, in key order: Kafka's
/// `NodeApiVersions.toString()`, so logs read the same as a Kafka client's.
// Parity: clients/src/main/java/org/apache/kafka/clients/NodeApiVersions.java@4.4.0 NodeApiVersions.toString, NodeApiVersions.apiVersionToText
impl fmt::Display for NodeApiVersions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut texts: BTreeMap<i16, String> = BTreeMap::new();
        for entry in self.known.values().chain(&self.unknown) {
            texts.insert(entry.api_key, api_version_text(*entry));
        }
        for api in ApiKey::served_on(Listener::Broker) {
            texts
                .entry(api.id())
                .or_insert_with(|| format!("{}({}): UNSUPPORTED", api.name(), api.id()));
        }
        let texts: Vec<String> = texts.into_values().collect();
        write!(f, "({})", texts.join(", "))
    }
}

fn api_version_text(entry: PeerApiVersion) -> String {
    let (min, max) = (entry.min_version, entry.max_version);
    let range = if min == max {
        min.to_string()
    } else {
        format!("{min} to {max}")
    };
    let Some(api) = ApiKey::from_id(entry.api_key) else {
        return format!("UNKNOWN({}): {range}", entry.api_key);
    };
    let (oldest, latest) = bounds(api.versions());
    let usable = if latest < min {
        " [unusable: node too new]".to_owned()
    } else if max < oldest {
        " [unusable: node too old]".to_owned()
    } else {
        format!(" [usable: {}]", latest.min(max))
    };
    format!("{}({}): {range}{usable}", api.name(), api.id())
}

/// No version of an API is usable with a peer: what this node allows and the peer
/// serves don't overlap, or the peer doesn't serve the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedVersion {
    /// The API.
    pub api: ApiKey,
    /// The versions this node has and allows.
    pub local: Versions,
    /// The versions the peer serves, or `None` if it didn't list the API.
    pub peer: Option<PeerApiVersion>,
}

impl fmt::Display for UnsupportedVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (api, id) = (self.api.name(), self.api.id());
        let Some(peer) = self.peer else {
            return write!(f, "the node does not support {api} (key {id})");
        };
        let (lowest, highest) = bounds(self.local);
        write!(
            f,
            "the node does not support {api} (key {id}) with version in range [{lowest},{highest}]; \
             the supported range is [{},{}]",
            peer.min_version, peer.max_version
        )
    }
}

impl std::error::Error for UnsupportedVersion {}

/// The `ApiVersions` exchange with a peer, as a Kafka client makes it (PROTO-0150): send
/// this node's highest version, and if the peer doesn't serve it, retry at the highest
/// it does (KIP-511).
///
/// [`ApiVersionsHandshake::version`] is the version to send the request at. Each
/// response body, after its header (always v0 for `ApiVersions`), goes to
/// [`ApiVersionsHandshake::on_response`], which says what to do next.
// Parity: clients/src/main/java/org/apache/kafka/clients/NetworkClient.java@4.4.0 NetworkClient.handleApiVersionsResponse
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiVersionsHandshake {
    node_id: i32,
    version: i16,
    /// The requests sent so far, this one included.
    sent: i16,
}

/// What an `ApiVersions` response leads to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeStep {
    /// Send the request again, at the retried handshake's version.
    Retry(ApiVersionsHandshake),
    /// The peer's versions are known.
    Negotiated(Negotiation),
    /// The exchange failed; Kafka disconnects.
    Failed(HandshakeError),
}

/// Why an `ApiVersions` exchange failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HandshakeError {
    /// The peer answered with an error that isn't `UNSUPPORTED_VERSION`, or with
    /// `UNSUPPORTED_VERSION` to version 0, below which there is nothing to retry.
    /// `REBOOTSTRAP_REQUIRED` is one too: a Kafka client configured to rebootstraps on
    /// it, which is its caller's decision here.
    #[error("node {node_id} answered ApiVersions v{version} with error {}", error_name(*error_code))]
    ErrorResponse {
        /// The peer.
        node_id: i32,
        /// The version sent.
        version: i16,
        /// The error code, as received.
        error_code: i16,
    },
    /// The peer answered `UNSUPPORTED_VERSION` naming a version that can't be retried:
    /// one this node doesn't have, such as a negative one, which Kafka's client fails to
    /// build a request at; or any, once this node has sent as many requests as it has
    /// versions, where Kafka's would go on retrying. So an exchange always ends.
    #[error(
        "node {node_id} answered ApiVersions v{version} with UNSUPPORTED_VERSION naming \
         v{peer_max_version}, which can't be retried"
    )]
    UnusableRetry {
        /// The peer.
        node_id: i32,
        /// The version sent.
        version: i16,
        /// The highest version the peer named.
        peer_max_version: i16,
    },
    /// The response didn't decode at the version sent, nor at version 0.
    #[error("node {node_id} sent an ApiVersions v{version} response that doesn't decode: {error}")]
    Malformed {
        /// The peer.
        node_id: i32,
        /// The version sent.
        version: i16,
        /// What went wrong decoding at version 0.
        error: DecodeError,
    },
}

/// A completed `ApiVersions` exchange: the peer, the version it was made at, and what the
/// peer serves (PROTO-0151). Its `Display` is the network layer's negotiation event:
/// `node 1 via ApiVersions v4: (Produce(0): 3 to 13 [usable: 13], ...)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Negotiation {
    /// The peer.
    pub node_id: i32,
    /// The `ApiVersions` version the exchange ended at.
    pub version: i16,
    /// What the peer serves.
    pub versions: NodeApiVersions,
}

impl fmt::Display for Negotiation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "node {} via ApiVersions v{}: {}",
            self.node_id, self.version, self.versions
        )
    }
}

impl ApiVersionsHandshake {
    /// An exchange with node `node_id`, starting at this node's highest `ApiVersions`
    /// version.
    pub fn new(node_id: i32) -> ApiVersionsHandshake {
        ApiVersionsHandshake {
            node_id,
            version: bounds(ApiKey::ApiVersions.versions()).1,
            sent: 1,
        }
    }

    /// The version to send the `ApiVersions` request at.
    pub fn version(&self) -> i16 {
        self.version
    }

    /// What the response `body`, which follows its header, leads to.
    ///
    /// A peer that doesn't serve the version sent answers at version 0, so a body that
    /// doesn't decode at the version sent is decoded at version 0, as Kafka's client
    /// does.
    pub fn on_response(self, body: Bytes) -> HandshakeStep {
        let (node_id, version) = (self.node_id, self.version);
        let response = match decode(body, version) {
            Ok(response) => response,
            Err(error) => {
                return HandshakeStep::Failed(HandshakeError::Malformed {
                    node_id,
                    version,
                    error,
                });
            }
        };
        let unsupported = ErrorCode::UnsupportedVersion.code();
        if response.error_code == ErrorCode::None.code() {
            return HandshakeStep::Negotiated(Negotiation {
                node_id,
                version,
                versions: NodeApiVersions::new(&response),
            });
        }
        if version == 0 || response.error_code != unsupported {
            return HandshakeStep::Failed(HandshakeError::ErrorResponse {
                node_id,
                version,
                error_code: response.error_code,
            });
        }
        // Since Kafka 2.4 the error lists the peer's ApiVersions range; without it,
        // version 0.
        let peer_max_version = response
            .api_keys
            .iter()
            .find(|entry| entry.api_key == ApiKey::ApiVersions.id())
            .map_or(0, |entry| entry.max_version);
        // Kafka retries at that version even if it is the one sent or a higher one.
        let versions = ApiKey::ApiVersions.versions();
        let tries = bounds(versions).1.saturating_add(1);
        if !versions.contains(peer_max_version) || self.sent >= tries {
            return HandshakeStep::Failed(HandshakeError::UnusableRetry {
                node_id,
                version,
                peer_max_version,
            });
        }
        HandshakeStep::Retry(ApiVersionsHandshake {
            node_id,
            version: peer_max_version,
            sent: self.sent.saturating_add(1),
        })
    }
}

/// `NAME (code)`, or the bare code if Kafka 4.4.0 doesn't define it.
fn error_name(code: i16) -> String {
    ErrorCode::from_code(code).map_or_else(|_| code.to_string(), |error| error.to_string())
}

/// An `ApiVersions` response body at `version`, or at version 0 if it doesn't decode at
/// `version`.
// Parity: clients/src/main/java/org/apache/kafka/common/requests/ApiVersionsResponse.java@4.4.0 ApiVersionsResponse.parse
fn decode(body: Bytes, version: i16) -> Result<ApiVersionsResponse, DecodeError> {
    match ApiVersionsResponse::decode(&mut Reader::new(body.clone()), version) {
        Ok(response) => Ok(response),
        Err(_) if version != 0 => ApiVersionsResponse::decode(&mut Reader::new(body), 0),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use proptest::prelude::*;

    use super::{
        ApiVersionsHandshake, HandshakeError, HandshakeStep, NodeApiVersions, PeerApiVersion,
        UnsupportedVersion, bounds,
    };
    use crate::generated::api_versions_response::{ApiVersion, ApiVersionsResponse};
    use crate::{ApiKey, ErrorCode, Listener, Message, Versions, Writer};

    fn response(error: ErrorCode, entries: &[(i16, i16, i16)]) -> ApiVersionsResponse {
        ApiVersionsResponse {
            error_code: error.code(),
            api_keys: entries
                .iter()
                .map(|&(api_key, min_version, max_version)| ApiVersion {
                    api_key,
                    min_version,
                    max_version,
                    ..ApiVersion::default()
                })
                .collect(),
            ..ApiVersionsResponse::default()
        }
    }

    fn peer(entries: &[(i16, i16, i16)]) -> NodeApiVersions {
        NodeApiVersions::new(&response(ErrorCode::None, entries))
    }

    /// Every API at its local versions, as Kafka's `ApiVersionsResponse.toApiVersion`.
    fn local(api: ApiKey) -> (i16, i16, i16) {
        let (oldest, latest) = bounds(api.versions());
        (api.id(), oldest, latest)
    }

    // Ported from: clients/src/test/java/org/apache/kafka/clients/NodeApiVersionsTest.java@4.4.0 (testUnsupportedVersionsToString)
    #[test]
    fn a_peer_listing_nothing_supports_no_broker_api() {
        let expected: Vec<String> = ApiKey::served_on(Listener::Broker)
            .map(|api| format!("{}({}): UNSUPPORTED", api.name(), api.id()))
            .collect();
        assert_eq!(peer(&[]).to_string(), format!("({})", expected.join(", ")));
    }

    // Ported from: clients/src/test/java/org/apache/kafka/clients/NodeApiVersionsTest.java@4.4.0 (testUnknownApiVersionsToString)
    #[test]
    fn an_unknown_api_is_kept_and_printed() {
        let versions = peer(&[(337, 0, 1)]);
        assert!(versions.to_string().ends_with("UNKNOWN(337): 0 to 1)"));
        assert_eq!(
            versions.unknown_apis(),
            [PeerApiVersion {
                api_key: 337,
                min_version: 0,
                max_version: 1
            }]
        );
    }

    // Ported from: clients/src/test/java/org/apache/kafka/clients/NodeApiVersionsTest.java@4.4.0 (testVersionsToString)
    #[test]
    fn each_api_prints_its_range_and_usable_version() {
        let entries: Vec<_> = ApiKey::ALL
            .into_iter()
            .map(|api| match api {
                ApiKey::DeleteTopics => (api.id(), 10000, 10001),
                api => local(api),
            })
            .collect();
        let expected: Vec<String> = ApiKey::ALL
            .into_iter()
            .map(|api| {
                let (_, oldest, latest) = local(api);
                let range = if oldest == latest {
                    oldest.to_string()
                } else {
                    format!("{oldest} to {latest}")
                };
                match api {
                    ApiKey::DeleteTopics => {
                        "DeleteTopics(20): 10000 to 10001 [unusable: node too new]".to_owned()
                    }
                    api if api.versions().highest().is_none() => {
                        format!(
                            "{}({}): 0 to -1 [unusable: node too new]",
                            api.name(),
                            api.id()
                        )
                    }
                    api => format!("{}({}): {range} [usable: {latest}]", api.name(), api.id()),
                }
            })
            .collect();
        assert_eq!(
            peer(&entries).to_string(),
            format!("({})", expected.join(", "))
        );
    }

    // Ported from: clients/src/test/java/org/apache/kafka/clients/NodeApiVersionsTest.java@4.4.0 (testLatestUsableVersion)
    #[test]
    fn the_latest_usable_version_is_the_highest_in_every_range() {
        let versions = peer(&[(ApiKey::Produce.id(), 8, 10)]);
        let usable = |lowest, highest| {
            versions.latest_usable_version(ApiKey::Produce, Versions::new(lowest, highest))
        };
        assert_eq!(
            versions.latest_usable_version(ApiKey::Produce, ApiKey::Produce.versions()),
            Ok(10)
        );
        for ((lowest, highest), expected) in [
            ((7, 8), 8),
            ((8, 8), 8),
            ((8, 9), 9),
            ((8, 10), 10),
            ((9, 9), 9),
            ((9, 10), 10),
            ((10, 10), 10),
            ((10, 11), 10),
        ] {
            assert_eq!(
                usable(lowest, highest),
                Ok(expected),
                "{lowest}..={highest}"
            );
        }
    }

    // Ported from: clients/src/test/java/org/apache/kafka/clients/NodeApiVersionsTest.java@4.4.0 (testLatestUsableVersionOutOfRangeLow, testLatestUsableVersionOutOfRangeHigh, testUsableVersionCalculationNoKnownVersions, testLatestUsableVersionOutOfRange)
    #[test]
    fn no_common_version_is_unsupported() {
        let produce = |min, max| PeerApiVersion {
            api_key: ApiKey::Produce.id(),
            min_version: min,
            max_version: max,
        };
        let cases = [
            ((1, 2), Versions::new(3, 4), Versions::new(3, 4)),
            ((2, 3), Versions::new(0, 1), Versions::NONE),
            (
                (300, 300),
                ApiKey::Produce.versions(),
                ApiKey::Produce.versions(),
            ),
        ];
        for ((min, max), allowed, local) in cases {
            let error = peer(&[(ApiKey::Produce.id(), min, max)])
                .latest_usable_version(ApiKey::Produce, allowed)
                .unwrap_err();
            assert_eq!(
                error,
                UnsupportedVersion {
                    api: ApiKey::Produce,
                    local,
                    peer: Some(produce(min, max))
                }
            );
        }
        let error = peer(&[])
            .latest_usable_version(ApiKey::Fetch, ApiKey::Fetch.versions())
            .unwrap_err();
        assert_eq!(error.to_string(), "the node does not support Fetch (key 1)");
        let error = peer(&[(ApiKey::Produce.id(), 1, 2)])
            .latest_usable_version(ApiKey::Produce, Versions::new(3, 4))
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "the node does not support Produce (key 0) with version in range [3,4]; the \
             supported range is [1,2]"
        );
    }

    proptest! {
        // PROTO-0149: the version chosen is in every range, and the highest such; when
        // the ranges don't meet, the error is typed.
        #[test]
        fn the_chosen_version_is_the_highest_in_local_peer_and_allowed(
            api in proptest::sample::select(ApiKey::ALL.to_vec()),
            (peer_min, peer_max) in (-2_i16..40, -2_i16..40),
            (allowed_min, allowed_max) in (-2_i16..40, -2_i16..40),
        ) {
            let allowed = Versions::new(allowed_min, allowed_max);
            let versions = peer(&[(api.id(), peer_min, peer_max)]);
            let common = api.versions().intersect(allowed).intersect(Versions::new(peer_min, peer_max));
            match versions.latest_usable_version(api, allowed) {
                Ok(version) => {
                    prop_assert!(api.versions().contains(version));
                    prop_assert!(allowed.contains(version));
                    prop_assert!(peer_min <= version && version <= peer_max);
                    prop_assert_eq!(Some(version), common.highest());
                }
                Err(error) => {
                    prop_assert_eq!(common.highest(), None);
                    prop_assert_eq!(error.api, api);
                }
            }
        }
    }

    fn body(response: &ApiVersionsResponse, version: i16) -> Bytes {
        let mut writer = Writer::new();
        response.encode(&mut writer, version).unwrap();
        writer.into_bytes()
    }

    /// A peer serving `ApiVersions` up to `peer_max`, answering a request at `version`
    /// as Kafka does: with its list at that version, or with `UNSUPPORTED_VERSION` and
    /// its `ApiVersions` range at version 0.
    fn answer(peer_max: i16, version: i16) -> Bytes {
        let api_versions = (ApiKey::ApiVersions.id(), 0, peer_max);
        if version > peer_max {
            body(&response(ErrorCode::UnsupportedVersion, &[api_versions]), 0)
        } else {
            body(
                &response(ErrorCode::None, &[api_versions, (0, 3, 9)]),
                version,
            )
        }
    }

    // PROTO-0150: a peer serving ApiVersions v0-3 is retried at v3 (KIP-511).
    #[test]
    fn an_older_peer_is_retried_at_its_highest_version() {
        let handshake = ApiVersionsHandshake::new(7);
        let first = handshake.version();
        assert_eq!(Some(first), ApiKey::ApiVersions.versions().highest());
        assert!(first > 3, "this node's highest is above the peer's");
        let HandshakeStep::Retry(retry) = handshake.on_response(answer(3, first)) else {
            panic!("a retry");
        };
        assert_eq!(retry.version(), 3);
        let HandshakeStep::Negotiated(negotiation) = retry.on_response(answer(3, 3)) else {
            panic!("negotiated");
        };
        assert_eq!((negotiation.node_id, negotiation.version), (7, 3));
        assert_eq!(
            negotiation
                .versions
                .latest_usable_version(ApiKey::Produce, ApiKey::Produce.versions()),
            Ok(9)
        );
    }

    // PROTO-0150: a peer whose error lists no range is retried at version 0, and an
    // error at version 0 ends the exchange.
    #[test]
    fn a_peer_listing_no_range_is_retried_at_version_0() {
        let handshake = ApiVersionsHandshake::new(1);
        let no_range = body(&response(ErrorCode::UnsupportedVersion, &[]), 0);
        let HandshakeStep::Retry(retry) = handshake.on_response(no_range.clone()) else {
            panic!("a retry");
        };
        assert_eq!(retry.version(), 0);
        assert_eq!(
            retry.on_response(no_range),
            HandshakeStep::Failed(HandshakeError::ErrorResponse {
                node_id: 1,
                version: 0,
                error_code: ErrorCode::UnsupportedVersion.code()
            })
        );
    }

    #[test]
    fn other_errors_and_bad_bodies_end_the_exchange() {
        let handshake = ApiVersionsHandshake::new(2);
        let version = handshake.version();
        let denied = body(
            &response(ErrorCode::ClusterAuthorizationFailed, &[]),
            version,
        );
        let HandshakeStep::Failed(error) = handshake.on_response(denied) else {
            panic!("failed");
        };
        assert_eq!(
            error.to_string(),
            format!(
                "node 2 answered ApiVersions v{version} with error CLUSTER_AUTHORIZATION_FAILED \
                 (31)"
            )
        );
        for peer_max_version in [-1, version + 1] {
            let refused = response(
                ErrorCode::UnsupportedVersion,
                &[(ApiKey::ApiVersions.id(), 0, peer_max_version)],
            );
            assert_eq!(
                handshake.on_response(body(&refused, 0)),
                HandshakeStep::Failed(HandshakeError::UnusableRetry {
                    node_id: 2,
                    version,
                    peer_max_version
                }),
                "{peer_max_version}"
            );
        }
        assert!(matches!(
            handshake.on_response(Bytes::from_static(&[0, 0, 0xff])),
            HandshakeStep::Failed(HandshakeError::Malformed { node_id: 2, .. })
        ));
    }

    // As in Kafka, the retry is at the version the peer names, even a higher one; but a
    // peer that keeps refusing gets at most one request per version this node has.
    #[test]
    fn a_peer_naming_other_versions_is_retried_a_bounded_number_of_times() {
        let refusing = |peer_max: i16| {
            body(
                &response(
                    ErrorCode::UnsupportedVersion,
                    &[(ApiKey::ApiVersions.id(), 0, peer_max)],
                ),
                0,
            )
        };
        let mut handshake = ApiVersionsHandshake::new(4);
        let highest = handshake.version();
        let HandshakeStep::Retry(retry) = handshake.on_response(refusing(1)) else {
            panic!("a retry");
        };
        assert_eq!(retry.version(), 1);
        let HandshakeStep::Retry(retry) = retry.on_response(refusing(2)) else {
            panic!("a retry upwards");
        };
        assert_eq!(retry.version(), 2);
        handshake = retry;
        let mut sent = 3;
        let failed = loop {
            match handshake.on_response(refusing(highest)) {
                HandshakeStep::Retry(retry) => {
                    sent += 1;
                    handshake = retry;
                }
                HandshakeStep::Failed(error) => break error,
                HandshakeStep::Negotiated(_) => panic!("refused every time"),
            }
        };
        assert_eq!(sent, highest + 1, "one request per version");
        assert!(matches!(
            failed,
            HandshakeError::UnusableRetry { node_id: 4, .. }
        ));
    }

    // PROTO-0151: the negotiation event, as the network layer logs it.
    #[test]
    fn a_negotiation_prints_the_peer_version_and_each_api() {
        let handshake = ApiVersionsHandshake::new(3);
        let version = handshake.version();
        let HandshakeStep::Negotiated(negotiation) =
            handshake.on_response(answer(version, version))
        else {
            panic!("negotiated");
        };
        let printed = negotiation.to_string();
        assert!(
            printed.starts_with(&format!(
                "node 3 via ApiVersions v{version}: (Produce(0): 3 to 9 [usable: 9], Fetch(1): \
                 UNSUPPORTED, "
            )),
            "{printed}"
        );
        assert!(printed.contains(&format!(
            "ApiVersions(18): 0 to {version} [usable: {version}]"
        )));
    }
}
