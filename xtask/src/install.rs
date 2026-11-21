//! `cargo xtask tools install [--step <name>]...`: installs the pinned external tools
//! that are missing or at the wrong version, using the step runner.
//!
//! With `--step`, only the tools those CI steps require are installed, so a CI job
//! installs what its steps need and nothing more. The step table in `ci` is the single
//! record of which step needs which tool.

use std::io::Write;

use crate::Exit;
use crate::ci;
use crate::output;
use crate::steps::{self, Step};
use crate::tools::{self, TOOLS, Tool};

pub(crate) fn run(step_names: &[String], out: &mut impl Write) -> Exit {
    let wanted = wanted_tools(step_names);
    if wanted.is_empty() {
        output::line(out, "The selected steps need no external tools.");
        return Exit::Success;
    }
    let installs: Vec<Step> = wanted
        .into_iter()
        .filter(|tool| tools::check(tool).is_err())
        .map(|tool| {
            Step::cargo(
                &format!("install {}", tool.crate_name),
                &[
                    "install",
                    "--locked",
                    "--version",
                    tool.version,
                    tool.crate_name,
                ],
            )
        })
        .collect();
    if installs.is_empty() {
        output::line(out, "All tools are installed at their pinned versions.");
        return Exit::Success;
    }
    if steps::all_passed(&steps::run(&installs, false, out)) {
        Exit::Success
    } else {
        Exit::CheckFailed
    }
}

/// Every tool when no step is named; otherwise the tools the named steps require, in
/// `TOOLS` order and without duplicates.
fn wanted_tools(step_names: &[String]) -> Vec<&'static Tool> {
    if step_names.is_empty() {
        return TOOLS.iter().collect();
    }
    let required: Vec<&Tool> = ci::STEPS
        .iter()
        .filter(|step| step_names.iter().any(|name| name == step.name))
        .flat_map(|step| step.requires.iter().copied())
        .collect();
    TOOLS
        .iter()
        .filter(|tool| {
            required
                .iter()
                .any(|needed| needed.crate_name == tool.crate_name)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::wanted_tools;

    fn names(steps: &[&str]) -> Vec<&'static str> {
        let steps: Vec<String> = steps.iter().map(|step| (*step).to_owned()).collect();
        wanted_tools(&steps)
            .into_iter()
            .map(|tool| tool.crate_name)
            .collect()
    }

    #[test]
    fn no_steps_means_every_tool() {
        assert_eq!(names(&[]), ["cargo-nextest", "cargo-deny", "lychee"]);
    }

    #[test]
    fn named_steps_get_only_their_tools_once() {
        assert_eq!(names(&["test"]), ["cargo-nextest", "cargo-deny"]);
        assert_eq!(
            names(&["links", "deny", "advisories"]),
            ["cargo-deny", "lychee"]
        );
        assert_eq!(names(&["fmt", "clippy"]), Vec::<&str>::new());
    }
}
