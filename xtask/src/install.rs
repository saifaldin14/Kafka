//! `cargo xtask tools install [--step <name>]... [--tool <name>]...`: installs the pinned
//! external tools that are missing or at the wrong version, using the step runner.
//!
//! With no arguments, it installs every tool a CI step needs. With `--step`, only the
//! tools those CI steps require are installed, so a CI job installs what its steps need
//! and nothing more. The step table in `ci` is the single record of which step needs
//! which tool. `--tool` names a tool directly, such as cargo-fuzz, which no step needs.

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use crate::Exit;
use crate::ci;
use crate::output;
use crate::steps::{self, Step};
use crate::tools::{self, Source, TOOLS, Tool};

pub(crate) fn run(step_names: &[String], tool_names: &[String], out: &mut impl Write) -> Exit {
    let wanted = wanted_tools(step_names, tool_names);
    if wanted.is_empty() {
        output::line(out, "The selected steps need no external tools.");
        return Exit::Success;
    }
    let (crates, releases): (Vec<&Tool>, Vec<&Tool>) = wanted
        .into_iter()
        .filter(|tool| tools::check(tool).is_err())
        .partition(|tool| tool.source == Source::Crate);
    if crates.is_empty() && releases.is_empty() {
        output::line(out, "All tools are installed at their pinned versions.");
        return Exit::Success;
    }
    let installs: Vec<Step> = crates
        .into_iter()
        .map(|tool| {
            Step::cargo(
                &format!("install {}", tool.name),
                &["install", "--locked", "--version", tool.version, tool.name],
            )
        })
        .collect();
    let mut ok = installs.is_empty() || steps::all_passed(&steps::run(&installs, false, out));
    for tool in releases {
        match install_release(tool) {
            Ok(path) => output::line(
                out,
                &format!(
                    "installed {} {} to {}",
                    tool.name,
                    tool.version,
                    path.display()
                ),
            ),
            Err(error) => {
                output::line(
                    out,
                    &format!("error: {} {}: {error}", tool.name, tool.version),
                );
                ok = false;
            }
        }
    }
    if ok { Exit::Success } else { Exit::CheckFailed }
}

/// Downloads this platform's release archive, checks its SHA-256, and copies the binary
/// into cargo's bin directory, which rustup puts on `PATH`.
fn install_release(tool: &Tool) -> Result<PathBuf, String> {
    let Source::Release {
        path_in_archive, ..
    } = tool.source
    else {
        return Err("not a release tool".to_owned());
    };
    let (os, arch) = (std::env::consts::OS, std::env::consts::ARCH);
    let asset = tool.release_asset(os, arch).ok_or_else(|| {
        format!("no pinned release for {os}/{arch}; install it yourself and check its version")
    })?;
    let work = crate::repo_root()
        .join("target")
        .join("tools")
        .join(tool.name);
    std::fs::create_dir_all(&work)
        .map_err(|error| format!("could not create {}: {error}", work.display()))?;
    let archive = work.join(format!("{}-{}.tar.gz", tool.name, tool.version));
    run_quietly(
        Command::new("curl")
            .args([
                "--silent",
                "--show-error",
                "--fail",
                "--location",
                "--retry",
                "3",
            ])
            .arg("--output")
            .arg(&archive)
            .arg(asset.url),
    )?;
    let digest = run_quietly(Command::new("shasum").args(["-a", "256"]).arg(&archive))?;
    let actual = sha256_of_shasum_output(&digest);
    if actual != Some(asset.sha256) {
        return Err(format!(
            "{} has SHA-256 {}, not the pinned {}; refusing to install it",
            asset.url,
            actual.unwrap_or("(unreadable)"),
            asset.sha256
        ));
    }
    run_quietly(
        Command::new("tar")
            .arg("-xzf")
            .arg(&archive)
            .arg("-C")
            .arg(&work)
            .arg(path_in_archive),
    )?;
    let bin = cargo_bin_dir()?;
    std::fs::create_dir_all(&bin)
        .map_err(|error| format!("could not create {}: {error}", bin.display()))?;
    let target = bin.join(tool.name);
    std::fs::copy(work.join(path_in_archive), &target)
        .map_err(|error| format!("could not copy to {}: {error}", target.display()))?;
    make_executable(&target)?;
    Ok(target)
}

/// The first word of `shasum -a 256 <file>` output, if it is a SHA-256.
fn sha256_of_shasum_output(output: &str) -> Option<&str> {
    output
        .split_whitespace()
        .next()
        .filter(|digest| digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// `$CARGO_HOME/bin`, or `~/.cargo/bin` when `CARGO_HOME` is unset.
fn cargo_bin_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
        .ok_or("neither CARGO_HOME nor HOME is set")?;
    Ok(home.join("bin"))
}

/// Runs a command and returns its stdout, or an error with its stderr.
fn run_quietly(command: &mut Command) -> Result<String, String> {
    let program = command.get_program().to_string_lossy().into_owned();
    let output = command
        .output()
        .map_err(|error| format!("could not run {program}: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .map_err(|error| format!("could not make {} executable: {error}", path.display()))
}

#[cfg(not(unix))]
fn make_executable(_path: &std::path::Path) -> Result<(), String> {
    Ok(())
}

/// Every tool when no step is named; otherwise the tools the named steps require, in
/// `TOOLS` order and without duplicates.
fn wanted_tools(step_names: &[String], tool_names: &[String]) -> Vec<&'static Tool> {
    if step_names.is_empty() && tool_names.is_empty() {
        return tools::required_by_steps().collect();
    }
    let required: Vec<&Tool> = ci::STEPS
        .iter()
        .filter(|step| step_names.iter().any(|name| name == step.name))
        .flat_map(|step| step.requires.iter().copied())
        .collect();
    TOOLS
        .iter()
        .filter(|tool| {
            required.iter().any(|needed| needed.name == tool.name)
                || tool_names.iter().any(|name| name == tool.name)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{sha256_of_shasum_output, wanted_tools};

    #[test]
    fn reads_the_digest_from_shasum_output() {
        let digest = "8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8";
        assert_eq!(
            sha256_of_shasum_output(&format!("{digest}  /tmp/actionlint.tar.gz\n")),
            Some(digest)
        );
        assert_eq!(sha256_of_shasum_output("shasum: no such file\n"), None);
        assert_eq!(sha256_of_shasum_output(""), None);
    }

    fn names(steps: &[&str]) -> Vec<&'static str> {
        names_with_tools(steps, &[])
    }

    fn names_with_tools(steps: &[&str], tools: &[&str]) -> Vec<&'static str> {
        let owned = |names: &[&str]| -> Vec<String> {
            names.iter().map(|name| (*name).to_owned()).collect()
        };
        wanted_tools(&owned(steps), &owned(tools))
            .into_iter()
            .map(|tool| tool.name)
            .collect()
    }

    #[test]
    fn no_arguments_means_every_tool_a_step_needs() {
        assert_eq!(
            names(&[]),
            [
                "cargo-nextest",
                "cargo-deny",
                "lychee",
                "actionlint",
                "shellcheck"
            ]
        );
    }

    #[test]
    fn named_steps_get_only_their_tools_once() {
        assert_eq!(names(&["test"]), ["cargo-nextest", "cargo-deny"]);
        assert_eq!(
            names(&["links", "deny", "advisories"]),
            ["cargo-deny", "lychee"]
        );
        assert_eq!(names(&["fmt", "clippy"]), Vec::<&str>::new());
    }

    #[test]
    fn a_tool_no_step_needs_is_installed_only_by_name() {
        assert_eq!(names_with_tools(&[], &["cargo-fuzz"]), ["cargo-fuzz"]);
        assert_eq!(
            names_with_tools(&["test"], &["cargo-fuzz"]),
            ["cargo-nextest", "cargo-deny", "cargo-fuzz"]
        );
    }
}
