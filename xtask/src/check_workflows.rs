//! `cargo xtask check-workflows`: the GitHub Actions files follow the CI security rules
//! (docs/standards/security.md#dependencies).
//!
//! Rules, for `.github/workflows/*.yml` and the local actions in
//! `.github/actions/*/action.yml`:
//!
//! 1. Every `uses:` is a local path (`./…`), a Docker image pinned by digest
//!    (`docker://…@sha256:<64 hex>`), or `owner/repo[/path]@<40 hex commit SHA>`
//!    followed by a `# v…` comment naming the version. A tag or branch can be moved to
//!    other code after review; a commit SHA can't.
//! 2. No workflow uses the `pull_request_target` trigger, which runs with write
//!    permissions and secrets on code from forks.
//! 3. Every job in a workflow sets `timeout-minutes` and `permissions`.
//!
//! The files are read line by line rather than parsed as YAML, so every problem has an
//! exact `file:line`. That relies on the style the workflows use: block YAML, indented
//! two spaces per level, with jobs at two spaces under a top-level `jobs:`. A workflow in
//! another style fails with "no jobs found" rather than passing unchecked.
//!
//! Then the pinned actionlint checks the workflows' full syntax: expressions, contexts,
//! runner labels, action inputs, and, through the pinned shellcheck, every `run:` script.
//! actionlint skips shell checks silently when shellcheck is missing, so both tools are
//! checked first.

use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::Exit;
use crate::output;
use crate::tools::{self, ACTIONLINT, SHELLCHECK};

/// Runs the command on the repository's workflow and action files.
pub(crate) fn run(out: &mut impl Write) -> Exit {
    let problems = match check_repository(&crate::repo_root()) {
        Ok(problems) => problems,
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            return Exit::CheckFailed;
        }
    };
    let mut problems = problems;
    problems.extend(actionlint(&crate::repo_root()));
    if problems.is_empty() {
        output::line(out, "check-workflows: ok");
        return Exit::Success;
    }
    for problem in &problems {
        output::line(out, problem);
    }
    output::line(
        out,
        &format!(
            "check-workflows: {} problem(s) (docs/standards/security.md#dependencies)",
            problems.len()
        ),
    );
    Exit::CheckFailed
}

/// Runs the pinned actionlint, with the pinned shellcheck, and returns its findings.
fn actionlint(root: &Path) -> Vec<String> {
    let missing: Vec<String> = [&ACTIONLINT, &SHELLCHECK]
        .into_iter()
        .filter_map(|tool| tools::check(tool).err())
        .map(|problem| problem.to_string())
        .collect();
    if !missing.is_empty() {
        return missing;
    }
    match Command::new(ACTIONLINT.name)
        .args(["-no-color", "-oneline", "-shellcheck", SHELLCHECK.name])
        .current_dir(root)
        .output()
    {
        Err(error) => vec![format!("could not run actionlint: {error}")],
        Ok(output) if output.status.success() => Vec::new(),
        Ok(output) => {
            let findings = String::from_utf8_lossy(&output.stdout);
            let errors = String::from_utf8_lossy(&output.stderr);
            findings
                .lines()
                .chain(errors.lines())
                .filter(|line| !line.trim().is_empty())
                .map(|line| format!("actionlint: {line}"))
                .collect()
        }
    }
}

fn check_repository(root: &Path) -> Result<Vec<String>, String> {
    let mut problems = Vec::new();
    let workflows = yaml_files(&root.join(".github/workflows"), ".github/workflows")?;
    if workflows.is_empty() {
        return Err("no workflow files in .github/workflows".to_owned());
    }
    for file in &workflows {
        let text = read(root, file)?;
        problems.extend(check_workflow(file, &text));
    }
    let actions_dir = root.join(".github/actions");
    if actions_dir.is_dir() {
        let entries = std::fs::read_dir(&actions_dir)
            .map_err(|error| format!("could not read .github/actions: {error}"))?;
        let mut actions = Vec::new();
        for entry in entries {
            let entry =
                entry.map_err(|error| format!("could not read .github/actions: {error}"))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            for file_name in ["action.yml", "action.yaml"] {
                if entry.path().join(file_name).is_file() {
                    actions.push(format!(".github/actions/{name}/{file_name}"));
                }
            }
        }
        actions.sort();
        for file in &actions {
            let text = read(root, file)?;
            problems.extend(check_uses(file, &text));
        }
    }
    Ok(problems)
}

fn read(root: &Path, file: &str) -> Result<String, String> {
    std::fs::read_to_string(root.join(file))
        .map_err(|error| format!("could not read {file}: {error}"))
}

/// The `.yml` and `.yaml` files directly in `dir`, as `relative/<name>`, sorted.
fn yaml_files(dir: &Path, relative: &str) -> Result<Vec<String>, String> {
    let entries =
        std::fs::read_dir(dir).map_err(|error| format!("could not read {relative}: {error}"))?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("could not read {relative}: {error}"))?;
        let path = entry.path();
        let is_yaml = path
            .extension()
            .is_some_and(|extension| extension == "yml" || extension == "yaml");
        if is_yaml && path.is_file() {
            files.push(format!(
                "{relative}/{}",
                entry.file_name().to_string_lossy()
            ));
        }
    }
    files.sort();
    Ok(files)
}

/// Every rule for one workflow file.
pub(crate) fn check_workflow(file: &str, text: &str) -> Vec<String> {
    let mut problems = check_uses(file, text);
    for (number, line) in numbered(text) {
        if without_comment(line).contains("pull_request_target") {
            problems.push(format!(
                "{file}:{number}: the `pull_request_target` trigger runs fork code with write \
                 permissions and secrets; use `pull_request`"
            ));
        }
    }
    problems.extend(check_jobs(file, text));
    problems
}

/// Rule 1, for a workflow or an action file.
pub(crate) fn check_uses(file: &str, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for (number, line) in numbered(text) {
        let item = line.trim_start();
        let item = item.strip_prefix("- ").unwrap_or(item).trim_start();
        let Some(rest) = item.strip_prefix("uses:") else {
            continue;
        };
        let (value, comment) = match rest.split_once('#') {
            Some((value, comment)) => (value.trim(), Some(comment.trim())),
            None => (rest.trim(), None),
        };
        let value = value.trim_matches(|c| c == '"' || c == '\'');
        if let Some(problem) = uses_problem(value, comment) {
            problems.push(format!("{file}:{number}: `uses: {value}` {problem}"));
        }
    }
    problems
}

fn uses_problem(value: &str, comment: Option<&str>) -> Option<String> {
    if value.starts_with("./") {
        return None;
    }
    if let Some(image) = value.strip_prefix("docker://") {
        let pinned = image
            .rsplit_once("@sha256:")
            .is_some_and(|(_, digest)| is_hex(digest, 64));
        return (!pinned).then(|| "must pin the image by digest (`@sha256:<64 hex>`)".to_owned());
    }
    let Some((_, reference)) = value.rsplit_once('@') else {
        return Some("has no `@<commit SHA>`".to_owned());
    };
    if !is_hex(reference, 40) {
        return Some(format!(
            "must be pinned to a full 40-character commit SHA, not `{reference}`"
        ));
    }
    let names_version = comment.is_some_and(|comment| {
        comment
            .strip_prefix('v')
            .is_some_and(|version| version.starts_with(|c: char| c.is_ascii_digit()))
    });
    (!names_version).then(|| "needs a `# vX.Y.Z` comment naming the pinned version".to_owned())
}

fn is_hex(text: &str, length: usize) -> bool {
    text.len() == length
        && text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Rule 3: each job under the top-level `jobs:` sets `timeout-minutes` and
/// `permissions` at its own level.
fn check_jobs(file: &str, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let mut in_jobs = false;
    let mut jobs: Vec<Job> = Vec::new();
    for (number, line) in numbered(text) {
        let content = without_comment(line);
        if content.trim().is_empty() {
            continue;
        }
        let indent = content.len() - content.trim_start().len();
        if indent == 0 {
            in_jobs = content.trim_end() == "jobs:";
            continue;
        }
        if !in_jobs {
            continue;
        }
        let key = content.trim().strip_suffix(':').filter(|key| is_key(key));
        match (indent, key) {
            (2, Some(name)) => jobs.push(Job {
                name: name.to_owned(),
                line: number,
                timeout: false,
                permissions: false,
            }),
            (4, _) => {
                if let Some(job) = jobs.last_mut() {
                    let key = content.trim();
                    job.timeout |= key.starts_with("timeout-minutes:");
                    job.permissions |= key.starts_with("permissions:");
                }
            }
            _ => {}
        }
    }
    if jobs.is_empty() {
        problems.push(format!(
            "{file}: no jobs found; jobs must be two-space-indented keys under a top-level `jobs:`"
        ));
    }
    for job in jobs {
        for (present, key) in [
            (job.timeout, "timeout-minutes"),
            (job.permissions, "permissions"),
        ] {
            if !present {
                problems.push(format!(
                    "{file}:{}: job `{}` must set `{key}`",
                    job.line, job.name
                ));
            }
        }
    }
    problems
}

#[derive(Debug)]
struct Job {
    name: String,
    line: usize,
    timeout: bool,
    permissions: bool,
}

/// A YAML key: letters, digits, `_` and `-`.
fn is_key(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn numbered(text: &str) -> impl Iterator<Item = (usize, &str)> {
    (1..).zip(text.lines())
}

/// The line up to a `#` that starts a comment (at the start or after whitespace), so
/// `#` inside values such as URLs is kept.
fn without_comment(line: &str) -> &str {
    let mut previous = ' ';
    for (index, c) in line.char_indices() {
        if c == '#' && previous.is_whitespace() {
            return line.get(..index).unwrap_or(line);
        }
        previous = c;
    }
    line
}

#[cfg(test)]
mod tests {
    use super::{check_uses, check_workflow};

    const SHA: &str = "3d3c42e5aac5ba805825da76410c181273ba90b1";

    fn workflow(job: &str) -> String {
        format!(
            "name: CI\non:\n  pull_request:\npermissions: {{}}\njobs:\n{job}  other:\n    runs-on: ubuntu-24.04\n    timeout-minutes: 5\n    permissions: {{}}\n    steps:\n      - run: true\n"
        )
    }

    fn good_job() -> String {
        format!(
            "  build:\n    runs-on: ubuntu-24.04\n    timeout-minutes: 10\n    permissions:\n      contents: read\n    steps:\n      - uses: actions/checkout@{SHA} # v7.0.1\n        with:\n          persist-credentials: false\n      - uses: ./.github/actions/xtask-ci\n"
        )
    }

    #[test]
    fn a_workflow_following_the_rules_passes() {
        assert_eq!(
            check_workflow("ci.yml", &workflow(&good_job())),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_tag_pinned_action_is_rejected_with_file_and_line() {
        let job = good_job().replace(&format!("@{SHA} # v7.0.1"), "@v7");
        assert_eq!(
            check_workflow("ci.yml", &workflow(&job)),
            [
                "ci.yml:12: `uses: actions/checkout@v7` must be pinned to a full 40-character commit SHA, not `v7`"
            ]
        );
    }

    #[test]
    fn a_sha_without_a_version_comment_is_rejected() {
        let job = good_job().replace(" # v7.0.1", "");
        assert_eq!(
            check_workflow("ci.yml", &workflow(&job)),
            [format!(
                "ci.yml:12: `uses: actions/checkout@{SHA}` needs a `# vX.Y.Z` comment naming the pinned version"
            )]
        );
    }

    #[test]
    fn a_missing_timeout_or_permissions_is_rejected_at_the_job() {
        let no_timeout = good_job().replace("    timeout-minutes: 10\n", "");
        assert_eq!(
            check_workflow("ci.yml", &workflow(&no_timeout)),
            ["ci.yml:6: job `build` must set `timeout-minutes`"]
        );
        let no_permissions = good_job().replace("    permissions:\n      contents: read\n", "");
        assert_eq!(
            check_workflow("ci.yml", &workflow(&no_permissions)),
            ["ci.yml:6: job `build` must set `permissions`"]
        );
    }

    #[test]
    fn nested_keys_do_not_count_for_the_job() {
        // `timeout-minutes` on a step is not the job's timeout.
        let job = good_job().replace("    timeout-minutes: 10\n", "").replace(
            "      - uses: ./.github/actions/xtask-ci\n",
            "      - uses: ./.github/actions/xtask-ci\n        timeout-minutes: 10\n",
        );
        assert_eq!(
            check_workflow("ci.yml", &workflow(&job)),
            ["ci.yml:6: job `build` must set `timeout-minutes`"]
        );
    }

    #[test]
    fn pull_request_target_is_rejected() {
        let text = workflow(&good_job()).replace("  pull_request:\n", "  pull_request_target:\n");
        assert_eq!(
            check_workflow("ci.yml", &text),
            [
                "ci.yml:3: the `pull_request_target` trigger runs fork code with write permissions and secrets; use `pull_request`"
            ]
        );
        let commented =
            workflow(&good_job()).replace("name: CI\n", "name: CI # never pull_request_target\n");
        assert_eq!(check_workflow("ci.yml", &commented), Vec::<String>::new());
    }

    #[test]
    fn a_workflow_in_another_style_fails_instead_of_passing() {
        let text = "on: push\njobs:\n    build:\n        runs-on: ubuntu-24.04\n";
        assert_eq!(
            check_workflow("ci.yml", text),
            [
                "ci.yml: no jobs found; jobs must be two-space-indented keys under a top-level `jobs:`"
            ]
        );
    }

    #[test]
    fn actions_and_docker_images_follow_the_pinning_rule() {
        let action = format!(
            "runs:\n  using: composite\n  steps:\n    - uses: Swatinem/rust-cache@{SHA} # v2.9.3\n    - uses: \"owner/repo/path@main\"\n    - uses: docker://alpine:3.20\n    - uses: docker://alpine@sha256:{}\n",
            "a".repeat(64)
        );
        assert_eq!(
            check_uses("action.yml", &action),
            [
                "action.yml:5: `uses: owner/repo/path@main` must be pinned to a full 40-character commit SHA, not `main`",
                "action.yml:6: `uses: docker://alpine:3.20` must pin the image by digest (`@sha256:<64 hex>`)",
            ]
        );
    }
}
