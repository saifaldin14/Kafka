# Golden vectors

Alluvium's protocol code is checked against Kafka 4.4.0 itself, not against our reading of Kafka's source. A Java harness, [`tools/kafka-golden/`](../../tools/kafka-golden/src/alluvium/golden/Main.java), runs Kafka's own classes from the released `kafka-clients-4.4.0` jar and records what they do. The Rust tests compare the protocol crate with those recordings.

## What is recorded

`cargo xtask golden <dump>` downloads the jars, each checked against a pinned SHA-256. It compiles and runs the harness in a JDK image pinned by digest, under Docker, without network access. Then it moves the output into the repository. You need Docker and the network; no JVM is needed on the host.

| Command | Writes | Checked by |
|---|---|---|
| `golden errors` | `xtask/tests/fixtures/kafka-4.4.0-errors.json`: `Errors.values()` | the `ErrorCode` golden test in `xtask/src/codegen/errors.rs` |
| `golden api-keys` | `tests/golden/protocol/registry/`: the `ApiKeys` table and the API version lists Kafka's listeners advertise | `crates/alluvium-protocol/tests/registry_golden.rs`; codegen reads `clusterAction` and `forwardable` from it |
| `golden messages` | `tests/golden/protocol/messages/`: Kafka's bytes and JSON for every message, version and case, indexed by `cases.json`, and `utf8.json` | `golden_tests` in the protocol crate |
| `golden verify` | nothing; Kafka reads and rewrites 1000 arbitrary Alluvium-encoded messages per message and version | its own report |

Every file under `tests/golden/protocol/` is listed with its SHA-256 in `MANIFEST.sha256`. A test fails if any file differs from the manifest. A dump rewrites the manifest, but refuses to if a file outside its own directory changed. Golden files therefore change only by being recorded again.

### The message cases

Each message in the vendored schemas is recorded at each version. The cases are described in [`MessagesDump.java`](../../tools/kafka-golden/src/alluvium/golden/MessagesDump.java):

- **`default`:** every field at its default.
- **`populated`:** every field the version has, with values derived from the field's path. Rerunning writes the same bytes.
- **`nulls`:** every field nullable in the version is null.
- **`inner_nulls`:** only the innermost nullable fields are null, under present parents.
- **`unknown_tags`:** in flexible versions, unknown tagged fields at the root and one level down.
- **Downgrades:** each case built at the highest version and written at every lower one. Kafka either writes it (a `.bin` file) or throws (the exception and message, in `cases.json`).

Files are named `messages/<api>/<request|response>_v<N>_<case>.bin`, `messages/header/<header>_v<N>_<case>.bin` and `messages/data/<type>_v<N>_<case>.bin`. A downgrade adds `_as_v<M>`.

## Regenerating

Run the dumps after vendoring a new Kafka version ([updating the Kafka schemas](updating-kafka-schemas.md)), after changing the harness, or when the nightly job reports a difference:

```sh
cargo xtask golden errors
cargo xtask golden api-keys
cargo xtask golden messages
cargo xtask golden verify
```

Review the diff like any other change. A changed `.bin` file means Kafka writes different bytes. The `.json` beside it shows the same case as Kafka's `JsonConverter` prints it, which is easier to read.

The [Golden nightly](../../.github/workflows/golden-nightly.yml) workflow runs all four every night. When a committed file differs, it fails and uploads the diff as the `golden-diff` artifact.

## Adding a case

1. Add it to `MessagesDump.java` or a new dump class. The harness builds each message as JSON from the schema and reads it with the message's `JsonConverter`, so a new case is usually a new way of building that JSON (`Values.java`) or of changing the built message.
2. Run `cargo xtask golden messages` and commit the new files with the manifest.
3. Run the protocol crate's tests. `golden_tests` picks the new case up from `cases.json`.

A new dump also needs a `GoldenDump` variant in `xtask/src/cli.rs` and a function in [`xtask/src/golden.rs`](../../xtask/src/golden.rs) that moves its output into place.

## Reading a failure

`golden_tests` lists every failing case, one per line:

- **`ProduceRequest v9 populated: first difference at byte 37 of 120 (Rust wrote 121 bytes)`:** Alluvium decoded Kafka's bytes and wrote different ones. Decode the file at that version and compare the fields around the offset with the case's `.json`.
- **`... decode failed: ...` or `... decoded with N bytes left over`:** Alluvium couldn't read what Kafka wrote.
- **`... built at v13, written at v9: Kafka threw {...}, Rust returned ...`:** a downgrade failed differently. [codegen.md](../protocol/codegen.md#errors) pairs each `EncodeError` with Kafka's exception, including which error wins when several apply.

`golden verify` prints Kafka's first mismatches, each with the message, version and index of the arbitrary value, and the first differing byte or Kafka's exception. The values come from a fixed seed. To reproduce one, run `write_messages_for_the_reverse_check` and read entry `index` of `target/kafka-golden/rust/<Message>_v<N>.bin`.
