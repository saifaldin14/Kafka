//! `cargo xtask lint-canary`: proves the workspace lints fire.
//!
//! The `lint-canary` crate (`xtask/lint-canary`) holds fixture code that breaks the
//! rules in `[workspace.lints]` and clippy.toml on purpose. Each offending line ends
//! with an `expect-lint:` line comment listing the lints it must trip, comma-separated,
//! spelled as in clippy's JSON output (`unsafe_code`, `clippy::todo`). This command runs
//! clippy on that crate with its `canary` feature and requires:
//!
//! 1. The diagnostics match the annotations exactly: every annotated lint fires on its
//!    line, and nothing else fires. A lint removed, misspelled or weakened to `allow`
//!    stops firing, and the command names the fixture line.
//! 2. Every `disallowed-methods` and `disallowed-types` diagnostic carries the reason
//!    clippy.toml configures for its path, and every entry fires at least once, so a
//!    new entry needs a fixture.
//! 3. Every lint `[workspace.lints]` configures one by one, at a level other than
//!    `allow`, is annotated somewhere, so by rule 1 it fires. A lint added to the
//!    table without a fixture would otherwise go unproven. Groups such as
//!    `clippy::pedantic`, written as tables with a `priority`, are left out: no one
//!    fixture can prove a group.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::Exit;
use crate::output;
use crate::tools;

/// The fixture crate's source directory, relative to the repository root.
const CANARY_SRC: &str = "xtask/lint-canary/src";

/// The clippy run whose diagnostics are compared with the annotations.
const CLIPPY_ARGS: &[&str] = &[
    "clippy",
    "--package",
    "lint-canary",
    "--features",
    "canary",
    "--locked",
    "--message-format=json",
];

const MARKER: &str = "// expect-lint:";

/// The clippy.toml keys whose entries carry a reason, and the lint each one drives.
const DISALLOWED_KEYS: [(&str, &str); 2] = [
    ("disallowed-methods", "clippy::disallowed_methods"),
    ("disallowed-types", "clippy::disallowed_types"),
];

/// Where a diagnostic points: a file relative to the repository root and a 1-based
/// line. `None` for diagnostics without a source location, such as an unknown lint
/// name in `[workspace.lints]`.
pub(crate) type Location = Option<(String, u64)>;

/// Lint names by location, for both the annotations and clippy's diagnostics.
pub(crate) type Lints = BTreeMap<Location, BTreeSet<String>>;

/// The configured reason for each `(lint, path)` entry in clippy.toml.
pub(crate) type Disallowed = BTreeMap<(&'static str, String), String>;

/// One diagnostic from clippy's JSON output, at one of its primary locations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Diagnostic {
    pub(crate) location: Location,
    /// The lint or error code. A diagnostic without a location also carries its
    /// message, and one without a code is its level and message.
    pub(crate) name: String,
    pub(crate) message: String,
    /// The `note` children. For the `disallowed_*` lints, one is the configured reason.
    pub(crate) notes: Vec<String>,
}

/// A location where the lints that fired differ from the annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Mismatch {
    pub(crate) location: Location,
    pub(crate) expected: BTreeSet<String>,
    pub(crate) actual: BTreeSet<String>,
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: expected {}; clippy reported {}",
            location_text(&self.location),
            list(&self.expected),
            list(&self.actual)
        )
    }
}

fn location_text(location: &Location) -> String {
    match location {
        Some((file, line)) => format!("{file}:{line}"),
        None => "(no source location)".to_owned(),
    }
}

fn list(lints: &BTreeSet<String>) -> String {
    if lints.is_empty() {
        "nothing".to_owned()
    } else {
        lints.iter().cloned().collect::<Vec<_>>().join(", ")
    }
}

/// Runs the command.
pub(crate) fn run(out: &mut impl Write) -> Exit {
    let root = crate::repo_root();
    let checked = read_annotations(&root).and_then(|expected| {
        let clippy_toml = std::fs::read_to_string(root.join("clippy.toml"))
            .map_err(|error| format!("could not read clippy.toml: {error}"))?;
        let disallowed = parse_disallowed(&clippy_toml)?;
        let cargo_toml = std::fs::read_to_string(root.join("Cargo.toml"))
            .map_err(|error| format!("could not read Cargo.toml: {error}"))?;
        let configured = configured_lints(&cargo_toml)?;
        let diagnostics = run_clippy(&root, out)?;
        let mismatches = compare(&expected, &by_location(&diagnostics));
        let mut rule_problems = check_reasons(&disallowed, &diagnostics);
        rule_problems.extend(check_fixture_coverage(&configured, &expected));
        Ok((expected, mismatches, rule_problems))
    });
    let (expected, mismatches, rule_problems) = match checked {
        Ok(checked) => checked,
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            return Exit::CheckFailed;
        }
    };
    if mismatches.is_empty() && rule_problems.is_empty() {
        let count: usize = expected.values().map(BTreeSet::len).sum();
        output::line(
            out,
            &format!(
                "lint-canary: ok ({count} annotated lints fired with their configured reasons, nothing else; every [workspace.lints] lint has a fixture)"
            ),
        );
        return Exit::Success;
    }
    for mismatch in &mismatches {
        output::line(out, &mismatch.to_string());
    }
    for problem in &rule_problems {
        output::line(out, problem);
    }
    output::line(
        out,
        &format!(
            "lint-canary: {} problem(s). A lint that no longer fires was removed or weakened in \
             [workspace.lints] or clippy.toml (docs/standards/rust.md#workspace-configuration)",
            mismatches.len() + rule_problems.len()
        ),
    );
    Exit::CheckFailed
}

/// Runs clippy on the fixtures and returns its diagnostics.
fn run_clippy(root: &Path, out: &mut impl Write) -> Result<Vec<Diagnostic>, String> {
    output::line(
        out,
        &format!(
            "running: cargo {}\n(the fixtures trip deny-level lints, so cargo's \"could not compile\" error is expected)",
            CLIPPY_ARGS.join(" ")
        ),
    );
    let clippy = Command::new(tools::cargo_program())
        .current_dir(root)
        .args(CLIPPY_ARGS)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|error| format!("could not run cargo clippy: {error}"))?;
    let diagnostics = parse_diagnostics(&String::from_utf8_lossy(&clippy.stdout))?;
    // A failed exit status is the normal result, since the fixtures trip deny-level
    // lints. It only explains an empty result, for example a stale lock file.
    if diagnostics.is_empty() && !clippy.status.success() {
        return Err(format!(
            "cargo clippy reported no diagnostics and exited with {}; see its output above",
            clippy.status
        ));
    }
    Ok(diagnostics)
}

/// Reads every `.rs` file under [`CANARY_SRC`] and collects its annotations.
fn read_annotations(root: &Path) -> Result<Lints, String> {
    let mut files = Vec::new();
    collect_rust_files(&root.join(CANARY_SRC), CANARY_SRC, &mut files)
        .map_err(|error| format!("could not read {CANARY_SRC}: {error}"))?;
    files.sort();
    let mut lints = Lints::new();
    for file in &files {
        let text = std::fs::read_to_string(root.join(file))
            .map_err(|error| format!("could not read {file}: {error}"))?;
        lints.extend(parse_annotations(file, &text)?);
    }
    if lints.is_empty() {
        return Err(format!(
            "no `expect-lint:` annotations found under {CANARY_SRC}"
        ));
    }
    Ok(lints)
}

/// Appends the `.rs` files under `dir` to `files`, as `/`-separated paths starting
/// with `relative` (the repository-relative form clippy reports).
fn collect_rust_files(dir: &Path, relative: &str, files: &mut Vec<String>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = format!("{relative}/{name}");
        if entry.file_type()?.is_dir() {
            collect_rust_files(&entry.path(), &path, files)?;
        } else if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "rs")
        {
            files.push(path);
        }
    }
    Ok(())
}

/// The annotations in one fixture file. An annotation trails the code it is about;
/// one on a comment-only line or with no lint names is an error, so a typo can't
/// silently drop a fixture.
pub(crate) fn parse_annotations(file: &str, text: &str) -> Result<Lints, String> {
    let mut lints = Lints::new();
    for (line_number, line) in (1_u64..).zip(text.lines()) {
        let Some((code, names)) = line.split_once(MARKER) else {
            continue;
        };
        let at = format!("{file}:{line_number}");
        if code.trim().is_empty() || code.trim_start().starts_with("//") {
            return Err(format!(
                "{at}: an `expect-lint:` comment must trail the code that trips the lint"
            ));
        }
        let names: BTreeSet<String> = names
            .split(',')
            .map(|name| name.trim().to_owned())
            .collect();
        if let Some(bad) = names.iter().find(|name| !is_lint_name(name)) {
            return Err(format!(
                "{at}: `{bad}` is not a lint name (expected names like `unsafe_code` or `clippy::todo`)"
            ));
        }
        lints.insert(Some((file.to_owned(), line_number)), names);
    }
    Ok(lints)
}

fn is_lint_name(name: &str) -> bool {
    let bare = name.strip_prefix("clippy::").unwrap_or(name);
    !bare.is_empty()
        && bare
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

/// The diagnostics in cargo's `--message-format=json` output, one per primary
/// location. Cargo's closing summary lines (no code and no location) are skipped.
pub(crate) fn parse_diagnostics(json_lines: &str) -> Result<Vec<Diagnostic>, String> {
    let mut diagnostics = Vec::new();
    for line in json_lines.lines().filter(|line| !line.trim().is_empty()) {
        let json: Value = serde_json::from_str(line)
            .map_err(|error| format!("cargo printed a line that is not JSON ({error}): {line}"))?;
        if json["reason"] != "compiler-message" {
            continue;
        }
        let diagnostic = &json["message"];
        let message = diagnostic["message"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        let mut locations: Vec<Location> = diagnostic["spans"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|span| span["is_primary"] == true)
            .filter_map(|span| {
                Some(Some((
                    span["file_name"].as_str()?.to_owned(),
                    span["line_start"].as_u64()?,
                )))
            })
            .collect();
        let name = match (diagnostic["code"]["code"].as_str(), locations.is_empty()) {
            (Some(code), false) => code.to_owned(),
            // Without a location, the message is the only clue (for example, which lint
            // name in `[workspace.lints]` is unknown).
            (Some(code), true) => format!("{code}: {message}"),
            (None, true) => continue,
            (None, false) => format!(
                "{}: {message}",
                diagnostic["level"].as_str().unwrap_or("diagnostic")
            ),
        };
        let notes: Vec<String> = diagnostic["children"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|child| child["level"] == "note")
            .filter_map(|child| child["message"].as_str().map(str::to_owned))
            .collect();
        if locations.is_empty() {
            locations.push(None);
        }
        for location in locations {
            diagnostics.push(Diagnostic {
                location,
                name: name.clone(),
                message: message.clone(),
                notes: notes.clone(),
            });
        }
    }
    Ok(diagnostics)
}

/// The lint names that fired, by location.
pub(crate) fn by_location(diagnostics: &[Diagnostic]) -> Lints {
    let mut lints = Lints::new();
    for diagnostic in diagnostics {
        lints
            .entry(diagnostic.location.clone())
            .or_default()
            .insert(diagnostic.name.clone());
    }
    lints
}

/// Every location where the expected and actual lints differ, in location order.
pub(crate) fn compare(expected: &Lints, actual: &Lints) -> Vec<Mismatch> {
    let locations: BTreeSet<&Location> = expected.keys().chain(actual.keys()).collect();
    let none = BTreeSet::new();
    locations
        .into_iter()
        .filter_map(|location| {
            let expected = expected.get(location).unwrap_or(&none);
            let actual = actual.get(location).unwrap_or(&none);
            (expected != actual).then(|| Mismatch {
                location: location.clone(),
                expected: expected.clone(),
                actual: actual.clone(),
            })
        })
        .collect()
}

/// The `disallowed-methods` and `disallowed-types` entries of clippy.toml. Every entry
/// must be a table with a `path` and a `reason`.
pub(crate) fn parse_disallowed(clippy_toml: &str) -> Result<Disallowed, String> {
    let config: Value = toml::from_str(clippy_toml)
        .map_err(|error| format!("could not parse clippy.toml: {error}"))?;
    let mut disallowed = Disallowed::new();
    for (key, lint) in DISALLOWED_KEYS {
        for entry in config[key].as_array().into_iter().flatten() {
            let path = entry["path"]
                .as_str()
                .ok_or_else(|| format!("clippy.toml: a `{key}` entry has no `path`: {entry}"))?;
            let reason = entry["reason"]
                .as_str()
                .ok_or_else(|| format!("clippy.toml: `{path}` in `{key}` has no `reason`"))?;
            disallowed.insert((lint, path.to_owned()), reason.to_owned());
        }
    }
    Ok(disallowed)
}

/// The lints `[workspace.lints]` in `cargo_toml` configures one by one, at a level other
/// than `allow`, named as clippy's JSON names them, with their levels.
pub(crate) fn configured_lints(cargo_toml: &str) -> Result<BTreeMap<String, String>, String> {
    let manifest: Value = toml::from_str(cargo_toml)
        .map_err(|error| format!("could not parse Cargo.toml: {error}"))?;
    let mut lints = BTreeMap::new();
    for (tool, prefix) in [("rust", ""), ("clippy", "clippy::")] {
        let Some(table) = manifest["workspace"]["lints"][tool].as_object() else {
            continue;
        };
        for (name, setting) in table {
            // A table entry is a group with a `priority`; single lints are strings.
            if let Some(level) = setting.as_str()
                && level != "allow"
            {
                lints.insert(format!("{prefix}{name}"), level.to_owned());
            }
        }
    }
    Ok(lints)
}

/// Checks that every configured lint is annotated on at least one fixture line.
pub(crate) fn check_fixture_coverage(
    configured: &BTreeMap<String, String>,
    expected: &Lints,
) -> Vec<String> {
    let annotated: BTreeSet<&String> = expected.values().flatten().collect();
    configured
        .iter()
        .filter(|(lint, _)| !annotated.contains(lint))
        .map(|(lint, level)| {
            format!(
                "Cargo.toml: `{lint}` ({level}) in [workspace.lints] has no fixture. Add one to \
                 xtask/lint-canary, so removing or weakening it is noticed"
            )
        })
        .collect()
}

/// Checks that each `disallowed_*` diagnostic carries the reason configured for its
/// path, and that every clippy.toml entry fired at least once.
pub(crate) fn check_reasons(disallowed: &Disallowed, diagnostics: &[Diagnostic]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut fired = BTreeSet::new();
    for diagnostic in diagnostics {
        let Some(lint) = DISALLOWED_KEYS
            .iter()
            .map(|(_, lint)| *lint)
            .find(|lint| *lint == diagnostic.name)
        else {
            continue;
        };
        let at = location_text(&diagnostic.location);
        // Clippy names the configured path in backticks: "use of a disallowed method `a::b`".
        let Some(path) = diagnostic.message.split('`').nth(1) else {
            problems.push(format!("{at}: no path in `{}`", diagnostic.message));
            continue;
        };
        let key = (lint, path.to_owned());
        match disallowed.get(&key) {
            None => problems.push(format!(
                "{at}: {lint} fired for `{path}`, which clippy.toml does not list"
            )),
            Some(reason) if !diagnostic.notes.contains(reason) => problems.push(format!(
                "{at}: `{path}` fired without its configured reason \"{reason}\""
            )),
            Some(_) => {}
        }
        fired.insert(key);
    }
    for (lint, path) in disallowed.keys() {
        if !fired.contains(&(*lint, path.clone())) {
            problems.push(format!(
                "clippy.toml: `{path}` ({lint}) never fired. Add a fixture for it to \
                 xtask/lint-canary, or remove it if another entry names the same function"
            ));
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        Diagnostic, Lints, by_location, check_fixture_coverage, check_reasons, compare,
        configured_lints, parse_annotations, parse_diagnostics, parse_disallowed,
    };

    fn at(file: &str, line: u64, names: &[&str]) -> Lints {
        Lints::from([(
            Some((file.to_owned(), line)),
            names.iter().map(|name| (*name).to_owned()).collect(),
        )])
    }

    fn disallowed_call(line: u64, path: &str, note: &str) -> Diagnostic {
        Diagnostic {
            location: Some(("a.rs".to_owned(), line)),
            name: "clippy::disallowed_methods".to_owned(),
            message: format!("use of a disallowed method `{path}`"),
            notes: vec![note.to_owned()],
        }
    }

    #[test]
    fn trailing_annotations_parse_with_one_or_more_lints() {
        let text = "fn f() {\n    x.unwrap() // expect-lint: clippy::unwrap_used\n    exit(1); // expect-lint: clippy::exit, clippy::disallowed_methods\n}\n";
        let mut expected = at("a.rs", 2, &["clippy::unwrap_used"]);
        expected.extend(at(
            "a.rs",
            3,
            &["clippy::disallowed_methods", "clippy::exit"],
        ));
        assert_eq!(parse_annotations("a.rs", text), Ok(expected));
    }

    #[test]
    fn misplaced_or_malformed_annotations_are_errors_with_file_and_line() {
        let own_line = parse_annotations("a.rs", "\n    // expect-lint: clippy::todo\n");
        assert!(
            own_line
                .as_ref()
                .is_err_and(|error| error.starts_with("a.rs:2: ") && error.contains("must trail")),
            "{own_line:?}"
        );
        let empty = parse_annotations("a.rs", "f(); // expect-lint:\n");
        assert!(
            empty
                .as_ref()
                .is_err_and(|error| error.starts_with("a.rs:1: ")),
            "{empty:?}"
        );
        let bad = parse_annotations("a.rs", "f(); // expect-lint: clippy::Todo\n");
        assert!(
            bad.as_ref()
                .is_err_and(|error| error.contains("`clippy::Todo`")),
            "{bad:?}"
        );
    }

    #[test]
    fn diagnostics_are_keyed_by_primary_span_and_summaries_are_skipped() {
        let json = [
            r#"{"reason":"compiler-artifact","target":{}}"#,
            r#"{"reason":"compiler-message","message":{"level":"error","message":"used unwrap","code":{"code":"clippy::unwrap_used"},"spans":[{"file_name":"a.rs","line_start":9,"is_primary":false},{"file_name":"a.rs","line_start":2,"is_primary":true}],"children":[]}}"#,
            r#"{"reason":"compiler-message","message":{"level":"error","message":"aborting due to 1 previous error","code":null,"spans":[],"children":[]}}"#,
            r#"{"reason":"compiler-message","message":{"level":"error","message":"expected `;`","code":null,"spans":[{"file_name":"a.rs","line_start":5,"is_primary":true}],"children":[]}}"#,
            r#"{"reason":"compiler-message","message":{"level":"warning","message":"unknown lint: `clippy::todoo`","code":{"code":"unknown_lints"},"spans":[],"children":[]}}"#,
        ]
        .join("\n");
        let mut expected = at("a.rs", 2, &["clippy::unwrap_used"]);
        expected.extend(at("a.rs", 5, &["error: expected `;`"]));
        expected.insert(
            None,
            BTreeSet::from(["unknown_lints: unknown lint: `clippy::todoo`".to_owned()]),
        );
        assert_eq!(
            parse_diagnostics(&json).map(|d| by_location(&d)),
            Ok(expected)
        );
    }

    #[test]
    fn notes_are_kept_and_other_children_dropped() {
        let json = r#"{"reason":"compiler-message","message":{"level":"warning","message":"use of a disallowed method `a::b`","code":{"code":"clippy::disallowed_methods"},"spans":[{"file_name":"a.rs","line_start":3,"is_primary":true}],"children":[{"level":"note","message":"use the Clock"},{"level":"help","message":"see the docs"}]}}"#;
        assert_eq!(
            parse_diagnostics(json),
            Ok(vec![disallowed_call(3, "a::b", "use the Clock")])
        );
    }

    #[test]
    fn output_that_is_not_json_is_an_error() {
        assert!(parse_diagnostics("Checking lint-canary").is_err());
    }

    #[test]
    fn matching_lints_give_no_mismatches() {
        let lints = at("a.rs", 2, &["clippy::todo"]);
        assert_eq!(compare(&lints, &lints), Vec::new());
    }

    #[test]
    fn missing_and_unexpected_lints_are_reported_with_both_sides() {
        let expected = at("a.rs", 2, &["clippy::todo"]);
        let mut actual = at("a.rs", 7, &["clippy::panic"]);
        actual.insert(None, BTreeSet::from(["unknown_lints".to_owned()]));
        let report: Vec<String> = compare(&expected, &actual)
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            report,
            [
                "(no source location): expected nothing; clippy reported unknown_lints",
                "a.rs:2: expected clippy::todo; clippy reported nothing",
                "a.rs:7: expected nothing; clippy reported clippy::panic",
            ]
        );
    }

    #[test]
    fn disallowed_entries_need_a_path_and_a_reason() {
        let config = r#"
            disallowed-methods = [{ path = "a::b", reason = "use c" }]
            disallowed-types = [{ path = "a::T", reason = "use U" }]
        "#;
        let disallowed = parse_disallowed(config);
        assert_eq!(
            disallowed.as_ref().map(std::collections::BTreeMap::len),
            Ok(2),
            "{disallowed:?}"
        );
        let no_reason = parse_disallowed(r#"disallowed-methods = [{ path = "a::b" }]"#);
        assert_eq!(
            no_reason,
            Err("clippy.toml: `a::b` in `disallowed-methods` has no `reason`".to_owned())
        );
        assert!(parse_disallowed(r#"disallowed-methods = ["a::b"]"#).is_err());
    }

    #[test]
    fn reasons_must_match_and_every_entry_must_fire() {
        let config = r#"disallowed-methods = [
            { path = "a::b", reason = "use c" },
            { path = "a::d", reason = "use e" },
        ]"#;
        let disallowed = parse_disallowed(config).unwrap_or_default();
        assert_eq!(disallowed.len(), 2);
        let correct = [
            disallowed_call(1, "a::b", "use c"),
            disallowed_call(2, "a::d", "use e"),
        ];
        assert_eq!(check_reasons(&disallowed, &correct), Vec::<String>::new());
        let problems = check_reasons(
            &disallowed,
            &[
                disallowed_call(1, "a::b", "stale reason"),
                disallowed_call(2, "x::y", "use e"),
            ],
        );
        assert_eq!(
            problems,
            [
                "a.rs:1: `a::b` fired without its configured reason \"use c\"",
                "a.rs:2: clippy::disallowed_methods fired for `x::y`, which clippy.toml does not list",
                "clippy.toml: `a::d` (clippy::disallowed_methods) never fired. Add a fixture for it to \
                 xtask/lint-canary, or remove it if another entry names the same function",
            ]
        );
    }

    #[test]
    fn configured_lints_are_single_lints_not_allowed_or_grouped() {
        let cargo_toml = "[workspace.lints.rust]\nunsafe_code = \"forbid\"\nrust_2018_idioms = { level = \"warn\", priority = -1 }\n\n[workspace.lints.clippy]\npedantic = { level = \"warn\", priority = -1 }\nmust_use_candidate = \"allow\"\npanic = \"warn\"\n";
        let lints = configured_lints(cargo_toml).unwrap();
        assert_eq!(
            lints.into_iter().collect::<Vec<_>>(),
            [
                ("clippy::panic".to_owned(), "warn".to_owned()),
                ("unsafe_code".to_owned(), "forbid".to_owned()),
            ]
        );
    }

    #[test]
    fn a_configured_lint_without_a_fixture_is_reported() {
        let cargo_toml =
            "[workspace.lints.clippy]\npanic = \"warn\"\nrc_buffer = \"warn\"\ntodo = \"deny\"\n";
        let mut expected = at("f.rs", 3, &["clippy::todo"]);
        expected.extend(at("f.rs", 9, &["clippy::panic", "unsafe_code"]));
        assert_eq!(
            check_fixture_coverage(&configured_lints(cargo_toml).unwrap(), &expected),
            [
                "Cargo.toml: `clippy::rc_buffer` (warn) in [workspace.lints] has no fixture. Add \
                 one to xtask/lint-canary, so removing or weakening it is noticed"
            ]
        );
    }
}
