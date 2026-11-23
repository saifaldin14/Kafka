//! `xtask`: the repository's automation entry point, run as `cargo xtask <command>`.
//!
//! Layer: dev ([ADR-0009](../../docs/adr/0009-crate-layering-and-dependency-rules.md)).
//! It may depend on anything, and nothing except tests depends on it.
//!
//! Kafka counterpart: Kafka's Gradle build tasks (`build.gradle` at 4.4.0).
//!
//! Every repository check that CI runs is a command here, so CI and a developer's
//! machine run exactly the same thing. `cargo xtask help` lists the commands.
//!
//! # Exit codes
//!
//! | Code | Meaning |
//! |---|---|
//! | 0 | Success |
//! | 1 | A check failed |
//! | 2 | Usage error: unknown command or bad arguments |
//!
//! # Adding a command
//!
//! 1. Add a variant to `cli::Command`, its parsing in `cli::parse` and its line in
//!    `cli::COMMANDS`, with parser tests.
//! 2. Implement it in its own module. It returns an `Exit` and writes only through
//!    the `output` module.
//! 3. Dispatch it in `run` below.
//!
//! CI checks are not commands: they are steps of `cargo xtask ci`, registered in the
//! `ci` module.
//!
//! Commands arrive with the backlog task that needs them; nothing is added ahead of use.

mod backlog;
mod backlog_check;
mod backlog_next;
mod backlog_stats;
mod bench;
mod check_layers;
mod check_manifests;
mod check_workflows;
mod ci;
mod cli;
mod clock;
mod codegen;
mod crate_docs;
mod deps_diff;
mod doctor;
mod fuzz;
mod golden;
mod graph;
mod hooks;
mod install;
mod layers;
mod lint_canary;
mod message_spec;
mod output;
mod schemas;
mod steps;
mod tools;

use std::process::ExitCode;

use cli::{BacklogCommand, Command, DocsCommand, SchemasCommand};

/// How an `xtask` invocation ended. Converted to the process exit code in `main`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Exit {
    Success,
    CheckFailed,
    Usage,
}

impl Exit {
    fn code(self) -> u8 {
        match self {
            Exit::Success => 0,
            Exit::CheckFailed => 1,
            Exit::Usage => 2,
        }
    }
}

/// The repository root (the parent of the `xtask` crate). Steps and checks run here.
pub(crate) fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn main() -> ExitCode {
    let exit = match cli::parse(std::env::args_os().skip(1)) {
        Ok(command) => run(&command, &mut std::io::stdout().lock()),
        Err(error) => {
            output::usage_error(&mut std::io::stderr().lock(), &error);
            Exit::Usage
        }
    };
    ExitCode::from(exit.code())
}

fn run(command: &Command, out: &mut impl std::io::Write) -> Exit {
    match command {
        Command::Help => {
            output::usage(out);
            Exit::Success
        }
        Command::Ci(options) => ci::run(options, out),
        Command::ToolsInstall { steps, tools } => install::run(steps, tools, out),
        Command::CheckLayers(options) => check_layers::run(options, out),
        Command::CheckManifests => check_manifests::run(out),
        Command::CheckWorkflows => check_workflows::run(out),
        Command::Backlog(BacklogCommand::Next { all, count }) => {
            backlog_next::run_next(*all, *count, out)
        }
        Command::Backlog(BacklogCommand::Show { id }) => backlog_next::run_show(id, out),
        Command::Backlog(BacklogCommand::Stats { write }) => backlog_stats::run(*write, out),
        Command::Backlog(BacklogCommand::Check(options)) => backlog_check::run(options, out),
        Command::Docs(DocsCommand::CheckCrates) => crate_docs::run_check(out),
        Command::Docs(DocsCommand::NewCrateDoc { name }) => crate_docs::run_new(name, out),
        Command::DepsDiff { base_ref } => deps_diff::run(base_ref, out),
        Command::Schemas(SchemasCommand::Verify) => schemas::run_verify(out),
        Command::Schemas(SchemasCommand::Vendor { tag, dry_run }) => {
            schemas::run_vendor(tag, *dry_run, out)
        }
        Command::LintCanary => lint_canary::run(out),
        Command::Doctor => doctor::run(out),
        Command::InstallHooks { uninstall } => hooks::run(*uninstall, out),
        Command::Fuzz(command) => fuzz::run(command, out),
        Command::Codegen { check } => codegen::run(*check, out),
        Command::Bench { base } => bench::run(base.as_deref(), out),
        Command::Golden(dump) => golden::run(*dump, out),
    }
}

#[cfg(test)]
mod tests {
    use super::{Exit, run};
    use crate::cli::Command;

    #[test]
    fn exit_codes_match_the_documented_contract() {
        assert_eq!(Exit::Success.code(), 0);
        assert_eq!(Exit::CheckFailed.code(), 1);
        assert_eq!(Exit::Usage.code(), 2);
    }

    #[test]
    fn help_prints_usage_and_succeeds() {
        let mut out = Vec::new();
        assert_eq!(run(&Command::Help, &mut out), Exit::Success);
        let text = String::from_utf8_lossy(&out);
        assert!(text.starts_with("Usage: cargo xtask <command>"), "{text}");
        assert!(text.contains("  help "), "{text}");
    }
}
