//! One message's Rust module: its struct, every struct its fields define or share, each
//! struct's `Default`, the message's version constants, and what the overlays add:
//! `find` and `find_all` for keyed arrays, typed accessors for known-value fields, and a
//! `Debug` that hides sensitive fields.
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
use super::overlays::{self, Overlays};
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
    /// Whether it has `mapKey` fields, which make arrays of it keyed.
    keyed: bool,
}

/// Renders `message`'s module. `runtime` is the path of the crate the generated code
/// uses for `RawTaggedField`, `RecordsField` and `Versions`.
pub(crate) fn module(
    message: &MessageSpec,
    runtime: &str,
    overlays: &Overlays,
) -> Result<Module, String> {
    let registry = StructRegistry::new(message)
        .map_err(|problem| format!("{} at {}", problem.message, problem.pointer))?;
    let mut structs = vec![Struct {
        name: &message.name,
        fields: &message.fields,
        versions: message.valid_versions,
        doc: message_doc(message),
        keyed: false,
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
        let sensitive: BTreeSet<&str> = item
            .fields
            .iter()
            .filter(|field| overlays.is_sensitive(&message.name, &field.name))
            .map(|field| field.name.as_str())
            .collect();
        render_struct(&mut source, item, flexible, runtime, &sensitive);
        if !sensitive.is_empty() {
            render_debug(&mut source, item, flexible, &sensitive);
        }
        if item.keyed {
            render_find(&mut source, item)?;
        }
        render_known_values(&mut source, item, overlays, runtime);
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
                keyed: registered.has_keys(),
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

/// The struct, and its `Default` impl unless it can derive one. A struct with
/// `sensitive` fields doesn't derive `Debug`; `render_debug` writes one that hides them.
fn render_struct(
    out: &mut String,
    item: &Struct<'_>,
    flexible: bool,
    runtime: &str,
    sensitive: &BTreeSet<&str>,
) {
    let defaults: Vec<(String, bool)> = item.fields.iter().map(default).collect();
    let derivable = defaults.iter().all(|(_, derivable)| *derivable);
    out.push_str(&docs::wrap(&item.doc, "/// "));
    let debug = if sensitive.is_empty() { "Debug, " } else { "" };
    let derives = if derivable {
        format!("{debug}Clone, PartialEq, Default")
    } else {
        format!("{debug}Clone, PartialEq")
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

/// A `Debug` that prints `***` for the `sensitive` fields and everything else as derived
/// `Debug` would.
fn render_debug(out: &mut String, item: &Struct<'_>, flexible: bool, sensitive: &BTreeSet<&str>) {
    let _ = writeln!(
        out,
        "impl std::fmt::Debug for {name} {{\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) \
         -> std::fmt::Result {{\n        f.debug_struct({name:?})",
        name = item.name
    );
    for field in item.fields {
        let name = names::identifier(&field.name);
        if sensitive.contains(field.name.as_str()) {
            let _ = writeln!(out, "            .field({name:?}, &format_args!(\"***\"))");
        } else {
            let _ = writeln!(out, "            .field({name:?}, &self.{name})");
        }
    }
    if flexible {
        let _ = writeln!(
            out,
            "            .field({UNKNOWN_TAGGED_FIELDS:?}, &self.{UNKNOWN_TAGGED_FIELDS})"
        );
    }
    out.push_str("            .finish()\n    }\n}\n\n");
}

/// A typed accessor, named like the field, for each field an overlay gives known values.
fn render_known_values(out: &mut String, item: &Struct<'_>, overlays: &Overlays, runtime: &str) {
    let accessors: Vec<String> = item
        .fields
        .iter()
        .filter_map(|field| {
            let known = overlays.known_value(field)?;
            let name = names::identifier(&field.name);
            let rust = overlays::rust_path(&known.rust, runtime);
            let error = overlays::rust_path(&known.error, runtime);
            let convert = overlays::rust_path(&known.convert, runtime);
            let summary = docs::wrap(
                &format!(
                    "`{name}` as a [`{rust}`]. The field keeps the raw value, so a value the \
                     enum doesn't know still round-trips."
                ),
                "    /// ",
            );
            Some(format!(
                "{summary}    ///\n    /// # Errors\n    ///\n    /// [`{error}`] if the value \
                 is not one the enum defines.\n    pub fn {name}(&self) -> Result<{rust}, {error}> \
                 {{\n        {convert}(self.{name})\n    }}\n"
            ))
        })
        .collect();
    if !accessors.is_empty() {
        let _ = writeln!(out, "impl {} {{\n{}}}\n", item.name, accessors.join("\n"));
    }
}

/// `find` and `find_all` for a struct with `mapKey` fields: Kafka generates them on the
/// collection class of a keyed array, taking the key fields in declaration order.
/// Generated arrays are plain `Vec`s in wire order, so these are associated functions
/// over a slice, and they keep duplicate keys, as Kafka's multi-collection does.
// Parity: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java@4.4.0 MessageDataGenerator.generateHashSetFindMethod, MessageDataGenerator.generateHashSetFindAllMethod
fn render_find(out: &mut String, item: &Struct<'_>) -> Result<(), String> {
    let mut parameters = Vec::new();
    let mut tests = Vec::new();
    let mut names = Vec::new();
    for field in item.fields.iter().filter(|field| field.map_key) {
        let name = names::identifier(&field.name);
        if name == "elements" {
            return Err(format!(
                "in struct {}: the key field {} clashes with `find`'s `elements` parameter",
                item.name, field.name
            ));
        }
        let nullable = is_nullable(field);
        let parameter = match (&field.field_type, nullable) {
            (FieldType::String, false) => "&str".to_owned(),
            (FieldType::String, true) => "Option<&str>".to_owned(),
            (
                field_type @ (FieldType::Bool
                | FieldType::Int8
                | FieldType::Int16
                | FieldType::Uint16
                | FieldType::Int32
                | FieldType::Uint32
                | FieldType::Int64
                | FieldType::Uuid),
                false,
            ) => value_type(field_type, ""),
            _ => {
                return Err(format!(
                    "in struct {}: the key field {} has a type the generator can't compare",
                    item.name, field.name
                ));
            }
        };
        let test = if nullable {
            format!("element.{name}.as_deref() == {name}")
        } else {
            format!("element.{name} == {name}")
        };
        parameters.push(format!("{name}: {parameter}"));
        tests.push(test);
        names.push(format!("`{}`", field.name));
    }
    let key = names.join(", ");
    // A borrowed key parameter needs the result's lifetime named; otherwise it elides.
    let (generic, lifetime) = if parameters.iter().any(|parameter| parameter.contains('&')) {
        ("<'a>", "'a ")
    } else {
        ("", "")
    };
    let parameters = parameters.join(", ");
    let test = tests.join(" && ");
    let find_doc = docs::wrap(
        &format!("The first of `elements`, in wire order, whose key ({key}) equals the arguments."),
        "    /// ",
    );
    let find_all_doc = docs::wrap(
        &format!(
            "Every element of `elements` whose key ({key}) equals the arguments, in wire order."
        ),
        "    /// ",
    );
    let _ = writeln!(
        out,
        "impl {name} {{\n{find_doc}    pub fn find{generic}(elements: &{lifetime}[{name}], \
         {parameters}) -> Option<&{lifetime}{name}> {{\n        elements.iter().find(|element| \
         {test})\n    }}\n\n{find_all_doc}    pub fn find_all{generic}(elements: \
         &{lifetime}[{name}], {parameters}) -> impl Iterator<Item = &{lifetime}{name}> {{\n        \
         elements.iter().filter(move |element| {test})\n    }}\n}}\n",
        name = item.name
    );
    Ok(())
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
    use super::{MAX_NESTING_DEPTH, Overlays, float_literal, integer_literal, module};
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
                    module(&message, "crate", &Overlays::default())
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
        let source = module(&message, "crate", &Overlays::default())
            .unwrap()
            .source;
        assert_eq!(source.matches("pub struct ReplicaState {").count(), 1);
        assert!(source.contains("pub current_voters: Vec<ReplicaState>,"));
        assert!(source.contains("pub observers: Vec<ReplicaState>,"));
    }

    #[test]
    fn nesting_deeper_than_the_limit_fails_and_the_limit_itself_is_allowed() {
        assert_eq!(
            module(&nested(MAX_NESTING_DEPTH), "crate", &Overlays::default())
                .unwrap()
                .depth,
            MAX_NESTING_DEPTH
        );
        let problem = module(
            &nested(MAX_NESTING_DEPTH + 1),
            "crate",
            &Overlays::default(),
        )
        .err()
        .unwrap();
        assert!(problem.contains("nests structs 9 levels deep"), "{problem}");
    }

    #[test]
    fn a_key_the_generator_cannot_compare_fails() {
        let message = fixture(
            r#"{ "type": "data", "name": "Outer", "validVersions": "0", "flexibleVersions": "none", "fields": [
                { "name": "Items", "type": "[]Item", "versions": "0+", "fields": [
                    { "name": "Payload", "type": "bytes", "versions": "0+", "mapKey": true } ] } ] }"#,
        );
        let problem = module(&message, "crate", &Overlays::default())
            .err()
            .unwrap();
        assert!(problem.contains("key field Payload"), "{problem}");
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
        let source = module(&message, "crate", &Overlays::default())
            .unwrap()
            .source;
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
