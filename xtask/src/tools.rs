//! Pinned versions of the external tools the checks use.
//!
//! [`TOOLS`] is the only place a tool version appears. CI and `cargo xtask tools install`
//! read it, and a step that needs a tool checks the installed version before it runs.
//! To upgrade a tool, change its version (and, for a release download, its URLs and
//! checksums) here in its own PR.

use std::ffi::OsString;
use std::fmt;
use std::process::Command;

/// An external tool at a pinned version.
#[derive(Debug)]
pub(crate) struct Tool {
    /// The tool's name: the crate to `cargo install`, or the binary of a release.
    pub(crate) name: &'static str,
    pub(crate) version: &'static str,
    pub(crate) invocation: Invocation,
    pub(crate) source: Source,
    /// How `--version` (or `-version`) output shows the version.
    pub(crate) version_output: VersionOutput,
}

/// How a tool is run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Invocation {
    /// As `cargo <subcommand>`.
    CargoSubcommand(&'static str),
    /// As a program on `PATH`.
    Binary(&'static str),
}

/// Where a tool is installed from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Source {
    /// crates.io, with `cargo install --locked --version`.
    Crate,
    /// A prebuilt release archive (`.tar.gz`) per platform, checked against its SHA-256
    /// and unpacked into cargo's bin directory. For tools not written in Rust.
    Release {
        assets: &'static [ReleaseAsset],
        /// The binary's path inside the archive.
        path_in_archive: &'static str,
    },
}

/// One platform's release archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReleaseAsset {
    /// `std::env::consts::OS`, such as `linux` or `macos`.
    pub(crate) os: &'static str,
    /// `std::env::consts::ARCH`, such as `x86_64` or `aarch64`.
    pub(crate) arch: &'static str,
    pub(crate) url: &'static str,
    pub(crate) sha256: &'static str,
}

/// Where the version is in a tool's version output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VersionOutput {
    /// The first line is `<name> <version> …`, as cargo tools print.
    NameThenVersion,
    /// The first line is the version itself (actionlint's `-version`).
    FirstLine,
    /// A line starting with this prefix holds the version (shellcheck's `version: `).
    Prefixed(&'static str),
}

/// Every pinned tool.
pub(crate) const TOOLS: &[Tool] = &[NEXTEST, DENY, LYCHEE, ACTIONLINT, SHELLCHECK, CARGO_FUZZ];

/// The tools some CI step needs. These are required; the others, used only by commands
/// such as `cargo xtask fuzz`, are installed on request with `--tool`.
pub(crate) fn required_by_steps() -> impl Iterator<Item = &'static Tool> {
    TOOLS.iter().filter(|tool| {
        crate::ci::STEPS
            .iter()
            .any(|step| step.requires.iter().any(|needed| needed.name == tool.name))
    })
}

/// Test runner used by the `test` step.
pub(crate) const NEXTEST: Tool = Tool {
    name: "cargo-nextest",
    version: "0.9.152",
    invocation: Invocation::CargoSubcommand("nextest"),
    source: Source::Crate,
    version_output: VersionOutput::NameThenVersion,
};

/// Dependency policy checker used by the `deny` step.
pub(crate) const DENY: Tool = Tool {
    name: "cargo-deny",
    version: "0.22.0",
    invocation: Invocation::CargoSubcommand("deny"),
    source: Source::Crate,
    version_output: VersionOutput::NameThenVersion,
};

/// Link checker used by the `links` step.
pub(crate) const LYCHEE: Tool = Tool {
    name: "lychee",
    version: "0.24.2",
    invocation: Invocation::Binary("lychee"),
    source: Source::Crate,
    version_output: VersionOutput::NameThenVersion,
};

/// The fuzzing driver `cargo xtask fuzz` builds targets with (fuzz/README.md).
pub(crate) const CARGO_FUZZ: Tool = Tool {
    name: "cargo-fuzz",
    version: "0.13.2",
    invocation: Invocation::CargoSubcommand("fuzz"),
    source: Source::Crate,
    version_output: VersionOutput::NameThenVersion,
};

/// GitHub Actions workflow linter used by the `workflows` step. Written in Go, so it
/// comes from its GitHub release; the checksums match the release's
/// `actionlint_1.7.12_checksums.txt`.
pub(crate) const ACTIONLINT: Tool = Tool {
    name: "actionlint",
    version: "1.7.12",
    invocation: Invocation::Binary("actionlint"),
    source: Source::Release {
        assets: &[
            ReleaseAsset {
                os: "linux",
                arch: "x86_64",
                url: "https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_linux_amd64.tar.gz",
                sha256: "8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8",
            },
            ReleaseAsset {
                os: "linux",
                arch: "aarch64",
                url: "https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_linux_arm64.tar.gz",
                sha256: "325e971b6ba9bfa504672e29be93c24981eeb1c07576d730e9f7c8805afff0c6",
            },
            ReleaseAsset {
                os: "macos",
                arch: "x86_64",
                url: "https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_darwin_amd64.tar.gz",
                sha256: "5b44c3bc2255115c9b69e30efc0fecdf498fdb63c5d58e17084fd5f16324c644",
            },
            ReleaseAsset {
                os: "macos",
                arch: "aarch64",
                url: "https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_darwin_arm64.tar.gz",
                sha256: "aba9ced2dee8d27fecca3dc7feb1a7f9a52caefa1eb46f3271ea66b6e0e6953f",
            },
        ],
        path_in_archive: "actionlint",
    },
    version_output: VersionOutput::FirstLine,
};

/// Shell script linter that actionlint runs on `run:` blocks. A Haskell binary from its
/// GitHub release, which publishes no checksums: these were computed when the version
/// was pinned.
pub(crate) const SHELLCHECK: Tool = Tool {
    name: "shellcheck",
    version: "0.11.0",
    invocation: Invocation::Binary("shellcheck"),
    source: Source::Release {
        assets: &[
            ReleaseAsset {
                os: "linux",
                arch: "x86_64",
                url: "https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.linux.x86_64.tar.gz",
                sha256: "b7af85e41cc99489dcc21d66c6d5f3685138f06d34651e6d34b42ec6d54fe6f6",
            },
            ReleaseAsset {
                os: "linux",
                arch: "aarch64",
                url: "https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.linux.aarch64.tar.gz",
                sha256: "68a8133197a50beb8803f8d42f9908d1af1c5540d4bb05fdfca8c1fa47decefc",
            },
            ReleaseAsset {
                os: "macos",
                arch: "x86_64",
                url: "https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.darwin.x86_64.tar.gz",
                sha256: "c2c15e08df0e8fbc374c335b230a7ee958c313fa5714817a59aa59f1aa594f51",
            },
            ReleaseAsset {
                os: "macos",
                arch: "aarch64",
                url: "https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.darwin.aarch64.tar.gz",
                sha256: "339b930feb1ea764467013cc1f72d09cd6b869ebf1013296ba9055ab2ffbd26f",
            },
        ],
        path_in_archive: "shellcheck-v0.11.0/shellcheck",
    },
    version_output: VersionOutput::Prefixed("version: "),
};

impl Tool {
    /// The command that installs exactly the pinned version.
    pub(crate) fn install_command(&self) -> String {
        match self.source {
            Source::Crate => format!(
                "cargo install --locked --version {} {}",
                self.version, self.name
            ),
            Source::Release { .. } => "cargo xtask tools install".to_owned(),
        }
    }

    /// This platform's release archive, for a release tool.
    pub(crate) fn release_asset(&self, os: &str, arch: &str) -> Option<&'static ReleaseAsset> {
        match self.source {
            Source::Crate => None,
            Source::Release { assets, .. } => assets
                .iter()
                .find(|asset| asset.os == os && asset.arch == arch),
        }
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
                "{tool} {version} is required but not installed; run `cargo xtask tools install --tool {tool}` or `{install}`"
            ),
            ToolProblem::WrongVersion {
                tool,
                version,
                installed,
                install,
            } => write!(
                f,
                "{tool} {version} is required but {installed} is installed; run `cargo xtask tools install --tool {tool}` or `{install}`"
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
        tool: tool.name,
        version: tool.version,
        install: tool.install_command(),
    };
    let mut command = match tool.invocation {
        Invocation::CargoSubcommand(subcommand) => {
            let mut command = Command::new(cargo_program());
            command.arg(subcommand);
            command
        }
        Invocation::Binary(program) => Command::new(program),
    };
    // actionlint takes Go's single-dash flag; the others take `--version`.
    let flag = if tool.version_output == VersionOutput::FirstLine {
        "-version"
    } else {
        "--version"
    };
    let output = command.arg(flag).output().map_err(|_| missing())?;
    if !output.status.success() {
        return Err(missing());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    match parse_version(&stdout, tool) {
        Some(installed) if installed == tool.version => Ok(()),
        installed => Err(ToolProblem::WrongVersion {
            tool: tool.name,
            version: tool.version,
            installed: installed.unwrap_or("an unrecognised version").to_owned(),
            install: tool.install_command(),
        }),
    }
}

/// Extracts the version from version output, as the tool's [`VersionOutput`] says.
fn parse_version<'a>(output: &'a str, tool: &Tool) -> Option<&'a str> {
    let version = match tool.version_output {
        VersionOutput::NameThenVersion => {
            let mut words = output.lines().next()?.split_whitespace();
            if words.next()? != tool.name {
                return None;
            }
            words.next()?
        }
        VersionOutput::FirstLine => output.lines().next()?.trim(),
        VersionOutput::Prefixed(prefix) => output
            .lines()
            .find_map(|line| line.strip_prefix(prefix))?
            .trim(),
    };
    Some(version.trim_start_matches('v')).filter(|version| !version.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{ACTIONLINT, DENY, LYCHEE, NEXTEST, SHELLCHECK, Source, TOOLS, parse_version};

    #[test]
    fn parses_nextest_version_output() {
        let output = "cargo-nextest 0.9.152 (abc1234 2026-11-01)\nrelease: 0.9.152\n";
        assert_eq!(parse_version(output, &NEXTEST), Some("0.9.152"));
    }

    #[test]
    fn parses_deny_version_output() {
        assert_eq!(parse_version("cargo-deny 0.22.0\n", &DENY), Some("0.22.0"));
    }

    #[test]
    fn rejects_output_from_another_program() {
        assert_eq!(parse_version("error: no such command", &NEXTEST), None);
        assert_eq!(parse_version("", &NEXTEST), None);
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
                tool.name,
                tool.version
            );
        }
    }

    #[test]
    fn parses_lychee_version_output() {
        assert_eq!(parse_version("lychee 0.24.2\n", &LYCHEE), Some("0.24.2"));
    }

    #[test]
    fn parses_actionlint_and_shellcheck_version_output() {
        let actionlint =
            "1.7.12\ninstalled by downloading from release page\nbuilt with go1.26.1\n";
        assert_eq!(parse_version(actionlint, &ACTIONLINT), Some("1.7.12"));
        let shellcheck =
            "ShellCheck - shell script analysis tool\nversion: 0.11.0\nlicense: GPL-3\n";
        assert_eq!(parse_version(shellcheck, &SHELLCHECK), Some("0.11.0"));
        assert_eq!(parse_version("", &ACTIONLINT), None);
        assert_eq!(parse_version("ShellCheck\n", &SHELLCHECK), None);
    }

    #[test]
    fn every_release_tool_covers_the_supported_platforms_with_checksums() {
        for tool in TOOLS {
            if let Source::Release { assets, .. } = tool.source {
                for (os, arch) in [
                    ("linux", "x86_64"),
                    ("linux", "aarch64"),
                    ("macos", "x86_64"),
                    ("macos", "aarch64"),
                ] {
                    let asset = tool
                        .release_asset(os, arch)
                        .unwrap_or_else(|| panic!("{} has no {os}/{arch} release", tool.name));
                    assert!(asset.url.contains(tool.version), "{}", asset.url);
                    assert!(asset.url.starts_with("https://") && asset.url.ends_with(".tar.gz"));
                    assert!(
                        asset.sha256.len() == 64
                            && asset
                                .sha256
                                .bytes()
                                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
                        "{}",
                        asset.sha256
                    );
                }
                assert_eq!(assets.len(), 4, "{}", tool.name);
            }
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
