//! Seeds for the generated fuzz targets, made from the golden vectors
//! (`tests/golden/protocol/messages/`, PROTO-F10) each time a target runs, so that every
//! target starts from Kafka's own bytes for its message without a copy of them in
//! `fuzz/seeds/` ([ADR-0018](../../../docs/adr/0018-fuzz-corpus-storage.md)).
//!
//! A target reads its message after an `int16` version (fuzz/src/lib.rs), so each
//! golden case becomes its version followed by its bytes. The request header target
//! reads a header as a broker does, so its seeds are the requests of `headers.json`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{FUZZ_DIR, create_dir};
use crate::codegen::names;

/// The golden vectors, relative to the repository root.
const GOLDEN_DIR: &str = "tests/golden/protocol/messages";

/// The target that reads request headers, whose seeds are whole headers.
const REQUEST_HEADER_TARGET: &str = "protocol_request_header";

/// Writes `target`'s seeds to `fuzz/target/golden-seeds/<target>/`, replacing any there,
/// and returns that directory relative to the repository root, or `None` if no golden
/// case is for the target.
pub(super) fn write(root: &Path, target: &str) -> Result<Option<PathBuf>, String> {
    let seeds = inputs(root, target)?;
    if seeds.is_empty() {
        return Ok(None);
    }
    let dir = Path::new(FUZZ_DIR)
        .join("target")
        .join("golden-seeds")
        .join(target);
    let full = root.join(&dir);
    if full.exists() {
        std::fs::remove_dir_all(&full)
            .map_err(|error| format!("could not remove {}: {error}", full.display()))?;
    }
    create_dir(&full)?;
    for (index, seed) in seeds.iter().enumerate() {
        let path = full.join(format!("golden-{index:04}"));
        std::fs::write(&path, seed)
            .map_err(|error| format!("could not write {}: {error}", path.display()))?;
    }
    Ok(Some(dir))
}

/// `target`'s seeds, without duplicates.
fn inputs(root: &Path, target: &str) -> Result<BTreeSet<Vec<u8>>, String> {
    let golden = root.join(GOLDEN_DIR);
    if target == REQUEST_HEADER_TARGET {
        return json(&golden.join("headers.json"))?
            .iter()
            .map(|case| {
                let request = case.get("request").and_then(Value::as_str);
                request
                    .and_then(from_hex)
                    .ok_or_else(|| format!("headers.json: a case without a hex `request`: {case}"))
            })
            .collect();
    }
    let mut seeds = BTreeSet::new();
    for case in json(&golden.join("cases.json"))? {
        // Cases Kafka failed to write have no file.
        let (Some(message), Some(version), Some(file)) = (
            case.get("message").and_then(Value::as_str),
            case.get("version").and_then(Value::as_i64),
            case.get("file").and_then(Value::as_str),
        ) else {
            continue;
        };
        if format!("protocol_{}", names::identifier(message)) != target {
            continue;
        }
        let version = i16::try_from(version)
            .map_err(|_| format!("cases.json: {message} has version {version}"))?;
        let path = golden.join(file);
        let body = std::fs::read(&path)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        let mut seed = version.to_be_bytes().to_vec();
        seed.extend(body);
        seeds.insert(seed);
    }
    Ok(seeds)
}

fn json(path: &Path) -> Result<Vec<Value>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    match serde_json::from_str(&text) {
        Ok(Value::Array(cases)) => Ok(cases),
        Ok(_) => Err(format!("{} is not a JSON array", path.display())),
        Err(error) => Err(format!("could not parse {}: {error}", path.display())),
    }
}

/// The bytes a string of hex digit pairs spells, if it is one.
fn from_hex(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|start| u8::from_str_radix(hex.get(start..start.checked_add(2)?)?, 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{from_hex, inputs};

    #[test]
    fn every_seed_of_a_message_target_starts_with_a_version_it_has() {
        let seeds = inputs(&crate::repo_root(), "protocol_produce_request").unwrap();
        assert!(seeds.len() > 10, "{} seeds", seeds.len());
        for seed in &seeds {
            let version = i16::from_be_bytes([seed[0], seed[1]]);
            assert!((3..=13).contains(&version), "version {version}");
        }
    }

    #[test]
    fn request_header_seeds_are_the_golden_requests() {
        let seeds = inputs(&crate::repo_root(), "protocol_request_header").unwrap();
        assert!(seeds.contains(&from_hex("0000000300000003ffff").unwrap()));
    }

    #[test]
    fn a_target_without_golden_cases_has_no_seeds() {
        assert!(
            inputs(&crate::repo_root(), "types_uuid_parse")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn hex_reads_pairs_only() {
        assert_eq!(from_hex("00ff7A"), Some(vec![0, 255, 122]));
        assert_eq!(from_hex("0"), None);
        assert_eq!(from_hex("zz"), None);
    }
}
