//! One message's Rust module: its struct, every struct its fields define or share, each
//! struct's `Default`, and the message's version constants.
//!
//! Field types map as follows (docs/protocol/codegen.md has the rest of the mapping):
//!
//! | Schema type | Rust type |
//! |---|---|
//! | `bool` | `bool` |
//! | `int8`, `int16`, `uint16`, `int32`, `uint32`, `int64` | `i8`, `i16`, `u16`, `i32`, `u32`, `i64` |
//! | `float64` | `f64` |
//! | `uuid` | `alluvium_types::Uuid` |
//! | `string` | `String` |
//! | `bytes` | `bytes::Bytes` |
//! | `records` | `Option<RecordsField>`, always: Kafka defaults records to null |
//! | `[]T` | `Vec<T>` |
//! | a struct name | the generated struct |
//!
//! A field nullable in any of its versions is an `Option` of its type.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use alluvium_types::Uuid;

use super::docs;
use super::names::{self, UNKNOWN_TAGGED_FIELDS};
use crate::message_spec::{
    FieldDefault, FieldSpec, FieldType, MessageKind, MessageSpec, StructRegistry, Versions,
};

/// The deepest struct nesting a message may have. Generated decoders read each struct
/// level in its own function call, so this bounds their stack depth for any input. The
/// deepest Kafka 4.4.0 message, `StreamsGroupDescribeResponse`, has 6 levels.
pub(crate) const MAX_NESTING_DEPTH: usize = 8;

/// A rendered module, before formatting.
pub(crate) struct Module {
    pub(crate) source: String,
    /// The levels of struct nesting, counting the message itself.
    pub(crate) depth: usize,
}

/// One struct to emit.
struct Struct<'a> {
    name: &'a str,
    fields: &'a [FieldSpec],
    /// The versions in which the struct is on the wire.
    versions: Versions,
    doc: String,
}

/// Renders `message`'s module. `runtime` is the path of the crate the generated code
/// uses for `RawTaggedField`, `RecordsField` and `Versions`.
pub(crate) fn module(message: &MessageSpec, runtime: &str) -> Result<Module, String> {
    let registry = StructRegistry::new(message)
        .map_err(|problem| format!("{} at {}", problem.message, problem.pointer))?;
    let mut structs = vec![Struct {
        name: &message.name,
        fields: &message.fields,
        versions: message.valid_versions,
        doc: message_doc(message),
    }];
    let mut seen = BTreeSet::from([message.name.as_str()]);
    let depth = collect(
        &registry,
        &message.name,
        &message.fields,
        &mut seen,
        &mut structs,
        &mut Vec::new(),
    )?;
    if depth > MAX_NESTING_DEPTH {
        return Err(format!(
            "{} nests structs {depth} levels deep, more than the generator's limit of \
             {MAX_NESTING_DEPTH}",
            message.name
        ));
    }
    let mut source = String::new();
    for item in &structs {
        names::check_struct_name(item.name)?;
        names::check_collisions(item.fields.iter().map(|field| field.name.as_str()))
            .map_err(|problem| format!("in struct {}: {problem}", item.name))?;
        let flexible = !item
            .versions
            .intersect(message.flexible_versions)
            .is_empty();
        render_struct(&mut source, item, flexible, runtime);
    }
    render_constants(&mut source, message, runtime);
    Ok(Module { source, depth })
}

/// Adds the structs `fields` define or name, depth first, each once; returns the nesting
/// depth below and including the struct that has `fields`.
fn collect<'a>(
    registry: &StructRegistry<'a>,
    parent: &str,
    fields: &'a [FieldSpec],
    seen: &mut BTreeSet<&'a str>,
    structs: &mut Vec<Struct<'a>>,
    path: &mut Vec<&'a str>,
) -> Result<usize, String> {
    let mut deepest = 0;
    for field in fields {
        let Some(name) = field.field_type.struct_name() else {
            continue;
        };
        if path.contains(&name) {
            return Err(format!("struct {name} contains itself"));
        }
        let registered = registry
            .find(name)
            .ok_or_else(|| format!("struct {name} is not defined"))?;
        if seen.insert(name) {
            let doc = if registry.common.contains(name) {
                format!("`{name}`, a struct several fields share.")
            } else if field.field_type.is_array() {
                format!("An element of `{parent}`'s `{}` array.", field.name)
            } else {
                format!("The `{}` field of `{parent}`.", field.name)
            };
            structs.push(Struct {
                name,
                fields: registered.fields,
                versions: registered.versions.intersect(registered.parent_versions),
                doc,
            });
        }
        path.push(name);
        let below = collect(registry, name, registered.fields, seen, structs, path)?;
        path.pop();
        deepest = deepest.max(below);
    }
    Ok(deepest + 1)
}

fn message_doc(message: &MessageSpec) -> String {
    let kind = match message.kind {
        MessageKind::Request => "request",
        MessageKind::Response => "response",
        MessageKind::Header => "header",
        _ => "data type",
    };
    let api = message
        .api_key
        .map_or_else(String::new, |key| format!(", API key {key}"));
    format!(
        "The `{}` {kind}{api}: versions {}, flexible versions {}.",
        message.name, message.valid_versions, message.flexible_versions
    )
}

/// Whether a field is `Option<T>`: nullable in any of its versions, or a `records`
/// field, which Kafka always defaults to null.
fn is_nullable(field: &FieldSpec) -> bool {
    field.field_type == FieldType::Records
        || !field.nullable_versions.intersect(field.versions).is_empty()
}

/// The Rust type of a value of `field_type`, without `Option`.
fn value_type(field_type: &FieldType, runtime: &str) -> String {
    match field_type {
        FieldType::Bool => "bool".to_owned(),
        FieldType::Int8 => "i8".to_owned(),
        FieldType::Int16 => "i16".to_owned(),
        FieldType::Uint16 => "u16".to_owned(),
        FieldType::Int32 => "i32".to_owned(),
        FieldType::Uint32 => "u32".to_owned(),
        FieldType::Int64 => "i64".to_owned(),
        FieldType::Float64 => "f64".to_owned(),
        FieldType::Uuid => "alluvium_types::Uuid".to_owned(),
        FieldType::String => "String".to_owned(),
        FieldType::Bytes => "bytes::Bytes".to_owned(),
        FieldType::Records => format!("{runtime}::RecordsField"),
        FieldType::Struct(name) => name.clone(),
        FieldType::Array(element) => format!("Vec<{}>", value_type(element, runtime)),
    }
}

/// The Rust type of `field`.
pub(crate) fn field_type(field: &FieldSpec, runtime: &str) -> String {
    let value = value_type(&field.field_type, runtime);
    if is_nullable(field) {
        format!("Option<{value}>")
    } else {
        value
    }
}

/// `field`'s default as a Rust expression, and whether it equals the Rust type's own
/// `Default`, so that a struct whose fields all do can derive it.
fn default(field: &FieldSpec) -> (String, bool) {
    let nullable = is_nullable(field);
    let wrap = |value: String| {
        if nullable {
            format!("Some({value})")
        } else {
            value
        }
    };
    match (&field.default, &field.field_type) {
        (_, FieldType::Records) | (FieldDefault::Null, _) => ("None".to_owned(), true),
        (FieldDefault::Bool(value), _) => (value.to_string(), !value),
        (FieldDefault::Integer(value), _) => (integer_literal(*value), *value == 0),
        (FieldDefault::Float64(value), _) => {
            (float_literal(*value), value.to_bits() == 0.0_f64.to_bits())
        }
        (FieldDefault::Uuid(value), _) => {
            // `Uuid` has no `Default`, so a struct with a uuid field never derives it.
            let expression = if *value == Uuid::ZERO {
                "alluvium_types::Uuid::ZERO".to_owned()
            } else {
                format!(
                    "alluvium_types::Uuid::from_bits({}, {})",
                    integer_literal(value.most_significant_bits()),
                    integer_literal(value.least_significant_bits())
                )
            };
            (expression, false)
        }
        (FieldDefault::String(text), _) => {
            let value = if text.is_empty() {
                "String::new()".to_owned()
            } else {
                format!("String::from({text:?})")
            };
            (wrap(value), text.is_empty() && !nullable)
        }
        (FieldDefault::Empty, field_type) => {
            let value = match field_type {
                FieldType::Bytes => "bytes::Bytes::new()".to_owned(),
                FieldType::Struct(name) => format!("{name}::default()"),
                _ => "Vec::new()".to_owned(),
            };
            (wrap(value), !nullable)
        }
    }
}

/// `value` as a Rust literal, with `_` between groups of three digits from five digits
/// on, as clippy's `unreadable_literal` asks: `-2_147_483_648`.
fn integer_literal(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let grouped = if digits.len() < 5 {
        digits
    } else {
        let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
        for (index, digit) in digits.chars().enumerate() {
            if index > 0 && (digits.len() - index).is_multiple_of(3) {
                grouped.push('_');
            }
            grouped.push(digit);
        }
        grouped
    };
    if value < 0 {
        format!("-{grouped}")
    } else {
        grouped
    }
}

/// A Rust `f64` expression for `value`, exact for every value.
fn float_literal(value: f64) -> String {
    if value.is_nan() {
        "f64::NAN".to_owned()
    } else if value.is_infinite() {
        if value > 0.0 {
            "f64::INFINITY"
        } else {
            "f64::NEG_INFINITY"
        }
        .to_owned()
    } else {
        // `{:?}` prints the shortest text that reads back as the same value.
        let text = format!("{value:?}");
        if text.contains(['.', 'e']) {
            text
        } else {
            format!("{text}.0")
        }
    }
}

fn render_struct(out: &mut String, item: &Struct<'_>, flexible: bool, runtime: &str) {
    let defaults: Vec<(String, bool)> = item.fields.iter().map(default).collect();
    let derivable = defaults.iter().all(|(_, derivable)| *derivable);
    out.push_str(&docs::wrap(&item.doc, "/// "));
    let derives = if derivable {
        "Debug, Clone, PartialEq, Default"
    } else {
        "Debug, Clone, PartialEq"
    };
    // Writing to a String can't fail; here and below.
    let _ = writeln!(out, "#[derive({derives})]\npub struct {} {{", item.name);
    for field in item.fields {
        let about = if field.about.is_empty() {
            format!("The `{}` field.", field.name)
        } else {
            docs::about(&field.about)
        };
        out.push_str(&docs::wrap(&about, "    /// "));
        let _ = writeln!(
            out,
            "    ///\n    /// {}\n    pub {}: {},",
            docs::field_versions(field),
            names::identifier(&field.name),
            field_type(field, runtime)
        );
    }
    if flexible {
        let _ = writeln!(
            out,
            "    /// Tagged fields this version doesn't define, kept to be written back unchanged.\n    pub {UNKNOWN_TAGGED_FIELDS}: Vec<{runtime}::RawTaggedField>,"
        );
    }
    out.push_str("}\n\n");
    if derivable {
        return;
    }
    let _ = writeln!(
        out,
        "impl Default for {name} {{\n    fn default() -> {name} {{\n        {name} {{",
        name = item.name
    );
    for (field, (expression, _)) in item.fields.iter().zip(&defaults) {
        let _ = writeln!(
            out,
            "            {}: {expression},",
            names::identifier(&field.name)
        );
    }
    if flexible {
        let _ = writeln!(out, "            {UNKNOWN_TAGGED_FIELDS}: Vec::new(),");
    }
    out.push_str("        }\n    }\n}\n\n");
}

/// A `Versions` constant expression.
fn versions(value: Versions, runtime: &str) -> String {
    if value.is_empty() {
        format!("{runtime}::Versions::NONE")
    } else {
        let highest = if value.highest() == i16::MAX {
            "i16::MAX".to_owned()
        } else {
            value.highest().to_string()
        };
        format!("{runtime}::Versions::new({}, {highest})", value.lowest())
    }
}

fn render_constants(out: &mut String, message: &MessageSpec, runtime: &str) {
    let _ = writeln!(out, "impl {} {{", message.name);
    if let Some(key) = message.api_key {
        let _ = writeln!(
            out,
            "    /// The API key.\n    pub const API_KEY: i16 = {key};"
        );
    }
    let _ = writeln!(
        out,
        "    /// The versions of this message.\n    pub const VERSIONS: {runtime}::Versions = {};",
        versions(message.valid_versions, runtime)
    );
    let _ = writeln!(
        out,
        "    /// The versions that use the flexible encoding.\n    pub const FLEXIBLE_VERSIONS: {runtime}::Versions = {};",
        versions(message.flexible_versions, runtime)
    );
    let _ = writeln!(
        out,
        "    /// The versions Kafka has deprecated.\n    pub const DEPRECATED_VERSIONS: {runtime}::Versions = {};",
        versions(message.deprecated_versions, runtime)
    );
    if message.kind == MessageKind::Request {
        let _ = writeln!(
            out,
            "    /// Whether the highest version is still unstable, and served only when\n    /// unstable versions are enabled.\n    pub const LATEST_VERSION_UNSTABLE: bool = {};",
            message.latest_version_unstable
        );
    }
    out.push_str("}\n");
}

#[cfg(test)]
mod tests {
    use super::{MAX_NESTING_DEPTH, float_literal, integer_literal, module};
    use crate::message_spec::{self, MessageSpec};

    fn fixture(json: &str) -> MessageSpec {
        message_spec::load("fixture.json", json).unwrap()
    }

    /// A data message whose struct `Level0` nests `levels - 1` more structs.
    fn nested(levels: usize) -> MessageSpec {
        let mut fields = String::from(r#"{ "name": "Leaf", "type": "int8", "versions": "0+" }"#);
        for level in (1..levels).rev() {
            fields = format!(
                r#"{{ "name": "Level{level}", "type": "Level{level}", "versions": "0+", "fields": [{fields}] }}"#
            );
        }
        fixture(&format!(
            r#"{{ "type": "data", "name": "Level0", "validVersions": "0", "flexibleVersions": "none", "fields": [{fields}] }}"#
        ))
    }

    #[test]
    fn every_vendored_schema_renders_without_name_collisions() {
        let root = crate::repo_root().join("schemas/kafka-4.4.0");
        let mut rendered = 0;
        for directory in std::fs::read_dir(root).unwrap() {
            let directory = directory.unwrap().path();
            if !directory.is_dir() {
                continue;
            }
            for file in std::fs::read_dir(&directory).unwrap() {
                let path = file.unwrap().path();
                if path
                    .extension()
                    .is_some_and(|extension| extension == "json")
                {
                    let text = std::fs::read_to_string(&path).unwrap();
                    let message = message_spec::load(&path.display().to_string(), &text).unwrap();
                    module(&message, "crate")
                        .unwrap_or_else(|problem| panic!("{}: {problem}", path.display()));
                    rendered += 1;
                }
            }
        }
        assert_eq!(rendered, 281);
    }

    #[test]
    fn a_common_struct_is_emitted_once_for_every_field_naming_it() {
        let path =
            crate::repo_root().join("schemas/kafka-4.4.0/clients/DescribeQuorumResponse.json");
        let message = message_spec::load(
            "DescribeQuorumResponse.json",
            &std::fs::read_to_string(path).unwrap(),
        )
        .unwrap();
        let source = module(&message, "crate").unwrap().source;
        assert_eq!(source.matches("pub struct ReplicaState {").count(), 1);
        assert!(source.contains("pub current_voters: Vec<ReplicaState>,"));
        assert!(source.contains("pub observers: Vec<ReplicaState>,"));
    }

    #[test]
    fn nesting_deeper_than_the_limit_fails_and_the_limit_itself_is_allowed() {
        assert_eq!(
            module(&nested(MAX_NESTING_DEPTH), "crate").unwrap().depth,
            MAX_NESTING_DEPTH
        );
        let problem = module(&nested(MAX_NESTING_DEPTH + 1), "crate")
            .err()
            .unwrap();
        assert!(problem.contains("nests structs 9 levels deep"), "{problem}");
    }

    #[test]
    fn only_structs_on_the_wire_in_a_flexible_version_keep_unknown_tagged_fields() {
        let message = fixture(
            r#"{ "type": "data", "name": "Outer", "validVersions": "0-2", "flexibleVersions": "2+", "fields": [
                { "name": "Old", "type": "Old", "versions": "0-1", "fields": [
                    { "name": "Value", "type": "int8", "versions": "0+" } ] },
                { "name": "New", "type": "New", "versions": "2+", "fields": [
                    { "name": "Value", "type": "int8", "versions": "0+" } ] } ] }"#,
        );
        let source = module(&message, "crate").unwrap().source;
        let body = |name: &str| {
            let (_, after) = source.split_once(&format!("pub struct {name} {{")).unwrap();
            after.split_once("\n}").unwrap().0.to_owned()
        };
        assert!(body("Outer").contains("unknown_tagged_fields"));
        assert!(body("New").contains("unknown_tagged_fields"));
        assert!(!body("Old").contains("unknown_tagged_fields"));
    }

    #[test]
    fn long_integer_literals_are_grouped() {
        assert_eq!(integer_literal(1000), "1000");
        assert_eq!(integer_literal(32_767), "32_767");
        assert_eq!(integer_literal(i64::from(i32::MIN)), "-2_147_483_648");
        assert_eq!(integer_literal(i64::MIN), "-9_223_372_036_854_775_808");
    }

    #[test]
    fn float_literals_read_back_exactly() {
        for value in [0.0, -0.0, 1.0, 0.1, 1e-300, 123_456.75, f64::MAX] {
            let text = float_literal(value);
            assert_eq!(
                text.parse::<f64>().unwrap().to_bits(),
                value.to_bits(),
                "{text}"
            );
        }
        assert_eq!(float_literal(1.0), "1.0");
        assert_eq!(float_literal(f64::NAN), "f64::NAN");
    }
}
