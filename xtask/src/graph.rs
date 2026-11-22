//! The crate dependency graph, loaded from `cargo metadata`.
//!
//! Only the fields the checks need are read: workspace members, package names and
//! manifest paths, and resolved edges with their dependency kinds for the layer and
//! manifest checks; package licenses for `deps-diff`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use crate::tools;

/// How one package depends on another. An edge can have several kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DepKind {
    Normal,
    Build,
    Dev,
}

/// A resolved dependency edge between two packages, by package id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Edge {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) kinds: BTreeSet<DepKind>,
}

impl Edge {
    /// True if the dependency is used outside tests (normal or build).
    pub(crate) fn is_non_dev(&self) -> bool {
        self.kinds.iter().any(|kind| *kind != DepKind::Dev)
    }
}

/// The resolved dependency graph of a workspace.
#[derive(Debug, Clone, Default)]
pub(crate) struct Graph {
    /// Package id to package name, for every package in the resolve.
    pub(crate) names: BTreeMap<String, String>,
    /// Package id to the absolute path of its `Cargo.toml`.
    pub(crate) manifest_paths: BTreeMap<String, PathBuf>,
    /// Package ids of the workspace members.
    pub(crate) members: BTreeSet<String>,
    /// The SPDX license expression of each package that declares one, by name and version.
    pub(crate) licenses: BTreeMap<(String, String), String>,
    pub(crate) edges: Vec<Edge>,
}

impl Graph {
    pub(crate) fn name<'a>(&'a self, id: &'a str) -> &'a str {
        self.names.get(id).map_or(id, String::as_str)
    }
}

/// Why the graph could not be loaded.
#[derive(Debug)]
pub(crate) enum GraphError {
    CouldNotRun(std::io::Error),
    CargoFailed { stderr: String },
    InvalidJson(serde_json::Error),
    MissingField(&'static str),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::CouldNotRun(error) => write!(f, "could not run `cargo metadata`: {error}"),
            GraphError::CargoFailed { stderr } => {
                write!(f, "`cargo metadata` failed:\n{}", stderr.trim_end())
            }
            GraphError::InvalidJson(error) => {
                write!(f, "`cargo metadata` printed invalid JSON: {error}")
            }
            GraphError::MissingField(field) => {
                write!(f, "`cargo metadata` output has no `{field}` field")
            }
        }
    }
}

/// Runs `cargo metadata --format-version 1 --locked`, for `manifest_path` if given.
pub(crate) fn load(manifest_path: Option<&Path>) -> Result<Graph, GraphError> {
    let mut command = Command::new(tools::cargo_program());
    command.args(["metadata", "--format-version", "1", "--locked"]);
    if let Some(path) = manifest_path {
        command.arg("--manifest-path").arg(path);
    }
    let output = command.output().map_err(GraphError::CouldNotRun)?;
    if !output.status.success() {
        return Err(GraphError::CargoFailed {
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    parse(&output.stdout)
}

/// Parses `cargo metadata --format-version 1` JSON.
pub(crate) fn parse(json: &[u8]) -> Result<Graph, GraphError> {
    let root: Value = serde_json::from_slice(json).map_err(GraphError::InvalidJson)?;
    let mut graph = Graph::default();
    for package in array(&root, "packages")? {
        let id = string(package, "id")?;
        graph
            .names
            .insert(id.to_owned(), string(package, "name")?.to_owned());
        graph
            .manifest_paths
            .insert(id.to_owned(), string(package, "manifest_path")?.into());
        if let Some(license) = package.get("license").and_then(Value::as_str) {
            graph.licenses.insert(
                (
                    string(package, "name")?.to_owned(),
                    string(package, "version")?.to_owned(),
                ),
                license.to_owned(),
            );
        }
    }
    for member in array(&root, "workspace_members")? {
        let id = member
            .as_str()
            .ok_or(GraphError::MissingField("workspace_members"))?;
        graph.members.insert(id.to_owned());
    }
    let resolve = root
        .get("resolve")
        .ok_or(GraphError::MissingField("resolve"))?;
    for node in array(resolve, "nodes")? {
        let from = string(node, "id")?;
        for dep in array(node, "deps")? {
            let kinds = array(dep, "dep_kinds")?
                .iter()
                .map(|kind| match kind.get("kind").and_then(Value::as_str) {
                    Some("dev") => DepKind::Dev,
                    Some("build") => DepKind::Build,
                    _ => DepKind::Normal,
                })
                .collect();
            graph.edges.push(Edge {
                from: from.to_owned(),
                to: string(dep, "pkg")?.to_owned(),
                kinds,
            });
        }
    }
    Ok(graph)
}

fn array<'a>(value: &'a Value, field: &'static str) -> Result<&'a Vec<Value>, GraphError> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or(GraphError::MissingField(field))
}

fn string<'a>(value: &'a Value, field: &'static str) -> Result<&'a str, GraphError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or(GraphError::MissingField(field))
}

#[cfg(test)]
mod tests {
    use super::{DepKind, GraphError, parse};

    const FIXTURE: &str = include_str!("../tests/fixtures/metadata.json");

    #[test]
    fn parses_members_names_and_edge_kinds() -> Result<(), GraphError> {
        let graph = parse(FIXTURE.as_bytes())?;
        let members: Vec<&str> = graph.members.iter().map(|id| graph.name(id)).collect();
        assert_eq!(
            members,
            ["alluvium-records", "alluvium-testkit", "alluvium-types"]
        );
        let types_manifest = graph
            .members
            .iter()
            .find(|id| graph.name(id) == "alluvium-types")
            .and_then(|id| graph.manifest_paths.get(id));
        assert_eq!(
            types_manifest.map(|path| path.to_string_lossy().into_owned()),
            Some("/ws/types/Cargo.toml".to_owned())
        );
        let edge = |from: &str, to: &str| {
            graph
                .edges
                .iter()
                .find(|edge| graph.name(&edge.from) == from && graph.name(&edge.to) == to)
                .map(|edge| edge.kinds.iter().copied().collect::<Vec<_>>())
        };
        assert_eq!(
            edge("alluvium-records", "alluvium-types"),
            Some(vec![DepKind::Normal])
        );
        assert_eq!(
            edge("alluvium-records", "alluvium-testkit"),
            Some(vec![DepKind::Dev])
        );
        assert_eq!(
            edge("alluvium-records", "cc"),
            Some(vec![DepKind::Normal, DepKind::Build])
        );
        Ok(())
    }

    #[test]
    fn rejects_invalid_json_and_missing_fields() {
        assert!(matches!(parse(b"{"), Err(GraphError::InvalidJson(_))));
        assert!(matches!(
            parse(br#"{"packages": [], "workspace_members": []}"#),
            Err(GraphError::MissingField("resolve"))
        ));
    }

    #[test]
    fn licenses_are_read_by_name_and_version_when_declared() -> Result<(), GraphError> {
        let json = br#"{"packages": [
            {"id": "a", "name": "serde", "version": "1.0.0", "manifest_path": "/a/Cargo.toml", "license": "MIT OR Apache-2.0"},
            {"id": "b", "name": "app", "version": "0.1.0", "manifest_path": "/b/Cargo.toml", "license": null}
        ], "workspace_members": ["b"], "resolve": {"nodes": []}}"#;
        let graph = parse(json)?;
        assert_eq!(
            graph.licenses.into_iter().collect::<Vec<_>>(),
            [(
                ("serde".to_owned(), "1.0.0".to_owned()),
                "MIT OR Apache-2.0".to_owned()
            )]
        );
        Ok(())
    }
}
