# ADR-0015: Project license and contributor terms

- **Status:** Accepted
- **Date:** 2026-11-22

## Context

The repository is public on GitHub, but no license had been chosen. The options were a permissive license (Apache-2.0, or MIT OR Apache-2.0), a copyleft or source-available license, or staying proprietary. The owner chose to stay proprietary for now.

Two facts constrain any choice:

- **Public is not licensed.** Without a license, copyright law reserves every right to the owner. GitHub's Terms of Service (section D.5) still let other GitHub users view and fork a public repository on GitHub, and nothing more.
- **Alluvium includes Apache Kafka material**, licensed under the Apache License 2.0, which permits use in a proprietary work if its §4 conditions are met:
  - §4(a): recipients get a copy of the Apache License;
  - §4(b): modified files carry prominent notices that they were changed;
  - §4(c): copyright and attribution notices from the source are kept;
  - §4(d): the attribution notices in Kafka's `NOTICE` that pertain to the included material are reproduced.

  The material today is a close port of `Utils.murmur2` and tests ported from `UuidTest`, `TopicTest` and `UtilsTest`. Kafka's protocol schemas will be vendored under `schemas/kafka-4.4.0/` ([ADR-0008](0008-protocol-code-generated-from-kafka-schemas.md)).

## Decision

1. **Alluvium is proprietary: all rights reserved.** `LICENSE` states the copyright and that no license is granted, except as GitHub's Terms of Service require for a public repository and except for the third-party material it lists. Every workspace crate declares `license-file = "LICENSE"`, inherited from `[workspace.package]`, so `cargo metadata` shows it.

2. **Kafka material keeps its own license.**
   - `LICENSES/Apache-2.0.txt` holds the Apache License 2.0 text, for §4(a).
   - `NOTICE` reproduces the attribution notices from Kafka 4.4.0's `NOTICE` that pertain to the included material, for §4(d), and lists the files that contain it.
   - Each closely ported file carries the header that [CONTRIBUTING](../../CONTRIBUTING.md) defines, naming the source and stating that it was modified, for §4(b) and §4(c).
   - Vendored files stay unmodified, with Kafka's `LICENSE` and `NOTICE` beside them (ADR-0008).

3. **No outside contributions.** Only the owner contributes, so there are no contributor terms (neither DCO nor CLA). If outside contributions are wanted later, this ADR is superseded first: accepting code under no agreement would leave its copyright with its author and block any later relicensing.

4. **No SPDX headers.** With one license for the whole repository, per-file headers would repeat `LICENSE` in every file. The only per-file notices are the attribution headers on Kafka material, which the license requires.

## Consequences

- Choosing an open-source license later is the owner's decision alone, because the owner holds the copyright of every line apart from the Kafka material. It needs a new ADR that supersedes this one, and a new `LICENSE`.
- The Apache-2.0 obligations apply whatever Alluvium's own license is. `NOTICE` and the file headers must be kept current: adding Kafka material means adding its file to `NOTICE` in the same change (CONTRIBUTING).
- `cargo deny` keeps ignoring the workspace's own crates (`[licenses.private]`): they are unpublished, and their license isn't on the open-source allow-list, which governs dependencies.
- Outside pull requests are closed with a pointer to this ADR.

## Alternatives considered

- **Apache-2.0, or MIT OR Apache-2.0.** The usual choice for Rust infrastructure, and compatible with the Kafka material. Not chosen now; the decision is reversible later, because the owner holds the copyright.
- **Copyleft (AGPL-3.0) or source-available (BSL, Elastic License).** These keep competitors from offering Alluvium as a service, but they need contributor terms to keep relicensing possible, and they restrict users. Not chosen.
- **A CLA or DCO now.** Unnecessary while nobody else contributes.
- **SPDX headers in every file.** Machine-readable, but they add a line to every file for no gain while one license covers the repository.
