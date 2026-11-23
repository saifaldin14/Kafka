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

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Exit;
use crate::cli::GoldenDump;
use crate::install;
use crate::output;

/// Where the `errors` dump goes, relative to the repository root.
pub(crate) const ERRORS_GOLDEN: &str = "xtask/tests/fixtures/kafka-4.4.0-errors.json";

/// Where the `api-keys` dump goes, relative to the repository root. The dump replaces
/// the directory's contents.
pub(crate) const API_KEYS_GOLDEN: &str = "tests/golden/protocol/registry";

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

/// The Kafka client, and slf4j-api, the version its 4.4.0 POM depends on, which
/// `Errors` needs when it loads.
const JARS: [Jar; 2] = [
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
];

/// The JDK image the harness runs in: `eclipse-temurin:21-jdk`, pinned by digest.
const IMAGE: &str =
    "eclipse-temurin@sha256:8ad118b146aa92da815b576441acd6b42b38236797ad0b6edc8c200b85bb0734";

pub(crate) fn run(dump: GoldenDump, out: &mut impl Write) -> Exit {
    let root = crate::repo_root();
    let result = match dump {
        GoldenDump::Errors => errors(&root),
        GoldenDump::ApiKeys => api_keys(&root),
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
    let dir = harness(root, GoldenDump::Errors)?;
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
    let dir = harness(root, GoldenDump::ApiKeys)?;
    let count = replace_dir(&dir, &root.join(API_KEYS_GOLDEN))?;
    Ok(format!("wrote {count} files to {API_KEYS_GOLDEN}"))
}

/// Replaces the files in `to` with those in `from`, a flat directory, and returns how
/// many there are. Files the dump no longer writes are removed.
fn replace_dir(from: &Path, to: &Path) -> Result<usize, String> {
    match std::fs::remove_dir_all(to) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("could not remove {}: {error}", to.display())),
    }
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
        std::fs::copy(&path, to.join(name))
            .map_err(|error| format!("could not copy {}: {error}", path.display()))?;
        count = count.saturating_add(1);
    }
    Ok(count)
}

/// Runs `dump` in the harness and returns the directory it wrote.
fn harness(root: &Path, dump: GoldenDump) -> Result<PathBuf, String> {
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
    // javac compiles every source file; then the dump runs with its output directory.
    let script = "javac -d /work/classes -cp \"$CLASS_PATH\" $(find /harness -name '*.java') \
                  && java -cp \"/work/classes:$CLASS_PATH\" \"$0\" \"$1\" \"$2\"";
    let status = Command::new("docker")
        .args(["run", "--rm", "--network", "none", "--user", &user()?])
        .args(["--env", &format!("CLASS_PATH={}", class_path.join(":"))])
        .arg("--volume")
        .arg(format!("{}:/harness:ro", root.join(HARNESS).display()))
        .arg("--volume")
        .arg(format!("{}:/work", work.display()))
        .args([IMAGE, "sh", "-c", script, MAIN_CLASS, dump.name()])
        .arg(format!("/work/out/{}", dump.name()))
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
