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
    /// Runs only when named with `--step`, never in the full `cargo xtask ci` run: for
    /// checks of things outside the repository, which can fail without a change here.
    pub(crate) on_demand: bool,
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
        on_demand: false,
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
        on_demand: false,
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
        on_demand: false,
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
        on_demand: false,
    },
    CiStep {
        name: "manifests",
        summary: "[workspace.lints], clippy.toml and member manifests match rust.md (cargo xtask check-manifests)",
        program: Program::Cargo,
        args: &[
            "run",
            "--quiet",
            "--locked",
            "--package",
            "xtask",
            "--",
            "check-manifests",
        ],
        env: &[],
        requires: &[],
        on_demand: false,
    },
    CiStep {
        name: "schemas",
        summary: "The vendored Kafka schemas match their MANIFEST.sha256 (cargo xtask schemas verify)",
        program: Program::Cargo,
        args: &[
            "run",
            "--quiet",
            "--locked",
            "--package",
            "xtask",
            "--",
            "schemas",
            "verify",
        ],
        env: &[],
        requires: &[],
        on_demand: false,
    },
    CiStep {
        name: "crate-docs",
        summary: "Every workspace crate has a deep dive in docs/crates/ (cargo xtask docs check-crates)",
        program: Program::Cargo,
        args: &[
            "run",
            "--quiet",
            "--locked",
            "--package",
            "xtask",
            "--",
            "docs",
            "check-crates",
        ],
        env: &[],
        requires: &[],
        on_demand: false,
    },
    CiStep {
        name: "workflows",
        summary: "GitHub Actions files pin actions by SHA, set timeouts and permissions, and pass actionlint and shellcheck (cargo xtask check-workflows)",
        program: Program::Cargo,
        args: &[
            "run",
            "--quiet",
            "--locked",
            "--package",
            "xtask",
            "--",
            "check-workflows",
        ],
        env: &[],
        requires: &[&tools::ACTIONLINT, &tools::SHELLCHECK],
        on_demand: false,
    },
    CiStep {
        name: "backlog",
        summary: "The backlog follows docs/backlog/README.md (cargo xtask backlog check)",
        program: Program::Cargo,
        args: &[
            "run",
            "--quiet",
            "--locked",
            "--package",
            "xtask",
            "--",
            "backlog",
            "check",
        ],
        env: &[],
        requires: &[],
        on_demand: false,
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
        on_demand: false,
    },
    CiStep {
        name: "deny",
        summary: "Dependency policy: bans, licenses, sources (deny.toml)",
        program: Program::Cargo,
        args: &["deny", "--locked", "check", "bans", "licenses", "sources"],
        env: &[],
        requires: &[&tools::DENY],
        on_demand: false,
    },
    // The fuzz targets are a workspace of their own, with their own lockfile, which
    // `deny` doesn't see. Its advisories don't matter: nothing it builds ships.
    CiStep {
        name: "fuzz-deny",
        summary: "Dependency policy for the fuzz workspace: bans, licenses, sources",
        program: Program::Cargo,
        args: &[
            "deny",
            "--manifest-path",
            "fuzz/Cargo.toml",
            "--locked",
            "check",
            "bans",
            "licenses",
            "sources",
        ],
        env: &[],
        requires: &[&tools::DENY],
        on_demand: false,
    },
    // Separate from `deny`: a newly published advisory depends on the date, not on the
    // change being tested, so CI reports it without blocking unrelated PRs.
    CiStep {
        name: "advisories",
        summary: "Security advisories against Cargo.lock (cargo deny check advisories)",
        program: Program::Cargo,
        args: &["deny", "--locked", "check", "advisories"],
        env: &[],
        requires: &[&tools::DENY],
        on_demand: false,
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
        on_demand: false,
    },
    CiStep {
        name: "links-external",
        summary: "External URLs in the same files, online, with retries (lychee); on demand, run weekly by .github/workflows/links.yml",
        program: Program::Tool(&tools::LYCHEE),
        args: &[
            "--no-progress",
            "--max-retries",
            "3",
            "--retry-wait-time",
            "10",
            // Some sites (eprint.iacr.org) refuse lychee's default user agent with 403.
            "--user-agent",
            "Mozilla/5.0 (compatible; alluvium-link-check)",
            "--format",
            "json",
            "--output",
            "target/links-external.json",
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
        on_demand: true,
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
        on_demand: false,
    },
    CiStep {
        name: "doctest",
        summary: "Doc tests, including compile_fail examples (nextest does not run them)",
        program: Program::Cargo,
        args: &["test", "--workspace", "--doc", "--locked"],
        env: &[],
        requires: &[],
        on_demand: false,
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
    STEPS.iter().filter(move |step| {
        if names.is_empty() {
            !step.on_demand
        } else {
            names.iter().any(|name| name == step.name)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{STEPS, is_step, selected};

    /// The inputs of a lychee step: its arguments after the last option value.
    fn link_inputs(name: &str) -> Vec<&'static str> {
        let step = STEPS.iter().find(|step| step.name == name).unwrap();
        let first_input = step
            .args
            .iter()
            .position(|arg| *arg == "README.md")
            .unwrap();
        step.args[first_input..].to_vec()
    }

    #[test]
    fn the_external_link_check_covers_the_same_files_as_the_offline_one() {
        assert_eq!(link_inputs("links-external"), link_inputs("links"));
    }

    #[test]
    fn on_demand_steps_run_only_when_named() {
        let all: Vec<&str> = selected(&[]).map(|step| step.name).collect();
        assert!(!all.contains(&"links-external"), "{all:?}");
        assert!(all.contains(&"links"));
        let named: Vec<&str> = selected(&["links-external".to_owned()])
            .map(|step| step.name)
            .collect();
        assert_eq!(named, ["links-external"]);
    }

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
    fn no_selection_means_every_step_that_is_not_on_demand() {
        assert_eq!(
            selected(&[]).count(),
            STEPS.iter().filter(|step| !step.on_demand).count()
        );
    }

    #[test]
    fn documented_steps_are_registered() {
        for name in [
            "fmt",
            "clippy",
            "doc",
            "layers",
            "manifests",
            "schemas",
            "crate-docs",
            "workflows",
            "backlog",
            "lint-canary",
            "deny",
            "advisories",
            "links",
            "links-external",
            "test",
            "doctest",
        ] {
            assert!(is_step(name), "{name}");
        }
        assert!(!is_step("nope"));
    }
}
