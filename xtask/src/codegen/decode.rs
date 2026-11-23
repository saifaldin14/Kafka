//! Each generated struct's decoder: a private `read(reader, version)` that starts from
//! the struct's `Default` and has `read_fields` assign the fields present in `version`,
//! and `Message::decode` for the message itself.
//!
//! It mirrors Kafka's `MessageDataGenerator.generateClassReader`, whose `read` assigns
//! each field present in the version over its default, in schema order. The versions a field is present, nullable, flexible or tagged in
//! become conditions on `version`, written only where they vary within the struct's own
//! versions, so a field present in every version reads unconditionally. In flexible
//! versions a tagged-field section follows: known tags are read from exactly their
//! declared bytes, and unknown ones are kept raw.

use std::fmt::Write as _;

use super::structs;
use crate::message_spec::{FieldSpec, FieldType, MessageSpec, Versions};

/// Whether something happens in a version, as Rust code over `version`.
pub(super) enum Condition {
    Always,
    Never,
    When(String),
}

/// Whether a version of `possible` is in `versions`, as the simplest test that tells
/// them apart.
pub(super) fn condition(versions: Versions, possible: Versions) -> Condition {
    let inside = versions.intersect(possible);
    if inside.is_empty() {
        return Condition::Never;
    }
    if inside == possible {
        return Condition::Always;
    }
    let (lowest, highest) = (inside.lowest(), inside.highest());
    if lowest == highest {
        return Condition::When(format!("version == {lowest}"));
    }
    match (lowest > possible.lowest(), highest < possible.highest()) {
        (true, false) => Condition::When(format!("version >= {lowest}")),
        (false, true) => Condition::When(format!("version <= {highest}")),
        _ => Condition::When(format!("({lowest}..={highest}).contains(&version)")),
    }
}

/// `then` when the condition holds and `otherwise` when it doesn't.
pub(super) fn choose(condition: &Condition, then: &str, otherwise: &str) -> String {
    match condition {
        Condition::Always => then.to_owned(),
        Condition::Never => otherwise.to_owned(),
        Condition::When(test) => format!("if {test} {{ {then} }} else {{ {otherwise} }}"),
    }
}

/// What the generated code needs to know about the struct being read.
struct Context<'a> {
    runtime: &'a str,
    message: &'a MessageSpec,
}

/// The framing of `field` in the `present` versions: compact where it is flexible.
pub(super) fn framing(
    runtime: &str,
    message: &MessageSpec,
    field: &FieldSpec,
    present: Versions,
) -> String {
    choose(
        &condition(field.flexible_versions(message.flexible_versions), present),
        &format!("{runtime}::Framing::Compact"),
        &format!("{runtime}::Framing::Classic"),
    )
}

impl Context<'_> {
    /// A function or closure that reads one value of `field_type` from `reader`, for
    /// `read_array` and `read_tagged_value`.
    fn reader_function(&self, field_type: &FieldType, framing: &str) -> String {
        let runtime = self.runtime;
        match fixed_read(field_type) {
            Some(method) => format!("{runtime}::Reader::{method}"),
            None => format!("|reader| {}", self.read(field_type, framing)),
        }
    }

    /// An expression reading a non-null value of `field_type` from `reader`, as a
    /// `Result`.
    fn read(&self, field_type: &FieldType, framing: &str) -> String {
        let runtime = self.runtime;
        if let Some(method) = fixed_read(field_type) {
            return format!("reader.{method}()");
        }
        match field_type {
            FieldType::String => format!("reader.read_string({framing})"),
            FieldType::Bytes => format!("reader.read_bytes({framing})"),
            FieldType::Records => {
                format!("reader.read_bytes({framing}).map({runtime}::RecordsField::new)")
            }
            FieldType::Struct(name) => format!("{name}::read(reader, version)"),
            FieldType::Array(element) => format!(
                "reader.read_array({framing}, {})",
                self.reader_function(element, framing)
            ),
            _ => unreachable!("fixed_read covers every other type"),
        }
    }

    /// An expression reading a value of `field_type` that may be null, as a
    /// `Result<Option<_>>`. A struct has a marker before it: an `int8` that is negative
    /// for null, or in a tagged field an unsigned varint that is zero for null.
    // Parity: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java@4.4.0 MessageDataGenerator.generateStructReader
    fn read_nullable(&self, field_type: &FieldType, framing: &str, tagged: bool) -> String {
        match field_type {
            FieldType::String => format!("reader.read_nullable_string({framing})"),
            FieldType::Bytes => format!("reader.read_nullable_bytes({framing})"),
            FieldType::Records => format!("reader.read_records({framing})"),
            FieldType::Array(element) => format!(
                "reader.read_nullable_array({framing}, {})",
                self.reader_function(element, framing)
            ),
            FieldType::Struct(name) => {
                let (marker, null) = if tagged {
                    ("reader.read_unsigned_varint()", "marker.cast_signed() <= 0")
                } else {
                    ("reader.read_i8()", "marker < 0")
                };
                format!(
                    "{marker}.and_then(|marker| if {null} {{ Ok(None) }} else {{ \
                     {name}::read(reader, version).map(Some) }})"
                )
            }
            _ => unreachable!("only variable-length types and structs can be null"),
        }
    }

    /// An expression reading `field` in the `present` versions, as a `Result` of the
    /// field's Rust type: an `Option` when the field is nullable in some version, with
    /// a null rejected in the versions where it isn't.
    // Parity: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java@4.4.0 MessageDataGenerator.generateVariableLengthReader
    fn read_field(&self, field: &FieldSpec, present: Versions, tagged: bool) -> String {
        let framing = framing(self.runtime, self.message, field, present);
        let value = self.read(&field.field_type, &framing);
        if !structs::is_nullable(field) {
            return value;
        }
        choose(
            &condition(field.nullable_versions, present),
            &self.read_nullable(&field.field_type, &framing, tagged),
            &format!("{value}.map(Some)"),
        )
    }
}

/// The `Reader` method that reads a fixed-size `field_type`, if it is one.
fn fixed_read(field_type: &FieldType) -> Option<&'static str> {
    Some(match field_type {
        FieldType::Bool => "read_bool",
        FieldType::Int8 => "read_i8",
        FieldType::Int16 => "read_i16",
        FieldType::Uint16 => "read_u16",
        FieldType::Int32 => "read_i32",
        FieldType::Uint32 => "read_u32",
        FieldType::Int64 => "read_i64",
        FieldType::Float64 => "read_f64",
        FieldType::Uuid => "read_uuid",
        _ => return None,
    })
}

/// `expression`, a `Result`, unwrapped with `?` after naming `field` in its error.
fn within(expression: &str, field: &FieldSpec) -> String {
    format!("{expression}.within({:?})?", field.name)
}

/// The private `read` of struct `name`, whose `fields` are on the wire in `versions`.
// Parity: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java@4.4.0 MessageDataGenerator.generateClassReader
pub(crate) fn render_read(
    out: &mut String,
    name: &str,
    fields: &[FieldSpec],
    versions: Versions,
    message: &MessageSpec,
    runtime: &str,
) -> Result<(), String> {
    let context = Context { runtime, message };
    let flexible = condition(message.flexible_versions, versions);
    // Each read, under the condition of the versions it is present in. Adjacent reads
    // under the same condition share one block.
    let mut groups: Vec<(Option<String>, String)> = Vec::new();
    let mut tags = String::new();
    for field in fields {
        let identifier = super::names::identifier(&field.name);
        let mandatory = field
            .versions
            .subtract(field.tagged_versions)
            .ok_or_else(|| {
                format!(
                    "in struct {name}: field {} has tagged versions {} inside its versions {}",
                    field.name, field.tagged_versions, field.versions
                )
            })?;
        let mandatory = mandatory.intersect(versions);
        let read = format!(
            "self.{identifier} = {};",
            within(&context.read_field(field, mandatory, false), field)
        );
        match (condition(mandatory, versions), groups.last_mut()) {
            (Condition::Never, _) => {}
            (Condition::When(test), Some((Some(last), block))) if *last == test => {
                block.push('\n');
                block.push_str(&read);
            }
            (Condition::When(test), _) => groups.push((Some(test), read)),
            (Condition::Always, _) => groups.push((None, read)),
        }
        render_tag(&mut tags, &context, field, versions)?;
    }
    let mut body = groups
        .iter()
        .fold(String::new(), |mut body, (test, block)| {
            let _ = match test {
                Some(test) => writeln!(body, "if {test} {{\n{block}\n}}"),
                None => writeln!(body, "{block}"),
            };
            body
        });
    if !matches!(flexible, Condition::Never) {
        let unknown = format!(
            "reader.read_unknown_tagged_field(tag, size, &mut self.{})?;",
            super::names::UNKNOWN_TAGGED_FIELDS
        );
        let dispatch = if tags.is_empty() {
            unknown
        } else {
            format!("{tags}{{ {unknown} }}")
        };
        let section = format!(
            "let count = reader.read_tagged_field_count()?;\nfor _ in 0..count {{\nlet (tag, \
             size) = reader.read_tagged_field_header()?;\n{dispatch}\n}}"
        );
        if let Condition::When(test) = &flexible {
            let _ = writeln!(body, "if {test} {{\n{section}\n}}");
        } else {
            let _ = writeln!(body, "{section}");
        }
    }
    let decoder = format!("{runtime}::Reader");
    let error = format!("{runtime}::DecodeError");
    if body.is_empty() {
        let _ = writeln!(
            out,
            "impl {name} {{\n    fn read(_reader: &mut {decoder}, _version: i16) -> \
             Result<{name}, {error}> {{\n        Ok({name}::default())\n    }}\n}}\n"
        );
        return Ok(());
    }
    let reader = if uses(&body, "reader") {
        "reader"
    } else {
        "_reader"
    };
    let version = if uses(&body, "version") {
        "version"
    } else {
        "_version"
    };
    // Like Kafka's `read`, this assigns the fields of a value that starts at its defaults.
    let _ = writeln!(
        out,
        "impl {name} {{\n    fn read(reader: &mut {decoder}, version: i16) -> Result<{name}, \
         {error}> {{\n        let mut value = {name}::default();\n        \
         value.read_fields(reader, version)?;\n        Ok(value)\n    }}\n\n    fn \
         read_fields(&mut self, {reader}: &mut {decoder}, {version}: i16) -> Result<(), \
         {error}> {{\n{body}Ok(())\n    }}\n}}\n"
    );
    Ok(())
}

/// The branch of the tagged-field section that reads `field`'s tag, if it has one: from
/// exactly the tag's declared bytes in the versions where it is tagged, and an error in
/// the struct's other flexible versions, as in Kafka.
fn render_tag(
    tags: &mut String,
    context: &Context<'_>,
    field: &FieldSpec,
    versions: Versions,
) -> Result<(), String> {
    let tagged = field.versions.intersect(field.tagged_versions);
    let Some(tag) = field.tag.filter(|_| !tagged.is_empty()) else {
        return Ok(());
    };
    let identifier = super::names::identifier(&field.name);
    let flexible_versions = versions.intersect(context.message.flexible_versions);
    let valid = tagged.intersect(flexible_versions);
    let read = within(
        &format!(
            "reader.read_tagged_value(tag, size, {})",
            tagged_function(context, field, valid)
        ),
        field,
    );
    let invalid = format!(
        "return Err(reader.tag_not_valid(tag, version).within({:?}));",
        field.name
    );
    match condition(valid, flexible_versions) {
        Condition::Always => {
            writeln!(
                tags,
                "if tag == {tag} {{ self.{identifier} = {read}; }} else "
            )
        }
        Condition::Never => writeln!(tags, "if tag == {tag} {{ {invalid} }} else "),
        Condition::When(test) => writeln!(
            tags,
            "if tag == {tag} && {test} {{ self.{identifier} = {read}; }} else if tag == \
             {tag} {{ {invalid} }} else "
        ),
    }
    .map_err(|error| error.to_string())
}

/// The function `read_tagged_value` calls to read `field` in its `valid` versions.
fn tagged_function(context: &Context<'_>, field: &FieldSpec, valid: Versions) -> String {
    if !structs::is_nullable(field)
        && let Some(method) = fixed_read(&field.field_type)
    {
        return format!("{}::Reader::{method}", context.runtime);
    }
    format!("|reader| {}", context.read_field(field, valid, true))
}

/// Whether `code` uses the variable `name`: a whole identifier that is neither a field
/// access (`.name`) nor a field label (`name:`).
pub(super) fn uses(code: &str, name: &str) -> bool {
    code.match_indices(name).any(|(at, _)| {
        let before = code.get(..at).and_then(|text| text.chars().next_back());
        let rest = code.get(at + name.len()..).unwrap_or_default();
        let part = |c: Option<char>| c.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
        let label = rest.starts_with(':') && !rest.starts_with("::");
        !part(before) && before != Some('.') && !part(rest.chars().next()) && !label
    })
}

/// `Message` for the message struct: the version check Kafka's callers make, then
/// `read`.
pub(crate) fn render_message(out: &mut String, message: &MessageSpec, runtime: &str, encode: &str) {
    let name = &message.name;
    let _ = writeln!(
        out,
        "impl {runtime}::Message for {name} {{\n    fn decode(reader: &mut {runtime}::Reader, \
         version: i16) -> Result<{name}, {runtime}::DecodeError> {{\n        if \
         !{name}::VERSIONS.contains(version) {{\n            return \
         Err(reader.unsupported_version({name:?}, version));\n        }}\n        \
         {name}::read(reader, version)\n    }}\n\n{encode}\n    fn write_json(&self, json: &mut \
         {runtime}::JsonWriter, version: i16) {{\n        self.write_json_fields(json, \
         version);\n    }}\n}}\n"
    );
}

#[cfg(test)]
mod tests {
    use super::{Condition, condition, uses};
    use crate::message_spec::Versions;

    fn range(lowest: i16, highest: i16) -> Versions {
        Versions::new(lowest, highest).unwrap()
    }

    fn text(condition: Condition) -> String {
        match condition {
            Condition::Always => "always".to_owned(),
            Condition::Never => "never".to_owned(),
            Condition::When(test) => test,
        }
    }

    #[test]
    fn conditions_test_only_the_bounds_that_vary() {
        let possible = range(3, 13);
        assert_eq!(text(condition(range(0, i16::MAX), possible)), "always");
        assert_eq!(text(condition(range(14, i16::MAX), possible)), "never");
        assert_eq!(text(condition(Versions::NONE, possible)), "never");
        assert_eq!(
            text(condition(range(9, i16::MAX), possible)),
            "version >= 9"
        );
        assert_eq!(text(condition(range(0, 12), possible)), "version <= 12");
        assert_eq!(
            text(condition(range(5, 7), possible)),
            "(5..=7).contains(&version)"
        );
    }

    #[test]
    fn identifiers_are_found_whole() {
        assert!(uses("reader.read_i8()", "reader"));
        assert!(!uses("value.reader_id", "reader"));
        assert!(!uses("my_version", "version"));
        assert!(uses("if version >= 3", "version"));
        assert!(!uses("Record { version: reader.read_i16()? }", "version"));
        assert!(!uses("value.version", "version"));
        assert!(uses("Header::read(reader, version)", "version"));
    }
}
