//! `cargo xtask golden <dump>`: records what Kafka 4.4.0's own classes do, as golden
//! files Alluvium's tests compare against.
//!
//! The harness is Java, in `tools/kafka-golden/`. This downloads the Kafka client jar
//! and the runtime dependencies the harness needs from Maven Central into
//! `target/kafka-golden/`, and checks each one's pinned SHA-256. It then compiles and
//! runs the harness in a JDK image pinned by digest, under Docker, without network
//! access and as the calling user. The harness writes into `target/kafka-golden/out/`,
//! and this moves the result into the repository. Nothing needs a JVM on the host.
//!
//! It needs the network and Docker, so it never runs in CI's per-PR jobs. Run it after
//! vendoring a new Kafka version (docs/guides/updating-kafka-schemas.md).
//!
//! The dumps:
//! - `errors`: `Errors.values()`, the golden table for the generated `ErrorCode`.
//!   The code generator reads `Errors.java` as source text; this reads what the compiled
//!   class says, so a mistake in the generator's parsing can't hide.
//! - `api-keys`: the `ApiKeys` registry, with each API's versions, listeners, flags and
//!   header versions, and the API version lists Kafka's broker and controller listeners
//!   advertise, as encoded `ApiVersionsResponse`s. Written to [`API_KEYS_GOLDEN`] for the
//!   generated `ApiKey` and the version negotiation tests.
//! - `messages`: Kafka's bytes, and its `JsonConverter` JSON, for every message in the
//!   vendored schemas at every version, in several cases, including downgrades Kafka
//!   refuses; and its decoding of invalid UTF-8. Written to [`MESSAGES_GOLDEN`]
//!   (tools/kafka-golden/src/alluvium/golden/MessagesDump.java describes the cases).
//!
//! The dumps under [`GOLDEN`] are listed with their SHA-256s in its `MANIFEST.sha256`,
//! rewritten after each dump and checked by a test, so a golden file edited by hand, or
//! left behind, fails.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Exit;
use crate::cli::GoldenDump;
use crate::install;
use crate::output;
use crate::schemas;

/// Where the `errors` dump goes, relative to the repository root.
pub(crate) const ERRORS_GOLDEN: &str = "xtask/tests/fixtures/kafka-4.4.0-errors.json";

/// The protocol golden files, relative to the repository root.
pub(crate) const GOLDEN: &str = "tests/golden/protocol";

/// Where the `api-keys` dump goes, relative to the repository root. The dump replaces
/// the directory's contents.
pub(crate) const API_KEYS_GOLDEN: &str = "tests/golden/protocol/registry";

/// Where the `messages` dump goes, relative to the repository root. The dump replaces
/// the directory's contents.
pub(crate) const MESSAGES_GOLDEN: &str = "tests/golden/protocol/messages";

/// The vendored Kafka directory the `messages` dump reads the schemas of.
const VENDORED: &str = "schemas/kafka-4.4.0";

/// The harness sources, relative to the repository root.
const HARNESS: &str = "tools/kafka-golden/src";

/// The class whose `main` runs a dump.
const MAIN_CLASS: &str = "alluvium.golden.Main";

/// A jar from Maven Central, pinned by SHA-256.
struct Jar {
    file: &'static str,
    url: &'static str,
    sha256: &'static str,
}

/// The Kafka client; slf4j-api, the version its 4.4.0 POM depends on, which `Errors`
/// needs when it loads; and Jackson, at the versions Kafka 4.4.0 builds with, which the
/// generated `JsonConverter`s use and the 4.4.0 POM doesn't declare.
const JARS: [Jar; 5] = [
    Jar {
        file: "kafka-clients-4.4.0.jar",
        url: "https://repo1.maven.org/maven2/org/apache/kafka/kafka-clients/4.4.0/kafka-clients-4.4.0.jar",
        sha256: "1ab07317eb6633241392449d329dbdc5d9251e6ffc4926860f42dd966e88bdf9",
    },
    Jar {
        file: "slf4j-api-1.7.36.jar",
        url: "https://repo1.maven.org/maven2/org/slf4j/slf4j-api/1.7.36/slf4j-api-1.7.36.jar",
        sha256: "d3ef575e3e4979678dc01bf1dcce51021493b4d11fb7f1be8ad982877c16a1c0",
    },
    Jar {
        file: "jackson-databind-2.21.7.jar",
        url: "https://repo1.maven.org/maven2/com/fasterxml/jackson/core/jackson-databind/2.21.7/jackson-databind-2.21.7.jar",
        sha256: "1290c2795e93e8a6861a6c4d9ff0d844d32f5ea178362cb2721edf5561e828b1",
    },
    Jar {
        file: "jackson-core-2.21.7.jar",
        url: "https://repo1.maven.org/maven2/com/fasterxml/jackson/core/jackson-core/2.21.7/jackson-core-2.21.7.jar",
        sha256: "8c5623b98f32d5f7e1287ff61ed50e08ab7b11381e991fff89327faabba42261",
    },
    Jar {
        file: "jackson-annotations-2.21.jar",
        url: "https://repo1.maven.org/maven2/com/fasterxml/jackson/core/jackson-annotations/2.21/jackson-annotations-2.21.jar",
        sha256: "53ca085f4a150f703f49e1aabd935bd03b43e1ea3d55d135438292af22cef56b",
    },
];

/// The JDK image the harness runs in: `eclipse-temurin:21-jdk`, pinned by digest.
const IMAGE: &str =
    "eclipse-temurin@sha256:8ad118b146aa92da815b576441acd6b42b38236797ad0b6edc8c200b85bb0734";

pub(crate) fn run(dump: GoldenDump, out: &mut impl Write) -> Exit {
    let root = crate::repo_root();
    let result = match dump {
        GoldenDump::Errors => errors(&root),
        GoldenDump::ApiKeys => api_keys(&root),
        GoldenDump::Messages => messages(&root),
    };
    match result {
        Ok(summary) => {
            output::line(out, &format!("golden {}: {summary}", dump.name()));
            Exit::Success
        }
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            Exit::CheckFailed
        }
    }
}

/// The `errors` dump, written to [`ERRORS_GOLDEN`].
fn errors(root: &Path) -> Result<String, String> {
    let dir = harness(root, GoldenDump::Errors, &[])?;
    let json = read(&dir.join("errors.json"))?;
    let count = serde_json::from_str::<serde_json::Value>(&json)
        .map_err(|error| format!("the harness wrote invalid JSON: {error}"))?
        .as_array()
        .map_or(0, Vec::len);
    std::fs::write(root.join(ERRORS_GOLDEN), json)
        .map_err(|error| format!("could not write {ERRORS_GOLDEN}: {error}"))?;
    Ok(format!("wrote {count} error codes to {ERRORS_GOLDEN}"))
}

/// The `api-keys` dump, written to [`API_KEYS_GOLDEN`].
fn api_keys(root: &Path) -> Result<String, String> {
    let dir = harness(root, GoldenDump::ApiKeys, &[])?;
    let count = replace_dir(&dir, &root.join(API_KEYS_GOLDEN))?;
    write_manifest(root, API_KEYS_GOLDEN)?;
    Ok(format!("wrote {count} files to {API_KEYS_GOLDEN}"))
}

/// The `messages` dump, written to [`MESSAGES_GOLDEN`].
fn messages(root: &Path) -> Result<String, String> {
    let dir = harness(root, GoldenDump::Messages, &["/kafka/clients"])?;
    let count = replace_dir(&dir, &root.join(MESSAGES_GOLDEN))?;
    write_manifest(root, MESSAGES_GOLDEN)?;
    Ok(format!("wrote {count} files to {MESSAGES_GOLDEN}"))
}

/// Replaces the files under `to` with those under `from`, and returns how many there
/// are. Files the dump no longer writes are removed.
fn replace_dir(from: &Path, to: &Path) -> Result<usize, String> {
    match std::fs::remove_dir_all(to) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("could not remove {}: {error}", to.display())),
    }
    copy_tree(from, to)
}

fn copy_tree(from: &Path, to: &Path) -> Result<usize, String> {
    std::fs::create_dir_all(to)
        .map_err(|error| format!("could not create {}: {error}", to.display()))?;
    let entries = std::fs::read_dir(from)
        .map_err(|error| format!("the harness wrote no {}: {error}", from.display()))?;
    let mut count = 0_usize;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("could not read {}: {error}", from.display()))?
            .path();
        let Some(name) = path.file_name() else {
            continue;
        };
        if path.is_dir() {
            count = count.saturating_add(copy_tree(&path, &to.join(name))?);
        } else {
            std::fs::copy(&path, to.join(name))
                .map_err(|error| format!("could not copy {}: {error}", path.display()))?;
            count = count.saturating_add(1);
        }
    }
    Ok(count)
}

/// Rewrites [`GOLDEN`]'s manifest after a dump replaced `written`, a directory under it:
/// every file with its SHA-256, and the Kafka version and commit of the vendored schemas
/// the dumps were made with.
///
/// Files outside `written` must still match the manifest, so a hand edit elsewhere fails
/// here rather than being listed as if Kafka had written it.
fn write_manifest(root: &Path, written: &str) -> Result<(), String> {
    let manifest = expected_manifest(root)?;
    let path = root.join(GOLDEN).join(schemas::MANIFEST);
    if let Ok(text) = std::fs::read_to_string(&path) {
        let previous = schemas::Manifest::parse(&text)?;
        let prefix = format!(
            "{}/",
            written
                .strip_prefix(GOLDEN)
                .unwrap_or(written)
                .trim_start_matches('/')
        );
        let elsewhere: Vec<_> = schemas::compare(&previous.hashes, &manifest.hashes)
            .into_iter()
            .filter(|change| {
                let (schemas::Change::Added(file)
                | schemas::Change::Removed(file)
                | schemas::Change::Modified(file)) = change;
                !file.starts_with(&prefix)
            })
            .collect();
        if !elsewhere.is_empty() {
            return Err(format!(
                "files outside {written} differ from {GOLDEN}/{}; golden files change only \
                 through their dump: {elsewhere:?}",
                schemas::MANIFEST
            ));
        }
    }
    std::fs::write(&path, manifest.render_by(MANIFEST_TOOLS))
        .map_err(|error| format!("could not write {}: {error}", path.display()))
}

/// What wrote and what checks the golden manifest.
const MANIFEST_TOOLS: &str = "`cargo xtask golden`; checked by `golden::tests`";

/// The manifest [`GOLDEN`] should have: its files as they are, with the vendored
/// schemas' Kafka version.
fn expected_manifest(root: &Path) -> Result<schemas::Manifest, String> {
    let vendored = root.join(VENDORED).join(schemas::MANIFEST);
    let text = std::fs::read_to_string(&vendored)
        .map_err(|error| format!("could not read {}: {error}", vendored.display()))?;
    let source = schemas::Manifest::parse(&text)?;
    Ok(schemas::Manifest {
        hashes: schemas::hash_tree(&root.join(GOLDEN))?,
        ..source
    })
}

/// Runs `dump` in the harness, with `inputs` (paths in the container) after its output
/// directory, and returns the directory it wrote. The vendored Kafka directory is
/// mounted read-only at `/kafka`.
fn harness(root: &Path, dump: GoldenDump, inputs: &[&str]) -> Result<PathBuf, String> {
    let work = root.join("target").join("kafka-golden");
    let out = work.join("out").join(dump.name());
    let classes = work.join("classes");
    for stale in [&out, &classes] {
        match std::fs::remove_dir_all(stale) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("could not remove {}: {error}", stale.display())),
        }
    }
    let jars = work.join("jars");
    std::fs::create_dir_all(&jars)
        .map_err(|error| format!("could not create {}: {error}", jars.display()))?;
    for jar in &JARS {
        install::download(jar.url, &jars.join(jar.file), jar.sha256)?;
    }
    let class_path: Vec<String> = JARS
        .iter()
        .map(|jar| format!("/work/jars/{}", jar.file))
        .collect();
    // javac compiles every source file; then the dump runs with its arguments.
    let script = "javac -d /work/classes -cp \"$CLASS_PATH\" $(find /harness -name '*.java') \
                  && java -cp \"/work/classes:$CLASS_PATH\" \"$@\"";
    let status = Command::new("docker")
        .args(["run", "--rm", "--network", "none", "--user", &user()?])
        .args(["--env", &format!("CLASS_PATH={}", class_path.join(":"))])
        .arg("--volume")
        .arg(format!("{}:/harness:ro", root.join(HARNESS).display()))
        .arg("--volume")
        .arg(format!("{}:/kafka:ro", root.join(VENDORED).display()))
        .arg("--volume")
        .arg(format!("{}:/work", work.display()))
        .args([IMAGE, "sh", "-c", script, "sh", MAIN_CLASS, dump.name()])
        .arg(format!("/work/out/{}", dump.name()))
        .args(inputs)
        .status()
        .map_err(|error| format!("could not run docker: {error}"))?;
    if !status.success() {
        return Err(format!("the harness failed in docker: {status}"));
    }
    Ok(out)
}

/// `uid:gid` of the calling user, so the files the container writes are theirs.
fn user() -> Result<String, String> {
    let id = |flag: &str| {
        let output = Command::new("id")
            .arg(flag)
            .output()
            .map_err(|error| format!("could not run id: {error}"))?;
        Ok::<_, String>(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    };
    Ok(format!("{}:{}", id("-u")?, id("-g")?))
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|error| format!("the harness wrote no {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{GOLDEN, expected_manifest};
    use crate::schemas::{self, Manifest};

    /// Golden files change only through `cargo xtask golden`, which rewrites the manifest.
    #[test]
    fn the_golden_files_match_their_manifest() {
        let root = crate::repo_root();
        let path = root.join(GOLDEN).join(schemas::MANIFEST);
        let committed = Manifest::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
        let expected = expected_manifest(&root).unwrap();
        let changes = schemas::compare(&committed.hashes, &expected.hashes);
        assert!(
            changes.is_empty(),
            "golden files differ from {GOLDEN}/MANIFEST.sha256; regenerate them with \
             `cargo xtask golden`: {changes:?}"
        );
        assert_eq!(
            (committed.tag, committed.commit),
            (expected.tag, expected.commit)
        );
    }
}
