//! `cargo xtask schemas dump-errors`: Kafka's error table as the Kafka 4.4.0 client jar
//! reports it at runtime, for the golden test of the generated `ErrorCode`.
//!
//! The code generator reads `Errors.java` as source text. This reads what Kafka's
//! compiled `Errors.values()` says, so a mistake in the generator's parsing can't hide:
//! `codegen::errors::tests::the_extracted_table_equals_kafkas_runtime_table` compares
//! the two.
//!
//! It downloads the client jar, and slf4j-api, which `Errors` needs when it loads, from
//! Maven Central into `target/errors-dump/`, and checks their pinned SHA-256s. Then it
//! runs `errors_dump.jsh` with `JShell` in a pinned JDK image under
//! Docker, without network access, and writes the JSON to [`GOLDEN`]. It needs the
//! network and Docker, so it never runs in CI; run it after vendoring a new Kafka version
//! (docs/guides/updating-kafka-schemas.md).

use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::Exit;
use crate::install;
use crate::output;

/// Where the dump is written, relative to the repository root.
pub(crate) const GOLDEN: &str = "xtask/tests/fixtures/kafka-4.4.0-errors.json";

/// A jar from Maven Central, pinned by SHA-256.
struct Jar {
    file: &'static str,
    url: &'static str,
    sha256: &'static str,
}

/// The Kafka client and the slf4j-api version its 4.4.0 POM depends on.
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

/// The JDK image `JShell` runs in: `eclipse-temurin:21-jdk`, pinned by digest.
const IMAGE: &str =
    "eclipse-temurin@sha256:8ad118b146aa92da815b576441acd6b42b38236797ad0b6edc8c200b85bb0734";

const SCRIPT: &str = include_str!("errors_dump.jsh");

pub(crate) fn run(out: &mut impl Write) -> Exit {
    match dump(&crate::repo_root()) {
        Ok(count) => {
            output::line(
                out,
                &format!("dump-errors: wrote {count} error codes to {GOLDEN}"),
            );
            Exit::Success
        }
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            Exit::CheckFailed
        }
    }
}

/// Writes the dump to [`GOLDEN`] and returns the number of codes in it.
fn dump(root: &Path) -> Result<usize, String> {
    let work = root.join("target").join("errors-dump");
    std::fs::create_dir_all(&work)
        .map_err(|error| format!("could not create {}: {error}", work.display()))?;
    for jar in &JARS {
        install::download(jar.url, &work.join(jar.file), jar.sha256)?;
    }
    std::fs::write(work.join("dump.jsh"), SCRIPT)
        .map_err(|error| format!("could not write the JShell script: {error}"))?;
    let class_path: Vec<String> = JARS
        .iter()
        .map(|jar| format!("/work/{}", jar.file))
        .collect();
    let status = Command::new("docker")
        .args(["run", "--rm", "--network", "none", "--volume"])
        .arg(format!("{}:/work", work.display()))
        .args([
            IMAGE,
            "jshell",
            "--class-path",
            &class_path.join(":"),
            "/work/dump.jsh",
        ])
        .status()
        .map_err(|error| format!("could not run docker: {error}"))?;
    if !status.success() {
        return Err(format!("JShell in docker failed: {status}"));
    }
    let json = std::fs::read_to_string(work.join("errors.json"))
        .map_err(|error| format!("JShell wrote no errors.json: {error}"))?;
    let count = serde_json::from_str::<serde_json::Value>(&json)
        .map_err(|error| format!("JShell wrote invalid JSON: {error}"))?
        .as_array()
        .map_or(0, Vec::len);
    std::fs::write(root.join(GOLDEN), json)
        .map_err(|error| format!("could not write {GOLDEN}: {error}"))?;
    Ok(count)
}
