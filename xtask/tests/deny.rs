//! The repository's deny.toml rejects banned crates, wildcard versions and disallowed
//! licenses (FND-0011, FND-0012). Each fixture is an offline workspace with path
//! dependencies only; the tests run the pinned cargo-deny (`cargo xtask tools install`).

use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

/// Runs `cargo deny check <check>` on `fixture` with the real deny.toml.
/// Returns the exit code and the diagnostics (stderr).
fn deny(fixture: &str, check: &str) -> Result<(Option<i32>, String), Box<dyn Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = root.join("tests/fixtures").join(fixture).join("Cargo.toml");
    let config = root.join("../deny.toml");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        // The assertions match plain text; CI sets CARGO_TERM_COLOR=always, which would
        // otherwise make cargo-deny wrap its diagnostics in ANSI colour codes.
        .args(["deny", "--frozen", "--color", "never", "--manifest-path"])
        .arg(manifest)
        .arg("--config")
        .arg(config)
        .args(["check", check])
        .output()?;
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        !stderr.contains("no such command"),
        "cargo-deny is not installed; run `cargo xtask tools install`"
    );
    Ok((output.status.code(), stderr))
}

#[test]
fn banned_tls_crates_are_rejected_with_the_reason() -> Result<(), Box<dyn Error>> {
    let (code, stderr) = deny("deny-bans", "bans")?;
    assert_ne!(code, Some(0), "{stderr}");
    assert!(
        stderr.contains("error[banned]: crate 'openssl-sys = 0.0.0' is explicitly banned"),
        "{stderr}"
    );
    assert!(stderr.contains("TLS is rustls only (ADR-0006)"), "{stderr}");
    Ok(())
}

#[test]
fn wildcard_versions_are_rejected() -> Result<(), Box<dyn Error>> {
    let (code, stderr) = deny("deny-bans", "bans")?;
    assert_ne!(code, Some(0), "{stderr}");
    assert!(
        stderr.contains("error[wildcard]: found 1 wildcard dependency for crate 'app'"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn licenses_outside_the_allow_list_are_rejected_by_name() -> Result<(), Box<dyn Error>> {
    let (code, stderr) = deny("deny-licenses", "licenses")?;
    assert_ne!(code, Some(0), "{stderr}");
    assert!(stderr.contains("error[rejected]"), "{stderr}");
    assert!(stderr.contains("GPL-3.0-only"), "{stderr}");
    Ok(())
}
