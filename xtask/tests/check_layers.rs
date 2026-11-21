//! `cargo xtask check-layers` against offline fixture workspaces, one per rule
//! (FND-0029). Each fixture is its own `[workspace]` root with path dependencies only,
//! so the tests run real `cargo metadata` without network access.

use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

/// Runs check-layers on `fixture` and returns the exit code and stdout lines.
fn check_layers(fixture: &str) -> Result<(Option<i32>, Vec<String>), Box<dyn Error>> {
    let manifest: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "fixtures",
        "layers",
        fixture,
        "Cargo.toml",
    ]
    .iter()
    .collect();
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["check-layers", "--manifest-path"])
        .arg(manifest)
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok((
        output.status.code(),
        stdout.lines().map(str::to_owned).collect(),
    ))
}

fn assert_rejected(fixture: &str, violation: &str) -> Result<(), Box<dyn Error>> {
    let (code, lines) = check_layers(fixture)?;
    assert_eq!(code, Some(1), "{fixture}: {lines:?}");
    assert_eq!(
        lines,
        [violation, "check-layers: 1 violation(s)"],
        "{fixture}"
    );
    Ok(())
}

#[test]
fn a_correctly_layered_workspace_passes() -> Result<(), Box<dyn Error>> {
    let (code, lines) = check_layers("passing")?;
    assert_eq!(code, Some(0), "{lines:?}");
    assert_eq!(lines, ["check-layers: ok"]);
    Ok(())
}

#[test]
fn an_upward_dependency_is_rejected() -> Result<(), Box<dyn Error>> {
    assert_rejected(
        "upward",
        "alluvium-types (layer 0) -> alluvium-server (layer 4): a crate may depend only on lower layers",
    )
}

#[test]
fn a_same_layer_dependency_is_rejected() -> Result<(), Box<dyn Error>> {
    assert_rejected(
        "same-layer",
        "alluvium-raft (layer 1) -> alluvium-storage (layer 1): same-layer dependencies need an allow-list entry backed by an accepted ADR",
    )
}

#[test]
fn an_async_runtime_under_a_core_crate_is_rejected_with_its_path() -> Result<(), Box<dyn Error>> {
    assert_rejected(
        "runtime-in-core",
        "alluvium-records (layer 0) -> stub -> tokio: layers 0-2 must not depend on an async runtime or networking crate (async runtime)",
    )
}

#[test]
fn a_dev_crate_used_as_a_normal_dependency_is_rejected() -> Result<(), Box<dyn Error>> {
    assert_rejected(
        "dev-as-normal",
        "alluvium-server (layer 4) -> alluvium-testkit (layer dev): dev crates may only be dev-dependencies",
    )
}

#[test]
fn a_crate_missing_from_the_layer_table_is_rejected() -> Result<(), Box<dyn Error>> {
    assert_rejected(
        "unknown-crate",
        "crate mystery has no layer: add it to xtask/src/layers.rs and docs/architecture.md §4",
    )
}
