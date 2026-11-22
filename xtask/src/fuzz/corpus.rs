//! `cargo xtask fuzz corpus pull` and `minimize`: the corpora CI grows, stored as
//! Actions artifacts ([ADR-0018](../../../docs/adr/0018-fuzz-corpus-storage.md)).

use std::path::Path;
use std::process::Command;

use serde_json::Value;

use super::{FUZZ_DIR, build, create_dir, libfuzzer};

/// The largest a corpus may be after minimization (ADR-0018).
const BUDGET_BYTES: u64 = 50 * 1024 * 1024;

/// The artifact holding a target's corpus.
fn artifact_name(target: &str) -> String {
    format!("fuzz-corpus-{target}")
}

/// Downloads the newest corpus for `target` that a run on `main` uploaded into
/// `fuzz/corpus/<target>/`, alongside any inputs already there.
pub(crate) fn pull(root: &Path, target: &str) -> Result<String, String> {
    let name = artifact_name(target);
    let listing = gh(
        root,
        &[
            "api",
            "--method",
            "GET",
            "repos/{owner}/{repo}/actions/artifacts",
            "-f",
            &format!("name={name}"),
            "-f",
            "per_page=100",
        ],
    )?;
    let listing: Value = serde_json::from_str(&listing)
        .map_err(|error| format!("could not read the artifact list for {name}: {error}"))?;
    let Some(run_id) = newest_trusted_run(&listing) else {
        return Ok(format!(
            "{target}: no corpus has been uploaded from main yet; runs start from the seeds"
        ));
    };
    let corpus = root.join(FUZZ_DIR).join("corpus").join(target);
    let download = root
        .join(FUZZ_DIR)
        .join("corpus")
        .join(format!(".{target}.download"));
    remove_dir(&download)?;
    gh(
        root,
        &[
            "run",
            "download",
            &run_id.to_string(),
            "--name",
            &name,
            "--dir",
            &download.display().to_string(),
        ],
    )?;
    create_dir(&corpus)?;
    // Corpus files are named after their content, so a name already present holds the
    // same input.
    let mut added = 0_usize;
    for entry in read_dir(&download)? {
        let destination = corpus.join(entry.file_name());
        if !destination.exists() {
            std::fs::rename(entry.path(), &destination)
                .map_err(|error| format!("could not move {}: {error}", entry.path().display()))?;
            added = added.saturating_add(1);
        }
    }
    remove_dir(&download)?;
    Ok(format!(
        "{target}: {added} new input(s) from run {run_id} in {}",
        corpus.display()
    ))
}

/// The run that uploaded the newest usable artifact in a GitHub artifact list: not
/// expired, and from a run on `main` of this repository. A pull request from a fork
/// runs its own copy of the workflows, so an artifact it uploads under the same name is
/// never taken.
fn newest_trusted_run(listing: &Value) -> Option<u64> {
    listing
        .get("artifacts")?
        .as_array()?
        .iter()
        .filter(|artifact| artifact.get("expired").and_then(Value::as_bool) == Some(false))
        .filter_map(|artifact| {
            let run = artifact.get("workflow_run")?;
            let trusted = run.get("head_branch").and_then(Value::as_str) == Some("main")
                && run.get("head_repository_id").is_some()
                && run.get("head_repository_id") == run.get("repository_id");
            let created = artifact.get("created_at").and_then(Value::as_str)?;
            trusted.then_some((created, run.get("id").and_then(Value::as_u64)?))
        })
        .max_by_key(|(created, _)| *created)
        .map(|(_, run_id)| run_id)
}

/// Shrinks `target`'s corpus with libFuzzer's merge, keeping only inputs that add
/// coverage. The corpus is replaced only if the merge completed and lost no coverage;
/// then it must be within [`BUDGET_BYTES`].
pub(crate) fn minimize(root: &Path, target: &str) -> Result<String, String> {
    // Relative to the repository root, where libFuzzer runs (`libfuzzer`).
    let corpus = Path::new(FUZZ_DIR).join("corpus").join(target);
    let merged = Path::new(FUZZ_DIR)
        .join("corpus")
        .join(format!(".{target}.minimized"));
    if !root.join(&corpus).is_dir() {
        return Err(format!(
            "{} doesn't exist; pull or grow a corpus first",
            corpus.display()
        ));
    }
    let binary = build(root, target)?;
    let before = coverage(root, &binary, target, &corpus)?;
    remove_dir(&root.join(&merged))?;
    create_dir(&root.join(&merged))?;
    let merge = libfuzzer(root, &binary, target)?
        .arg("-merge=1")
        .arg(&merged)
        .arg(&corpus)
        .output()
        .map_err(|error| format!("could not run {}: {error}", binary.display()))?;
    let log = String::from_utf8_lossy(&merge.stderr);
    // cargo-fuzz's cmin trusts the exit status alone; libFuzzer prints this line only
    // once the merge has finished.
    if !merge.status.success() || !log.lines().any(|line| line.starts_with("MERGE-OUTER: ")) {
        return Err(format!(
            "the merge of {target}'s corpus did not complete; the corpus is unchanged:\n{log}"
        ));
    }
    let after = coverage(root, &binary, target, &merged)?;
    if after.edges < before.edges || after.features < before.features {
        return Err(format!(
            "the merge of {target}'s corpus lost coverage ({before} before, {after} after); the corpus is unchanged"
        ));
    }
    std::fs::remove_dir_all(root.join(&corpus))
        .and_then(|()| std::fs::rename(root.join(&merged), root.join(&corpus)))
        .map_err(|error| format!("could not replace {}: {error}", corpus.display()))?;
    let report = format!("{target}: minimized from {before} to {after}");
    if after.bytes > BUDGET_BYTES {
        return Err(format!(
            "{report}, over the {} MiB budget (ADR-0018)",
            BUDGET_BYTES / (1024 * 1024)
        ));
    }
    Ok(report)
}

/// What a corpus covers, as libFuzzer reports it when it loads the corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Coverage {
    /// Coverage edges (libFuzzer's `cov:`).
    edges: u64,
    /// Features, finer than edges (libFuzzer's `ft:`).
    features: u64,
    files: usize,
    bytes: u64,
}

impl std::fmt::Display for Coverage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} file(s), {} byte(s), cov {}, ft {}",
            self.files, self.bytes, self.edges, self.features
        )
    }
}

/// Loads `dir` into the target without fuzzing and reads the coverage it reports.
fn coverage(root: &Path, binary: &Path, target: &str, dir: &Path) -> Result<Coverage, String> {
    let run = libfuzzer(root, binary, target)?
        .arg("-runs=0")
        .arg(dir)
        .output()
        .map_err(|error| format!("could not run {}: {error}", binary.display()))?;
    let log = String::from_utf8_lossy(&run.stderr);
    let (edges, features) = run
        .status
        .success()
        .then(|| parse_inited(&log))
        .flatten()
        .ok_or_else(|| format!("could not read the coverage of {}:\n{log}", dir.display()))?;
    let mut files = 0_usize;
    let mut bytes = 0_u64;
    for entry in read_dir(&root.join(dir))? {
        files = files.saturating_add(1);
        bytes = bytes.saturating_add(entry.metadata().map_or(0, |metadata| metadata.len()));
    }
    Ok(Coverage {
        edges,
        features,
        files,
        bytes,
    })
}

/// The `cov:` and `ft:` counts of libFuzzer's `INITED` line.
fn parse_inited(log: &str) -> Option<(u64, u64)> {
    let line = log
        .lines()
        .find(|line| line.split_whitespace().any(|word| word == "INITED"))?;
    let count = |label: &str| {
        let mut words = line.split_whitespace();
        words.find(|word| *word == label)?;
        words.next()?.parse().ok()
    };
    Some((count("cov:")?, count("ft:")?))
}

/// Runs `gh` in the repository and returns its stdout.
fn gh(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("gh")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| {
            format!(
                "could not run the GitHub CLI ({error}); install it from https://cli.github.com and run `gh auth login`"
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "`gh {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn read_dir(dir: &Path) -> Result<Vec<std::fs::DirEntry>, String> {
    std::fs::read_dir(dir)
        .and_then(Iterator::collect)
        .map_err(|error| format!("could not read {}: {error}", dir.display()))
}

fn remove_dir(dir: &Path) -> Result<(), String> {
    match std::fs::remove_dir_all(dir) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(format!("could not remove {}: {error}", dir.display()))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{newest_trusted_run, parse_inited};

    fn artifact(
        created: &str,
        run: u64,
        branch: &str,
        head_repository: u64,
        expired: bool,
    ) -> Value {
        json!({
            "created_at": created,
            "expired": expired,
            "workflow_run": {
                "id": run,
                "head_branch": branch,
                "repository_id": 1,
                "head_repository_id": head_repository,
            }
        })
    }

    #[test]
    fn the_newest_unexpired_artifact_from_main_of_this_repository_is_chosen() {
        let listing = json!({ "artifacts": [
            artifact("2026-11-20T03:00:00Z", 10, "main", 1, false),
            // A fork's pull request, from its own branch named main: never taken.
            artifact("2026-11-22T03:00:00Z", 30, "main", 2, false),
            artifact("2026-11-21T03:00:00Z", 20, "main", 1, false),
            artifact("2026-11-21T09:00:00Z", 40, "feature", 1, false),
            artifact("2026-11-21T10:00:00Z", 50, "main", 1, true),
        ]});
        assert_eq!(newest_trusted_run(&listing), Some(20));
        assert_eq!(newest_trusted_run(&json!({ "artifacts": [] })), None);
    }

    #[test]
    fn coverage_is_read_from_libfuzzers_inited_line() {
        let log = "INFO: seed corpus: files: 3\n#3\tINITED cov: 151 ft: 355 corp: 3/27b exec/s: 0 rss: 30Mb\n";
        assert_eq!(parse_inited(log), Some((151, 355)));
        assert_eq!(parse_inited("#1\tDONE cov: 1 ft: 1\n"), None);
    }
}
