// Portions of this file are ported from Apache Kafka 4.4.0 and modified by Alluvium;
// each is marked `// Ported from:`. Apache Kafka is Copyright The Apache Software
// Foundation and licensed under the Apache License 2.0 (LICENSES/Apache-2.0.txt).
// See NOTICE.

//! The structs a message defines: its common structs, and the inline struct each
//! struct-typed field defines the first time its type name appears.
//!
//! As in Kafka, a struct-typed field without inline fields whose type is not a common
//! struct defines an empty struct. Fields of common structs are never registered, so a
//! struct they name must be a common struct or one the message's fields define.

use std::collections::{BTreeMap, BTreeSet};

use super::field_type::FieldType;
use super::spec::{FieldSpec, MessageSpec, Problem};
use super::versions::Versions;

/// One struct of a message.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RegisteredStruct<'a> {
    /// The struct's own versions: a common struct's, or the defining field's.
    pub(crate) versions: Versions,
    /// The versions of the struct that contains it, within the message's.
    pub(crate) parent_versions: Versions,
    pub(crate) fields: &'a [FieldSpec],
}

impl RegisteredStruct<'_> {
    /// Whether any field is a `mapKey`, which makes arrays of this struct keyed.
    pub(crate) fn has_keys(&self) -> bool {
        self.fields.iter().any(|field| field.map_key)
    }
}

/// Every struct of one message, by name.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StructRegistry<'a> {
    structs: BTreeMap<&'a str, RegisteredStruct<'a>>,
    /// The names of the message's common structs.
    pub(crate) common: BTreeSet<&'a str>,
}

// Parity: generator/src/main/java/org/apache/kafka/message/StructRegistry.java@4.4.0 StructRegistry.register, StructRegistry.findStruct, StructRegistry.isStructArrayWithKeys
impl<'a> StructRegistry<'a> {
    /// Registers `message`'s structs, and checks that every struct its fields name is
    /// registered.
    pub(crate) fn new(message: &'a MessageSpec) -> Result<StructRegistry<'a>, Problem> {
        let mut registry = StructRegistry {
            structs: BTreeMap::new(),
            common: BTreeSet::new(),
        };
        for (index, common) in message.common_structs.iter().enumerate() {
            let at = format!("/commonStructs/{index}");
            let name = common.name.as_str();
            if !name.chars().next().is_some_and(char::is_uppercase) {
                return Err(Problem::at(
                    &at,
                    format!(
                        "Can't process structure {name}: the first letter of structure names \
                         must be capitalized."
                    ),
                ));
            }
            if registry.structs.contains_key(name) {
                return Err(Problem::at(
                    &at,
                    format!("Common struct {name} was specified twice."),
                ));
            }
            registry.structs.insert(
                name,
                RegisteredStruct {
                    versions: common.versions,
                    parent_versions: common.versions,
                    fields: &common.fields,
                },
            );
            registry.common.insert(name);
        }
        registry.add_inline(message.valid_versions, &message.fields, "/fields")?;
        registry.check_references(&message.fields, "/fields")?;
        for (index, common) in message.common_structs.iter().enumerate() {
            registry.check_references(&common.fields, &format!("/commonStructs/{index}/fields"))?;
        }
        Ok(registry)
    }

    fn add_inline(
        &mut self,
        parent: Versions,
        fields: &'a [FieldSpec],
        pointer: &str,
    ) -> Result<(), Problem> {
        for (index, field) in fields.iter().enumerate() {
            let Some(name) = field.field_type.struct_name() else {
                continue;
            };
            let at = format!("{pointer}/{index}");
            if self.common.contains(name) {
                if !field.fields.is_empty() {
                    return Err(Problem::at(
                        &at,
                        format!("Can't re-specify the common struct {name} as an inline struct."),
                    ));
                }
            } else if self.structs.contains_key(name) {
                return Err(Problem::at(
                    &at,
                    format!("Struct {name} was specified twice."),
                ));
            } else {
                self.structs.insert(
                    name,
                    RegisteredStruct {
                        // Kafka rebuilds these from the versions' text, so an empty
                        // range becomes `none`.
                        versions: if field.versions.is_empty() {
                            Versions::NONE
                        } else {
                            field.versions
                        },
                        parent_versions: parent,
                        fields: &field.fields,
                    },
                );
            }
            self.add_inline(
                field.present_versions(parent),
                &field.fields,
                &format!("{at}/fields"),
            )?;
        }
        Ok(())
    }

    fn check_references(&self, fields: &[FieldSpec], pointer: &str) -> Result<(), Problem> {
        for (index, field) in fields.iter().enumerate() {
            let at = format!("{pointer}/{index}");
            if let Some(name) = field.field_type.struct_name()
                && !self.structs.contains_key(name)
            {
                return Err(Problem::at(
                    &at,
                    format!("Unable to locate a specification for the structure {name}"),
                ));
            }
            // Only structs' fields are generated; an array of primitives may list fields,
            // which Kafka ignores.
            if field.field_type.struct_name().is_some() {
                self.check_references(&field.fields, &format!("{at}/fields"))?;
            }
        }
        Ok(())
    }

    /// The struct named `name`.
    pub(crate) fn find(&self, name: &str) -> Option<&RegisteredStruct<'a>> {
        self.structs.get(name)
    }

    /// Whether `field` is an array of a struct with a `mapKey` field.
    pub(crate) fn is_struct_array_with_keys(&self, field: &FieldSpec) -> bool {
        match &field.field_type {
            FieldType::Array(element) => element
                .struct_name()
                .and_then(|name| self.find(name))
                .is_some_and(RegisteredStruct::has_keys),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::StructRegistry;
    use crate::message_spec::spec::MessageSpec;
    use crate::message_spec::versions::Versions;

    fn message(value: &Value) -> MessageSpec {
        MessageSpec::from_json(value).unwrap()
    }

    fn register_error(value: &Value) -> String {
        StructRegistry::new(&message(value)).unwrap_err().message
    }

    // Ported from: generator/src/test/java/org/apache/kafka/message/StructRegistryTest.java@4.4.0.

    #[test]
    fn common_structs() {
        let spec = message(&json!({
            "type": "request", "name": "LeaderAndIsrRequest", "validVersions": "0-4",
            "deprecatedVersions": "0-1", "flexibleVersions": "0+",
            "fields": [
                { "name": "field1", "type": "int32", "versions": "0+" },
                { "name": "field2", "type": "[]TestCommonStruct", "versions": "1+" },
                { "name": "field3", "type": "[]TestInlineStruct", "versions": "0+",
                  "fields": [{ "name": "inlineField1", "type": "int64", "versions": "0+" }] }
            ],
            "commonStructs": [
                { "name": "TestCommonStruct", "versions": "0+",
                  "fields": [{ "name": "commonField1", "type": "int64", "versions": "0+" }] }
            ]
        }));
        let registry = StructRegistry::new(&spec).unwrap();
        assert_eq!(
            registry.common.iter().copied().collect::<Vec<_>>(),
            ["TestCommonStruct"]
        );
        assert!(!registry.is_struct_array_with_keys(&spec.fields[1]));
        assert!(!registry.is_struct_array_with_keys(&spec.fields[2]));
        assert!(registry.find("TestCommonStruct").is_some());
    }

    #[test]
    fn re_specified_common_struct_error() {
        assert_eq!(
            register_error(&json!({
                "type": "request", "name": "LeaderAndIsrRequest", "validVersions": "0-2",
                "flexibleVersions": "0+",
                "fields": [
                    { "name": "field1", "type": "int32", "versions": "0+" },
                    { "name": "field2", "type": "[]TestCommonStruct", "versions": "0+",
                      "fields": [{ "name": "inlineField1", "type": "int64", "versions": "0+" }] }
                ],
                "commonStructs": [
                    { "name": "TestCommonStruct", "versions": "0+",
                      "fields": [{ "name": "commonField1", "type": "int64", "versions": "0+" }] }
                ]
            })),
            "Can't re-specify the common struct TestCommonStruct as an inline struct."
        );
    }

    #[test]
    fn duplicate_common_struct_error() {
        let common = json!({ "name": "TestCommonStruct", "versions": "0+",
                             "fields": [{ "name": "commonField1", "type": "int64", "versions": "0+" }] });
        assert_eq!(
            register_error(&json!({
                "type": "request", "name": "LeaderAndIsrRequest", "validVersions": "0-2",
                "flexibleVersions": "0+",
                "fields": [{ "name": "field1", "type": "int32", "versions": "0+" }],
                "commonStructs": [common.clone(), common]
            })),
            "Common struct TestCommonStruct was specified twice."
        );
    }

    #[test]
    fn single_struct() {
        let spec = message(&json!({
            "type": "request", "name": "LeaderAndIsrRequest", "validVersions": "0-2",
            "flexibleVersions": "0+",
            "fields": [
                { "name": "field1", "type": "int32", "versions": "0+" },
                { "name": "field2", "type": "TestInlineStruct", "versions": "0+",
                  "fields": [{ "name": "inlineField1", "type": "int64", "versions": "0+" }] }
            ]
        }));
        let registry = StructRegistry::new(&spec).unwrap();
        let field2 = &spec.fields[1];
        assert!(field2.field_type.is_struct());
        assert_eq!(field2.field_type.to_string(), "TestInlineStruct");
        assert_eq!(field2.name, "field2");
        assert_eq!(
            registry
                .find("TestInlineStruct")
                .map(|found| found.fields.len()),
            Some(1)
        );
        assert!(!registry.is_struct_array_with_keys(field2));
    }

    #[test]
    fn valid_versions_is_none() {
        let spec =
            message(&json!({ "type": "request", "name": "FooBar", "validVersions": "none" }));
        let registry = StructRegistry::new(&spec).unwrap();
        assert!(spec.valid_versions.is_empty());
        assert_eq!(spec.fields, []);
        assert!(registry.common.is_empty() && registry.find("FooBar").is_none());
    }

    // End of the ported tests.

    #[test]
    fn inline_structs_are_registered_once_with_their_parents_versions() {
        let spec = message(&json!({
            "type": "request", "name": "R", "validVersions": "0-5", "flexibleVersions": "0+",
            "fields": [{ "name": "Outer", "type": "[]Outer", "versions": "2+", "fields": [
                { "name": "Name", "type": "string", "versions": "2+", "mapKey": true },
                { "name": "Inner", "type": "Inner", "versions": "3+",
                  "fields": [{ "name": "X", "type": "int8", "versions": "3+" }] }
            ]}]
        }));
        let registry = StructRegistry::new(&spec).unwrap();
        let inner = registry.find("Inner").unwrap();
        assert_eq!(inner.versions, Versions::new(3, i16::MAX).unwrap());
        assert_eq!(inner.parent_versions, Versions::new(2, 5).unwrap());
        assert!(registry.is_struct_array_with_keys(&spec.fields[0]));

        let twice = json!({
            "type": "request", "name": "R", "validVersions": "0", "flexibleVersions": "0+",
            "fields": [
                { "name": "A", "type": "S", "versions": "0+", "fields": [{ "name": "X", "type": "int8", "versions": "0+" }] },
                { "name": "B", "type": "[]S", "versions": "0+", "fields": [{ "name": "Y", "type": "int8", "versions": "0+" }] }
            ]
        });
        assert_eq!(register_error(&twice), "Struct S was specified twice.");
    }

    #[test]
    fn lower_case_common_structs_and_unknown_references_are_rejected() {
        let lower = json!({
            "type": "data", "name": "D", "validVersions": "0", "flexibleVersions": "none",
            "commonStructs": [{ "name": "lower", "versions": "0+", "fields": [] }]
        });
        assert!(register_error(&lower).starts_with("Can't process structure lower"));
        let unknown = json!({
            "type": "data", "name": "D", "validVersions": "0", "flexibleVersions": "none",
            "commonStructs": [{ "name": "C", "versions": "0+",
                                "fields": [{ "name": "X", "type": "Missing", "versions": "0+" }] }]
        });
        assert_eq!(
            register_error(&unknown),
            "Unable to locate a specification for the structure Missing"
        );
    }

    #[test]
    fn empty_inline_ranges_become_none_and_primitive_array_children_are_ignored() {
        let spec = message(&json!({
            "type": "data", "name": "D", "validVersions": "0", "flexibleVersions": "none",
            "fields": [
                { "name": "F", "type": "S", "versions": "2-1" },
                { "name": "G", "type": "[]int32", "versions": "0",
                  "fields": [{ "name": "X", "type": "Missing", "versions": "0" }] }
            ]
        }));
        let registry = StructRegistry::new(&spec).unwrap();
        assert_eq!(
            registry.find("S").map(|found| found.versions),
            Some(Versions::NONE)
        );
    }
}
