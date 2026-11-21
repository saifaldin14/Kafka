# Contributing to Alluvium

This page gets a new engineer from zero to a first useful change. Read it top to bottom once.

## 1. Learn the system (about two hours)

Read these in order:

1. [README.md](README.md): what Alluvium is and its current status.
2. [docs/glossary.md](docs/glossary.md): the vocabulary. Kafka has many precise terms (high watermark, ISR, leader epoch…), and we use them exactly.
3. [docs/architecture.md](docs/architecture.md): the crate map, layers, runtime model and invariants. This is the most important document.
4. [docs/adr/README.md](docs/adr/README.md): skim the index; read the ADRs for the area you'll work in.
5. [.agents/skills/alluvium-engineering/SKILL.md](.agents/skills/alluvium-engineering/SKILL.md): the rules and the definition of done. It is written for AI agents and humans alike.
6. [docs/backlog/README.md](docs/backlog/README.md): how work is planned, how to pick the next task, and the milestones. Then skim the epic file for the area you'll work in.
7. The `//!` module docs of the crate you'll touch. *(Available once code exists.)*

Keep [docs/standards/](docs/standards/) open as a reference. You don't need to memorize it.

## 2. Set up your machine

*Milestone M0 is building these commands. `cargo xtask ci --list` shows the checks that exist today; the [tooling table](docs/standards/rust.md#tooling-commands-contract-implemented-in-m0) lists what exists and what is still to come.*

- **Rust via rustup.** Don't use Homebrew's or your distribution's `cargo`. They ignore `rust-toolchain.toml`, so you'd build with the wrong compiler. Check with `which cargo`, which should point into `~/.cargo/bin`.
- The pinned toolchain installs automatically on the first `cargo` command.
- Tools: `cargo xtask tools install` installs the pinned versions of cargo-nextest, cargo-deny and lychee. `cargo-fuzz` and a nightly toolchain are needed only for fuzzing.
- Docker, only for compatibility and differential tests (they run real Kafka clients and a real Kafka 4.4.0 broker).

```bash
cargo xtask ci                # everything CI runs, step by step
cargo xtask ci --step test    # one step only; --list shows them all
cargo xtask codegen     # regenerate protocol code from the Kafka schemas
cargo xtask compat      # compatibility and differential tests (needs Docker)
cargo run -- server --config config/dev.toml   # local single-node broker
```

## 3. Make a change

1. **Pick a backlog task.** Run `python3 docs/backlog/tools/backlog.py next`, or follow the [selection rules](docs/backlog/README.md#choosing-the-next-task). Set the task to `In Progress` with your name. If the work you need has no task, [add one](docs/backlog/README.md#adding-work) first. Each change has a reason someone can find later.
2. **Find the Kafka reference.** For Kafka behavior, locate the code in Kafka at tag `4.4.0` and the KIP. Link them in the PR.
3. **Write the test first** for bugs, and early for features.
4. **Keep it small and purposeful.** One concern per PR, ideally under ~400 changed lines excluding generated code. Refactors and behavior changes go in separate PRs. Add only what the goal needs; adding more can make the system worse ([more is not better](.agents/skills/alluvium-engineering/SKILL.md#more-is-not-better-braess-paradox)).
5. **Update docs and the backlog in the same PR**: module docs, standards, ADRs, glossary, the task's status (`In Review`, then `Done`).
6. **Run `cargo xtask ci`** before pushing.
7. **Fill in the PR template.** For bug fixes, the root cause and "why it escaped" are required.

### Commit messages

```
<area>: <imperative summary under 72 chars>

Why the change is needed and what it does. Reference Kafka source and KIPs
where relevant. Refs #123.
```

`<area>` is a crate name without the `alluvium-` prefix (`storage`, `protocol`, `replication`), or `docs`, `xtask` or `ci`.

## 4. Review

- Reviewers check the definition of done and the anti-slop rules in the skill.
- "Add a test that shows it" beats a long argument.
- Approve only if you could debug this code yourself without the author's help.

## 5. Good first tasks

Once M0 lands, good starting points are self-contained, well-specified pieces with clear Kafka references. Examples: a protocol primitive codec with property tests, a fuzz target for an existing decoder, or a config key with validation. Issues labelled `good-first-task` point at them.
