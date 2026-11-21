---
name: alluvium-engineering
description: Engineering standards and workflow for the Alluvium repository, a Rust streaming platform that is fully compatible with Apache Kafka 4.4.0 and goes beyond it. Use for EVERY task in this repository, whether or not it is requested — choosing what to work on next from the backlog, designing, writing or reviewing code, debugging, fixing bugs, refactoring, testing, adding dependencies, crates, Kafka features or enhancements, and editing docs, ADRs or the backlog.
---

# Alluvium engineering

Alluvium is a new streaming platform written in Rust. It is fully compatible with **Apache Kafka 4.4.0** clients and tools, and is designed from the start to go beyond Kafka's limitations ([ADR-0010](../../../docs/adr/0010-alluvium-is-its-own-platform.md)). All work is planned in the [backlog](../../../docs/backlog/README.md). This skill is how work gets done here. It applies to every change, including one-line fixes and docs edits.

The goal is a codebase that a new engineer can understand in a day and debug at 3 a.m. Every rule below serves one of these: **correctness, security, debuggability, maintainability, scalability**. When they conflict, that order decides. Performance only wins with benchmark evidence.

## More is not better (Braess' paradox)

In Braess' paradox, adding a road to a traffic network makes everyone's journey slower. Software behaves the same way: every extra line, feature, option, abstraction, dependency, task or doc adds paths to reason about, test, secure and debug, and it can make the whole system worse even when it looks like an improvement on its own.

So every addition must earn its place:

- **Purposeful.** Each line of code, feature, config key and doc sentence exists because the end goal needs it, and a backlog task traces it there: Kafka 4.4.0 parity and the planned foundations first, then ADR-approved enhancements. "Might be useful", "for completeness" and "while I'm here" are not reasons.
- **The best choice, not just a working one.** Before adding something, consider the alternatives, including changing or removing existing code, and doing nothing. Pick the option that best serves the goal with the least new surface.
- **Removal is progress.** Deleting code, options or docs that no longer serve the goal is a valued change, not a risky one.
- **Prove the gain.** If you claim an addition makes things better (faster, safer, easier to debug), show it: a test, a benchmark, or a concrete failure it prevents.

The anti-slop rules below are specific cases of this principle. Apply its intent to cases they don't name.

## Current phase

**Milestone M0 (workspace skeleton and tooling) is in progress.** The Cargo workspace and the `cargo xtask` checks exist; the first product crate is `alluvium-types`, which is part of M0.

Do not implement any feature outside M0 until M0 has landed: the workspace, lint config, `xtask` checks, CI and `alluvium-types` (see the [milestones](../../../docs/backlog/README.md#milestones)). If asked to build a feature before then, say so and offer to do M0 first.

To find the next task, run `python3 docs/backlog/tools/backlog.py next`. From M0 onward this moves to `cargo xtask backlog next`.

Update this section when the phase changes.

## Before any task

1. **Find the backlog task.** If asked "what's next", or given no specific task, pick one with `backlog.py next` and follow the [selection rules](../../../docs/backlog/README.md#choosing-the-next-task). If you were given work, find its task (`grep` or `backlog.py show <ID>`). If no task exists for it, add one first ([adding work](../../../docs/backlog/README.md#adding-work)). Check that the task's dependencies are `Done`, then set it to `In Progress`.
2. Identify the crate(s) and layer you are touching in [docs/architecture.md](../../../docs/architecture.md). If the work doesn't fit the layering rules, stop and propose an ADR. Do not bend the rules.
3. Read the module docs (`//!`) of the code you are changing. They state its responsibility and invariants.
4. For Kafka behavior, read the **Kafka 4.4.0 source** (tag `4.4.0` of `apache/kafka`) and the relevant KIP. Never guess Kafka semantics from memory.
5. Check [docs/adr/](../../../docs/adr/README.md) for existing decisions, and the task's `ref:` for the Kafka source or ADR it builds on.
6. Use the Kafka terms from [docs/glossary.md](../../../docs/glossary.md). Don't invent synonyms.

## Non-negotiable rules

Each rule has a reason. If a rule blocks you, raise it. Don't work around it.

1. **Kafka compatibility is a contract.** On the Kafka protocol surface, wire format, error codes per API version, ordering, durability and delivery semantics match Kafka 4.4.0. Any deviation needs an ADR. Features beyond Kafka are opt-in and off by default: existing clients must never notice a difference unless an operator opts in. ([ADR-0010](../../../docs/adr/0010-alluvium-is-its-own-platform.md))
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

- **No speculative abstraction.** No trait with a single implementation unless it is a documented test seam (`Clock`, file-system, transport) or a planned extension point: a backlog feature labelled `foundation-for:<epic>` ([ADR-0010](../../../docs/adr/0010-alluvium-is-its-own-platform.md)). No generic parameter "for flexibility". No builder for a struct with three fields.
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

**Choosing work.** Follow the backlog's [selection rules](../../../docs/backlog/README.md#choosing-the-next-task):
- lowest open milestone first;
- then ready `To Do` tasks;
- then by priority, epic build order (the number in each epic's file name) and ID.

Don't start a task whose dependencies aren't `Done`. Don't start work that has no backlog task; add the task first. Keep the task's status current: `In Progress`, `In Review`, `Done`.

**Adding Kafka functionality.** Follow [docs/guides/adding-a-kafka-feature.md](../../../docs/guides/adding-a-kafka-feature.md). In short: study Kafka 4.4.0 → claim the backlog task → protocol (codegen) → pure state machine plus tests → driver and handler → config, metrics, events, authorization → compatibility and differential tests → advertise in ApiVersions.

**Adding an enhancement (beyond Kafka).**
1. Start with the epic's design spike and get the ADR accepted.
2. Build on the existing `foundation-for:<epic>` interfaces rather than adding new ones.
3. Keep the feature off by default.
4. Prove that the Kafka compatibility and differential suites still pass with it enabled but unused.

**Fixing a bug.** Follow [docs/guides/debugging.md](../../../docs/guides/debugging.md):
1. Reproduce deterministically: a unit test, a simulation seed, or a compatibility test.
2. Commit the failing test.
3. Find the root cause, not the symptom.
4. Fix it.
5. Ask why the existing tests, invariant checks or lints didn't catch it, and close that gap too: add an invariant assertion, property test or lint.
6. The PR description states the root cause, why it escaped, and the fix.

**Refactoring.** Make no behavior change in the same PR. Tests must pass before and after, unchanged. If tests need changing, it isn't a pure refactor.

**Adding a crate.** Only when its first real code lands, never as an empty placeholder. Register its layer in `xtask` and [docs/architecture.md](../../../docs/architecture.md). It inherits `[workspace.lints]`. Its `lib.rs` starts with a `//!` doc covering responsibility, layer, Kafka counterpart and key invariants. Its deep dive in [docs/crates/](../../../docs/crates/README.md) is written from the template as the crate fills out (each crate has a DOCS-F11 task).

**Adding a dependency.** Add it in `[workspace.dependencies]` with minimal features. `cargo deny check` must pass. Justify it in the PR: what it does, why std or existing dependencies can't, maintenance status, `unsafe` usage, transitive dependency count.

**Making a design decision.** Write an ADR if it is hard to reverse, crosses crates, deviates from Kafka or from these standards, or a future engineer would ask "why is it like this?". Use [docs/adr/template.md](../../../docs/adr/template.md). Superseding is fine; silently diverging is not.

**Reviewing.** Apply the definition of done below plus the anti-slop rules. Prefer asking for a test that demonstrates a concern over arguing about it.

## Definition of done

A change is done only when all of these hold:

- [ ] Every added line, feature, option and dependency is needed for the end goal and is the best available choice ([more is not better](#more-is-not-better-braess-paradox)).
- [ ] `cargo xtask ci` passes: fmt, clippy `-D warnings`, nextest, doc, deny, layers, codegen drift. (Available from M0.)
- [ ] New behavior has tests at the right level: unit/property for logic, simulation for distributed logic, a fuzz target for any decoder, compatibility tests for client-visible behavior.
- [ ] Errors carry context. New state transitions emit events with a `reason` and are in the flight recorder. New hot paths have metrics.
- [ ] No new `unsafe`, unbounded queue, untimed network wait, unnamed task, secret in logs, or panic reachable from input.
- [ ] Kafka-mirroring code has `// Parity:` citations.
- [ ] The backlog task is updated (`In Review`, then `Done`), and new work found along the way is added as tasks. `backlog.py check` passes.
- [ ] Docs that describe the changed thing are updated in the same PR: module docs, architecture, standards, glossary, ADR.
- [ ] A new engineer could understand the change from the code, its docs and the PR description alone.

## When a standard is wrong

Standards evolve. If a rule is wrong for a case, change the standard (and add an ADR if it is architectural) in the same PR, with the reason. Never leave code that quietly contradicts the docs. The docs and this skill must always describe the code as it actually is.

## Reference map

| Need | Read |
|---|---|
| System design, crates, layers, runtime model, invariants | [docs/architecture.md](../../../docs/architecture.md) |
| How one crate works inside, and how to debug it | [docs/crates/](../../../docs/crates/README.md) |
| Rust rules, lint config, async, errors, tooling commands | [docs/standards/rust.md](../../../docs/standards/rust.md) |
| Logging, tracing fields, metrics, flight recorders, debug tools | [docs/standards/debuggability.md](../../../docs/standards/debuggability.md) |
| Threat model, input handling, TLS/SASL/ACLs, dependencies | [docs/standards/security.md](../../../docs/standards/security.md) |
| Test levels, simulation, fuzzing, compatibility tests | [docs/standards/testing.md](../../../docs/standards/testing.md) |
| Implementing a Kafka API or feature | [docs/guides/adding-a-kafka-feature.md](../../../docs/guides/adding-a-kafka-feature.md) |
| Investigating a bug | [docs/guides/debugging.md](../../../docs/guides/debugging.md) |
| What to work on next; every planned task and its status | [docs/backlog/](../../../docs/backlog/README.md) |
| Kafka API scope and version ranges | [docs/parity.md](../../../docs/parity.md) |
| Why things are the way they are | [docs/adr/](../../../docs/adr/README.md) |
| Kafka vocabulary | [docs/glossary.md](../../../docs/glossary.md) |
