//! The only place `xtask` writes to the terminal.
//!
//! Writes go through `std::io::Write`, so callers pass stdout, stderr or a buffer in
//! tests. A failed write (for example stdout piped into `head`) is not an `xtask`
//! failure, so write errors are deliberately ignored here and nowhere else.

use std::io::Write;

use crate::cli::{COMMANDS, UsageError};

/// Writes the usage text and the command list.
pub(crate) fn usage(out: &mut impl Write) {
    let width = COMMANDS
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0);
    write_all(out, "Usage: cargo xtask <command>\n\nCommands:\n");
    for (name, summary) in COMMANDS {
        write_all(out, &format!("  {name:width$}  {summary}\n"));
    }
}

/// Writes a usage error followed by the usage text.
pub(crate) fn usage_error(out: &mut impl Write, error: &UsageError) {
    write_all(out, &format!("error: {error}\n\n"));
    usage(out);
}

fn write_all(out: &mut impl Write, text: &str) {
    // Ignored because a closed terminal or pipe must not turn into a failed check.
    let _ = out.write_all(text.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::{usage, usage_error};
    use crate::cli::UsageError;

    #[test]
    fn usage_lists_every_command_aligned() {
        let mut out = Vec::new();
        usage(&mut out);
        let text = String::from_utf8_lossy(&out);
        for (name, summary) in crate::cli::COMMANDS {
            assert!(text.contains(name) && text.contains(summary), "{text}");
        }
    }

    #[test]
    fn usage_error_names_the_problem_first() {
        let mut out = Vec::new();
        usage_error(&mut out, &UsageError::UnknownCommand("x".to_owned()));
        assert!(String::from_utf8_lossy(&out).starts_with("error: unknown command `x`\n\nUsage:"));
    }
}
