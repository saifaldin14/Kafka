//! Each generated struct's JSON writer: `write_json_fields(&self, json, version)`, and
//! `Message::write_json` for the message itself (`JsonWriter` in the protocol crate).
//!
//! It mirrors Kafka's `JsonConverterGenerator.generateWrite`: the fields present in the
//! version, in schema order, each under its camelCase name; a tagged field only when it
//! differs from its default. Records are summarized and credentials redacted, as the
//! protocol crate's `json` module describes.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use super::decode::{Condition, condition};
use super::encode::non_default;
use super::names;
use crate::message_spec::{FieldSpec, FieldType, Versions};

/// A field's JSON name: its schema name with the first letter lowercased.
// Parity: generator/src/main/java/org/apache/kafka/message/FieldSpec.java@4.4.0 FieldSpec.camelCaseName
fn json_name(name: &str) -> String {
    let mut characters = name.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_lowercase().chain(characters).collect()
    })
}

/// Where a value to write is: a field of `self`, or a reference such as a closure's
/// `value`.
enum Place<'a> {
    Field(&'a str),
    Reference(&'a str),
}

impl Place<'_> {
    fn value(&self) -> String {
        match self {
            Place::Field(name) => format!("self.{name}"),
            Place::Reference(name) => format!("*{name}"),
        }
    }

    fn reference(&self) -> String {
        match self {
            Place::Field(name) => format!("&self.{name}"),
            Place::Reference(name) => (*name).to_owned(),
        }
    }

    fn receiver(&self) -> String {
        match self {
            Place::Field(name) => format!("self.{name}"),
            Place::Reference(name) => (*name).to_owned(),
        }
    }
}

/// An expression writing the non-null value at `place` as `field_type` to the writer
/// `json`, an expression such as `json.key("name")`.
// Parity: generator/src/main/java/org/apache/kafka/message/JsonConverterGenerator.java@4.4.0 JsonConverterGenerator.generateTargetToJson
fn write_value(field_type: &FieldType, place: &Place<'_>, json: &str) -> String {
    match field_type {
        FieldType::Bool => format!("{json}.boolean({})", place.value()),
        FieldType::Int8
        | FieldType::Int16
        | FieldType::Uint16
        | FieldType::Int32
        | FieldType::Uint32 => {
            format!("{json}.integer(i64::from({}))", place.value())
        }
        FieldType::Int64 => format!("{json}.integer({})", place.value()),
        FieldType::Float64 => format!("{json}.float({})", place.value()),
        FieldType::Uuid => format!("{json}.uuid({})", place.value()),
        FieldType::String => format!("{json}.string({})", place.reference()),
        FieldType::Bytes => format!("{json}.bytes({})", place.reference()),
        FieldType::Struct(_) => format!("{}.write_json_fields({json}, version)", place.receiver()),
        FieldType::Array(element) => format!(
            "{json}.array({}, |json, value| {{ {}; }})",
            place.reference(),
            write_value(element, &Place::Reference("value"), "json")
        ),
        FieldType::Records => unreachable!("records are written by write_field"),
    }
}

/// A statement writing `field`, present in the version, under its JSON name.
fn write_field(field: &FieldSpec, sensitive: bool) -> String {
    let key = json_name(&field.name);
    let ident = names::identifier(&field.name);
    let json = format!("json.key({key:?})");
    if field.field_type == FieldType::Records {
        return format!("json.records({key:?}, self.{ident}.as_ref());");
    }
    let nullable = super::structs::is_nullable(field);
    let optional = || {
        let view = if matches!(field.field_type, FieldType::Struct(_)) {
            "as_ref"
        } else {
            "as_deref"
        };
        format!("self.{ident}.{view}()")
    };
    let write = match (sensitive, nullable) {
        (true, true) => format!(
            "{json}.nullable({}, |json, value| {{ json.redacted(value.is_empty()); }})",
            optional()
        ),
        (true, false) => format!("{json}.redacted(self.{ident}.is_empty())"),
        (false, true) => format!(
            "{json}.nullable({}, |json, value| {{ {}; }})",
            optional(),
            write_value(&field.field_type, &Place::Reference("value"), "json")
        ),
        (false, false) => write_value(&field.field_type, &Place::Field(&ident), &json),
    };
    format!("{write};")
}

/// Renders `write_json_fields` for the struct `name` with `fields`, on the wire in
/// `versions`; the fields named in `sensitive` are redacted.
// Parity: generator/src/main/java/org/apache/kafka/message/JsonConverterGenerator.java@4.4.0 JsonConverterGenerator.generateWrite
pub(super) fn render_write_json(
    out: &mut String,
    name: &str,
    fields: &[FieldSpec],
    versions: Versions,
    sensitive: &BTreeSet<&str>,
    runtime: &str,
) {
    let mut body = String::new();
    // Whether the code tests the version or passes it to a nested struct.
    let mut needs_version = false;
    for field in fields {
        let present = condition(field.versions, versions);
        if matches!(present, Condition::Never) {
            continue;
        }
        needs_version |= has_struct(&field.field_type) || matches!(present, Condition::When(_));
        let write = write_field(field, sensitive.contains(field.name.as_str()));
        // One test: the field is present, and if tagged, not its default.
        let tagged = match condition(field.tagged_versions, field.versions.intersect(versions)) {
            Condition::Never => None,
            Condition::Always => Some(non_default(field)),
            Condition::When(test) => {
                needs_version = true;
                Some(format!("(!({test}) || {})", non_default(field)))
            }
        };
        let tests: Vec<String> = [present_test(present), tagged]
            .into_iter()
            .flatten()
            .collect();
        if tests.is_empty() {
            body.push_str(&write);
            body.push('\n');
        } else {
            let _ = writeln!(body, "if {} {{ {write} }}", tests.join(" && "));
        }
    }
    let version = if needs_version { "version" } else { "_version" };
    let _ = write!(
        out,
        "impl {name} {{\n    /// Writes the struct's fields at `version` as a JSON object.\n    \
         pub(crate) fn write_json_fields(&self, json: &mut {runtime}::JsonWriter, {version}: \
         i16) {{\n        json.begin_object();\n        {body}json.end_object();\n    }}\n}}\n\n"
    );
}

/// The test for a field present in some versions, or `None` if it is in all of them.
fn present_test(present: Condition) -> Option<String> {
    match present {
        Condition::When(test) => Some(test),
        Condition::Always | Condition::Never => None,
    }
}

/// Whether `field_type` is a struct or an array of them, whose writer takes the version.
fn has_struct(field_type: &FieldType) -> bool {
    match field_type {
        FieldType::Struct(_) => true,
        FieldType::Array(element) => has_struct(element),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::json_name;

    #[test]
    fn json_names_lowercase_the_first_letter() {
        assert_eq!(json_name("TransactionalId"), "transactionalId");
        assert_eq!(json_name("KRaftVersion"), "kRaftVersion");
        assert_eq!(json_name("myInt8"), "myInt8");
        assert_eq!(json_name(""), "");
    }
}
