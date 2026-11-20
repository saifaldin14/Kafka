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
| [0001](0001-kafka-4-4-compatibility-is-the-parity-contract.md) | Kafka 4.4 compatibility is the parity contract | Accepted |
| [0002](0002-sans-io-state-machines-driven-by-tokio.md) | Sans-I/O state machines driven by Tokio | Accepted |
| [0003](0003-deterministic-simulation-testing.md) | Deterministic simulation testing | Accepted |
| [0004](0004-error-model-and-panic-policy.md) | Error model and panic policy | Accepted |
| [0005](0005-observability-model.md) | Observability model | Accepted |
| [0006](0006-secure-defaults.md) | Secure defaults | Accepted |
| [0007](0007-record-batches-stored-in-wire-format.md) | Record batches stored in wire format | Accepted |
| [0008](0008-protocol-code-generated-from-kafka-schemas.md) | Protocol code generated from Kafka schemas | Accepted |
| [0009](0009-crate-layering-and-dependency-rules.md) | Crate layering and dependency rules | Accepted |
