//! `cargo xtask docs check-crates` and `cargo xtask docs new-crate-doc <crate>`: every
//! workspace crate has a contributor deep dive at `docs/crates/<crate>.md`
//! (docs/crates/README.md).
//!
//! The check fails when a member has no deep dive, and lists the deep dives that are
//! still stubs. A stub is a deep dive that still contains [`STUB_MARKER`]:
//! `new-crate-doc` writes one from docs/crates/template.md, so a new crate passes CI
//! straight away and its stub stays visible until someone fills it in.
//!
//! That every member is in architecture §4 is `check-layers`' rule (the layer table
//! lists every member and equals §4), so it isn't repeated here.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;

use crate::Exit;
use crate::graph;
use crate::output;

/// Where the deep dives live, relative to the repository root.
const DOCS_DIR: &str = "docs/crates";

/// The template a stub is made from.
const TEMPLATE: &str = "docs/crates/template.md";

/// The template's placeholder for the crate name.
const PLACEHOLDER: &str = "<crate>";

/// The first line of every stub. A deep dive is finished when this line is gone.
pub(crate) const STUB_MARKER: &str =
    "<!-- stub: fill every section from the code, then delete this line -->";

/// What the check found, by crate name.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Findings {
    /// Members with no deep dive.
    pub(crate) missing: Vec<String>,
    /// Members whose deep dive is still a stub.
    pub(crate) stubs: Vec<String>,
}

/// Runs `docs check-crates`.
pub(crate) fn run_check(out: &mut impl Write) -> Exit {
    let root = crate::repo_root();
    let members = match member_names() {
        Ok(members) => members,
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            return Exit::CheckFailed;
        }
    };
    let findings = check(&members, |name| {
        std::fs::read_to_string(root.join(doc_path(name))).ok()
    });
    for name in &findings.missing {
        output::line(
            out,
            &format!(
                "{}: missing; `cargo xtask docs new-crate-doc {name}` writes a stub",
                doc_path(name)
            ),
        );
    }
    for name in &findings.stubs {
        output::line(out, &format!("{}: still a stub", doc_path(name)));
    }
    if findings.missing.is_empty() {
        output::line(
            out,
            &format!(
                "docs check-crates: ok, {} crate(s), {} deep dive(s) still a stub",
                members.len(),
                findings.stubs.len()
            ),
        );
        Exit::Success
    } else {
        output::line(
            out,
            &format!(
                "docs check-crates: {} crate(s) without a deep dive (docs/crates/README.md)",
                findings.missing.len()
            ),
        );
        Exit::CheckFailed
    }
}

/// Runs `docs new-crate-doc <crate>`.
pub(crate) fn run_new(name: &str, out: &mut impl Write) -> Exit {
    match write_stub(&crate::repo_root(), name) {
        Ok(path) => {
            output::line(
                out,
                &format!(
                    "wrote {path}: fill every section from the code, delete the stub line, \
                     and link it from docs/crates/README.md"
                ),
            );
            Exit::Success
        }
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            Exit::CheckFailed
        }
    }
}

fn write_stub(root: &Path, name: &str) -> Result<String, String> {
    if !member_names()?.contains(name) {
        return Err(format!(
            "{name} is not a workspace member; a deep dive is written when the crate's \
             first code lands"
        ));
    }
    let path = doc_path(name);
    let target = root.join(&path);
    if target.exists() {
        return Err(format!("{path} already exists"));
    }
    let template = std::fs::read_to_string(root.join(TEMPLATE))
        .map_err(|error| format!("could not read {TEMPLATE}: {error}"))?;
    std::fs::write(&target, stub(&template, name))
        .map_err(|error| format!("could not write {path}: {error}"))?;
    Ok(path)
}

fn member_names() -> Result<BTreeSet<String>, String> {
    let graph = graph::load(None).map_err(|error| error.to_string())?;
    Ok(graph
        .members
        .iter()
        .map(|id| graph.name(id).to_owned())
        .collect())
}

fn doc_path(name: &str) -> String {
    format!("{DOCS_DIR}/{name}.md")
}

/// Checks each member's deep dive; `read` returns a deep dive's text, or `None` if the
/// file doesn't exist.
pub(crate) fn check(members: &BTreeSet<String>, read: impl Fn(&str) -> Option<String>) -> Findings {
    let mut findings = Findings::default();
    for name in members {
        match read(name) {
            None => findings.missing.push(name.clone()),
            Some(text) if text.contains(STUB_MARKER) => findings.stubs.push(name.clone()),
            Some(_) => {}
        }
    }
    findings
}

/// The stub for `name`: the marker line, then the template with the crate name filled in.
pub(crate) fn stub(template: &str, name: &str) -> String {
    format!("{STUB_MARKER}\n{}", template.replace(PLACEHOLDER, name))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{Findings, STUB_MARKER, check, stub};

    fn members(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn a_member_without_a_deep_dive_is_missing_and_stubs_are_listed() {
        let docs = BTreeMap::from([
            (
                "alluvium-types",
                "# `alluvium-types`: deep dive\n".to_owned(),
            ),
            ("xtask", stub("# `<crate>`: deep dive\n", "xtask")),
        ]);
        let findings = check(
            &members(&["alluvium-types", "lint-canary", "xtask"]),
            |name| docs.get(name).cloned(),
        );
        assert_eq!(
            findings,
            Findings {
                missing: vec!["lint-canary".to_owned()],
                stubs: vec!["xtask".to_owned()],
            }
        );
    }

    #[test]
    fn a_stub_is_the_marker_then_the_template_with_the_crate_name() {
        let text = stub(
            "# `<crate>`: deep dive\n\nSee `<crate>`.\n",
            "alluvium-raft",
        );
        assert_eq!(
            text,
            format!("{STUB_MARKER}\n# `alluvium-raft`: deep dive\n\nSee `alluvium-raft`.\n")
        );
        let findings = check(&members(&["alluvium-raft"]), |_| Some(text.clone()));
        assert_eq!(findings.stubs, ["alluvium-raft"]);
    }
}
