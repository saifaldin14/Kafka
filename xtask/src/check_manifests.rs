//! `cargo xtask check-manifests`: the workspace configuration matches the standards.
//!
//! Rules:
//!
//! 1. `[workspace.lints]` in the root `Cargo.toml` equals the lint block in
//!    docs/standards/rust.md#workspace-configuration.
//! 2. `clippy.toml` equals the clippy.toml block there.
//! 3. Every workspace member declares `[lints] workspace = true`, so it inherits the
//!    lints, and `license-file.workspace = true`, so it carries the repository's
//!    license ([ADR-0015](../../docs/adr/0015-project-license-and-contributor-terms.md)).
//! 4. Every member declares each dependency as `{ workspace = true }`, adding only
//!    `features` or `optional`: versions and sources live once, in the root
//!    `[workspace.dependencies]` (docs/standards/rust.md#dependencies). Workspace
//!    crates that depend on each other are listed there too, so there are no
//!    exceptions.
//! 5. Every `[advisories] ignore` entry in deny.toml is a table with an `id`, a
//!    `reason` and an `expiry` (docs/standards/security.md#dependencies). cargo-deny
//!    itself rejects a malformed date and reports the advisory again once the date
//!    passes; it does not require either field.
//! 6. Every member's crate root (`src/lib.rs`, or `src/main.rs` for a binary-only
//!    crate) starts with a `//!` doc that has a `Layer:` line naming the crate's layer
//!    from the layer table, and a `Kafka counterpart:` line
//!    (docs/standards/rust.md#modules-and-documentation).
//! 7. Every file in [`DECODER_FILES`] whose crate is a workspace member denies
//!    `clippy::indexing_slicing` and `clippy::arithmetic_side_effects` with an inner
//!    attribute, because it decodes untrusted bytes
//!    (docs/standards/rust.md#workspace-configuration). Cargo can't add lints to a
//!    crate that inherits `[workspace.lints]`, so the attribute is the only way, and
//!    nothing else would notice it missing. A crate that doesn't exist yet is skipped.
//!
//! Files are parsed and compared as TOML values: comments, formatting, and the order of
//! keys and of array entries don't matter. Every difference is reported in one run,
//! naming the file and the dotted key.

use std::io::Write;
use std::path::Path;

use serde_json::Value;

use crate::Exit;
use crate::graph;
use crate::layers;
use crate::output;

/// The standard that the manifests must match, relative to the repository root.
const RUST_MD: &str = "docs/standards/rust.md";

/// The rust.md section that holds the configuration blocks.
const SECTION: &str = "## Workspace configuration";

/// The configuration blocks from rust.md, parsed.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Standard {
    /// The `workspace.lints` table of the lint block.
    pub(crate) lints: Value,
    /// The whole clippy.toml block.
    pub(crate) clippy: Value,
}

/// The files that decode untrusted bytes, by crate and path within the crate. Each must
/// deny the lints in [`DECODER_LINTS`]. A storage module that decodes segments or
/// indexes is added here in the change that creates it.
pub(crate) const DECODER_FILES: &[(&str, &str)] = &[
    ("alluvium-protocol", "src/lib.rs"),
    ("alluvium-records", "src/lib.rs"),
];

/// The lints a decoder file must deny.
const DECODER_LINTS: [&str; 2] = [
    "clippy::indexing_slicing",
    "clippy::arithmetic_side_effects",
];

/// Runs the command.
pub(crate) fn run(out: &mut impl Write) -> Exit {
    let problems = match check_repository(&crate::repo_root()) {
        Ok(problems) => problems,
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            return Exit::CheckFailed;
        }
    };
    if problems.is_empty() {
        output::line(out, "check-manifests: ok");
        return Exit::Success;
    }
    for problem in &problems {
        output::line(out, problem);
    }
    output::line(
        out,
        &format!(
            "check-manifests: {} problem(s). Each names the standard it breaks; to change a \
             rule, change the standard and the files together",
            problems.len()
        ),
    );
    Exit::CheckFailed
}

/// Checks every rule against the files in `root`.
fn check_repository(root: &Path) -> Result<Vec<String>, String> {
    let read = |file: &str| {
        std::fs::read_to_string(root.join(file))
            .map_err(|error| format!("could not read {file}: {error}"))
    };
    let standard = standard(&read(RUST_MD)?)?;
    let mut problems = check_lints(&parse("Cargo.toml", &read("Cargo.toml")?)?, &standard);
    problems.extend(check_clippy(
        &parse("clippy.toml", &read("clippy.toml")?)?,
        &standard,
    ));
    problems.extend(check_advisory_ignores(&parse(
        "deny.toml",
        &read("deny.toml")?,
    )?));
    let graph = graph::load(None).map_err(|error| error.to_string())?;
    let root = root
        .canonicalize()
        .map_err(|error| format!("could not resolve the repository root: {error}"))?;
    for id in &graph.members {
        let Some(path) = graph.manifest_paths.get(id) else {
            return Err(format!("`cargo metadata` has no manifest path for {id}"));
        };
        let file = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("could not read {file}: {error}"))?;
        let manifest = parse(&file, &text)?;
        problems.extend(check_member(&file, &manifest));
        problems.extend(check_member_dependencies(&file, &manifest));
        let source = path.parent().unwrap_or(&root).join("src");
        let crate_root = ["lib.rs", "main.rs"]
            .into_iter()
            .map(|name| source.join(name))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| format!("{file}: no src/lib.rs or src/main.rs"))?;
        let doc_file = crate_root
            .strip_prefix(&root)
            .unwrap_or(&crate_root)
            .to_string_lossy()
            .into_owned();
        let text = std::fs::read_to_string(&crate_root)
            .map_err(|error| format!("could not read {doc_file}: {error}"))?;
        problems.extend(check_crate_doc(&doc_file, graph.name(id), &text));
        let crate_dir = path.parent().unwrap_or(&root);
        for (_, relative) in DECODER_FILES
            .iter()
            .filter(|(name, _)| *name == graph.name(id))
        {
            let decoder = crate_dir.join(relative);
            let decoder_file = decoder
                .strip_prefix(&root)
                .unwrap_or(&decoder)
                .to_string_lossy()
                .into_owned();
            let text = std::fs::read_to_string(&decoder)
                .map_err(|error| format!("could not read {decoder_file}: {error}"))?;
            problems.extend(check_decoder_lints(&decoder_file, &text));
        }
    }
    Ok(problems)
}

/// Rule 7: `text` has an inner `#![deny(…)]` or `#![forbid(…)]` attribute naming every
/// lint in [`DECODER_LINTS`].
pub(crate) fn check_decoder_lints(file: &str, text: &str) -> Vec<String> {
    let denied: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            line.strip_prefix("#![deny(")
                .or_else(|| line.strip_prefix("#![forbid("))
        })
        .filter_map(|rest| rest.strip_suffix(")]"))
        .flat_map(|lints| lints.split(','))
        .map(str::trim)
        .collect();
    let missing: Vec<&str> = DECODER_LINTS
        .into_iter()
        .filter(|lint| !denied.contains(lint))
        .collect();
    if missing.is_empty() {
        return Vec::new();
    }
    vec![format!(
        "{file}: decodes untrusted bytes, so it must start with `#![deny({})]`; missing {} \
         (docs/standards/rust.md#workspace-configuration)",
        DECODER_LINTS.join(", "),
        missing.join(", ")
    )]
}

/// Rule 6: the crate root's leading `//!` doc names the crate's layer and its Kafka
/// counterpart.
pub(crate) fn check_crate_doc(file: &str, crate_name: &str, text: &str) -> Vec<String> {
    let standard = "docs/standards/rust.md#modules-and-documentation";
    let doc: Vec<&str> = text
        .lines()
        .map_while(|line| line.strip_prefix("//!"))
        .map(str::trim)
        .collect();
    if doc.is_empty() {
        return vec![format!(
            "{file}: the crate root must start with a `//!` doc ({standard})"
        )];
    }
    let mut problems = Vec::new();
    let expected = layers::layer_of(crate_name);
    match doc.iter().find_map(|line| line.strip_prefix("Layer:")) {
        None => problems.push(format!(
            "{file}: the crate doc has no `Layer:` line ({standard})"
        )),
        Some(rest) => {
            let named = rest
                .trim_start()
                .split(|c: char| c.is_whitespace() || matches!(c, '(' | '.' | ','))
                .next()
                .unwrap_or_default();
            match expected {
                None => problems.push(format!(
                    "{file}: {crate_name} is not in the layer table (xtask/src/layers.rs)"
                )),
                Some(layer) if layer.to_string() != named => problems.push(format!(
                    "{file}: the crate doc says `Layer: {named}`, but the layer table puts \
                     {crate_name} in layer {layer}"
                )),
                Some(_) => {}
            }
        }
    }
    if !doc.iter().any(|line| {
        line.strip_prefix("Kafka counterpart:")
            .is_some_and(|rest| !rest.trim().is_empty())
    }) {
        problems.push(format!(
            "{file}: the crate doc has no `Kafka counterpart:` line; write `Kafka \
             counterpart: none` if there is none ({standard})"
        ));
    }
    problems
}

fn parse(file: &str, text: &str) -> Result<Value, String> {
    toml::from_str(text).map_err(|error| format!("could not parse {file}: {error}"))
}

/// Finds and parses the lint and clippy.toml blocks in rust.md's
/// [`SECTION`]. Each must appear exactly once.
pub(crate) fn standard(rust_md: &str) -> Result<Standard, String> {
    let section = rust_md
        .split_once(&format!("\n{SECTION}\n"))
        .map(|(_, rest)| {
            rest.split_once("\n## ")
                .map_or(rest, |(section, _)| section)
        })
        .ok_or_else(|| format!("{RUST_MD} has no `{SECTION}` section"))?;
    let mut lints = Vec::new();
    let mut clippy = Vec::new();
    for block in toml_blocks(section) {
        let value = parse(&format!("a TOML block in {RUST_MD}"), block)?;
        if let Some(table) = value
            .get("workspace")
            .and_then(|workspace| workspace.get("lints"))
        {
            lints.push(table.clone());
        } else if value.get("disallowed-methods").is_some() {
            clippy.push(value);
        }
    }
    match (lints.as_slice(), clippy.as_slice()) {
        ([lints], [clippy]) => Ok(Standard {
            lints: lints.clone(),
            clippy: clippy.clone(),
        }),
        _ => Err(format!(
            "{RUST_MD} `{SECTION}` must have exactly one `[workspace.lints]` block and one \
             clippy.toml block (with `disallowed-methods`); found {} and {}",
            lints.len(),
            clippy.len()
        )),
    }
}

/// The contents of the fenced code blocks tagged `toml` in `markdown`.
fn toml_blocks(markdown: &str) -> Vec<&str> {
    markdown
        .split("```toml\n")
        .skip(1)
        .filter_map(|rest| rest.split_once("```").map(|(block, _)| block))
        .collect()
}

/// Rule 1: the root manifest's `[workspace.lints]` equals the standard's.
pub(crate) fn check_lints(cargo_toml: &Value, standard: &Standard) -> Vec<String> {
    let mut problems = Vec::new();
    let actual = &cargo_toml["workspace"]["lints"];
    diff(
        "Cargo.toml",
        "workspace.lints",
        actual,
        &standard.lints,
        &mut problems,
    );
    problems
}

/// Rule 2: clippy.toml equals the standard's block.
pub(crate) fn check_clippy(clippy_toml: &Value, standard: &Standard) -> Vec<String> {
    let mut problems = Vec::new();
    diff(
        "clippy.toml",
        "",
        clippy_toml,
        &standard.clippy,
        &mut problems,
    );
    problems
}

/// Rule 3: a member manifest inherits the workspace lints and license file.
pub(crate) fn check_member(file: &str, manifest: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    if manifest["lints"]["workspace"] != true {
        problems.push(format!(
            "{file}: `lints.workspace` must be true, so the crate inherits [workspace.lints] \
             (add `[lints]` with `workspace = true`)"
        ));
    }
    if manifest["package"]["license-file"]["workspace"] != true {
        problems.push(format!(
            "{file}: the package must declare `license-file.workspace = true`, so it carries \
             the repository's LICENSE (ADR-0015)"
        ));
    }
    problems
}

/// The keys Cargo allows next to `workspace = true` in a member's dependency.
const INHERITED_DEPENDENCY_KEYS: [&str; 3] = ["workspace", "features", "optional"];

const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// Rule 4: a member inherits every dependency from `[workspace.dependencies]`.
pub(crate) fn check_member_dependencies(file: &str, manifest: &Value) -> Vec<String> {
    let mut tables = Vec::new();
    for table in DEPENDENCY_TABLES {
        tables.push((table.to_owned(), &manifest[table]));
    }
    for (target, target_tables) in manifest["target"].as_object().into_iter().flatten() {
        for table in DEPENDENCY_TABLES {
            tables.push((format!("target.{target}.{table}"), &target_tables[table]));
        }
    }
    let mut problems = Vec::new();
    for (table, dependencies) in tables {
        for (name, spec) in dependencies.as_object().into_iter().flatten() {
            let inline_keys: Vec<String> = match spec {
                Value::Object(spec) if spec.get("workspace") == Some(&Value::Bool(true)) => spec
                    .keys()
                    .filter(|key| !INHERITED_DEPENDENCY_KEYS.contains(&key.as_str()))
                    .map(|key| format!("`{key}`"))
                    .collect(),
                Value::Object(spec) => spec.keys().map(|key| format!("`{key}`")).collect(),
                _ => vec![format!("the version {spec}")],
            };
            if !inline_keys.is_empty() {
                problems.push(format!(
                    "{file}: `[{table}]` dependency `{name}` sets {} inline; declare it once in the \
                     root [workspace.dependencies] and use `{{ workspace = true }}` here, adding \
                     only `features` or `optional` (docs/standards/rust.md#dependencies)",
                    inline_keys.join(", ")
                ));
            }
        }
    }
    problems
}

/// Rule 5: every advisory ignore in deny.toml carries a reason and an expiry.
pub(crate) fn check_advisory_ignores(deny_toml: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    for entry in deny_toml["advisories"]["ignore"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let id = entry["id"]
            .as_str()
            .or_else(|| entry.as_str())
            .unwrap_or("(no id)");
        for field in ["reason", "expiry"] {
            if entry[field].as_str().is_none_or(str::is_empty) {
                problems.push(format!(
                    "deny.toml: the ignore for advisory `{id}` has no `{field}`; write \
                     {{ id = \"{id}\", reason = \"…\", expiry = \"YYYY-MM-DD\" }} \
                     (docs/standards/security.md#dependencies)"
                ));
            }
        }
    }
    problems
}

/// Appends a line to `problems` for every place where `actual` differs from
/// `expected`. Tables are compared key by key and arrays as sets of entries.
fn diff(file: &str, key: &str, actual: &Value, expected: &Value, problems: &mut Vec<String>) {
    match (actual, expected) {
        (Value::Object(actual), Value::Object(expected)) => {
            for (name, expected) in expected {
                let key = join(key, name);
                match actual.get(name) {
                    Some(actual) => diff(file, &key, actual, expected, problems),
                    None => problems.push(format!(
                        "{file}: `{key}` is missing; {RUST_MD} has {expected}"
                    )),
                }
            }
            for (name, actual) in actual
                .iter()
                .filter(|(name, _)| !expected.contains_key(*name))
            {
                let key = join(key, name);
                problems.push(format!("{file}: `{key}` = {actual} is not in {RUST_MD}"));
            }
        }
        (Value::Array(actual), Value::Array(expected)) => {
            for entry in expected.iter().filter(|entry| !actual.contains(entry)) {
                problems.push(format!(
                    "{file}: `{key}` lacks {entry}, which {RUST_MD} has"
                ));
            }
            for entry in actual.iter().filter(|entry| !expected.contains(entry)) {
                problems.push(format!(
                    "{file}: `{key}` has {entry}, which {RUST_MD} does not"
                ));
            }
        }
        (Value::Null, _) => problems.push(format!(
            "{file}: `{key}` is missing; {RUST_MD} has {expected}"
        )),
        _ if actual != expected => {
            problems.push(format!(
                "{file}: `{key}` is {actual}; {RUST_MD} has {expected}"
            ));
        }
        _ => {}
    }
}

fn join(key: &str, name: &str) -> String {
    if key.is_empty() {
        name.to_owned()
    } else {
        format!("{key}.{name}")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{
        Standard, check_advisory_ignores, check_clippy, check_crate_doc, check_decoder_lints,
        check_lints, check_member, check_member_dependencies, parse, standard,
    };

    const RUST_MD: &str = "# Rust\n\n## Workspace configuration\n\nLints:\n\n```toml\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n\n[workspace.lints.clippy]\npedantic = { level = \"warn\", priority = -1 }\ntodo = \"deny\"\n```\n\nClippy:\n\n```toml\nallow-unwrap-in-tests = true\ndisallowed-methods = [\n  { path = \"std::process::exit\", reason = \"return errors\" },\n  { path = \"rand::rng\", reason = \"use the seeded RNG\" },\n]\n```\n\nRelease profile:\n\n```toml\n[profile.release]\npanic = \"abort\"\n```\n\n## Errors\n\n```toml\n[workspace.lints.rust]\nignored = \"because it is in another section\"\n```\n";

    fn toml(text: &str) -> Value {
        parse("test", text).unwrap_or_default()
    }

    fn fixture_standard() -> Standard {
        standard(RUST_MD).unwrap_or_else(|error| panic!("{error}"))
    }

    const CARGO_TOML: &str = "[workspace]\nmembers = []\n\n# A comment and reordered keys don't matter.\n[workspace.lints.clippy]\ntodo = \"deny\"\npedantic = { priority = -1, level = \"warn\" }\n\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n";

    const CLIPPY_TOML: &str = "disallowed-methods = [\n  { path = \"rand::rng\", reason = \"use the seeded RNG\" },\n  { path = \"std::process::exit\", reason = \"return errors\" },\n]\nallow-unwrap-in-tests = true\n";

    #[test]
    fn crate_docs_need_the_layer_and_kafka_counterpart_lines() {
        let good = "//! Domain types.\n//!\n//! Layer: 0 ([ADR-0009](x.md)).\n//! Kafka counterpart: `Uuid.java`.\n\nmod a;\n";
        assert_eq!(
            check_crate_doc("lib.rs", "alluvium-types", good),
            Vec::<String>::new()
        );
        let dev = "//! Tooling.\n//!\n//! Layer: dev.\n//! Kafka counterpart: none.\n";
        assert_eq!(
            check_crate_doc("main.rs", "xtask", dev),
            Vec::<String>::new()
        );

        let missing_doc = "// not a crate doc\nmod a;\n";
        assert_eq!(
            check_crate_doc("lib.rs", "alluvium-types", missing_doc),
            ["lib.rs: the crate root must start with a `//!` doc \
              (docs/standards/rust.md#modules-and-documentation)"]
        );
        let no_counterpart = "//! Domain types.\n//! Layer: 0.\n";
        assert_eq!(
            check_crate_doc("lib.rs", "alluvium-types", no_counterpart),
            [
                "lib.rs: the crate doc has no `Kafka counterpart:` line; write `Kafka \
              counterpart: none` if there is none \
              (docs/standards/rust.md#modules-and-documentation)"
            ]
        );
        let empty_counterpart = "//! Layer: 0.\n//! Kafka counterpart:\n";
        assert_eq!(
            check_crate_doc("lib.rs", "alluvium-types", empty_counterpart).len(),
            1
        );
        let no_layer = "//! Domain types.\n//! Kafka counterpart: none.\n";
        assert_eq!(
            check_crate_doc("lib.rs", "alluvium-types", no_layer),
            ["lib.rs: the crate doc has no `Layer:` line \
              (docs/standards/rust.md#modules-and-documentation)"]
        );
        let wrong_layer = "//! Layer: 2.\n//! Kafka counterpart: none.\n";
        assert_eq!(
            check_crate_doc("lib.rs", "alluvium-types", wrong_layer),
            [
                "lib.rs: the crate doc says `Layer: 2`, but the layer table puts alluvium-types \
              in layer 0"
            ]
        );
        // The lines count only inside the leading doc, not in a later module doc.
        let late = "//! Domain types.\n\n//! Layer: 0.\n//! Kafka counterpart: none.\n";
        assert_eq!(check_crate_doc("lib.rs", "alluvium-types", late).len(), 2);
        assert_eq!(
            check_crate_doc(
                "lib.rs",
                "mystery",
                "//! Layer: 0.\n//! Kafka counterpart: none.\n"
            ),
            ["lib.rs: mystery is not in the layer table (xtask/src/layers.rs)"]
        );
    }

    #[test]
    fn the_blocks_are_found_in_their_section_only() {
        let standard = fixture_standard();
        assert_eq!(standard.lints["rust"]["unsafe_code"], "forbid");
        assert!(standard.lints["rust"].get("ignored").is_none());
        assert_eq!(standard.clippy["allow-unwrap-in-tests"], true);
    }

    #[test]
    fn a_missing_or_duplicated_block_is_an_error() {
        let no_clippy = RUST_MD.replace("disallowed-methods", "other-methods");
        assert!(standard(&no_clippy).is_err_and(|error| error.contains("found 1 and 0")));
        assert!(
            standard("# Rust\n")
                .is_err_and(|error| error.contains("no `## Workspace configuration`"))
        );
    }

    #[test]
    fn matching_files_pass_whatever_their_formatting_and_order() {
        let standard = fixture_standard();
        assert_eq!(
            check_lints(&toml(CARGO_TOML), &standard),
            Vec::<String>::new()
        );
        assert_eq!(
            check_clippy(&toml(CLIPPY_TOML), &standard),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_changed_lint_level_names_the_file_and_key() {
        let changed = CARGO_TOML.replace("todo = \"deny\"", "todo = \"warn\"");
        assert_eq!(
            check_lints(&toml(&changed), &fixture_standard()),
            [
                "Cargo.toml: `workspace.lints.clippy.todo` is \"warn\"; docs/standards/rust.md has \"deny\""
            ]
        );
        let priority = CARGO_TOML.replace("priority = -1", "priority = 0");
        assert_eq!(
            check_lints(&toml(&priority), &fixture_standard()),
            [
                "Cargo.toml: `workspace.lints.clippy.pedantic.priority` is 0; docs/standards/rust.md has -1"
            ]
        );
    }

    #[test]
    fn added_and_removed_lints_are_reported() {
        let edited = CARGO_TOML.replace("todo = \"deny\"\n", "dbg_macro = \"deny\"\n");
        assert_eq!(
            check_lints(&toml(&edited), &fixture_standard()),
            [
                "Cargo.toml: `workspace.lints.clippy.todo` is missing; docs/standards/rust.md has \"deny\"",
                "Cargo.toml: `workspace.lints.clippy.dbg_macro` = \"deny\" is not in docs/standards/rust.md",
            ]
        );
        assert_eq!(
            check_lints(&toml("[workspace]\n"), &fixture_standard()).len(),
            1
        );
    }

    #[test]
    fn an_added_or_removed_disallowed_method_is_reported() {
        let removed = CLIPPY_TOML.replace(
            "  { path = \"rand::rng\", reason = \"use the seeded RNG\" },\n",
            "",
        );
        assert_eq!(
            check_clippy(&toml(&removed), &fixture_standard()),
            [
                "clippy.toml: `disallowed-methods` lacks {\"path\":\"rand::rng\",\"reason\":\"use the seeded RNG\"}, which docs/standards/rust.md has"
            ]
        );
        let added = CLIPPY_TOML.replace(
            "disallowed-methods = [\n",
            "disallowed-methods = [\n  { path = \"std::thread::sleep\", reason = \"use the Clock\" },\n",
        );
        assert_eq!(
            check_clippy(&toml(&added), &fixture_standard()),
            [
                "clippy.toml: `disallowed-methods` has {\"path\":\"std::thread::sleep\",\"reason\":\"use the Clock\"}, which docs/standards/rust.md does not"
            ]
        );
    }

    #[test]
    fn a_member_must_inherit_the_workspace_lints_and_license() {
        let license = "license-file.workspace = true\n";
        let inherits = format!("[package]\nname = \"a\"\n{license}\n[lints]\nworkspace = true\n");
        assert_eq!(
            check_member("a/Cargo.toml", &toml(&inherits)),
            Vec::<String>::new()
        );
        for manifest in [
            format!("[package]\nname = \"a\"\n{license}"),
            format!("[package]\nname = \"a\"\n{license}\n[lints]\nworkspace = false\n"),
        ] {
            let problems = check_member("a/Cargo.toml", &toml(&manifest));
            assert_eq!(problems.len(), 1, "{manifest}");
            assert!(problems[0].starts_with("a/Cargo.toml: `lints.workspace` must be true"));
        }
        for manifest in [
            "[package]\nname = \"a\"\n\n[lints]\nworkspace = true\n",
            "[package]\nname = \"a\"\nlicense-file = \"OTHER\"\n\n[lints]\nworkspace = true\n",
        ] {
            assert_eq!(
                check_member("a/Cargo.toml", &toml(manifest)),
                [
                    "a/Cargo.toml: the package must declare `license-file.workspace = true`, so it \
                  carries the repository's LICENSE (ADR-0015)"
                ],
                "{manifest}"
            );
        }
    }

    #[test]
    fn advisory_ignores_need_a_reason_and_an_expiry() {
        let complete = "[advisories]\nignore = [\n  { id = \"RUSTSEC-2026-0001\", reason = \"not reachable\", expiry = \"2026-12-31\" },\n]\n";
        assert_eq!(
            check_advisory_ignores(&toml(complete)),
            Vec::<String>::new()
        );
        assert_eq!(
            check_advisory_ignores(&toml("[advisories]\nignore = []\n")),
            Vec::<String>::new()
        );
        let missing_reason = complete.replace("reason = \"not reachable\", ", "");
        let problems = check_advisory_ignores(&toml(&missing_reason));
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(
            problems[0].starts_with(
                "deny.toml: the ignore for advisory `RUSTSEC-2026-0001` has no `reason`"
            ),
            "{problems:?}"
        );
        let missing_expiry = complete.replace(", expiry = \"2026-12-31\"", "");
        let problems = check_advisory_ignores(&toml(&missing_expiry));
        assert!(
            problems.len() == 1 && problems[0].contains("`RUSTSEC-2026-0001` has no `expiry`"),
            "{problems:?}"
        );
        let bare_id = "[advisories]\nignore = [\"RUSTSEC-2026-0002\"]\n";
        let problems = check_advisory_ignores(&toml(bare_id));
        assert_eq!(problems.len(), 2, "{problems:?}");
        assert!(
            problems
                .iter()
                .all(|problem| problem.contains("`RUSTSEC-2026-0002`")),
            "{problems:?}"
        );
    }

    #[test]
    fn inherited_dependencies_may_add_only_features_and_optional() {
        let manifest = "[dependencies]\nserde_json = { workspace = true }\ntokio = { workspace = true, optional = true, features = [\"rt\"] }\n\n[dev-dependencies]\nalluvium-testkit = { workspace = true }\n";
        assert_eq!(
            check_member_dependencies("a/Cargo.toml", &toml(manifest)),
            Vec::<String>::new()
        );
    }

    #[test]
    fn inline_versions_paths_and_git_sources_are_rejected_by_crate_and_dependency() {
        let manifest = "[dependencies]\nbytes = \"1.10\"\nthiserror = { version = \"2\" }\nhelper = { path = \"../helper\" }\nforked = { git = \"https://example.com/forked\" }\nrand = { workspace = true, version = \"0.10\" }\n\n[target.'cfg(unix)'.build-dependencies]\ncc = \"1\"\n";
        let problems = check_member_dependencies("a/Cargo.toml", &toml(manifest));
        let starts: Vec<&str> = problems
            .iter()
            .map(|problem| problem.split(" inline;").next().unwrap_or_default())
            .collect();
        assert_eq!(
            starts,
            [
                "a/Cargo.toml: `[dependencies]` dependency `bytes` sets the version \"1.10\"",
                "a/Cargo.toml: `[dependencies]` dependency `forked` sets `git`",
                "a/Cargo.toml: `[dependencies]` dependency `helper` sets `path`",
                "a/Cargo.toml: `[dependencies]` dependency `rand` sets `version`",
                "a/Cargo.toml: `[dependencies]` dependency `thiserror` sets `version`",
                "a/Cargo.toml: `[target.cfg(unix).build-dependencies]` dependency `cc` sets the version \"1\"",
            ]
        );
    }

    #[test]
    fn a_decoder_file_without_the_attribute_is_rejected() {
        let lib = "//! Kafka's wire primitives.\n\nmod varint;\n";
        assert_eq!(
            check_decoder_lints("crates/alluvium-protocol/src/lib.rs", lib),
            [
                "crates/alluvium-protocol/src/lib.rs: decodes untrusted bytes, so it must start \
                 with `#![deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)]`; \
                 missing clippy::indexing_slicing, clippy::arithmetic_side_effects \
                 (docs/standards/rust.md#workspace-configuration)"
            ]
        );
        let partial = "//! Records.\n#![deny(clippy::indexing_slicing)]\n";
        let problems = check_decoder_lints("lib.rs", partial);
        assert!(
            problems[0].contains("missing clippy::arithmetic_side_effects ("),
            "{problems:?}"
        );
    }

    #[test]
    fn the_attribute_passes_in_either_order_as_deny_or_forbid() {
        for lib in [
            "//! Doc.\n\n#![deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)]\n",
            "//! Doc.\n#![forbid(clippy::arithmetic_side_effects,clippy::indexing_slicing)]\n",
            "#![deny(clippy::indexing_slicing)]\n#![deny(clippy::arithmetic_side_effects)]\n",
        ] {
            assert_eq!(
                check_decoder_lints("lib.rs", lib),
                Vec::<String>::new(),
                "{lib}"
            );
        }
    }
}
