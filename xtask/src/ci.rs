//! `cargo xtask ci`: every check CI runs, in one table, runnable locally.
//!
//! [`STEPS`] is the single list of CI steps, in the order they run. CI jobs select
//! steps with `--step`, so a check is never defined twice.
//!
//! # Adding a step
//!
//! Add a [`CiStep`] to [`STEPS`] at the position it should run, with the pinned tool
//! it needs (if any) from `tools::TOOLS`, and update the tooling table in
//! docs/standards/rust.md.

use std::io::Write;

use crate::Exit;
use crate::cli::CiOptions;
use crate::output;
use crate::steps::{self, Step};
use crate::tools::{self, Tool};

/// One CI check.
#[derive(Debug)]
pub(crate) struct CiStep {
    pub(crate) name: &'static str,
    pub(crate) summary: &'static str,
    /// Arguments to `cargo`.
    pub(crate) cargo_args: &'static [&'static str],
    /// Environment variables for the command.
    pub(crate) env: &'static [(&'static str, &'static str)],
    /// Pinned tools the step needs, checked before it runs.
    pub(crate) requires: &'static [&'static Tool],
}

/// Every CI step, in run order.
pub(crate) const STEPS: &[CiStep] = &[
    CiStep {
        name: "fmt",
        summary: "Formatting (rustfmt)",
        cargo_args: &["fmt", "--all", "--check"],
        env: &[],
        requires: &[],
    },
    CiStep {
        name: "clippy",
        summary: "Lints, with warnings as errors",
        cargo_args: &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
        env: &[],
        requires: &[],
    },
    CiStep {
        name: "doc",
        summary: "API docs build, private items included, warnings as errors",
        cargo_args: &[
            "doc",
            "--workspace",
            "--no-deps",
            "--document-private-items",
            "--locked",
        ],
        env: &[("RUSTDOCFLAGS", "-D warnings")],
        requires: &[],
    },
    CiStep {
        name: "layers",
        summary: "Crate layering rules (cargo xtask check-layers)",
        cargo_args: &[
            "run",
            "--quiet",
            "--locked",
            "--package",
            "xtask",
            "--",
            "check-layers",
        ],
        env: &[],
        requires: &[],
    },
    CiStep {
        name: "deny",
        summary: "Dependency policy: advisories, bans, licenses, sources (deny.toml)",
        cargo_args: &["deny", "--locked", "check"],
        env: &[],
        requires: &[&tools::DENY],
    },
    CiStep {
        name: "test",
        summary: "Tests (cargo-nextest, ci profile)",
        cargo_args: &[
            "nextest",
            "run",
            "--workspace",
            "--locked",
            "--profile",
            "ci",
        ],
        env: &[],
        // The deny.toml fixture tests in xtask run cargo-deny.
        requires: &[&tools::NEXTEST, &tools::DENY],
    },
];

/// True if `name` is a registered step.
pub(crate) fn is_step(name: &str) -> bool {
    STEPS.iter().any(|step| step.name == name)
}

/// Runs the selected steps, or lists them with `--list`.
pub(crate) fn run(options: &CiOptions, out: &mut impl Write) -> Exit {
    if options.list {
        output::step_list(out, STEPS);
        return Exit::Success;
    }
    let steps: Vec<Step> = selected(&options.steps)
        .map(|ci_step| Step {
            env: ci_step.env,
            requires: ci_step.requires,
            ..Step::cargo(ci_step.name, ci_step.cargo_args)
        })
        .collect();
    let reports = steps::run(&steps, options.keep_going, out);
    if steps::all_passed(&reports) {
        Exit::Success
    } else {
        Exit::CheckFailed
    }
}

/// The steps to run, in registered order: all of them, or only those named.
fn selected(names: &[String]) -> impl Iterator<Item = &'static CiStep> + '_ {
    STEPS
        .iter()
        .filter(move |step| names.is_empty() || names.iter().any(|name| name == step.name))
}

#[cfg(test)]
mod tests {
    use super::{STEPS, is_step, selected};

    #[test]
    fn step_names_are_unique() {
        for (index, step) in STEPS.iter().enumerate() {
            assert!(
                STEPS[index + 1..]
                    .iter()
                    .all(|other| other.name != step.name),
                "duplicate step {}",
                step.name
            );
        }
    }

    #[test]
    fn selection_keeps_registered_order_whatever_the_flag_order() {
        let names = ["test".to_owned(), "fmt".to_owned()];
        let picked: Vec<_> = selected(&names).map(|step| step.name).collect();
        assert_eq!(picked, ["fmt", "test"]);
    }

    #[test]
    fn no_selection_means_every_step() {
        assert_eq!(selected(&[]).count(), STEPS.len());
    }

    #[test]
    fn documented_steps_are_registered() {
        for name in ["fmt", "clippy", "doc", "layers", "deny", "test"] {
            assert!(is_step(name), "{name}");
        }
        assert!(!is_step("nope"));
    }
}
