//! Each generated struct's encoder: a private `write_fields(&self, writer, version)`
//! that writes the fields present in `version`, and `Message::encode` for the message
//! itself.
//!
//! It mirrors Kafka's `MessageDataGenerator.generateClassWriter`:
//! - **Present fields** are written in schema order, with the framing and nullability of
//!   the version. A `None` in a version where the field can't be null fails as
//!   `NullForNonNullable`.
//! - **Absent fields** must hold their defaults unless the field is `ignorable`; a value
//!   that would be lost fails as `NonDefaultFieldUnsupported`.
//! - **Tagged fields** are written only when they differ from their defaults. They are
//!   interleaved with the unknown ones by `RawTaggedFieldWriter`, so the section is in
//!   ascending tag order. Unknown tagged fields in a version without tagged fields fail
//!   as `TaggedFieldsUnsupported`.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use super::decode::{Condition, condition, framing, uses};
use super::names::{self, UNKNOWN_TAGGED_FIELDS};
use super::structs;
use crate::message_spec::{FieldDefault, FieldSpec, FieldType, MessageSpec, Versions};

/// What the generated code needs to know about the struct being written.
struct Context<'a> {
    runtime: &'a str,
    message: &'a MessageSpec,
    /// The structs whose encoding can fail, so `write_fields` returns a `Result`.
    fallible: &'a BTreeSet<String>,
}

/// The `Writer` method that writes a fixed-size `field_type`, if it is one.
fn fixed_write(field_type: &FieldType) -> Option<&'static str> {
    Some(match field_type {
        FieldType::Bool => "write_bool",
        FieldType::Int8 => "write_i8",
        FieldType::Int16 => "write_i16",
        FieldType::Uint16 => "write_u16",
        FieldType::Int32 => "write_i32",
        FieldType::Uint32 => "write_u32",
        FieldType::Int64 => "write_i64",
        FieldType::Float64 => "write_f64",
        FieldType::Uuid => "write_uuid",
        _ => return None,
    })
}

/// Where a value to write is: a field of `self`, or a reference such as a loop's
/// `value`.
enum Place<'a> {
    Field(&'a str),
    Reference(&'a str),
}

impl Place<'_> {
    /// The value itself, for a `Copy` type.
    fn value(&self) -> String {
        match self {
            Place::Field(name) => format!("self.{name}"),
            Place::Reference(name) => format!("*{name}"),
        }
    }

    /// A reference to the value.
    fn reference(&self) -> String {
        match self {
            Place::Field(name) => format!("&self.{name}"),
            Place::Reference(name) => (*name).to_owned(),
        }
    }

    /// The value as a method receiver.
    fn receiver(&self) -> String {
        match self {
            Place::Field(name) => format!("self.{name}"),
            Place::Reference(name) => (*name).to_owned(),
        }
    }
}

/// Statements writing the non-null value at `place` as `field_type`.
fn write(
    field_type: &FieldType,
    place: &Place<'_>,
    framing: &str,
    zero_copy: bool,
    fallible: &BTreeSet<String>,
) -> String {
    if let Some(method) = fixed_write(field_type) {
        return format!("writer.{method}({});", place.value());
    }
    let reference = place.reference();
    match field_type {
        FieldType::String => format!("writer.write_string({framing}, {reference})?;"),
        FieldType::Bytes if zero_copy => {
            format!("writer.write_zero_copy_bytes({framing}, {reference})?;")
        }
        FieldType::Bytes => format!("writer.write_bytes({framing}, {reference})?;"),
        FieldType::Records => format!("writer.write_records({framing}, Some({reference}))?;"),
        FieldType::Struct(name) => write_struct(&place.receiver(), name, fallible),
        FieldType::Array(element) => format!(
            "writer.write_array_length({framing}, {}.len())?;\nfor value in {reference} {{ {} }}",
            place.receiver(),
            write(
                element,
                &Place::Reference("value"),
                framing,
                false,
                fallible
            )
        ),
        _ => unreachable!("fixed_write covers every other type"),
    }
}

/// A call writing the struct `name` at `receiver`, with `?` if its encoding can fail.
fn write_struct(receiver: &str, name: &str, fallible: &BTreeSet<String>) -> String {
    if fallible.contains(name) {
        format!("{receiver}.write_fields(writer, version)?;")
    } else {
        format!("{receiver}.write_fields(writer, version);")
    }
}

impl Context<'_> {
    /// Statements writing the `Option` field `name` in a version where it may be null. A
    /// struct has a marker before it: an `int8`, or in a tagged field an unsigned varint,
    /// that is 1 for a value and -1 or 0 for null.
    // Parity: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java@4.4.0 MessageDataGenerator.generateVariableLengthWriter
    fn write_nullable(
        &self,
        field_type: &FieldType,
        name: &str,
        framing: &str,
        zero_copy: bool,
        tagged: bool,
    ) -> String {
        match field_type {
            FieldType::String => {
                format!("writer.write_nullable_string({framing}, self.{name}.as_deref())?;")
            }
            FieldType::Bytes if zero_copy => {
                format!("writer.write_nullable_zero_copy_bytes({framing}, self.{name}.as_ref())?;")
            }
            FieldType::Bytes => {
                format!("writer.write_nullable_bytes({framing}, self.{name}.as_deref())?;")
            }
            FieldType::Records => {
                format!("writer.write_records({framing}, self.{name}.as_ref())?;")
            }
            FieldType::Array(element) => format!(
                "writer.write_nullable_array_length({framing}, self.{name}.as_ref().map(Vec::len))?;\n\
                 if let Some(values) = &self.{name} {{ for value in values {{ {} }} }}",
                write(
                    element,
                    &Place::Reference("value"),
                    framing,
                    false,
                    self.fallible
                )
            ),
            FieldType::Struct(struct_name) => {
                let (null, present) = if tagged {
                    (
                        "writer.write_unsigned_varint(0)",
                        "writer.write_unsigned_varint(1)",
                    )
                } else {
                    ("writer.write_i8(-1)", "writer.write_i8(1)")
                };
                format!(
                    "match &self.{name} {{ None => {null}, Some(value) => {{ {present}; {} }} }}",
                    write_struct("value", struct_name, self.fallible)
                )
            }
            _ => unreachable!("only variable-length types and structs can be null"),
        }
    }

    /// Statements writing `field` in the `present` versions, from `self`.
    fn write_field(&self, field: &FieldSpec, present: Versions, tagged: bool) -> String {
        let runtime = self.runtime;
        let framing = framing(runtime, self.message, field, present);
        let name = names::identifier(&field.name);
        let zero_copy = field.zero_copy;
        if !structs::is_nullable(field) {
            return write(
                &field.field_type,
                &Place::Field(&name),
                &framing,
                zero_copy,
                self.fallible,
            );
        }
        let nullable = self.write_nullable(&field.field_type, &name, &framing, zero_copy, tagged);
        let present_value = format!(
            "let value = self.{name}.as_ref().ok_or({runtime}::EncodeError::NullForNonNullable \
             {{ field: {:?} }})?;\n{}",
            field.name,
            write(
                &field.field_type,
                &Place::Reference("value"),
                &framing,
                zero_copy,
                self.fallible
            )
        );
        match condition(field.nullable_versions, present) {
            Condition::Always => nullable,
            Condition::Never => format!("{{ {present_value} }}"),
            Condition::When(test) => {
                format!("if {test} {{ {nullable} }} else {{ {present_value} }}")
            }
        }
    }
}

/// An expression that is true when `field` holds something other than its default, as
/// Kafka's `generateNonDefaultValueCheck`: a null counts as non-default unless the
/// default is null.
// Parity: generator/src/main/java/org/apache/kafka/message/FieldSpec.java@4.4.0 FieldSpec.generateNonDefaultValueCheck
fn non_default(field: &FieldSpec) -> String {
    let value = format!("self.{}", names::identifier(&field.name));
    let (default, _) = structs::default(field);
    if field.field_type == FieldType::Records || field.default == FieldDefault::Null {
        return format!("{value}.is_some()");
    }
    let empty_check = |optional: bool| {
        if optional {
            format!("{value}.as_ref().is_none_or(|value| !value.is_empty())")
        } else {
            format!("!{value}.is_empty()")
        }
    };
    let optional = structs::is_nullable(field);
    match (&field.field_type, &field.default) {
        (FieldType::String, FieldDefault::String(text)) if optional => {
            format!("{value}.as_deref() != Some({text:?})")
        }
        (FieldType::String, FieldDefault::String(text)) if text.is_empty() => empty_check(false),
        (FieldType::String, FieldDefault::String(text)) => format!("{value} != {text:?}"),
        (FieldType::Bytes | FieldType::Array(_), _) => empty_check(optional),
        (FieldType::Bool, FieldDefault::Bool(true)) => format!("!{value}"),
        (FieldType::Bool, _) => value,
        (FieldType::Float64, _) => {
            format!("{value}.partial_cmp(&{default}) != Some(std::cmp::Ordering::Equal)")
        }
        _ => format!("{value} != {default}"),
    }
}

/// The versions of `possible` that are not in `versions`, as a condition.
fn outside(versions: Versions, possible: Versions) -> Condition {
    let inside = versions.intersect(possible);
    if inside.is_empty() {
        return Condition::Always;
    }
    if inside == possible {
        return Condition::Never;
    }
    let (lowest, highest) = (inside.lowest(), inside.highest());
    match (lowest > possible.lowest(), highest < possible.highest()) {
        (true, false) => Condition::When(format!("version < {lowest}")),
        (false, true) => Condition::When(format!("version > {highest}")),
        _ => Condition::When(format!("!({lowest}..={highest}).contains(&version)")),
    }
}

/// Wraps `statements` in `if test { ... }`, or not, or drops them, by `condition`.
fn guarded(condition: &Condition, statements: &str) -> String {
    match condition {
        Condition::Always => format!("{statements}\n"),
        Condition::Never => String::new(),
        Condition::When(test) => format!("if {test} {{ {statements} }}\n"),
    }
}

/// One struct to write.
pub(crate) struct Struct<'a> {
    pub(crate) name: &'a str,
    pub(crate) fields: &'a [FieldSpec],
    /// The versions in which the struct is on the wire.
    pub(crate) versions: Versions,
}

/// The structs of a message whose encoding can fail, so whose `write_fields` returns a
/// `Result`: those with a variable-length, nullable or tagged field, a field that must
/// hold its default in some version, or a field of a struct that can fail. The others
/// write only fixed-size fields, which can't.
pub(crate) fn fallible_structs(structs: &[Struct<'_>], message: &MessageSpec) -> BTreeSet<String> {
    let mut fallible = BTreeSet::new();
    loop {
        let before = fallible.len();
        for item in structs {
            let can_fail = !matches!(
                condition(message.flexible_versions, item.versions),
                Condition::Never
            ) || item.fields.iter().any(|field| {
                let absent = !matches!(outside(field.versions, item.versions), Condition::Never);
                let nested = field
                    .field_type
                    .struct_name()
                    .is_some_and(|name| fallible.contains(name));
                structs::is_nullable(field)
                    || (fixed_write(&field.field_type).is_none() && !field.field_type.is_struct())
                    || (absent && !field.ignorable)
                    || nested
            });
            if can_fail {
                fallible.insert(item.name.to_owned());
            }
        }
        if fallible.len() == before {
            return fallible;
        }
    }
}

/// What `write_fields` does with each field of a struct.
struct Plan<'a> {
    /// Writes under their version conditions; adjacent ones under the same condition
    /// share a block, as in the decoder.
    writes: Vec<(Option<String>, String)>,
    /// Checks of absent fields, by condition; all the checks under one condition share a
    /// block. Kafka checks each while writing, in field order, so where several fail it
    /// may name another field (docs/protocol/codegen.md#errors).
    checks: Vec<(String, String)>,
    /// The tagged fields, by tag, and the versions they are tagged in.
    tagged: Vec<(u32, &'a FieldSpec, Versions)>,
}

/// Sorts each field of `item` into the writes, checks and tagged fields of its plan.
fn plan<'a>(context: &Context<'_>, item: &Struct<'a>) -> Result<Plan<'a>, String> {
    let (name, versions) = (item.name, item.versions);
    let flexible_versions = versions.intersect(context.message.flexible_versions);
    let mut plan = Plan {
        writes: Vec::new(),
        checks: Vec::new(),
        tagged: Vec::new(),
    };
    for field in item.fields {
        let mandatory = field
            .versions
            .subtract(field.tagged_versions)
            .ok_or_else(|| {
                format!(
                    "in struct {name}: field {} has tagged versions inside its versions",
                    field.name
                )
            })?;
        let mandatory = mandatory.intersect(versions);
        let write = context.write_field(field, mandatory, false);
        match (condition(mandatory, versions), plan.writes.last_mut()) {
            (Condition::Never, _) => {}
            (Condition::When(test), Some((Some(last), block))) if *last == test => {
                block.push('\n');
                block.push_str(&write);
            }
            (Condition::When(test), _) => plan.writes.push((Some(test), write)),
            (Condition::Always, _) => plan.writes.push((None, write)),
        }
        let absent = match outside(field.versions, versions) {
            Condition::Never => None,
            Condition::Always => Some("true".to_owned()),
            Condition::When(test) => Some(test),
        };
        if let (Some(test), false) = (absent, field.ignorable) {
            let check = format!("check({}, {:?})?;", non_default(field), field.name);
            match plan
                .checks
                .iter_mut()
                .find(|(existing, _)| *existing == test)
            {
                Some((_, block)) => {
                    block.push('\n');
                    block.push_str(&check);
                }
                None => plan.checks.push((test, check)),
            }
        }
        if let Some(tag) = field.tag {
            let tag = u32::try_from(tag).map_err(|_| {
                format!("in struct {name}: field {} has a negative tag", field.name)
            })?;
            let tagged_versions = field
                .versions
                .intersect(field.tagged_versions)
                .intersect(flexible_versions);
            plan.tagged.push((tag, field, tagged_versions));
        }
    }
    plan.tagged.sort_by_key(|(tag, _, _)| *tag);
    Ok(plan)
}

/// `check_absent_fields`, which fails before anything is written if a field the version
/// doesn't have holds a value other than its default and isn't `ignorable`. Kafka makes
/// this check while writing, not while sizing, so a sizing writer skips it.
fn render_checks(checks: &[(String, String)], runtime: &str) -> String {
    let blocks = checks
        .iter()
        .fold(String::new(), |mut blocks, (test, block)| {
            let _ = if test == "true" {
                writeln!(blocks, "{block}")
            } else {
                writeln!(blocks, "if {test} {{\n{block}\n}}")
            };
            blocks
        });
    format!(
        "    /// Fails if a field `version` doesn't have holds a value other than its default,\n    \
         /// unless the field is `ignorable`: writing would lose it. Kafka checks this while \
         writing,\n    /// not while sizing.\n    fn check_absent_fields(&self, writer: \
         &{runtime}::Writer, version: i16) -> Result<(), {runtime}::EncodeError> {{\n        \
         if writer.is_sizing() {{\n            return Ok(());\n        }}\n        let check = |non_default: bool, field: &'static str| {{\n            if non_default \
         {{\n                return Err({runtime}::EncodeError::NonDefaultFieldUnsupported {{ \
         field, version }});\n            }}\n            Ok(())\n        }};\n{blocks}Ok(())\n    }}\n"
    )
}

/// The private `write_fields` of `item`; `check_absent_fields` if it has fields that
/// must hold their defaults in some versions; and `write_tagged_fields` if it has known
/// tagged fields.
// Parity: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java@4.4.0 MessageDataGenerator.generateClassWriter
pub(crate) fn render_write(
    out: &mut String,
    item: &Struct<'_>,
    message: &MessageSpec,
    runtime: &str,
    fallible: &BTreeSet<String>,
) -> Result<(), String> {
    let context = Context {
        runtime,
        message,
        fallible,
    };
    let (name, versions) = (item.name, item.versions);
    let plan = plan(&context, item)?;
    let mut body = plan
        .writes
        .iter()
        .fold(String::new(), |mut body, (test, block)| {
            let _ = match test {
                Some(test) => writeln!(body, "if {test} {{\n{block}\n}}"),
                None => writeln!(body, "{block}"),
            };
            body
        });
    let mut methods = String::new();
    let flexible = condition(message.flexible_versions, versions);
    if !matches!(flexible, Condition::Never) {
        let section = tagged_section(
            &context,
            &plan.tagged,
            versions.intersect(message.flexible_versions),
        );
        if plan.tagged.is_empty() {
            body.push_str(&guarded(&flexible, &section));
        } else {
            // A section with known fields is long; it gets its own method.
            body.push_str(&guarded(
                &flexible,
                "self.write_tagged_fields(writer, version)?;",
            ));
            let version = if uses(&section, "version") {
                "version"
            } else {
                "_version"
            };
            let _ = writeln!(
                methods,
                "    fn write_tagged_fields(&self, writer: &mut {runtime}::Writer, {version}: i16) \
                 -> Result<(), {runtime}::EncodeError> {{\n{section}\nOk(())\n    }}\n"
            );
        }
        if let Condition::When(test) = outside(message.flexible_versions, versions) {
            let _ = writeln!(
                body,
                "if {test} && !self.{UNKNOWN_TAGGED_FIELDS}.is_empty() {{ return \
                 Err({runtime}::EncodeError::TaggedFieldsUnsupported {{ version }}); }}"
            );
        }
    }
    if !plan.checks.is_empty() {
        body.insert_str(0, "self.check_absent_fields(writer, version)?;\n");
        methods.push_str(&render_checks(&plan.checks, runtime));
    }
    let writer = if uses(&body, "writer") {
        "writer"
    } else {
        "_writer"
    };
    let version = if uses(&body, "version") {
        "version"
    } else {
        "_version"
    };
    let signature =
        format!("fn write_fields(&self, {writer}: &mut {runtime}::Writer, {version}: i16)");
    let _ = if fallible.contains(name) {
        writeln!(
            out,
            "impl {name} {{\n{methods}    {signature} -> Result<(), {runtime}::EncodeError> \
             {{\n{body}Ok(())\n    }}\n}}\n"
        )
    } else {
        writeln!(
            out,
            "impl {name} {{\n{methods}    {signature} {{\n{body}    }}\n}}\n"
        )
    };
    Ok(())
}

/// The tagged-field section: the count of known fields to write and unknown ones, then
/// each in ascending tag order. Whether each known field is written is decided once, in
/// `wanted_<tag>`.
fn tagged_section(
    context: &Context<'_>,
    tagged: &[(u32, &FieldSpec, Versions)],
    flexible_versions: Versions,
) -> String {
    let runtime = context.runtime;
    let mut wanted = String::new();
    let mut flags = Vec::new();
    let mut writes = String::new();
    for (tag, field, tagged_versions) in tagged {
        let test = match condition(*tagged_versions, flexible_versions) {
            Condition::Never => {
                // The struct defines the tag in every version, so no unknown field may
                // use it, even where the field isn't tagged.
                let _ = writeln!(writes, "raw.write_below(writer, {tag})?;");
                continue;
            }
            Condition::Always => non_default(field),
            Condition::When(test) => format!("{test} && {}", non_default(field)),
        };
        let _ = writeln!(wanted, "let wanted_{tag} = {test};");
        flags.push(format!("wanted_{tag}"));
        let _ = writeln!(
            writes,
            "raw.write_known(writer, {tag}, wanted_{tag}, |writer| {{ {} Ok(()) }})?;",
            context.write_field(field, *tagged_versions, true)
        );
    }
    let count = if flags.is_empty() {
        "raw.len()".to_owned()
    } else {
        format!(
            "[{}].into_iter().filter(|wanted| *wanted).count().saturating_add(raw.len())",
            flags.join(", ")
        )
    };
    format!(
        "let mut raw = {runtime}::RawTaggedFieldWriter::new(&self.{UNKNOWN_TAGGED_FIELDS})?;\n\
         {wanted}writer.write_tagged_field_count({count})?;\n{writes}raw.write_rest(writer)?;"
    )
}

/// `Message::encode` for the message struct: the version check, then `write_fields`.
/// When that fails, a rerun into a sizing writer finds the error Kafka would report
/// first, if it differs (`Writer::sizing` in the protocol crate).
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java@4.4.0 MessageUtil.toByteBufferAccessor
pub(crate) fn render_encode(message: &MessageSpec, runtime: &str, fallible: bool) -> String {
    let name = &message.name;
    let write = if fallible {
        format!(
            "self.write_fields(writer, version).map_err(|error| {{
            let mut sizing =              {runtime}::Writer::sizing();
            self.write_fields(&mut sizing,              version).err().unwrap_or(error)
        }})"
        )
    } else {
        "self.write_fields(writer, version);\n        Ok(())".to_owned()
    };
    format!(
        "    fn encode(&self, writer: &mut {runtime}::Writer, version: i16) -> Result<(), \
         {runtime}::EncodeError> {{\n        if !{name}::VERSIONS.contains(version) {{\n            \
         return Err({runtime}::EncodeError::UnsupportedVersion {{ message: {name:?}, version \
         }});\n        }}\n        {write}\n    }}\n"
    )
}
