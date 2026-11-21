//! Pinned versions of the external tools the checks use.
//!
//! [`TOOLS`] is the only place a tool version appears. CI and `cargo xtask tools install`
//! read it, and a step that needs a tool checks the installed version before it runs.
//! To upgrade a tool, change its version here in its own PR.

use std::ffi::OsString;
use std::fmt;
use std::process::Command;

/// An external cargo subcommand installed from crates.io.
#[derive(Debug)]
pub(crate) struct Tool {
    /// The crate to `cargo install`, which is also the first word of `--version` output.
    pub(crate) crate_name: &'static str,
    pub(crate) version: &'static str,
    /// The cargo subcommand it provides: `cargo <subcommand>`.
    pub(crate) subcommand: &'static str,
}

/// Every pinned tool.
pub(crate) const TOOLS: &[Tool] = &[NEXTEST, DENY];

/// Test runner used by the `test` step.
pub(crate) const NEXTEST: Tool = Tool {
    crate_name: "cargo-nextest",
    version: "0.9.152",
    subcommand: "nextest",
};

/// Dependency policy checker used by the `deny` step.
pub(crate) const DENY: Tool = Tool {
    crate_name: "cargo-deny",
    version: "0.22.0",
    subcommand: "deny",
};

impl Tool {
    /// The command that installs exactly the pinned version.
    pub(crate) fn install_command(&self) -> String {
        format!(
            "cargo install --locked --version {} {}",
            self.version, self.crate_name
        )
    }
}

/// Why a pinned tool can't be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ToolProblem {
    Missing {
        tool: &'static str,
        version: &'static str,
        install: String,
    },
    WrongVersion {
        tool: &'static str,
        version: &'static str,
        installed: String,
        install: String,
    },
}

impl fmt::Display for ToolProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolProblem::Missing {
                tool,
                version,
                install,
            } => write!(
                f,
                "{tool} {version} is required but not installed; run `cargo xtask tools install` or `{install}`"
            ),
            ToolProblem::WrongVersion {
                tool,
                version,
                installed,
                install,
            } => write!(
                f,
                "{tool} {version} is required but {installed} is installed; run `cargo xtask tools install` or `{install}`"
            ),
        }
    }
}

/// The `cargo` that started `xtask`, so steps use the toolchain from
/// `rust-toolchain.toml` even when another `cargo` comes first on `PATH`.
pub(crate) fn cargo_program() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

/// Checks that `tool` is installed at its pinned version.
pub(crate) fn check(tool: &Tool) -> Result<(), ToolProblem> {
    let missing = || ToolProblem::Missing {
        tool: tool.crate_name,
        version: tool.version,
        install: tool.install_command(),
    };
    let output = Command::new(cargo_program())
        .args([tool.subcommand, "--version"])
        .output()
        .map_err(|_| missing())?;
    if !output.status.success() {
        return Err(missing());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    match parse_version(&stdout, tool.crate_name) {
        Some(installed) if installed == tool.version => Ok(()),
        installed => Err(ToolProblem::WrongVersion {
            tool: tool.crate_name,
            version: tool.version,
            installed: installed.unwrap_or("an unrecognised version").to_owned(),
            install: tool.install_command(),
        }),
    }
}

/// Extracts the version from `--version` output such as `cargo-nextest 0.9.152 (abc 2026-11-01)`.
fn parse_version<'a>(output: &'a str, crate_name: &str) -> Option<&'a str> {
    let mut words = output.lines().next()?.split_whitespace();
    if words.next()? != crate_name {
        return None;
    }
    words.next().map(|version| version.trim_start_matches('v'))
}

#[cfg(test)]
mod tests {
    use super::{DENY, NEXTEST, TOOLS, parse_version};

    #[test]
    fn parses_nextest_version_output() {
        let output = "cargo-nextest 0.9.152 (abc1234 2026-11-01)\nrelease: 0.9.152\n";
        assert_eq!(parse_version(output, NEXTEST.crate_name), Some("0.9.152"));
    }

    #[test]
    fn parses_deny_version_output() {
        assert_eq!(
            parse_version("cargo-deny 0.22.0\n", DENY.crate_name),
            Some("0.22.0")
        );
    }

    #[test]
    fn rejects_output_from_another_program() {
        assert_eq!(
            parse_version("error: no such command", NEXTEST.crate_name),
            None
        );
        assert_eq!(parse_version("", NEXTEST.crate_name), None);
    }

    #[test]
    fn every_tool_has_an_exact_version() {
        for tool in TOOLS {
            assert!(
                tool.version.split('.').count() == 3
                    && tool
                        .version
                        .split('.')
                        .all(|part| part.parse::<u32>().is_ok()),
                "{} must pin an exact x.y.z version, not {}",
                tool.crate_name,
                tool.version
            );
        }
    }

    #[test]
    fn install_command_pins_the_version() {
        assert_eq!(
            NEXTEST.install_command(),
            "cargo install --locked --version 0.9.152 cargo-nextest"
        );
    }
}
