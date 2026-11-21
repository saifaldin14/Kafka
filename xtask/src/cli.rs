//! Command-line parsing for `xtask`.
//!
//! Std only: the grammar is a command name plus a few flags, and an argument-parsing
//! crate would add a dependency for no gain (docs/standards/security.md#dependencies).

use std::ffi::OsString;
use std::fmt;

/// A parsed `xtask` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
    Help,
}

/// Every command with its one-line summary, in the order `help` prints them.
pub(crate) const COMMANDS: &[(&str, &str)] = &[("help", "Print this list of commands")];

/// Why the command line could not be parsed. Always exit code 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UsageError {
    MissingCommand,
    UnknownCommand(String),
    UnexpectedArgument {
        command: &'static str,
        argument: String,
    },
    NonUtf8Argument(OsString),
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UsageError::MissingCommand => write!(f, "no command given"),
            UsageError::UnknownCommand(name) => write!(f, "unknown command `{name}`"),
            UsageError::UnexpectedArgument { command, argument } => {
                write!(f, "`{command}` takes no argument `{argument}`")
            }
            UsageError::NonUtf8Argument(argument) => {
                write!(f, "argument `{}` is not valid UTF-8", argument.display())
            }
        }
    }
}

/// Parses the arguments after the program name.
pub(crate) fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Command, UsageError> {
    let mut args = args
        .into_iter()
        .map(|arg| arg.into_string().map_err(UsageError::NonUtf8Argument));
    let name = args.next().ok_or(UsageError::MissingCommand)??;
    let command = match name.as_str() {
        "help" | "--help" | "-h" => Command::Help,
        _ => return Err(UsageError::UnknownCommand(name)),
    };
    if let Some(argument) = args.next() {
        return Err(UsageError::UnexpectedArgument {
            command: "help",
            argument: argument?,
        });
    }
    Ok(command)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{Command, UsageError, parse};

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
    fn extra_arguments_are_rejected() {
        assert_eq!(
            parse(args(&["help", "now"])),
            Err(UsageError::UnexpectedArgument {
                command: "help",
                argument: "now".to_owned()
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
