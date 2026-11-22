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

*`cargo xtask ci --list` shows the checks; the [tooling table](docs/standards/rust.md#tooling-commands-contract-implemented-in-m0) lists every command, and those still to come with later milestones.*

- **Rust via rustup.** Don't use Homebrew's or your distribution's `cargo`. They ignore `rust-toolchain.toml`, so you'd build with the wrong compiler. Check with `which cargo`, which should point into `~/.cargo/bin`.
- The pinned toolchain installs automatically on the first `cargo` command.
- Tools: `cargo xtask tools install` installs the pinned versions of cargo-nextest, cargo-deny and lychee (from crates.io), and actionlint and shellcheck (release downloads checked against pinned SHA-256s, into `~/.cargo/bin`). `cargo-fuzz` and a nightly toolchain are needed only for fuzzing: `cargo xtask tools install --tool cargo-fuzz` and the `rustup` command in [fuzz/README.md](fuzz/README.md).
- Or open the repository in its [devcontainer](.devcontainer/devcontainer.json) (VS Code: "Reopen in Container", or `devcontainer up`): it has rustup, the pinned toolchain, every pinned tool and Docker, and puts `target/` on a volume.
- Then run `cargo xtask doctor`. It checks all of the above and prints the exact command that fixes each problem.
- Optionally, `cargo xtask install-hooks` adds a pre-push hook that runs the formatting and clippy checks, so they fail on your machine before CI; `--uninstall` removes it.
- Docker, only for compatibility and differential tests (they run real Kafka clients and a real Kafka 4.4.0 broker).
- Editors: [`.editorconfig`](.editorconfig) sets UTF-8, LF, final newlines and indentation to match `cargo fmt`. VS Code picks up [`.vscode/settings.json`](.vscode/settings.json) (rust-analyzer runs clippy, and Rust formats on save) and offers the extensions in [`.vscode/extensions.json`](.vscode/extensions.json).

```bash
cargo xtask ci                # everything CI runs, step by step
cargo xtask ci --step test    # one step only; --list shows them all
cargo xtask codegen     # regenerate protocol code from the Kafka schemas
cargo xtask compat      # compatibility and differential tests (needs Docker)
cargo run -- server --config config/dev.toml   # local single-node broker
```

## 3. Make a change

1. **Pick a backlog task.** Run `cargo xtask backlog next`, or follow the [selection rules](docs/backlog/README.md#choosing-the-next-task). Set the task to `In Progress` with your name. If the work you need has no task, [add one](docs/backlog/README.md#adding-work) first. Each change has a reason someone can find later.
2. **Find the Kafka reference.** For Kafka behavior, locate the code in Kafka at tag `4.4.0` and the KIP. Link them in the PR.
3. **Write the test first** for bugs, and early for features.
4. **Keep it small and purposeful.** One concern per PR, ideally under ~400 changed lines excluding generated code. Refactors and behavior changes go in separate PRs. Add only what the goal needs; adding more can make the system worse ([more is not better](.agents/skills/alluvium-engineering/SKILL.md#more-is-not-better-braess-paradox)).
5. **Update docs and the backlog in the same PR**: module docs, standards, ADRs, glossary, the task's status (`In Review`, then `Done`).
6. **Run `cargo xtask ci`** before pushing.
7. **Fill in the PR template.** For bug fixes, the root cause and "why it escaped" are required.

### Attributing Kafka material

Most code reimplements Kafka behavior after reading Kafka's source. That needs only a `// Parity: <path>@4.4.0 <Class.method>` citation ([rust.md](docs/standards/rust.md#modules-and-documentation)), not attribution.

Code or a test is **closely ported** when it follows Kafka's code statement by statement, or copies Kafka's test cases or test vectors. Kafka's Apache License 2.0 then applies to that part ([ADR-0015](docs/adr/0015-project-license-and-contributor-terms.md)), and section 4 requires three things:

1. **The file header.** The file starts with exactly these lines, before its `//!` doc:

   ```rust
   // Portions of this file are ported from Apache Kafka 4.4.0 and modified by Alluvium;
   // each is marked `// Ported from:`. Apache Kafka is Copyright The Apache Software
   // Foundation and licensed under the Apache License 2.0 (LICENSES/Apache-2.0.txt).
   // See NOTICE.
   ```

2. **A `// Ported from:` comment** on each ported item (a function, or a test or group of tests) naming its source, followed by anything left out or changed:

   ```rust
   // Ported from: clients/src/test/java/org/apache/kafka/common/UuidTest.java@4.4.0.
   // testHashCode is not ported: Java's hashCode has no counterpart here.
   ```

   Ported code uses this comment in place of `// Parity:`.

3. **An entry in [`NOTICE`](NOTICE)** listing the file and its Kafka sources.

Vendored Kafka files, such as the schemas in `schemas/kafka-4.4.0/` ([ADR-0008](docs/adr/0008-protocol-code-generated-from-kafka-schemas.md)), stay unmodified with their own license headers; add their directory to `NOTICE`. The PR template asks whether a change ports Kafka code.

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
- `main` accepts changes only through a PR whose `ci-success` check passed on a branch that is up to date with `main`, merged by rebase or squash. The [repository settings](docs/guides/repository-settings.md) page lists every rule and how to check it.

## 5. Good first tasks

Good starting points are self-contained, well-specified pieces with clear Kafka references. Examples: a protocol primitive codec with property tests, a fuzz target for an existing decoder, or a config key with validation. Issues labelled `good-first-task` point at them.
