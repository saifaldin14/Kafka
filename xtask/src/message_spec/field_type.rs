//! Field types in schemas: the primitives, struct names and `[]T` arrays.

use std::fmt;

/// A field's type, as Kafka's generator parses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FieldType {
    Bool,
    Int8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Int64,
    Float64,
    Uuid,
    String,
    Bytes,
    Records,
    /// A struct, by its name, which starts with an upper-case letter.
    Struct(String),
    /// An array of any type except another array.
    Array(Box<FieldType>),
}

// Parity: generator/src/main/java/org/apache/kafka/message/FieldType.java@4.4.0 FieldType.parse, FieldType.canBeNullable
impl FieldType {
    /// Parses a type name, ignoring surrounding whitespace.
    pub(crate) fn parse(text: &str) -> Result<FieldType, String> {
        let text = text.trim();
        Ok(match text {
            "bool" => FieldType::Bool,
            "int8" => FieldType::Int8,
            "int16" => FieldType::Int16,
            "uint16" => FieldType::Uint16,
            "int32" => FieldType::Int32,
            "uint32" => FieldType::Uint32,
            "int64" => FieldType::Int64,
            "float64" => FieldType::Float64,
            "uuid" => FieldType::Uuid,
            "string" => FieldType::String,
            "bytes" => FieldType::Bytes,
            "records" => FieldType::Records,
            _ => {
                if let Some(element) = text.strip_prefix("[]") {
                    if element.is_empty() {
                        return Err(format!(
                            "Can't parse array type {text}.  No element type found."
                        ));
                    }
                    let element = FieldType::parse(element)?;
                    if element.is_array() {
                        return Err("Can't have an array of arrays.  Use an array of structs \
                                    containing an array instead."
                            .to_owned());
                    }
                    FieldType::Array(Box::new(element))
                } else if text.chars().next().is_some_and(char::is_uppercase) {
                    FieldType::Struct(text.to_owned())
                } else {
                    return Err(format!("Can't parse type {text}"));
                }
            }
        })
    }

    pub(crate) fn is_array(&self) -> bool {
        matches!(self, FieldType::Array(_))
    }

    pub(crate) fn is_struct(&self) -> bool {
        matches!(self, FieldType::Struct(_))
    }

    /// The struct this type names, directly or as an array's element.
    pub(crate) fn struct_name(&self) -> Option<&str> {
        match self {
            FieldType::Struct(name) => Some(name),
            FieldType::Array(element) => element.struct_name(),
            _ => None,
        }
    }

    /// Whether a value of this type can be null on the wire.
    pub(crate) fn can_be_nullable(&self) -> bool {
        matches!(
            self,
            FieldType::String
                | FieldType::Bytes
                | FieldType::Records
                | FieldType::Struct(_)
                | FieldType::Array(_)
        )
    }
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            FieldType::Bool => "bool",
            FieldType::Int8 => "int8",
            FieldType::Int16 => "int16",
            FieldType::Uint16 => "uint16",
            FieldType::Int32 => "int32",
            FieldType::Uint32 => "uint32",
            FieldType::Int64 => "int64",
            FieldType::Float64 => "float64",
            FieldType::Uuid => "uuid",
            FieldType::String => "string",
            FieldType::Bytes => "bytes",
            FieldType::Records => "records",
            FieldType::Struct(name) => name,
            FieldType::Array(element) => return write!(f, "[]{element}"),
        };
        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::FieldType;

    #[test]
    fn every_type_parses_and_prints_back() {
        for name in [
            "bool",
            "int8",
            "int16",
            "uint16",
            "int32",
            "uint32",
            "int64",
            "float64",
            "uuid",
            "string",
            "bytes",
            "records",
            "TopicData",
            "[]int32",
            "[]TopicData",
            "[]string",
        ] {
            assert_eq!(FieldType::parse(name).unwrap().to_string(), name);
        }
        assert_eq!(FieldType::parse(" int32 "), Ok(FieldType::Int32));
        assert_eq!(
            FieldType::parse("[]Topic"),
            Ok(FieldType::Array(Box::new(FieldType::Struct(
                "Topic".to_owned()
            ))))
        );
    }

    #[test]
    fn arrays_of_arrays_unknown_primitives_and_lower_case_structs_are_rejected() {
        assert_eq!(
            FieldType::parse("[][]int32"),
            Err(
                "Can't have an array of arrays.  Use an array of structs containing an array \
                 instead."
                    .to_owned()
            )
        );
        assert_eq!(
            FieldType::parse("[]"),
            Err("Can't parse array type [].  No element type found.".to_owned())
        );
        for name in ["int128", "topicData", "", "[]int128"] {
            assert!(FieldType::parse(name).is_err(), "{name}");
        }
    }

    #[test]
    fn only_strings_bytes_records_structs_and_arrays_can_be_nullable() {
        for (name, nullable) in [
            ("bool", false),
            ("int8", false),
            ("int16", false),
            ("uint16", false),
            ("int32", false),
            ("uint32", false),
            ("int64", false),
            ("float64", false),
            ("uuid", false),
            ("string", true),
            ("bytes", true),
            ("records", true),
            ("Topic", true),
            ("[]int32", true),
        ] {
            assert_eq!(
                FieldType::parse(name).unwrap().can_be_nullable(),
                nullable,
                "{name}"
            );
        }
    }
}
