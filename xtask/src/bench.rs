//! `cargo xtask bench [--base <ref>]`: runs the gated benchmarks and, with `--base`,
//! fails if one executes more instructions than at `<ref>` by more than its threshold
//! ([ADR-0019](../../docs/adr/0019-benchmark-regression-gating.md)).
//!
//! Gated benchmarks are the bench targets named `*_instructions`. They are gungraun
//! library benchmarks, run under valgrind's callgrind, and identified as
//! `<area>/<operation>/<case>`: their group, function and case. Instruction counts
//! differ between architectures, so the base is built and measured in the same run, in
//! a git worktree under `target/bench/`. Thresholds are in `benches/thresholds.toml`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::Exit;
use crate::output;
use crate::tools::{self, GUNGRAUN_RUNNER};

/// The suffix that marks a bench target as gated.
const GATED_SUFFIX: &str = "_instructions";

/// Where the thresholds are, relative to the repository root.
const THRESHOLDS_FILE: &str = "benches/thresholds.toml";

pub(crate) fn run(base: Option<&str>, out: &mut impl Write) -> Exit {
    match measure_and_compare(base) {
        Ok((report, regressed)) => {
            output::line(out, &report);
            if let Ok(summary) = std::env::var("GITHUB_STEP_SUMMARY") {
                // The job summary is a convenience; the report is already in the log.
                let appended = std::fs::OpenOptions::new()
                    .append(true)
                    .open(summary)
                    .and_then(|mut file| writeln!(file, "{report}"));
                if let Err(error) = appended {
                    output::line(
                        out,
                        &format!("warning: could not write the job summary: {error}"),
                    );
                }
            }
            if regressed {
                output::line(
                    out,
                    &format!(
                        "bench: a benchmark regressed beyond its threshold ({THRESHOLDS_FILE})"
                    ),
                );
                Exit::CheckFailed
            } else {
                Exit::Success
            }
        }
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            Exit::CheckFailed
        }
    }
}

/// Measures the working tree and, with a base, the base; returns the report and whether
/// anything regressed.
fn measure_and_compare(base: Option<&str>) -> Result<(String, bool), String> {
    let root = crate::repo_root();
    let thresholds = Thresholds::parse(&read(&root.join(THRESHOLDS_FILE))?)?;
    check_tools()?;
    let bench_dir = root.join("target").join("bench");
    let base_counts = match base {
        Some(reference) => {
            let tree = bench_dir.join("base-tree");
            let counts = with_worktree(&root, &tree, reference, || {
                measure(&tree, &root, &bench_dir.join("base"))
            })?;
            Some((reference, counts))
        }
        None => None,
    };
    let head = measure(&root, &root, &bench_dir.join("head"))?;
    thresholds.check_overrides(&head)?;
    Ok(match base_counts {
        Some((reference, base)) => {
            let rows = compare(&base, &head, &thresholds);
            let regressed = rows.iter().any(|row| row.change == Change::Regressed);
            (render_comparison(reference, &rows), regressed)
        }
        None => (render_counts(&head), false),
    })
}

/// Fails with the install command when valgrind or the pinned gungraun-runner is
/// missing.
fn check_tools() -> Result<(), String> {
    let valgrind = Command::new("valgrind")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success());
    if !valgrind {
        return Err(
            "valgrind is not installed. The gated benchmarks run only on Linux: install it with your package manager (`sudo apt-get install valgrind`), or on macOS run `cargo xtask bench` in the devcontainer".to_owned(),
        );
    }
    tools::check(&GUNGRAUN_RUNNER).map_err(|problem| problem.to_string())
}

/// Runs `f` with a detached worktree of `reference` at `tree`, removing it afterwards.
fn with_worktree<T>(
    root: &Path,
    tree: &Path,
    reference: &str,
    f: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let git = |args: &[&str]| -> Result<(), String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .map_err(|error| format!("could not run git: {error}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "`git {}` failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    };
    let tree_arg = tree.display().to_string();
    // A worktree left by an interrupted run is replaced; its absence isn't an error.
    let _ = git(&["worktree", "remove", "--force", &tree_arg]);
    git(&[
        "worktree", "add", "--detach", "--force", &tree_arg, reference,
    ])?;
    let result = f();
    git(&["worktree", "remove", "--force", &tree_arg])?;
    result
}

/// What callgrind counted for one benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Counts {
    instructions: u64,
    estimated_cycles: u64,
}

/// Runs every gated target of the workspace at `dir` and reads the results, sharing the
/// repository's target directory so unchanged dependencies build once.
fn measure(dir: &Path, root: &Path, home: &Path) -> Result<BTreeMap<String, Counts>, String> {
    remove_dir(home)?;
    for (package, target) in gated_targets(dir)? {
        let status = Command::new(tools::cargo_program())
            .args([
                "bench",
                "--locked",
                "--package",
                &package,
                "--bench",
                &target,
            ])
            .env("CARGO_TARGET_DIR", root.join("target"))
            .env("GUNGRAUN_HOME", home)
            .env("GUNGRAUN_SAVE_SUMMARY", "json")
            .current_dir(dir)
            .status()
            .map_err(|error| format!("could not run cargo bench: {error}"))?;
        if !status.success() {
            return Err(format!(
                "`cargo bench --package {package} --bench {target}` failed in {}",
                dir.display()
            ));
        }
    }
    read_summaries(home)
}

/// The workspace's gated bench targets, as `(package, target)`.
fn gated_targets(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let output = Command::new(tools::cargo_program())
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(dir)
        .output()
        .map_err(|error| format!("could not run cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed in {}: {}",
            dir.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("could not read cargo metadata: {error}"))?;
    Ok(gated_targets_in(&metadata))
}

fn gated_targets_in(metadata: &Value) -> Vec<(String, String)> {
    let mut targets = Vec::new();
    for package in metadata["packages"].as_array().into_iter().flatten() {
        for target in package["targets"].as_array().into_iter().flatten() {
            let is_bench = target["kind"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bench"));
            if let (true, Some(name), Some(package)) =
                (is_bench, target["name"].as_str(), package["name"].as_str())
                && name.ends_with(GATED_SUFFIX)
            {
                targets.push((package.to_owned(), name.to_owned()));
            }
        }
    }
    targets.sort();
    targets
}

/// Every `summary.json` gungraun wrote under `home`, by benchmark id.
fn read_summaries(home: &Path) -> Result<BTreeMap<String, Counts>, String> {
    let mut counts = BTreeMap::new();
    let mut pending = vec![home.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            // No gated benchmarks ran, so nothing was written.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("could not read {}: {error}", dir.display())),
        };
        for entry in entries {
            let path = entry
                .map_err(|error| format!("could not read {}: {error}", dir.display()))?
                .path();
            if path.is_dir() {
                pending.push(path);
            } else if path.file_name().is_some_and(|name| name == "summary.json") {
                let (id, value) = parse_summary(&read(&path)?)
                    .map_err(|problem| format!("{}: {problem}", path.display()))?;
                if counts.insert(id.clone(), value).is_some() {
                    return Err(format!(
                        "two gated benchmarks are named {id}; benchmark groups must be unique across the workspace"
                    ));
                }
            }
        }
    }
    Ok(counts)
}

/// A benchmark's id and its callgrind counts, from one gungraun `summary.json`.
fn parse_summary(text: &str) -> Result<(String, Counts), String> {
    let summary: Value =
        serde_json::from_str(text).map_err(|error| format!("not JSON: {error}"))?;
    let field = |name: &str| summary[name].as_str().ok_or_else(|| format!("no `{name}`"));
    let id = format!(
        "{}/{}/{}",
        field("group")?,
        field("function_name")?,
        field("id")?
    );
    let callgrind = summary["profiles"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|profile| profile["tool"] == "Callgrind")
        .ok_or("no callgrind profile")?;
    let metric = |name: &str| {
        callgrind["data"]["total"]["metrics"][name]["values"]["new"]
            .as_u64()
            .ok_or_else(|| format!("no `{name}` count"))
    };
    Ok((
        id,
        Counts {
            instructions: metric("Ir")?,
            estimated_cycles: metric("EstimatedCycles")?,
        },
    ))
}

/// The thresholds in `benches/thresholds.toml`.
#[derive(Debug, Clone, PartialEq)]
struct Thresholds {
    default_percent: f64,
    /// Overrides by benchmark id.
    overrides: BTreeMap<String, f64>,
}

impl Thresholds {
    fn parse(text: &str) -> Result<Thresholds, String> {
        let file: Value = toml::from_str(text)
            .map_err(|error| format!("{THRESHOLDS_FILE}: not valid TOML: {error}"))?;
        let percent = |value: &Value, what: &str| {
            value
                .as_f64()
                .filter(|percent| *percent >= 0.0)
                .ok_or_else(|| format!("{THRESHOLDS_FILE}: {what} must be a non-negative number"))
        };
        let default_percent = percent(&file["default_percent"], "`default_percent`")?;
        let mut overrides = BTreeMap::new();
        for entry in file["override"].as_array().into_iter().flatten() {
            let benchmark = entry["benchmark"]
                .as_str()
                .ok_or_else(|| format!("{THRESHOLDS_FILE}: an override has no `benchmark`"))?;
            let what = format!("the override for {benchmark}: `percent`");
            let value = percent(&entry["percent"], &what)?;
            if entry["reason"]
                .as_str()
                .is_none_or(|reason| reason.trim().is_empty())
            {
                return Err(format!(
                    "{THRESHOLDS_FILE}: the override for {benchmark} needs a `reason`"
                ));
            }
            if overrides.insert(benchmark.to_owned(), value).is_some() {
                return Err(format!(
                    "{THRESHOLDS_FILE}: {benchmark} is overridden twice"
                ));
            }
        }
        Ok(Thresholds {
            default_percent,
            overrides,
        })
    }

    /// Fails for an override naming a benchmark that doesn't exist, so none goes stale.
    fn check_overrides(&self, benchmarks: &BTreeMap<String, Counts>) -> Result<(), String> {
        match self
            .overrides
            .keys()
            .find(|id| !benchmarks.contains_key(*id))
        {
            Some(id) => Err(format!(
                "{THRESHOLDS_FILE}: the override for {id} names no gated benchmark"
            )),
            None => Ok(()),
        }
    }

    fn percent_for(&self, id: &str) -> f64 {
        self.overrides
            .get(id)
            .copied()
            .unwrap_or(self.default_percent)
    }
}

/// How a benchmark's count moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Change {
    /// Within its threshold, or fewer instructions.
    Within,
    /// More instructions than its threshold allows.
    Regressed,
    /// Only at the head.
    New,
    /// Only at the base.
    Removed,
}

/// One benchmark's comparison.
#[derive(Debug, Clone, PartialEq)]
struct Row {
    id: String,
    base: Option<Counts>,
    head: Option<Counts>,
    /// The change in instructions, in percent of the base.
    percent: Option<f64>,
    threshold: f64,
    change: Change,
}

fn compare(
    base: &BTreeMap<String, Counts>,
    head: &BTreeMap<String, Counts>,
    thresholds: &Thresholds,
) -> Vec<Row> {
    let mut ids: Vec<&String> = base.keys().chain(head.keys()).collect();
    ids.sort();
    ids.dedup();
    ids.into_iter()
        .map(|id| {
            let (before, after) = (base.get(id).copied(), head.get(id).copied());
            let threshold = thresholds.percent_for(id);
            let percent = before.zip(after).map(|(before, after)| {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "a percentage for display and a threshold; counts are far below 2^52"
                )]
                let (before, after) = (before.instructions as f64, after.instructions as f64);
                if before == 0.0 {
                    0.0
                } else {
                    (after - before) / before * 100.0
                }
            });
            let change = match (before, after, percent) {
                (None, _, _) => Change::New,
                (_, None, _) => Change::Removed,
                (_, _, Some(percent)) if percent > threshold => Change::Regressed,
                _ => Change::Within,
            };
            Row {
                id: id.clone(),
                base: before,
                head: after,
                percent,
                threshold,
                change,
            }
        })
        .collect()
}

fn render_comparison(reference: &str, rows: &[Row]) -> String {
    let mut text = format!(
        "## Gated benchmarks: instructions, against {reference}\n\n| Benchmark | Base | This change | Change | Threshold | Estimated cycles (base → this) | |\n|---|---:|---:|---:|---:|---:|---|\n"
    );
    let count = |counts: Option<Counts>, metric: fn(Counts) -> u64| {
        counts.map_or_else(|| "–".to_owned(), |counts| metric(counts).to_string())
    };
    for row in rows {
        let verdict = match row.change {
            Change::Within => "ok",
            Change::Regressed => "**regressed**",
            Change::New => "new",
            Change::Removed => "removed",
        };
        let percent = row
            .percent
            .map_or_else(|| "–".to_owned(), |percent| format!("{percent:+.2}%"));
        // Writing to a String can't fail.
        let _ = writeln!(
            text,
            "| `{}` | {} | {} | {percent} | {:.1}% | {} → {} | {verdict} |",
            row.id,
            count(row.base, |counts| counts.instructions),
            count(row.head, |counts| counts.instructions),
            row.threshold,
            count(row.base, |counts| counts.estimated_cycles),
            count(row.head, |counts| counts.estimated_cycles),
        );
    }
    text
}

fn render_counts(head: &BTreeMap<String, Counts>) -> String {
    let mut text = "## Gated benchmarks: instructions\n\n| Benchmark | Instructions | Estimated cycles |\n|---|---:|---:|\n".to_owned();
    for (id, counts) in head {
        // Writing to a String can't fail.
        let _ = writeln!(
            text,
            "| `{id}` | {} | {} |",
            counts.instructions, counts.estimated_cycles
        );
    }
    text
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))
}

fn remove_dir(dir: &Path) -> Result<(), String> {
    match std::fs::remove_dir_all(dir) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(format!("could not remove {}: {error}", dir.display()))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::{Change, Counts, Thresholds, compare, gated_targets_in, parse_summary};

    fn counts(instructions: u64) -> Counts {
        Counts {
            instructions,
            estimated_cycles: instructions * 2,
        }
    }

    #[test]
    fn a_summary_gives_the_id_and_callgrind_counts() {
        let summary = json!({
            "group": "types", "function_name": "murmur2", "id": "size_16",
            "profiles": [{ "tool": "Callgrind", "data": { "total": { "metrics": {
                "Ir": { "values": { "new": 166 } },
                "EstimatedCycles": { "values": { "new": 460 } },
            }}}}],
        });
        assert_eq!(
            parse_summary(&summary.to_string()),
            Ok((
                "types/murmur2/size_16".to_owned(),
                Counts {
                    instructions: 166,
                    estimated_cycles: 460
                }
            ))
        );
        assert!(parse_summary(r#"{"group": "types"}"#).is_err());
    }

    #[test]
    fn only_bench_targets_ending_in_instructions_are_gated() {
        let metadata = json!({ "packages": [{
            "name": "alluvium-types",
            "targets": [
                { "name": "types", "kind": ["bench"] },
                { "name": "types_instructions", "kind": ["bench"] },
                { "name": "other_instructions", "kind": ["test"] },
            ],
        }]});
        assert_eq!(
            gated_targets_in(&metadata),
            [("alluvium-types".to_owned(), "types_instructions".to_owned())]
        );
    }

    #[test]
    fn thresholds_need_a_reason_for_every_override() {
        let parsed = Thresholds::parse(
            "default_percent = 3.0\n[[override]]\nbenchmark = \"a/b/c\"\npercent = 5.0\nreason = \"tiny\"\n",
        )
        .unwrap();
        assert_eq!(parsed.percent_for("a/b/c"), 5.0);
        assert_eq!(parsed.percent_for("x/y/z"), 3.0);
        for bad in [
            "",
            "default_percent = -1.0",
            "default_percent = 3.0\n[[override]]\nbenchmark = \"a/b/c\"\npercent = 5.0\n",
            "default_percent = 3.0\n[[override]]\nbenchmark = \"a/b/c\"\npercent = 5.0\nreason = \"r\"\n[[override]]\nbenchmark = \"a/b/c\"\npercent = 6.0\nreason = \"r\"\n",
        ] {
            assert!(Thresholds::parse(bad).is_err(), "{bad}");
        }
        let benchmarks = BTreeMap::from([("x/y/z".to_owned(), counts(1))]);
        assert!(parsed.check_overrides(&benchmarks).is_err());
    }

    #[test]
    fn a_regression_beyond_the_threshold_fails_and_others_pass() {
        let thresholds = Thresholds::parse("default_percent = 3.0").unwrap();
        let base = BTreeMap::from([
            ("a/same/1".to_owned(), counts(1000)),
            ("a/slower/1".to_owned(), counts(1000)),
            ("a/slightly/1".to_owned(), counts(1000)),
            ("a/faster/1".to_owned(), counts(1000)),
            ("a/removed/1".to_owned(), counts(1000)),
        ]);
        let head = BTreeMap::from([
            ("a/same/1".to_owned(), counts(1000)),
            ("a/slower/1".to_owned(), counts(1031)),
            ("a/slightly/1".to_owned(), counts(1030)),
            ("a/faster/1".to_owned(), counts(500)),
            ("a/new/1".to_owned(), counts(1)),
        ]);
        let changes: Vec<(String, Change)> = compare(&base, &head, &thresholds)
            .into_iter()
            .map(|row| (row.id, row.change))
            .collect();
        assert_eq!(
            changes,
            [
                ("a/faster/1".to_owned(), Change::Within),
                ("a/new/1".to_owned(), Change::New),
                ("a/removed/1".to_owned(), Change::Removed),
                ("a/same/1".to_owned(), Change::Within),
                ("a/slightly/1".to_owned(), Change::Within),
                ("a/slower/1".to_owned(), Change::Regressed),
            ]
        );
    }
}
