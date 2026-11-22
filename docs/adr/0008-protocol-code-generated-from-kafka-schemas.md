# ADR-0008: Protocol code generated from Kafka schemas

- **Status:** Accepted
- **Date:** 2026-11-20

## Context

Kafka 4.4.0 defines 91 active APIs, each with several versions, flexible versions and tagged fields. The definitions are JSON schemas in `clients/src/main/resources/common/message/`. Metadata records are defined in `metadata/src/main/resources/common/metadata/`. Kafka generates its Java message classes from these schemas. Hand-writing codecs for all of them would be slow and error-prone.

Generated code is often hard to debug, especially when produced by proc macros or `build.rs`, where it is invisible in the source tree.

Kafka is licensed under Apache-2.0. Alluvium currently has no license (all rights reserved). Apache-2.0 allows use in such projects, provided its conditions are followed.

## Decision

1. **Vendor the schemas** from Kafka tag `4.4.0` into `schemas/kafka-4.4.0/`, unmodified, together with Kafka's `LICENSE` and `NOTICE`.
2. **`cargo xtask codegen` generates Rust** types and codecs into `src/generated/` in `alluvium-protocol` and `alluvium-metadata`. The output is **committed**, formatted with rustfmt, and marked `@generated`. It is readable, greppable and debuggable, and it shows up in diffs.
3. **CI checks for drift** (`cargo xtask codegen --check`). Changes go through the generator, never by hand.
4. **The generated code follows the decoder rules** ([security](../standards/security.md#untrusted-input)): no panics, bounded allocation, checked arithmetic. Fuzz targets are generated for every request type.
5. **Attribution rules** for anything derived from Kafka:
   - Vendored schemas keep their license headers. `schemas/kafka-4.4.0/` contains Kafka's `LICENSE` and `NOTICE`.
   - Alluvium's [`NOTICE`](../../NOTICE) file states that it includes material from Apache Kafka.
   - **We reimplement behavior; we don't transliterate code.** If a piece of code or a test is ported closely, its header cites the source file and states that it was modified, as Apache-2.0 §4 requires. [Attributing Kafka material](../../CONTRIBUTING.md#attributing-kafka-material) gives the exact header.
   - `// Parity:` comments that merely reference Kafka need no attribution.

## Consequences

- All APIs get correct encoding for every version from one generator. A generator bug shows up everywhere at once, so it is caught early.
- Moving to a new Kafka version starts with vendoring new schemas and reviewing the generated diff.
- The generator is a significant piece of code, owned by `xtask` and tested with golden files.

## Alternatives considered

- **Hand-written codecs.** Rejected: too much surface, too many chances for version-specific mistakes.
- **`build.rs` or proc-macro generation.** Rejected: the output is invisible in the source tree and painful to debug.
- **An existing Rust Kafka protocol crate.** Rejected as the foundation: we need full control of allocation limits, zero-copy handling and exact version coverage. Existing crates may be used as differential test oracles.
