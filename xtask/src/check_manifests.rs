//! `cargo xtask check-manifests`: the workspace configuration matches the standards.
//!
//! Rules:
//!
//! 1. `[workspace.lints]` in the root `Cargo.toml` equals the lint block in
//!    docs/standards/rust.md#workspace-configuration.
//! 2. `clippy.toml` equals the clippy.toml block there.
//! 3. Every workspace member declares `[lints] workspace = true`, so it inherits the
//!    lints.
//! 4. Every `[advisories] ignore` entry in deny.toml is a table with an `id`, a
//!    `reason` and an `expiry` (docs/standards/security.md#dependencies). cargo-deny
//!    itself rejects a malformed date and reports the advisory again once the date
//!    passes; it does not require either field.
//!
//! Files are parsed and compared as TOML values: comments, formatting, and the order of
//! keys and of array entries don't matter. Every difference is reported in one run,
//! naming the file and the dotted key.

use std::io::Write;
use std::path::Path;

use serde_json::Value;

use crate::Exit;
use crate::graph;
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
        problems.extend(check_member(&file, &parse(&file, &text)?));
    }
    Ok(problems)
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

/// Rule 3: a member manifest declares `[lints] workspace = true`.
pub(crate) fn check_member(file: &str, manifest: &Value) -> Option<String> {
    (manifest["lints"]["workspace"] != true).then(|| {
        format!(
            "{file}: `lints.workspace` must be true, so the crate inherits [workspace.lints] \
             (add `[lints]` with `workspace = true`)"
        )
    })
}

/// Rule 4: every advisory ignore in deny.toml carries a reason and an expiry.
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
        Standard, check_advisory_ignores, check_clippy, check_lints, check_member, parse, standard,
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
    fn a_member_must_inherit_the_workspace_lints() {
        let inherits = "[package]\nname = \"a\"\n\n[lints]\nworkspace = true\n";
        assert_eq!(check_member("a/Cargo.toml", &toml(inherits)), None);
        for manifest in [
            "[package]\nname = \"a\"\n",
            "[package]\nname = \"a\"\n\n[lints]\nworkspace = false\n",
        ] {
            assert!(
                check_member("a/Cargo.toml", &toml(manifest))
                    .is_some_and(|problem| problem
                        .starts_with("a/Cargo.toml: `lints.workspace` must be true")),
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
}
