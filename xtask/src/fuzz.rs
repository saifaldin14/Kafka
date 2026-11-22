//! `cargo xtask fuzz`: builds a fuzz target with cargo-fuzz and runs it with the
//! standard libFuzzer limits (fuzz/README.md).
//!
//! The target is built by `cargo fuzz build` on the nightly pinned in
//! `fuzz/rust-toolchain.toml`, then its binary is run directly. cargo-fuzz's own `run`
//! would start a second run of a crashing input without these limits, so a hang or an
//! allocation crash could stall it; running the binary keeps every run within
//! [`LIMITS`].

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::Exit;
use crate::cli::FuzzCommand;
use crate::output;
use crate::tools::{self, CARGO_FUZZ};

/// One libFuzzer resource limit, passed as `-<flag>=<value>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Limit {
    pub(crate) flag: &'static str,
    pub(crate) value: u32,
}

/// The limits every run uses. A run that exceeds one is reported as a crash, so a hang
/// or a runaway allocation fails like a panic. Why each value was chosen is in
/// fuzz/README.md, whose table a test keeps equal to this one.
pub(crate) const LIMITS: &[Limit] = &[
    Limit {
        flag: "max_len",
        value: 65_536,
    },
    Limit {
        flag: "timeout",
        value: 10,
    },
    Limit {
        flag: "malloc_limit_mb",
        value: 64,
    },
    Limit {
        flag: "rss_limit_mb",
        value: 2048,
    },
];

/// The fuzz workspace, relative to the repository root.
const FUZZ_DIR: &str = "fuzz";

pub(crate) fn run(command: &FuzzCommand, out: &mut impl Write) -> Exit {
    let root = crate::repo_root();
    let targets = match targets(&root) {
        Ok(targets) => targets,
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            return Exit::CheckFailed;
        }
    };
    let (target, input) = match command {
        FuzzCommand::List => {
            for target in &targets {
                output::line(out, target);
            }
            return Exit::Success;
        }
        FuzzCommand::Run { target, seconds } => (target, Input::Fuzz { seconds: *seconds }),
        FuzzCommand::Reproduce { target, input } => (target, Input::One(PathBuf::from(input))),
    };
    if !targets.contains(target) {
        output::line(
            out,
            &format!(
                "error: unknown fuzz target `{target}`; the targets are: {}",
                targets.join(", ")
            ),
        );
        return Exit::Usage;
    }
    match build(&root, target).and_then(|binary| run_binary(&root, &binary, target, &input)) {
        Ok(true) => Exit::Success,
        Ok(false) => {
            let message = match input {
                Input::Fuzz { .. } => format!(
                    "fuzz: {target} found a crash; the input is in {FUZZ_DIR}/artifacts/{target}/. Reproduce it with `cargo xtask fuzz {target} <input>`"
                ),
                Input::One(path) => {
                    format!("fuzz: {} fails {target}", path.display())
                }
            };
            output::line(out, &message);
            Exit::CheckFailed
        }
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            Exit::CheckFailed
        }
    }
}

/// What a run feeds the target.
enum Input {
    /// Generated inputs, for this many seconds, growing the corpus.
    Fuzz { seconds: u64 },
    /// One input file, once.
    One(PathBuf),
}

/// The target names: the `[[bin]]` entries of `fuzz/Cargo.toml`, sorted.
fn targets(root: &Path) -> Result<Vec<String>, String> {
    let path = root.join(FUZZ_DIR).join("Cargo.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let manifest: Value = toml::from_str(&text)
        .map_err(|error| format!("could not parse {}: {error}", path.display()))?;
    let mut targets: Vec<String> = manifest
        .get("bin")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|bin| bin.get("name").and_then(Value::as_str).map(str::to_owned))
        .collect();
    targets.sort();
    Ok(targets)
}

/// The nightly channel `fuzz/rust-toolchain.toml` pins.
pub(crate) fn nightly_channel(root: &Path) -> Result<String, String> {
    let path = root.join(FUZZ_DIR).join("rust-toolchain.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    crate::doctor::parse_pinned(&text)
        .map(|pinned| pinned.channel)
        .ok_or_else(|| format!("{} has no [toolchain] channel", path.display()))
}

/// Why the pinned nightly can't be used, with the command that fixes it.
pub(crate) fn nightly_problem(channel: &str) -> Option<String> {
    let installed = Command::new("rustup")
        .args(["toolchain", "list"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .is_some_and(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line == channel || line.starts_with(&format!("{channel}-")))
        });
    (!installed).then(|| {
        format!(
            "the {channel} toolchain fuzzing needs is not installed; run `rustup toolchain install {channel} --profile minimal`"
        )
    })
}

/// Builds `target` with debug assertions, so that overflow is a crash, and returns its
/// binary.
fn build(root: &Path, target: &str) -> Result<PathBuf, String> {
    let channel = nightly_channel(root)?;
    if let Some(problem) = nightly_problem(&channel) {
        return Err(problem);
    }
    tools::check(&CARGO_FUZZ).map_err(|problem| problem.to_string())?;
    let host = Command::new("rustup")
        .args(["run", &channel, "rustc", "-vV"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .find_map(|line| line.strip_prefix("host: ").map(str::to_owned))
        })
        .ok_or_else(|| format!("could not read the host target from `rustc -vV` on {channel}"))?;
    let status = Command::new("rustup")
        .args(["run", &channel, "cargo", "fuzz", "build"])
        .args([
            "--fuzz-dir",
            FUZZ_DIR,
            "--release",
            "--debug-assertions",
            target,
        ])
        .current_dir(root)
        .status()
        .map_err(|error| format!("could not run cargo fuzz build: {error}"))?;
    if !status.success() {
        return Err(format!("cargo fuzz build {target} failed"));
    }
    Ok(root
        .join(FUZZ_DIR)
        .join("target")
        .join(host)
        .join("release")
        .join(target))
}

/// Runs the target's binary on `input` with [`LIMITS`]. Returns whether it ran clean.
fn run_binary(root: &Path, binary: &Path, target: &str, input: &Input) -> Result<bool, String> {
    let fuzz_dir = root.join(FUZZ_DIR);
    let artifacts = fuzz_dir.join("artifacts").join(target);
    create_dir(&artifacts)?;
    let mut command = Command::new(binary);
    command
        .args(limit_args())
        .arg(format!("-artifact_prefix={}/", artifacts.display()))
        .env("ASAN_OPTIONS", asan_options())
        .current_dir(root)
        .stdin(Stdio::null());
    match input {
        Input::Fuzz { seconds } => {
            // libFuzzer writes new inputs to the first directory only, so the
            // committed seeds are read but never added to.
            let corpus = fuzz_dir.join("corpus").join(target);
            create_dir(&corpus)?;
            command
                .arg(format!("-max_total_time={seconds}"))
                .arg(corpus);
            let seeds = fuzz_dir.join("seeds").join(target);
            if seeds.is_dir() {
                command.arg(seeds);
            }
        }
        Input::One(path) => {
            if !path.is_file() {
                return Err(format!("{} is not a file", path.display()));
            }
            command.arg(path);
        }
    }
    let status = command
        .status()
        .map_err(|error| format!("could not run {}: {error}", binary.display()))?;
    Ok(status.success())
}

/// [`LIMITS`] as libFuzzer flags.
pub(crate) fn limit_args() -> Vec<String> {
    LIMITS
        .iter()
        .map(|limit| format!("-{}={}", limit.flag, limit.value))
        .collect()
}

/// The address sanitizer's options, as cargo-fuzz sets them: the caller's, then its
/// default, which turns off a check that Rust's codegen trips falsely.
fn asan_options() -> String {
    match std::env::var("ASAN_OPTIONS") {
        Ok(options) if !options.is_empty() => format!("{options}:detect_odr_violation=0"),
        _ => "detect_odr_violation=0".to_owned(),
    }
}

fn create_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path)
        .map_err(|error| format!("could not create {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{LIMITS, targets};

    #[test]
    fn every_target_in_the_fuzz_manifest_is_listed_in_order() {
        let targets = targets(&crate::repo_root()).unwrap();
        assert!(
            targets.contains(&"types_uuid_parse".to_owned()),
            "{targets:?}"
        );
        assert!(targets.is_sorted(), "{targets:?}");
    }

    #[test]
    fn the_readme_documents_every_limit_with_its_value() {
        let readme = std::fs::read_to_string(crate::repo_root().join("fuzz/README.md")).unwrap();
        for limit in LIMITS {
            let row = format!("| `-{}` | {} |", limit.flag, limit.value);
            assert!(readme.contains(&row), "fuzz/README.md has no row `{row}`");
        }
        let documented = readme
            .lines()
            .filter(|line| line.starts_with("| `-"))
            .count();
        assert_eq!(
            documented,
            LIMITS.len(),
            "fuzz/README.md lists other limits"
        );
    }
}
