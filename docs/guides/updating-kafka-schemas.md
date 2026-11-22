# Updating the vendored Kafka schemas

Alluvium's protocol code is generated from Kafka's JSON schemas, copied unmodified into `schemas/kafka-<tag>/` ([ADR-0008](../adr/0008-protocol-code-generated-from-kafka-schemas.md)). This guide covers the two reasons to touch them, and what each one takes.

## Never edit a vendored file

A vendored file only ever changes by being copied again from a Kafka tag. The `schemas` step of `cargo xtask ci` (`cargo xtask schemas verify`) fails on any file that differs from its `MANIFEST.sha256`, and on any file added or removed by hand. Behavior that differs from Kafka goes into Alluvium's code with an ADR, never into a schema. The only files under `schemas/kafka-<tag>/` that are Alluvium's are its `README.md` and the manifest.

## When the vendored files need to change

| Situation | What to do | ADR needed |
|---|---|---|
| A schema directory was missed, or Kafka's files at the same tag must be re-copied | Re-run `vendor` for the same tag (below). The tag's commit must not change. | No |
| Moving to a newer Kafka version | Vendor the new tag into its own directory (below), then follow [moving to a new Kafka version](#moving-to-a-new-kafka-version) | Yes: the target version is set by [ADR-0001](../adr/0001-kafka-4-4-compatibility-is-the-parity-contract.md), and only a new ADR changes it |

## Vendoring a tag

1. Preview the change. This needs the network:
   ```sh
   cargo xtask schemas vendor --tag <tag> --dry-run
   ```
   It sparse-clones apache/kafka at the tag into `target/schemas-vendor/` and prints the tag's commit and every file that would be added, removed or changed. For the tag already vendored, it should print `no files change`. Anything else, such as `the tag now points at …`, means the tag moved upstream: stop and find out why before going further.
2. Vendor it:
   ```sh
   cargo xtask schemas vendor --tag <tag>
   ```
   This copies the directories listed in `SOURCES` in [`xtask/src/schemas.rs`](../../xtask/src/schemas.rs), with Kafka's `LICENSE` and `NOTICE`, and writes `MANIFEST.sha256`. A schema directory Kafka adds in a new version has to be added to `SOURCES` first. The directory's `README.md` is kept as it is.
3. For a new tag, write `schemas/kafka-<tag>/README.md` from the previous version's README: the tag, its commit (the first line of the manifest), and the source of each directory. Add the directory to Alluvium's [`NOTICE`](../../NOTICE), as for any Kafka material ([attributing Kafka material](../../CONTRIBUTING.md#attributing-kafka-material)).
4. Run `cargo xtask schemas verify`, then `cargo xtask ci`.

Vendoring is its own commit, with no other change, so the review sees exactly what Kafka changed.

## Moving to a new Kafka version

After vendoring the new tag, in this order:

1. **Write the ADR** that supersedes ADR-0001 as the compatibility target. It lists the APIs and versions that change, as found in the steps below.
2. **Regenerate and review the generated diff.** Point `cargo xtask codegen` at the new directory and read the diff of the generated code: every new field, version, tagged field and API shows up there. `cargo xtask codegen --check` (PROTO-0138) fails CI until the committed code matches. Changes to a schema's model, such as a new property, fail in the schema parser first, naming the file.
3. **Regenerate the golden vectors** from the new version's `kafka-clients` jar with the golden-vector harness (PROTO-F10). A vector that changes is a wire change to read carefully.
4. **Update [docs/parity.md](../parity.md)**: its API table comes from the request schemas, so new APIs, new version ranges and changed listeners appear there, each linked to a backlog feature. New work goes into the [backlog](../backlog/README.md#adding-work).
5. **Remove the old directory** once nothing reads it.

Codegen and the golden-vector harness are being built in milestone M1 (PROTO-F04 onward and PROTO-F10). Until they exist, steps 2 and 3 don't apply.
