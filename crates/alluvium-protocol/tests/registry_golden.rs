//! The API registry against Kafka's own, recorded by `cargo xtask golden api-keys` in
//! tests/golden/protocol/registry/: `ApiKey` and its rules must say what Kafka's
//! `ApiKeys` says about every API, and the API version lists each listener advertises
//! must equal Kafka's, byte for byte on the wire.

use std::path::{Path, PathBuf};

use alluvium_protocol::generated::api_versions_response::ApiVersionsResponse;
use alluvium_protocol::{
    ApiKey, Listener, Message, Reader, Versions, Writer, advertised_api_versions,
};
use bytes::Bytes;

fn registry() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/protocol/registry")
}

#[test]
fn every_api_matches_kafkas_api_keys() {
    let text = std::fs::read_to_string(registry().join("api_keys.json")).unwrap();
    let entries: serde_json::Value = serde_json::from_str(&text).unwrap();
    let entries = entries.as_array().unwrap();
    assert_eq!(entries.len(), ApiKey::ALL.len(), "API count");
    for entry in entries {
        let number = |key: &str| i16::try_from(entry[key].as_i64().unwrap()).unwrap();
        let flag = |key: &str| entry[key].as_bool().unwrap();
        let id = number("id");
        let api = ApiKey::from_id(id).unwrap_or_else(|| panic!("API key {id} is missing"));
        let name = entry["name"].as_str().unwrap();
        assert_eq!(api.name(), name, "API key {id}: name");
        // Kafka writes an API without versions as 0 to -1.
        let (oldest, latest) = (number("oldest"), number("latest"));
        let versions = if oldest > latest {
            Versions::NONE
        } else {
            Versions::new(oldest, latest)
        };
        assert_eq!(api.versions(), versions, "{name}: versions");
        let stable = number("latest_stable");
        let served = |latest: i16| (latest >= oldest).then_some(latest);
        assert_eq!(
            api.latest_version(false),
            served(stable),
            "{name}: latest stable"
        );
        assert_eq!(api.latest_version(true), served(latest), "{name}: latest");
        let unstable = flag("latest_version_unstable");
        assert_eq!(api.latest_version_unstable(), unstable, "{name}: unstable");
        assert_eq!(
            api.is_cluster_action(),
            flag("cluster_action"),
            "{name}: clusterAction"
        );
        assert_eq!(
            api.is_forwardable(),
            flag("forwardable"),
            "{name}: forwardable"
        );
        let listeners: Vec<&str> = entry["listeners"]
            .as_array()
            .unwrap()
            .iter()
            .map(|listener| listener.as_str().unwrap())
            .collect();
        for (listener, kafka) in [
            (Listener::Broker, "BROKER"),
            (Listener::Controller, "CONTROLLER"),
        ] {
            let served = listeners.contains(&kafka);
            assert_eq!(
                api.is_served_on(listener),
                served,
                "{name}: {kafka} listener"
            );
            assert_eq!(
                ApiKey::served_on(listener).any(|served| served == api),
                served
            );
        }
        let deprecated = entry["deprecated"].as_array().unwrap();
        let (low, high) = (
            deprecated[0].as_i64().unwrap(),
            deprecated[1].as_i64().unwrap(),
        );
        for version in -1..=latest.saturating_add(1) {
            let kafka = (low..=high).contains(&i64::from(version));
            assert_eq!(
                api.is_version_deprecated(version),
                kafka,
                "{name}: deprecated version {version}"
            );
        }
    }
}

#[test]
fn each_listener_advertises_kafkas_api_versions() {
    let everything: Vec<(ApiKey, Versions)> = ApiKey::ALL
        .into_iter()
        .map(|api| (api, api.versions()))
        .collect();
    let mut checked = 0;
    for entry in std::fs::read_dir(registry()).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        // api_versions_<listener>[_unstable][_telemetry]_v<N>.bin
        let Some(stem) = name
            .strip_prefix("api_versions_")
            .and_then(|n| n.strip_suffix(".bin"))
        else {
            continue;
        };
        let (setting, version) = stem.rsplit_once("_v").unwrap();
        let version: i16 = version.parse().unwrap();
        let listener = if setting.starts_with("broker") {
            Listener::Broker
        } else {
            Listener::Controller
        };
        let bytes = Bytes::from(std::fs::read(&path).unwrap());
        let kafka = ApiVersionsResponse::decode(&mut Reader::new(bytes.clone()), version)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let advertised = advertised_api_versions(
            listener,
            setting.contains("unstable"),
            setting.contains("telemetry"),
            &everything,
        );
        assert_eq!(advertised, kafka.api_keys, "{name}");
        let response = ApiVersionsResponse {
            api_keys: advertised,
            ..ApiVersionsResponse::default()
        };
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
