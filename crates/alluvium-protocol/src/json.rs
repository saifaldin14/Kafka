//! [`JsonWriter`]: a message as JSON, field by field in schema order, with the names and
//! value forms of Kafka's generated `JsonConverter`, for debugging tools such as
//! `alluvium dump frame`.
//!
//! Generated code writes each message through [`Message::write_json`](crate::Message::write_json):
//! - the fields its version has, each under its camelCase name; a tagged field only
//!   when it differs from its default; no unknown tagged fields;
//! - integers and `float64` as numbers, `NaN` and the infinities as strings, as
//!   Jackson writes them; uuids in Kafka's base64url form; `bytes` as standard base64;
//! - `records` as `<name>SizeInBytes` and `<name>Crc32c`: Kafka's request log writes the
//!   size instead of the records (KIP-673), and the checksum tells batches apart without
//!   printing them;
//! - credentials (codegen.md's redaction list) as `"***"`, or `""` when empty, which
//!   reveals nothing.
//!
//! Output is compact, one line per message.

use std::fmt::Write as _;

use alluvium_types::{PayloadSummary, Uuid};

use crate::records::RecordsField;

/// Builds compact JSON text. Values follow keys inside objects and each other inside
/// arrays; the writer adds the commas and colons.
#[derive(Debug, Clone, Default)]
pub struct JsonWriter {
    out: String,
    /// Whether the next value or key needs a comma before it.
    after_value: bool,
}

impl JsonWriter {
    /// An empty writer.
    pub fn new() -> JsonWriter {
        JsonWriter::default()
    }

    /// The JSON written.
    pub fn finish(self) -> String {
        self.out
    }

    fn separate(&mut self) {
        if self.after_value {
            self.out.push(',');
        }
    }

    /// Starts an object.
    pub fn begin_object(&mut self) {
        self.separate();
        self.out.push('{');
        self.after_value = false;
    }

    /// Ends the object started last.
    pub fn end_object(&mut self) {
        self.out.push('}');
        self.after_value = true;
    }

    /// Starts an array.
    pub fn begin_array(&mut self) {
        self.separate();
        self.out.push('[');
        self.after_value = false;
    }

    /// Ends the array started last.
    pub fn end_array(&mut self) {
        self.out.push(']');
        self.after_value = true;
    }

    /// An object's key, before its value, which the writer returned is for.
    pub fn key(&mut self, name: &str) -> &mut JsonWriter {
        self.separate();
        self.quoted(name);
        self.out.push(':');
        self.after_value = false;
        self
    }

    /// An array of `values`, each written by `write`.
    pub fn array<T>(&mut self, values: &[T], mut write: impl FnMut(&mut JsonWriter, &T)) {
        self.begin_array();
        for value in values {
            write(self, value);
        }
        self.end_array();
    }

    /// `null`, or `value` written by `write`.
    pub fn nullable<T: ?Sized>(
        &mut self,
        value: Option<&T>,
        write: impl FnOnce(&mut JsonWriter, &T),
    ) {
        match value {
            Some(value) => write(self, value),
            None => self.null(),
        }
    }

    /// `null`.
    pub fn null(&mut self) {
        self.separate();
        self.out.push_str("null");
        self.after_value = true;
    }

    /// `true` or `false`.
    pub fn boolean(&mut self, value: bool) {
        self.separate();
        self.out.push_str(if value { "true" } else { "false" });
        self.after_value = true;
    }

    /// An integer of any of the schema's integer types.
    pub fn integer(&mut self, value: i64) {
        self.separate();
        let _ = write!(self.out, "{value}");
        self.after_value = true;
    }

    /// A `float64`: a number, or for `NaN` and the infinities the string Jackson writes.
    pub fn float(&mut self, value: f64) {
        if value.is_finite() {
            self.separate();
            // `{:?}` keeps a decimal point or exponent, so the value reads back as a float.
            let _ = write!(self.out, "{value:?}");
            self.after_value = true;
        } else if value.is_nan() {
            self.string("NaN");
        } else if value > 0.0 {
            self.string("Infinity");
        } else {
            self.string("-Infinity");
        }
    }

    /// A string.
    pub fn string(&mut self, value: &str) {
        self.separate();
        self.quoted(value);
        self.after_value = true;
    }

    /// A uuid, in Kafka's base64url form.
    pub fn uuid(&mut self, value: Uuid) {
        self.string(&value.to_string());
    }

    /// A byte array, as standard base64 with padding, as Jackson writes a `BinaryNode`.
    pub fn bytes(&mut self, value: &[u8]) {
        self.string(&base64(value));
    }

    /// A credential: `"***"`, or `""` if it is `empty`.
    pub fn redacted(&mut self, empty: bool) {
        self.string(if empty { "" } else { "***" });
    }

    /// A `records` field named `name`: `null` under its name, or for batches their size,
    /// and for batches in memory their CRC-32C, never the records.
    // Parity: generator/src/main/java/org/apache/kafka/message/JsonConverterGenerator.java@4.4.0 JsonConverterGenerator.generateVariableLengthTargetToJson
    pub fn records(&mut self, name: &str, value: Option<&RecordsField>) {
        let Some(value) = value else {
            self.key(name);
            self.null();
            return;
        };
        self.key(&format!("{name}SizeInBytes"));
        self.integer(i64::try_from(value.len()).unwrap_or(i64::MAX));
        if let Some(bytes) = value.as_bytes() {
            self.key(&format!("{name}Crc32c"));
            self.integer(i64::from(PayloadSummary(bytes).crc32c()));
        }
    }

    fn quoted(&mut self, text: &str) {
        self.out.push('"');
        for character in text.chars() {
            match character {
                '"' => self.out.push_str("\\\""),
                '\\' => self.out.push_str("\\\\"),
                '\n' => self.out.push_str("\\n"),
                '\r' => self.out.push_str("\\r"),
                '\t' => self.out.push_str("\\t"),
                control if u32::from(control) < 0x20 => {
                    let _ = write!(self.out, "\\u{:04x}", u32::from(control));
                }
                other => self.out.push(other),
            }
        }
        self.out.push('"');
    }
}

/// `bytes` in standard base64 with padding (RFC 4648 section 4).
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let symbol = |six_bits: u8| {
        char::from(
            ALPHABET
                .get(usize::from(six_bits & 0x3f))
                .copied()
                .unwrap_or(b'='),
        )
    };
    let mut text = String::with_capacity(bytes.len().div_ceil(3).saturating_mul(4));
    for chunk in bytes.chunks(3) {
        let first = chunk.first().copied().unwrap_or(0);
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        text.push(symbol(first >> 2));
        text.push(symbol((first << 4) | (second >> 4)));
        text.push(if chunk.len() > 1 {
            symbol((second << 2) | (third >> 6))
        } else {
            '='
        });
        text.push(if chunk.len() > 2 { symbol(third) } else { '=' });
    }
    text
}

#[cfg(test)]
mod tests {
    use alluvium_types::Uuid;
    use bytes::Bytes;

    use super::{JsonWriter, base64};
    use crate::generated::sasl_authenticate_request::SaslAuthenticateRequest;
    use crate::{Message, RecordsField};

    #[test]
    fn credentials_are_redacted() {
        let request = SaslAuthenticateRequest {
            auth_bytes: Bytes::from_static(b"user\0password"),
            ..SaslAuthenticateRequest::default()
        };
        assert_eq!(request.to_json(2), "{\"authBytes\":\"***\"}");
        assert_eq!(
            SaslAuthenticateRequest::default().to_json(2),
            "{\"authBytes\":\"\"}"
        );
    }

    #[test]
    fn base64_matches_rfc_4648() {
        for (input, expected) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(input.as_bytes()), expected);
        }
        assert_eq!(base64(&[0xfb, 0xff]), "+/8=");
    }

    #[test]
    fn values_are_separated_and_escaped() {
        let mut json = JsonWriter::new();
        json.begin_object();
        json.key("a");
        json.integer(-1);
        json.key("b");
        json.begin_array();
        json.string("q\"\\\n\u{1}é");
        json.null();
        json.boolean(true);
        json.float(12.0);
        json.float(f64::NAN);
        json.float(f64::NEG_INFINITY);
        json.end_array();
        json.key("c");
        json.begin_object();
        json.end_object();
        json.key("d");
        json.uuid(Uuid::ZERO);
        json.records(
            "records",
            Some(&RecordsField::new(Bytes::from_static(b"abc"))),
        );
        json.records(
            "deferred",
            Some(&RecordsField::Deferred { len: 9, token: 1 }),
        );
        json.records("none", None);
        json.key("e");
        json.redacted(false);
        json.key("f");
        json.redacted(true);
        json.end_object();
        assert_eq!(
            json.finish(),
            "{\"a\":-1,\"b\":[\"q\\\"\\\\\\n\\u0001é\",null,true,12.0,\"NaN\",\"-Infinity\"],\
             \"c\":{},\"d\":\"AAAAAAAAAAAAAAAAAAAAAA\",\"recordsSizeInBytes\":3,\
             \"recordsCrc32c\":910901175,\"deferredSizeInBytes\":9,\"none\":null,\"e\":\"***\",\
             \"f\":\"\"}"
        );
    }
}
