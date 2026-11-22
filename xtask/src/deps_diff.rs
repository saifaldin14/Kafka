//! `cargo xtask deps-diff <base-ref>`: the registry crates added, removed and upgraded in
//! `Cargo.lock` since `<base-ref>`, as a Markdown table for a PR's dependency review
//! (docs/standards/security.md#dependencies).
//!
//! The base lockfile comes from `git show <base-ref>:Cargo.lock`, and the head one from
//! the working tree. For each added or upgraded crate the table gives its license, from
//! `cargo metadata`, and how many crates it pulls in transitively, counted in the head
//! lockfile (normal and build dependencies, on every platform, as Cargo locks them).
//! Nothing needs the network. Workspace crates, which have no `source`, are left out.
//!
//! It is an aid for the reviewer, not a check: it exits 0 whatever changed, and 1 only
//! when it can't read its inputs. CI writes its output to the `deny` job's summary when
//! a PR changes `Cargo.lock`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Write;
use std::process::Command;

use crate::Exit;
use crate::graph;
use crate::output;

/// A locked package: its name and version.
type Package = (String, String);

/// A parsed `Cargo.lock`: each registry or path package and the packages it depends on.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Lockfile {
    /// Every locked package with its resolved dependencies.
    dependencies: BTreeMap<Package, Vec<Package>>,
    /// The packages that come from a registry (they have a `source`).
    registry: BTreeSet<Package>,
}

/// One difference between two lockfiles.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Change {
    Added {
        name: String,
        version: String,
    },
    Removed {
        name: String,
        version: String,
    },
    Upgraded {
        name: String,
        from: String,
        to: String,
    },
}

/// Runs the command.
pub(crate) fn run(base_ref: &str, out: &mut impl Write) -> Exit {
    match report(base_ref) {
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

fn report(base_ref: &str) -> Result<Vec<String>, String> {
    let root = crate::repo_root();
    let shown = Command::new("git")
        .args(["show", &format!("{base_ref}:Cargo.lock")])
        .current_dir(&root)
        .output()
        .map_err(|error| format!("could not run git: {error}"))?;
    if !shown.status.success() {
        return Err(format!(
            "could not read Cargo.lock at {base_ref}: {}",
            String::from_utf8_lossy(&shown.stderr).trim_end()
        ));
    }
    let base = parse(&String::from_utf8_lossy(&shown.stdout))
        .map_err(|error| format!("Cargo.lock at {base_ref}: {error}"))?;
    let head_text = std::fs::read_to_string(root.join("Cargo.lock"))
        .map_err(|error| format!("could not read Cargo.lock: {error}"))?;
    let head = parse(&head_text).map_err(|error| format!("Cargo.lock: {error}"))?;
    let licenses = graph::load(None)
        .map_err(|error| error.to_string())?
        .licenses;
    Ok(render(base_ref, &diff(&base, &head), &head, &licenses))
}

/// Parses a version 3 or 4 `Cargo.lock`.
pub(crate) fn parse(text: &str) -> Result<Lockfile, String> {
    let document: toml::Table = toml::from_str(text).map_err(|error| error.to_string())?;
    let entries = document
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or("no [[package]] entries")?;
    let mut raw = Vec::new();
    for entry in entries {
        let field = |key: &str| {
            entry
                .get(key)
                .and_then(toml::Value::as_str)
                .ok_or_else(|| format!("a [[package]] entry has no `{key}`"))
        };
        let package = (field("name")?.to_owned(), field("version")?.to_owned());
        let dependencies: Vec<&str> = entry
            .get("dependencies")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
            .collect();
        raw.push((package, entry.get("source").is_some(), dependencies));
    }
    let mut versions: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for ((name, version), _, _) in &raw {
        versions.entry(name).or_default().push(version);
    }
    let mut lockfile = Lockfile::default();
    for (package, from_registry, dependencies) in &raw {
        let resolved = dependencies
            .iter()
            .map(|reference| resolve(reference, &versions))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("{} {}: {error}", package.0, package.1))?;
        lockfile.dependencies.insert(package.clone(), resolved);
        if *from_registry {
            lockfile.registry.insert(package.clone());
        }
    }
    Ok(lockfile)
}

/// Resolves a lockfile dependency reference: `name`, `name version` or
/// `name version (source)`. A bare name is used only when one version is locked.
fn resolve(reference: &str, versions: &BTreeMap<&str, Vec<&str>>) -> Result<Package, String> {
    let mut words = reference.split_whitespace();
    let name = words.next().ok_or("an empty dependency")?;
    let locked = versions
        .get(name)
        .ok_or_else(|| format!("depends on `{name}`, which isn't locked"))?;
    let version = match (words.next(), locked.as_slice()) {
        (Some(version), _) => version,
        (None, [only]) => only,
        (None, _) => return Err(format!("depends on `{name}`, locked in several versions")),
    };
    Ok((name.to_owned(), version.to_owned()))
}

/// The registry packages that differ, by name. A name locked in exactly one version on
/// each side, with different versions, is an upgrade (or a downgrade); otherwise each
/// version that appears on one side only is added or removed.
pub(crate) fn diff(base: &Lockfile, head: &Lockfile) -> Vec<Change> {
    let by_name = |lockfile: &Lockfile| {
        let mut names: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (name, version) in &lockfile.registry {
            names
                .entry(name.clone())
                .or_default()
                .insert(version.clone());
        }
        names
    };
    let (before, after) = (by_name(base), by_name(head));
    let empty = BTreeSet::new();
    let names: BTreeSet<&String> = before.keys().chain(after.keys()).collect();
    let mut changes = Vec::new();
    for name in names {
        let old = before.get(name).unwrap_or(&empty);
        let new = after.get(name).unwrap_or(&empty);
        if let ([from], [to]) = (
            old.iter().collect::<Vec<_>>().as_slice(),
            new.iter().collect::<Vec<_>>().as_slice(),
        ) && from != to
        {
            changes.push(Change::Upgraded {
                name: name.clone(),
                from: (*from).clone(),
                to: (*to).clone(),
            });
            continue;
        }
        for version in new.difference(old) {
            changes.push(Change::Added {
                name: name.clone(),
                version: version.clone(),
            });
        }
        for version in old.difference(new) {
            changes.push(Change::Removed {
                name: name.clone(),
                version: version.clone(),
            });
        }
    }
    changes
}

/// How many other packages `package` reaches through its locked dependencies.
pub(crate) fn transitive_count(lockfile: &Lockfile, package: &Package) -> usize {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::from([package]);
    while let Some(next) = queue.pop_front() {
        for dependency in lockfile.dependencies.get(next).into_iter().flatten() {
            if dependency != package && seen.insert(dependency) {
                queue.push_back(dependency);
            }
        }
    }
    seen.len()
}

/// The Markdown report.
pub(crate) fn render(
    base_ref: &str,
    changes: &[Change],
    head: &Lockfile,
    licenses: &BTreeMap<Package, String>,
) -> Vec<String> {
    let mut lines = vec![
        format!("## Dependency changes against {base_ref}"),
        String::new(),
    ];
    if changes.is_empty() {
        lines.push("No registry crates were added, removed or upgraded.".to_owned());
        return lines;
    }
    lines.push("| Change | Crate | Version | License | Transitive crates |".to_owned());
    lines.push("|---|---|---|---|---|".to_owned());
    for change in changes {
        let (kind, name, shown, locked) = match change {
            Change::Added { name, version } => ("added", name, version.clone(), Some(version)),
            Change::Removed { name, version } => ("removed", name, version.clone(), None),
            Change::Upgraded { name, from, to } => {
                ("upgraded", name, format!("{from} → {to}"), Some(to))
            }
        };
        let (license, transitive) = match locked {
            None => (String::new(), String::new()),
            Some(version) => {
                let package = (name.clone(), version.clone());
                (
                    licenses
                        .get(&package)
                        .cloned()
                        .unwrap_or_else(|| "not declared".to_owned()),
                    transitive_count(head, &package).to_string(),
                )
            }
        };
        lines.push(format!(
            "| {kind} | `{name}` | {shown} | {license} | {transitive} |"
        ));
    }
    lines.push(String::new());
    lines.push(
        "Each new crate needs answers to the dependency questions in \
         docs/standards/security.md#dependencies."
            .to_owned(),
    );
    lines
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{Change, diff, parse, render, transitive_count};

    const BASE: &str = include_str!("../tests/fixtures/deps-diff/base.lock");
    const HEAD: &str = include_str!("../tests/fixtures/deps-diff/head.lock");

    fn package(name: &str, version: &str) -> (String, String) {
        (name.to_owned(), version.to_owned())
    }

    #[test]
    fn the_diff_names_added_removed_and_upgraded_registry_crates() {
        let (base, head) = (parse(BASE).unwrap(), parse(HEAD).unwrap());
        assert_eq!(
            diff(&base, &head),
            [
                Change::Added {
                    name: "bitflags".to_owned(),
                    version: "2.9.0".to_owned()
                },
                Change::Removed {
                    name: "lazy_static".to_owned(),
                    version: "1.5.0".to_owned()
                },
                Change::Upgraded {
                    name: "memchr".to_owned(),
                    from: "2.7.4".to_owned(),
                    to: "2.7.5".to_owned()
                },
                Change::Added {
                    name: "serde".to_owned(),
                    version: "1.0.228".to_owned()
                },
                Change::Added {
                    name: "syn".to_owned(),
                    version: "2.0.100".to_owned()
                },
            ]
        );
        assert_eq!(transitive_count(&head, &package("serde", "1.0.228")), 2);
    }

    #[test]
    fn the_report_gives_license_and_transitive_count_for_new_versions() {
        let (base, head) = (parse(BASE).unwrap(), parse(HEAD).unwrap());
        let licenses = BTreeMap::from([
            (package("serde", "1.0.228"), "MIT OR Apache-2.0".to_owned()),
            (package("memchr", "2.7.5"), "Unlicense OR MIT".to_owned()),
        ]);
        let lines = render("main", &diff(&base, &head), &head, &licenses);
        assert_eq!(
            lines[2..9],
            [
                "| Change | Crate | Version | License | Transitive crates |",
                "|---|---|---|---|---|",
                "| added | `bitflags` | 2.9.0 | not declared | 0 |",
                "| removed | `lazy_static` | 1.5.0 |  |  |",
                "| upgraded | `memchr` | 2.7.4 → 2.7.5 | Unlicense OR MIT | 0 |",
                "| added | `serde` | 1.0.228 | MIT OR Apache-2.0 | 2 |",
                "| added | `syn` | 2.0.100 | not declared | 1 |",
            ]
        );
    }

    #[test]
    fn identical_lockfiles_report_no_changes_and_bad_references_are_errors() {
        let head = parse(HEAD).unwrap();
        let lines = render("main", &diff(&head, &head), &head, &BTreeMap::new());
        assert_eq!(
            lines[2],
            "No registry crates were added, removed or upgraded."
        );
        let dangling = "version = 4\n\n[[package]]\nname = \"a\"\nversion = \"1.0.0\"\ndependencies = [\"b\"]\n";
        assert_eq!(
            parse(dangling),
            Err("a 1.0.0: depends on `b`, which isn't locked".to_owned())
        );
    }
}
