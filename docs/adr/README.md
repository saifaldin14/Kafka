# Architecture decision records

An ADR records a decision that is hard to reverse, crosses crates, deviates from Kafka or from our standards, or that a future engineer would otherwise have to rediscover. Each one is short: context, decision, consequences, alternatives.

## How to write one

1. Copy [template.md](template.md) to `NNNN-short-title.md`, using the next free number.
2. Set the status to **Proposed** and open a PR. Discussion happens on the PR.
3. On merge, set the status to **Accepted**.
4. ADRs are not edited after acceptance except to fix typos or add a "Superseded by" link. To change a decision, write a new ADR that supersedes the old one.

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-kafka-4-4-compatibility-is-the-parity-contract.md) | Kafka 4.4 compatibility is the parity contract | Superseded by 0010 |
| [0002](0002-sans-io-state-machines-driven-by-tokio.md) | Sans-I/O state machines driven by Tokio | Accepted |
| [0003](0003-deterministic-simulation-testing.md) | Deterministic simulation testing | Accepted |
| [0004](0004-error-model-and-panic-policy.md) | Error model and panic policy | Accepted |
| [0005](0005-observability-model.md) | Observability model | Accepted |
| [0006](0006-secure-defaults.md) | Secure defaults | Accepted |
| [0007](0007-record-batches-stored-in-wire-format.md) | Record batches stored in wire format | Accepted |
| [0008](0008-protocol-code-generated-from-kafka-schemas.md) | Protocol code generated from Kafka schemas | Accepted |
| [0009](0009-crate-layering-and-dependency-rules.md) | Crate layering and dependency rules | Accepted |
| [0010](0010-alluvium-is-its-own-platform.md) | Alluvium is its own platform; Kafka 4.4 is a compatibility surface | Accepted |
| [0011](0011-time-model-and-clock-injection.md) | Time model and clock injection | Accepted |
| [0012](0012-seeded-randomness-and-hashing.md) | Seeded randomness and hashing | Accepted |
| [0013](0013-key-hash-space.md) | Key hash space | Accepted |
| [0014](0014-namespace-identity.md) | Namespace identity | Accepted |
| [0015](0015-project-license-and-contributor-terms.md) | Project license and contributor terms | Accepted |
| [0016](0016-layer-0-crates-may-depend-on-alluvium-types.md) | Layer 0 crates may depend on alluvium-types | Accepted |
| [0017](0017-per-decode-allocation-budget.md) | A per-decode allocation budget | Accepted |
| [0018](0018-fuzz-corpus-storage.md) | Fuzz corpus storage | Accepted |
| [0019](0019-benchmark-regression-gating.md) | Benchmark regression gating | Accepted |
| [0020](0020-known-tagged-fields-read-within-their-declared-size.md) | Known tagged fields are read within their declared size | Accepted |
| [0021](0021-alluvium-internal-api-keys.md) | Alluvium-internal API keys | Accepted |
