// Portions of this file are ported from Apache Kafka 4.4.0 and modified by Alluvium;
// each is marked `// Ported from:`. Apache Kafka is Copyright The Apache Software
// Foundation and licensed under the Apache License 2.0 (LICENSES/Apache-2.0.txt).
// See NOTICE.

//! The typed model of a schema, built from its JSON the way Kafka's `MessageSpec`,
//! `StructSpec` and `FieldSpec` constructors build theirs, with every rule they check,
//! and the per-field rules Kafka's `MessageDataGenerator` checks before generating.
//!
//! Properties are read as Jackson reads them: an unknown property is an error, JSON
//! `null` means the property is absent, a text property also takes a number or a
//! boolean (Jackson's scalar coercion: `"default": -1` is the text `-1`), and a list
//! property also takes a single value (`ACCEPT_SINGLE_VALUE_AS_ARRAY`). Numeric and
//! boolean properties take only numbers and booleans: no vendored schema writes them
//! as text.
//!
//! Note that `"default": null` is an absent default, so the field's default is the
//! type's empty value, not null. Only the text `"null"` asks for a null default.

use serde_json::{Map, Value};

use super::defaults::FieldDefault;
use super::field_type::FieldType;
use super::versions::Versions;

/// A rule a schema breaks, at the JSON pointer of the offending value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Problem {
    pub(crate) pointer: String,
    pub(crate) message: String,
}

impl Problem {
    pub(crate) fn at(pointer: &str, message: impl Into<String>) -> Problem {
        Problem {
            pointer: pointer.to_owned(),
            message: message.into(),
        }
    }
}

/// What a schema describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MessageKind {
    Request,
    Response,
    Header,
    Metadata,
    Data,
    CoordinatorKey,
    CoordinatorValue,
}

/// Where Kafka serves a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Listener {
    Broker,
    Controller,
}

/// What a field's value identifies, for authorization and quota checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntityType {
    Unknown,
    TransactionalId,
    ProducerId,
    GroupId,
    TopicName,
    BrokerId,
}

/// A schema file: one message.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MessageSpec {
    pub(crate) name: String,
    pub(crate) kind: MessageKind,
    pub(crate) api_key: Option<i16>,
    pub(crate) valid_versions: Versions,
    pub(crate) deprecated_versions: Versions,
    /// `none` when the message has no valid versions.
    pub(crate) flexible_versions: Versions,
    pub(crate) listeners: Vec<Listener>,
    pub(crate) latest_version_unstable: bool,
    pub(crate) fields: Vec<FieldSpec>,
    pub(crate) common_structs: Vec<StructSpec>,
}

/// A struct shared by several fields of one message (`commonStructs`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StructSpec {
    pub(crate) name: String,
    pub(crate) versions: Versions,
    pub(crate) fields: Vec<FieldSpec>,
}

/// A field of a message or struct.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FieldSpec {
    pub(crate) name: String,
    pub(crate) versions: Versions,
    /// The fields of an inline struct, for a struct or struct-array field.
    pub(crate) fields: Vec<FieldSpec>,
    pub(crate) field_type: FieldType,
    pub(crate) map_key: bool,
    pub(crate) nullable_versions: Versions,
    pub(crate) default: FieldDefault,
    pub(crate) ignorable: bool,
    pub(crate) entity_type: EntityType,
    pub(crate) about: String,
    pub(crate) tagged_versions: Versions,
    /// The field's own `flexibleVersions`, which overrides the message's.
    pub(crate) flexible_override: Option<Versions>,
    pub(crate) tag: Option<i32>,
    pub(crate) zero_copy: bool,
}

impl FieldSpec {
    /// The versions in which the field is on the wire: its own, within those of the
    /// struct it belongs to.
    // Parity: generator/src/main/java/org/apache/kafka/message/StructRegistry.java@4.4.0 StructRegistry.addStructSpecs
    pub(crate) fn present_versions(&self, parent: Versions) -> Versions {
        self.versions.intersect(parent)
    }

    /// The versions in which the field uses the flexible encoding: its override, or the
    /// message's.
    pub(crate) fn flexible_versions(&self, message: Versions) -> Versions {
        self.flexible_override.unwrap_or(message)
    }
}

const MESSAGE_PROPERTIES: &[&str] = &[
    "name",
    "validVersions",
    "deprecatedVersions",
    "fields",
    "apiKey",
    "type",
    "commonStructs",
    "flexibleVersions",
    "listeners",
    "latestVersionUnstable",
];

const STRUCT_PROPERTIES: &[&str] = &["name", "versions", "deprecatedVersions", "fields"];

const FIELD_PROPERTIES: &[&str] = &[
    "name",
    "versions",
    "fields",
    "type",
    "mapKey",
    "nullableVersions",
    "default",
    "ignorable",
    "entityType",
    "about",
    "taggedVersions",
    "flexibleVersions",
    "tag",
    "zeroCopy",
];

// Parity: generator/src/main/java/org/apache/kafka/message/MessageSpec.java@4.4.0 MessageSpec.MessageSpec
impl MessageSpec {
    /// Builds a message from a schema file's JSON.
    pub(crate) fn from_json(value: &Value) -> Result<MessageSpec, Problem> {
        let object = Object::new(value, "", MESSAGE_PROPERTIES)?;
        let fields = parse_fields(&object)?;
        let name = object.required_text("name")?;
        let valid_versions = object
            .versions("validVersions", None)?
            .ok_or_else(|| missing_versions(&object, &name))?;
        let deprecated_versions = object
            .versions("deprecatedVersions", Some(Versions::NONE))?
            .unwrap_or(Versions::NONE);
        check_struct_fields(&object.pointer, &name, &fields)?;
        let api_key = object.integer("apiKey")?;
        let kind = parse_kind(&object)?;
        let common_structs = object
            .list("commonStructs")
            .into_iter()
            .map(|(pointer, value)| StructSpec::from_json(value, &pointer))
            .collect::<Result<Vec<_>, _>>()?;
        let mut message = MessageSpec {
            name,
            kind,
            api_key,
            valid_versions,
            deprecated_versions,
            flexible_versions: Versions::NONE,
            listeners: Vec::new(),
            latest_version_unstable: false,
            fields,
            common_structs,
        };
        // A message with no valid versions keeps its API key reserved and nothing else.
        if !message.valid_versions.is_empty() {
            let flexible = object.text("flexibleVersions")?.ok_or_else(|| {
                Problem::at(
                    "",
                    "You must specify a value for flexibleVersions. Please use 0+ for all new \
                     messages.",
                )
            })?;
            message.flexible_versions = Versions::parse(Some(&flexible), Some(Versions::NONE))
                .map_err(|error| Problem::at(&object.at("flexibleVersions"), error))?
                .unwrap_or(Versions::NONE);
            message.listeners = object
                .list("listeners")
                .into_iter()
                .map(|(pointer, value)| parse_listener(value, &pointer))
                .collect::<Result<_, _>>()?;
            message.latest_version_unstable = object.boolean("latestVersionUnstable")?;
            message.check()?;
        }
        Ok(message)
    }

    fn check(&self) -> Result<(), Problem> {
        let name = &self.name;
        if !self.flexible_versions.is_empty() && self.flexible_versions.highest() < i16::MAX {
            return Err(Problem::at(
                "/flexibleVersions",
                format!(
                    "Field {name} specifies flexibleVersions {}, which is not open-ended.  \
                     flexibleVersions must be either none, or an open-ended range (that ends \
                     with a plus sign).",
                    self.flexible_versions
                ),
            ));
        }
        if !self.listeners.is_empty() && self.kind != MessageKind::Request {
            return Err(Problem::at(
                "/listeners",
                "The `listeners` property is only valid for messages with type `request`",
            ));
        }
        if self.latest_version_unstable && self.kind != MessageKind::Request {
            return Err(Problem::at(
                "/latestVersionUnstable",
                "The `latestVersionUnstable` property is only valid for messages with type \
                 `request`",
            ));
        }
        match self.kind {
            MessageKind::CoordinatorKey => {
                if self.api_key.is_none() {
                    return Err(Problem::at(
                        "",
                        format!(
                            "The ApiKey must be set for messages {name} with type \
                             `coordinator-key`"
                        ),
                    ));
                }
                if self.valid_versions.to_string() != "0" {
                    return Err(Problem::at(
                        "/validVersions",
                        format!(
                            "The Versions must be set to `0` for messages {name} with type \
                             `coordinator-key`"
                        ),
                    ));
                }
                if !self.flexible_versions.is_empty() {
                    return Err(Problem::at(
                        "/flexibleVersions",
                        format!(
                            "The FlexibleVersions are not supported for messages {name} with \
                             type `coordinator-key`"
                        ),
                    ));
                }
            }
            MessageKind::CoordinatorValue if self.api_key.is_none() => {
                return Err(Problem::at(
                    "",
                    "The ApiKey must be set for messages with type `coordinator-value`",
                ));
            }
            _ => {}
        }
        check_flexible_fields(&self.fields, "/fields", self.flexible_versions)?;
        for (index, common) in self.common_structs.iter().enumerate() {
            check_flexible_fields(
                &common.fields,
                &format!("/commonStructs/{index}/fields"),
                self.flexible_versions,
            )?;
        }
        Ok(())
    }
}

// Parity: generator/src/main/java/org/apache/kafka/message/StructSpec.java@4.4.0 StructSpec.StructSpec
impl StructSpec {
    fn from_json(value: &Value, pointer: &str) -> Result<StructSpec, Problem> {
        let object = Object::new(value, pointer, STRUCT_PROPERTIES)?;
        let fields = parse_fields(&object)?;
        let name = object.required_text("name")?;
        let versions = object
            .versions("versions", None)?
            .ok_or_else(|| missing_versions(&object, &name))?;
        object.versions("deprecatedVersions", Some(Versions::NONE))?;
        check_struct_fields(pointer, &name, &fields)?;
        Ok(StructSpec {
            name,
            versions,
            fields,
        })
    }
}

// Parity: generator/src/main/java/org/apache/kafka/message/FieldSpec.java@4.4.0 FieldSpec.FieldSpec, FieldSpec.checkTagInvariants
impl FieldSpec {
    fn from_json(value: &Value, pointer: &str) -> Result<FieldSpec, Problem> {
        let object = Object::new(value, pointer, FIELD_PROPERTIES)?;
        let fields = parse_fields(&object)?;
        let name = object.required_text("name")?;
        if !is_valid_field_name(&name) {
            return Err(Problem::at(pointer, format!("Invalid field name {name}")));
        }
        let tagged_versions = object
            .versions("taggedVersions", Some(Versions::NONE))?
            .unwrap_or(Versions::NONE);
        let default_versions = (!tagged_versions.is_empty()).then_some(tagged_versions);
        let versions = object
            .versions("versions", default_versions)?
            .ok_or_else(|| missing_versions(&object, &name))?;
        let field_type = FieldType::parse(&object.required_text("type")?)
            .map_err(|error| Problem::at(&object.at("type"), error))?;
        let nullable_versions = object
            .versions("nullableVersions", Some(Versions::NONE))?
            .unwrap_or(Versions::NONE);
        if !nullable_versions.is_empty() && !field_type.can_be_nullable() {
            return Err(Problem::at(
                &object.at("nullableVersions"),
                format!("Type {field_type} cannot be nullable."),
            ));
        }
        let entity_type = parse_entity_type(&object)?;
        verify_entity_type(entity_type, &name, &field_type)
            .map_err(|error| Problem::at(&object.at("entityType"), error))?;
        check_inline_struct(&object, &name, &field_type, &fields)?;
        let flexible_override = parse_flexible_override(&object, &name, &field_type)?;
        let map_key = object.boolean("mapKey")?;
        let tag = object.integer("tag")?;
        if tag.is_some() && map_key {
            return Err(Problem::at(
                pointer,
                "Tagged fields cannot be used as keys.",
            ));
        }
        check_tag_invariants(
            pointer,
            &name,
            tag,
            tagged_versions,
            versions,
            nullable_versions,
        )?;
        let zero_copy = object.boolean("zeroCopy")?;
        if zero_copy && field_type != FieldType::Bytes {
            return Err(Problem::at(
                &object.at("zeroCopy"),
                format!(
                    "Invalid zeroCopy value for {name}. Only fields of type bytes can use \
                     zeroCopy flag."
                ),
            ));
        }
        let default = FieldDefault::parse(
            object.text("default")?.as_deref().unwrap_or_default(),
            &name,
            &field_type,
            versions,
            nullable_versions,
        )
        .map_err(|error| Problem::at(&object.at("default"), error))?;
        Ok(FieldSpec {
            name,
            versions,
            fields,
            field_type,
            map_key,
            nullable_versions,
            default,
            ignorable: object.boolean("ignorable")?,
            entity_type,
            about: object.text("about")?.unwrap_or_default(),
            tagged_versions,
            flexible_override,
            tag,
            zero_copy,
        })
    }
}

/// A field with inline fields is a struct or an array, and an inline struct follows the
/// struct rules.
fn check_inline_struct(
    object: &Object<'_>,
    name: &str,
    field_type: &FieldType,
    fields: &[FieldSpec],
) -> Result<(), Problem> {
    if fields.is_empty() {
        return Ok(());
    }
    if !field_type.is_array() && !field_type.is_struct() {
        return Err(Problem::at(
            &object.pointer,
            format!("Non-array or Struct field {name} cannot have fields"),
        ));
    }
    if field_type.struct_name().is_some() {
        // Kafka checks an inline struct with the field's own `versions` text, so one with
        // only `taggedVersions` is rejected.
        if object.text("versions")?.is_none() {
            return Err(missing_versions(object, name));
        }
        check_struct_fields(&object.pointer, name, fields)?;
    }
    Ok(())
}

/// A field's own `flexibleVersions`, allowed only on strings and bytes. Absent or empty
/// means none.
fn parse_flexible_override(
    object: &Object<'_>,
    name: &str,
    field_type: &FieldType,
) -> Result<Option<Versions>, Problem> {
    let at = object.at("flexibleVersions");
    let Some(text) = object
        .text("flexibleVersions")?
        .filter(|text| !text.is_empty())
    else {
        return Ok(None);
    };
    let parsed = Versions::parse(Some(&text), None)
        .map_err(|error| Problem::at(&at, error))?
        .ok_or_else(|| Problem::at(&at, "blank flexibleVersions"))?;
    if !matches!(field_type, FieldType::String | FieldType::Bytes) {
        return Err(Problem::at(
            &at,
            format!(
                "Invalid flexibleVersions override for {name}.  Only fields of type string or \
                 bytes can specify a flexibleVersions override."
            ),
        ));
    }
    Ok(Some(parsed))
}

/// `[A-Za-z][A-Za-z0-9]*`.
// Parity: generator/src/main/java/org/apache/kafka/message/FieldSpec.java@4.4.0 FieldSpec.VALID_FIELD_NAMES
fn is_valid_field_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric())
}

fn missing_versions(object: &Object<'_>, name: &str) -> Problem {
    Problem::at(
        &object.pointer,
        format!("You must specify the version of the {name} structure."),
    )
}

fn parse_fields(object: &Object<'_>) -> Result<Vec<FieldSpec>, Problem> {
    object
        .list("fields")
        .into_iter()
        .map(|(pointer, value)| FieldSpec::from_json(value, &pointer))
        .collect()
}

/// Field names and tags are unique within a struct, and tags run from 0 with no gap.
// Parity: generator/src/main/java/org/apache/kafka/message/StructSpec.java@4.4.0 StructSpec.StructSpec
fn check_struct_fields(
    pointer: &str,
    struct_name: &str,
    fields: &[FieldSpec],
) -> Result<(), Problem> {
    let mut tags = std::collections::BTreeSet::new();
    let mut names = std::collections::BTreeSet::new();
    for field in fields {
        let field_name = &field.name;
        if let Some(tag) = field.tag
            && !tags.insert(tag)
        {
            return Err(Problem::at(
                pointer,
                format!(
                    "In {struct_name}, field {field_name} has a duplicate tag ID {tag}. All tags \
                     IDs must be unique."
                ),
            ));
        }
        if !names.insert(field_name.as_str()) {
            return Err(Problem::at(
                pointer,
                format!(
                    "In {struct_name}, field {field_name} has a duplicate name {field_name}. All \
                     field names must be unique."
                ),
            ));
        }
    }
    let count = i32::try_from(tags.len()).unwrap_or(i32::MAX);
    if let Some(missing) = (0..count).find(|tag| !tags.contains(tag)) {
        return Err(Problem::at(
            pointer,
            format!(
                "In {struct_name}, the tag IDs are not contiguous.  Make use of tag {missing} \
                 before using any higher tag IDs."
            ),
        ));
    }
    Ok(())
}

fn check_tag_invariants(
    pointer: &str,
    name: &str,
    tag: Option<i32>,
    tagged: Versions,
    versions: Versions,
    nullable: Versions,
) -> Result<(), Problem> {
    let Some(tag) = tag else {
        if tagged.is_empty() {
            return Ok(());
        }
        return Err(Problem::at(
            pointer,
            format!(
                "Field {name} does not specify a tag, but specifies tagged versions of \
                 {tagged}.  Please specify a tag, or remove the taggedVersions."
            ),
        ));
    };
    let message = if tag < 0 {
        format!("Field {name} specifies a tag of {tag}.  Tags cannot be negative.")
    } else if tagged.is_empty() {
        format!(
            "Field {name} specifies a tag of {tag}, but has no tagged versions.  If a tag is \
             specified, taggedVersions must be specified as well."
        )
    } else if !nullable.intersect(tagged).is_empty() && nullable.intersect(tagged) != tagged {
        format!(
            "Field {name} specifies nullableVersions {nullable} and taggedVersions {tagged}.  \
             Either all tagged versions must be nullable, or none must be."
        )
    } else if tagged.highest() < i16::MAX {
        format!(
            "Field {name} specifies taggedVersions {tagged}, which is not open-ended.  \
             taggedVersions must be either none, or an open-ended range (that ends with a plus \
             sign)."
        )
    } else if tagged.intersect(versions) != tagged {
        format!(
            "Field {name} specifies taggedVersions {tagged}, and versions {versions}.  \
             taggedVersions must be a subset of versions."
        )
    } else {
        return Ok(());
    };
    Err(Problem::at(pointer, message))
}

/// A field's flexible versions are within the message's, and its tagged versions within
/// its flexible versions, at every depth.
// Parity: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java@4.4.0 MessageDataGenerator.fieldFlexibleVersions, MessageDataGenerator.generateClassReader
fn check_flexible_fields(
    fields: &[FieldSpec],
    pointer: &str,
    message_flexible: Versions,
) -> Result<(), Problem> {
    for (index, field) in fields.iter().enumerate() {
        let at = format!("{pointer}/{index}");
        let name = &field.name;
        if let Some(own) = field.flexible_override
            && message_flexible.intersect(own) != own
        {
            return Err(Problem::at(
                &at,
                format!(
                    "The flexible versions for field {name} are {own}, which are not a subset \
                     of the flexible versions for the message as a whole, which are \
                     {message_flexible}"
                ),
            ));
        }
        let flexible = field.flexible_versions(message_flexible);
        if field.tagged_versions.intersect(flexible) != field.tagged_versions {
            return Err(Problem::at(
                &at,
                format!(
                    "Field {name} specifies tagged versions {} that are not a subset of the \
                     flexible versions {flexible}",
                    field.tagged_versions
                ),
            ));
        }
        check_flexible_fields(&field.fields, &format!("{at}/fields"), message_flexible)?;
    }
    Ok(())
}

fn parse_kind(object: &Object<'_>) -> Result<MessageKind, Problem> {
    let text = object.required_text("type")?;
    Ok(match text.as_str() {
        "request" => MessageKind::Request,
        "response" => MessageKind::Response,
        "header" => MessageKind::Header,
        "metadata" => MessageKind::Metadata,
        "data" => MessageKind::Data,
        "coordinator-key" => MessageKind::CoordinatorKey,
        "coordinator-value" => MessageKind::CoordinatorValue,
        _ => {
            return Err(Problem::at(
                &object.at("type"),
                format!("unknown message type \"{text}\""),
            ));
        }
    })
}

fn parse_listener(value: &Value, pointer: &str) -> Result<Listener, Problem> {
    match value.as_str() {
        Some("broker") => Ok(Listener::Broker),
        Some("controller") => Ok(Listener::Controller),
        _ => Err(Problem::at(
            pointer,
            format!("unknown listener {value}; expected \"broker\" or \"controller\""),
        )),
    }
}

// Parity: generator/src/main/java/org/apache/kafka/message/EntityType.java@4.4.0 EntityType
fn parse_entity_type(object: &Object<'_>) -> Result<EntityType, Problem> {
    Ok(match object.text("entityType")?.as_deref() {
        None | Some("unknown") => EntityType::Unknown,
        Some("transactionalId") => EntityType::TransactionalId,
        Some("producerId") => EntityType::ProducerId,
        Some("groupId") => EntityType::GroupId,
        Some("topicName") => EntityType::TopicName,
        Some("brokerId") => EntityType::BrokerId,
        Some(other) => {
            return Err(Problem::at(
                &object.at("entityType"),
                format!("unknown entity type \"{other}\""),
            ));
        }
    })
}

/// The entity type's base type is the field's type, or its arrays' element type.
// Parity: generator/src/main/java/org/apache/kafka/message/EntityType.java@4.4.0 EntityType.verifyTypeMatches
fn verify_entity_type(
    entity: EntityType,
    field: &str,
    field_type: &FieldType,
) -> Result<(), String> {
    let (base, java_name) = match entity {
        EntityType::Unknown => return Ok(()),
        EntityType::TransactionalId => (FieldType::String, "TRANSACTIONAL_ID"),
        EntityType::ProducerId => (FieldType::Int64, "PRODUCER_ID"),
        EntityType::GroupId => (FieldType::String, "GROUP_ID"),
        EntityType::TopicName => (FieldType::String, "TOPIC_NAME"),
        EntityType::BrokerId => (FieldType::Int32, "BROKER_ID"),
    };
    match field_type {
        FieldType::Array(element) => verify_entity_type(entity, field, element),
        _ if *field_type == base => Ok(()),
        _ => Err(format!(
            "Field {field} has entity type {java_name}, but field type {field_type}, which does \
             not match."
        )),
    }
}

/// A JSON object at a pointer, whose properties are all among `allowed`.
struct Object<'a> {
    map: &'a Map<String, Value>,
    pointer: String,
}

impl<'a> Object<'a> {
    fn new(value: &'a Value, pointer: &str, allowed: &[&str]) -> Result<Object<'a>, Problem> {
        let map = value
            .as_object()
            .ok_or_else(|| Problem::at(pointer, format!("expected an object, found {value}")))?;
        if let Some(unknown) = map.keys().find(|key| !allowed.contains(&key.as_str())) {
            return Err(Problem::at(
                &format!("{pointer}/{unknown}"),
                format!("Unrecognized field \"{unknown}\""),
            ));
        }
        Ok(Object {
            map,
            pointer: pointer.to_owned(),
        })
    }

    fn at(&self, key: &str) -> String {
        format!("{}/{key}", self.pointer)
    }

    fn get(&self, key: &str) -> Option<&'a Value> {
        self.map.get(key).filter(|value| !value.is_null())
    }

    /// A text property; numbers and booleans are coerced to their text.
    fn text(&self, key: &str) -> Result<Option<String>, Problem> {
        match self.get(key) {
            None => Ok(None),
            Some(Value::String(text)) => Ok(Some(text.clone())),
            Some(scalar @ (Value::Number(_) | Value::Bool(_))) => Ok(Some(scalar.to_string())),
            Some(other) => Err(Problem::at(
                &self.at(key),
                format!("expected text, found {other}"),
            )),
        }
    }

    fn required_text(&self, key: &str) -> Result<String, Problem> {
        self.text(key)?.ok_or_else(|| {
            Problem::at(
                &self.pointer,
                format!("missing required property \"{key}\""),
            )
        })
    }

    fn versions(&self, key: &str, default: Option<Versions>) -> Result<Option<Versions>, Problem> {
        Versions::parse(self.text(key)?.as_deref(), default)
            .map_err(|error| Problem::at(&self.at(key), error))
    }

    fn boolean(&self, key: &str) -> Result<bool, Problem> {
        match self.get(key) {
            None => Ok(false),
            Some(Value::Bool(flag)) => Ok(*flag),
            Some(other) => Err(Problem::at(
                &self.at(key),
                format!("expected true or false, found {other}"),
            )),
        }
    }

    fn integer<T: TryFrom<i64>>(&self, key: &str) -> Result<Option<T>, Problem> {
        let Some(value) = self.get(key) else {
            return Ok(None);
        };
        value
            .as_i64()
            .and_then(|number| T::try_from(number).ok())
            .map(Some)
            .ok_or_else(|| {
                Problem::at(
                    &self.at(key),
                    format!("expected an integer in range, found {value}"),
                )
            })
    }

    /// A list property, or a single value as a list of one, with each item's pointer.
    fn list(&self, key: &str) -> Vec<(String, &'a Value)> {
        match self.get(key) {
            None => Vec::new(),
            Some(Value::Array(items)) => items
                .iter()
                .enumerate()
                .map(|(index, item)| (format!("{}/{index}", self.at(key)), item))
                .collect(),
            Some(single) => vec![(self.at(key), single)],
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{EntityType, FieldSpec, MessageKind, MessageSpec, Problem, verify_entity_type};
    use crate::message_spec::field_type::FieldType;
    use crate::message_spec::versions::Versions;

    fn message(value: &Value) -> Result<MessageSpec, Problem> {
        MessageSpec::from_json(value)
    }

    fn error(value: &Value) -> String {
        message(value).unwrap_err().message
    }

    fn request(fields: &Value) -> Value {
        json!({ "type": "request", "name": "TestRequest", "apiKey": 99,
                "validVersions": "0-4", "flexibleVersions": "2+", "fields": fields })
    }

    fn field(value: &Value) -> Result<FieldSpec, Problem> {
        FieldSpec::from_json(value, "/fields/0")
    }

    fn field_error(value: &Value) -> String {
        field(value).unwrap_err().message
    }

    // Ported from: generator/src/test/java/org/apache/kafka/message/StructSpecTest.java@4.4.0.

    #[test]
    fn names_must_be_unique() {
        let schema = json!({
            "type": "request", "name": "LeaderAndIsrRequest", "validVersions": "0-4",
            "deprecatedVersions": "0-1", "flexibleVersions": "0+",
            "fields": [
                { "name": "field1", "type": "int32", "versions": "0+" },
                { "name": "field1", "type": "[]int64", "versions": "1+" }
            ]
        });
        assert_eq!(
            error(&schema),
            "In LeaderAndIsrRequest, field field1 has a duplicate name field1. All field names \
             must be unique."
        );
    }

    #[test]
    fn tags_must_be_unique() {
        let schema = json!({
            "type": "request", "name": "LeaderAndIsrRequest", "validVersions": "0-4",
            "deprecatedVersions": "0-1", "flexibleVersions": "0+",
            "fields": [
                { "name": "field1", "type": "int32", "versions": "0+", "taggedVersions": "0+", "tag": 0 },
                { "name": "field2", "type": "[]int64", "versions": "0+", "taggedVersions": "0+", "tag": 0 }
            ]
        });
        assert_eq!(
            error(&schema),
            "In LeaderAndIsrRequest, field field2 has a duplicate tag ID 0. All tags IDs must be \
             unique."
        );
    }

    // Ported from: generator/src/test/java/org/apache/kafka/message/EntityTypeTest.java@4.4.0.

    #[test]
    fn unknown_entity_type_matches_every_type() {
        for field_type in [
            FieldType::String,
            FieldType::Int8,
            FieldType::Int16,
            FieldType::Int32,
            FieldType::Int64,
            FieldType::Array(Box::new(FieldType::String)),
        ] {
            assert_eq!(
                verify_entity_type(EntityType::Unknown, "unknown", &field_type),
                Ok(())
            );
        }
    }

    #[test]
    fn verify_type_matches() {
        let array = |element| FieldType::Array(Box::new(element));
        for (entity, field_type) in [
            (EntityType::TransactionalId, FieldType::String),
            (EntityType::TransactionalId, array(FieldType::String)),
            (EntityType::ProducerId, FieldType::Int64),
            (EntityType::ProducerId, array(FieldType::Int64)),
            (EntityType::GroupId, FieldType::String),
            (EntityType::GroupId, array(FieldType::String)),
            (EntityType::TopicName, FieldType::String),
            (EntityType::TopicName, array(FieldType::String)),
            (EntityType::BrokerId, FieldType::Int32),
            (EntityType::BrokerId, array(FieldType::Int32)),
        ] {
            assert_eq!(
                verify_entity_type(entity, "field", &field_type),
                Ok(()),
                "{entity:?}"
            );
        }
    }

    #[test]
    fn verify_type_mismatches() {
        for (entity, field_type) in [
            (EntityType::TransactionalId, FieldType::Int32),
            (EntityType::ProducerId, FieldType::String),
            (EntityType::GroupId, FieldType::Int8),
            (
                EntityType::TopicName,
                FieldType::Array(Box::new(FieldType::Int64)),
            ),
            (EntityType::BrokerId, FieldType::Int64),
        ] {
            assert!(
                verify_entity_type(entity, "field", &field_type).is_err(),
                "{entity:?}"
            );
        }
    }

    // End of the ported tests.

    #[test]
    fn every_property_is_modeled_and_unknown_ones_are_rejected_at_their_pointer() {
        let schema = json!({
            "type": "request", "name": "TestRequest", "apiKey": 99, "validVersions": "0-4",
            "deprecatedVersions": "0", "flexibleVersions": "2+", "listeners": "broker",
            "latestVersionUnstable": true,
            "fields": [{
                "name": "Topics", "type": "[]Topic", "versions": "0+", "about": "The topics.",
                "fields": { "name": "Name", "type": "string", "versions": "0+", "mapKey": true,
                            "entityType": "topicName", "ignorable": true, "default": "x" }
            }]
        });
        let parsed = message(&schema).unwrap();
        assert_eq!(parsed.kind, MessageKind::Request);
        assert_eq!(parsed.api_key, Some(99));
        assert_eq!(parsed.deprecated_versions, Versions::new(0, 0).unwrap());
        assert_eq!(parsed.listeners, [super::Listener::Broker]);
        assert!(parsed.latest_version_unstable);
        let topics = &parsed.fields[0];
        assert_eq!(topics.about, "The topics.");
        let name = &topics.fields[0];
        assert!(name.map_key && name.ignorable);
        assert_eq!(name.entity_type, EntityType::TopicName);

        let mut unknown = schema.clone();
        unknown["fields"][0]["fields"]["color"] = json!("blue");
        assert_eq!(
            message(&unknown),
            Err(Problem::at(
                "/fields/0/fields/color",
                "Unrecognized field \"color\""
            ))
        );
    }

    #[test]
    fn message_level_rules_are_checked() {
        let base = || request(&json!([]));
        let mut no_flexible = base();
        no_flexible
            .as_object_mut()
            .unwrap()
            .remove("flexibleVersions");
        assert!(error(&no_flexible).starts_with("You must specify a value for flexibleVersions"));
        let mut closed = base();
        closed["flexibleVersions"] = json!("2-3");
        assert!(error(&closed).contains("which is not open-ended"));
        let mut response = base();
        response["type"] = json!("response");
        response["listeners"] = json!(["broker"]);
        assert!(error(&response).starts_with("The `listeners` property is only valid"));
        response["listeners"] = json!([]);
        response["latestVersionUnstable"] = json!(true);
        assert!(error(&response).starts_with("The `latestVersionUnstable` property"));

        let key = |changes: Value| {
            let mut schema = json!({ "type": "coordinator-key", "name": "K", "apiKey": 1,
                                     "validVersions": "0", "flexibleVersions": "none", "fields": [] });
            for (name, value) in changes.as_object().unwrap() {
                schema[name] = value.clone();
            }
            schema
        };
        assert!(message(&key(json!({}))).is_ok());
        assert!(error(&key(json!({ "apiKey": null }))).starts_with("The ApiKey must be set"));
        assert!(
            error(&key(json!({ "validVersions": "0-1" })))
                .starts_with("The Versions must be set to `0`")
        );
        assert!(
            error(&key(json!({ "flexibleVersions": "0+" })))
                .starts_with("The FlexibleVersions are not supported")
        );
        let value = json!({ "type": "coordinator-value", "name": "V", "validVersions": "0",
                            "flexibleVersions": "0+", "fields": [] });
        assert!(
            error(&value)
                .starts_with("The ApiKey must be set for messages with type `coordinator-value`")
        );
        assert_eq!(
            message(&json!({ "type": "request", "name": "Gone", "validVersions": "none" }))
                .map(|m| m.fields.len()),
            Ok(0)
        );
    }

    #[test]
    fn field_names_and_tag_ids_are_checked() {
        assert_eq!(
            field_error(&json!({ "name": "1st", "type": "int32", "versions": "0+" })),
            "Invalid field name 1st"
        );
        let gap = request(&json!([
            { "name": "A", "type": "int32", "versions": "2+", "taggedVersions": "2+", "tag": 0 },
            { "name": "B", "type": "int32", "versions": "2+", "taggedVersions": "2+", "tag": 2 }
        ]));
        assert_eq!(
            error(&gap),
            "In TestRequest, the tag IDs are not contiguous.  Make use of tag 1 before using any \
             higher tag IDs."
        );
    }

    #[test]
    fn versions_are_required_unless_tagged_and_presence_is_within_the_parent() {
        assert_eq!(
            field_error(&json!({ "name": "A", "type": "int32" })),
            "You must specify the version of the A structure."
        );
        let tagged =
            field(&json!({ "name": "A", "type": "int32", "taggedVersions": "2+", "tag": 0 }))
                .unwrap();
        assert_eq!(tagged.versions, Versions::new(2, i16::MAX).unwrap());
        let removed = field(&json!({ "name": "Old", "type": "int32", "versions": "0-2" })).unwrap();
        let parent = Versions::new(1, 9).unwrap();
        assert_eq!(
            removed.present_versions(parent),
            Versions::new(1, 2).unwrap()
        );
        let added = field(&json!({ "name": "New", "type": "int32", "versions": "3+" })).unwrap();
        assert_eq!(added.present_versions(parent), Versions::new(3, 9).unwrap());
    }

    #[test]
    fn flexible_overrides_are_for_strings_and_bytes_within_the_message() {
        assert!(field_error(&json!({ "name": "A", "type": "int32", "versions": "0+", "flexibleVersions": "none" }))
            .starts_with("Invalid flexibleVersions override for A."));
        let client_id = field(
            &json!({ "name": "ClientId", "type": "string", "versions": "1+",
                                       "nullableVersions": "1+", "flexibleVersions": "none" }),
        )
        .unwrap();
        let header_flexible = Versions::new(2, i16::MAX).unwrap();
        assert!(!client_id.flexible_versions(header_flexible).contains(2));
        let wider = request(
            &json!([{ "name": "A", "type": "string", "versions": "0+", "flexibleVersions": "1+" }]),
        );
        assert!(
            error(&wider)
                .starts_with("The flexible versions for field A are 1+, which are not a subset")
        );
    }

    #[test]
    fn nullable_versions_need_a_nullable_type() {
        for name in ["bool", "int32", "float64", "uuid"] {
            assert_eq!(
                field_error(
                    &json!({ "name": "A", "type": name, "versions": "0+", "nullableVersions": "0+" })
                ),
                format!("Type {name} cannot be nullable.")
            );
        }
        let message = field(
            &json!({ "name": "ErrorMessage", "type": "string", "versions": "8+",
                                     "nullableVersions": "8+", "default": "null" }),
        )
        .unwrap();
        assert_eq!(
            message.nullable_versions,
            Versions::new(8, i16::MAX).unwrap()
        );
    }

    #[test]
    fn tag_rules_are_checked() {
        let tagged = |changes: Value| {
            let mut value = json!({ "name": "A", "type": "string", "versions": "2+",
                                    "taggedVersions": "2+", "tag": 0 });
            for (name, change) in changes.as_object().unwrap() {
                value[name] = change.clone();
            }
            field(&value).map(drop).map_err(|problem| problem.message)
        };
        assert_eq!(tagged(json!({})), Ok(()));
        for (changes, expected) in [
            (json!({ "tag": -1 }), "Tags cannot be negative."),
            (
                json!({ "taggedVersions": null }),
                "but has no tagged versions.",
            ),
            (json!({ "tag": null }), "does not specify a tag"),
            (
                json!({ "taggedVersions": "2-3" }),
                "which is not open-ended.",
            ),
            (
                json!({ "versions": "3+" }),
                "taggedVersions must be a subset of versions.",
            ),
            (
                json!({ "nullableVersions": "3+" }),
                "Either all tagged versions must be nullable",
            ),
            (
                json!({ "mapKey": true }),
                "Tagged fields cannot be used as keys.",
            ),
        ] {
            let result = tagged(changes.clone());
            assert!(
                result
                    .as_ref()
                    .is_err_and(|message| message.contains(expected)),
                "{changes}: {result:?}"
            );
        }
        let outside_flexible = request(&json!([
            { "name": "A", "type": "int32", "versions": "1+", "taggedVersions": "1+", "tag": 0 }
        ]));
        assert!(
            error(&outside_flexible).contains("that are not a subset of the flexible versions 2+")
        );
    }

    #[test]
    fn zero_copy_is_for_bytes_only() {
        assert!(
            field_error(
                &json!({ "name": "A", "type": "string", "versions": "0+", "zeroCopy": true })
            )
            .starts_with("Invalid zeroCopy value for A.")
        );
        assert!(
            field(&json!({ "name": "A", "type": "bytes", "versions": "0+", "zeroCopy": true }))
                .unwrap()
                .zero_copy
        );
    }

    #[test]
    fn single_values_count_as_lists_and_scalars_as_text() {
        let schema =
            request(&json!({ "name": "A", "type": "int64", "versions": 2, "default": -1 }));
        let parsed = message(&schema).unwrap();
        assert_eq!(parsed.fields.len(), 1);
        assert_eq!(parsed.fields[0].versions, Versions::new(2, 2).unwrap());
    }
}
