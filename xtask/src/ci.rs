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
use crate::tools::{self, Invocation, Tool};

/// What a step runs.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Program {
    /// The `cargo` that started `xtask`.
    Cargo,
    /// A pinned tool from `tools::TOOLS`, run the way its `Invocation` says.
    Tool(&'static Tool),
}

/// One CI check.
#[derive(Debug)]
pub(crate) struct CiStep {
    pub(crate) name: &'static str,
    pub(crate) summary: &'static str,
    pub(crate) program: Program,
    pub(crate) args: &'static [&'static str],
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
        program: Program::Cargo,
        args: &["fmt", "--all", "--check"],
        env: &[],
        requires: &[],
    },
    CiStep {
        name: "clippy",
        summary: "Lints, with warnings as errors",
        program: Program::Cargo,
        args: &[
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
        program: Program::Cargo,
        args: &[
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
        program: Program::Cargo,
        args: &[
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
        name: "lint-canary",
        summary: "The workspace lints fire on their fixtures (cargo xtask lint-canary)",
        program: Program::Cargo,
        args: &[
            "run",
            "--quiet",
            "--locked",
            "--package",
            "xtask",
            "--",
            "lint-canary",
        ],
        env: &[],
        requires: &[],
    },
    CiStep {
        name: "deny",
        summary: "Dependency policy: advisories, bans, licenses, sources (deny.toml)",
        program: Program::Cargo,
        args: &["deny", "--locked", "check"],
        env: &[],
        requires: &[&tools::DENY],
    },
    CiStep {
        name: "links",
        summary: "Relative links and #anchors in the docs, offline (lychee)",
        program: Program::Tool(&tools::LYCHEE),
        args: &[
            "--offline",
            "--include-fragments",
            "--no-progress",
            "README.md",
            "AGENTS.md",
            "CLAUDE.md",
            "CONTRIBUTING.md",
            "docs",
            ".agents",
            ".github",
        ],
        env: &[],
        requires: &[&tools::LYCHEE],
    },
    CiStep {
        name: "test",
        summary: "Tests (cargo-nextest, ci profile)",
        program: Program::Cargo,
        args: &[
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
        .map(|ci_step| {
            let (program, prefix) = match ci_step.program {
                Program::Cargo => (tools::cargo_program(), None),
                Program::Tool(tool) => match tool.invocation {
                    Invocation::CargoSubcommand(subcommand) => {
                        (tools::cargo_program(), Some(subcommand))
                    }
                    Invocation::Binary(binary) => (binary.into(), None),
                },
            };
            Step {
                name: ci_step.name.to_owned(),
                program,
                args: prefix
                    .into_iter()
                    .chain(ci_step.args.iter().copied())
                    .map(str::to_owned)
                    .collect(),
                env: ci_step.env,
                requires: ci_step.requires,
            }
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
        for name in [
            "fmt",
            "clippy",
            "doc",
            "layers",
            "lint-canary",
            "deny",
            "links",
            "test",
        ] {
            assert!(is_step(name), "{name}");
        }
        assert!(!is_step("nope"));
    }
}
