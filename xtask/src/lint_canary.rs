//! `cargo xtask lint-canary`: proves the workspace lints fire.
//!
//! The `lint-canary` crate (`xtask/lint-canary`) holds fixture code that breaks the
//! rules in `[workspace.lints]` and clippy.toml on purpose. Each offending line ends
//! with an `expect-lint:` line comment listing the lints it must trip, comma-separated,
//! spelled as in clippy's JSON output (`unsafe_code`, `clippy::todo`). This command runs
//! clippy on that crate with its `canary` feature and requires the diagnostics to match
//! the annotations exactly: every annotated lint fires on its line, and nothing else
//! fires. A lint removed, misspelled or weakened to `allow` stops firing, and the
//! command fails naming the fixture line.

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

/// Where a diagnostic points: a file relative to the repository root and a 1-based
/// line. `None` for diagnostics without a source location, such as an unknown lint
/// name in `[workspace.lints]`.
pub(crate) type Location = Option<(String, u64)>;

/// Lint names by location, for both the annotations and clippy's diagnostics.
pub(crate) type Lints = BTreeMap<Location, BTreeSet<String>>;

/// A location where the lints that fired differ from the annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Mismatch {
    pub(crate) location: Location,
    pub(crate) expected: BTreeSet<String>,
    pub(crate) actual: BTreeSet<String>,
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.location {
            Some((file, line)) => write!(f, "{file}:{line}")?,
            None => f.write_str("(no source location)")?,
        }
        write!(
            f,
            ": expected {}; clippy reported {}",
            list(&self.expected),
            list(&self.actual)
        )
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
    let expected = match read_annotations(&root) {
        Ok(expected) => expected,
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            return Exit::CheckFailed;
        }
    };
    output::line(
        out,
        &format!(
            "running: cargo {}\n(the fixtures trip deny-level lints, so cargo's \"could not compile\" error is expected)",
            CLIPPY_ARGS.join(" ")
        ),
    );
    let clippy = Command::new(tools::cargo_program())
        .current_dir(&root)
        .args(CLIPPY_ARGS)
        .stderr(Stdio::inherit())
        .output();
    let clippy = match clippy {
        Ok(clippy) => clippy,
        Err(error) => {
            output::line(out, &format!("error: could not run cargo clippy: {error}"));
            return Exit::CheckFailed;
        }
    };
    let actual = match parse_diagnostics(&String::from_utf8_lossy(&clippy.stdout)) {
        Ok(actual) => actual,
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            return Exit::CheckFailed;
        }
    };
    // A failed exit status is the normal result, since the fixtures trip deny-level
    // lints. It only explains an empty result, for example a stale lock file.
    if actual.is_empty() && !clippy.status.success() {
        output::line(
            out,
            &format!(
                "error: cargo clippy reported no diagnostics and exited with {}; see its output above",
                clippy.status
            ),
        );
        return Exit::CheckFailed;
    }
    let mismatches = compare(&expected, &actual);
    if mismatches.is_empty() {
        let count: usize = expected.values().map(BTreeSet::len).sum();
        output::line(
            out,
            &format!("lint-canary: ok ({count} annotated lints fired, nothing else)"),
        );
        return Exit::Success;
    }
    for mismatch in &mismatches {
        output::line(out, &mismatch.to_string());
    }
    output::line(
        out,
        &format!(
            "lint-canary: {} line(s) differ from their annotations. A lint that no longer fires \
             was removed or weakened in [workspace.lints] or clippy.toml \
             (docs/standards/rust.md#workspace-configuration)",
            mismatches.len()
        ),
    );
    Exit::CheckFailed
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

/// The diagnostics in cargo's `--message-format=json` output, by location. A
/// diagnostic is named by its lint or error code; one without a location also carries
/// its message, and one without a code is named by its level and message. Cargo's
/// closing summary lines (no code and no location) are skipped.
pub(crate) fn parse_diagnostics(json_lines: &str) -> Result<Lints, String> {
    let mut lints = Lints::new();
    for line in json_lines.lines().filter(|line| !line.trim().is_empty()) {
        let message: Value = serde_json::from_str(line)
            .map_err(|error| format!("cargo printed a line that is not JSON ({error}): {line}"))?;
        if message["reason"] != "compiler-message" {
            continue;
        }
        let diagnostic = &message["message"];
        let code = diagnostic["code"]["code"].as_str();
        let locations: Vec<Location> = diagnostic["spans"]
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
        let text = diagnostic["message"].as_str().unwrap_or_default();
        let name = match (code, locations.is_empty()) {
            (Some(code), false) => code.to_owned(),
            // Without a location, the message is the only clue (for example, which lint
            // name in `[workspace.lints]` is unknown).
            (Some(code), true) => format!("{code}: {text}"),
            (None, true) => continue,
            (None, false) => format!(
                "{}: {text}",
                diagnostic["level"].as_str().unwrap_or("diagnostic")
            ),
        };
        if locations.is_empty() {
            lints.entry(None).or_default().insert(name);
        } else {
            for location in locations {
                lints.entry(location).or_default().insert(name.clone());
            }
        }
    }
    Ok(lints)
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{Lints, compare, parse_annotations, parse_diagnostics};

    fn at(file: &str, line: u64, names: &[&str]) -> Lints {
        Lints::from([(
            Some((file.to_owned(), line)),
            names.iter().map(|name| (*name).to_owned()).collect(),
        )])
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
            r#"{"reason":"compiler-message","message":{"level":"error","message":"used unwrap","code":{"code":"clippy::unwrap_used"},"spans":[{"file_name":"a.rs","line_start":9,"is_primary":false},{"file_name":"a.rs","line_start":2,"is_primary":true}]}}"#,
            r#"{"reason":"compiler-message","message":{"level":"error","message":"aborting due to 1 previous error","code":null,"spans":[]}}"#,
            r#"{"reason":"compiler-message","message":{"level":"error","message":"expected `;`","code":null,"spans":[{"file_name":"a.rs","line_start":5,"is_primary":true}]}}"#,
            r#"{"reason":"compiler-message","message":{"level":"warning","message":"unknown lint: `clippy::todoo`","code":{"code":"unknown_lints"},"spans":[]}}"#,
        ]
        .join("\n");
        let mut expected = at("a.rs", 2, &["clippy::unwrap_used"]);
        expected.extend(at("a.rs", 5, &["error: expected `;`"]));
        expected.insert(
            None,
            BTreeSet::from(["unknown_lints: unknown lint: `clippy::todoo`".to_owned()]),
        );
        assert_eq!(parse_diagnostics(&json), Ok(expected));
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
}
