//! `cargo xtask backlog stats [--write]`: task totals per epic and per milestone.
//!
//! The tables are the "Totals" section of docs/backlog/README.md, between the
//! `backlog-stats` markers. `--write` regenerates that section, and the backlog check
//! fails when it is stale. The output is byte-identical to
//! docs/backlog/tools/backlog.py's, checked by a golden produced from it.

use std::collections::BTreeMap;
use std::io::Write;

use crate::Exit;
use crate::backlog::{self, Backlog, Level, Milestone};
use crate::output;

const START_MARKER: &str = "<!-- backlog-stats:start -->\n";
const END_MARKER: &str = "<!-- backlog-stats:end -->";

/// Runs the command: prints the tables, and with `write` also puts them in the README.
pub(crate) fn run(write: bool, out: &mut impl Write) -> Exit {
    backlog::with_repository_backlog(out, |backlog, out| {
        let text = stats_text(backlog);
        if write {
            let path = backlog::repository_dir().join("README.md");
            let written = std::fs::read_to_string(&path)
                .map_err(|error| format!("could not read {}: {error}", path.display()))
                .and_then(|readme| {
                    replace_stats(&readme, &text)
                        .ok_or_else(|| format!("{} has no backlog-stats markers", path.display()))
                })
                .and_then(|readme| {
                    std::fs::write(&path, readme)
                        .map_err(|error| format!("could not write {}: {error}", path.display()))
                });
            if let Err(error) = written {
                output::line(out, &format!("error: {error}"));
                return Exit::CheckFailed;
            }
        }
        output::line(out, &text);
        Exit::Success
    })
}

/// The two tables: per epic in README epic-table order, with a total row, then per
/// milestone. Done counts closed tasks (Done or Won't Do).
pub(crate) fn stats_text(backlog: &Backlog) -> String {
    let mut features: BTreeMap<&str, usize> = BTreeMap::new();
    for feature in backlog
        .items
        .iter()
        .filter(|item| item.level == Level::Feature)
    {
        *features.entry(feature.epic.as_str()).or_default() += 1;
    }
    let mut lines = vec![
        "| Epic | Features | Tasks | Done | Open | Highest task id |".to_owned(),
        "|---|---|---|---|---|---|".to_owned(),
    ];
    for code in &backlog.epic_order {
        let tasks: Vec<_> = backlog.tasks().filter(|task| task.epic == *code).collect();
        let done = tasks.iter().filter(|task| task.status.is_closed()).count();
        let highest = tasks
            .iter()
            .map(|task| task.id.as_str())
            .max()
            .unwrap_or("—");
        lines.push(format!(
            "| {code} | {} | {} | {done} | {} | {highest} |",
            features.get(code.as_str()).copied().unwrap_or(0),
            tasks.len(),
            tasks.len() - done
        ));
    }
    let total = backlog.tasks().count();
    let done = backlog
        .tasks()
        .filter(|task| task.status.is_closed())
        .count();
    lines.push(format!(
        "| **Total** | **{}** | **{total}** | **{done}** | **{}** | |",
        features.values().sum::<usize>(),
        total - done
    ));
    lines.push(String::new());
    lines.push("| Milestone | Tasks | Done | Open |".to_owned());
    lines.push("|---|---|---|---|".to_owned());
    for milestone in Milestone::all() {
        let tasks: Vec<_> = backlog
            .tasks()
            .filter(|task| task.milestone == milestone)
            .collect();
        let done = tasks.iter().filter(|task| task.status.is_closed()).count();
        lines.push(format!(
            "| {milestone} | {} | {done} | {} |",
            tasks.len(),
            tasks.len() - done
        ));
    }
    lines.join("\n")
}

/// `readme` with the text between the markers replaced by `text`, or `None` if the
/// markers are missing. Nothing outside the markers changes.
pub(crate) fn replace_stats(readme: &str, text: &str) -> Option<String> {
    let (before, rest) = readme.split_once(START_MARKER)?;
    let (_, after) = rest.split_once(END_MARKER)?;
    Some(format!("{before}{START_MARKER}{text}\n{END_MARKER}{after}"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{replace_stats, stats_text};
    use crate::backlog;

    #[test]
    fn the_tables_match_backlog_py() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backlog");
        let parsed =
            backlog::load(&fixtures.join("valid")).unwrap_or_else(|error| panic!("{error}"));
        let golden = std::fs::read_to_string(fixtures.join("valid.stats.txt"))
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(stats_text(&parsed.backlog), golden.trim_end());
    }

    #[test]
    fn writing_changes_only_the_text_between_the_markers() {
        let readme = "# Backlog\n\n<!-- backlog-stats:start -->\nold\ntables\n<!-- backlog-stats:end -->\n\nMore.\n";
        let written = replace_stats(readme, "| new |");
        assert_eq!(
            written.as_deref(),
            Some(
                "# Backlog\n\n<!-- backlog-stats:start -->\n| new |\n<!-- backlog-stats:end -->\n\nMore.\n"
            )
        );
        assert_eq!(replace_stats("# No markers\n", "x"), None);
    }
}
