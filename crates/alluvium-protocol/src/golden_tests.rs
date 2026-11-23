//! Kafka's own bytes for every message, version and case, recorded from Kafka 4.4.0 by
//! `cargo xtask golden messages` in tests/golden/protocol/messages/ (PROTO-F10).
//!
//! - Every case Kafka wrote must decode at its version, consume all of its input, and
//!   encode back to the same bytes, with `encoded_size` equal to their length. Data
//!   types also round-trip with their version prefix, as Kafka's
//!   `MessageUtil.toVersionPrefixedBytes` writes them.
//! - Every downgrade (a case built at the highest version and written at a lower one)
//!   must encode as Kafka did, or fail where Kafka throws, with the error matching
//!   Kafka's exception and message ([codegen errors](../../../docs/protocol/codegen.md#errors)).
//!
//! The generated `TRANSCODE` table maps each schema name to [`transcode`] for its type.
//! A failure names the message, version and case, and the first differing byte.

use std::path::{Path, PathBuf};

use bytes::Bytes;

use crate::{
    DecodeError, EncodeError, Message, Reader, Writer, decode_version_prefixed,
    encode_version_prefixed,
};

/// Why a case couldn't be transcoded.
pub(crate) enum Failure {
    Decode(DecodeError),
    /// The message decoded, with this many bytes left over.
    Trailing(usize),
    Encode(EncodeError),
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Decode(error) => write!(f, "decode failed: {error}"),
            Failure::Trailing(left) => write!(f, "decoded with {left} bytes left over"),
            Failure::Encode(error) => write!(f, "encode failed: {error}"),
        }
    }
}

impl From<DecodeError> for Failure {
    fn from(error: DecodeError) -> Failure {
        Failure::Decode(error)
    }
}

impl From<EncodeError> for Failure {
    fn from(error: EncodeError) -> Failure {
        Failure::Encode(error)
    }
}

/// [`transcode`] for one message type.
pub(crate) type Transcode = fn(&Bytes, i16, i16, bool) -> Result<(Bytes, usize), Failure>;

/// Decodes `input` as `M` at version `from`, and encodes it at version `to`, returning the
/// bytes and the size `encoded_size` reports. With `prefixed`, the input and output start
/// with the version, as an `int16`.
pub(crate) fn transcode<M: Message>(
    input: &Bytes,
    from: i16,
    to: i16,
    prefixed: bool,
) -> Result<(Bytes, usize), Failure> {
    let mut reader = Reader::new(input.clone());
    let message = if prefixed {
        let (version, message) = decode_version_prefixed::<M>(&mut reader)?;
        assert_eq!(version, from, "the prefix is the version written");
        message
    } else {
        M::decode(&mut reader, from)?
    };
    if reader.remaining() > 0 {
        return Err(Failure::Trailing(reader.remaining()));
    }
    let size = message.encoded_size(to)?.total;
    if prefixed {
        Ok((
            encode_version_prefixed(&message, to)?,
            size.saturating_add(2),
        ))
    } else {
        let mut writer = Writer::new();
        message.encode(&mut writer, to)?;
        Ok((writer.into_bytes(), size))
    }
}

fn golden() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/protocol/messages")
}

fn read(file: &str) -> Bytes {
    Bytes::from(std::fs::read(golden().join(file)).unwrap())
}

/// Whether `error` is the failure Kafka reported as `exception` with `message`, as
/// codegen.md's error table pairs them.
///
/// For a non-default field the version lacks, the kind and version must match but the
/// field may not: where several such fields hold values, Kafka names the first in
/// depth-first write order and Alluvium the first in the outermost struct that has one
/// ([codegen errors](../../../docs/protocol/codegen.md#errors)).
fn matches_kafka(error: &EncodeError, exception: &str, message: Option<&str>) -> bool {
    match error {
        EncodeError::TaggedFieldsUnsupported { version } => {
            exception == "UnsupportedVersionException"
                && message
                    == Some(&*format!(
                        "Tagged fields were set, but version {version} of this message does not \
                         support them."
                    ))
        }
        EncodeError::NonDefaultFieldUnsupported { version, .. } => {
            exception == "UnsupportedVersionException"
                && message.is_some_and(|text| {
                    text.starts_with("Attempted to write a non-default ")
                        && text.ends_with(&format!(" at version {version}"))
                })
        }
        EncodeError::NullForNonNullable { .. } => {
            exception == "NullPointerException" && message.is_none()
        }
        _ => false,
    }
}

/// Where `actual` first differs from `expected`, or `None` if they are equal.
fn first_difference(expected: &[u8], actual: &[u8]) -> Option<String> {
    if expected == actual {
        return None;
    }
    let offset = expected
        .iter()
        .zip(actual)
        .position(|(a, b)| a != b)
        .unwrap_or(expected.len().min(actual.len()));
    Some(format!(
        "first difference at byte {offset} of {} (Rust wrote {} bytes)",
        expected.len(),
        actual.len()
    ))
}

#[test]
fn every_case_kafka_wrote_decodes_and_encodes_as_kafka_writes_it() {
    let cases: serde_json::Value =
        serde_json::from_slice(&std::fs::read(golden().join("cases.json")).unwrap()).unwrap();
    let cases = cases.as_array().unwrap();
    let transcoders: std::collections::BTreeMap<&str, Transcode> =
        crate::generated::TRANSCODE.iter().copied().collect();
    // The file of each case written at the version it was built at.
    let built: std::collections::BTreeMap<(&str, &str, i64), &str> = cases
        .iter()
        .filter(|case| case.get("built_at").is_none())
        .filter_map(|case| {
            Some((
                (
                    case["message"].as_str()?,
                    case["variant"].as_str()?,
                    case["version"].as_i64()?,
                ),
                case["file"].as_str()?,
            ))
        })
        .collect();
    let mut failures = Vec::new();
    let mut checked = 0_usize;
    for case in cases {
        let message = case["message"].as_str().unwrap();
        let variant = case["variant"].as_str().unwrap();
        let version = i16::try_from(case["version"].as_i64().unwrap()).unwrap();
        let built_at = case
            .get("built_at")
            .map(|at| i16::try_from(at.as_i64().unwrap()).unwrap());
        let name = match built_at {
            Some(at) => format!("{message} {variant} built at v{at}, written at v{version}"),
            None => format!("{message} v{version} {variant}"),
        };
        let transcode = transcoders
            .get(message)
            .unwrap_or_else(|| panic!("no codec for {message}"));
        let from = built_at.unwrap_or(version);
        let input = read(built[&(message, variant, i64::from(from))]);
        let result = transcode(&input, from, version, false);
        checked = checked.saturating_add(1);
        match (case.get("file").and_then(serde_json::Value::as_str), result) {
            (Some(file), Ok((bytes, size))) => {
                let expected = read(file);
                if let Some(difference) = first_difference(&expected, &bytes) {
                    failures.push(format!("{name}: {difference}"));
                } else if size != expected.len() {
                    failures.push(format!(
                        "{name}: encoded_size {size}, wrote {}",
                        expected.len()
                    ));
                } else if file.starts_with("data/") && built_at.is_none() {
                    let mut prefixed = version.to_be_bytes().to_vec();
                    prefixed.extend_from_slice(&expected);
                    let prefixed = Bytes::from(prefixed);
                    match transcode(&prefixed, version, version, true) {
                        Ok((bytes, size)) if bytes == prefixed && size == prefixed.len() => {}
                        Ok((bytes, _)) => failures.push(format!(
                            "{name}: version-prefixed: {}",
                            first_difference(&prefixed, &bytes).unwrap_or_default()
                        )),
                        Err(failure) => {
                            failures.push(format!("{name}: version-prefixed: {failure}"));
                        }
                    }
                }
            }
            (None, Ok(_)) => failures.push(format!(
                "{name}: Rust encoded what Kafka refused: {}",
                case["error"]
            )),
            (None, Err(Failure::Encode(error))) => {
                let kafka = &case["error"];
                let exception = kafka["exception"].as_str().unwrap_or_default();
                if !matches_kafka(&error, exception, kafka["message"].as_str()) {
                    failures.push(format!(
                        "{name}: Kafka threw {kafka}, Rust returned {error:?}"
                    ));
                }
            }
            (_, Err(failure)) => failures.push(format!("{name}: {failure}")),
        }
    }
    assert!(checked > 3000, "the cases are all read: {checked}");
    assert!(
        failures.is_empty(),
        "{} of {checked} golden cases fail:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Each sequence of `utf8.json` in a classic string, as Kafka 4.4.0 decodes it and
/// writes it back (PROTO-0124).
#[test]
fn strings_with_invalid_utf8_decode_as_kafka_decodes_them() {
    let cases: serde_json::Value =
        serde_json::from_slice(&std::fs::read(golden().join("utf8.json")).unwrap()).unwrap();
    let mut failures = Vec::new();
    for case in cases.as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let hex = |key: &str| -> Vec<u8> {
            case[key]
                .as_str()
                .unwrap()
                .as_bytes()
                .chunks(2)
                .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                .collect()
        };
        let (input, kafka) = (hex("input"), hex("kafka"));
        let mut framed = i16::try_from(input.len()).unwrap().to_be_bytes().to_vec();
        framed.extend_from_slice(&input);
        let text = Reader::new(Bytes::from(framed))
            .read_string(crate::Framing::Classic)
            .unwrap();
        if text.as_bytes() != kafka {
            failures.push(format!(
                "{name}: Kafka {kafka:02x?}, Rust {:02x?}",
                text.as_bytes()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
