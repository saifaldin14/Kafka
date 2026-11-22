//! `cargo xtask schemas verify`: the vendored Kafka schemas are exactly the files their
//! manifest lists, unmodified ([ADR-0008](../../docs/adr/0008-protocol-code-generated-from-kafka-schemas.md)).
//! `cargo xtask schemas vendor --tag <tag> [--dry-run]`: copies them from an
//! apache/kafka tag and writes the manifest.
//!
//! Each `schemas/kafka-<tag>/` directory holds Kafka files copied from that tag and a
//! `MANIFEST.sha256` naming the tag, its commit, and every copied file with its SHA-256.
//! The check fails, naming the file, when a file is modified, added or removed. Only
//! the directory's own `README.md` and the manifest are not Kafka's, so they are not
//! listed.
//!
//! Hashes come from one `shasum -a 256` run per directory, the tool `tools install`
//! already relies on, so no hashing crate is needed. Then every `.json` file is loaded
//! as a schema (`message_spec`), so a file the code generator couldn't read fails here.
//!
//! `vendor` sparse-clones the tag into `target/`, copies [`SOURCES`] and Kafka's
//! `LICENSE` and `NOTICE` into a staging directory, and compares it with the vendored
//! one. `--dry-run` stops there and prints what would change; otherwise it replaces the
//! directory's files, keeping its README.md, and writes the manifest. It needs the
//! network, so it never runs in CI.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::Exit;
use crate::message_spec;
use crate::output;

/// Where the vendored directories live, relative to the repository root.
pub(crate) const SCHEMAS_DIR: &str = "schemas";

/// The manifest's name inside each vendored directory.
pub(crate) const MANIFEST: &str = "MANIFEST.sha256";

/// Alluvium's description of a vendored directory, the other file that isn't Kafka's.
const README: &str = "README.md";

/// The Kafka directories vendored: where each goes inside `schemas/kafka-<tag>/`, and
/// its path in apache/kafka. Streams' `SubscriptionInfoData.json` is left out on
/// purpose; the vendored README says why.
pub(crate) const SOURCES: &[(&str, &str)] = &[
    ("clients", "clients/src/main/resources/common/message"),
    ("metadata", "metadata/src/main/resources/common/metadata"),
    (
        "group-coordinator",
        "group-coordinator/src/main/resources/common/message",
    ),
    (
        "transaction-coordinator",
        "transaction-coordinator/src/main/resources/common/message",
    ),
    (
        "share-coordinator",
        "share-coordinator/src/main/resources/common/message",
    ),
    ("raft", "raft/src/main/resources/common/message"),
];

/// Kafka's files copied to the top of each vendored directory.
const KAFKA_FILES: [&str; 2] = ["LICENSE", "NOTICE"];

/// The repository `vendor` clones.
const KAFKA_REPOSITORY: &str = "https://github.com/apache/kafka.git";

/// A vendored directory's manifest: which Kafka tag and commit the files came from, and
/// the SHA-256 of each, by path relative to the directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Manifest {
    pub(crate) tag: String,
    pub(crate) commit: String,
    pub(crate) hashes: BTreeMap<String, String>,
}

/// How a directory's files differ from a manifest.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Change {
    /// In the directory, not in the manifest.
    Added(String),
    /// In the manifest, not in the directory.
    Removed(String),
    /// In both, with a different SHA-256.
    Modified(String),
}

impl Manifest {
    /// Parses a manifest: a `# Apache Kafka <tag>, commit <sha>.` line, other `#`
    /// comment lines, then `<sha256>  <path>` lines.
    pub(crate) fn parse(text: &str) -> Result<Manifest, String> {
        let mut source = None;
        let mut hashes = BTreeMap::new();
        for (index, line) in text.lines().enumerate() {
            let number = index + 1;
            if let Some(comment) = line.strip_prefix('#') {
                if let Some(rest) = comment.trim().strip_prefix("Apache Kafka ") {
                    source = Some(parse_source(rest).ok_or_else(|| {
                        format!("line {number}: expected `# Apache Kafka <tag>, commit <40 hex>.`")
                    })?);
                }
                continue;
            }
            let (hash, path) = line
                .split_once("  ")
                .filter(|(hash, path)| is_hex(hash, 64) && !path.is_empty())
                .ok_or_else(|| format!("line {number}: expected `<sha256>  <path>`"))?;
            if hashes.insert(path.to_owned(), hash.to_owned()).is_some() {
                return Err(format!("line {number}: {path} is listed twice"));
            }
        }
        let (tag, commit) =
            source.ok_or("no `# Apache Kafka <tag>, commit <sha>.` line".to_owned())?;
        Ok(Manifest {
            tag,
            commit,
            hashes,
        })
    }
}

impl Manifest {
    /// The manifest's text, as `parse` reads it, with paths in sorted order.
    pub(crate) fn render(&self) -> String {
        let lines: Vec<String> = self
            .hashes
            .iter()
            .map(|(path, hash)| format!("{hash}  {path}\n"))
            .collect();
        let lines = lines.concat();
        format!(
            "# Apache Kafka {}, commit {}.\n# Written by `cargo xtask schemas vendor`; \
             checked by `cargo xtask schemas verify`.\n{lines}",
            self.tag, self.commit
        )
    }
}

/// `<tag>, commit <sha>.` into its tag and commit.
fn parse_source(text: &str) -> Option<(String, String)> {
    let (tag, commit) = text.strip_suffix('.')?.split_once(", commit ")?;
    (!tag.is_empty() && is_hex(commit, 40)).then(|| (tag.to_owned(), commit.to_owned()))
}

fn is_hex(text: &str, length: usize) -> bool {
    text.len() == length && text.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// The differences between `expected` and `actual` hashes, by path.
pub(crate) fn compare(
    expected: &BTreeMap<String, String>,
    actual: &BTreeMap<String, String>,
) -> Vec<Change> {
    let mut changes = Vec::new();
    for (path, hash) in actual {
        match expected.get(path) {
            None => changes.push(Change::Added(path.clone())),
            Some(listed) if listed != hash => changes.push(Change::Modified(path.clone())),
            Some(_) => {}
        }
    }
    for path in expected.keys().filter(|path| !actual.contains_key(*path)) {
        changes.push(Change::Removed(path.clone()));
    }
    changes.sort();
    changes
}

/// The SHA-256 of every file under `dir` except its `README.md` and manifest, by path
/// relative to `dir` with `/` separators.
pub(crate) fn hash_tree(dir: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut files = Vec::new();
    collect_files(dir, "", &mut files)
        .map_err(|error| format!("could not read {}: {error}", dir.display()))?;
    files.retain(|path| path != README && path != MANIFEST);
    files.sort();
    if files.is_empty() {
        return Ok(BTreeMap::new());
    }
    let output = Command::new("shasum")
        .args(["-a", "256", "--"])
        .args(&files)
        .current_dir(dir)
        .output()
        .map_err(|error| format!("could not run shasum: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "shasum failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let mut hashes = BTreeMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let (hash, path) = line
            .split_once("  ")
            .filter(|(hash, _)| is_hex(hash, 64))
            .ok_or_else(|| format!("unexpected shasum output: {line}"))?;
        hashes.insert(path.to_owned(), hash.to_owned());
    }
    Ok(hashes)
}

fn collect_files(dir: &Path, prefix: &str, files: &mut Vec<String>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        if entry.file_type()?.is_dir() {
            collect_files(&entry.path(), &path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

/// Checks one vendored directory, named `kafka-<tag>`, against its manifest. Problems
/// name files as `<label>/<path>`.
pub(crate) fn verify_dir(dir: &Path, label: &str) -> Result<Vec<String>, String> {
    let manifest_path = dir.join(MANIFEST);
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|error| format!("could not read {label}/{MANIFEST}: {error}"))?;
    let manifest =
        Manifest::parse(&text).map_err(|error| format!("{label}/{MANIFEST}: {error}"))?;
    let mut problems = Vec::new();
    let name = dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned());
    if name.as_deref() != Some(&format!("kafka-{}", manifest.tag)) {
        problems.push(format!(
            "{label}: the manifest is for Kafka {}, so the directory must be named kafka-{}",
            manifest.tag, manifest.tag
        ));
    }
    let actual = hash_tree(dir)?;
    for path in actual
        .keys()
        .filter(|path| Path::new(path).extension().is_some_and(|ext| ext == "json"))
    {
        let text = std::fs::read_to_string(dir.join(path))
            .map_err(|error| format!("could not read {label}/{path}: {error}"))?;
        if let Err(error) = message_spec::load(&format!("{label}/{path}"), &text) {
            problems.push(error.to_string());
        }
    }
    for change in compare(&manifest.hashes, &actual) {
        problems.push(match change {
            Change::Added(path) => format!(
                "{label}/{path}: not in {MANIFEST}; vendored files come only from \
                 `cargo xtask schemas vendor`"
            ),
            Change::Removed(path) => format!("{label}/{path}: listed in {MANIFEST} but missing"),
            Change::Modified(path) => format!(
                "{label}/{path}: modified; vendored Kafka files are never edited (ADR-0008)"
            ),
        });
    }
    Ok(problems)
}

/// Runs `schemas verify` on every `schemas/kafka-*` directory.
pub(crate) fn run_verify(out: &mut impl Write) -> Exit {
    match verify_all(&crate::repo_root()) {
        Ok((0, _)) => {
            output::line(out, "error: no schemas/kafka-<tag>/ directory to verify");
            Exit::CheckFailed
        }
        Ok((count, problems)) if problems.is_empty() => {
            output::line(
                out,
                &format!(
                    "schemas verify: ok, {count} vendored directory(ies) match their manifests"
                ),
            );
            Exit::Success
        }
        Ok((_, problems)) => {
            for problem in &problems {
                output::line(out, problem);
            }
            output::line(
                out,
                &format!(
                    "schemas verify: {} problem(s); vendored Kafka files are copied, never edited (ADR-0008)",
                    problems.len()
                ),
            );
            Exit::CheckFailed
        }
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            Exit::CheckFailed
        }
    }
}

/// The number of vendored directories checked, and every problem found.
fn verify_all(root: &Path) -> Result<(usize, Vec<String>), String> {
    let schemas = root.join(SCHEMAS_DIR);
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(&schemas)
        .map_err(|error| format!("could not read {SCHEMAS_DIR}/: {error}"))?
    {
        let entry = entry.map_err(|error| format!("could not read {SCHEMAS_DIR}/: {error}"))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("kafka-") && entry.path().is_dir() {
            dirs.push(name);
        }
    }
    dirs.sort();
    let mut problems = Vec::new();
    for name in &dirs {
        problems.extend(verify_dir(
            &schemas.join(name),
            &format!("{SCHEMAS_DIR}/{name}"),
        )?);
    }
    Ok((dirs.len(), problems))
}

/// Runs `schemas vendor --tag <tag> [--dry-run]`.
pub(crate) fn run_vendor(tag: &str, dry_run: bool, out: &mut impl Write) -> Exit {
    match vendor(&crate::repo_root(), tag, dry_run) {
        Ok(lines) => {
            for line in lines {
                output::line(out, &line);
            }
            Exit::Success
        }
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            Exit::CheckFailed
        }
    }
}

/// A tag safe to pass to git: it can't be read as an option, and has no path or
/// shell-significant characters.
pub(crate) fn is_valid_tag(tag: &str) -> bool {
    tag.chars()
        .next()
        .is_some_and(|first| first.is_ascii_alphanumeric())
        && tag
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

fn vendor(root: &Path, tag: &str, dry_run: bool) -> Result<Vec<String>, String> {
    if !is_valid_tag(tag) {
        return Err(format!(
            "`{tag}` is not a tag: use letters, digits, `.`, `-` and `_`, starting with a \
             letter or digit"
        ));
    }
    let work = root.join("target").join("schemas-vendor");
    let clone = work.join(format!("kafka-{tag}-source"));
    let staging = work.join(format!("kafka-{tag}"));
    for dir in [&clone, &staging] {
        if dir.exists() {
            std::fs::remove_dir_all(dir)
                .map_err(|error| format!("could not clear {}: {error}", dir.display()))?;
        }
    }
    std::fs::create_dir_all(&work)
        .map_err(|error| format!("could not create {}: {error}", work.display()))?;
    let clone_path = clone.to_string_lossy().into_owned();
    git(
        &work,
        &[
            "clone",
            "--quiet",
            "--depth",
            "1",
            "--filter=blob:none",
            "--sparse",
            "--branch",
            tag,
            "--",
            KAFKA_REPOSITORY,
            &clone_path,
        ],
    )?;
    let mut sparse = vec!["sparse-checkout", "set", "--no-cone"];
    let patterns: Vec<String> = KAFKA_FILES
        .iter()
        .map(|file| format!("/{file}"))
        .chain(SOURCES.iter().map(|(_, source)| format!("/{source}/")))
        .collect();
    sparse.extend(patterns.iter().map(String::as_str));
    git(&clone, &sparse)?;
    let commit = git(&clone, &["rev-parse", "HEAD"])?.trim().to_owned();
    for file in KAFKA_FILES {
        copy_file(&clone.join(file), &staging.join(file))?;
    }
    for (directory, source) in SOURCES {
        let from = clone.join(source);
        let entries = std::fs::read_dir(&from)
            .map_err(|error| format!("{source} is missing at {tag}: {error}"))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("could not read {source}: {error}"))?;
            if entry.path().is_file() {
                copy_file(
                    &entry.path(),
                    &staging.join(directory).join(entry.file_name()),
                )?;
            }
        }
    }
    let manifest = Manifest {
        tag: tag.to_owned(),
        commit,
        hashes: hash_tree(&staging)?,
    };
    let target = root.join(SCHEMAS_DIR).join(format!("kafka-{tag}"));
    let label = format!("{SCHEMAS_DIR}/kafka-{tag}");
    let current = match std::fs::read_to_string(target.join(MANIFEST)) {
        Ok(text) => {
            Some(Manifest::parse(&text).map_err(|error| format!("{label}/{MANIFEST}: {error}"))?)
        }
        Err(_) => None,
    };
    let mut lines = report(&label, current.as_ref(), &manifest);
    if dry_run {
        lines.push("dry run: nothing was written".to_owned());
        return Ok(lines);
    }
    replace_dir(&staging, &target)?;
    std::fs::write(target.join(MANIFEST), manifest.render())
        .map_err(|error| format!("could not write {label}/{MANIFEST}: {error}"))?;
    if !target.join(README).exists() {
        lines.push(format!(
            "{label}/{README} is missing: write one from the previous version's (docs/guides/updating-kafka-schemas.md)"
        ));
    }
    lines.push(format!("wrote {label} and its {MANIFEST}"));
    Ok(lines)
}

/// What vendoring `next` would change in a directory whose manifest is `current`.
pub(crate) fn report(label: &str, current: Option<&Manifest>, next: &Manifest) -> Vec<String> {
    let empty = BTreeMap::new();
    let before = current.map_or(&empty, |manifest| &manifest.hashes);
    let mut lines = vec![format!(
        "{label}: Apache Kafka {} at commit {}",
        next.tag, next.commit
    )];
    if let Some(current) = current
        && current.commit != next.commit
    {
        lines.push(format!(
            "  the tag now points at {}, not {}",
            next.commit, current.commit
        ));
    }
    let changes = compare(before, &next.hashes);
    if changes.is_empty() {
        lines.push("  no files change".to_owned());
    }
    for change in changes {
        lines.push(match change {
            Change::Added(path) => format!("  added    {path}"),
            Change::Removed(path) => format!("  removed  {path}"),
            Change::Modified(path) => format!("  changed  {path}"),
        });
    }
    lines
}

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|error| format!("could not run git: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!(
            "git {} failed: {}",
            args.first().unwrap_or(&""),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn copy_file(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
    }
    std::fs::copy(from, to)
        .map(drop)
        .map_err(|error| format!("could not copy {}: {error}", from.display()))
}

/// Replaces every file in `target` except its README.md with the files in `staging`.
fn replace_dir(staging: &Path, target: &Path) -> Result<(), String> {
    if target.exists() {
        for entry in std::fs::read_dir(target)
            .map_err(|error| format!("could not read {}: {error}", target.display()))?
        {
            let path = entry
                .map_err(|error| format!("could not read {}: {error}", target.display()))?
                .path();
            if path.file_name().is_some_and(|name| name == README) {
                continue;
            }
            let removed = if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            };
            removed.map_err(|error| format!("could not remove {}: {error}", path.display()))?;
        }
    }
    let mut files = Vec::new();
    collect_files(staging, "", &mut files)
        .map_err(|error| format!("could not read {}: {error}", staging.display()))?;
    for file in files {
        copy_file(&staging.join(&file), &target.join(&file))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::{Change, Manifest, compare, is_valid_tag, report, verify_dir};

    const COMMIT: &str = "1156b2752aaa28eafc0b4c4d387e63356793268d";
    const HASH_A: &str = "0000000000000000000000000000000000000000000000000000000000000000";
    const HASH_B: &str = "1111111111111111111111111111111111111111111111111111111111111111";

    fn fixture(name: &str) -> PathBuf {
        [
            env!("CARGO_MANIFEST_DIR"),
            "tests",
            "fixtures",
            "schemas",
            name,
            "kafka-1.0.0",
        ]
        .iter()
        .collect()
    }

    fn hashes(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
        entries
            .iter()
            .map(|(path, hash)| ((*path).to_owned(), (*hash).to_owned()))
            .collect()
    }

    #[test]
    fn a_manifest_names_its_source_and_lists_each_file() {
        let text = format!(
            "# Apache Kafka 4.4.0, commit {COMMIT}.\n# A comment.\n{HASH_A}  LICENSE\n{HASH_B}  clients/b.json\n"
        );
        assert_eq!(
            Manifest::parse(&text),
            Ok(Manifest {
                tag: "4.4.0".to_owned(),
                commit: COMMIT.to_owned(),
                hashes: hashes(&[("LICENSE", HASH_A), ("clients/b.json", HASH_B)]),
            })
        );
    }

    #[test]
    fn a_rendered_manifest_parses_back_with_paths_sorted() {
        let manifest = Manifest {
            tag: "4.4.0".to_owned(),
            commit: COMMIT.to_owned(),
            hashes: hashes(&[("clients/b.json", HASH_B), ("LICENSE", HASH_A)]),
        };
        let text = manifest.render();
        assert!(text.ends_with(&format!("{HASH_A}  LICENSE\n{HASH_B}  clients/b.json\n")));
        assert_eq!(Manifest::parse(&text), Ok(manifest));
    }

    #[test]
    fn the_vendor_report_lists_each_change_and_a_moved_tag() {
        let current = Manifest {
            tag: "4.4.0".to_owned(),
            commit: COMMIT.to_owned(),
            hashes: hashes(&[("kept", HASH_A), ("edited", HASH_A), ("gone", HASH_A)]),
        };
        let next = Manifest {
            commit: "2".repeat(40),
            hashes: hashes(&[("kept", HASH_A), ("edited", HASH_B), ("new", HASH_A)]),
            ..current.clone()
        };
        assert_eq!(
            report("schemas/kafka-4.4.0", Some(&current), &next),
            [
                format!(
                    "schemas/kafka-4.4.0: Apache Kafka 4.4.0 at commit {}",
                    "2".repeat(40)
                ),
                format!("  the tag now points at {}, not {COMMIT}", "2".repeat(40)),
                "  added    new".to_owned(),
                "  removed  gone".to_owned(),
                "  changed  edited".to_owned(),
            ]
        );
        assert_eq!(
            report("d", Some(&current), &current)[1],
            "  no files change"
        );
    }

    #[test]
    fn only_plain_tags_reach_git() {
        for tag in ["4.4.0", "3.9.1-rc0", "trunk_1"] {
            assert!(is_valid_tag(tag), "{tag}");
        }
        for tag in [
            "",
            "-c",
            "--upload-pack=x",
            "../x",
            "4.4 0",
            "a;b",
            ".hidden",
        ] {
            assert!(!is_valid_tag(tag), "{tag}");
        }
    }

    #[test]
    fn malformed_manifests_are_rejected_with_the_line() {
        assert_eq!(
            Manifest::parse(&format!("{HASH_A}  a\n")),
            Err("no `# Apache Kafka <tag>, commit <sha>.` line".to_owned())
        );
        assert_eq!(
            Manifest::parse("# Apache Kafka 4.4.0, commit abc.\n"),
            Err("line 1: expected `# Apache Kafka <tag>, commit <40 hex>.`".to_owned())
        );
        let header = format!("# Apache Kafka 4.4.0, commit {COMMIT}.\n");
        assert_eq!(
            Manifest::parse(&format!("{header}{HASH_A} a\n")),
            Err("line 2: expected `<sha256>  <path>`".to_owned())
        );
        assert_eq!(
            Manifest::parse(&format!("{header}{HASH_A}  a\n{HASH_B}  a\n")),
            Err("line 3: a is listed twice".to_owned())
        );
    }

    #[test]
    fn compare_names_added_removed_and_modified_files() {
        let expected = hashes(&[("kept", HASH_A), ("edited", HASH_A), ("gone", HASH_A)]);
        let actual = hashes(&[("kept", HASH_A), ("edited", HASH_B), ("new", HASH_A)]);
        assert_eq!(
            compare(&expected, &actual),
            [
                Change::Added("new".to_owned()),
                Change::Removed("gone".to_owned()),
                Change::Modified("edited".to_owned()),
            ]
        );
    }

    #[test]
    fn the_intact_fixture_passes_and_the_tampered_one_names_each_file() {
        assert_eq!(
            verify_dir(&fixture("intact"), "intact").unwrap(),
            Vec::<String>::new()
        );
        assert_eq!(
            verify_dir(&fixture("tampered"), "t").unwrap(),
            [
                "t/clients/Added.json: not in MANIFEST.sha256; vendored files come only from \
                 `cargo xtask schemas vendor`",
                "t/clients/Removed.json: listed in MANIFEST.sha256 but missing",
                "t/clients/Request.json: modified; vendored Kafka files are never edited \
                 (ADR-0008)",
            ]
        );
    }
}
