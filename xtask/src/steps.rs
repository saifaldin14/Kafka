//! Runs external commands as named steps and reports how each one ended.
//!
//! A step prints a `==> [n/N] <step>` header, then its command's output goes straight
//! to the terminal. After the last step, a summary lists every step with its outcome and
//! duration. By default the runner stops at the first failed step; with `keep_going` it
//! runs the rest.

use std::ffi::OsString;
use std::io::Write;
use std::process::Command;
use std::time::Duration;

use crate::clock::Stopwatch;
use crate::output;
use crate::tools::{self, Tool};

/// One external command to run as a named step.
#[derive(Debug, Clone)]
pub(crate) struct Step {
    pub(crate) name: String,
    pub(crate) program: OsString,
    pub(crate) args: Vec<String>,
    /// Environment variables set for the command, on top of the inherited ones.
    pub(crate) env: &'static [(&'static str, &'static str)],
    /// Tools that must be installed at their pinned versions before the step can run.
    pub(crate) requires: &'static [&'static Tool],
}

impl Step {
    /// A step that runs `cargo` (the one that started `xtask`) with `args`. Every step
    /// runs in the repository root.
    pub(crate) fn cargo(name: &str, args: &[&str]) -> Self {
        Self {
            name: name.to_owned(),
            program: tools::cargo_program(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            env: &[],
            requires: &[],
        }
    }

    /// The command line as the user would type it, for the step header.
    pub(crate) fn command_line(&self) -> String {
        let env = self
            .env
            .iter()
            .map(|(name, value)| format!("{name}=\"{value}\" "));
        let program = if self.program == tools::cargo_program() {
            "cargo".to_owned()
        } else {
            self.program.to_string_lossy().into_owned()
        };
        let command = std::iter::once(program)
            .chain(self.args.iter().cloned())
            .collect::<Vec<_>>()
            .join(" ");
        env.chain(std::iter::once(command)).collect()
    }
}

/// How a step ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Outcome {
    Passed,
    /// The command ran and failed. `None` means it was killed by a signal.
    Failed {
        exit_code: Option<i32>,
    },
    /// The command could not be started: the OS refused, or a required tool is
    /// missing or at the wrong version.
    CouldNotStart {
        reason: String,
    },
    /// Not run because an earlier step failed.
    Skipped,
}

impl Outcome {
    pub(crate) fn is_failure(&self) -> bool {
        matches!(self, Outcome::Failed { .. } | Outcome::CouldNotStart { .. })
    }
}

/// The result of one step.
#[derive(Debug, Clone)]
pub(crate) struct Report {
    pub(crate) name: String,
    pub(crate) outcome: Outcome,
    pub(crate) elapsed: Duration,
}

/// Runs `steps` in order, then writes the summary. Returns one report per step.
pub(crate) fn run(steps: &[Step], keep_going: bool, out: &mut impl Write) -> Vec<Report> {
    let mut reports = Vec::with_capacity(steps.len());
    let mut stopped = false;
    for (index, step) in steps.iter().enumerate() {
        if stopped {
            reports.push(Report {
                name: step.name.clone(),
                outcome: Outcome::Skipped,
                elapsed: Duration::ZERO,
            });
            continue;
        }
        output::step_header(
            out,
            index + 1,
            steps.len(),
            &step.name,
            &step.command_line(),
        );
        let stopwatch = Stopwatch::start();
        let outcome = run_one(step);
        let report = Report {
            name: step.name.clone(),
            outcome,
            elapsed: stopwatch.elapsed(),
        };
        if let Outcome::CouldNotStart { reason } = &report.outcome {
            output::line(out, &format!("error: {reason}"));
        }
        stopped = report.outcome.is_failure() && !keep_going;
        reports.push(report);
    }
    output::summary(out, &reports);
    reports
}

/// True when no step failed or could not start.
pub(crate) fn all_passed(reports: &[Report]) -> bool {
    !reports.iter().any(|report| report.outcome.is_failure())
}

fn run_one(step: &Step) -> Outcome {
    for tool in step.requires {
        if let Err(problem) = tools::check(tool) {
            return Outcome::CouldNotStart {
                reason: problem.to_string(),
            };
        }
    }
    let status = Command::new(&step.program)
        .current_dir(crate::repo_root())
        .args(&step.args)
        .envs(step.env.iter().copied())
        .status();
    match status {
        Ok(status) if status.success() => Outcome::Passed,
        Ok(status) => Outcome::Failed {
            exit_code: status.code(),
        },
        Err(error) => Outcome::CouldNotStart {
            reason: format!("could not run `{}`: {error}", step.command_line()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{Outcome, Step, all_passed, run};
    use crate::tools::{Invocation, Source, Tool, VersionOutput};

    fn missing_program() -> Step {
        Step {
            name: "missing".to_owned(),
            program: "alluvium-xtask-test-no-such-program".into(),
            args: Vec::new(),
            env: &[],
            requires: &[],
        }
    }

    fn outcomes(steps: &[Step], keep_going: bool) -> (Vec<Outcome>, String) {
        let mut out = Vec::new();
        let reports = run(steps, keep_going, &mut out);
        (
            reports.into_iter().map(|report| report.outcome).collect(),
            String::from_utf8_lossy(&out).into_owned(),
        )
    }

    #[test]
    fn a_command_that_succeeds_passes() {
        let (outcomes, text) = outcomes(&[Step::cargo("version", &["--version"])], false);
        assert_eq!(outcomes, [Outcome::Passed]);
        assert!(
            text.contains("==> [1/1] version (cargo --version)"),
            "{text}"
        );
        assert!(text.contains("ok      version"), "{text}");
    }

    #[test]
    fn a_failing_command_reports_its_exit_code() {
        let (outcomes, _) = outcomes(&[Step::cargo("bad", &["--no-such-flag"])], false);
        assert!(
            matches!(outcomes[..], [Outcome::Failed { exit_code: Some(code) }] if code != 0),
            "{outcomes:?}"
        );
    }

    #[test]
    fn a_program_that_cannot_start_reports_the_os_error() {
        let (outcomes, text) = outcomes(&[missing_program()], false);
        assert!(
            matches!(&outcomes[..], [Outcome::CouldNotStart { reason }] if reason.contains("could not run")),
            "{outcomes:?}"
        );
        assert!(text.contains("FAILED  missing"), "{text}");
    }

    #[test]
    fn the_runner_stops_at_the_first_failure_by_default() {
        let steps = [missing_program(), Step::cargo("version", &["--version"])];
        let (outcomes, text) = outcomes(&steps, false);
        assert!(matches!(outcomes[1], Outcome::Skipped), "{outcomes:?}");
        assert!(text.contains("skipped version"), "{text}");
    }

    #[test]
    fn keep_going_runs_the_remaining_steps() {
        let steps = [missing_program(), Step::cargo("version", &["--version"])];
        let (outcomes, _) = outcomes(&steps, true);
        assert_eq!(outcomes[1], Outcome::Passed);
    }

    #[test]
    fn a_missing_required_tool_blocks_the_step_with_the_install_command() {
        const TOOL: Tool = Tool {
            name: "cargo-alluvium-test-absent",
            version: "1.2.3",
            invocation: Invocation::CargoSubcommand("alluvium-test-absent"),
            source: Source::Crate,
            version_output: VersionOutput::NameThenVersion,
        };
        static REQUIRES: [&Tool; 1] = [&TOOL];
        let mut step = Step::cargo("needs-tool", &["--version"]);
        step.requires = &REQUIRES;
        let (outcomes, text) = outcomes(&[step], false);
        assert!(
            matches!(&outcomes[..], [Outcome::CouldNotStart { .. }]),
            "{outcomes:?}"
        );
        assert!(
            text.contains("cargo install --locked --version 1.2.3 cargo-alluvium-test-absent"),
            "{text}"
        );
    }

    #[test]
    fn step_environment_reaches_the_command_and_the_header() {
        let mut step = Step::cargo("env", &["locate-project"]);
        let (without_env, _) = outcomes(std::slice::from_ref(&step), false);
        assert_eq!(without_env, [Outcome::Passed]);
        step.env = &[("CARGO_TERM_COLOR", "bogus")];
        let (with_env, text) = outcomes(&[step], false);
        assert!(
            matches!(with_env[..], [Outcome::Failed { .. }]),
            "cargo should reject the colour setting: {with_env:?}"
        );
        assert!(
            text.contains("(CARGO_TERM_COLOR=\"bogus\" cargo locate-project)"),
            "{text}"
        );
    }

    #[test]
    fn all_passed_is_false_if_any_step_failed() {
        let mut out = Vec::new();
        let reports = run(&[missing_program()], false, &mut out);
        assert!(!all_passed(&reports));
        let reports = run(&[Step::cargo("version", &["--version"])], false, &mut out);
        assert!(all_passed(&reports));
    }
}
