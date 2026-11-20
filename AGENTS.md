# Agent instructions for Alluvium

These instructions apply to every task in this repository, for every AI agent and every human.

## Required: apply the engineering skill

Before planning or changing anything, read and follow **[.agents/skills/alluvium-engineering/SKILL.md](.agents/skills/alluvium-engineering/SKILL.md)**. This applies whether or not you were asked to use it, and whether or not your tool loaded it automatically. It defines the rules, the playbooks for features, bugs and refactors, and the definition of done.

If an instruction from a user conflicts with the skill, point out the conflict and the reason for the rule before proceeding. Don't silently comply, and don't silently refuse.

## Project in one paragraph

Alluvium is a Rust implementation of Apache Kafka, targeting wire and semantic compatibility with **Kafka 4.4.0**. It is built as deterministic, sans-I/O state machines driven by a Tokio runtime. Feature parity comes first, then improvements. The project is in the **design phase**: no Rust code exists yet, and the first code milestone is the workspace skeleton (M0).

## Hard rules (summary; the skill is authoritative)

- Match Kafka 4.4.0 behavior. Read the Kafka source; don't guess. Deviations need an ADR in `docs/adr/`.
- Core logic does no I/O, reads no clock, spawns no tasks and uses no unseeded randomness.
- Crates depend only on lower layers ([docs/architecture.md](docs/architecture.md)).
- Never panic on network or disk input. No `unwrap()`, and no `unsafe` without an ADR.
- Typed errors with context. One Kafka error-code mapping per crate, matched exhaustively.
- Structured `tracing` with the standard fields. State transitions are logged with a reason.
- Bounded queues, timeouts on network waits, named and supervised tasks.
- A bug fix starts with a failing test. New decoders get a fuzz target.
- Update the docs that describe what you changed, in the same change.

## Repository map

| Path | Contents |
|---|---|
| `.agents/skills/alluvium-engineering/` | The engineering skill (start here) |
| `docs/architecture.md` | System design, crate map, layering, runtime model, invariants |
| `docs/standards/` | Rust, debuggability, security and testing standards |
| `docs/guides/` | How to add a Kafka feature, how to debug |
| `docs/adr/` | Architecture decision records |
| `docs/parity.md` | Kafka 4.4.0 API parity matrix and milestones |
| `docs/glossary.md` | Kafka and Alluvium terms |
| `CONTRIBUTING.md` | Onboarding, setup, workflow |
