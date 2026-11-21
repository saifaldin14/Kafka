//! The only place `xtask` writes to the terminal.
//!
//! Writes go through `std::io::Write`, so callers pass stdout, stderr or a buffer in
//! tests. A failed write (for example stdout piped into `head`) is not an `xtask`
//! failure, so write errors are deliberately ignored here and nowhere else.

use std::io::Write;

use crate::ci::CiStep;
use crate::cli::{COMMANDS, UsageError};
use crate::steps::{Outcome, Report};

/// Writes the usage text and the command list.
pub(crate) fn usage(out: &mut impl Write) {
    write_all(out, "Usage: cargo xtask <command>\n\nCommands:\n");
    let rows: Vec<[String; 2]> = COMMANDS
        .iter()
        .map(|(name, summary)| [(*name).to_owned(), (*summary).to_owned()])
        .collect();
    table(out, &rows);
}

/// Writes a usage error followed by the usage text.
pub(crate) fn usage_error(out: &mut impl Write, error: &UsageError) {
    write_all(out, &format!("error: {error}\n\n"));
    usage(out);
}

/// Writes one line of text.
pub(crate) fn line(out: &mut impl Write, text: &str) {
    write_all(out, &format!("{text}\n"));
}

/// Writes the CI steps in run order.
pub(crate) fn step_list(out: &mut impl Write, steps: &[CiStep]) {
    let rows: Vec<[String; 2]> = steps
        .iter()
        .map(|step| [step.name.to_owned(), step.summary.to_owned()])
        .collect();
    table(out, &rows);
}

/// Writes the header that starts a step: `==> [n/N] <step> (<command>)`.
pub(crate) fn step_header(
    out: &mut impl Write,
    number: usize,
    total: usize,
    name: &str,
    command: &str,
) {
    write_all(
        out,
        &format!("\n==> [{number}/{total}] {name} ({command})\n"),
    );
    // Flushed so the header appears before the child's output, which goes straight to
    // the terminal. Ignored for the same reason as `write_all`.
    let _ = out.flush();
}

/// Writes the summary of every step: outcome, name and duration.
pub(crate) fn summary(out: &mut impl Write, reports: &[Report]) {
    let width = reports
        .iter()
        .map(|report| report.name.len())
        .max()
        .unwrap_or(0);
    write_all(out, "\nSummary:\n");
    for report in reports {
        let (mark, detail) = match &report.outcome {
            Outcome::Passed => ("ok     ", String::new()),
            Outcome::Failed {
                exit_code: Some(code),
            } => ("FAILED ", format!("  (exit code {code})")),
            Outcome::Failed { exit_code: None } => ("FAILED ", "  (killed by a signal)".to_owned()),
            Outcome::CouldNotStart { .. } => ("FAILED ", "  (could not start)".to_owned()),
            Outcome::Skipped => ("skipped", "  (an earlier step failed)".to_owned()),
        };
        let name = &report.name;
        let seconds = report.elapsed.as_secs_f64();
        write_all(
            out,
            &format!("  {mark} {name:width$}  {seconds:6.1}s{detail}\n"),
        );
    }
}

fn table(out: &mut impl Write, rows: &[[String; 2]]) {
    let width = rows.iter().map(|[name, _]| name.len()).max().unwrap_or(0);
    for [name, summary] in rows {
        write_all(out, &format!("  {name:width$}  {summary}\n"));
    }
}

fn write_all(out: &mut impl Write, text: &str) {
    // Ignored because a closed terminal or pipe must not turn into a failed check.
    let _ = out.write_all(text.as_bytes());
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{summary, usage, usage_error};
    use crate::cli::UsageError;
    use crate::steps::{Outcome, Report};

    #[test]
    fn usage_lists_every_command() {
        let mut out = Vec::new();
        usage(&mut out);
        let text = String::from_utf8_lossy(&out);
        for (name, summary) in crate::cli::COMMANDS {
            assert!(text.contains(name) && text.contains(summary), "{text}");
        }
    }

    #[test]
    fn usage_error_names_the_problem_first() {
        let mut out = Vec::new();
        usage_error(&mut out, &UsageError::UnknownCommand("x".to_owned()));
        assert!(String::from_utf8_lossy(&out).starts_with("error: unknown command `x`\n\nUsage:"));
    }

    #[test]
    fn summary_marks_each_outcome() {
        let report = |name: &str, outcome| Report {
            name: name.to_owned(),
            outcome,
            elapsed: Duration::from_millis(1500),
        };
        let mut out = Vec::new();
        summary(
            &mut out,
            &[
                report("fmt", Outcome::Passed),
                report(
                    "clippy",
                    Outcome::Failed {
                        exit_code: Some(101),
                    },
                ),
                report("test", Outcome::Skipped),
            ],
        );
        let text = String::from_utf8_lossy(&out);
        assert!(text.contains("  ok      fmt        1.5s\n"), "{text}");
        assert!(
            text.contains("  FAILED  clippy     1.5s  (exit code 101)\n"),
            "{text}"
        );
        assert!(
            text.contains("  skipped test       1.5s  (an earlier step failed)\n"),
            "{text}"
        );
    }
}
