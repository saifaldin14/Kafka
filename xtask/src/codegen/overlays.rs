//! The overlays in `xtask/codegen/`: facts about Kafka's fields that the schemas don't
//! state, applied by name when generating code.
//!
//! - `known_values.toml`: integer fields whose values are a known set, such as error
//!   codes. Each matching field gets a typed accessor beside the raw integer.
//! - `sensitive_fields.toml`: fields that hold credentials. The generated `Debug` of
//!   their structs prints `***` for them.
//!
//! [`Overlays::check`] fails on an entry that names no generated field, or names a field
//! of another type, so a stale or mistyped entry can't silently stop applying.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::message_spec::{FieldSpec, FieldType, MessageSpec};

/// An enum that reads an integer field's values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KnownValue {
    /// The wire type every field it applies to must have.
    pub(crate) wire_type: FieldType,
    /// The enum, the function that converts a raw value to it, and that function's
    /// error, each as a path with a `protocol::` or `types::` prefix.
    pub(crate) rust: String,
    pub(crate) convert: String,
    pub(crate) error: String,
}

/// Every overlay, as loaded.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Overlays {
    /// Known values, by field name.
    pub(crate) known_values: BTreeMap<String, KnownValue>,
    /// Sensitive field names, by message.
    pub(crate) sensitive: BTreeMap<String, BTreeSet<String>>,
}

const KNOWN_VALUES: &str = "known_values.toml";
const SENSITIVE_FIELDS: &str = "sensitive_fields.toml";

impl Overlays {
    /// Loads the overlay files in `dir`.
    pub(crate) fn load(dir: &Path) -> Result<Overlays, String> {
        let read = |file: &str| -> Result<toml::Table, String> {
            let text = std::fs::read_to_string(dir.join(file))
                .map_err(|error| format!("could not read {file}: {error}"))?;
            toml::from_str(&text).map_err(|error| format!("{file}: {error}"))
        };
        let mut overlays = Overlays::default();
        for (index, entry) in entries(&read(KNOWN_VALUES)?, "known_value", KNOWN_VALUES)? {
            let at = |problem: String| format!("{KNOWN_VALUES}: known_value {index}: {problem}");
            let wire_type = FieldType::parse(&text(entry, "type").map_err(at)?).map_err(at)?;
            let known = KnownValue {
                wire_type,
                rust: path(entry, "rust").map_err(at)?,
                convert: path(entry, "convert").map_err(at)?,
                error: path(entry, "error").map_err(at)?,
            };
            for field in names(entry).map_err(at)? {
                if overlays
                    .known_values
                    .insert(field.clone(), known.clone())
                    .is_some()
                {
                    return Err(at(format!("{field} is listed twice")));
                }
            }
        }
        for (index, entry) in entries(&read(SENSITIVE_FIELDS)?, "sensitive", SENSITIVE_FIELDS)? {
            let at = |problem: String| format!("{SENSITIVE_FIELDS}: sensitive {index}: {problem}");
            let message = text(entry, "message").map_err(at)?;
            overlays
                .sensitive
                .entry(message)
                .or_default()
                .extend(names(entry).map_err(at)?);
        }
        Ok(overlays)
    }

    /// The known value of `field`, if an overlay names it.
    pub(crate) fn known_value(&self, field: &FieldSpec) -> Option<&KnownValue> {
        self.known_values.get(&field.name)
    }

    /// Whether `field` of `message` holds a secret.
    pub(crate) fn is_sensitive(&self, message: &str, field: &str) -> bool {
        self.sensitive
            .get(message)
            .is_some_and(|fields| fields.contains(field))
    }

    /// Checks every entry against the messages being generated: each known-value name
    /// matches at least one field, all of its type, and each sensitive field exists in
    /// its message.
    pub(crate) fn check(&self, messages: &[MessageSpec]) -> Result<(), String> {
        let mut problems = Vec::new();
        let mut matched = BTreeSet::new();
        for message in messages {
            for field in all_fields(message) {
                if let Some(known) = self.known_value(field) {
                    matched.insert(field.name.as_str());
                    if field.field_type != known.wire_type {
                        problems.push(format!(
                            "{KNOWN_VALUES}: {}.{} is {}, not {}",
                            message.name, field.name, field.field_type, known.wire_type
                        ));
                    }
                }
            }
        }
        for name in self.known_values.keys() {
            if !matched.contains(name.as_str()) {
                problems.push(format!(
                    "{KNOWN_VALUES}: no generated field is named {name}"
                ));
            }
        }
        for (message_name, fields) in &self.sensitive {
            let Some(message) = messages
                .iter()
                .find(|message| message.name == *message_name)
            else {
                problems.push(format!(
                    "{SENSITIVE_FIELDS}: {message_name} is not a generated message"
                ));
                continue;
            };
            for name in fields {
                if !all_fields(message).any(|field| field.name == *name) {
                    problems.push(format!(
                        "{SENSITIVE_FIELDS}: {message_name} has no field {name}"
                    ));
                }
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems.join("\n"))
        }
    }
}

/// `path`, an overlay path, as Rust code: `protocol::ErrorCode` is `{runtime}::ErrorCode`.
pub(crate) fn rust_path(path: &str, runtime: &str) -> String {
    match path.split_once("::") {
        Some(("protocol", rest)) => format!("{runtime}::{rest}"),
        _ => format!("alluvium_{path}"),
    }
}

/// Every field of `message`, including those of its inline and common structs.
fn all_fields(message: &MessageSpec) -> impl Iterator<Item = &FieldSpec> {
    let mut pending: Vec<&FieldSpec> = message.fields.iter().collect();
    pending.extend(
        message
            .common_structs
            .iter()
            .flat_map(|common| &common.fields),
    );
    std::iter::from_fn(move || {
        let field = pending.pop()?;
        pending.extend(&field.fields);
        Some(field)
    })
}

/// The tables of the array `key`, numbered from 1.
fn entries<'a>(
    table: &'a toml::Table,
    key: &str,
    file: &str,
) -> Result<Vec<(usize, &'a toml::Table)>, String> {
    let unknown: Vec<&String> = table.keys().filter(|found| *found != key).collect();
    if let Some(found) = unknown.first() {
        return Err(format!(
            "{file}: unknown key `{found}`; expected [[{key}]] entries"
        ));
    }
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .ok_or_else(|| format!("{file}: no [[{key}]] entries"))?
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            entry
                .as_table()
                .map(|entry| (index + 1, entry))
                .ok_or_else(|| format!("{file}: {key} {} is not a table", index + 1))
        })
        .collect()
}

fn text(entry: &toml::Table, key: &str) -> Result<String, String> {
    entry
        .get(key)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("`{key}` must be a string"))
}

/// A path that starts with `protocol::` or `types::`.
fn path(entry: &toml::Table, key: &str) -> Result<String, String> {
    let path = text(entry, key)?;
    if path.starts_with("protocol::") || path.starts_with("types::") {
        Ok(path)
    } else {
        Err(format!(
            "`{key}` = `{path}` must start with `protocol::` or `types::`"
        ))
    }
}

fn names(entry: &toml::Table) -> Result<Vec<String>, String> {
    let fields = entry
        .get("fields")
        .and_then(toml::Value::as_array)
        .ok_or("`fields` must be an array of field names")?;
    fields
        .iter()
        .map(|field| {
            field
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "`fields` must hold strings".to_owned())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Overlays, rust_path};
    use crate::message_spec;

    fn overlays(known: &str, sensitive: &str) -> Result<Overlays, String> {
        let dir = std::env::temp_dir().join(format!(
            "xtask-overlays-{}-{}",
            known.len(),
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("known_values.toml"), known).unwrap();
        std::fs::write(dir.join("sensitive_fields.toml"), sensitive).unwrap();
        let loaded = Overlays::load(&dir);
        std::fs::remove_dir_all(&dir).unwrap();
        loaded
    }

    const KNOWN: &str = r#"
        [[known_value]]
        fields = ["ErrorCode"]
        type = "int16"
        rust = "protocol::ErrorCode"
        convert = "protocol::ErrorCode::from_code"
        error = "protocol::UnknownErrorCode"
    "#;

    const SENSITIVE: &str = r#"
        [[sensitive]]
        message = "Login"
        fields = ["Password"]
    "#;

    fn message(fields: &str) -> message_spec::MessageSpec {
        message_spec::load(
            "Login.json",
            &format!(
                r#"{{ "type": "data", "name": "Login", "validVersions": "0",
                     "flexibleVersions": "none", "fields": [{fields}] }}"#
            ),
        )
        .unwrap()
    }

    #[test]
    fn the_vendored_overlays_load_and_match_the_generated_messages() {
        let loaded = Overlays::load(&crate::repo_root().join("xtask/codegen")).unwrap();
        assert!(loaded.known_values.contains_key("ErrorCode"));
        assert!(loaded.is_sensitive("SaslAuthenticateRequest", "AuthBytes"));
        assert!(!loaded.is_sensitive("SaslAuthenticateRequest", "ErrorCode"));
    }

    #[test]
    fn entries_must_match_fields_of_their_type() {
        let loaded = overlays(KNOWN, SENSITIVE).unwrap();
        let good = message(
            r#"{ "name": "ErrorCode", "type": "int16", "versions": "0+" },
               { "name": "Password", "type": "bytes", "versions": "0+" }"#,
        );
        assert_eq!(loaded.check(std::slice::from_ref(&good)), Ok(()));

        let wrong_type = message(
            r#"{ "name": "ErrorCode", "type": "int32", "versions": "0+" },
               { "name": "Password", "type": "bytes", "versions": "0+" }"#,
        );
        let problem = loaded.check(&[wrong_type]).unwrap_err();
        assert!(
            problem.contains("Login.ErrorCode is int32, not int16"),
            "{problem}"
        );

        let missing = message(r#"{ "name": "Other", "type": "int8", "versions": "0+" }"#);
        let problem = loaded.check(&[missing]).unwrap_err();
        assert!(
            problem.contains("no generated field is named ErrorCode"),
            "{problem}"
        );
        assert!(problem.contains("Login has no field Password"), "{problem}");
        let problem = loaded.check(&[]).unwrap_err();
        assert!(
            problem.contains("Login is not a generated message"),
            "{problem}"
        );
    }

    #[test]
    fn malformed_entries_fail_naming_the_file_and_entry() {
        let problem = overlays(
            &KNOWN.replace("protocol::ErrorCode\"", "ErrorCode\""),
            SENSITIVE,
        )
        .unwrap_err();
        assert!(
            problem.starts_with("known_values.toml: known_value 1:"),
            "{problem}"
        );
        let problem = overlays(KNOWN, "[[secret]]\nmessage = \"A\"").unwrap_err();
        assert!(problem.contains("unknown key `secret`"), "{problem}");
    }

    #[test]
    fn paths_resolve_to_the_runtime_crate_or_alluvium_types() {
        assert_eq!(
            rust_path("protocol::ErrorCode", "crate"),
            "crate::ErrorCode"
        );
        assert_eq!(
            rust_path("types::IsolationLevel::from_wire", "crate"),
            "alluvium_types::IsolationLevel::from_wire"
        );
    }
}
