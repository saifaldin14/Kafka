// Portions of this file are ported from Apache Kafka 4.4.0 and modified by Alluvium;
// each is marked `// Ported from:`. Apache Kafka is Copyright The Apache Software
// Foundation and licensed under the Apache License 2.0 (LICENSES/Apache-2.0.txt).
// See NOTICE.

//! The API registry against Kafka's own, recorded by `cargo xtask golden api-keys` in
//! tests/golden/protocol/registry/: `ApiKey` and its rules must say what Kafka's
//! `ApiKeys` says about every API, and the API version lists each listener advertises
//! must equal Kafka's, byte for byte on the wire. A peer's versions read from each list
//! must print as Kafka's `NodeApiVersions` prints them.

use std::path::{Path, PathBuf};

use alluvium_protocol::generated::api_versions_response::ApiVersionsResponse;
use alluvium_protocol::{
    ApiKey, Listener, Message, NodeApiVersions, Reader, Versions, Writer, advertised_api_versions,
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
        for (key, header_version) in [
            (
                "request_header_versions",
                ApiKey::request_header_version as fn(ApiKey, i16) -> _,
            ),
            ("response_header_versions", ApiKey::response_header_version),
        ] {
            match entry[key].as_array() {
                None => assert_eq!(header_version(api, 0), None, "{name}: {key}"),
                Some(pairs) => {
                    for pair in pairs {
                        let version = i16::try_from(pair[0].as_i64().unwrap()).unwrap();
                        let kafka = i16::try_from(pair[1].as_i64().unwrap()).unwrap();
                        assert_eq!(
                            header_version(api, version),
                            Some(kafka),
                            "{name}: {key} at version {version}"
                        );
                    }
                }
            }
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

// PROTO-0148, PROTO-0151: a Kafka broker's or controller's list, read as a peer's.
// Ported from: clients/src/test/java/org/apache/kafka/clients/NodeApiVersionsTest.java@4.4.0 (testUsableVersionLatestVersions, testConstructionFromApiVersionsResponse)
#[test]
fn kafkas_lists_read_as_kafkas_client_reads_them() {
    let mut checked = 0;
    for entry in std::fs::read_dir(registry()).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let Some(stem) = name.strip_suffix(".bin") else {
            continue;
        };
        let Some((_, version)) = stem.rsplit_once("_v") else {
            continue;
        };
        let bytes = Bytes::from(std::fs::read(&path).unwrap());
        let response =
            ApiVersionsResponse::decode(&mut Reader::new(bytes), version.parse().unwrap()).unwrap();
        let peer = NodeApiVersions::new(&response);
        let printed = std::fs::read_to_string(path.with_extension("txt")).unwrap();
        assert_eq!(peer.to_string(), printed.trim_end(), "{name}");
        assert!(peer.unknown_apis().is_empty(), "{name}");
        for listed in &response.api_keys {
            let api = ApiKey::from_id(listed.api_key).unwrap();
            let versions = peer.api_version(api).unwrap();
            assert_eq!(
                (versions.min_version, versions.max_version),
                (listed.min_version, listed.max_version),
                "{name}: {}",
                api.name()
            );
            let latest = api.versions().highest().unwrap();
            assert_eq!(
                peer.latest_usable_version(api, api.versions()),
                Ok(latest.min(listed.max_version)),
                "{name}: {}",
                api.name()
            );
            if stem.contains("unstable") {
                assert_eq!(listed.max_version, latest, "{name}: {}", api.name());
            }
        }
        checked += 1;
    }
    assert_eq!(checked, 6);
}
