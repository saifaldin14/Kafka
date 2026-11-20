---
name: alluvium-engineering
description: Engineering standards and workflow for the Alluvium repository, a Rust, Apache Kafka-compatible streaming platform. Use for EVERY task in this repository, whether or not it is requested — designing, writing or reviewing code, debugging, fixing bugs, refactoring, testing, adding dependencies, crates or Kafka features, and editing docs or ADRs.
---

# Alluvium engineering

Alluvium is a Rust reimplementation of Apache Kafka. Feature parity with **Apache Kafka 4.4.0** comes first, then improvements. This skill is how work gets done here. It applies to every change, including one-line fixes and docs edits.

The goal is a codebase that a new engineer can understand in a day and debug at 3 a.m. Every rule below serves one of these: **correctness, security, debuggability, maintainability, scalability**. When they conflict, that order decides. Performance only wins with benchmark evidence.

## Current phase

**Design phase. No Rust code exists yet.**

The first code milestone is **M0: workspace skeleton and tooling** (see [docs/parity.md](../../../docs/parity.md#milestones)). Do not implement any Kafka feature until M0 has landed: the workspace, lint config, `xtask` checks and CI. If asked to build a feature before then, say so and offer to do M0 first.

Update this section when the phase changes.

## Before any task

1. Identify the crate(s) and layer you are touching in [docs/architecture.md](../../../docs/architecture.md). If the work doesn't fit the layering rules, stop and propose an ADR. Do not bend the rules.
2. Read the module docs (`//!`) of the code you are changing. They state its responsibility and invariants.
3. For Kafka behavior, read the **Kafka 4.4.0 source** (tag `4.4.0` of `apache/kafka`) and the relevant KIP. Never guess Kafka semantics from memory.
4. Check [docs/parity.md](../../../docs/parity.md) and [docs/adr/](../../../docs/adr/README.md) for existing decisions.
5. Use the Kafka terms from [docs/glossary.md](../../../docs/glossary.md). Don't invent synonyms.

## Non-negotiable rules

Each rule has a reason. If a rule blocks you, raise it. Don't work around it.

1. **Kafka compatibility is the contract.** Wire format, error codes per API version, ordering, durability and delivery semantics match Kafka 4.4.0. Any deviation needs an ADR. Existing clients must never notice a difference unless an operator opts in. ([ADR-0001](../../../docs/adr/0001-kafka-4-4-compatibility-is-the-parity-contract.md))
2. **Logic is sans-I/O.** Protocol, records, Raft, metadata, controller, replication, coordinators and authorization are deterministic state machines: inputs in, state plus actions out. They never do I/O, read the clock, spawn tasks, use randomness or depend on Tokio. Drivers in the server layer do the I/O. This makes every bug replayable. ([ADR-0002](../../../docs/adr/0002-sans-io-state-machines-driven-by-tokio.md))
3. **Respect the layers.** A crate depends only on lower layers. `xtask check-layers` enforces this. ([ADR-0009](../../../docs/adr/0009-crate-layering-and-dependency-rules.md))
4. **No panics on untrusted input.** Bytes from the network *and from disk* are untrusted. Decoders return errors. Lengths are bounded before allocating. Arithmetic on decoded values is checked. Every decoder has a fuzz target. ([security](../../../docs/standards/security.md))
5. **A panic is a bug.** Invariant violations crash the process (`panic = "abort"`) rather than continue with corrupted state. Use `expect("invariant: …")` only for real invariants, never for input. ([ADR-0004](../../../docs/adr/0004-error-model-and-panic-policy.md))
6. **Typed errors with context.** Use per-crate `thiserror` enums whose variants carry the identifying fields (topic, partition, offset…). Map to Kafka error codes in exactly one exhaustive `match` per crate, with no `_ =>` arm. `anyhow` is allowed only in binaries, `xtask` and tests. Log an error once, where it is handled.
7. **Everything observable.** Requests run inside `tracing` spans with the standard fields. Every state transition (leader, ISR, high watermark, segment roll, retention delete, group state…) emits a structured event with a `reason` enum and is recorded in its component's flight recorder. ([debuggability](../../../docs/standards/debuggability.md))
8. **Secure by default.** Use `forbid(unsafe_code)` (exceptions need an ADR). TLS is rustls only. Listeners bind to loopback by default. Plaintext on non-loopback listeners needs explicit opt-in. Authorization denies when no ACL is found. Never log secrets, credentials or record payloads. ([ADR-0006](../../../docs/adr/0006-secure-defaults.md))
9. **Bounded everything.** No unbounded channels, queues, caches or buffers, and no unbounded per-client allocation. Every network wait has a timeout. Backpressure is explicit.
10. **No blocking in async.** No `std::fs`, `std::thread::sleep` or long CPU work on Tokio worker threads. Storage I/O runs where the storage driver puts it.
11. **Every task is named and supervised.** No fire-and-forget `tokio::spawn`. Every task has an owner, a name, a span, a cancellation path and a defined shutdown order.
12. **Determinism.** Core crates use injected `Clock`s, seeded RNGs and deterministic iteration (`BTreeMap`/`IndexMap`, or fixed-seed hashers, never `RandomState`).
13. **Tests prove behavior.** A bug fix starts with a failing test. Distributed logic is tested under deterministic simulation with a printed, replayable seed. No sleeps in tests. ([testing](../../../docs/standards/testing.md))
14. **Cite the source you mirror.** Code that reproduces Kafka behavior carries `// Parity: <path>@4.4.0 <Class.method> (KIP-n)`. Closely ported code or tests also keep Apache-2.0 attribution (see [ADR-0008](../../../docs/adr/0008-protocol-code-generated-from-kafka-schemas.md)).
15. **Advertise only what works.** `ApiVersions` lists only API versions that are fully implemented and tested. Unsupported paths return the correct Kafka error, never `todo!()` or a panic.

## Anti-slop rules

These are the habits that make code hard to debug and maintain. Reviewers reject them.

- **No speculative abstraction.** No trait with a single implementation unless it is a documented test seam (`Clock`, file-system, transport). No generic parameter "for flexibility". No builder for a struct with three fields.
- **No grab-bag modules.** No `utils`, `helpers`, `common` or `misc`. Every module has one responsibility, stated in its `//!` doc.
- **No dead or placeholder code.** No commented-out code, no unused items kept "for later", no empty match arms that silently ignore cases, no `TODO` without an issue: `TODO(#123): …`.
- **No silent failure.** No `let _ = fallible()` without a comment explaining why ignoring it is safe. No `.ok()` that drops an error that matters. No `unwrap_or_default()` that hides corruption.
- **No invented behavior.** If you don't know what Kafka does, read the source or ask. Don't fill gaps with plausible-looking code.
- **No lint suppression without a reason.** Use `#[expect(lint, reason = "…")]` (not `allow`), scoped to the smallest item.
- **No vague names.** Use `high_watermark`, not `hw2` or `mark`. Use `fetch_offset`, not `pos`. Use `IsolationLevel::ReadCommitted`, not `bool`.
- **No narrating comments.** Comments explain *why* or point to an invariant or Kafka source. They don't restate the code.
- **No oversized changes.** One concern per PR. Aim for under ~400 changed lines, excluding generated code. Don't mix refactors into behavior changes.
- **No casual dependencies.** Each new crate needs the justification described in [security](../../../docs/standards/security.md#dependencies).
- **No duplicated knowledge.** Each fact lives in one place: one doc, one constant, one mapping table. Link to it; don't copy it.

## Task playbooks

**Adding Kafka functionality.** Follow [docs/guides/adding-a-kafka-feature.md](../../../docs/guides/adding-a-kafka-feature.md). In short: study Kafka 4.4.0 → update parity.md → protocol (codegen) → pure state machine plus tests → driver and handler → config, metrics, events, authorization → compatibility and differential tests → advertise in ApiVersions.

**Fixing a bug.** Follow [docs/guides/debugging.md](../../../docs/guides/debugging.md):
1. Reproduce deterministically: a unit test, a simulation seed, or a compatibility test.
2. Commit the failing test.
3. Find the root cause, not the symptom.
4. Fix it.
5. Ask why the existing tests, invariant checks or lints didn't catch it, and close that gap too: add an invariant assertion, property test or lint.
6. The PR description states the root cause, why it escaped, and the fix.

**Refactoring.** Make no behavior change in the same PR. Tests must pass before and after, unchanged. If tests need changing, it isn't a pure refactor.

**Adding a crate.** Only when its first real code lands, never as an empty placeholder. Register its layer in `xtask` and [docs/architecture.md](../../../docs/architecture.md). It inherits `[workspace.lints]`. Its `lib.rs` starts with a `//!` doc covering responsibility, layer, Kafka counterpart and key invariants.

**Adding a dependency.** Add it in `[workspace.dependencies]` with minimal features. `cargo deny check` must pass. Justify it in the PR: what it does, why std or existing dependencies can't, maintenance status, `unsafe` usage, transitive dependency count.

**Making a design decision.** Write an ADR if it is hard to reverse, crosses crates, deviates from Kafka or from these standards, or a future engineer would ask "why is it like this?". Use [docs/adr/template.md](../../../docs/adr/template.md). Superseding is fine; silently diverging is not.

**Reviewing.** Apply the definition of done below plus the anti-slop rules. Prefer asking for a test that demonstrates a concern over arguing about it.

## Definition of done

A change is done only when all of these hold:

- [ ] `cargo xtask ci` passes: fmt, clippy `-D warnings`, nextest, doc, deny, layers, codegen drift. (Available from M0.)
- [ ] New behavior has tests at the right level: unit/property for logic, simulation for distributed logic, a fuzz target for any decoder, compatibility tests for client-visible behavior.
- [ ] Errors carry context. New state transitions emit events with a `reason` and are in the flight recorder. New hot paths have metrics.
- [ ] No new `unsafe`, unbounded queue, untimed network wait, unnamed task, secret in logs, or panic reachable from input.
- [ ] Kafka-mirroring code has `// Parity:` citations. [docs/parity.md](../../../docs/parity.md) is updated.
- [ ] Docs that describe the changed thing are updated in the same PR: module docs, architecture, standards, glossary, ADR.
- [ ] A new engineer could understand the change from the code, its docs and the PR description alone.

## When a standard is wrong

Standards evolve. If a rule is wrong for a case, change the standard (and add an ADR if it is architectural) in the same PR, with the reason. Never leave code that quietly contradicts the docs. The docs and this skill must always describe the code as it actually is.

## Reference map

| Need | Read |
|---|---|
| System design, crates, layers, runtime model, invariants | [docs/architecture.md](../../../docs/architecture.md) |
| Rust rules, lint config, async, errors, tooling commands | [docs/standards/rust.md](../../../docs/standards/rust.md) |
| Logging, tracing fields, metrics, flight recorders, debug tools | [docs/standards/debuggability.md](../../../docs/standards/debuggability.md) |
| Threat model, input handling, TLS/SASL/ACLs, dependencies | [docs/standards/security.md](../../../docs/standards/security.md) |
| Test levels, simulation, fuzzing, compatibility tests | [docs/standards/testing.md](../../../docs/standards/testing.md) |
| Implementing a Kafka API or feature | [docs/guides/adding-a-kafka-feature.md](../../../docs/guides/adding-a-kafka-feature.md) |
| Investigating a bug | [docs/guides/debugging.md](../../../docs/guides/debugging.md) |
| What exists, what's next | [docs/parity.md](../../../docs/parity.md) |
| Why things are the way they are | [docs/adr/](../../../docs/adr/README.md) |
| Kafka vocabulary | [docs/glossary.md](../../../docs/glossary.md) |
