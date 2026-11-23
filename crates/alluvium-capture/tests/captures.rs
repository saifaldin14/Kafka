//! The packet captures in `tests/captures/`, which `record.sh` recorded from a Kafka
//! 4.4.0 broker and Kafka's own console producer, a Java client: every frame of the
//! session reads and decodes, and every response pairs with its request (PROTO-0144,
//! PROTO-0145).

use std::collections::BTreeSet;
use std::path::Path;

use alluvium_capture::exchanges::exchanges;
use alluvium_capture::read_capture;
use alluvium_protocol::{FrameDump, FrameHeader};

/// A dump that read its whole frame and decoded its body.
fn decoded(dump: &FrameDump) -> bool {
    !dump.is_truncated()
        && (dump.trailing, dump.beyond_frame) == (0, 0)
        && matches!(dump.body, Some(Ok(_)))
}

#[test]
fn a_java_producer_session_decodes_end_to_end() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/captures");
    // Each request's API name, request JSON and response JSON, in capture order.
    let session = |name: &str| -> Vec<(String, String, String)> {
        let capture = read_capture(&std::fs::read(directory.join(name)).unwrap()).unwrap();
        assert!(
            capture.problems.is_empty(),
            "{name}: {:?}",
            capture.problems
        );
        assert!(
            capture.connections.len() >= 2,
            "{name}: an admin and a producer"
        );
        let mut session = Vec::new();
        for connection in &capture.connections {
            assert!(connection.roles_from_handshake, "{name}");
            assert_eq!(connection.server.port(), 9092, "{name}");
            assert!(
                connection.problems.is_empty(),
                "{name}: {:?}",
                connection.problems
            );
            let paired = exchanges(connection);
            assert!(
                paired.unmatched.is_empty(),
                "{name}: {:?}",
                paired.unmatched
            );
            for exchange in paired.exchanges {
                let request = exchange.request_dump.unwrap();
                assert!(decoded(&request), "{name}: {request:?}");
                let Ok(FrameHeader::Request(header)) = &request.header else {
                    panic!("{name}: a request header");
                };
                let (_, response) = exchange.response.expect("every request is answered");
                let response = response.unwrap();
                assert!(decoded(&response), "{name}: {response:?}");
                session.push((
                    header.api.name().to_owned(),
                    request.body.unwrap().unwrap(),
                    response.body.unwrap().unwrap(),
                ));
            }
        }
        session
    };
    let ethernet = session("producer-eth0.pcap");
    let apis: BTreeSet<&str> = ethernet.iter().map(|(api, _, _)| api.as_str()).collect();
    for api in ["ApiVersions", "Metadata", "CreateTopics", "Produce"] {
        assert!(apis.contains(api), "{api} in {apis:?}");
    }
    let produce = ethernet
        .iter()
        .find(|(api, _, _)| api == "Produce")
        .unwrap();
    assert!(
        produce.1.contains("\"recordsSizeInBytes\":"),
        "{}",
        produce.1
    );
    assert!(
        !produce.1.contains("hello alluvium"),
        "records are never printed"
    );
    // The same session, captured on every interface as Linux cooked frames in pcapng.
    assert_eq!(session("producer-any.pcapng"), ethernet);
}
