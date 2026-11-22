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
    /// `tools install [--step <name>]...`: only the tools those CI steps need, or all.
    ToolsInstall {
        steps: Vec<String>,
    },
    CheckLayers(LayerOptions),
    CheckManifests,
    CheckWorkflows,
    LintCanary,
    Backlog(BacklogCommand),
    Doctor,
}

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
        "tools install [--step <name>]...",
        "Install the pinned external tools, or only those the named CI steps need",
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
        "doctor",
        "Check the local toolchain and tools, and print the fix for each problem",
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
        "lint-canary" => no_arguments("lint-canary", rest).map(|()| Command::LintCanary),
        "doctor" => no_arguments("doctor", rest).map(|()| Command::Doctor),
        "tools" => match rest.split_first() {
            None => Err(UsageError::MissingSubcommand { command: "tools" }),
            Some((sub, rest)) if sub == "install" => {
                parse_tools_install(rest).map(|steps| Command::ToolsInstall { steps })
            }
            Some((sub, _)) => Err(UsageError::UnknownSubcommand {
                command: "tools",
                name: sub.clone(),
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

fn parse_tools_install(rest: &[String]) -> Result<Vec<String>, UsageError> {
    let mut options = CiOptions::default();
    let mut args = rest.iter();
    while let Some(arg) = args.next() {
        if arg != "--step" {
            return Err(UsageError::UnexpectedArgument {
                command: "tools install",
                argument: arg.clone(),
            });
        }
        let name = args
            .next()
            .ok_or(UsageError::MissingValue { flag: "--step" })?;
        add_step(&mut options, name)?;
    }
    Ok(options.steps)
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

    use super::{BacklogCommand, CiOptions, Command, UsageError, parse};

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
            Ok(Command::ToolsInstall { steps: Vec::new() })
        );
        assert_eq!(
            parse(args(&[
                "tools", "install", "--step", "test", "--step", "links"
            ])),
            Ok(Command::ToolsInstall {
                steps: vec!["test".to_owned(), "links".to_owned()]
            })
        );
        assert_eq!(
            parse(args(&["tools", "install", "--step", "nope"])),
            Err(UsageError::UnknownStep("nope".to_owned()))
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

    #[cfg(unix)]
    #[test]
    fn non_utf8_arguments_are_rejected_without_panicking() {
        use std::os::unix::ffi::OsStringExt;
        let bad = OsString::from_vec(vec![0x66, 0xff]);
        assert_eq!(parse([bad.clone()]), Err(UsageError::NonUtf8Argument(bad)));
    }
}
