//! Test-only proptest strategies for each generated struct, and a round-trip test for
//! each message (PROTO-0081, PROTO-0082).
//!
//! `arbitrary(version)` generates values a message can carry at `version`:
//! - fields absent from the version hold their defaults;
//! - a field is `None` only in versions where it is nullable;
//! - in flexible versions, unknown tagged fields use tags above every tag the struct
//!   defines, so they can't collide with known ones.
//!
//! Each field's strategy calls the helpers in the protocol crate's `test_strategies`,
//! which build on TEST-F16's shared strategies (`alluvium_types::strategies`) and favor
//! boundary values.

use std::fmt::Write as _;

use super::decode::{Condition, condition, uses};
use super::names::{self, UNKNOWN_TAGGED_FIELDS};
use super::structs;
use crate::message_spec::{FieldSpec, FieldType, MessageSpec, Versions};

/// The most fields one tuple strategy combines; larger structs nest tuples.
const TUPLE_SIZE: usize = 10;

/// The helpers in the protocol crate's `test_strategies` that generated strategies call.
const HELPERS: &[&str] = &[
    "nullable",
    "present",
    "boundary_i8",
    "boundary_i16",
    "boundary_i32",
    "boundary_i64",
    "boundary_u16",
    "boundary_u32",
    "uuid",
    "float64",
    "string",
    "bytes",
    "records",
    "array",
    "unknown_tagged_fields",
];

/// A strategy for a non-null value of `field_type`, from the helpers in the protocol
/// crate's `test_strategies` and TEST-F16's shared strategies it re-exports.
fn value(field_type: &FieldType) -> String {
    match field_type {
        FieldType::Bool => "any::<bool>()".to_owned(),
        FieldType::Int8 => "boundary_i8()".to_owned(),
        FieldType::Int16 => "boundary_i16()".to_owned(),
        FieldType::Uint16 => "boundary_u16()".to_owned(),
        FieldType::Int32 => "boundary_i32()".to_owned(),
        FieldType::Uint32 => "boundary_u32()".to_owned(),
        FieldType::Int64 => "boundary_i64()".to_owned(),
        FieldType::Float64 => "float64()".to_owned(),
        FieldType::Uuid => "uuid()".to_owned(),
        FieldType::String => "string()".to_owned(),
        FieldType::Bytes => "bytes()".to_owned(),
        FieldType::Records => "records()".to_owned(),
        FieldType::Struct(name) => format!("{name}::arbitrary(version)"),
        FieldType::Array(element) => format!("array({})", value(element)),
    }
}

/// Whether `code` calls the function `name`.
fn calls(code: &str, name: &str) -> bool {
    code.match_indices(&format!("{name}(")).any(|(at, _)| {
        code.get(..at)
            .and_then(|before| before.chars().next_back())
            .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '.'))
    })
}

/// A strategy for `field` in the struct's `versions`, as one expression.
fn field_strategy(field: &FieldSpec, versions: Versions) -> String {
    let present_versions = field.versions.intersect(versions);
    let mut strategy = value(&field.field_type);
    if structs::is_nullable(field) {
        strategy = match condition(field.nullable_versions, present_versions) {
            Condition::Always => format!("nullable(true, {strategy})"),
            Condition::Never => format!("nullable(false, {strategy})"),
            Condition::When(test) => format!("nullable({test}, {strategy})"),
        };
    }
    let (default, _) = structs::default(field);
    match condition(field.versions, versions) {
        Condition::Always => strategy,
        Condition::Never => format!("Just({default})"),
        Condition::When(test) => format!("present({test}, {strategy}, {default})"),
    }
}

/// A strategy for unknown tagged fields, with tags from `first_tag`, in the struct's
/// flexible versions.
fn unknown_strategy(flexible: &Condition, first_tag: u32) -> String {
    match flexible {
        Condition::Always => format!("unknown_tagged_fields({first_tag})"),
        Condition::Never => "Just(Vec::new())".to_owned(),
        Condition::When(test) => {
            format!("present({test}, unknown_tagged_fields({first_tag}), Vec::new())")
        }
    }
}

/// `arbitrary(version)` for the struct `name`, whose `fields` are on the wire in
/// `versions`.
pub(crate) fn render_arbitrary(
    out: &mut String,
    name: &str,
    fields: &[FieldSpec],
    versions: Versions,
    message: &MessageSpec,
) {
    let mut strategies: Vec<String> = fields
        .iter()
        .map(|field| field_strategy(field, versions))
        .collect();
    let mut labels: Vec<String> = fields
        .iter()
        .map(|field| names::identifier(&field.name))
        .collect();
    if !condition(message.flexible_versions, versions).is_never() {
        let first_tag = fields
            .iter()
            .filter_map(|field| field.tag)
            .max()
            .and_then(|tag| u32::try_from(tag).ok())
            .map_or(0, |tag| tag.saturating_add(1));
        strategies.push(unknown_strategy(
            &condition(message.flexible_versions, versions),
            first_tag,
        ));
        labels.push(UNKNOWN_TAGGED_FIELDS.to_owned());
    }
    let chunks: Vec<&[String]> = strategies.chunks(TUPLE_SIZE).collect();
    let tuple = chunks
        .iter()
        .map(|chunk| {
            let boxed: Vec<String> = chunk
                .iter()
                .map(|strategy| format!("{strategy}.boxed()"))
                .collect();
            format!("({},)", boxed.join(", "))
        })
        .collect::<Vec<_>>()
        .join(", ");
    let mut index = 0;
    let pattern = chunks
        .iter()
        .map(|chunk| {
            let names: Vec<String> = chunk
                .iter()
                .map(|_| {
                    let name = format!("value_{index}");
                    index += 1;
                    name
                })
                .collect();
            format!("({},)", names.join(", "))
        })
        .collect::<Vec<_>>()
        .join(", ");
    let literal = labels
        .iter()
        .enumerate()
        .fold(String::new(), |mut literal, (index, label)| {
            let _ = write!(literal, "{label}: value_{index}, ");
            literal
        });
    let version = if strategies.iter().any(|strategy| uses(strategy, "version")) {
        "version"
    } else {
        "_version"
    };
    let body = if strategies.is_empty() {
        format!("Just({name} {{}}).boxed()")
    } else {
        format!("({tuple},).prop_map(|({pattern},)| {name} {{ {literal} }}).boxed()")
    };
    let mut imports = String::from("        use proptest::prelude::*;\n");
    let helpers: Vec<&str> = HELPERS
        .iter()
        .copied()
        .filter(|helper| calls(&body, helper))
        .collect();
    if !helpers.is_empty() {
        let _ = writeln!(
            imports,
            "        use crate::test_strategies::{{{}}};",
            helpers.join(", ")
        );
    }
    let _ = writeln!(
        out,
        "#[cfg(test)]\nimpl {name} {{\n    /// Values of `{name}` at `version`, for property tests: fields the version \
         doesn't\n    /// have hold their defaults.\n    pub(crate) fn arbitrary({version}: i16) -> \
         proptest::strategy::BoxedStrategy<{name}> {{\n{imports}        {body}\n    }}\n}}\n"
    );
}

/// The round-trip property test of `message`, at each of its versions.
pub(crate) fn render_round_trip_test(out: &mut String, message: &MessageSpec) {
    let name = &message.name;
    let _ = writeln!(
        out,
        "#[cfg(test)]\n#[test]\nfn round_trips_at_every_version() {{\n    \
         crate::encode_tests::round_trip_arbitrary({name}::arbitrary, {name}::VERSIONS);\n}}\n"
    );
}

impl Condition {
    /// Whether this is `Condition::Never`.
    pub(super) fn is_never(&self) -> bool {
        matches!(self, Condition::Never)
    }
}
