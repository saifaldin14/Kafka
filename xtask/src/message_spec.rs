//! Kafka's message schemas, read and checked as Kafka's generator reads them
//! ([ADR-0008](../../docs/adr/0008-protocol-code-generated-from-kafka-schemas.md)).
//!
//! This is the input to `cargo xtask codegen`. `cargo xtask schemas verify` loads every
//! vendored schema with it, so a schema it can't read fails CI before any code is
//! generated from it.
//!
//! Kafka counterpart: `generator/src/main/java/org/apache/kafka/message/` at 4.4.0.
//! Each rule cites the Kafka code it mirrors with `// Parity:`.

mod json;

use std::fmt;

use serde_json::Value;

/// Why a schema file can't be used, and where in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchemaError {
    /// The file, as the caller named it.
    pub(crate) file: String,
    /// A 1-based line and byte column in the file.
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) message: String,
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.file, self.line, self.column, self.message
        )
    }
}

/// Reads the schema in `text`, from the file named `file`.
pub(crate) fn load(file: &str, text: &str) -> Result<Value, SchemaError> {
    json::parse(text).map_err(|error| SchemaError {
        file: file.to_owned(),
        line: error.line,
        column: error.column,
        message: error.message,
    })
}
