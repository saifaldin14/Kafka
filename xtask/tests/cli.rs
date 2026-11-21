//! End-to-end tests of the `xtask` binary: dispatch and exit codes.

use std::error::Error;
use std::process::{Command, Output};

fn xtask(args: &[&str]) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .output()?)
}

#[test]
fn help_lists_commands_and_exits_0() -> Result<(), Box<dyn Error>> {
    let output = xtask(&["help"])?;
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("Commands:") && stdout.contains("help"),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn unknown_command_exits_2_with_usage_on_stderr() -> Result<(), Box<dyn Error>> {
    let output = xtask(&["no-such-command"])?;
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8(output.stdout)?, "");
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.starts_with("error: unknown command `no-such-command`"),
        "{stderr}"
    );
    assert!(stderr.contains("Usage: cargo xtask"), "{stderr}");
    Ok(())
}

#[test]
fn missing_command_exits_2() -> Result<(), Box<dyn Error>> {
    assert_eq!(xtask(&[])?.status.code(), Some(2));
    Ok(())
}
