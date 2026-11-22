//! Kafka's message schemas, read and checked as Kafka's generator reads them
//! ([ADR-0008](../../docs/adr/0008-protocol-code-generated-from-kafka-schemas.md)).
//!
//! This is the input to `cargo xtask codegen`. `cargo xtask schemas verify` loads every
//! vendored schema with it, so a schema it can't read fails CI before any code is
//! generated from it.
//!
//! Loading is three steps:
//!
//! 1. [`json`] reads the file's JSON with comments; a syntax error has a line and column.
//! 2. [`spec`] builds the typed [`MessageSpec`] with every rule Kafka's constructors
//!    check; a problem has the JSON pointer of the offending value.
//! 3. [`registry`] registers the message's structs and checks every struct a field names.
//!
//! Kafka counterpart: `generator/src/main/java/org/apache/kafka/message/` at 4.4.0
//! (`MessageSpec`, `StructSpec`, `FieldSpec`, `FieldType`, `Versions`, `EntityType`,
//! `StructRegistry`). Each rule cites the Kafka code it mirrors with `// Parity:`.

mod defaults;
mod field_type;
mod json;
mod registry;
mod spec;
mod versions;

use std::collections::BTreeMap;
use std::fmt;

pub(crate) use registry::StructRegistry;
pub(crate) use spec::{MessageKind, MessageSpec};

/// Why a schema file can't be used, and where in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchemaError {
    /// The file, as the caller named it.
    pub(crate) file: String,
    pub(crate) location: Location,
    pub(crate) message: String,
}

/// Where in a schema file a problem is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Location {
    /// A 1-based line and byte column, for JSON syntax errors.
    Text { line: usize, column: usize },
    /// A JSON pointer, such as `/fields/3/type`, for valid JSON that breaks a rule.
    Pointer(String),
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.location {
            Location::Text { line, column } => {
                write!(f, "{}:{line}:{column}: {}", self.file, self.message)
            }
            Location::Pointer(pointer) if pointer.is_empty() => {
                write!(f, "{}: {}", self.file, self.message)
            }
            Location::Pointer(pointer) => {
                write!(f, "{} at {pointer}: {}", self.file, self.message)
            }
        }
    }
}

/// Reads and checks the schema in `text`, from the file named `file`.
pub(crate) fn load(file: &str, text: &str) -> Result<MessageSpec, SchemaError> {
    let error = |location, message| SchemaError {
        file: file.to_owned(),
        location,
        message,
    };
    let value = json::parse(text).map_err(|syntax| {
        error(
            Location::Text {
                line: syntax.line,
                column: syntax.column,
            },
            syntax.message,
        )
    })?;
    let from_problem =
        |problem: spec::Problem| error(Location::Pointer(problem.pointer), problem.message);
    let message = MessageSpec::from_json(&value).map_err(from_problem)?;
    StructRegistry::new(&message).map_err(from_problem)?;
    Ok(message)
}

/// What a set of schemas defines, for `schemas verify` to report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Inventory {
    pub(crate) schemas: usize,
    /// Request schemas with valid versions, by API key.
    pub(crate) requests: BTreeMap<i16, String>,
    /// Response schemas with valid versions, by API key.
    pub(crate) responses: BTreeMap<i16, String>,
    /// Fields that are arrays of a struct with a `mapKey`.
    pub(crate) keyed_arrays: usize,
}

impl Inventory {
    /// Adds a loaded schema.
    pub(crate) fn add(&mut self, message: &MessageSpec) {
        self.schemas += 1;
        let by_key = match message.kind {
            MessageKind::Request => Some(&mut self.requests),
            MessageKind::Response => Some(&mut self.responses),
            _ => None,
        };
        if let (Some(by_key), Some(api_key)) = (by_key, message.api_key)
            && !message.valid_versions.is_empty()
        {
            by_key.insert(api_key, message.name.clone());
        }
        if let Ok(registry) = StructRegistry::new(message) {
            self.keyed_arrays += count_keyed(&registry, &message.fields)
                + message
                    .common_structs
                    .iter()
                    .map(|common| count_keyed(&registry, &common.fields))
                    .sum::<usize>();
        }
    }

    /// API keys with a request but no response, or a response but no request.
    pub(crate) fn unpaired(&self) -> Vec<i16> {
        let requests = self.requests.keys();
        let responses = self.responses.keys();
        let mut unpaired: Vec<i16> = requests
            .filter(|key| !self.responses.contains_key(key))
            .chain(responses.filter(|key| !self.requests.contains_key(key)))
            .copied()
            .collect();
        unpaired.sort_unstable();
        unpaired
    }
}

fn count_keyed(registry: &StructRegistry<'_>, fields: &[spec::FieldSpec]) -> usize {
    fields
        .iter()
        .map(|field| {
            usize::from(registry.is_struct_array_with_keys(field))
                + count_keyed(registry, &field.fields)
        })
        .sum()
}

impl fmt::Display for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} schemas: {} request and {} response APIs, {} keyed struct arrays",
            self.schemas,
            self.requests.len(),
            self.responses.len(),
            self.keyed_arrays
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::spec::FieldSpec;
    use super::versions::Versions;
    use super::{Inventory, MessageKind, MessageSpec, load};

    /// Every vendored 4.4.0 schema, by message name.
    fn vendored() -> BTreeMap<String, MessageSpec> {
        let dir = crate::repo_root().join("schemas/kafka-4.4.0");
        let mut messages = BTreeMap::new();
        for directory in std::fs::read_dir(&dir).unwrap() {
            let directory = directory.unwrap().path();
            if !directory.is_dir() {
                continue;
            }
            for file in std::fs::read_dir(&directory).unwrap() {
                let path = file.unwrap().path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    let text = std::fs::read_to_string(&path).unwrap();
                    let message = load(&path.display().to_string(), &text)
                        .unwrap_or_else(|error| panic!("{error}"));
                    messages.insert(message.name.clone(), message);
                }
            }
        }
        messages
    }

    fn field<'a>(messages: &'a BTreeMap<String, MessageSpec>, path: &[&str]) -> &'a FieldSpec {
        let (message, names) = path.split_first().unwrap();
        let mut fields = &messages[*message].fields;
        let mut found = None;
        for name in names {
            let next = fields.iter().find(|field| field.name == *name).unwrap();
            fields = &next.fields;
            found = Some(next);
        }
        found.unwrap()
    }

    fn range(text: &str) -> Versions {
        Versions::parse(Some(text), None).unwrap().unwrap()
    }

    #[test]
    fn every_vendored_schema_loads_and_the_inventory_matches_kafka_4_4_0() {
        let messages = vendored();
        let mut inventory = Inventory::default();
        for message in messages.values() {
            inventory.add(message);
        }
        assert_eq!(inventory.schemas, 281);
        let requests = messages
            .values()
            .filter(|message| message.kind == MessageKind::Request)
            .count();
        // 95 request schemas; four ZooKeeper-era APIs keep their key with `none`.
        assert_eq!(requests, 95);
        assert_eq!(inventory.requests.len(), 91);
        assert_eq!(inventory.responses.len(), 91);
        assert_eq!(inventory.unpaired(), Vec::<i16>::new());
        assert_eq!(inventory.keyed_arrays, 76);
        assert_eq!(messages["ProduceRequest"].valid_versions, range("3-13"));
        assert_eq!(messages["FetchRequest"].valid_versions, range("4-18"));
    }

    #[test]
    fn vendored_schemas_are_modeled_as_kafka_reads_them() {
        let messages = vendored();
        let transactional_id = field(&messages, &["ProduceRequest", "TransactionalId"]);
        assert_eq!(transactional_id.versions, range("3+"));
        assert_eq!(
            transactional_id.present_versions(messages["ProduceRequest"].valid_versions),
            range("3-13")
        );

        let client_id = field(&messages, &["RequestHeader", "ClientId"]);
        let header = &messages["RequestHeader"];
        assert!(header.flexible_versions.contains_all(range("2")));
        assert!(
            !client_id
                .flexible_versions(header.flexible_versions)
                .contains_all(range("2"))
        );

        let error_message = field(
            &messages,
            &[
                "ProduceResponse",
                "Responses",
                "PartitionResponses",
                "ErrorMessage",
            ],
        );
        assert_eq!(error_message.nullable_versions, range("8+"));
        assert!(
            field(
                &messages,
                &[
                    "ProduceResponse",
                    "Responses",
                    "PartitionResponses",
                    "RecordErrors"
                ]
            )
            .ignorable
        );
        assert!(
            !field(
                &messages,
                &[
                    "ProduceResponse",
                    "Responses",
                    "PartitionResponses",
                    "ErrorCode"
                ]
            )
            .ignorable
        );

        let unstable: Vec<&str> = messages
            .values()
            .filter(|message| message.latest_version_unstable)
            .map(|message| message.name.as_str())
            .collect();
        assert_eq!(unstable, ["InitProducerIdRequest"]);

        let mut zero_copy = Vec::new();
        for message in messages.values() {
            for top in &message.fields {
                if top.zero_copy {
                    zero_copy.push(format!("{}.{}", message.name, top.name));
                }
            }
        }
        zero_copy.sort();
        assert_eq!(
            zero_copy,
            [
                "ConsumerProtocolAssignment.UserData",
                "ConsumerProtocolSubscription.UserData",
                "EnvelopeRequest.RequestData",
                "EnvelopeResponse.ResponseData",
                "PushTelemetryRequest.Metrics",
            ]
        );
    }
}
