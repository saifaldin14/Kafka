//! `cargo xtask check-layers`: enforces the crate layering rules of
//! [ADR-0009](../../docs/adr/0009-crate-layering-and-dependency-rules.md) and
//! docs/architecture.md §5.
//!
//! Rules, checked on the resolved dependency graph:
//!
//! 1. Every workspace member is in the layer table (`layers::LAYERS`).
//! 2. Normal and build dependencies between workspace crates point to a strictly lower
//!    layer. Same-layer edges need an entry in [`SAME_LAYER_ALLOWED`] backed by an
//!    accepted ADR, and every entry must be used.
//! 3. A non-dev crate uses a dev crate only as a dev-dependency.
//! 4. Nothing depends on the layer 5 binary.
//! 5. No layer 0-2 crate reaches an async runtime or networking crate
//!    ([`RUNTIME_CRATES`]) through normal or build edges, directly or transitively.
//!
//! The layer table must also match docs/architecture.md §4. Every violation is reported
//! in one run.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::Exit;
use crate::graph::{self, Graph};
use crate::layers::{LAYERS, Layer, layer_of};
use crate::output;

/// A same-layer dependency allowed by an accepted ADR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AllowedEdge {
    pub(crate) from: &'static str,
    pub(crate) to: &'static str,
    /// ADR number, for example `"0012"`.
    pub(crate) adr: &'static str,
}

/// Same-layer dependencies that are allowed. Empty: architecture §5 has no exceptions.
pub(crate) const SAME_LAYER_ALLOWED: &[AllowedEdge] = &[];

/// Crates that layers 0-2 must never reach, with the reason each is banned.
pub(crate) const RUNTIME_CRATES: &[(&str, &str)] = &[
    ("tokio", "async runtime"),
    ("tokio-util", "async runtime utilities"),
    ("mio", "event loop for network I/O"),
    ("async-std", "async runtime"),
    ("smol", "async runtime"),
    ("async-io", "async runtime reactor"),
    ("async-executor", "async executor"),
    ("futures-executor", "async executor"),
    ("hyper", "HTTP networking"),
    ("h2", "HTTP/2 networking"),
    ("reqwest", "HTTP client"),
    ("tonic", "gRPC networking"),
    ("axum", "HTTP server"),
    ("tower", "network service stack"),
    ("socket2", "socket networking"),
];

/// Options for `cargo xtask check-layers`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LayerOptions {
    pub(crate) print: bool,
    /// Check another workspace (used by the fixture tests).
    pub(crate) manifest_path: Option<PathBuf>,
}

/// One broken rule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Violation {
    UnknownCrate {
        name: String,
    },
    Edge {
        from: String,
        from_layer: Layer,
        to: String,
        to_layer: Layer,
        rule: EdgeRule,
    },
    Runtime {
        from: String,
        from_layer: Layer,
        path: Vec<String>,
        reason: &'static str,
    },
    AllowListAdrMissing {
        entry: AllowedEdgeText,
    },
    AllowListAdrNotAccepted {
        entry: AllowedEdgeText,
    },
    AllowListUnused {
        entry: AllowedEdgeText,
    },
    Architecture {
        problem: String,
    },
}

/// An allow-list entry, printable in a violation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AllowedEdgeText(String);

impl From<&AllowedEdge> for AllowedEdgeText {
    fn from(entry: &AllowedEdge) -> Self {
        Self(format!(
            "{} -> {} (ADR-{})",
            entry.from, entry.to, entry.adr
        ))
    }
}

/// Which edge rule was broken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum EdgeRule {
    Upward,
    SameLayer,
    DevCrateAsNormal,
    DependsOnBinary,
}

impl fmt::Display for EdgeRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            EdgeRule::Upward => "a crate may depend only on lower layers",
            EdgeRule::SameLayer => {
                "same-layer dependencies need an allow-list entry backed by an accepted ADR"
            }
            EdgeRule::DevCrateAsNormal => "dev crates may only be dev-dependencies",
            EdgeRule::DependsOnBinary => "nothing may depend on the layer 5 binary",
        })
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Violation::UnknownCrate { name } => write!(
                f,
                "crate {name} has no layer: add it to xtask/src/layers.rs and docs/architecture.md §4"
            ),
            Violation::Edge {
                from,
                from_layer,
                to,
                to_layer,
                rule,
            } => write!(
                f,
                "{from} (layer {from_layer}) -> {to} (layer {to_layer}): {rule}"
            ),
            Violation::Runtime {
                from,
                from_layer,
                path,
                reason,
            } => write!(
                f,
                "{from} (layer {from_layer}) -> {}: layers 0-2 must not depend on an async runtime or networking crate ({reason})",
                path.join(" -> ")
            ),
            Violation::AllowListAdrMissing { entry } => {
                write!(f, "allow-list entry {}: the ADR does not exist", entry.0)
            }
            Violation::AllowListAdrNotAccepted { entry } => {
                write!(f, "allow-list entry {}: the ADR is not Accepted", entry.0)
            }
            Violation::AllowListUnused { entry } => write!(
                f,
                "allow-list entry {}: no such dependency exists; remove the entry",
                entry.0
            ),
            Violation::Architecture { problem } => write!(
                f,
                "xtask/src/layers.rs and docs/architecture.md §4 disagree: {problem}"
            ),
        }
    }
}

/// The repository root (the parent of the `xtask` crate).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Runs the command.
pub(crate) fn run(options: &LayerOptions, out: &mut impl Write) -> Exit {
    let graph = match graph::load(options.manifest_path.as_deref()) {
        Ok(graph) => graph,
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            return Exit::CheckFailed;
        }
    };
    if options.print {
        let present: BTreeSet<&str> = graph.members.iter().map(|id| graph.name(id)).collect();
        output::layer_table(out, LAYERS, &present);
        return Exit::Success;
    }
    let root = repo_root();
    let mut violations = check_graph(&graph, SAME_LAYER_ALLOWED, &root.join("docs/adr"));
    match std::fs::read_to_string(root.join("docs/architecture.md")) {
        Ok(text) => violations.extend(check_architecture(&text)),
        Err(error) => violations.push(Violation::Architecture {
            problem: format!("could not read docs/architecture.md: {error}"),
        }),
    }
    violations.sort();
    violations.dedup();
    if violations.is_empty() {
        output::line(out, "check-layers: ok");
        Exit::Success
    } else {
        for violation in &violations {
            output::line(out, &violation.to_string());
        }
        output::line(
            out,
            &format!("check-layers: {} violation(s)", violations.len()),
        );
        Exit::CheckFailed
    }
}

/// Checks every graph rule and the allow-list. `adr_dir` is where ADR files live.
pub(crate) fn check_graph(
    graph: &Graph,
    allowed: &[AllowedEdge],
    adr_dir: &Path,
) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut member_layers = BTreeMap::new();
    for id in &graph.members {
        let name = graph.name(id);
        match layer_of(name) {
            Some(layer) => {
                member_layers.insert(id.as_str(), layer);
            }
            None => violations.push(Violation::UnknownCrate {
                name: name.to_owned(),
            }),
        }
    }
    let mut used = BTreeSet::new();
    for edge in &graph.edges {
        let (Some(&from_layer), Some(&to_layer)) = (
            member_layers.get(edge.from.as_str()),
            member_layers.get(edge.to.as_str()),
        ) else {
            continue;
        };
        let (from, to) = (graph.name(&edge.from), graph.name(&edge.to));
        let mut rule = edge_rule(from_layer, to_layer, edge.is_non_dev());
        if rule == Some(EdgeRule::SameLayer)
            && let Some(index) = allowed
                .iter()
                .position(|entry| entry.from == from && entry.to == to)
        {
            used.insert(index);
            rule = None;
        }
        if let Some(rule) = rule {
            violations.push(Violation::Edge {
                from: from.to_owned(),
                from_layer,
                to: to.to_owned(),
                to_layer,
                rule,
            });
        }
    }
    violations.extend(check_runtime_crates(graph, &member_layers));
    for (index, entry) in allowed.iter().enumerate() {
        match adr_status(adr_dir, entry.adr) {
            AdrStatus::Missing => violations.push(Violation::AllowListAdrMissing {
                entry: entry.into(),
            }),
            AdrStatus::NotAccepted => violations.push(Violation::AllowListAdrNotAccepted {
                entry: entry.into(),
            }),
            AdrStatus::Accepted => {}
        }
        if !used.contains(&index) {
            violations.push(Violation::AllowListUnused {
                entry: entry.into(),
            });
        }
    }
    violations
}

/// The rule an edge between two workspace crates breaks, if any. Dev crates may
/// depend on anything except the binary, and dev-dependencies are unrestricted.
fn edge_rule(from: Layer, to: Layer, non_dev: bool) -> Option<EdgeRule> {
    if to == Layer::BINARY {
        return Some(EdgeRule::DependsOnBinary);
    }
    let Layer::Numbered(from) = from else {
        return None;
    };
    if !non_dev {
        return None;
    }
    match to {
        Layer::Dev => Some(EdgeRule::DevCrateAsNormal),
        Layer::Numbered(to) if to > from => Some(EdgeRule::Upward),
        Layer::Numbered(to) if to == from => Some(EdgeRule::SameLayer),
        Layer::Numbered(_) => None,
    }
}

/// Finds, for each layer 0-2 member, the shortest normal or build path to each banned crate.
fn check_runtime_crates(graph: &Graph, member_layers: &BTreeMap<&str, Layer>) -> Vec<Violation> {
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in graph.edges.iter().filter(|edge| edge.is_non_dev()) {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }
    let mut violations = Vec::new();
    for (&start, &from_layer) in member_layers {
        if !from_layer.is_runtime_free() {
            continue;
        }
        let mut parent: BTreeMap<&str, &str> = BTreeMap::new();
        let mut queue = VecDeque::from([start]);
        let mut seen = BTreeSet::from([start]);
        while let Some(node) = queue.pop_front() {
            if node != start
                && let Some((_, reason)) = RUNTIME_CRATES
                    .iter()
                    .find(|(name, _)| *name == graph.name(node))
            {
                let mut path = vec![graph.name(node).to_owned()];
                let mut current = node;
                while let Some(&previous) = parent.get(current) {
                    if previous != start {
                        path.push(graph.name(previous).to_owned());
                    }
                    current = previous;
                }
                path.reverse();
                violations.push(Violation::Runtime {
                    from: graph.name(start).to_owned(),
                    from_layer,
                    path,
                    reason,
                });
                continue;
            }
            for &next in adjacency.get(node).map_or(&[][..], Vec::as_slice) {
                if seen.insert(next) {
                    parent.insert(next, node);
                    queue.push_back(next);
                }
            }
        }
    }
    violations
}

enum AdrStatus {
    Missing,
    NotAccepted,
    Accepted,
}

/// Reads the status line of ADR `number` in `adr_dir`.
fn adr_status(adr_dir: &Path, number: &str) -> AdrStatus {
    let prefix = format!("{number}-");
    let file = std::fs::read_dir(adr_dir).ok().and_then(|entries| {
        entries
            .filter_map(Result::ok)
            .find(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
    });
    let Some(text) = file.and_then(|entry| std::fs::read_to_string(entry.path()).ok()) else {
        return AdrStatus::Missing;
    };
    if text
        .lines()
        .any(|line| line.trim() == "- **Status:** Accepted")
    {
        AdrStatus::Accepted
    } else {
        AdrStatus::NotAccepted
    }
}

/// Compares the layer table with the code-map table in docs/architecture.md §4.
pub(crate) fn check_architecture(architecture: &str) -> Vec<Violation> {
    let documented = match parse_code_map(architecture) {
        Ok(documented) => documented,
        Err(problem) => return vec![Violation::Architecture { problem }],
    };
    let table: BTreeMap<&str, Layer> = LAYERS.iter().copied().collect();
    let mut violations = Vec::new();
    for (name, layer) in &table {
        match documented.get(*name) {
            None => violations.push(Violation::Architecture {
                problem: format!("{name} is in xtask but not in architecture.md"),
            }),
            Some(documented_layer) if documented_layer != layer => {
                violations.push(Violation::Architecture {
                    problem: format!(
                        "{name} is layer {layer} in xtask but layer {documented_layer} in architecture.md"
                    ),
                });
            }
            Some(_) => {}
        }
    }
    for name in documented.keys() {
        if !table.contains_key(name.as_str()) {
            violations.push(Violation::Architecture {
                problem: format!("{name} is in architecture.md but not in xtask"),
            });
        }
    }
    violations
}

/// Reads the layer and crate columns of the table rows in the "## 4. Code map" section.
fn parse_code_map(architecture: &str) -> Result<BTreeMap<String, Layer>, String> {
    let section = architecture
        .split("\n## ")
        .find(|section| section.starts_with("4. Code map"))
        .ok_or("no `## 4. Code map` section")?;
    let mut crates = BTreeMap::new();
    for row in section.lines().filter(|line| line.starts_with("| ")) {
        let mut cells = row.split('|').skip(1).map(str::trim);
        let (Some(layer), Some(crate_cell)) = (cells.next(), cells.next()) else {
            continue;
        };
        let Some(layer) = Layer::parse(layer) else {
            continue; // the header row
        };
        let name = crate_cell
            .split('`')
            .nth(1)
            .ok_or_else(|| format!("row without a `crate` name: {row}"))?;
        crates.insert(name.to_owned(), layer);
    }
    if crates.is_empty() {
        return Err("the §4 code map has no crate rows".to_owned());
    }
    Ok(crates)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    use super::{AllowedEdge, EdgeRule, Violation, check_architecture, check_graph};
    use crate::graph::{DepKind, Edge, Graph};

    /// A graph whose package ids are their names. `members` are workspace crates.
    fn graph(members: &[&str], others: &[&str], edges: &[(&str, &str, DepKind)]) -> Graph {
        let mut graph = Graph::default();
        for name in members.iter().chain(others) {
            graph.names.insert((*name).to_owned(), (*name).to_owned());
        }
        graph.members = members.iter().map(|name| (*name).to_owned()).collect();
        graph.edges = edges
            .iter()
            .map(|(from, to, kind)| Edge {
                from: (*from).to_owned(),
                to: (*to).to_owned(),
                kinds: BTreeSet::from([*kind]),
            })
            .collect();
        graph
    }

    fn check(graph: &Graph) -> Vec<String> {
        check_graph(graph, &[], Path::new("/nonexistent"))
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn a_layered_graph_passes() {
        let graph = graph(
            &[
                "alluvium-types",
                "alluvium-storage",
                "alluvium-server",
                "alluvium-testkit",
            ],
            &[],
            &[
                ("alluvium-storage", "alluvium-types", DepKind::Normal),
                ("alluvium-server", "alluvium-storage", DepKind::Build),
                ("alluvium-storage", "alluvium-testkit", DepKind::Dev),
                ("alluvium-testkit", "alluvium-server", DepKind::Normal),
                ("alluvium-types", "alluvium-server", DepKind::Dev),
            ],
        );
        assert_eq!(check(&graph), Vec::<String>::new());
    }

    #[test]
    fn each_edge_rule_is_reported_and_all_violations_come_out_in_one_run() {
        let graph = graph(
            &[
                "alluvium-types",
                "alluvium-storage",
                "alluvium-raft",
                "alluvium-server",
                "alluvium-testkit",
                "alluvium",
            ],
            &[],
            &[
                ("alluvium-types", "alluvium-server", DepKind::Normal),
                ("alluvium-raft", "alluvium-storage", DepKind::Normal),
                ("alluvium-server", "alluvium-testkit", DepKind::Normal),
                ("alluvium-testkit", "alluvium", DepKind::Dev),
            ],
        );
        let rules: Vec<EdgeRule> = check_graph(&graph, &[], Path::new("/nonexistent"))
            .into_iter()
            .filter_map(|violation| match violation {
                Violation::Edge { rule, .. } => Some(rule),
                _ => None,
            })
            .collect();
        assert_eq!(
            rules,
            [
                EdgeRule::Upward,
                EdgeRule::SameLayer,
                EdgeRule::DevCrateAsNormal,
                EdgeRule::DependsOnBinary
            ]
        );
        assert!(check(&graph).contains(
            &"alluvium-types (layer 0) -> alluvium-server (layer 4): a crate may depend only on lower layers"
                .to_owned()
        ));
    }

    #[test]
    fn an_unknown_member_is_reported() {
        let graph = graph(&["mystery"], &[], &[]);
        assert_eq!(
            check(&graph),
            [
                "crate mystery has no layer: add it to xtask/src/layers.rs and docs/architecture.md §4"
            ]
        );
    }

    #[test]
    fn runtime_crates_are_found_directly_and_three_hops_away_with_the_path() {
        let graph = graph(
            &["alluvium-records", "alluvium-raft", "alluvium-network"],
            &["a", "b", "c", "tokio", "mio"],
            &[
                ("alluvium-records", "a", DepKind::Normal),
                ("a", "b", DepKind::Normal),
                ("b", "c", DepKind::Build),
                ("c", "tokio", DepKind::Normal),
                ("alluvium-raft", "mio", DepKind::Normal),
                ("alluvium-network", "tokio", DepKind::Normal),
            ],
        );
        let found = check(&graph);
        assert!(found.contains(&"alluvium-records (layer 0) -> a -> b -> c -> tokio: layers 0-2 must not depend on an async runtime or networking crate (async runtime)".to_owned()), "{found:?}");
        assert!(
            found
                .iter()
                .any(|line| line.starts_with("alluvium-raft (layer 1) -> mio:")),
            "{found:?}"
        );
        assert_eq!(found.len(), 2, "layer 3 may use tokio: {found:?}");
    }

    #[test]
    fn dev_dependencies_on_runtime_crates_are_allowed_in_core_layers() {
        let graph = graph(
            &["alluvium-records"],
            &["tokio"],
            &[("alluvium-records", "tokio", DepKind::Dev)],
        );
        assert_eq!(check(&graph), Vec::<String>::new());
    }

    fn adr_dir(status: &str) -> Result<std::path::PathBuf, std::io::Error> {
        let dir = std::env::temp_dir().join(format!(
            "xtask-adr-{}-{}",
            std::process::id(),
            status.replace(' ', "-")
        ));
        std::fs::create_dir_all(&dir)?;
        std::fs::write(
            dir.join("0099-allow-raft-storage.md"),
            format!("# ADR-0099\n\n- **Status:** {status}\n"),
        )?;
        Ok(dir)
    }

    const ENTRY: AllowedEdge = AllowedEdge {
        from: "alluvium-raft",
        to: "alluvium-storage",
        adr: "0099",
    };

    fn same_layer_graph() -> Graph {
        graph(
            &["alluvium-raft", "alluvium-storage"],
            &[],
            &[("alluvium-raft", "alluvium-storage", DepKind::Normal)],
        )
    }

    #[test]
    fn an_allowed_same_layer_edge_with_an_accepted_adr_passes() -> Result<(), std::io::Error> {
        let dir = adr_dir("Accepted")?;
        assert_eq!(check_graph(&same_layer_graph(), &[ENTRY], &dir), []);
        Ok(())
    }

    #[test]
    fn allow_list_entries_need_an_existing_accepted_adr_and_a_real_edge()
    -> Result<(), std::io::Error> {
        let proposed = adr_dir("Proposed")?;
        assert!(matches!(
            check_graph(&same_layer_graph(), &[ENTRY], &proposed)[..],
            [Violation::AllowListAdrNotAccepted { .. }]
        ));
        assert!(matches!(
            check_graph(&same_layer_graph(), &[ENTRY], Path::new("/nonexistent"))[..],
            [Violation::AllowListAdrMissing { .. }]
        ));
        let accepted = adr_dir("Accepted")?;
        let unused = check_graph(&graph(&["alluvium-raft"], &[], &[]), &[ENTRY], &accepted);
        assert!(
            matches!(unused[..], [Violation::AllowListUnused { .. }]),
            "{unused:?}"
        );
        Ok(())
    }

    #[test]
    fn the_real_architecture_doc_matches_the_layer_table() {
        let text = include_str!("../../docs/architecture.md");
        assert_eq!(check_architecture(text), []);
    }

    #[test]
    fn a_crate_moved_in_only_one_place_is_reported_both_ways() {
        let text = include_str!("../../docs/architecture.md")
            .replace("| 1 | `alluvium-raft` |", "| 2 | `alluvium-raft` |")
            .replace("| 0 | `alluvium-records` |", "| 0 | `alluvium-codecs` |");
        let problems: Vec<String> = check_architecture(&text)
            .iter()
            .map(ToString::to_string)
            .collect();
        let expected = [
            "alluvium-raft is layer 1 in xtask but layer 2 in architecture.md",
            "alluvium-records is in xtask but not in architecture.md",
            "alluvium-codecs is in architecture.md but not in xtask",
        ];
        for problem in expected {
            assert!(
                problems.iter().any(|line| line.ends_with(problem)),
                "{problems:?}"
            );
        }
        assert_eq!(problems.len(), 3, "{problems:?}");
    }
}
