//! `cargo xtask doctor`: checks the local toolchain and tools, and prints the command
//! that fixes each problem.
//!
//! Required checks (exit code 1 if any fails):
//!
//! - the `cargo` first on `PATH` is rustup's proxy, so it honours `rust-toolchain.toml`
//!   (Homebrew's or a distribution's `cargo` doesn't);
//! - the active toolchain is the one `rust-toolchain.toml` pins, with its components;
//! - every tool in the xtask tool table is installed at its pinned version.
//!
//! Optional checks (reported, never failing): Docker for compatibility tests, and a
//! nightly toolchain with cargo-fuzz for fuzzing.
//!
//! Facts are gathered by running commands, then judged by [`evaluate`], a pure function
//! the tests feed with fake command output.

use std::io::Write;
use std::process::Command;

use serde_json::Value;

use crate::Exit;
use crate::output;
use crate::tools::{self, TOOLS};

/// What the machine reports, gathered by [`gather`].
#[derive(Debug, Clone, Default)]
pub(crate) struct Facts {
    /// `cargo --version` of the `cargo` first on `PATH`, run in the repository.
    pub(crate) path_cargo_version: Option<String>,
    /// `rustup which cargo` in the repository: the active toolchain's real `cargo`.
    pub(crate) rustup_cargo_path: Option<String>,
    /// `--version` of that `cargo`.
    pub(crate) rustup_cargo_version: Option<String>,
    /// `rustup component list --installed` for the active toolchain.
    pub(crate) installed_components: Option<String>,
    /// Each pinned tool, and the problem with it if any.
    pub(crate) tools: Vec<(&'static str, Option<String>)>,
    pub(crate) docker: bool,
    pub(crate) nightly: bool,
    pub(crate) cargo_fuzz: bool,
}

/// The toolchain `rust-toolchain.toml` pins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pinned {
    pub(crate) channel: String,
    pub(crate) components: Vec<String>,
}

/// One check's result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Check {
    pub(crate) name: String,
    pub(crate) required: bool,
    /// `None` when the check passed; otherwise the command or action that fixes it.
    pub(crate) fix: Option<String>,
}

pub(crate) fn run(out: &mut impl Write) -> Exit {
    let pinned = match read_pinned() {
        Ok(pinned) => pinned,
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            return Exit::CheckFailed;
        }
    };
    let checks = evaluate(&gather(), &pinned);
    for check in &checks {
        let kind = if check.required { "" } else { " (optional)" };
        match &check.fix {
            None => output::line(out, &format!("ok       {}{kind}", check.name)),
            Some(fix) => output::line(out, &format!("missing  {}{kind}: {fix}", check.name)),
        }
    }
    if checks
        .iter()
        .any(|check| check.required && check.fix.is_some())
    {
        output::line(
            out,
            "doctor: a required check failed; run the fix shown above",
        );
        Exit::CheckFailed
    } else {
        output::line(out, "doctor: ok");
        Exit::Success
    }
}

fn read_pinned() -> Result<Pinned, String> {
    let file = "rust-toolchain.toml";
    let text = std::fs::read_to_string(crate::repo_root().join(file))
        .map_err(|error| format!("could not read {file}: {error}"))?;
    parse_pinned(&text).ok_or_else(|| format!("{file} has no [toolchain] channel"))
}

fn parse_pinned(text: &str) -> Option<Pinned> {
    let value: Value = toml::from_str(text).ok()?;
    let toolchain = value.get("toolchain")?;
    Some(Pinned {
        channel: toolchain.get("channel")?.as_str()?.to_owned(),
        components: toolchain
            .get("components")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|component| component.as_str().map(str::to_owned))
            .collect(),
    })
}

/// Runs `program args…` in the repository and returns its trimmed stdout if it
/// succeeded.
fn stdout_of(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(crate::repo_root())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn gather() -> Facts {
    let rustup_cargo_path = stdout_of("rustup", &["which", "cargo"]);
    let rustup_cargo_version = rustup_cargo_path
        .as_deref()
        .and_then(|path| stdout_of(path, &["--version"]));
    Facts {
        path_cargo_version: stdout_of("cargo", &["--version"]),
        rustup_cargo_path,
        rustup_cargo_version,
        installed_components: stdout_of("rustup", &["component", "list", "--installed"]),
        tools: TOOLS
            .iter()
            .map(|tool| {
                (
                    tool.name,
                    tools::check(tool).err().map(|problem| problem.to_string()),
                )
            })
            .collect(),
        docker: stdout_of("docker", &["--version"]).is_some(),
        nightly: stdout_of("rustup", &["toolchain", "list"])
            .is_some_and(|list| list.lines().any(|line| line.starts_with("nightly"))),
        cargo_fuzz: stdout_of("cargo", &["fuzz", "--version"]).is_some(),
    }
}

/// Judges the facts against the pinned toolchain.
pub(crate) fn evaluate(facts: &Facts, pinned: &Pinned) -> Vec<Check> {
    let channel = &pinned.channel;
    let mut checks = Vec::new();
    let mut push = |name: String, required: bool, fix: Option<String>| {
        checks.push(Check {
            name,
            required,
            fix,
        });
    };

    let rustup_ok = facts.rustup_cargo_path.is_some();
    push(
        "rustup".to_owned(),
        true,
        (!rustup_ok).then(|| "install rustup from https://rustup.rs".to_owned()),
    );

    // rustup's proxy runs the active toolchain's cargo, so the two report the same
    // version; any other cargo on PATH reports its own.
    let proxied = facts.path_cargo_version.is_some()
        && facts.path_cargo_version == facts.rustup_cargo_version;
    push(
        "cargo on PATH is rustup's proxy".to_owned(),
        true,
        (rustup_ok && !proxied).then(|| {
            format!(
                "the first cargo on PATH reports `{}`, not rustup's `{}`; put rustup's bin \
                 directory first (`export PATH=\"$HOME/.cargo/bin:$PATH\"`) or uninstall the \
                 other Rust (for Homebrew: `brew uninstall rust`)",
                facts.path_cargo_version.as_deref().unwrap_or("nothing"),
                facts.rustup_cargo_version.as_deref().unwrap_or("unknown"),
            )
        }),
    );

    let marker = format!("/toolchains/{channel}-");
    let toolchain_ok = facts
        .rustup_cargo_path
        .as_deref()
        .is_some_and(|path| path.replace('\\', "/").contains(&marker));
    push(
        format!("toolchain {channel} from rust-toolchain.toml"),
        true,
        (rustup_ok && !toolchain_ok).then(|| {
            format!(
                "rustup uses `{}`; remove the override with `rustup override unset` and unset \
                 RUSTUP_TOOLCHAIN, so rust-toolchain.toml selects {channel}",
                facts.rustup_cargo_path.as_deref().unwrap_or_default()
            )
        }),
    );

    for component in &pinned.components {
        let installed = facts.installed_components.as_deref().is_some_and(|list| {
            list.lines()
                .any(|line| line == component || line.starts_with(&format!("{component}-")))
        });
        push(
            format!("component {component}"),
            true,
            (rustup_ok && toolchain_ok && !installed)
                .then(|| format!("rustup component add {component} --toolchain {channel}")),
        );
    }

    for (tool, problem) in &facts.tools {
        push((*tool).to_owned(), true, problem.clone());
    }

    push(
        "docker, for compatibility tests".to_owned(),
        false,
        (!facts.docker).then(|| "install Docker: https://docs.docker.com/get-docker/".to_owned()),
    );
    push(
        "nightly toolchain, for fuzzing".to_owned(),
        false,
        (!facts.nightly).then(|| "rustup toolchain install nightly".to_owned()),
    );
    push(
        "cargo-fuzz, for fuzzing".to_owned(),
        false,
        (!facts.cargo_fuzz).then(|| "cargo install --locked cargo-fuzz".to_owned()),
    );
    checks
}

#[cfg(test)]
mod tests {
    use super::{Check, Facts, Pinned, evaluate, parse_pinned};

    fn pinned() -> Pinned {
        Pinned {
            channel: "1.99.0".to_owned(),
            components: vec!["rustfmt".to_owned(), "clippy".to_owned()],
        }
    }

    fn healthy() -> Facts {
        Facts {
            path_cargo_version: Some("cargo 1.99.0 (abc 2026-10-01)".to_owned()),
            rustup_cargo_path: Some(
                "/home/dev/.rustup/toolchains/1.99.0-x86_64-unknown-linux-gnu/bin/cargo"
                    .to_owned(),
            ),
            rustup_cargo_version: Some("cargo 1.99.0 (abc 2026-10-01)".to_owned()),
            installed_components: Some(
                "cargo-x86_64-unknown-linux-gnu\nclippy-x86_64-unknown-linux-gnu\nrustfmt-x86_64-unknown-linux-gnu"
                    .to_owned(),
            ),
            tools: vec![("cargo-nextest", None), ("cargo-deny", None)],
            docker: true,
            nightly: true,
            cargo_fuzz: true,
        }
    }

    fn failing(checks: &[Check]) -> Vec<(&str, bool, &str)> {
        checks
            .iter()
            .filter_map(|check| {
                check
                    .fix
                    .as_deref()
                    .map(|fix| (check.name.as_str(), check.required, fix))
            })
            .collect()
    }

    #[test]
    fn a_healthy_machine_passes_every_check() {
        assert_eq!(failing(&evaluate(&healthy(), &pinned())), []);
    }

    #[test]
    fn homebrew_cargo_first_on_path_is_reported_with_the_path_fix() {
        let facts = Facts {
            path_cargo_version: Some("cargo 1.88.0 (Homebrew)".to_owned()),
            ..healthy()
        };
        let failures = evaluate(&facts, &pinned());
        let failures = failing(&failures);
        assert_eq!(failures.len(), 1, "{failures:?}");
        let (name, required, fix) = failures[0];
        assert_eq!(name, "cargo on PATH is rustup's proxy");
        assert!(required);
        assert!(fix.contains("cargo 1.88.0 (Homebrew)"), "{fix}");
        assert!(
            fix.contains("export PATH=\"$HOME/.cargo/bin:$PATH\""),
            "{fix}"
        );
    }

    #[test]
    fn a_wrong_toolchain_is_reported_with_the_override_fix() {
        let facts = Facts {
            path_cargo_version: Some("cargo 1.95.0 (def 2026-06-01)".to_owned()),
            rustup_cargo_path: Some(
                "/home/dev/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo".to_owned(),
            ),
            rustup_cargo_version: Some("cargo 1.95.0 (def 2026-06-01)".to_owned()),
            ..healthy()
        };
        let checks = evaluate(&facts, &pinned());
        let failures = failing(&checks);
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert_eq!(failures[0].0, "toolchain 1.99.0 from rust-toolchain.toml");
        assert!(
            failures[0].2.contains("rustup override unset"),
            "{}",
            failures[0].2
        );
    }

    #[test]
    fn missing_components_tools_and_rustup_are_required_with_their_fixes() {
        let facts = Facts {
            installed_components: Some("cargo-x86_64-unknown-linux-gnu".to_owned()),
            tools: vec![(
                "cargo-deny",
                Some("cargo-deny 0.22.0 is required".to_owned()),
            )],
            ..healthy()
        };
        let checks = evaluate(&facts, &pinned());
        assert_eq!(
            failing(&checks),
            [
                (
                    "component rustfmt",
                    true,
                    "rustup component add rustfmt --toolchain 1.99.0"
                ),
                (
                    "component clippy",
                    true,
                    "rustup component add clippy --toolchain 1.99.0"
                ),
                ("cargo-deny", true, "cargo-deny 0.22.0 is required"),
            ]
        );

        let no_rustup = Facts {
            rustup_cargo_path: None,
            rustup_cargo_version: None,
            ..healthy()
        };
        // Only the root cause is reported, not every check that depends on rustup.
        assert_eq!(
            failing(&evaluate(&no_rustup, &pinned())),
            [("rustup", true, "install rustup from https://rustup.rs")]
        );
    }

    #[test]
    fn optional_tools_never_fail_the_run() {
        let facts = Facts {
            docker: false,
            nightly: false,
            cargo_fuzz: false,
            ..healthy()
        };
        let checks = evaluate(&facts, &pinned());
        let failures = failing(&checks);
        assert_eq!(failures.len(), 3);
        assert!(failures.iter().all(|(_, required, _)| !required));
    }

    #[test]
    fn the_pinned_toolchain_is_read_from_rust_toolchain_toml() {
        let text = "[toolchain]\nchannel = \"1.99.0\"\ncomponents = [\"rustfmt\", \"clippy\"]\n";
        assert_eq!(parse_pinned(text), Some(pinned()));
        assert_eq!(parse_pinned("[other]\n"), None);
    }
}
