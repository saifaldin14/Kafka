//! Field defaults: the value a field has when it is created, and when a version that
//! doesn't carry it is read.
//!
//! An empty default is the type's zero value. Integers take decimal or `0x` hex, each
//! within its type's range. `"null"` asks for null and is allowed only when the field is
//! nullable in all its versions. Kafka ignores a `records` field's default: it is always
//! null.

use std::str::FromStr;

use alluvium_types::Uuid;

use super::field_type::FieldType;
use super::versions::Versions;

/// A field's default, checked against its type.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FieldDefault {
    Bool(bool),
    /// For every integer type; the value is within that type's range.
    Integer(i64),
    Float64(f64),
    Uuid(Uuid),
    String(String),
    /// Empty bytes, an empty struct or an empty array.
    Empty,
    Null,
}

// Parity: generator/src/main/java/org/apache/kafka/message/FieldSpec.java@4.4.0 FieldSpec.fieldDefault, FieldSpec.validateNullDefault
impl FieldDefault {
    /// The default `text` gives a field `name` of `field_type`.
    pub(crate) fn parse(
        text: &str,
        name: &str,
        field_type: &FieldType,
        versions: Versions,
        nullable: Versions,
    ) -> Result<FieldDefault, String> {
        let null = || {
            if nullable.contains_all(versions) {
                Ok(FieldDefault::Null)
            } else {
                Err(format!(
                    "null cannot be the default for field {name}, because not all versions of \
                     this field are nullable."
                ))
            }
        };
        match field_type {
            FieldType::Bool => match text {
                "" => Ok(FieldDefault::Bool(false)),
                _ if text.eq_ignore_ascii_case("true") => Ok(FieldDefault::Bool(true)),
                _ if text.eq_ignore_ascii_case("false") => Ok(FieldDefault::Bool(false)),
                _ => Err(format!("Invalid default for boolean field {name}: {text}")),
            },
            FieldType::Int8
            | FieldType::Int16
            | FieldType::Uint16
            | FieldType::Int32
            | FieldType::Uint32
            | FieldType::Int64 => parse_integer(text, name, field_type).map(FieldDefault::Integer),
            FieldType::Uuid if text.is_empty() => Ok(FieldDefault::Uuid(Uuid::ZERO)),
            FieldType::Uuid => Uuid::from_str(text)
                .map(FieldDefault::Uuid)
                .map_err(|_| format!("Invalid default for uuid field {name}: {text}")),
            FieldType::Float64 if text.is_empty() => Ok(FieldDefault::Float64(0.0)),
            FieldType::Float64 => parse_java_double(text)
                .map(FieldDefault::Float64)
                .ok_or_else(|| format!("Invalid default for float64 field {name}: {text}")),
            FieldType::String if text == "null" => null(),
            FieldType::String => Ok(FieldDefault::String(text.to_owned())),
            FieldType::Records => Ok(FieldDefault::Null),
            FieldType::Bytes | FieldType::Struct(_) | FieldType::Array(_) => match text {
                "null" => null(),
                "" => Ok(FieldDefault::Empty),
                _ => {
                    let kind = match field_type {
                        FieldType::Bytes => "bytes",
                        FieldType::Struct(_) => "struct",
                        _ => "array",
                    };
                    let empty = match field_type {
                        FieldType::Bytes => "empty",
                        FieldType::Struct(_) => "the empty struct",
                        _ => "the empty array",
                    };
                    Err(format!(
                        "Invalid default for {kind} field {name}.  The only valid default for a \
                         {kind} field is {empty} or null."
                    ))
                }
            },
        }
    }
}

/// An integer default: decimal, or hex after a `0x` prefix, with an optional sign, within
/// the Java type's range. `0x` alone, like an empty default, is 0.
fn parse_integer(text: &str, name: &str, field_type: &FieldType) -> Result<i64, String> {
    let (digits, radix) = match text.strip_prefix("0x") {
        Some(hex) => (hex, 16),
        None => (text, 10),
    };
    if digits.is_empty() {
        return Ok(0);
    }
    let invalid = || format!("Invalid default for {field_type} field {name}: {digits}");
    let value = match field_type {
        FieldType::Int8 => i8::from_str_radix(digits, radix).map(i64::from),
        FieldType::Int16 => i16::from_str_radix(digits, radix).map(i64::from),
        FieldType::Int32 | FieldType::Uint16 => i32::from_str_radix(digits, radix).map(i64::from),
        _ => i64::from_str_radix(digits, radix),
    }
    .map_err(|_| invalid())?;
    let range = match field_type {
        FieldType::Uint16 => Some(0..=i64::from(u16::MAX)),
        FieldType::Uint32 => Some(0..=i64::from(u32::MAX)),
        _ => None,
    };
    match range {
        Some(range) if !range.contains(&value) => Err(format!(
            "Invalid default for {field_type} field {name}: out of range."
        )),
        _ => Ok(value),
    }
}

/// Java's `Double.parseDouble` for the forms schemas use: surrounding whitespace, a sign,
/// decimal digits with an optional fraction and exponent, an `f`, `F`, `d` or `D`
/// suffix, `NaN` and `Infinity`. Java's hexadecimal floating-point form is not accepted.
fn parse_java_double(text: &str) -> Option<f64> {
    let text = text.trim();
    let unsigned = text.trim_start_matches(['+', '-']);
    let negative = text.starts_with('-');
    if text.len() - unsigned.len() > 1 {
        return None;
    }
    let value = match unsigned {
        "NaN" => f64::NAN,
        "Infinity" => f64::INFINITY,
        _ => {
            let number = unsigned
                .strip_suffix(['f', 'F', 'd', 'D'])
                .unwrap_or(unsigned);
            let decimal = number
                .chars()
                .all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-'));
            if !decimal || !number.chars().any(|c| c.is_ascii_digit()) {
                return None;
            }
            number.parse::<f64>().ok()?
        }
    };
    Some(if negative { -value } else { value })
}

#[cfg(test)]
mod tests {
    use alluvium_types::Uuid;

    use super::{FieldDefault, parse_java_double};
    use crate::message_spec::field_type::FieldType;
    use crate::message_spec::versions::Versions;

    fn default(text: &str, field_type: &str) -> Result<FieldDefault, String> {
        FieldDefault::parse(
            text,
            "F",
            &FieldType::parse(field_type).unwrap(),
            Versions::ALL,
            Versions::NONE,
        )
    }

    #[test]
    fn integers_take_decimal_and_hex_within_each_types_bounds() {
        for (field_type, low, high) in [
            ("int8", i64::from(i8::MIN), i64::from(i8::MAX)),
            ("int16", i64::from(i16::MIN), i64::from(i16::MAX)),
            ("uint16", 0, i64::from(u16::MAX)),
            ("int32", i64::from(i32::MIN), i64::from(i32::MAX)),
            ("uint32", 0, i64::from(u32::MAX)),
            ("int64", i64::MIN, i64::MAX),
        ] {
            assert_eq!(
                default("", field_type),
                Ok(FieldDefault::Integer(0)),
                "{field_type}"
            );
            for bound in [low, high] {
                assert_eq!(
                    default(&bound.to_string(), field_type),
                    Ok(FieldDefault::Integer(bound)),
                    "{field_type} {bound}"
                );
            }
            if low > i64::MIN {
                assert!(
                    default(&(low - 1).to_string(), field_type).is_err(),
                    "{field_type}"
                );
            }
            if high < i64::MAX {
                assert!(
                    default(&(high + 1).to_string(), field_type).is_err(),
                    "{field_type}"
                );
            }
            assert!(default("one", field_type).is_err(), "{field_type}");
        }
        assert_eq!(default("0x7f", "int8"), Ok(FieldDefault::Integer(127)));
        assert_eq!(default("0x-80", "int8"), Ok(FieldDefault::Integer(-128)));
        assert_eq!(default("0x", "int32"), Ok(FieldDefault::Integer(0)));
        assert_eq!(default("010", "int32"), Ok(FieldDefault::Integer(10)));
        assert_eq!(
            default("0x80", "int8"),
            Err("Invalid default for int8 field F: 80".to_owned())
        );
        assert_eq!(
            default("65536", "uint16"),
            Err("Invalid default for uint16 field F: out of range.".to_owned())
        );
        assert!(default("0X10", "int32").is_err());
    }

    #[test]
    fn other_types_follow_kafkas_rules() {
        assert_eq!(default("TRUE", "bool"), Ok(FieldDefault::Bool(true)));
        assert_eq!(default("", "bool"), Ok(FieldDefault::Bool(false)));
        assert!(default("yes", "bool").is_err());
        assert_eq!(default("", "uuid"), Ok(FieldDefault::Uuid(Uuid::ZERO)));
        assert_eq!(
            default("AAAAAAAAAAAAAAAAAAAAAQ", "uuid"),
            Ok(FieldDefault::Uuid(Uuid::ONE))
        );
        assert!(default("AAAA", "uuid").is_err());
        assert_eq!(default("1.5", "float64"), Ok(FieldDefault::Float64(1.5)));
        assert_eq!(
            default("", "string"),
            Ok(FieldDefault::String(String::new()))
        );
        assert_eq!(
            default("x", "string"),
            Ok(FieldDefault::String("x".to_owned()))
        );
        assert_eq!(default("anything", "records"), Ok(FieldDefault::Null));
        for field_type in ["bytes", "Topic", "[]int32"] {
            assert_eq!(
                default("", field_type),
                Ok(FieldDefault::Empty),
                "{field_type}"
            );
            assert!(default("x", field_type).is_err(), "{field_type}");
        }
    }

    #[test]
    fn null_needs_every_version_nullable() {
        let string = FieldType::String;
        let all = Versions::ALL;
        let from_2 = Versions::new(2, i16::MAX).unwrap();
        assert_eq!(
            FieldDefault::parse("null", "F", &string, from_2, from_2),
            Ok(FieldDefault::Null)
        );
        assert_eq!(
            FieldDefault::parse("null", "F", &string, all, from_2),
            Err(
                "null cannot be the default for field F, because not all versions of this \
                 field are nullable."
                    .to_owned()
            )
        );
    }

    #[test]
    fn doubles_parse_like_java_for_the_supported_forms() {
        for (text, value) in [
            ("1", 1.0),
            (" -2.5e3 ", -2500.0),
            (".5", 0.5),
            ("5.", 5.0),
            ("1d", 1.0),
            ("+7F", 7.0),
            ("-Infinity", f64::NEG_INFINITY),
        ] {
            assert_eq!(parse_java_double(text), Some(value), "{text}");
        }
        assert!(parse_java_double("NaN").is_some_and(f64::is_nan));
        for text in ["inf", "nan", "infinity", "", "e5", "--1", "1x", "0x1p3"] {
            assert_eq!(parse_java_double(text), None, "{text}");
        }
    }
}
