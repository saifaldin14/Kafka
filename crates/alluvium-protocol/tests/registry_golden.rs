//! Kafka's own encodings of the API version lists its listeners advertise, recorded by
//! `cargo xtask golden api-keys` in tests/golden/protocol/registry/: each must decode
//! with the generated `ApiVersionsResponse` and re-encode byte for byte.

use std::path::Path;

use alluvium_protocol::generated::api_versions_response::ApiVersionsResponse;
use alluvium_protocol::{Message, Reader, Writer};
use bytes::Bytes;

#[test]
fn kafkas_api_versions_responses_decode_and_re_encode_byte_for_byte() {
    let registry =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/protocol/registry");
    let mut checked = 0;
    for entry in std::fs::read_dir(registry).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        // api_versions_<listener>[_unstable][_telemetry]_v<N>.bin
        let Some(stem) = name
            .strip_prefix("api_versions_")
            .and_then(|n| n.strip_suffix(".bin"))
        else {
            continue;
        };
        let version: i16 = stem.rsplit_once("_v").unwrap().1.parse().unwrap();
        let bytes = Bytes::from(std::fs::read(&path).unwrap());
        let response = ApiVersionsResponse::decode(&mut Reader::new(bytes.clone()), version)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert!(!response.api_keys.is_empty(), "{name}");
        let mut writer = Writer::new();
        response.encode(&mut writer, version).unwrap();
        assert_eq!(writer.into_bytes(), bytes, "{name}");
        checked += 1;
    }
    assert_eq!(
        checked, 6,
        "one file per listener, unstable and telemetry setting"
    );
}
