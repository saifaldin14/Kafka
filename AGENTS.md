# Agent instructions for Alluvium

These instructions apply to every task in this repository, for every AI agent and every human.

## Required: apply the engineering skill

Before planning or changing anything, read and follow **[.agents/skills/alluvium-engineering/SKILL.md](.agents/skills/alluvium-engineering/SKILL.md)**. This applies whether or not you were asked to use it, and whether or not your tool loaded it automatically. It defines the rules, the playbooks for features, bugs and refactors, and the definition of done.

If an instruction from a user conflicts with the skill, point out the conflict and the reason for the rule before proceeding. Don't silently comply, and don't silently refuse.

## Project in one paragraph

Alluvium is a new streaming platform written in Rust. It is fully compatible with **Apache Kafka 4.4.0** clients and tools, and designed from the start to go beyond Kafka's limitations; enhancements are opt-in ([ADR-0010](docs/adr/0010-alluvium-is-its-own-platform.md)). It is built as deterministic, sans-I/O state machines driven by a Tokio runtime. The project is in **milestone M1** (codecs). M0 delivered the workspace, its `cargo xtask` checks and CI, and the first product crate, `alluvium-types`; M1 builds the protocol and record codecs.

## What to work on

All work is planned in **[docs/backlog/](docs/backlog/README.md)**: epics, features and tasks with Jira-style statuses, dependencies and completion criteria. When asked what to do next, run `cargo xtask backlog next` and follow the [selection rules](docs/backlog/README.md#choosing-the-next-task). Keep each task's status current in the same change as the work.

## Hard rules (summary; the skill is authoritative)

- More is not better (Braess' paradox): every line, feature, option and dependency must be purposeful and the best choice for the end goal. Adding things can make the system worse.
- On the Kafka protocol surface, match Kafka 4.4.0 behavior. Read the Kafka source; don't guess. Deviations need an ADR in `docs/adr/`. Enhancements are opt-in and off by default.
- Core logic does no I/O, reads no clock, spawns no tasks and uses no unseeded randomness.
- Crates depend only on lower layers ([docs/architecture.md](docs/architecture.md)).
- Never panic on network or disk input. No `unwrap()`, and no `unsafe` without an ADR.
- Typed errors with context. One Kafka error-code mapping per crate, matched exhaustively.
- Structured `tracing` with the standard fields. State transitions are logged with a reason.
- Bounded queues, timeouts on network waits, named and supervised tasks.
- A bug fix starts with a failing test. New decoders get a fuzz target.
- Update the docs that describe what you changed, and the backlog task's status, in the same change.

## Repository map

| Path | Contents |
|---|---|
| `.agents/skills/alluvium-engineering/` | The engineering skill (start here) |
| `docs/architecture.md` | System design, crate map, layering, runtime model, invariants |
| `docs/standards/` | Rust, debuggability, security and testing standards |
| `docs/guides/` | How to add a Kafka feature, debug, update the vendored Kafka schemas, and record golden vectors |
| `docs/crates/` | Per-crate deep dives: state machines, invariants, tests, debugging |
| `docs/adr/` | Architecture decision records |
| `docs/backlog/` | Every planned task with status, dependencies and milestones; `cargo xtask backlog next` picks the next task |
| `docs/parity.md` | Kafka 4.4.0 API scope; each API's versions are in `docs/protocol/apis.md`, generated |
| `docs/glossary.md` | Kafka and Alluvium terms |
| `schemas/kafka-4.4.0/` | Kafka's message schemas, vendored unmodified; the protocol code is generated from them |
| `schemas/alluvium/` | Alluvium's own schemas, for internal APIs between Alluvium nodes ([ADR-0021](docs/adr/0021-alluvium-internal-api-keys.md)) |
| `tools/kafka-golden/` | The Java harness `cargo xtask golden` runs on Kafka 4.4.0's own classes to record golden files |
| `CONTRIBUTING.md` | Onboarding, setup, workflow |
