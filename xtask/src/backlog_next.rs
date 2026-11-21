//! `cargo xtask backlog next` and `backlog show`: what to work on, and one item in
//! detail.
//!
//! The selection rules are in docs/backlog/README.md#choosing-the-next-task. A task is
//! ready when it is `To Do` and every task it waits for
//! ([`Backlog::effective_deps`]) is closed. Ready tasks are ordered by milestone,
//! priority, the epic's position in the README epic table, then id. `next` shows the
//! ready tasks of the lowest milestone that still has open tasks, or, when that
//! milestone has none ready, the first ready tasks of later milestones.
//!
//! `next` prints exactly what docs/backlog/tools/backlog.py prints, checked by goldens
//! produced from it.

use std::io::Write;

use crate::Exit;
use crate::backlog::{self, Backlog, Item, Level, Milestone, Status};
use crate::output;

/// The number of tasks `next` shows without `--all` or `-n`.
pub(crate) const DEFAULT_COUNT: usize = 10;

/// Runs `backlog next`.
pub(crate) fn run_next(all: bool, count: usize, out: &mut impl Write) -> Exit {
    backlog::with_repository_backlog(out, |backlog, out| {
        for line in next_lines(backlog, all, count) {
            output::line(out, &line);
        }
        Exit::Success
    })
}

/// Runs `backlog show <id>`.
pub(crate) fn run_show(id: &str, out: &mut impl Write) -> Exit {
    backlog::with_repository_backlog(out, |backlog, out| {
        let Some(lines) = show_lines(backlog, id) else {
            output::line(out, &format!("error: unknown id {id}"));
            return Exit::CheckFailed;
        };
        for line in lines {
            output::line(out, &line);
        }
        Exit::Success
    })
}

/// The lowest milestone with an open task, and every ready task in selection order.
pub(crate) fn ready(backlog: &Backlog) -> (Option<Milestone>, Vec<&Item>) {
    let current = backlog
        .tasks()
        .filter(|task| !task.status.is_closed())
        .map(|task| task.milestone)
        .min();
    let mut ready: Vec<&Item> = backlog
        .tasks()
        .filter(|task| task.status == Status::ToDo)
        .filter(|task| {
            backlog
                .effective_deps(task)
                .iter()
                .all(|dep| backlog.get(dep).is_some_and(|dep| dep.status.is_closed()))
        })
        .collect();
    let epic_rank = |epic: &str| {
        backlog
            .epic_order
            .iter()
            .position(|code| code == epic)
            .unwrap_or(usize::MAX)
    };
    ready.sort_by(|a, b| {
        (a.milestone, a.priority, epic_rank(&a.epic), &a.id).cmp(&(
            b.milestone,
            b.priority,
            epic_rank(&b.epic),
            &b.id,
        ))
    });
    (current, ready)
}

/// The output of `next`: every ready task with `all`, otherwise the first `count` of
/// the current milestone (or of later ones, when the current milestone has none ready).
pub(crate) fn next_lines(backlog: &Backlog, all: bool, count: usize) -> Vec<String> {
    let (current, mut ready) = ready(backlog);
    let Some(current) = current else {
        return vec!["Everything is Done or Won't Do.".to_owned()];
    };
    if !all {
        let in_current: Vec<&Item> = ready
            .iter()
            .copied()
            .filter(|task| task.milestone == current)
            .collect();
        if !in_current.is_empty() {
            ready = in_current;
        }
        ready.truncate(count);
    }
    let mut lines = vec![format!("Current milestone: {current}")];
    for task in &ready {
        lines.push(format!(
            "{:16} {} {:4} {:6} {}",
            task.id,
            task.priority,
            task.milestone.to_string(),
            task.kind.to_string(),
            task.title
        ));
        lines.push(format!(
            "{:16} {}:{} · Done when: {}",
            "",
            task.file,
            task.line,
            task.done_when.as_deref().unwrap_or_default()
        ));
    }
    if ready.is_empty() {
        lines.push("No ready tasks: everything open is blocked or claimed.".to_owned());
    }
    lines
}

/// The output of `show`: the item, its fields and trailers, what it waits for (for a
/// task, after expanding feature dependencies), and what depends on it.
pub(crate) fn show_lines(backlog: &Backlog, id: &str) -> Option<Vec<String>> {
    let item = backlog.get(id)?;
    let mut lines = vec![
        format!("{} [{}] {}", item.id, item.status, item.title),
        format!(
            "  {} · {} · {} · {}:{}",
            item.priority, item.milestone, item.kind, item.file, item.line
        ),
    ];
    if let Some(done_when) = &item.done_when {
        lines.push(format!("  Done when: {done_when}"));
    }
    if !item.labels.is_empty() {
        lines.push(format!("  labels: {}", item.labels.join(", ")));
    }
    if !item.reference.is_empty() {
        lines.push(format!("  ref: {}", item.reference));
    }
    for (key, value) in &item.trailers {
        lines.push(format!("  {key}: {value}"));
    }
    let needs: Vec<String> = match item.level {
        Level::Task => backlog.effective_deps(item).into_iter().collect(),
        Level::Feature => item.deps.clone(),
    };
    for dep in &needs {
        lines.push(match backlog.get(dep) {
            Some(dep) => format!("  needs {} [{}] {}", dep.id, dep.status, dep.title),
            None => format!("  needs {dep} (unknown id)"),
        });
    }
    for other in backlog
        .items
        .iter()
        .filter(|other| other.deps.contains(&item.id))
    {
        lines.push(format!(
            "  needed by {} [{}] {}",
            other.id, other.status, other.title
        ));
    }
    Some(lines)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{next_lines, show_lines};
    use crate::backlog::{self, Backlog};

    fn fixture_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backlog")
    }

    fn valid() -> Backlog {
        backlog::load(&fixture_dir().join("valid"))
            .unwrap_or_else(|error| panic!("{error}"))
            .backlog
    }

    fn golden(name: &str) -> Vec<String> {
        std::fs::read_to_string(fixture_dir().join(name))
            .unwrap_or_else(|error| panic!("{name}: {error}"))
            .lines()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn next_matches_backlog_py() {
        let backlog = valid();
        assert_eq!(next_lines(&backlog, false, 10), golden("valid.next.txt"));
        assert_eq!(next_lines(&backlog, true, 10), golden("valid.next-all.txt"));
    }

    #[test]
    fn next_works_ahead_when_the_current_milestone_has_nothing_ready() {
        let dir = fixture_dir().join("valid");
        let read = |name: &str| {
            std::fs::read_to_string(dir.join(name)).unwrap_or_else(|error| panic!("{error}"))
        };
        let claim = |text: String, id: &str| {
            text.replace(
                &format!("### [To Do] {id}"),
                &format!("### [In Progress] {id}"),
            )
        };
        let files = [
            (
                "01-aaa-alpha.md".to_owned(),
                claim(read("01-aaa-alpha.md"), "AAA-0002"),
            ),
            (
                "02-bbb-beta.md".to_owned(),
                claim(read("02-bbb-beta.md"), "BBB-0001"),
            ),
        ];
        let parsed = backlog::parse(&read("README.md"), &files);
        assert_eq!(parsed.errors, Vec::<String>::new());
        assert_eq!(
            next_lines(&parsed.backlog, false, 10),
            golden("work-ahead.next.txt")
        );
    }

    #[test]
    fn next_limits_the_count_and_reports_when_nothing_is_ready() {
        let backlog = valid();
        assert_eq!(next_lines(&backlog, false, 1).len(), 3);
        assert_eq!(
            next_lines(&Backlog::default(), false, 10),
            ["Everything is Done or Won't Do."]
        );
    }

    #[test]
    fn show_lists_fields_trailers_and_both_directions_of_dependencies() {
        let backlog = valid();
        assert_eq!(
            show_lines(&backlog, "AAA-0004"),
            Some(
                [
                    "AAA-0004 [In Progress] A claimed spike",
                    "  P2 · M0 · Spike · 01-aaa-alpha.md:30",
                    "  Done when: An ADR is accepted.",
                    "  labels: research",
                    "  ref: KIP-1 · section 3",
                    "  Owner: someone",
                    "  Notes: Claimed tasks are skipped by `next`.",
                    "  needs AAA-0001 [Done] A finished task",
                    "  needed by AAA-0003 [To Do] A task waiting for a claimed task",
                ]
                .map(str::to_owned)
                .to_vec()
            )
        );
        let feature = show_lines(&backlog, "BBB-F01").unwrap_or_default();
        assert_eq!(
            feature.get(2).map(String::as_str),
            Some("  needs AAA-F01 [In Progress] First feature")
        );
        assert_eq!(show_lines(&backlog, "AAA-9999"), None);
    }
}
