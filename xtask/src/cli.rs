//! Command-line parsing for `xtask`.
//!
//! Std only: the grammar is a command name plus a few flags, and an argument-parsing
//! crate would add a dependency for no gain (docs/standards/security.md#dependencies).

use std::ffi::OsString;
use std::fmt;

use crate::backlog_check::CheckOptions;
use crate::check_layers::LayerOptions;
use crate::ci;

/// A parsed `xtask` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
    Help,
    Ci(CiOptions),
    /// `tools install [--step <name>]... [--tool <name>]...`: the tools those CI steps
    /// need and the tools named, or every tool a step needs.
    ToolsInstall {
        steps: Vec<String>,
        tools: Vec<String>,
    },
    CheckLayers(LayerOptions),
    CheckManifests,
    CheckWorkflows,
    LintCanary,
    Backlog(BacklogCommand),
    Docs(DocsCommand),
    Schemas(SchemasCommand),
    /// `deps-diff <base-ref>`.
    DepsDiff {
        base_ref: String,
    },
    Doctor,
    /// `install-hooks [--uninstall]`.
    InstallHooks {
        uninstall: bool,
    },
    Fuzz(FuzzCommand),
    Codegen,
    /// `bench [--base <ref>]`.
    Bench {
        base: Option<String>,
    },
}

/// A `cargo xtask fuzz` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FuzzCommand {
    /// `--list`.
    List,
    /// `<target> [--time <duration>]`: fuzz for that long.
    Run { target: String, seconds: u64 },
    /// `<target> <input>`: run one input, such as a crash artifact.
    Reproduce { target: String, input: String },
    /// `corpus pull [<target>]...`: download the corpora CI grew, for every target if
    /// none is named (ADR-0018).
    CorpusPull { targets: Vec<String> },
    /// `corpus minimize <target>`: shrink a corpus without losing coverage.
    CorpusMinimize { target: String },
}

/// How long `cargo xtask fuzz <target>` runs without `--time`.
pub(crate) const DEFAULT_FUZZ_SECONDS: u64 = 60;

/// A `cargo xtask backlog` subcommand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BacklogCommand {
    /// `next [--all] [-n <count>]`.
    Next { all: bool, count: usize },
    /// `show <id>`.
    Show { id: String },
    /// `stats [--write]`.
    Stats { write: bool },
    /// `check [--allow-empty-features] [--epic <code>]...`.
    Check(CheckOptions),
}

/// A `cargo xtask docs` subcommand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DocsCommand {
    /// `check-crates`.
    CheckCrates,
    /// `new-crate-doc <crate>`.
    NewCrateDoc { name: String },
}

/// A `cargo xtask schemas` subcommand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SchemasCommand {
    /// `verify`.
    Verify,
    /// `vendor --tag <tag> [--dry-run]`.
    Vendor { tag: String, dry_run: bool },
}

/// Options for `cargo xtask ci`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CiOptions {
    /// Steps chosen with `--step`; empty means every step.
    pub(crate) steps: Vec<String>,
    pub(crate) list: bool,
    pub(crate) keep_going: bool,
}

/// Every command with its one-line summary, in the order `help` prints them.
pub(crate) const COMMANDS: &[(&str, &str)] = &[
    ("help", "Print this list of commands"),
    (
        "ci [--step <name>]... [--keep-going] [--list]",
        "Run the CI checks; `--list` shows the steps",
    ),
    (
        "tools install [--step <name>]... [--tool <name>]...",
        "Install the pinned tools the CI steps need, or only those the named steps need and the named tools",
    ),
    (
        "fuzz <target> [--time <duration>]",
        "Fuzz a target with the standard limits, 60s unless `--time` says otherwise (`90s`, `5m`, `1h`)",
    ),
    (
        "fuzz <target> <input>",
        "Run one input through a target with the standard limits, such as a crash artifact",
    ),
    ("fuzz --list", "List the fuzz targets"),
    (
        "codegen",
        "Generate the protocol crate's message types from the vendored Kafka schemas",
    ),
    (
        "bench [--base <ref>]",
        "Run the gated benchmarks (Linux, valgrind); with --base, fail on an instruction-count regression against <ref>",
    ),
    (
        "fuzz corpus pull [<target>]...",
        "Download the corpora the nightly fuzz run grew on main (needs a `gh` login; ADR-0018)",
    ),
    (
        "fuzz corpus minimize <target>",
        "Shrink a target's corpus with libFuzzer's merge, checking coverage and the size budget",
    ),
    (
        "check-layers [--print] [--manifest-path <path>]",
        "Check the crate layering rules (ADR-0009); `--print` shows the layer table",
    ),
    (
        "check-manifests",
        "Check [workspace.lints], clippy.toml and member manifests against rust.md, and deny.toml ignores",
    ),
    (
        "check-workflows",
        "Check that GitHub Actions files pin actions by SHA and set timeouts and permissions",
    ),
    (
        "docs check-crates",
        "Check that every workspace crate has a deep dive in docs/crates/, and list the stubs",
    ),
    (
        "docs new-crate-doc <crate>",
        "Write a stub deep dive for a workspace crate from docs/crates/template.md",
    ),
    (
        "deps-diff <base-ref>",
        "List the registry crates added, removed and upgraded in Cargo.lock since <base-ref>",
    ),
    (
        "doctor",
        "Check the local toolchain and tools, and print the fix for each problem",
    ),
    (
        "install-hooks [--uninstall]",
        "Install (or remove) an optional pre-push hook that runs the fmt and clippy steps",
    ),
    (
        "schemas verify",
        "Check that the vendored Kafka schemas match their MANIFEST.sha256",
    ),
    (
        "schemas vendor --tag <tag> [--dry-run]",
        "Copy the schemas from an apache/kafka tag and write the manifest (needs the network)",
    ),
    (
        "lint-canary",
        "Prove the workspace lints fire on the fixtures in xtask/lint-canary",
    ),
    (
        "backlog check [--allow-empty-features] [--epic <code>]...",
        "Check the backlog against the rules in docs/backlog/README.md",
    ),
    (
        "backlog next [--all] [-n <count>]",
        "List the ready backlog tasks in the order docs/backlog/README.md defines",
    ),
    (
        "backlog show <id>",
        "Show one backlog item, what it waits for and what needs it",
    ),
    (
        "backlog stats [--write]",
        "Print the backlog totals; `--write` updates them in docs/backlog/README.md",
    ),
];

/// Why the command line could not be parsed. Always exit code 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UsageError {
    MissingCommand,
    UnknownCommand(String),
    UnexpectedArgument {
        command: &'static str,
        argument: String,
    },
    MissingValue {
        flag: &'static str,
    },
    InvalidNumber {
        flag: &'static str,
        value: String,
    },
    MissingArgument {
        command: &'static str,
        name: &'static str,
    },
    UnknownStep(String),
    UnknownTool(String),
    MissingSubcommand {
        command: &'static str,
    },
    UnknownSubcommand {
        command: &'static str,
        name: String,
    },
    NonUtf8Argument(OsString),
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UsageError::MissingCommand => write!(f, "no command given"),
            UsageError::UnknownCommand(name) => write!(f, "unknown command `{name}`"),
            UsageError::UnexpectedArgument { command, argument } => {
                write!(f, "`{command}` does not take the argument `{argument}`")
            }
            UsageError::MissingValue { flag } => write!(f, "`{flag}` needs a value"),
            UsageError::InvalidNumber { flag, value } => {
                write!(f, "`{flag}` needs a number, not `{value}`")
            }
            UsageError::MissingArgument { command, name } => {
                write!(f, "`{command}` needs {name}")
            }
            UsageError::UnknownStep(name) => write!(
                f,
                "unknown step `{name}`; `cargo xtask ci --list` shows the steps"
            ),
            UsageError::UnknownTool(name) => write!(
                f,
                "unknown tool `{name}`; the tools are listed in xtask/src/tools.rs"
            ),
            UsageError::MissingSubcommand { command } => {
                write!(f, "`{command}` needs a subcommand")
            }
            UsageError::UnknownSubcommand { command, name } => {
                write!(f, "unknown subcommand `{command} {name}`")
            }
            UsageError::NonUtf8Argument(argument) => {
                write!(f, "argument `{}` is not valid UTF-8", argument.display())
            }
        }
    }
}

/// Parses the arguments after the program name.
pub(crate) fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Command, UsageError> {
    let args: Vec<String> = args
        .into_iter()
        .map(|arg| arg.into_string().map_err(UsageError::NonUtf8Argument))
        .collect::<Result<_, _>>()?;
    let (name, rest) = args.split_first().ok_or(UsageError::MissingCommand)?;
    match name.as_str() {
        "help" | "--help" | "-h" => no_arguments("help", rest).map(|()| Command::Help),
        "ci" => parse_ci(rest).map(Command::Ci),
        "check-layers" => parse_check_layers(rest).map(Command::CheckLayers),
        "check-manifests" => {
            no_arguments("check-manifests", rest).map(|()| Command::CheckManifests)
        }
        "check-workflows" => {
            no_arguments("check-workflows", rest).map(|()| Command::CheckWorkflows)
        }
        "backlog" => parse_backlog(rest).map(Command::Backlog),
        "docs" => parse_docs(rest).map(Command::Docs),
        "schemas" => parse_schemas(rest).map(Command::Schemas),
        "deps-diff" => match rest {
            [base_ref] => Ok(Command::DepsDiff {
                base_ref: base_ref.clone(),
            }),
            [] => Err(UsageError::MissingArgument {
                command: "deps-diff",
                name: "a base git ref, such as origin/main",
            }),
            [_, extra, ..] => Err(UsageError::UnexpectedArgument {
                command: "deps-diff",
                argument: extra.clone(),
            }),
        },
        "lint-canary" => no_arguments("lint-canary", rest).map(|()| Command::LintCanary),
        "doctor" => no_arguments("doctor", rest).map(|()| Command::Doctor),
        "install-hooks" => match rest {
            [] => Ok(Command::InstallHooks { uninstall: false }),
            [flag] if flag == "--uninstall" => Ok(Command::InstallHooks { uninstall: true }),
            [flag, extra, ..] if flag == "--uninstall" => Err(UsageError::UnexpectedArgument {
                command: "install-hooks",
                argument: extra.clone(),
            }),
            [argument, ..] => Err(UsageError::UnexpectedArgument {
                command: "install-hooks",
                argument: argument.clone(),
            }),
        },
        "tools" => match rest.split_first() {
            None => Err(UsageError::MissingSubcommand { command: "tools" }),
            Some((sub, rest)) if sub == "install" => parse_tools_install(rest)
                .map(|(steps, tools)| Command::ToolsInstall { steps, tools }),
            Some((sub, _)) => Err(UsageError::UnknownSubcommand {
                command: "tools",
                name: sub.clone(),
            }),
        },
        "fuzz" => parse_fuzz(rest).map(Command::Fuzz),
        "codegen" => no_arguments("codegen", rest).map(|()| Command::Codegen),
        "bench" => match rest {
            [] => Ok(Command::Bench { base: None }),
            [flag] if flag == "--base" => Err(UsageError::MissingValue { flag: "--base" }),
            [flag, base] if flag == "--base" => Ok(Command::Bench {
                base: Some(base.clone()),
            }),
            [flag, _, extra, ..] if flag == "--base" => Err(UsageError::UnexpectedArgument {
                command: "bench",
                argument: extra.clone(),
            }),
            [argument, ..] => Err(UsageError::UnexpectedArgument {
                command: "bench",
                argument: argument.clone(),
            }),
        },
        _ => Err(UsageError::UnknownCommand(name.clone())),
    }
}

fn no_arguments(command: &'static str, rest: &[String]) -> Result<(), UsageError> {
    match rest.first() {
        None => Ok(()),
        Some(argument) => Err(UsageError::UnexpectedArgument {
            command,
            argument: argument.clone(),
        }),
    }
}

fn parse_ci(rest: &[String]) -> Result<CiOptions, UsageError> {
    let mut options = CiOptions::default();
    let mut args = rest.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list" => options.list = true,
            "--keep-going" => options.keep_going = true,
            "--step" => {
                let name = args
                    .next()
                    .ok_or(UsageError::MissingValue { flag: "--step" })?;
                add_step(&mut options, name)?;
            }
            _ => match arg.strip_prefix("--step=") {
                Some(name) => add_step(&mut options, name)?,
                None => {
                    return Err(UsageError::UnexpectedArgument {
                        command: "ci",
                        argument: arg.clone(),
                    });
                }
            },
        }
    }
    Ok(options)
}

fn parse_check_layers(rest: &[String]) -> Result<LayerOptions, UsageError> {
    let mut options = LayerOptions::default();
    let mut args = rest.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--print" => options.print = true,
            "--manifest-path" => {
                let path = args.next().ok_or(UsageError::MissingValue {
                    flag: "--manifest-path",
                })?;
                options.manifest_path = Some(path.into());
            }
            _ => {
                return Err(UsageError::UnexpectedArgument {
                    command: "check-layers",
                    argument: arg.clone(),
                });
            }
        }
    }
    Ok(options)
}

fn parse_tools_install(rest: &[String]) -> Result<(Vec<String>, Vec<String>), UsageError> {
    let mut options = CiOptions::default();
    let mut tools = Vec::new();
    let mut args = rest.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--step" => {
                let name = args
                    .next()
                    .ok_or(UsageError::MissingValue { flag: "--step" })?;
                add_step(&mut options, name)?;
            }
            "--tool" => {
                let name = args
                    .next()
                    .ok_or(UsageError::MissingValue { flag: "--tool" })?;
                if !crate::tools::TOOLS.iter().any(|tool| tool.name == name) {
                    return Err(UsageError::UnknownTool(name.clone()));
                }
                if !tools.contains(name) {
                    tools.push(name.clone());
                }
            }
            _ => {
                return Err(UsageError::UnexpectedArgument {
                    command: "tools install",
                    argument: arg.clone(),
                });
            }
        }
    }
    Ok((options.steps, tools))
}

fn parse_fuzz(rest: &[String]) -> Result<FuzzCommand, UsageError> {
    let unexpected = |argument: &String| UsageError::UnexpectedArgument {
        command: "fuzz",
        argument: argument.clone(),
    };
    match rest {
        [] => Err(UsageError::MissingArgument {
            command: "fuzz",
            name: "a target, or `--list`",
        }),
        [flag] if flag == "--list" => Ok(FuzzCommand::List),
        [corpus, rest @ ..] if corpus == "corpus" => parse_fuzz_corpus(rest),
        [flag, extra, ..] if flag == "--list" => Err(unexpected(extra)),
        [target] => Ok(FuzzCommand::Run {
            target: target.clone(),
            seconds: DEFAULT_FUZZ_SECONDS,
        }),
        [target, flag] if flag == "--time" => Err(UsageError::MissingValue { flag: "--time" }),
        [target, flag, duration] if flag == "--time" => Ok(FuzzCommand::Run {
            target: target.clone(),
            seconds: parse_duration(duration).ok_or_else(|| UsageError::InvalidNumber {
                flag: "--time",
                value: duration.clone(),
            })?,
        }),
        [_, flag, _, extra, ..] if flag == "--time" => Err(unexpected(extra)),
        [_, input, ..] if input.starts_with('-') => Err(unexpected(input)),
        [target, input] => Ok(FuzzCommand::Reproduce {
            target: target.clone(),
            input: input.clone(),
        }),
        [_, _, extra, ..] => Err(unexpected(extra)),
    }
}

fn parse_fuzz_corpus(rest: &[String]) -> Result<FuzzCommand, UsageError> {
    match rest.split_first() {
        None => Err(UsageError::MissingSubcommand {
            command: "fuzz corpus",
        }),
        Some((sub, targets)) if sub == "pull" => {
            match targets.iter().find(|t| t.starts_with('-')) {
                Some(flag) => Err(UsageError::UnexpectedArgument {
                    command: "fuzz corpus pull",
                    argument: flag.clone(),
                }),
                None => Ok(FuzzCommand::CorpusPull {
                    targets: targets.to_vec(),
                }),
            }
        }
        Some((sub, rest)) if sub == "minimize" => match rest {
            [target] if !target.starts_with('-') => Ok(FuzzCommand::CorpusMinimize {
                target: target.clone(),
            }),
            [] => Err(UsageError::MissingArgument {
                command: "fuzz corpus minimize",
                name: "a target",
            }),
            [first] => Err(UsageError::UnexpectedArgument {
                command: "fuzz corpus minimize",
                argument: first.clone(),
            }),
            [_, extra, ..] => Err(UsageError::UnexpectedArgument {
                command: "fuzz corpus minimize",
                argument: extra.clone(),
            }),
        },
        Some((sub, _)) => Err(UsageError::UnknownSubcommand {
            command: "fuzz corpus",
            name: sub.clone(),
        }),
    }
}

/// Seconds from `90`, `90s`, `5m` or `1h`. Zero is rejected: libFuzzer reads it as
/// "no limit".
fn parse_duration(text: &str) -> Option<u64> {
    let (number, unit) = match text.char_indices().last()? {
        (index, 's') => (text.get(..index)?, 1),
        (index, 'm') => (text.get(..index)?, 60),
        (index, 'h') => (text.get(..index)?, 3600),
        _ => (text, 1),
    };
    number
        .parse::<u64>()
        .ok()?
        .checked_mul(unit)
        .filter(|&seconds| seconds > 0)
}

fn parse_backlog(rest: &[String]) -> Result<BacklogCommand, UsageError> {
    let Some((sub, rest)) = rest.split_first() else {
        return Err(UsageError::MissingSubcommand { command: "backlog" });
    };
    match sub.as_str() {
        "next" => {
            let mut all = false;
            let mut count = crate::backlog_next::DEFAULT_COUNT;
            let mut args = rest.iter();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--all" => all = true,
                    "-n" => {
                        let value = args.next().ok_or(UsageError::MissingValue { flag: "-n" })?;
                        count = value.parse().map_err(|_| UsageError::InvalidNumber {
                            flag: "-n",
                            value: value.clone(),
                        })?;
                    }
                    _ => {
                        return Err(UsageError::UnexpectedArgument {
                            command: "backlog next",
                            argument: arg.clone(),
                        });
                    }
                }
            }
            Ok(BacklogCommand::Next { all, count })
        }
        "show" => match rest {
            [id] => Ok(BacklogCommand::Show { id: id.clone() }),
            [] => Err(UsageError::MissingArgument {
                command: "backlog show",
                name: "an item id",
            }),
            [_, extra, ..] => Err(UsageError::UnexpectedArgument {
                command: "backlog show",
                argument: extra.clone(),
            }),
        },
        "check" => {
            let mut options = CheckOptions::default();
            let mut args = rest.iter();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--allow-empty-features" => options.allow_empty_features = true,
                    "--epic" => {
                        let code = args
                            .next()
                            .ok_or(UsageError::MissingValue { flag: "--epic" })?;
                        options.epics.push(code.clone());
                    }
                    _ => {
                        return Err(UsageError::UnexpectedArgument {
                            command: "backlog check",
                            argument: arg.clone(),
                        });
                    }
                }
            }
            Ok(BacklogCommand::Check(options))
        }
        "stats" => match rest {
            [] => Ok(BacklogCommand::Stats { write: false }),
            [flag] if flag == "--write" => Ok(BacklogCommand::Stats { write: true }),
            [other, ..] => Err(UsageError::UnexpectedArgument {
                command: "backlog stats",
                argument: other.clone(),
            }),
        },
        _ => Err(UsageError::UnknownSubcommand {
            command: "backlog",
            name: sub.clone(),
        }),
    }
}

fn parse_schemas(rest: &[String]) -> Result<SchemasCommand, UsageError> {
    let Some((sub, rest)) = rest.split_first() else {
        return Err(UsageError::MissingSubcommand { command: "schemas" });
    };
    match sub.as_str() {
        "verify" => no_arguments("schemas verify", rest).map(|()| SchemasCommand::Verify),
        "vendor" => {
            let mut tag = None;
            let mut dry_run = false;
            let mut args = rest.iter();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--dry-run" => dry_run = true,
                    "--tag" => {
                        tag = Some(
                            args.next()
                                .ok_or(UsageError::MissingValue { flag: "--tag" })?
                                .clone(),
                        );
                    }
                    _ => {
                        return Err(UsageError::UnexpectedArgument {
                            command: "schemas vendor",
                            argument: arg.clone(),
                        });
                    }
                }
            }
            let tag = tag.ok_or(UsageError::MissingArgument {
                command: "schemas vendor",
                name: "`--tag <tag>`",
            })?;
            Ok(SchemasCommand::Vendor { tag, dry_run })
        }
        _ => Err(UsageError::UnknownSubcommand {
            command: "schemas",
            name: sub.clone(),
        }),
    }
}

fn parse_docs(rest: &[String]) -> Result<DocsCommand, UsageError> {
    let Some((sub, rest)) = rest.split_first() else {
        return Err(UsageError::MissingSubcommand { command: "docs" });
    };
    match sub.as_str() {
        "check-crates" => {
            no_arguments("docs check-crates", rest).map(|()| DocsCommand::CheckCrates)
        }
        "new-crate-doc" => match rest {
            [name] => Ok(DocsCommand::NewCrateDoc { name: name.clone() }),
            [] => Err(UsageError::MissingArgument {
                command: "docs new-crate-doc",
                name: "a crate name",
            }),
            [_, extra, ..] => Err(UsageError::UnexpectedArgument {
                command: "docs new-crate-doc",
                argument: extra.clone(),
            }),
        },
        _ => Err(UsageError::UnknownSubcommand {
            command: "docs",
            name: sub.clone(),
        }),
    }
}

fn add_step(options: &mut CiOptions, name: &str) -> Result<(), UsageError> {
    if !ci::is_step(name) {
        return Err(UsageError::UnknownStep(name.to_owned()));
    }
    if !options.steps.iter().any(|step| step == name) {
        options.steps.push(name.to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{
        BacklogCommand, CiOptions, Command, DocsCommand, FuzzCommand, SchemasCommand, UsageError,
        parse,
    };

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    #[test]
    fn help_and_its_aliases_parse() {
        for name in ["help", "--help", "-h"] {
            assert_eq!(parse(args(&[name])), Ok(Command::Help), "{name}");
        }
    }

    #[test]
    fn no_command_is_a_usage_error() {
        assert_eq!(parse(args(&[])), Err(UsageError::MissingCommand));
    }

    #[test]
    fn unknown_command_is_named_in_the_error() {
        let error = parse(args(&["frobnicate"])).err();
        assert_eq!(
            error,
            Some(UsageError::UnknownCommand("frobnicate".to_owned()))
        );
        assert_eq!(
            error.map(|e| e.to_string()).as_deref(),
            Some("unknown command `frobnicate`")
        );
    }

    #[test]
    fn extra_arguments_to_help_are_rejected() {
        assert_eq!(
            parse(args(&["help", "now"])),
            Err(UsageError::UnexpectedArgument {
                command: "help",
                argument: "now".to_owned()
            })
        );
    }

    #[test]
    fn ci_without_flags_selects_every_step() {
        assert_eq!(parse(args(&["ci"])), Ok(Command::Ci(CiOptions::default())));
    }

    #[test]
    fn ci_flags_parse_in_both_spellings_and_repeat() {
        let parsed = parse(args(&[
            "ci",
            "--step",
            "fmt",
            "--step=test",
            "--step",
            "fmt",
            "--keep-going",
        ]));
        assert_eq!(
            parsed,
            Ok(Command::Ci(CiOptions {
                steps: vec!["fmt".to_owned(), "test".to_owned()],
                list: false,
                keep_going: true,
            }))
        );
        assert!(matches!(
            parse(args(&["ci", "--list"])),
            Ok(Command::Ci(CiOptions { list: true, .. }))
        ));
    }

    #[test]
    fn ci_rejects_unknown_steps_missing_values_and_unknown_flags() {
        assert_eq!(
            parse(args(&["ci", "--step", "nope"])),
            Err(UsageError::UnknownStep("nope".to_owned()))
        );
        assert_eq!(
            parse(args(&["ci", "--step"])),
            Err(UsageError::MissingValue { flag: "--step" })
        );
        assert_eq!(
            parse(args(&["ci", "--fast"])),
            Err(UsageError::UnexpectedArgument {
                command: "ci",
                argument: "--fast".to_owned()
            })
        );
    }

    #[test]
    fn tools_install_parses_and_other_subcommands_do_not() {
        assert_eq!(
            parse(args(&["tools", "install"])),
            Ok(Command::ToolsInstall {
                steps: Vec::new(),
                tools: Vec::new()
            })
        );
        assert_eq!(
            parse(args(&[
                "tools", "install", "--step", "test", "--step", "links"
            ])),
            Ok(Command::ToolsInstall {
                steps: vec!["test".to_owned(), "links".to_owned()],
                tools: Vec::new()
            })
        );
        assert_eq!(
            parse(args(&["tools", "install", "--step", "nope"])),
            Err(UsageError::UnknownStep("nope".to_owned()))
        );
        assert_eq!(
            parse(args(&["tools", "install", "--tool", "cargo-fuzz"])),
            Ok(Command::ToolsInstall {
                steps: Vec::new(),
                tools: vec!["cargo-fuzz".to_owned()]
            })
        );
        assert_eq!(
            parse(args(&["tools", "install", "--tool", "nope"])),
            Err(UsageError::UnknownTool("nope".to_owned()))
        );
        assert_eq!(
            parse(args(&["tools"])),
            Err(UsageError::MissingSubcommand { command: "tools" })
        );
        assert_eq!(
            parse(args(&["tools", "remove"])),
            Err(UsageError::UnknownSubcommand {
                command: "tools",
                name: "remove".to_owned()
            })
        );
    }

    #[test]
    fn bench_parses_an_optional_base() {
        assert_eq!(parse(args(&["bench"])), Ok(Command::Bench { base: None }));
        assert_eq!(
            parse(args(&["bench", "--base", "origin/main"])),
            Ok(Command::Bench {
                base: Some("origin/main".to_owned())
            })
        );
        assert_eq!(
            parse(args(&["bench", "--base"])),
            Err(UsageError::MissingValue { flag: "--base" })
        );
        assert_eq!(
            parse(args(&["bench", "main"])),
            Err(UsageError::UnexpectedArgument {
                command: "bench",
                argument: "main".to_owned()
            })
        );
    }

    #[test]
    fn fuzz_commands_parse() {
        let run = |target: &str, seconds| {
            Ok(Command::Fuzz(FuzzCommand::Run {
                target: target.to_owned(),
                seconds,
            }))
        };
        assert_eq!(
            parse(args(&["fuzz", "--list"])),
            Ok(Command::Fuzz(FuzzCommand::List))
        );
        assert_eq!(parse(args(&["fuzz", "t"])), run("t", 60));
        assert_eq!(parse(args(&["fuzz", "t", "--time", "90"])), run("t", 90));
        assert_eq!(parse(args(&["fuzz", "t", "--time", "90s"])), run("t", 90));
        assert_eq!(parse(args(&["fuzz", "t", "--time", "5m"])), run("t", 300));
        assert_eq!(parse(args(&["fuzz", "t", "--time", "1h"])), run("t", 3600));
        assert_eq!(
            parse(args(&["fuzz", "t", "crash-1"])),
            Ok(Command::Fuzz(FuzzCommand::Reproduce {
                target: "t".to_owned(),
                input: "crash-1".to_owned()
            }))
        );
        for bad in ["0", "0s", "s", "1d", "-1", ""] {
            assert_eq!(
                parse(args(&["fuzz", "t", "--time", bad])),
                Err(UsageError::InvalidNumber {
                    flag: "--time",
                    value: bad.to_owned()
                }),
                "{bad}"
            );
        }
        assert_eq!(
            parse(args(&["fuzz", "t", "--time"])),
            Err(UsageError::MissingValue { flag: "--time" })
        );
        assert_eq!(
            parse(args(&["fuzz", "t", "--fast"])),
            Err(UsageError::UnexpectedArgument {
                command: "fuzz",
                argument: "--fast".to_owned()
            })
        );
        assert_eq!(
            parse(args(&["fuzz"])),
            Err(UsageError::MissingArgument {
                command: "fuzz",
                name: "a target, or `--list`"
            })
        );
        assert_eq!(
            parse(args(&["fuzz", "corpus", "pull"])),
            Ok(Command::Fuzz(FuzzCommand::CorpusPull { targets: vec![] }))
        );
        assert_eq!(
            parse(args(&["fuzz", "corpus", "pull", "a", "b"])),
            Ok(Command::Fuzz(FuzzCommand::CorpusPull {
                targets: vec!["a".to_owned(), "b".to_owned()]
            }))
        );
        assert_eq!(
            parse(args(&["fuzz", "corpus", "minimize", "a"])),
            Ok(Command::Fuzz(FuzzCommand::CorpusMinimize {
                target: "a".to_owned()
            }))
        );
        assert_eq!(
            parse(args(&["fuzz", "corpus", "minimize"])),
            Err(UsageError::MissingArgument {
                command: "fuzz corpus minimize",
                name: "a target"
            })
        );
        assert_eq!(
            parse(args(&["fuzz", "corpus", "push"])),
            Err(UsageError::UnknownSubcommand {
                command: "fuzz corpus",
                name: "push".to_owned()
            })
        );
    }

    #[test]
    fn check_layers_flags_parse() {
        assert_eq!(
            parse(args(&[
                "check-layers",
                "--print",
                "--manifest-path",
                "a/Cargo.toml"
            ])),
            Ok(Command::CheckLayers(crate::check_layers::LayerOptions {
                print: true,
                manifest_path: Some("a/Cargo.toml".into()),
            }))
        );
        assert_eq!(
            parse(args(&["check-layers", "--manifest-path"])),
            Err(UsageError::MissingValue {
                flag: "--manifest-path"
            })
        );
    }

    #[test]
    fn check_manifests_and_lint_canary_take_no_arguments() {
        assert_eq!(
            parse(args(&["check-manifests"])),
            Ok(Command::CheckManifests)
        );
        assert_eq!(parse(args(&["lint-canary"])), Ok(Command::LintCanary));
        assert_eq!(parse(args(&["doctor"])), Ok(Command::Doctor));
        assert_eq!(
            parse(args(&["install-hooks"])),
            Ok(Command::InstallHooks { uninstall: false })
        );
        assert_eq!(
            parse(args(&["install-hooks", "--uninstall"])),
            Ok(Command::InstallHooks { uninstall: true })
        );
        assert_eq!(
            parse(args(&["install-hooks", "--force"])),
            Err(UsageError::UnexpectedArgument {
                command: "install-hooks",
                argument: "--force".to_owned()
            })
        );
        assert_eq!(
            parse(args(&["install-hooks", "x", "--uninstall"])),
            Err(UsageError::UnexpectedArgument {
                command: "install-hooks",
                argument: "x".to_owned()
            })
        );
        assert_eq!(
            parse(args(&["install-hooks", "--uninstall", "x"])),
            Err(UsageError::UnexpectedArgument {
                command: "install-hooks",
                argument: "x".to_owned()
            })
        );
        assert_eq!(
            parse(args(&["check-workflows"])),
            Ok(Command::CheckWorkflows)
        );
        assert_eq!(
            parse(args(&["lint-canary", "--fix"])),
            Err(UsageError::UnexpectedArgument {
                command: "lint-canary",
                argument: "--fix".to_owned()
            })
        );
    }

    #[test]
    fn backlog_subcommands_parse() {
        assert_eq!(
            parse(args(&["backlog", "next"])),
            Ok(Command::Backlog(BacklogCommand::Next {
                all: false,
                count: 10
            }))
        );
        assert_eq!(
            parse(args(&["backlog", "next", "-n", "3", "--all"])),
            Ok(Command::Backlog(BacklogCommand::Next {
                all: true,
                count: 3
            }))
        );
        assert_eq!(
            parse(args(&[
                "backlog",
                "check",
                "--epic",
                "FND",
                "--allow-empty-features",
                "--epic",
                "TYPE"
            ])),
            Ok(Command::Backlog(BacklogCommand::Check(
                crate::backlog_check::CheckOptions {
                    allow_empty_features: true,
                    epics: vec!["FND".to_owned(), "TYPE".to_owned()],
                }
            )))
        );
        assert_eq!(
            parse(args(&["backlog", "stats", "--write"])),
            Ok(Command::Backlog(BacklogCommand::Stats { write: true }))
        );
        assert_eq!(
            parse(args(&["backlog", "show", "FND-0041"])),
            Ok(Command::Backlog(BacklogCommand::Show {
                id: "FND-0041".to_owned()
            }))
        );
    }

    #[test]
    fn backlog_usage_errors_name_the_problem() {
        assert_eq!(
            parse(args(&["backlog", "next", "-n", "many"])),
            Err(UsageError::InvalidNumber {
                flag: "-n",
                value: "many".to_owned()
            })
        );
        assert_eq!(
            parse(args(&["backlog", "show"])),
            Err(UsageError::MissingArgument {
                command: "backlog show",
                name: "an item id"
            })
        );
        assert_eq!(
            parse(args(&["backlog", "show", "A", "B"])),
            Err(UsageError::UnexpectedArgument {
                command: "backlog show",
                argument: "B".to_owned()
            })
        );
        assert_eq!(
            parse(args(&["backlog"])),
            Err(UsageError::MissingSubcommand { command: "backlog" })
        );
    }

    #[test]
    fn schemas_subcommands_parse() {
        assert_eq!(
            parse(args(&["schemas", "verify"])),
            Ok(Command::Schemas(SchemasCommand::Verify))
        );
        assert_eq!(
            parse(args(&["schemas", "vendor", "--dry-run", "--tag", "4.4.0"])),
            Ok(Command::Schemas(SchemasCommand::Vendor {
                tag: "4.4.0".to_owned(),
                dry_run: true
            }))
        );
        assert_eq!(
            parse(args(&["schemas", "vendor"])),
            Err(UsageError::MissingArgument {
                command: "schemas vendor",
                name: "`--tag <tag>`"
            })
        );
        assert_eq!(
            parse(args(&["schemas"])),
            Err(UsageError::MissingSubcommand { command: "schemas" })
        );
        assert_eq!(
            parse(args(&["schemas", "verify", "x"])),
            Err(UsageError::UnexpectedArgument {
                command: "schemas verify",
                argument: "x".to_owned()
            })
        );
    }

    #[test]
    fn deps_diff_takes_exactly_one_base_ref() {
        assert_eq!(
            parse(args(&["deps-diff", "origin/main"])),
            Ok(Command::DepsDiff {
                base_ref: "origin/main".to_owned()
            })
        );
        assert_eq!(
            parse(args(&["deps-diff"])),
            Err(UsageError::MissingArgument {
                command: "deps-diff",
                name: "a base git ref, such as origin/main"
            })
        );
        assert_eq!(
            parse(args(&["deps-diff", "a", "b"])),
            Err(UsageError::UnexpectedArgument {
                command: "deps-diff",
                argument: "b".to_owned()
            })
        );
    }

    #[test]
    fn docs_subcommands_parse_and_reject_bad_arguments() {
        assert_eq!(
            parse(args(&["docs", "check-crates"])),
            Ok(Command::Docs(DocsCommand::CheckCrates))
        );
        assert_eq!(
            parse(args(&["docs", "new-crate-doc", "alluvium-raft"])),
            Ok(Command::Docs(DocsCommand::NewCrateDoc {
                name: "alluvium-raft".to_owned()
            }))
        );
        assert_eq!(
            parse(args(&["docs", "check-crates", "x"])),
            Err(UsageError::UnexpectedArgument {
                command: "docs check-crates",
                argument: "x".to_owned()
            })
        );
        assert_eq!(
            parse(args(&["docs", "new-crate-doc"])),
            Err(UsageError::MissingArgument {
                command: "docs new-crate-doc",
                name: "a crate name"
            })
        );
        assert_eq!(
            parse(args(&["docs", "new-crate-doc", "a", "b"])),
            Err(UsageError::UnexpectedArgument {
                command: "docs new-crate-doc",
                argument: "b".to_owned()
            })
        );
        assert_eq!(
            parse(args(&["docs"])),
            Err(UsageError::MissingSubcommand { command: "docs" })
        );
        assert_eq!(
            parse(args(&["docs", "build"])),
            Err(UsageError::UnknownSubcommand {
                command: "docs",
                name: "build".to_owned()
            })
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_arguments_are_rejected_without_panicking() {
        use std::os::unix::ffi::OsStringExt;
        let bad = OsString::from_vec(vec![0x66, 0xff]);
        assert_eq!(parse([bad.clone()]), Err(UsageError::NonUtf8Argument(bad)));
    }
}
