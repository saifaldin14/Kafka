//! `xtask`: the repository's automation entry point, run as `cargo xtask <command>`.
//!
//! Layer: dev ([ADR-0009](../../docs/adr/0009-crate-layering-and-dependency-rules.md)).
//! It may depend on anything, and nothing except tests depends on it.
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
mod backlog_next;
mod check_layers;
mod check_manifests;
mod ci;
mod cli;
mod clock;
mod graph;
mod install;
mod layers;
mod lint_canary;
mod output;
mod steps;
mod tools;

use std::process::ExitCode;

use cli::{BacklogCommand, Command};

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
        Command::ToolsInstall => install::run(out),
        Command::CheckLayers(options) => check_layers::run(options, out),
        Command::CheckManifests => check_manifests::run(out),
        Command::Backlog(BacklogCommand::Next { all, count }) => {
            backlog_next::run_next(*all, *count, out)
        }
        Command::Backlog(BacklogCommand::Show { id }) => backlog_next::run_show(id, out),
        Command::LintCanary => lint_canary::run(out),
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
