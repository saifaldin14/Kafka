// Portions of this file are ported from Apache Kafka 4.4.0 and modified by Alluvium;
// each is marked `// Ported from:`. Apache Kafka is Copyright The Apache Software
// Foundation and licensed under the Apache License 2.0 (LICENSES/Apache-2.0.txt).
// See NOTICE.

//! `cargo xtask codegen`: Rust types from Kafka's message schemas and error codes,
//! committed under each crate's `src/generated/`
//! ([ADR-0008](../../docs/adr/0008-protocol-code-generated-from-kafka-schemas.md)).
//!
//! [`generate`] is the single entry point: it loads the schemas a [`Filter`] selects
//! with the checks of `message_spec`, renders one module per message, and, for the
//! protocol crate, `ErrorCode` and docs/errors.md from `Errors.java` ([`errors`]). It
//! formats each module with the pinned toolchain's `rustfmt` and writes them in sorted
//! order with the `mod.rs` files, removing any file it no longer generates. The output
//! depends only on its inputs and the generator: no timestamps or absolute paths, and
//! the same bytes on every run.
//!
//! The overlays in `xtask/codegen/` ([`overlays`]) add what the schemas don't say:
//! which integer fields have typed values, and which fields hold secrets.
//!
//! `cargo xtask codegen --check` ([`check`]) renders the same files in memory and fails,
//! naming each stale file and its first differing line, when the committed ones differ.
//! Both first check the vendored Kafka directory against its manifest, so a vendored file
//! changed without `schemas vendor` fails here as well as in `schemas verify`. How each
//! schema feature maps to Rust is in docs/protocol/codegen.md.

mod api_keys;
mod decode;
mod docs;
mod encode;
mod errors;
mod fuzz_targets;
mod internal;
mod json;
pub(crate) mod names;
mod overlays;
mod strategy;
mod structs;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::Exit;
use crate::message_spec::{self, MessageKind};
use crate::output;
use crate::schemas;

/// What one run generates, and from what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Filter {
    /// The kinds of message to generate; other schemas are skipped.
    pub(crate) kinds: Vec<MessageKind>,
    /// The path the generated code uses for `alluvium-protocol`'s types: `crate` inside
    /// that crate, `alluvium_protocol` elsewhere.
    pub(crate) runtime: String,
    /// Whether to generate `ErrorCode` too, and from where.
    pub(crate) error_codes: Option<ErrorCodes>,
    /// Whether to generate `ApiKey` too, and from where.
    pub(crate) api_keys: Option<ApiKeys>,
    /// Where to write the generated fuzz targets' package, if anywhere.
    pub(crate) fuzz_targets: Option<PathBuf>,
    /// Whether the schemas are Alluvium's internal APIs, whose rules codegen then checks.
    pub(crate) internal: Option<internal::Internal>,
    /// Whether the root module gets `TRANSCODE`, the table through which the golden
    /// vector tests decode and encode each message by its schema name.
    pub(crate) golden_tests: bool,
    /// The directory of the overlay files (`overlays`) applied to the messages, if any.
    pub(crate) overlays: Option<PathBuf>,
    /// The doc comment of the output directory's `mod.rs`: what the directory holds.
    pub(crate) doc: String,
}

/// The inputs and outputs of `ErrorCode`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ErrorCodes {
    /// The directory holding `Errors.java` and the exception classes.
    pub(crate) source: PathBuf,
    /// The Markdown page listing every code.
    pub(crate) page: PathBuf,
}

/// The inputs and outputs of `ApiKey`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ApiKeys {
    /// The registry recorded from Kafka's `ApiKeys`, which holds the flags the schemas
    /// don't state.
    pub(crate) recorded: PathBuf,
    /// The Markdown page listing every API.
    pub(crate) page: PathBuf,
}

/// The vendored Kafka directory the protocol crate is generated from.
const VENDORED: &str = "schemas/kafka-4.4.0";

/// The protocol crate's generated code: every request, response, header and data type
/// in Kafka's `clients` schemas, and `ErrorCode`.
fn protocol() -> (Vec<PathBuf>, PathBuf, Filter) {
    let root = crate::repo_root();
    (
        vec![root.join(VENDORED).join("clients")],
        root.join("crates/alluvium-protocol/src/generated"),
        Filter {
            kinds: vec![
                MessageKind::Request,
                MessageKind::Response,
                MessageKind::Header,
                MessageKind::Data,
            ],
            runtime: "crate".to_owned(),
            error_codes: Some(ErrorCodes {
                source: root.join(VENDORED).join("errors"),
                page: root.join("docs/errors.md"),
            }),
            golden_tests: true,
            fuzz_targets: Some(root.join("fuzz/generated")),
            internal: None,
            api_keys: Some(ApiKeys {
                recorded: root.join(crate::golden::API_KEYS_GOLDEN).join("api_keys.json"),
                page: root.join("docs/protocol/apis.md"),
            }),
            overlays: Some(root.join("xtask/codegen")),
            doc: "Types generated from Kafka's sources: one module per message schema, \
                  `api_key` from the request and response schemas, and `error_code` from \
                  `Errors.java` \
                  ([ADR-0008](../../../../docs/adr/0008-protocol-code-generated-from-kafka-schemas.md))."
                .to_owned(),
        },
    )
}

/// Alluvium's own schemas, for its internal APIs (ADR-0021).
const INTERNAL_SCHEMAS: &str = "schemas/alluvium/messages";

/// The protocol crate's internal APIs: Alluvium's own request and response schemas, kept
/// apart from Kafka's so that neither `ApiKey` nor an `ApiVersions` response includes
/// them.
fn internal_apis() -> (Vec<PathBuf>, PathBuf, Filter) {
    let root = crate::repo_root();
    (
        vec![root.join(INTERNAL_SCHEMAS)],
        root.join("crates/alluvium-protocol/src/internal"),
        Filter {
            kinds: vec![
                MessageKind::Request,
                MessageKind::Response,
                MessageKind::Data,
            ],
            runtime: "crate".to_owned(),
            error_codes: None,
            api_keys: None,
            golden_tests: false,
            fuzz_targets: None,
            internal: Some(internal::Internal {
                kafka: vec![root.join(VENDORED).join("clients")],
                declared_in: Some(root.join("crates/alluvium-protocol/src/lib.rs")),
            }),
            overlays: None,
            doc: "Alluvium's internal APIs, generated from `schemas/alluvium/messages/`: \
                  requests between Alluvium nodes, never advertised to or accepted from Kafka \
                  clients ([ADR-0021](../../../../docs/adr/0021-alluvium-internal-api-keys.md))."
                .to_owned(),
        },
    )
}

/// The protocol crate's test types: Kafka's generator test schemas, each combining many
/// schema features in one small message, and a sample internal API. Compiled only for
/// tests.
fn test_messages() -> (Vec<PathBuf>, PathBuf, Filter) {
    let root = crate::repo_root();
    (
        vec![
            root.join(VENDORED).join("clients-test"),
            root.join("schemas/alluvium/test"),
        ],
        root.join("crates/alluvium-protocol/src/test_messages"),
        Filter {
            kinds: vec![
                MessageKind::Header,
                MessageKind::Request,
                MessageKind::Response,
            ],
            runtime: "crate".to_owned(),
            error_codes: None,
            api_keys: None,
            golden_tests: false,
            fuzz_targets: None,
            internal: Some(internal::Internal {
                kafka: vec![root.join(VENDORED).join("clients")],
                declared_in: None,
            }),
            overlays: None,
            doc: "Message types generated from the schemas Kafka's own generator tests use \
                  (`schemas/kafka-4.4.0/clients-test/`), and from a sample internal API \
                  (`schemas/alluvium/test/`). They exercise every schema feature in small \
                  messages, for the tests of the generated code; nothing else uses them."
                .to_owned(),
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
    /// The error codes in the generated `ErrorCode`.
    pub(crate) error_codes: usize,
    /// The APIs in the generated `ApiKey`.
    pub(crate) api_keys: usize,
}

/// The most problems `--check` prints before summarizing the rest.
const SHOWN_PROBLEMS: usize = 10;

pub(crate) fn run(check_only: bool, out: &mut impl Write) -> Exit {
    if let Err(error) = verify_vendored() {
        output::line(out, &format!("error: {error}"));
        return Exit::CheckFailed;
    }
    let mut problems = Vec::new();
    for (inputs, out_dir, filter) in [protocol(), internal_apis(), test_messages()] {
        let result = if check_only {
            check(&inputs, &out_dir, &filter)
        } else {
            generate(&inputs, &out_dir, &filter).map(|report| (report, Vec::new()))
        };
        match result {
            Ok((report, found)) if found.is_empty() => {
                output::line(out, &summary(&report, &out_dir, check_only));
            }
            Ok((_, found)) => problems.extend(found),
            Err(error) => {
                output::line(out, &format!("error: {error}"));
                return Exit::CheckFailed;
            }
        }
    }
    if problems.is_empty() {
        return Exit::Success;
    }
    for problem in problems.iter().take(SHOWN_PROBLEMS) {
        output::line(out, problem);
    }
    if let Some(more) = problems
        .len()
        .checked_sub(SHOWN_PROBLEMS)
        .filter(|n| *n > 0)
    {
        output::line(out, &format!("... and {more} more"));
    }
    output::line(
        out,
        &format!(
            "codegen --check: {} file(s) differ from the generated output; run `cargo xtask \
             codegen` and commit the result (generated files are never edited by hand)",
            problems.len()
        ),
    );
    Exit::CheckFailed
}

/// The line `run` prints for one output directory.
fn summary(report: &Report, out_dir: &Path, check_only: bool) -> String {
    let verb = if check_only {
        "up to date"
    } else {
        "generated"
    };
    let error_codes = if report.error_codes == 0 {
        String::new()
    } else {
        format!(" and {} error codes", report.error_codes)
    };
    let api_keys = if report.api_keys == 0 {
        String::new()
    } else {
        format!(", {} API keys", report.api_keys)
    };
    let deepest = report
        .deepest
        .as_ref()
        .map_or_else(String::new, |(name, depth)| {
            format!(
                "; deepest {name} ({depth} levels, limit {})",
                structs::MAX_NESTING_DEPTH
            )
        });
    format!(
        "codegen: {verb}: {} message modules{api_keys}{error_codes} in {}{deepest}",
        report.modules,
        shown_path(out_dir)
    )
}

/// Checks the vendored Kafka directory against its manifest.
fn verify_vendored() -> Result<(), String> {
    let (problems, _) = schemas::verify_dir(&crate::repo_root().join(VENDORED), VENDORED)?;
    match problems.first() {
        None => Ok(()),
        Some(first) => Err(format!(
            "{first} ({} problem(s); `cargo xtask schemas verify` lists them)",
            problems.len()
        )),
    }
}

/// Generates the modules for the schemas in `inputs` that `filter` selects into
/// `out_dir`, with `ErrorCode` and its page if the filter asks for them, and removes the
/// files there it no longer generates.
pub(crate) fn generate(
    inputs: &[PathBuf],
    out_dir: &Path,
    filter: &Filter,
) -> Result<Report, String> {
    let rendered = render(inputs, filter)?;
    if let Some(internal) = &filter.internal {
        internal::check_declared(internal, !rendered.files.is_empty())?;
    }
    write(out_dir, &rendered.files)?;
    if let Some(fuzz_dir) = &filter.fuzz_targets {
        write(fuzz_dir, &rendered.fuzz)?;
    }
    for (page, contents) in &rendered.pages {
        if std::fs::read_to_string(page).is_ok_and(|existing| existing == *contents) {
            continue;
        }
        std::fs::write(page, contents)
            .map_err(|error| format!("could not write {}: {error}", page.display()))?;
    }
    Ok(rendered.report)
}

/// What `generate` would report, and a line for each file that differs from what it
/// would write: changed, missing, or no longer generated.
pub(crate) fn check(
    inputs: &[PathBuf],
    out_dir: &Path,
    filter: &Filter,
) -> Result<(Report, Vec<String>), String> {
    let rendered = render(inputs, filter)?;
    if let Some(internal) = &filter.internal {
        internal::check_declared(internal, !rendered.files.is_empty())?;
    }
    let mut problems = Vec::new();
    let expected = rendered
        .files
        .iter()
        .map(|(file, contents)| (out_dir.join(file), contents))
        .chain(
            rendered
                .pages
                .iter()
                .map(|(page, contents)| (page.clone(), contents)),
        );
    for (path, contents) in expected {
        if let Some(problem) = difference(&path, contents) {
            problems.push(problem);
        }
    }
    let mut outputs = vec![(out_dir.to_path_buf(), &rendered.files)];
    if let Some(fuzz_dir) = &filter.fuzz_targets {
        for (file, contents) in &rendered.fuzz {
            if let Some(problem) = difference(&fuzz_dir.join(file), contents) {
                problems.push(problem);
            }
        }
        outputs.push((fuzz_dir.clone(), &rendered.fuzz));
    }
    for (dir, files) in outputs {
        for stale in existing_files(&dir)? {
            let relative = stale.strip_prefix(&dir).unwrap_or(&stale);
            if !files.contains_key(relative) {
                problems.push(format!(
                    "{}: no longer generated, so codegen would remove it",
                    shown_path(&stale)
                ));
            }
        }
    }
    Ok((rendered.report, problems))
}

/// How the file at `path` differs from `expected`, if it does: its first differing
/// line, both ways.
fn difference(path: &Path, expected: &str) -> Option<String> {
    let shown = shown_path(path);
    let Ok(actual) = std::fs::read_to_string(path) else {
        return Some(format!("{shown}: missing"));
    };
    if actual == expected {
        return None;
    }
    let mut actual_lines = actual.lines();
    let mut expected_lines = expected.lines();
    let mut line = 1;
    loop {
        match (actual_lines.next(), expected_lines.next()) {
            (Some(found), Some(wanted)) if found == wanted => line += 1,
            (found, wanted) => {
                return Some(format!(
                    "{shown}:{line}: found {:?}, generated {:?}",
                    found.unwrap_or("(end of file)"),
                    wanted.unwrap_or("(end of file)")
                ));
            }
        }
    }
}

/// Everything one run writes.
struct Rendered {
    /// The generated modules and `mod.rs` files, by path within the output directory.
    files: BTreeMap<PathBuf, String>,
    /// Generated pages outside the output directory, by full path.
    pages: BTreeMap<PathBuf, String>,
    /// The fuzz targets' package, by path within `Filter::fuzz_targets`.
    fuzz: BTreeMap<PathBuf, String>,
    report: Report,
}

/// Every file `generate` would write.
/// The generated fuzz targets' package for `messages`, formatted, if `filter` asks for
/// one.
fn fuzz_package(
    inputs: &[PathBuf],
    filter: &Filter,
    messages: &[(PathBuf, String, message_spec::MessageSpec)],
) -> Result<BTreeMap<PathBuf, String>, String> {
    if filter.fuzz_targets.is_none() {
        return Ok(BTreeMap::new());
    }
    let messages: Vec<_> = messages.iter().map(|(_, _, message)| message).collect();
    let sources: Vec<String> = inputs.iter().map(|input| shown_path(input)).collect();
    let header = header(
        &format!("from the schemas in {}", sources.join(", ")),
        from_kafka(inputs),
    );
    fuzz_targets::render(&messages, &header)
        .into_iter()
        .map(|(file, source)| {
            if file.extension().is_some_and(|extension| extension == "rs") {
                Ok((file, rustfmt(&source)?))
            } else {
                Ok((file, source))
            }
        })
        .collect()
}

/// A loaded schema: its path, the path shown in messages, and the message.
type Loaded = (PathBuf, String, message_spec::MessageSpec);

/// The schemas in `inputs` that `filter` generates, checked against the internal API
/// rules if it asks for them, and every request and response, versions or not, since
/// the registry keeps removed APIs.
fn load(
    inputs: &[PathBuf],
    filter: &Filter,
) -> Result<(Vec<Loaded>, Vec<message_spec::MessageSpec>), String> {
    let mut messages = Vec::new();
    let mut apis = Vec::new();
    // Alluvium's own schemas, generated or not, for the internal API rules.
    let mut own = Vec::new();
    for path in schema_files(inputs)? {
        let shown = shown_path(&path);
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("could not read {shown}: {error}"))?;
        let message = message_spec::load(&shown, &text).map_err(|error| error.to_string())?;
        if matches!(message.kind, MessageKind::Request | MessageKind::Response) {
            apis.push(message.clone());
        }
        if kafka_version(&path).is_empty() {
            own.push((path.clone(), shown.clone(), message.clone()));
        }
        // Parity: generator/src/main/java/org/apache/kafka/message/MessageGenerator.java@4.4.0 MessageGenerator.generateAndWriteMessageClasses
        // Kafka generates nothing for a message without valid versions.
        if filter.kinds.contains(&message.kind) && !message.valid_versions.is_empty() {
            messages.push((path, shown, message));
        }
    }
    if let Some(internal) = &filter.internal {
        internal::check(&own, &internal::kafka_keys(&internal.kafka)?)?;
    }
    Ok((messages, apis))
}

fn render(inputs: &[PathBuf], filter: &Filter) -> Result<Rendered, String> {
    let mut modules: BTreeMap<PathBuf, String> = BTreeMap::new();
    let mut pages = BTreeMap::new();
    let mut report = Report {
        modules: 0,
        deepest: None,
        error_codes: 0,
        api_keys: 0,
    };
    let (messages, apis) = load(inputs, filter)?;
    let overlays = match &filter.overlays {
        Some(dir) => load_overlays(dir, &messages)?,
        None => overlays::Overlays::default(),
    };
    // Each message's path from the output directory's root module, for the test that
    // decodes arbitrary input as every message.
    let mut decodable = Vec::new();
    for (path, shown, message) in &messages {
        let module = structs::module(message, &filter.runtime, &overlays)
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
        let module_path = directory
            .join(names::identifier(&message.name))
            .display()
            .to_string()
            .replace(['/', '\\'], "::");
        decodable.push((
            format!("{module_path}::{}", message.name),
            message.valid_versions.lowest(),
            message.valid_versions.highest(),
        ));
        let source = format!(
            "{}\n{}\n{}",
            header(
                &format!("from {shown}{}", kafka_version(path)),
                !kafka_version(path).is_empty()
            ),
            docs::wrap(&module_doc(message), "//! "),
            module.source
        );
        if modules.insert(file.clone(), source).is_some() {
            return Err(format!("two schemas generate {}", file.display()));
        }
        report.modules += 1;
    }
    if let Some(error_codes) = &filter.error_codes {
        let (module, page, count) = error_code_module(error_codes)?;
        if modules
            .insert(PathBuf::from("error_code.rs"), module)
            .is_some()
        {
            return Err("a schema generates error_code.rs".to_owned());
        }
        pages.insert(error_codes.page.clone(), page);
        report.error_codes = count;
    }
    if let Some(api_keys) = &filter.api_keys {
        let (module, page, count) =
            api_key_module(inputs, &api_keys.recorded, &apis, &filter.runtime)?;
        if modules
            .insert(PathBuf::from("api_key.rs"), module)
            .is_some()
        {
            return Err("a schema generates api_key.rs".to_owned());
        }
        pages.insert(api_keys.page.clone(), page);
        report.api_keys = count;
    }
    let fuzz = fuzz_package(inputs, filter, &messages)?;
    let mut files = mod_files(
        &modules,
        &filter.doc,
        &decodable,
        filter.golden_tests,
        from_kafka(inputs),
    );
    files.extend(modules);
    let mut formatted = BTreeMap::new();
    for (file, source) in files {
        formatted.insert(file, rustfmt(&source)?);
    }
    Ok(Rendered {
        files: formatted,
        pages,
        fuzz,
        report,
    })
}

/// The overlays in `dir`, checked against the `messages` generated.
fn load_overlays(
    dir: &Path,
    messages: &[(PathBuf, String, message_spec::MessageSpec)],
) -> Result<overlays::Overlays, String> {
    let overlays = overlays::Overlays::load(dir)
        .map_err(|problem| format!("{}/{problem}", shown_path(dir)))?;
    let specs: Vec<_> = messages
        .iter()
        .map(|(_, _, message)| message.clone())
        .collect();
    overlays.check(&specs)?;
    Ok(overlays)
}

/// The `error_code` module and the page listing every code, from `Errors.java`, and the
/// number of codes.
fn error_code_module(error_codes: &ErrorCodes) -> Result<(String, String, usize), String> {
    let entries = errors::extract(&error_codes.source)
        .map_err(|problem| format!("{}: {problem}", shown_path(&error_codes.source)))?;
    let source = shown_path(&error_codes.source.join("Errors.java"));
    let module = format!(
        "{}\n{}",
        header(
            &format!("from {source}{}", kafka_version(&error_codes.source)),
            true
        ),
        errors::module(&entries)
    );
    Ok((module, errors::page(&entries, &source), entries.len()))
}

/// The `api_key` module and the page listing every API, from the requests and responses
/// in `messages` and the registry `recorded` from Kafka, and the number of APIs.
fn api_key_module(
    inputs: &[PathBuf],
    recorded: &Path,
    messages: &[message_spec::MessageSpec],
    runtime: &str,
) -> Result<(String, String, usize), String> {
    let flags = api_keys::flags(recorded)?;
    let apis = api_keys::collect(messages, &flags)?;
    let inputs: Vec<String> = inputs.iter().map(|input| shown_path(input)).collect();
    let sources = format!(
        "the request and response schemas in {} and {}",
        inputs.join(", "),
        shown_path(recorded)
    );
    let module = format!(
        "{}\n{}",
        header(&format!("from {sources}"), true),
        api_keys::module(&apis, runtime)
    );
    Ok((module, api_keys::page(&apis, &sources), apis.len()))
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
    // A directory codegen hasn't written yet is shown through its parent.
    let canonical = path
        .canonicalize()
        .or_else(|_| {
            let parent = path.parent().unwrap_or(path).canonicalize()?;
            Ok::<_, std::io::Error>(parent.join(path.file_name().unwrap_or_default()))
        })
        .unwrap_or_else(|_| path.to_path_buf());
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

/// The comment every generated file starts with. Code generated from Kafka's sources
/// notes their license; Alluvium's own schemas are under the project's.
fn header(source: &str, from_kafka: bool) -> String {
    let license = if from_kafka {
        " (Apache License 2.0; see NOTICE)"
    } else {
        ""
    };
    docs::wrap(
        &format!(
            "@generated by `cargo xtask codegen` {source}{license}. Do not edit: change the \
             schema or the generator in xtask/src/codegen/."
        ),
        "// ",
    )
}

/// Whether any of `inputs` is a vendored Kafka directory.
fn from_kafka(inputs: &[PathBuf]) -> bool {
    inputs.iter().any(|input| !kafka_version(input).is_empty())
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
fn mod_files(
    modules: &BTreeMap<PathBuf, String>,
    root_doc: &str,
    decodable: &[(String, i16, i16)],
    golden_tests: bool,
    from_kafka: bool,
) -> BTreeMap<PathBuf, String> {
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
                root_doc
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
            let tests = if directory.as_os_str().is_empty() {
                let mut tests = decode_every_version(decodable);
                if golden_tests {
                    tests.push_str(&transcode_table(decodable));
                }
                tests
            } else {
                String::new()
            };
            (
                directory.join("mod.rs"),
                format!(
                    "{}\n{}\n{declarations}{tests}",
                    header("", from_kafka),
                    docs::wrap(doc, "//! ")
                ),
            )
        })
        .collect()
}

/// The test-only tables of each message's schema name with `golden_tests::transcode` for
/// its type, and with its versions and a function encoding arbitrary values, for the
/// golden vector tests and the reverse check.
fn transcode_table(decodable: &[(String, i16, i16)]) -> String {
    let mut transcode = String::new();
    let mut arbitrary = String::new();
    let mut to_json = String::new();
    for (message, lowest, highest) in decodable {
        let name = message.rsplit("::").next().unwrap_or(message);
        let _ = writeln!(
            transcode,
            "    (\"{name}\", crate::golden_tests::transcode::<{message}>),"
        );
        let _ = writeln!(
            to_json,
            "    (\"{name}\", crate::golden_tests::to_json::<{message}>),"
        );
        let _ = writeln!(
            arbitrary,
            "    (\"{name}\", {lowest}, {highest}, |version, count| \
             crate::golden_tests::encode_arbitrary(&{message}::arbitrary(version), version, \
             count)),"
        );
    }
    format!(
        "\n/// Each message's schema name with the function that decodes and re-encodes it, for \
         the\n/// golden vector tests.\n#[cfg(test)]\npub(crate) const TRANSCODE: &[(&str, \
         crate::golden_tests::Transcode)] = &[\n{transcode}];\n\n/// Each message's schema \
         name, lowest and highest version, with a function that encodes\n/// `count` arbitrary \
         values at a version, for the reverse check by Kafka.\n#[cfg(test)]\npub(crate) const \
         ENCODE_ARBITRARY: &[(&str, i16, i16, crate::golden_tests::EncodeArbitrary)] = \
         &[\n{arbitrary}];\n\n/// Each message's schema name with the function that decodes it and \
         writes it as JSON,\n/// for the comparison with Kafka's `JsonConverter`.\n#[cfg(test)]\n\
         pub(crate) const TO_JSON: &[(&str, crate::golden_tests::ToJson)] = &[\n{to_json}];\n"
    )
}

/// Test-only tables over every message: functions that each decode their input as one
/// message at every version, so a property test can show that no input makes a decoder
/// panic; and functions that each encode one message's default at every version and
/// decode it back. Each entry is `module::Message` with its lowest and highest version.
fn decode_every_version(decodable: &[(String, i16, i16)]) -> String {
    if decodable.is_empty() {
        return String::new();
    }
    let entries = |function: &str| {
        decodable
            .iter()
            .fold(String::new(), |mut entries, (message, lowest, highest)| {
                let _ = writeln!(entries, "    {function}::<{message}, {lowest}, {highest}>,");
                entries
            })
    };
    format!(
        "\n/// Functions that decode their input as one message at each of its versions, for \
         the test\n/// that no input makes a decoder panic.\n#[cfg(test)]\npub(crate) const \
         DECODE_EVERY_VERSION: &[fn(&bytes::Bytes)] = &[\n{}];\n\n#[cfg(test)]\nfn \
         decode_at_every_version<M: crate::Message, const LOWEST: i16, const HIGHEST: \
         i16>(input: &bytes::Bytes) {{\n    for version in LOWEST..=HIGHEST {{\n        // \
         Any result will do: the test is that decoding returns.\n        let _ = \
         M::decode(&mut crate::Reader::new(input.clone()), version);\n    }}\n}}\n\n/// \
         Functions that encode one message's default at each of its versions, check its \
         size,\n/// and decode it back.\n#[cfg(test)]\npub(crate) const \
         ROUND_TRIP_DEFAULT_EVERY_VERSION: &[fn()] = &[\n{}];\n\n#[cfg(test)]\nfn \
         round_trip_default<M: crate::Message + Default + PartialEq + std::fmt::Debug, const \
         LOWEST: i16, const HIGHEST: i16>() {{\n    crate::encode_tests::round_trip::<M>(&M::default(), \
         LOWEST..=HIGHEST);\n}}\n",
        entries("decode_at_every_version"),
        entries("round_trip_default")
    )
}

/// `source` formatted by the pinned toolchain's `rustfmt`, with every width heuristic at
/// the full line width so that a field's read or default fits on one line. `cargo fmt`
/// skips the generated modules (`#[rustfmt::skip]` where they are declared), so it
/// doesn't undo this; `codegen --check` keeps them exactly as generated.
fn rustfmt(source: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024", "--config", "use_small_heuristics=Max"])
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

    use super::{
        Filter, VENDORED, check, existing_files, generate, internal_apis, names, protocol, render,
        schema_files, test_messages,
    };
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
            error_codes: None,
            api_keys: None,
            golden_tests: false,
            fuzz_targets: None,
            internal: None,
            overlays: None,
            doc: "Test records.".to_owned(),
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

        let (_, problems) = check(std::slice::from_ref(&inputs), &out_dir, &filter).unwrap();
        assert_eq!(problems, Vec::<String>::new());
        let record = out_dir.join("test_record.rs");
        let edited = first[Path::new("test_record.rs")].replace("epoch: -1,", "epoch: 0,");
        std::fs::write(&record, edited).unwrap();
        std::fs::write(out_dir.join("extra.rs"), "").unwrap();
        let (_, problems) = check(std::slice::from_ref(&inputs), &out_dir, &filter).unwrap();
        assert_eq!(problems.len(), 2, "{problems:?}");
        assert!(
            problems[0].contains("test_record.rs:") && problems[0].contains("epoch: 0,"),
            "{problems:?}"
        );
        assert!(problems[1].ends_with("extra.rs: no longer generated, so codegen would remove it"));
        std::fs::remove_dir_all(inputs).unwrap();
        std::fs::remove_dir_all(out_dir).unwrap();
    }

    // Ported from: generator/src/test/java/org/apache/kafka/message/MessageGeneratorTest.java@4.4.0
    // (testToSnakeCase, testGenerateAndWriteMessageClasses).
    #[test]
    fn names_become_snake_case_as_in_kafka() {
        assert_eq!(names::snake_case(""), "");
        assert_eq!(names::snake_case("FooBarBaz"), "foo_bar_baz");
        assert_eq!(names::snake_case("fooBarBaz"), "foo_bar_baz");
        assert_eq!(names::snake_case("FORTRAN"), "fortran");
    }

    #[test]
    fn a_message_without_valid_versions_generates_nothing() {
        let schema = |kind: &str, valid: &str| {
            let (name, listeners) = if kind == "request" {
                ("Request", r#""listeners": ["broker"],"#)
            } else {
                ("Response", "")
            };
            format!(
                r#"{{ "apiKey": 0, "type": "{kind}", {listeners} "name": "FooBar{name}",
                     "validVersions": "{valid}", "flexibleVersions": "none",
                     "fields": [ {{ "name": "field1", "type": "int32", "versions": "0+" }} ] }}"#
            )
        };
        let filter = Filter {
            kinds: vec![MessageKind::Request, MessageKind::Response],
            runtime: "crate".to_owned(),
            error_codes: None,
            api_keys: None,
            golden_tests: false,
            fuzz_targets: None,
            internal: None,
            overlays: None,
            doc: "Test messages.".to_owned(),
        };
        for (valid, expected) in [
            ("none", Vec::<&str>::new()),
            (
                "0-2",
                vec!["foo_bar_request.rs", "foo_bar_response.rs", "mod.rs"],
            ),
        ] {
            let inputs = scratch("no-versions");
            let out_dir = scratch("no-versions-out");
            for kind in ["request", "response"] {
                std::fs::write(inputs.join(format!("{kind}.json")), schema(kind, valid)).unwrap();
            }
            let report = generate(std::slice::from_ref(&inputs), &out_dir, &filter).unwrap();
            let files = contents(&out_dir);
            let names: Vec<&str> = files.keys().filter_map(|path| path.to_str()).collect();
            assert_eq!(names, expected, "{valid}");
            assert_eq!(report.modules, expected.len().saturating_sub(1), "{valid}");
            std::fs::remove_dir_all(inputs).unwrap();
            std::fs::remove_dir_all(out_dir).unwrap();
        }
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
            FieldDefault::Uuid(expected) if *expected == Uuid::ZERO => {
                value == "alluvium_types::Uuid::ZERO"
            }
            FieldDefault::Uuid(expected) => value
                .strip_prefix("alluvium_types::Uuid::from_bits(")
                .and_then(|rest| rest.strip_suffix(",)").or_else(|| rest.strip_suffix(')')))
                .and_then(|bits| bits.split_once(','))
                .is_some_and(|(most, least)| {
                    let parse = |text: &str| text.trim().replace('_', "").parse::<i64>().ok();
                    parse(most) == Some(expected.most_significant_bits())
                        && parse(least) == Some(expected.least_significant_bits())
                }),
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
        let defaults = literal_fields(block, name);
        for field in fields {
            let identifier = names::identifier(&field.name);
            let expression = defaults
                .iter()
                .find_map(|(label, expression)| (*label == identifier).then_some(expression))
                .unwrap_or_else(|| panic!("{name}.{} is not in Default", field.name));
            assert!(
                is_schema_default(field, expression),
                "{name}.{}: {expression}",
                field.name
            );
        }
    }

    /// The fields of the `name { ... }` literal that `fn default` in `code` returns, as
    /// label and expression. Lines are joined first, which keeps string literals whole:
    /// rustfmt never splits them.
    fn literal_fields(code: &str, name: &str) -> Vec<(String, String)> {
        let joined: String = code.lines().map(str::trim).collect();
        let body = joined
            .split_once(&format!("-> {name} {{"))
            .and_then(|(_, after)| after.split_once(&format!("{name} {{")))
            .map_or_else(|| panic!("no {name} literal"), |(_, body)| body);
        let mut fields = Vec::new();
        let (mut depth, mut quoted, mut start) = (0_i32, false, 0);
        for (at, c) in body.char_indices() {
            match c {
                '"' => quoted = !quoted,
                _ if quoted => {}
                '}' if depth == 0 => break,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    let field = body.get(start..at).unwrap();
                    let (label, expression) = field.split_once(':').unwrap();
                    fields.push((label.trim().to_owned(), expression.trim().to_owned()));
                    start = at + 1;
                }
                _ => {}
            }
        }
        fields
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
            if registered.has_keys() {
                assert!(
                    source.contains(&format!("impl {name} {{\n    /// The first of `elements`")),
                    "keyed struct {name} has no find"
                );
            }
            check_structs(source, registry, registered.fields, flexible_versions);
        }
    }

    #[test]
    fn committed_modules_match_the_schema_model() {
        for ((inputs, out_dir, filter), expected) in [
            (protocol(), 195),
            (internal_apis(), 0),
            (test_messages(), 8),
        ] {
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
            assert_eq!(checked, expected, "{}", out_dir.display());
        }
    }

    #[test]
    fn output_is_independent_of_input_order_and_names_no_absolute_path() {
        let (mut inputs, _, mut filter) = test_messages();
        // Adds RequestHeader and ResponseHeader, so the inputs interleave when sorted; only
        // headers, since Kafka's requests aren't internal APIs.
        inputs.push(crate::repo_root().join(VENDORED).join("clients"));
        inputs.retain(|input| !input.ends_with("schemas/alluvium/test"));
        filter.kinds.retain(|kind| *kind == MessageKind::Header);
        let first = render(&inputs, &filter).unwrap();
        inputs.reverse();
        let second = render(&inputs, &filter).unwrap();
        assert_eq!(first.files, second.files);
        assert_eq!(first.files.len(), 9, "8 headers and mod.rs");
        let root = crate::repo_root().display().to_string();
        for (file, source) in &first.files {
            for absolute in [root.as_str(), "/Users/", "/home/", ":\\"] {
                assert!(!source.contains(absolute), "{}: {absolute}", file.display());
            }
        }
    }

    #[test]
    fn generated_code_suppresses_no_lint() {
        let files = [protocol(), internal_apis(), test_messages()]
            .into_iter()
            .flat_map(|(_, out_dir, _)| existing_files(&out_dir).unwrap());
        for file in files {
            let source = std::fs::read_to_string(&file).unwrap();
            for suppression in ["#[allow", "#![allow", "#[expect", "#![expect"] {
                assert!(
                    !source.contains(suppression),
                    "{}: {suppression}",
                    file.display()
                );
            }
        }
    }

    /// Generated decoders allocate only through the `Reader`, which checks every declared
    /// size against its `DecodeLimits` and charges the allocation budget first
    /// (PROTO-0112): no generated code allocates on its own.
    #[test]
    fn generated_code_allocates_only_through_the_reader() {
        let files = [protocol(), internal_apis(), test_messages()]
            .into_iter()
            .flat_map(|(_, out_dir, _)| existing_files(&out_dir).unwrap());
        for file in files {
            let source = std::fs::read_to_string(&file).unwrap();
            for allocation in [
                "with_capacity",
                "reserve(",
                "vec![",
                ".to_vec()",
                "from_utf8",
                "Box::new",
                ".collect",
                ".extend(",
                ".push(",
            ] {
                assert!(
                    !source.contains(allocation),
                    "{}: {allocation}",
                    file.display()
                );
            }
        }
    }
}
