//! The generated fuzz targets (PROTO-F11), in `fuzz/generated/`: a package of the fuzz
//! workspace whose every file codegen writes, so the drift check covers it.
//!
//! There is one target per request and per response, one for the request header (read
//! as the broker reads it, at the version its API selects), one for the response header,
//! and one per data type a client sends with its version first. Control records are
//! fuzzed with the records they arrive in (REC-F14). Each target is one call to the
//! checks in `fuzz/src/lib.rs`. Targets are named `protocol_<module>`, as
//! `protocol_produce_request`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use super::names;
use crate::message_spec::{MessageKind, MessageSpec};

/// The data types a client sends after their version, which a broker decodes as such.
// Parity: clients/src/main/java/org/apache/kafka/clients/consumer/internals/ConsumerProtocol.java@4.4.0 ConsumerProtocol.deserializeSubscription, ConsumerProtocol.deserializeAssignment
// Parity: clients/src/main/java/org/apache/kafka/common/security/authenticator/DefaultKafkaPrincipalBuilder.java@4.4.0 DefaultKafkaPrincipalBuilder.deserialize
const VERSION_PREFIXED: [&str; 3] = [
    "ConsumerProtocolAssignment",
    "ConsumerProtocolSubscription",
    "DefaultPrincipalData",
];

/// The fuzz package's files, by path within it, for `messages`, which the protocol
/// crate generates; `header` is the comment each file starts with.
pub(super) fn render(messages: &[&MessageSpec], header: &str) -> BTreeMap<PathBuf, String> {
    let mut targets = BTreeMap::new();
    for message in messages {
        let module = names::identifier(&message.name);
        let name = &message.name;
        let (body, what) = match message.kind {
            MessageKind::Header if name == "RequestHeader" => (
                "alluvium_fuzz::request_header(input);".to_owned(),
                "a request header, at the version its API and version select".to_owned(),
            ),
            MessageKind::Request | MessageKind::Response | MessageKind::Header => (
                format!(
                    "alluvium_fuzz::message::<alluvium_protocol::generated::{module}::{name}>(input);"
                ),
                format!("`{name}` after its version"),
            ),
            _ if VERSION_PREFIXED.contains(&name.as_str()) => (
                format!(
                    "alluvium_fuzz::message::<alluvium_protocol::generated::data::{module}::{name}>(input);"
                ),
                format!("`{name}` after its version"),
            ),
            _ => continue,
        };
        targets.insert(
            format!("protocol_{module}"),
            format!(
                "{header}\n//! Fuzzes {what} (fuzz/src/lib.rs).\n#![no_main]\n\n\
                 libfuzzer_sys::fuzz_target!(|input: &[u8]| {{\n    {body}\n}});\n"
            ),
        );
    }
    let mut files: BTreeMap<PathBuf, String> = targets
        .iter()
        .map(|(target, source)| {
            (
                PathBuf::from("fuzz_targets").join(format!("{target}.rs")),
                source.clone(),
            )
        })
        .collect();
    files.insert(PathBuf::from("Cargo.toml"), manifest(&targets, header));
    files
}

/// The package manifest, listing every target.
fn manifest(targets: &BTreeMap<String, String>, header: &str) -> String {
    let mut out = format!(
        "{}\n[package]\nname = \"alluvium-fuzz-generated\"\nversion = \"0.0.0\"\nedition = \
         \"2024\"\npublish = false\nlicense-file = \"../../LICENSE\"\n\n[package.metadata]\n\
         cargo-fuzz = true\n\n[dependencies]\nalluvium-fuzz = {{ path = \"..\" }}\n\
         alluvium-protocol = {{ path = \"../../crates/alluvium-protocol\" }}\n\
         libfuzzer-sys = \"0.4.13\"\n",
        header.replace("// ", "# ")
    );
    for target in targets.keys() {
        let _ = write!(
            out,
            "\n[[bin]]\nname = \"{target}\"\npath = \"fuzz_targets/{target}.rs\"\ntest = \
             false\ndoc = false\nbench = false\n"
        );
    }
    out
}
