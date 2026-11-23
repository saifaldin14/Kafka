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
    DecodeError, EncodeError, Message, Reader, Segment, Writer, decode_version_prefixed,
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

/// What [`transcode`] writes around the message.
pub(crate) enum Wrap {
    /// Nothing.
    None,
    /// The version, as an `int16`, before the input and the output.
    VersionPrefix,
    /// The output is a frame from `encode_response`: a response header, then the message.
    Response {
        api: crate::ApiKey,
        correlation_id: i32,
    },
    /// The output is a frame from `encode_request`: this header, then the message.
    Request(crate::generated::request_header::RequestHeader),
}

/// [`to_json`] for one message type.
pub(crate) type ToJson = fn(&Bytes, i16) -> Result<String, DecodeError>;

/// Decodes `input` as `M` at `version` and writes it as JSON.
pub(crate) fn to_json<M: Message>(input: &Bytes, version: i16) -> Result<String, DecodeError> {
    Ok(M::decode(&mut Reader::new(input.clone()), version)?.to_json(version))
}

/// [`transcode`] for one message type.
pub(crate) type Transcode = fn(&Bytes, i16, i16, &Wrap) -> Result<(Bytes, usize), Failure>;

/// Decodes `input` as `M` at version `from`, and encodes it at version `to`, wrapped as
/// `wrap` says, returning the bytes and the size `encoded_size` reports for them.
pub(crate) fn transcode<M: Message>(
    input: &Bytes,
    from: i16,
    to: i16,
    wrap: &Wrap,
) -> Result<(Bytes, usize), Failure> {
    let prefixed = matches!(wrap, Wrap::VersionPrefix);
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
    let header_error = |error: crate::HeaderError| match error {
        crate::HeaderError::Encode(error) => Failure::Encode(error),
        other => panic!("the header of a golden case: {other}"),
    };
    let frame = match wrap {
        Wrap::None => {
            let mut writer = Writer::new();
            message.encode(&mut writer, to)?;
            return Ok((writer.into_bytes(), size));
        }
        Wrap::VersionPrefix => {
            return Ok((
                encode_version_prefixed(&message, to)?,
                size.saturating_add(2),
            ));
        }
        Wrap::Response {
            api,
            correlation_id,
        } => crate::encode_response(*api, to, *correlation_id, &message).map_err(header_error)?,
        Wrap::Request(header) => crate::encode_request(header, &message).map_err(header_error)?,
    };
    let mut bytes = Vec::with_capacity(frame.len());
    for segment in frame.segments() {
        match segment {
            Segment::Bytes(part) => bytes.extend_from_slice(part),
            // A decoded message holds its batches in memory, so this can't happen.
            Segment::Deferred { len, .. } => {
                return Err(Failure::Encode(EncodeError::DeferredRecordsNotInFrame {
                    len: *len,
                }));
            }
        }
    }
    Ok((Bytes::from(bytes), frame.len()))
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

/// How a case's message is also checked wrapped, and the bytes it must then give: a data
/// type with its version prefix, as Kafka's `toVersionPrefixedBytes` writes it, and a
/// request or response in a frame after its header (whose own bytes the header vectors
/// check). `body` is Kafka's encoding of the message.
fn wrapped(message: &str, file: &str, version: i16, body: &[u8]) -> Option<(Wrap, Bytes)> {
    use crate::generated::request_header::RequestHeader;
    let api = |suffix: &str| {
        let name = message.strip_suffix(suffix)?;
        crate::ApiKey::ALL
            .into_iter()
            .find(|api| api.name() == name)
    };
    let mut writer = Writer::new();
    let wrap = if file.starts_with("data/") {
        writer.write_i16(version);
        Wrap::VersionPrefix
    } else if let Some(api) = api("Request") {
        let header = RequestHeader {
            request_api_key: api.id(),
            request_api_version: version,
            correlation_id: 7,
            client_id: Some("golden".to_owned()),
            ..RequestHeader::default()
        };
        crate::encode_request_header(&header, &mut writer).ok()?;
        Wrap::Request(header)
    } else {
        let api = api("Response")?;
        crate::encode_response_header(api, version, 7, &mut writer).ok()?;
        Wrap::Response {
            api,
            correlation_id: 7,
        }
    };
    let mut framed = writer.into_bytes().to_vec();
    framed.extend_from_slice(body);
    Some((wrap, Bytes::from(framed)))
}

/// What [`transcode`] reads for `wrap`: the prefixed bytes for a version prefix, and
/// otherwise the message alone, since only the output carries a header.
fn framed_input(wrap: &Wrap, framed: &Bytes, body: &Bytes) -> Bytes {
    match wrap {
        Wrap::VersionPrefix => framed.clone(),
        _ => body.clone(),
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
    let mut wrapped_checked = 0_usize;
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
        let result = transcode(&input, from, version, &Wrap::None);
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
                } else if built_at.is_none()
                    && let Some((wrap, framed)) = wrapped(message, file, version, &expected)
                {
                    wrapped_checked = wrapped_checked.saturating_add(1);
                    match transcode(
                        &framed_input(&wrap, &framed, &expected),
                        version,
                        version,
                        &wrap,
                    ) {
                        Ok((bytes, size)) if bytes == framed && size == framed.len() => {}
                        Ok((bytes, _)) => failures.push(format!(
                            "{name}: wrapped: {}",
                            first_difference(&framed, &bytes).unwrap_or_default()
                        )),
                        Err(failure) => failures.push(format!("{name}: wrapped: {failure}")),
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
        wrapped_checked > 1900,
        "every case Kafka wrote is also wrapped: {wrapped_checked}"
    );
    assert!(
        failures.is_empty(),
        "{} of {checked} golden cases fail:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// PROTO-0141: every message Kafka wrote, read and written as JSON, is the JSON Kafka's
/// `JsonConverter` wrote for it, after two documented normalizations
/// ([`normalize_json`]).
#[test]
fn every_case_kafka_wrote_reads_as_kafkas_json() {
    let cases: serde_json::Value =
        serde_json::from_slice(&std::fs::read(golden().join("cases.json")).unwrap()).unwrap();
    let to_json: std::collections::BTreeMap<&str, ToJson> =
        crate::generated::TO_JSON.iter().copied().collect();
    let mut failures = Vec::new();
    let mut checked = 0_usize;
    for case in cases.as_array().unwrap() {
        let (Some(file), None) = (case["file"].as_str(), case.get("built_at")) else {
            continue;
        };
        let message = case["message"].as_str().unwrap();
        let version = i16::try_from(case["version"].as_i64().unwrap()).unwrap();
        let ours = to_json[message](&read(file), version).unwrap();
        let mut ours: serde_json::Value =
            serde_json::from_str(&ours).unwrap_or_else(|error| panic!("{file}: {error}: {ours}"));
        let kafka_file = file.replace(".bin", ".json");
        let kafka: serde_json::Value = serde_json::from_slice(&read(&kafka_file)).unwrap();
        normalize_json(&mut ours, &kafka);
        if ours != kafka {
            failures.push(format!("{file}:\n  ours:  {ours}\n  Kafka: {kafka}"));
        }
        checked = checked.saturating_add(1);
    }
    assert!(
        checked > 1900,
        "every case Kafka wrote is checked: {checked}"
    );
    assert!(
        failures.is_empty(),
        "{} of {checked} cases differ from Kafka's JSON:\n{}",
        failures.len(),
        failures
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Makes `ours` comparable with Kafka's JSON:
/// - Kafka's golden JSON writes every non-null `records` field as an empty string. Ours
///   writes `<name>SizeInBytes` and `<name>Crc32c` instead, which become `<name>: ""`.
/// - A redacted credential (`"***"`) stands for whatever Kafka wrote there.
fn normalize_json(ours: &mut serde_json::Value, kafka: &serde_json::Value) {
    match (ours, kafka) {
        (serde_json::Value::Object(ours), serde_json::Value::Object(kafka)) => {
            let summarized: Vec<String> = ours
                .keys()
                .filter_map(|key| key.strip_suffix("SizeInBytes"))
                .filter(|name| kafka.get(*name) == Some(&serde_json::Value::from("")))
                .map(str::to_owned)
                .collect();
            for name in summarized {
                ours.remove(&format!("{name}SizeInBytes"));
                ours.remove(&format!("{name}Crc32c"));
                ours.insert(name, serde_json::Value::from(""));
            }
            for (key, value) in ours.iter_mut() {
                if let Some(theirs) = kafka.get(key) {
                    normalize_json(value, theirs);
                }
            }
        }
        (serde_json::Value::Array(ours), serde_json::Value::Array(kafka)) => {
            for (value, theirs) in ours.iter_mut().zip(kafka) {
                normalize_json(value, theirs);
            }
        }
        (ours @ serde_json::Value::String(_), theirs @ serde_json::Value::String(_))
            if ours == "***" =>
        {
            *ours = theirs.clone();
        }
        _ => {}
    }
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
        let (input, kafka) = (hex(&case["input"]), hex(&case["kafka"]));
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

/// A function encoding `count` arbitrary values of one message at a version.
pub(crate) type EncodeArbitrary = fn(i16, u32) -> Vec<Bytes>;

/// `count` values from `strategy`, encoded at `version`, from a fixed seed.
pub(crate) fn encode_arbitrary<M: Message + std::fmt::Debug>(
    strategy: &proptest::strategy::BoxedStrategy<M>,
    version: i16,
    count: u32,
) -> Vec<Bytes> {
    use proptest::strategy::{Strategy, ValueTree};
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    (0..count)
        .map(|_| {
            let value = strategy.new_tree(&mut runner).unwrap().current();
            let mut writer = Writer::new();
            value.encode(&mut writer, version).unwrap();
            writer.into_bytes()
        })
        .collect()
}

/// How many arbitrary messages the reverse check sends Kafka per message and version.
const REVERSE_CHECK_MESSAGES: u32 = 1000;

/// Writes the messages `cargo xtask golden verify` has Kafka 4.4.0 decode and re-encode
/// (PROTO-0119): [`REVERSE_CHECK_MESSAGES`] arbitrary values of each message at each
/// version, in `$ALLUVIUM_REVERSE_CHECK_DIR/<Message>_v<version>.bin`, each value
/// preceded by its length as an `int32`.
#[test]
#[ignore = "writes files for `cargo xtask golden verify`, which runs it"]
fn write_messages_for_the_reverse_check() {
    let dir = PathBuf::from(std::env::var("ALLUVIUM_REVERSE_CHECK_DIR").unwrap());
    std::fs::create_dir_all(&dir).unwrap();
    for (name, lowest, highest, encode) in crate::generated::ENCODE_ARBITRARY {
        for version in *lowest..=*highest {
            let mut file = Vec::new();
            for message in encode(version, REVERSE_CHECK_MESSAGES) {
                file.extend_from_slice(&i32::try_from(message.len()).unwrap().to_be_bytes());
                file.extend_from_slice(&message);
            }
            std::fs::write(dir.join(format!("{name}_v{version}.bin")), file).unwrap();
        }
    }
}

/// The bytes a JSON string of hex digits stands for.
fn hex(value: &serde_json::Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

/// Each request and response header in `headers.json`, as Kafka 4.4.0 writes and reads
/// it, for every API at its oldest, latest and next version, with a null, an empty and a
/// non-ASCII client id (PROTO-0092, PROTO-0120).
#[test]
fn headers_are_written_and_read_as_kafka_does() {
    use crate::generated::request_header::RequestHeader;
    use crate::{
        ApiKey, decode_request_header, decode_response_header, encode_request_header,
        encode_response_header,
    };
    let cases: serde_json::Value =
        serde_json::from_slice(&std::fs::read(golden().join("headers.json")).unwrap()).unwrap();
    let cases = cases.as_array().unwrap();
    assert!(cases.len() > 800, "the cases are all read");
    let mut failures = Vec::new();
    for case in cases {
        let number = |key: &str| case[key].as_i64().unwrap();
        let api = ApiKey::from_id(i16::try_from(number("api_key")).unwrap()).unwrap();
        let version = i16::try_from(number("version")).unwrap();
        let correlation_id = i32::try_from(number("correlation_id")).unwrap();
        let client_id = case["client_id"].as_str().map(str::to_owned);
        let name = format!("{} v{version}, client id {client_id:?}", api.name());
        let header = RequestHeader {
            request_api_key: api.id(),
            request_api_version: version,
            correlation_id,
            client_id,
            ..RequestHeader::default()
        };
        let (request, response) = (hex(&case["request"]), hex(&case["response"]));
        let mut writer = Writer::new();
        encode_request_header(&header, &mut writer).unwrap();
        if let Some(difference) = first_difference(&request, &writer.into_bytes()) {
            failures.push(format!("{name}: request header: {difference}"));
        }
        match decode_request_header(&mut Reader::new(Bytes::from(request))) {
            Ok(decoded) => {
                let kafka = (
                    i64::from(decoded.header_version),
                    decoded.header.client_id.as_deref(),
                    decoded.size,
                );
                let expected = (
                    number("request_header_version"),
                    case["parsed_client_id"].as_str(),
                    usize::try_from(number("parsed_size")).unwrap(),
                );
                if kafka != expected
                    || decoded.api != api
                    || decoded.header.correlation_id != correlation_id
                {
                    failures.push(format!(
                        "{name}: decoded {decoded:?}, Kafka read {expected:?}"
                    ));
                }
            }
            Err(error) => failures.push(format!("{name}: request header: {error}")),
        }
        let mut writer = Writer::new();
        encode_response_header(api, version, correlation_id, &mut writer).unwrap();
        if let Some(difference) = first_difference(&response, &writer.into_bytes()) {
            failures.push(format!("{name}: response header: {difference}"));
        }
        let decoded = decode_response_header(api, version, &mut Reader::new(Bytes::from(response)));
        if decoded != Ok(correlation_id) {
            failures.push(format!("{name}: response header decoded as {decoded:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
