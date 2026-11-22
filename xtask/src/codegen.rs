//! `cargo xtask codegen`: Rust types from Kafka's message schemas, committed under each
//! crate's `src/generated/` ([ADR-0008](../../docs/adr/0008-protocol-code-generated-from-kafka-schemas.md)).
//!
//! [`generate`] is the single entry point: it loads the schemas a [`Filter`] selects
//! with the checks of `message_spec`, renders one module per message, formats each with
//! the pinned toolchain's `rustfmt`, and writes them in sorted order with the `mod.rs`
//! files, removing any file it no longer generates. The output depends only on the
//! schemas and the generator: no timestamps or absolute paths, and the same bytes on
//! every run. How each schema feature maps to Rust is in docs/protocol/codegen.md.

mod docs;
mod names;
mod structs;

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::Exit;
use crate::message_spec::{self, MessageKind};
use crate::output;

/// What one run generates, and from what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Filter {
    /// The kinds of message to generate; other schemas are skipped.
    pub(crate) kinds: Vec<MessageKind>,
    /// The path the generated code uses for `alluvium-protocol`'s types: `crate` inside
    /// that crate, `alluvium_protocol` elsewhere.
    pub(crate) runtime: String,
}

/// The protocol crate's generated code: every request, response, header and data type
/// in Kafka's `clients` schemas.
fn protocol() -> (Vec<PathBuf>, PathBuf, Filter) {
    let root = crate::repo_root();
    (
        vec![root.join("schemas/kafka-4.4.0/clients")],
        root.join("crates/alluvium-protocol/src/generated"),
        Filter {
            kinds: vec![
                MessageKind::Request,
                MessageKind::Response,
                MessageKind::Header,
                MessageKind::Data,
            ],
            runtime: "crate".to_owned(),
        },
    )
}

/// What a run produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Report {
    /// The message modules generated.
    pub(crate) modules: usize,
    /// The most deeply nested message, and its depth.
    pub(crate) deepest: Option<(String, usize)>,
}

pub(crate) fn run(out: &mut impl Write) -> Exit {
    let (inputs, out_dir, filter) = protocol();
    match generate(&inputs, &out_dir, &filter) {
        Ok(report) => {
            let deepest = report.deepest.map_or_else(String::new, |(name, depth)| {
                format!(
                    "; deepest {name} ({depth} levels, limit {})",
                    structs::MAX_NESTING_DEPTH
                )
            });
            let shown = out_dir
                .strip_prefix(crate::repo_root())
                .unwrap_or(&out_dir)
                .display()
                .to_string();
            output::line(
                out,
                &format!("codegen: {} modules in {shown}{deepest}", report.modules),
            );
            Exit::Success
        }
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            Exit::CheckFailed
        }
    }
}

/// Generates the modules for the schemas in `inputs` that `filter` selects into
/// `out_dir`, and removes the files there it no longer generates.
pub(crate) fn generate(
    inputs: &[PathBuf],
    out_dir: &Path,
    filter: &Filter,
) -> Result<Report, String> {
    let (files, report) = render(inputs, filter)?;
    write(out_dir, &files)?;
    Ok(report)
}

/// Every file `generate` would write, by path within the output directory.
fn render(
    inputs: &[PathBuf],
    filter: &Filter,
) -> Result<(BTreeMap<PathBuf, String>, Report), String> {
    let mut modules: BTreeMap<PathBuf, String> = BTreeMap::new();
    let mut report = Report {
        modules: 0,
        deepest: None,
    };
    for path in schema_files(inputs)? {
        let shown = shown_path(&path);
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("could not read {shown}: {error}"))?;
        let message = message_spec::load(&shown, &text).map_err(|error| error.to_string())?;
        // Parity: generator/src/main/java/org/apache/kafka/message/MessageGenerator.java@4.4.0 MessageGenerator.generateAndWriteMessageClasses
        // Kafka generates nothing for a message without valid versions.
        if !filter.kinds.contains(&message.kind) || message.valid_versions.is_empty() {
            continue;
        }
        let module = structs::module(&message, &filter.runtime)
            .map_err(|problem| format!("{shown}: {problem}"))?;
        if report
            .deepest
            .as_ref()
            .is_none_or(|(_, depth)| module.depth > *depth)
        {
            report.deepest = Some((message.name.clone(), module.depth));
        }
        let directory = if message.kind == MessageKind::Data {
            PathBuf::from("data")
        } else {
            PathBuf::new()
        };
        let file = directory.join(format!("{}.rs", names::identifier(&message.name)));
        let source = format!(
            "{}\n{}\n{}",
            header(&format!("from {shown}{}", kafka_version(&path))),
            docs::wrap(&module_doc(&message), "//! "),
            module.source
        );
        if modules.insert(file.clone(), source).is_some() {
            return Err(format!("two schemas generate {}", file.display()));
        }
        report.modules += 1;
    }
    let mut files = mod_files(&modules);
    files.extend(modules);
    let mut formatted = BTreeMap::new();
    for (file, source) in files {
        formatted.insert(file, rustfmt(&source)?);
    }
    Ok((formatted, report))
}

/// The `.json` schema files in `inputs`, sorted, so the output doesn't depend on the
/// order the file system lists them in.
fn schema_files(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for input in inputs {
        let entries = std::fs::read_dir(input)
            .map_err(|error| format!("could not read {}: {error}", input.display()))?;
        for entry in entries {
            let path = entry
                .map_err(|error| format!("could not read {}: {error}", input.display()))?
                .path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// `path` relative to the repository, as generated files cite it.
fn shown_path(path: &Path) -> String {
    let root = crate::repo_root();
    let canonical_root = root.canonicalize().unwrap_or(root);
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    canonical
        .strip_prefix(&canonical_root)
        .unwrap_or(&canonical)
        .display()
        .to_string()
        .replace('\\', "/")
}

/// `, Apache Kafka <version>` for a schema under `schemas/kafka-<version>/`.
fn kafka_version(path: &Path) -> String {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .find_map(|name| name.strip_prefix("kafka-"))
        .map_or_else(String::new, |version| format!(", Apache Kafka {version}"))
}

/// The comment every generated file starts with.
fn header(source: &str) -> String {
    docs::wrap(
        &format!(
            "@generated by `cargo xtask codegen` {source} (Apache License 2.0; see NOTICE). Do \
             not edit: change the schema or the generator in xtask/src/codegen/."
        ),
        "// ",
    )
}

fn module_doc(message: &message_spec::MessageSpec) -> String {
    let kind = match message.kind {
        MessageKind::Request => "request",
        MessageKind::Response => "response",
        MessageKind::Header => "header",
        _ => "data type",
    };
    format!("The `{}` {kind}.", message.name)
}

/// The `mod.rs` of the output directory and of each subdirectory, declaring the modules
/// in them.
fn mod_files(modules: &BTreeMap<PathBuf, String>) -> BTreeMap<PathBuf, String> {
    let mut declared: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
    for file in modules.keys() {
        let directory = file.parent().map(Path::to_path_buf).unwrap_or_default();
        let name = file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_owned();
        if let Some(name) = directory.file_name().and_then(|name| name.to_str()) {
            declared
                .entry(PathBuf::new())
                .or_default()
                .push(name.to_owned());
        }
        declared.entry(directory).or_default().push(name);
    }
    declared
        .into_iter()
        .map(|(directory, mut names)| {
            names.sort();
            names.dedup();
            let doc = if directory.as_os_str().is_empty() {
                "Message types generated from Kafka's schemas, one module per schema \
                 ([ADR-0008](../../../../docs/adr/0008-protocol-code-generated-from-kafka-schemas.md))."
            } else {
                "Data types generated from Kafka's schemas: payloads that travel inside records \
                 and other messages rather than as requests."
            };
            let declarations = names.iter().fold(String::new(), |mut text, name| {
                text.push_str("pub mod ");
                text.push_str(name);
                text.push_str(";\n");
                text
            });
            (
                directory.join("mod.rs"),
                format!("{}\n{}\n{declarations}", header(""), docs::wrap(doc, "//! ")),
            )
        })
        .collect()
}

/// `source` formatted by the pinned toolchain's `rustfmt`, as `cargo fmt` formats it.
fn rustfmt(source: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .current_dir(crate::repo_root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not run rustfmt: {error}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(source.as_bytes())
            .map_err(|error| format!("could not write to rustfmt: {error}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("could not run rustfmt: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "rustfmt rejected generated code: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(|_| "rustfmt wrote invalid UTF-8".to_owned())
}

/// Writes `files` under `out_dir`, skipping files whose contents are unchanged, and
/// removes every other `.rs` file there.
fn write(out_dir: &Path, files: &BTreeMap<PathBuf, String>) -> Result<(), String> {
    for (file, contents) in files {
        let path = out_dir.join(file);
        if std::fs::read_to_string(&path).is_ok_and(|existing| existing == *contents) {
            continue;
        }
        if let Some(directory) = path.parent() {
            std::fs::create_dir_all(directory)
                .map_err(|error| format!("could not create {}: {error}", directory.display()))?;
        }
        std::fs::write(&path, contents)
            .map_err(|error| format!("could not write {}: {error}", path.display()))?;
    }
    for stale in existing_files(out_dir)? {
        let relative = stale.strip_prefix(out_dir).unwrap_or(&stale);
        if !files.contains_key(relative) {
            std::fs::remove_file(&stale)
                .map_err(|error| format!("could not remove {}: {error}", stale.display()))?;
        }
    }
    Ok(())
}

/// Every `.rs` file under `dir`.
fn existing_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("could not read {}: {error}", directory.display())),
        };
        for entry in entries {
            let path = entry
                .map_err(|error| format!("could not read {}: {error}", directory.display()))?
                .path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    use alluvium_types::Uuid;

    use super::{Filter, existing_files, generate, names, protocol, schema_files};
    use crate::message_spec::{
        self, FieldDefault, FieldSpec, FieldType, MessageKind, MessageSpec, StructRegistry,
        Versions,
    };

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xtask-codegen-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn contents(dir: &Path) -> BTreeMap<PathBuf, String> {
        existing_files(dir)
            .unwrap()
            .into_iter()
            .map(|path| {
                let text = std::fs::read_to_string(&path).unwrap();
                (path.strip_prefix(dir).unwrap().to_path_buf(), text)
            })
            .collect()
    }

    #[test]
    fn generate_writes_the_selected_kinds_removes_stale_files_and_is_stable() {
        let inputs = scratch("inputs");
        let out_dir = scratch("out");
        std::fs::write(
            inputs.join("TestRecord.json"),
            r#"// A metadata record, as META-F01 will generate.
            { "apiKey": 2, "type": "metadata", "name": "TestRecord", "validVersions": "0-1",
              "flexibleVersions": "0+", "fields": [
                { "name": "Name", "type": "string", "versions": "0+", "about": "The name." },
                { "name": "Epoch", "type": "int32", "versions": "1+", "default": "-1" } ] }"#,
        )
        .unwrap();
        std::fs::write(
            inputs.join("TestRequest.json"),
            r#"{ "apiKey": 0, "type": "request", "listeners": ["broker"], "name": "TestRequest",
                 "validVersions": "0", "flexibleVersions": "none", "fields": [] }"#,
        )
        .unwrap();
        std::fs::write(out_dir.join("stale.rs"), "// no longer generated\n").unwrap();
        let filter = Filter {
            kinds: vec![MessageKind::Metadata],
            runtime: "alluvium_protocol".to_owned(),
        };

        let report = generate(std::slice::from_ref(&inputs), &out_dir, &filter).unwrap();
        let first = contents(&out_dir);
        assert_eq!(report.modules, 1);
        assert_eq!(report.deepest, Some(("TestRecord".to_owned(), 1)));
        assert_eq!(
            first.keys().collect::<Vec<_>>(),
            [Path::new("mod.rs"), Path::new("test_record.rs")]
        );
        let record = &first[Path::new("test_record.rs")];
        assert!(record.starts_with("// @generated by `cargo xtask codegen` from"));
        assert!(record.contains("Do not edit"));
        assert!(record.contains("pub struct TestRecord {"));
        assert!(record.contains("epoch: -1,"));
        assert!(record.contains("Vec<alluvium_protocol::RawTaggedField>"));
        assert!(record.contains("pub const API_KEY: i16 = 2;"));
        assert!(first[Path::new("mod.rs")].contains("pub mod test_record;\n"));

        generate(std::slice::from_ref(&inputs), &out_dir, &filter).unwrap();
        assert_eq!(contents(&out_dir), first);
        std::fs::remove_dir_all(inputs).unwrap();
        std::fs::remove_dir_all(out_dir).unwrap();
    }

    /// The committed generated module of `message`.
    fn committed(out_dir: &Path, message: &MessageSpec) -> String {
        let dir = if message.kind == MessageKind::Data {
            out_dir.join("data")
        } else {
            out_dir.to_path_buf()
        };
        let file = dir.join(format!("{}.rs", names::identifier(&message.name)));
        std::fs::read_to_string(&file).unwrap_or_else(|error| panic!("{}: {error}", file.display()))
    }

    /// The text of a constant's value, such as `crate::Versions::new(3, 13)`.
    fn constant<'a>(source: &'a str, name: &str) -> Option<&'a str> {
        let (_, declaration) = source.split_once(&format!("pub const {name}: "))?;
        let (_, value) = declaration.split_once(" = ")?;
        Some(value.split_once(';')?.0.trim())
    }

    /// A `Versions` constant read back as Kafka's range text.
    fn versions_text(value: &str) -> String {
        if value == "crate::Versions::NONE" {
            return "none".to_owned();
        }
        let arguments = value
            .strip_prefix("crate::Versions::new(")
            .and_then(|rest| rest.strip_suffix(')'))
            .unwrap_or_else(|| panic!("not a Versions constant: {value}"));
        let (lowest, highest) = arguments.split_once(", ").unwrap();
        match highest {
            "i16::MAX" => format!("{lowest}+"),
            _ if lowest == highest => lowest.to_owned(),
            _ => format!("{lowest}-{highest}"),
        }
    }

    fn check_constants(message: &MessageSpec, source: &str) {
        let name = &message.name;
        assert_eq!(
            constant(source, "API_KEY").map(|key| key.parse::<i16>().unwrap()),
            message.api_key,
            "{name}"
        );
        for (constant_name, expected) in [
            ("VERSIONS", message.valid_versions),
            ("FLEXIBLE_VERSIONS", message.flexible_versions),
            ("DEPRECATED_VERSIONS", message.deprecated_versions),
        ] {
            let value = constant(source, constant_name).unwrap();
            assert_eq!(
                versions_text(value),
                expected.to_string(),
                "{name}::{constant_name}"
            );
        }
        let unstable = constant(source, "LATEST_VERSION_UNSTABLE");
        if message.kind == MessageKind::Request {
            assert_eq!(
                unstable,
                Some(message.latest_version_unstable.to_string().as_str()),
                "{name}"
            );
        } else {
            assert_eq!(unstable, None, "{name}");
        }
    }

    /// Whether a field is an `Option`, from the model: nullable in some version it is
    /// present in, or a records field.
    fn optional(field: &FieldSpec) -> bool {
        field.field_type == FieldType::Records
            || !field.nullable_versions.intersect(field.versions).is_empty()
    }

    /// Whether `expression`, as the generated `Default` sets `field`, is the default
    /// the schema gives it.
    fn is_schema_default(field: &FieldSpec, expression: &str) -> bool {
        if field.field_type == FieldType::Records || field.default == FieldDefault::Null {
            return expression == "None";
        }
        let value = match expression
            .strip_prefix("Some(")
            .and_then(|rest| rest.strip_suffix(')'))
        {
            Some(value) if optional(field) => value,
            Some(_) => return false,
            None if optional(field) => return false,
            None => expression,
        };
        match &field.default {
            FieldDefault::Integer(expected) => {
                value.replace('_', "").parse::<i64>().ok() == Some(*expected)
            }
            FieldDefault::Bool(expected) => value == expected.to_string(),
            FieldDefault::Float64(expected) => value
                .parse::<f64>()
                .is_ok_and(|parsed| parsed.to_bits() == expected.to_bits()),
            FieldDefault::Uuid(expected) => {
                *expected == Uuid::ZERO && value == "alluvium_types::Uuid::ZERO"
            }
            FieldDefault::String(expected) if expected.is_empty() => value == "String::new()",
            FieldDefault::String(expected) => value == format!("String::from({expected:?})"),
            FieldDefault::Empty => match &field.field_type {
                FieldType::Bytes => value == "bytes::Bytes::new()",
                FieldType::Struct(name) => value == format!("{name}::default()"),
                FieldType::Array(_) => value == "Vec::new()",
                _ => false,
            },
            FieldDefault::Null => unreachable!("handled above"),
        }
    }

    /// Whether the Rust type's own `Default` gives `field` the schema's default.
    fn rust_default_is_schema_default(field: &FieldSpec) -> bool {
        match &field.default {
            _ if field.field_type == FieldType::Records => true,
            FieldDefault::Null => true,
            FieldDefault::Integer(value) => *value == 0,
            FieldDefault::Bool(value) => !value,
            FieldDefault::Float64(value) => value.to_bits() == 0.0_f64.to_bits(),
            FieldDefault::Uuid(_) => false,
            FieldDefault::String(text) => text.is_empty() && !optional(field),
            FieldDefault::Empty => !optional(field),
        }
    }

    /// Checks one generated struct's `unknown_tagged_fields` and `Default` against the
    /// model.
    fn check_struct(source: &str, name: &str, fields: &[FieldSpec], flexible: bool) {
        let (before, after) = source
            .split_once(&format!("pub struct {name} {{"))
            .unwrap_or_else(|| panic!("no struct {name}"));
        let body = after.split_once("\n}\n").unwrap().0;
        assert_eq!(
            body.contains("pub unknown_tagged_fields: Vec<crate::RawTaggedField>,"),
            flexible,
            "{name}"
        );
        let derives = before.lines().last().unwrap();
        if derives.contains("Default") {
            for field in fields {
                assert!(
                    rust_default_is_schema_default(field),
                    "{name}.{} derives a default that isn't the schema's",
                    field.name
                );
            }
            return;
        }
        let block = source
            .split_once(&format!("impl Default for {name} {{"))
            .and_then(|(_, after)| after.split_once("\n}\n"))
            .unwrap_or_else(|| panic!("{name} has no Default"))
            .0;
        for field in fields {
            let prefix = format!("            {}: ", names::identifier(&field.name));
            let expression = block
                .lines()
                .find_map(|line| line.strip_prefix(&prefix))
                .and_then(|rest| rest.strip_suffix(','))
                .unwrap_or_else(|| panic!("{name}.{} is not in Default", field.name));
            assert!(
                is_schema_default(field, expression),
                "{name}.{}: {expression}",
                field.name
            );
        }
    }

    fn check_structs(
        source: &str,
        registry: &StructRegistry<'_>,
        fields: &[FieldSpec],
        flexible_versions: Versions,
    ) {
        for field in fields {
            let Some(name) = field.field_type.struct_name() else {
                continue;
            };
            let registered = registry.find(name).unwrap();
            let versions = registered.versions.intersect(registered.parent_versions);
            let flexible = !versions.intersect(flexible_versions).is_empty();
            check_struct(source, name, registered.fields, flexible);
            check_structs(source, registry, registered.fields, flexible_versions);
        }
    }

    #[test]
    fn committed_modules_match_the_schema_model() {
        let (inputs, out_dir, filter) = protocol();
        let mut checked = 0;
        for path in schema_files(&inputs).unwrap() {
            let text = std::fs::read_to_string(&path).unwrap();
            let message = message_spec::load(&path.display().to_string(), &text).unwrap();
            if !filter.kinds.contains(&message.kind) || message.valid_versions.is_empty() {
                continue;
            }
            let source = committed(&out_dir, &message);
            check_constants(&message, &source);
            let flexible = !message
                .valid_versions
                .intersect(message.flexible_versions)
                .is_empty();
            check_struct(&source, &message.name, &message.fields, flexible);
            let registry = StructRegistry::new(&message).unwrap();
            check_structs(
                &source,
                &registry,
                &message.fields,
                message.flexible_versions,
            );
            checked += 1;
        }
        assert_eq!(checked, 195);
    }
}
