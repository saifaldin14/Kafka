//! Alluvium's internal APIs ([ADR-0021](../../../docs/adr/0021-alluvium-internal-api-keys.md)):
//! the rules their schemas must follow beyond Kafka's, which codegen checks before
//! generating them.
//!
//! Every Alluvium schema must be flexible in every version. An internal request or
//! response must also:
//! - have an API key in [`INTERNAL_API_KEYS`];
//! - not use a key Kafka's schemas use, removed APIs included;
//! - not share its key with another internal request, or response;
//! - have a response with its key, or a request.
//!
//! The rules hold for schemas without valid versions too, so a retired API keeps its key.

use std::collections::BTreeMap;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use crate::message_spec::{self, MessageKind, MessageSpec};

/// The API keys reserved for Alluvium's internal APIs.
pub(crate) const INTERNAL_API_KEYS: RangeInclusive<i16> = 32000..=32767;

/// How the protocol crate's `lib.rs` declares a generated module: unformatted, since
/// codegen formats it, and unconditionally.
const DECLARATION: [&str; 2] = ["#[rustfmt::skip]", "pub mod internal;"];

/// How a run generates internal APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Internal {
    /// The Kafka schema directories whose API keys internal APIs must not use.
    pub(crate) kafka: Vec<PathBuf>,
    /// The `lib.rs` that must declare the output directory as `internal` once codegen
    /// writes anything there, so that no generated code goes uncompiled; `None` where
    /// the output is declared anyway.
    pub(crate) declared_in: Option<PathBuf>,
}

/// Fails, naming the schema and the rule, if one of Alluvium's `schemas` breaks one of
/// the module's rules; `kafka` is every API key Kafka's schemas use, by the API's
/// request name.
pub(super) fn check(
    schemas: &[(PathBuf, String, MessageSpec)],
    kafka: &BTreeMap<i16, String>,
) -> Result<(), String> {
    let mut requests: BTreeMap<i16, &str> = BTreeMap::new();
    let mut responses: BTreeMap<i16, &str> = BTreeMap::new();
    for (_, shown, message) in schemas {
        if message.valid_versions.intersect(message.flexible_versions) != message.valid_versions {
            return Err(format!(
                "{shown}: an Alluvium schema must be flexible in every version \
                 (\"flexibleVersions\": \"0+\", ADR-0021)"
            ));
        }
        let by_key = match message.kind {
            MessageKind::Request => &mut requests,
            MessageKind::Response => &mut responses,
            _ => continue,
        };
        let Some(key) = message.api_key else {
            return Err(format!(
                "{shown}: an internal {} has no API key",
                message.name
            ));
        };
        if !INTERNAL_API_KEYS.contains(&key) {
            return Err(format!(
                "{shown}: API key {key} is outside the block reserved for Alluvium's internal \
                 APIs, {} to {} (ADR-0021)",
                INTERNAL_API_KEYS.start(),
                INTERNAL_API_KEYS.end()
            ));
        }
        if let Some(name) = kafka.get(&key) {
            return Err(format!("{shown}: API key {key} is Kafka's {name}"));
        }
        if let Some(other) = by_key.insert(key, &message.name) {
            return Err(format!(
                "{shown}: {} and {other} both use API key {key}",
                message.name
            ));
        }
    }
    // Parity: generator/src/main/java/org/apache/kafka/message/ApiMessageTypeGenerator.java@4.4.0 ApiMessageTypeGenerator.registerMessageType, ApiMessageTypeGenerator.messageSpec
    for (key, name) in &requests {
        if !responses.contains_key(key) {
            return Err(format!("{name} (API key {key}) has no response"));
        }
    }
    for (key, name) in &responses {
        if !requests.contains_key(key) {
            return Err(format!("{name} (API key {key}) has no request"));
        }
    }
    Ok(())
}

/// Every API key the request schemas in `dirs` use, with or without valid versions, by
/// request name.
pub(super) fn kafka_keys(dirs: &[PathBuf]) -> Result<BTreeMap<i16, String>, String> {
    let mut keys = BTreeMap::new();
    for path in super::schema_files(dirs)? {
        let shown = super::shown_path(&path);
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("could not read {shown}: {error}"))?;
        let message = message_spec::load(&shown, &text).map_err(|error| error.to_string())?;
        if let (MessageKind::Request, Some(key)) = (message.kind, message.api_key) {
            keys.insert(key, message.name);
        }
    }
    Ok(keys)
}

/// Fails if codegen writes internal modules that `internal.declared_in` doesn't
/// declare, unconditionally and unformatted, since they would never be compiled.
pub(super) fn check_declared(internal: &Internal, generates_any: bool) -> Result<(), String> {
    let Some(lib) = internal.declared_in.as_deref().filter(|_| generates_any) else {
        return Ok(());
    };
    let text = std::fs::read_to_string(lib)
        .map_err(|error| format!("could not read {}: {error}", lib.display()))?;
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let declared = lines.windows(2).enumerate().any(|(at, pair)| {
        let attribute_before = at
            .checked_sub(1)
            .and_then(|before| lines.get(before))
            .is_some_and(|line| line.starts_with("#["));
        pair == DECLARATION && !attribute_before
    });
    if declared {
        Ok(())
    } else {
        Err(format!(
            "{}: declare the internal APIs codegen generates, with no other attribute: `{}`",
            shown(lib),
            DECLARATION.join(" ")
        ))
    }
}

fn shown(path: &Path) -> String {
    super::shown_path(path)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use serde_json::{Value, json};

    use super::{Internal, check, check_declared, kafka_keys};
    use crate::message_spec::{self, MessageSpec};

    fn schema(kind: &str, key: i64, valid: &str, flexible: &str) -> Value {
        let mut schema = json!({
            "type": kind, "name": format!("Sample{key}{kind}"), "validVersions": valid,
            "flexibleVersions": flexible, "fields": []
        });
        if kind != "data" {
            schema["apiKey"] = json!(key);
        }
        if kind == "request" {
            schema["listeners"] = json!(["broker"]);
        }
        schema
    }

    /// A request and its response, at `key`.
    fn api(key: i64) -> [Value; 2] {
        [
            schema("request", key, "0-1", "0+"),
            schema("response", key, "0-1", "0+"),
        ]
    }

    fn spec(schema: &Value) -> (PathBuf, String, MessageSpec) {
        let message = message_spec::load("Sample.json", &schema.to_string()).unwrap();
        (
            PathBuf::from("Sample.json"),
            "Sample.json".to_owned(),
            message,
        )
    }

    fn checked(schemas: &[Value]) -> Result<(), String> {
        let kafka = BTreeMap::from([(32100, "FutureKafkaRequest".to_owned())]);
        check(&schemas.iter().map(spec).collect::<Vec<_>>(), &kafka)
    }

    #[test]
    fn flexible_apis_in_the_block_pass() {
        let mut schemas = api(32000).to_vec();
        schemas.extend(api(32767));
        schemas.push(schema("data", 0, "0-2", "0+"));
        assert_eq!(checked(&schemas), Ok(()));
    }

    #[test]
    fn each_rule_fails_naming_the_schema() {
        let outside = |key| {
            format!(
                "Sample.json: API key {key} is outside the block reserved for Alluvium's \
                 internal APIs, 32000 to 32767 (ADR-0021)"
            )
        };
        let not_flexible = "Sample.json: an Alluvium schema must be flexible in every version \
                            (\"flexibleVersions\": \"0+\", ADR-0021)";
        let mut twice = api(32001).to_vec();
        twice.push(schema("request", 32001, "0", "0+"));
        for (schemas, expected) in [
            (api(94).to_vec(), outside(94)),
            (api(-1).to_vec(), outside(-1)),
            (
                api(32100).to_vec(),
                "Sample.json: API key 32100 is Kafka's FutureKafkaRequest".to_owned(),
            ),
            (
                twice,
                "Sample.json: Sample32001request and Sample32001request both use API key 32001"
                    .to_owned(),
            ),
            (
                vec![schema("request", 32001, "0-1", "1+")],
                not_flexible.to_owned(),
            ),
            (
                vec![schema("data", 0, "0-1", "none")],
                not_flexible.to_owned(),
            ),
            (
                vec![schema("request", 32002, "0", "0+")],
                "Sample32002request (API key 32002) has no response".to_owned(),
            ),
            (
                vec![schema("response", 32002, "0", "0+")],
                "Sample32002response (API key 32002) has no request".to_owned(),
            ),
        ] {
            assert_eq!(checked(&schemas), Err(expected));
        }
    }

    // A retired API, with no valid versions, still holds its key.
    #[test]
    fn a_retired_api_keeps_its_key() {
        let mut schemas = api(32003).to_vec();
        schemas.push(schema("request", 32003, "none", "0+"));
        schemas.push(schema("response", 32003, "none", "0+"));
        assert!(
            checked(&schemas)
                .unwrap_err()
                .ends_with("both use API key 32003")
        );
        let retired = [
            schema("request", 7, "none", "none"),
            schema("response", 7, "none", "none"),
        ];
        assert!(
            checked(&retired)
                .unwrap_err()
                .contains("API key 7 is outside the block")
        );
    }

    #[test]
    fn kafkas_keys_include_removed_apis() {
        let keys = kafka_keys(&[crate::repo_root().join("schemas/kafka-4.4.0/clients")]).unwrap();
        assert_eq!(keys.get(&0).map(String::as_str), Some("ProduceRequest"));
        // ControlledShutdown, removed with ZooKeeper: its schema has no valid versions.
        assert_eq!(
            keys.get(&7).map(String::as_str),
            Some("ControlledShutdownRequest")
        );
        assert_eq!(keys.keys().next_back(), Some(&94));
    }

    #[test]
    fn generated_internal_apis_must_be_declared_unconditionally() {
        let lib =
            std::env::temp_dir().join(format!("xtask-internal-lib-{}.rs", std::process::id()));
        let internal = Internal {
            kafka: Vec::new(),
            declared_in: Some(lib.clone()),
        };
        std::fs::write(&lib, "pub mod generated;\n").unwrap();
        assert_eq!(check_declared(&internal, false), Ok(()));
        for (text, declared) in [
            ("pub mod generated;\n", false),
            ("pub mod internal;\n", false),
            (
                "#[cfg(any())]\n#[rustfmt::skip]\npub mod internal;\n",
                false,
            ),
            (
                "mod error;\n#[rustfmt::skip]\npub mod internal;\nmod reader;\n",
                true,
            ),
        ] {
            std::fs::write(&lib, text).unwrap();
            let result = check_declared(&internal, true);
            assert_eq!(result.is_ok(), declared, "{text}: {result:?}");
        }
        std::fs::remove_file(&lib).unwrap();
    }
}
