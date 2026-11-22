//! Every exception to clippy's `disallowed-methods` in the workspace, listed and reviewed.
//!
//! Each banned function in `clippy.toml` has at most one sanctioned wrapper, and the
//! wrapper carries one `#[expect(clippy::disallowed_methods, reason = "…")]`
//! (docs/standards/rust.md#workspace-configuration). This test finds every such
//! attribute, so a new exception fails here until its ADR allows it and it is added to
//! [`SANCTIONED`]. The lint canary's fixtures break the lints on purpose and are
//! skipped.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// `(file, number of exceptions in it, what they are)`.
const SANCTIONED: &[(&str, usize, &str)] = &[
    (
        "crates/alluvium-types/src/clock.rs",
        2,
        "SystemClock: Instant::now and SystemTime::now (ADR-0011)",
    ),
    (
        "crates/alluvium-types/src/rng.rs",
        1,
        "SeededRng::from_os_entropy: getrandom::fill (ADR-0012)",
    ),
    (
        "xtask/src/clock.rs",
        2,
        "xtask's Stopwatch, the dev-tool exception: Instant::now and Instant::elapsed (ADR-0011)",
    ),
];

const SCANNED: &[&str] = &["crates", "xtask/src", "xtask/tests"];

/// Built with `concat!` so that this file doesn't match itself.
const LINT: &str = concat!("clippy::", "disallowed_methods");

#[test]
fn disallowed_method_exceptions_are_only_the_sanctioned_ones() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask has no parent directory")?;
    let mut found = BTreeMap::new();
    for dir in SCANNED {
        for file in rust_files(&root.join(dir))? {
            let count = fs::read_to_string(&file)?
                .lines()
                .filter(|line| is_exception(line))
                .count();
            if count > 0 {
                let relative = file
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .replace('\\', "/");
                found.insert(relative, count);
            }
        }
    }
    let expected: BTreeMap<String, usize> = SANCTIONED
        .iter()
        .map(|(file, count, _)| ((*file).to_owned(), *count))
        .collect();
    assert_eq!(
        found, expected,
        "a {LINT} exception was added or removed; each one needs an ADR that sanctions it \
         and an entry in SANCTIONED"
    );
    Ok(())
}

/// An `#[expect(…)]` naming the lint, on one line or as rustfmt splits it over several.
fn is_exception(line: &str) -> bool {
    let code = line.trim_start();
    !code.starts_with("//") && (code.starts_with(LINT) || code.contains(&format!("expect({LINT}")))
}

fn rust_files(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    Ok(files)
}
