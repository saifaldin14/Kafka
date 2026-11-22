# `lint-canary`: deep dive

## Responsibility and layer

`lint-canary` holds fixture code that breaks the workspace lints on purpose. `cargo xtask lint-canary` checks that every lint fires on exactly the lines annotated for it, and that every configured lint has such a line. A lint that is misspelled, removed or weakened in `[workspace.lints]` or `clippy.toml` would otherwise fail silently, because code just stops being checked. The crate proves the lints; enforcing them on real code is clippy's job in the `clippy` step.

It is a dev-layer crate in the [code map](../architecture.md#4-code-map), at `xtask/lint-canary`, and nothing depends on it ([ADR-0009](../adr/0009-crate-layering-and-dependency-rules.md)). Its optional `getrandom`, `rand` and `tokio` dependencies exist only so the fixtures can call the functions `clippy.toml` bans. The `canary` feature, which only `cargo xtask lint-canary` enables, compiles the fixtures; normal builds compile an empty crate.

[`src/lib.rs`](../../xtask/lint-canary/src/lib.rs) specifies the `expect-lint:` annotation format and the sanctioned exceptions. The checker is [`xtask/src/lint_canary.rs`](../../xtask/src/lint_canary.rs), described in the [`xtask` deep dive](xtask.md).

| Fixture module | Breaks |
|---|---|
| [`api`](../../xtask/lint-canary/src/fixtures/api.rs), [`strings`](../../xtask/lint-canary/src/fixtures/strings.rs), [`output`](../../xtask/lint-canary/src/fixtures/output.rs) | Public API hygiene (documented, printable, reachable `pub`, no needless path qualification), byte-slicing a `str`, writing to the terminal instead of through `tracing` |
| [`panics`](../../xtask/lint-canary/src/fixtures/panics.rs), [`unsafe_code`](../../xtask/lint-canary/src/fixtures/unsafe_code.rs), [`memory`](../../xtask/lint-canary/src/fixtures/memory.rs) | Panicking shortcuts (`unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`), `unsafe` and undocumented `unsafe` blocks, leaking values, a shared growable buffer, a future too large for the stack |
| [`time`](../../xtask/lint-canary/src/fixtures/time.rs), [`entropy`](../../xtask/lint-canary/src/fixtures/entropy.rs), [`randomness`](../../xtask/lint-canary/src/fixtures/randomness.rs) | Reading the clock, sleeping, OS entropy, unseeded RNGs and `RandomState` ([ADR-0011](../adr/0011-time-model-and-clock-injection.md), [ADR-0012](../adr/0012-seeded-randomness-and-hashing.md)) |
| [`tasks`](../../xtask/lint-canary/src/fixtures/tasks.rs), [`queues`](../../xtask/lint-canary/src/fixtures/queues.rs), [`locks`](../../xtask/lint-canary/src/fixtures/locks.rs), [`process`](../../xtask/lint-canary/src/fixtures/process.rs) | Threads and tasks started outside the supervisor, queues without a capacity, a blocking lock or `RefCell` borrow held across `.await`, ending the process without an orderly shutdown |
| [`suppression`](../../xtask/lint-canary/src/fixtures/suppression.rs) | The suppression rules: a bare `#[allow]` or an `#[expect]` without a reason must fire, and the sanctioned forms must not |

The modules follow the topics of [rust.md](../standards/rust.md); each module's `//!` doc names its rule.

## Kafka counterpart

Kafka's checkstyle configuration ([`checkstyle/`][checkstyle] at 4.4.0) plays the role of these lints. Kafka has no test of its own rules, which is the gap this crate closes. Nothing here mirrors Kafka behavior, so there are no `// Parity:` citations.

## State machines

None. It is fixture code that is compiled and linted, never run.

## Invariants

| Invariant | Where it is enforced |
|---|---|
| Every annotated lint fires on its line, and nothing else fires anywhere | `lint_canary::run` compares clippy's JSON diagnostics with the annotations and prints each `Mismatch` |
| Every `clippy.toml` `disallowed-methods` and `disallowed-types` entry fires at least once, with its configured reason | The reason check in `lint_canary.rs` compares each diagnostic's note with `clippy.toml` and lists entries that never fired |
| Every lint `[workspace.lints]` sets one by one, at a level other than `allow`, has a fixture. Groups such as `clippy::pedantic` are exempt | `check_fixture_coverage` in `lint_canary.rs` names each configured lint that no annotation mentions |
| The fixtures never reach a normal build | They compile only under the `canary` feature; [`tests/disallowed_exceptions.rs`](../../xtask/tests/disallowed_exceptions.rs) skips them |

## Events, metrics and flight recorder

None. The crate is never run.

## Errors

None. The crate has no API. Failures are reported by `cargo xtask lint-canary`, as described under [Debugging](#debugging).

## Tests

The crate is itself the test: the `lint-canary` CI step, in the `policy` job, runs `cargo xtask lint-canary`. The checker's own logic (annotation parsing, matching, reason checks) has unit tests in [`xtask/src/lint_canary.rs`](../../xtask/src/lint_canary.rs). Simulation, property, fuzz and compatibility tests don't apply to fixture code.

```sh
cargo xtask lint-canary     # or: cargo xtask ci --step lint-canary
```

### Adding a fixture

When a lint is added to `[workspace.lints]`, or an entry to `clippy.toml`:

1. Add a function that breaks it to the fixture module for its topic. Add a new module (and its line in [`fixtures/mod.rs`](../../xtask/lint-canary/src/fixtures/mod.rs)) only for a new topic.
2. End the offending line with `// expect-lint: <lint>`, spelled as clippy's JSON spells it (`unsafe_code`, `clippy::todo`), comma-separated when one line trips several. Keep the offending expression on a line of its own: rustfmt moves a comment that follows an opening `{` into the block.
3. If the call needs a crate the fixtures don't have yet, add it to `[dependencies]` as optional and to the `canary` feature, with a reason.
4. If the lint doesn't fire, check whether clippy skips the public API for it (`avoid-breaking-exported-api`, as for `rc_buffer`) and put the offending code in a private item.
5. Run `cargo xtask lint-canary`.

## Debugging

The command prints clippy's run first. Cargo's "could not compile" error is expected, because the fixtures trip deny-level lints. Then it prints one line per problem:

- `<file>:<line>: expected <lints>; clippy reported <lints>`: the annotation and the diagnostics differ on that line.
  - "clippy reported nothing" means the lint no longer fires: it was removed, renamed or weakened to `allow` in `[workspace.lints]` or `clippy.toml`. Restore it, or update the annotation in the same change as the standard ([rust.md](../standards/rust.md#workspace-configuration)).
  - An extra lint means a new lint fires on the fixture, for example after a toolchain update. Add it to the annotation if the fixture should trip it, or reshape the fixture so it doesn't.
- `` `<path>` fired without its configured reason ``: a diagnostic's note no longer carries the reason `clippy.toml` gives for that path.
- `` clippy.toml: `<path>` (<lint>) never fired ``: an entry has no fixture. Add one (see [Adding a fixture](#adding-a-fixture)).
- `` <lint> fired for `<path>`, which clippy.toml does not list ``: a fixture trips a disallowed lint for a path with no `clippy.toml` entry.
- `` Cargo.toml: `<lint>` (<level>) in [workspace.lints] has no fixture ``: a lint was added to the table without a fixture (see [Adding a fixture](#adding-a-fixture)).

A run with no problems ends with `lint-canary: ok (<n> annotated lints fired with their configured reasons, nothing else; every [workspace.lints] lint has a fixture)`.

[checkstyle]: https://github.com/apache/kafka/tree/4.4.0/checkstyle
