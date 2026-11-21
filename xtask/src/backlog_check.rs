//! `cargo xtask backlog check`: the backlog follows the rules in docs/backlog/README.md.
//!
//! On top of the format errors found while parsing, it checks:
//!
//! - every dependency exists, is not the item itself, names a feature when it crosses
//!   epics, and is not a task in a later milestone; a warning flags a task depending on
//!   a feature with no tasks at or before the task's milestone;
//! - each feature has tasks (unless `--allow-empty-features`), its milestone is its
//!   earliest task's, and its status follows from its tasks' statuses;
//! - no dependency cycles between tasks, after expanding feature dependencies;
//! - the README epic table: numbers match file names, files exist, every epic is
//!   listed, and the totals are current (unless `--allow-empty-features`).
//!
//! `--epic CODE` (repeatable) keeps only messages about those epics; everything is
//! still parsed. Goldens in `xtask/tests/fixtures/backlog/` specify the output.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;

use crate::Exit;
use crate::backlog::{self, Backlog, Item, Level, Status};
use crate::backlog_stats;
use crate::output;

/// Options for `backlog check`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CheckOptions {
    /// Construction mode: allow features without tasks and skip the totals check.
    pub(crate) allow_empty_features: bool,
    /// Report only problems in these epics; empty means all.
    pub(crate) epics: Vec<String>,
}

/// Runs the command on the repository backlog.
pub(crate) fn run(options: &CheckOptions, out: &mut impl Write) -> Exit {
    match check_dir(&backlog::repository_dir(), options) {
        Ok((lines, passed)) => {
            for line in lines {
                output::line(out, &line);
            }
            if passed {
                Exit::Success
            } else {
                Exit::CheckFailed
            }
        }
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            Exit::CheckFailed
        }
    }
}

/// Checks the backlog in `dir`. Returns the report lines and whether there were no
/// errors (warnings don't fail the check).
pub(crate) fn check_dir(dir: &Path, options: &CheckOptions) -> Result<(Vec<String>, bool), String> {
    let parsed = backlog::load(dir)?;
    let readme_path = dir.join("README.md");
    let readme = std::fs::read_to_string(&readme_path)
        .map_err(|error| format!("could not read {}: {error}", readme_path.display()))?;
    let mut errors = parsed.errors.clone();
    let mut warnings = Vec::new();
    let backlog = &parsed.backlog;
    check_items(
        backlog,
        options.allow_empty_features,
        &mut errors,
        &mut warnings,
    );
    check_cycles(backlog, &mut errors);
    check_readme(
        backlog,
        &readme,
        dir,
        options.allow_empty_features,
        &mut errors,
    );
    if !options.epics.is_empty() {
        let keep = |message: &String| about_epics(message, &options.epics);
        errors.retain(keep);
        warnings.retain(keep);
    }
    let mut lines: Vec<String> = warnings
        .iter()
        .map(|warning| format!("warning: {warning}"))
        .chain(errors.iter().map(|error| format!("error: {error}")))
        .collect();
    lines.push(format!(
        "{} items ({} tasks), {} errors, {} warnings",
        backlog.items.len(),
        backlog.tasks().count(),
        errors.len(),
        warnings.len()
    ));
    Ok((lines, errors.is_empty()))
}

/// True if `message` concerns one of `epics`: it names an epic file (`02-fnd-….md`
/// contains `-fnd-`) or an item id (`FND-…`).
fn about_epics(message: &str, epics: &[String]) -> bool {
    epics.iter().any(|code| {
        message.contains(&format!("-{}-", code.to_lowercase()))
            || message.contains(&format!("{code}-"))
    })
}

fn check_items(
    backlog: &Backlog,
    allow_empty_features: bool,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    for item in &backlog.items {
        let location = format!("{}:{}", item.file, item.line);
        let id = &item.id;
        for dep_id in &item.deps {
            let Some(dep) = backlog.get(dep_id) else {
                errors.push(format!("{location}: {id}: unknown dependency {dep_id}"));
                continue;
            };
            if dep_id == id {
                errors.push(format!("{location}: {id}: depends on itself"));
            }
            if dep.epic != item.epic && dep.level != Level::Feature {
                errors.push(format!(
                    "{location}: {id}: cross-epic dependency {dep_id} must be a feature id"
                ));
            }
            if dep.level == Level::Task && dep.milestone > item.milestone {
                errors.push(format!(
                    "{location}: {id} ({}) depends on {dep_id} in later milestone {}",
                    item.milestone, dep.milestone
                ));
            }
            let dep_tasks = tasks_of(backlog, dep);
            if dep.level == Level::Feature
                && !dep_tasks.is_empty()
                && item.level == Level::Task
                && !dep_tasks
                    .iter()
                    .any(|task| task.milestone <= item.milestone)
            {
                warnings.push(format!(
                    "{location}: {id} ({}) depends on feature {dep_id}, which has no tasks at or before {}",
                    item.milestone, item.milestone
                ));
            }
        }
        if item.level == Level::Feature {
            check_feature(backlog, item, &location, allow_empty_features, errors);
        }
    }
}

fn tasks_of<'a>(backlog: &'a Backlog, feature: &Item) -> Vec<&'a Item> {
    feature
        .tasks
        .iter()
        .filter_map(|task| backlog.get(task))
        .collect()
}

fn check_feature(
    backlog: &Backlog,
    feature: &Item,
    location: &str,
    allow_empty_features: bool,
    errors: &mut Vec<String>,
) {
    let id = &feature.id;
    let tasks = tasks_of(backlog, feature);
    let Some(earliest) = tasks.iter().map(|task| task.milestone).min() else {
        if !allow_empty_features {
            errors.push(format!("{location}: feature {id} has no tasks"));
        }
        return;
    };
    if feature.milestone != earliest {
        errors.push(format!(
            "{location}: feature {id} milestone {} must equal its earliest task milestone {earliest}",
            feature.milestone
        ));
    }
    let expected = if tasks.iter().all(|task| task.status.is_closed()) {
        Status::Done
    } else if tasks.iter().any(|task| {
        matches!(
            task.status,
            Status::InProgress | Status::InReview | Status::Done | Status::Blocked
        )
    }) {
        Status::InProgress
    } else {
        Status::ToDo
    };
    if feature.status != expected {
        errors.push(format!(
            "{location}: feature {id} status should be '{expected}' given its tasks"
        ));
    }
}

/// Depth-first search over task dependencies (feature dependencies expanded), visiting
/// tasks in file order and dependencies in id order, so each cycle
/// is reported once and in the same form.
fn check_cycles(backlog: &Backlog, errors: &mut Vec<String>) {
    let graph: BTreeMap<&str, Vec<String>> = backlog
        .tasks()
        .map(|task| {
            (
                task.id.as_str(),
                backlog.effective_deps(task).into_iter().collect(),
            )
        })
        .collect();
    let mut finished: BTreeSet<&str> = BTreeSet::new();
    let mut visiting: BTreeSet<&str> = BTreeSet::new();
    for start in backlog.tasks().map(|task| task.id.as_str()) {
        if finished.contains(start) || visiting.contains(start) {
            continue;
        }
        let mut stack: Vec<(&str, usize)> = vec![(start, 0)];
        visiting.insert(start);
        while let Some((node, next)) = stack.last_mut() {
            let node: &str = node;
            let Some(dep) = graph.get(node).and_then(|deps| deps.get(*next)) else {
                visiting.remove(node);
                finished.insert(node);
                stack.pop();
                continue;
            };
            *next += 1;
            let Some((dep, _)) = graph.get_key_value(dep.as_str()) else {
                continue;
            };
            if visiting.contains(dep) {
                let path: Vec<&str> = stack.iter().map(|(id, _)| *id).collect();
                let from = path.iter().position(|id| id == dep).unwrap_or(0);
                let cycle: Vec<&str> = path
                    .get(from..)
                    .unwrap_or_default()
                    .iter()
                    .copied()
                    .chain([*dep])
                    .collect();
                errors.push(format!("dependency cycle: {}", cycle.join(" -> ")));
            } else if !finished.contains(dep) {
                visiting.insert(dep);
                stack.push((dep, 0));
            }
        }
    }
}

fn check_readme(
    backlog: &Backlog,
    readme: &str,
    dir: &Path,
    allow_empty_features: bool,
    errors: &mut Vec<String>,
) {
    for row in backlog::epic_rows(readme) {
        let prefix = format!("{:0>2}-", row.number.trim_start_matches('0'));
        let prefix = if prefix == "-" {
            "00-".to_owned()
        } else {
            prefix
        };
        if !row.file.starts_with(&prefix) {
            errors.push(format!(
                "README.md: epic {} is #{} but links to {}",
                row.code, row.number, row.file
            ));
        }
        if !dir.join(&row.file).exists() {
            errors.push(format!(
                "README.md: epic table links to missing file {}",
                row.file
            ));
        }
    }
    match backlog_stats::current_stats(readme) {
        None => errors.push("README.md: missing backlog-stats markers".to_owned()),
        Some(current) => {
            let stale = current.trim() != backlog_stats::stats_text(backlog).trim();
            if !allow_empty_features && backlog.tasks().next().is_some() && stale {
                errors.push(
                    "README.md: totals are out of date; run `cargo xtask backlog stats --write`"
                        .to_owned(),
                );
            }
        }
    }
    let listed: BTreeSet<&str> = backlog.epic_order.iter().map(String::as_str).collect();
    let missing: BTreeSet<&str> = backlog
        .items
        .iter()
        .map(|item| item.epic.as_str())
        .filter(|epic| !listed.contains(epic))
        .collect();
    if !missing.is_empty() {
        let list: Vec<String> = missing.iter().map(|code| format!("'{code}'")).collect();
        errors.push(format!(
            "README.md: epics missing from the epic table: [{}]",
            list.join(", ")
        ));
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{CheckOptions, check_dir};

    fn check(fixture: &str, allow_empty_features: bool, epics: &[&str]) -> (Vec<String>, bool) {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backlog");
        let options = CheckOptions {
            allow_empty_features,
            epics: epics.iter().map(|code| (*code).to_owned()).collect(),
        };
        check_dir(&dir.join(fixture), &options).unwrap_or_else(|error| panic!("{error}"))
    }

    fn golden(name: &str) -> Vec<String> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/backlog")
            .join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
            .lines()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn a_valid_backlog_passes() {
        assert_eq!(
            check("valid", false, &[]),
            (golden("valid.check.txt"), true)
        );
    }

    #[test]
    fn every_rule_reports_its_error() {
        assert_eq!(
            check("check-errors", false, &[]),
            (golden("check-errors.check.txt"), false)
        );
    }

    #[test]
    fn allowing_empty_features_skips_the_empty_feature_and_totals_rules() {
        assert_eq!(
            check("check-errors", true, &[]),
            (golden("check-errors.allow-empty.txt"), false)
        );
    }

    #[test]
    fn epic_filters_keep_messages_naming_the_epic_or_its_files() {
        assert_eq!(
            check("check-errors", false, &["FFF"]),
            (golden("check-errors.epic-fff.txt"), false)
        );
        // A file-level error has no item id; it is kept through the file name.
        assert_eq!(
            check("malformed", true, &["DDD"]),
            (golden("malformed.epic-ddd.txt"), false)
        );
    }
}
