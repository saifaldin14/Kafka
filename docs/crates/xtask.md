# `xtask`: deep dive

## Responsibility and layer

`xtask` is the repository's automation, run as `cargo xtask <command>`. Every check CI runs is a step of `cargo xtask ci`, so CI and a developer's machine run exactly the same thing. It also holds the tools around the checks: the backlog commands, the pinned-tool installer, `doctor` and the optional pre-push hook. It never ships in a product binary. Codegen and the compatibility runner arrive later with their backlog tasks ([planned commands](../standards/rust.md#tooling-commands-contract-implemented-in-m0)).

It is a dev-layer crate in the [code map](../architecture.md#4-code-map): it may depend on anything except the binary, and nothing except tests depends on it ([ADR-0009](../adr/0009-crate-layering-and-dependency-rules.md)). Its only dependencies are `serde_json`, which reads `cargo metadata` and clippy's JSON, and `toml`, which reads manifests, `clippy.toml` and lockfiles. Argument parsing is std-only.

[`src/main.rs`](../../xtask/src/main.rs) is the entry point. Its module doc gives the exit codes and how to add a command; [`src/ci.rs`](../../xtask/src/ci.rs) gives how to add a CI step. [rust.md's tooling table](../standards/rust.md#tooling-commands-contract-implemented-in-m0) is the user-facing contract for every command and step.

| Module | Owns |
|---|---|
| [`cli`](../../xtask/src/cli.rs) | Parsing arguments into `Command`, the `help` list, and `UsageError` |
| [`ci`](../../xtask/src/ci.rs) | `STEPS`, the one table of CI steps in run order, and `cargo xtask ci` |
| [`steps`](../../xtask/src/steps.rs) | Running a step: the `==> [n/N]` header, inherited output, timing, the summary table |
| [`tools`](../../xtask/src/tools.rs), [`install`](../../xtask/src/install.rs) | `TOOLS`, the pinned external tools and their version checks; `tools install` |
| [`graph`](../../xtask/src/graph.rs), [`layers`](../../xtask/src/layers.rs), [`check_layers`](../../xtask/src/check_layers.rs) | The dependency graph from `cargo metadata`; the layer table; the layering rules |
| [`check_manifests`](../../xtask/src/check_manifests.rs) | `[workspace.lints]` and `clippy.toml` against rust.md, member manifests, `deny.toml` ignores, crate-root docs |
| [`check_workflows`](../../xtask/src/check_workflows.rs) | Pinned actions, triggers, job timeouts and permissions, ci.yml running every step, then actionlint and shellcheck |
| [`schemas`](../../xtask/src/schemas.rs) | `schemas verify`: the vendored Kafka schemas against their manifests; `schemas vendor`: copying them from a Kafka tag |
| [`crate_docs`](../../xtask/src/crate_docs.rs) | `docs check-crates` and `docs new-crate-doc` ([crate deep dives](README.md)) |
| [`lint_canary`](../../xtask/src/lint_canary.rs) | Running clippy on [`lint-canary`](../../xtask/lint-canary/src/lib.rs) and matching its `expect-lint:` annotations |
| [`backlog`](../../xtask/src/backlog.rs), [`backlog_check`](../../xtask/src/backlog_check.rs), [`backlog_next`](../../xtask/src/backlog_next.rs), [`backlog_stats`](../../xtask/src/backlog_stats.rs) | Parsing docs/backlog/, its rules, task selection and `show`, and the README totals ([backlog rules](../backlog/README.md)) |
| [`deps_diff`](../../xtask/src/deps_diff.rs) | `deps-diff`: the crates a change adds, removes or upgrades in `Cargo.lock`, for dependency review |
| [`doctor`](../../xtask/src/doctor.rs), [`hooks`](../../xtask/src/hooks.rs) | Local setup checks; the optional pre-push hook |
| [`output`](../../xtask/src/output.rs), [`clock`](../../xtask/src/clock.rs) | The only terminal writer; the only clock reader (`Stopwatch`) |

## Kafka counterpart

Kafka's Gradle build ([`build.gradle`][build-gradle] at 4.4.0): its checkstyle, spotbugs and test tasks play the role of these steps. No Kafka behavior is mirrored, so there are no `// Parity:` citations. The exception is the backlog of Kafka 4.4.0 work that `backlog` parses, which is data.

Differences from Kafka's build, by design:
- One table of steps, run the same way locally and in CI.
- Pinned tools verified by version (and by SHA-256 for release downloads).
- Policy checks that Kafka doesn't have: layering, lint canaries, workflow pinning and the backlog.

## State machines

None. Each command runs once, from arguments to an exit code. The closest thing to state is the step runner's `stopped` flag: after a failure, without `--keep-going`, every remaining step reports `Skipped`.

## Invariants

| Invariant | Where it is enforced |
|---|---|
| Each check is defined once and CI runs every PR step | `ci::STEPS` is the only list, and CI jobs pick from it with `--step`. `check_workflows::check_ci_steps` fails if no job in ci.yml runs a step that isn't on demand, or if a job names an unknown or on-demand step |
| The exit code is 0 for success, 1 for a failed check, 2 for a usage error | `Exit::code`, tested by `exit_codes_match_the_documented_contract`. End to end, [`tests/cli.rs`](../../xtask/tests/cli.rs) checks 0 and 2, and [`tests/check_layers.rs`](../../xtask/tests/check_layers.rs) checks 1 |
| A tool version appears in one place, and steps run only with the pinned version | `tools::TOOLS`; `tools::check` runs before every step that `requires` a tool. Release archives must match their SHA-256 before `install` unpacks them |
| Only `output` writes to the terminal, and a closed pipe never fails xtask's own output | `output` is the only module that ignores write errors: in `write_all`, and when `step_header` flushes, each with the reason beside it. A step's command writes to the terminal directly, and its own exit status decides its outcome |
| Only `clock::Stopwatch` reads the clock | clippy's `disallowed-methods`, with `xtask/src/clock.rs` in the sanctioned list of [`tests/disallowed_exceptions.rs`](../../xtask/tests/disallowed_exceptions.rs) ([ADR-0011](../adr/0011-time-model-and-clock-injection.md)) |
| The layer table equals architecture §4 and lists every member | `check_layers::check_architecture` and rule 1 of `check_layers`, run by the `layers` step |
| A check reports every problem it finds in one run, not just the first | The checkers collect their findings (`Vec`s of problem lines, `Violation`s, `Mismatch`es, `crate_docs::Findings`) before printing. The backlog parser collects every format error, and the commands that refuse to run on them print the first 20. Two inputs stop at their first error, because nothing after it can be trusted: a malformed `expect-lint:` annotation, and output from `cargo metadata` or clippy that isn't valid JSON |
| A hook xtask didn't write is never touched | `hooks` overwrites or removes only a hook carrying its marker line |

## Events, metrics and flight recorder

None. `xtask` is a short-lived command-line tool with no runtime to observe. Its observable output is the step headers, each step's own output, and the summary table with outcomes and durations ([`steps`](../../xtask/src/steps.rs)). The `build-times.yml` workflow records build timings separately.

## Errors

- **Usage.** `cli::UsageError` names the problem (unknown command, step or subcommand, a missing value or argument, non-UTF-8 input) and exits 2 after printing the usage text.
- **Loading.** `graph::GraphError` (running or parsing `cargo metadata`) and `tools::ToolProblem` (a tool missing or at the wrong version, with the install command) are typed.
- **Check results.** Most are lines of text naming the file (with a line, or a manifest key such as `workspace.lints.clippy.todo`), the rule and the fix, and they exit 1. They are reports for people, so they stay strings. Where tests or the output need structure, the result is typed: `check_layers::Violation` (an ordered enum, so violations print in a stable order), `lint_canary::Mismatch` (expected and reported lints at one location), and `crate_docs::Findings`.

There is no Kafka error mapping: `xtask` never speaks the Kafka protocol. `anyhow` is allowed here by [ADR-0004](../adr/0004-error-model-and-panic-policy.md), but isn't needed.

## Tests

| Level | Where | Covers |
|---|---|---|
| Unit | `mod tests` in each module except `clock` | Parsing; every checker rule, mostly on in-memory input (`backlog_check` reads the fixture backlogs); the step runner with small real commands, including one that can't start; `doctor::evaluate` with fake command output |
| Integration | [`tests/`](../../xtask/tests/) | `cli.rs`: the binary's dispatch, exit codes and step order. `check_layers.rs`: offline fixture workspaces for a correct workspace and five violations (rule 4, depending on the binary, is a unit test). `deny.rs`: deny.toml against fixture workspaces (needs the pinned cargo-deny). `disallowed_exceptions.rs`: the sanctioned clippy exceptions |
| Golden | [`tests/fixtures/backlog/`](../../xtask/tests/fixtures/backlog/) | The backlog parser's items and error messages, and the output of `check`, `next` and `stats` |
| Lint canary | [`xtask/lint-canary`](../../xtask/lint-canary/src/lib.rs), run by the `lint-canary` step | That every lint `[workspace.lints]` sets one by one, and every `clippy.toml` entry, still fires ([`lint-canary` deep dive](lint-canary.md)) |
| Property, simulation, fuzz, compatibility | None | `xtask` reads only repository files and tool output, not untrusted input, and has no distributed behavior |

Commands:

```sh
cargo nextest run -p xtask      # unit and integration tests
cargo xtask tools install       # the pinned tools that the deny tests and steps need
cargo xtask ci --list           # the steps, in run order
```

## Debugging

- **A step fails in CI but passes locally.** Run `cargo xtask doctor`: it reports a wrong `cargo` on `PATH` (Homebrew's ignores `rust-toolchain.toml`), the wrong toolchain, or a tool at the wrong version, each with its fix. Then run the same step alone with `cargo xtask ci --step <name>`.
- **Several steps fail.** Add `--keep-going` to see them all; the summary lists each outcome and duration.
- **A check reports a problem.** The line names the file, the line or manifest key, and the rule. The rule's text is in the checker's module doc, and the standard it enforces is linked from [rust.md's tooling table](../standards/rust.md#tooling-commands-contract-implemented-in-m0).
- **A layering failure is unclear.** `cargo xtask check-layers --print` shows the layer table that the dependency graph is checked against.
- **The lint canary fails.** The output names the fixture line and the lint that stopped firing or started firing; [`lint-canary`](../../xtask/lint-canary/src/lib.rs) explains the annotations.
- **The backlog commands refuse to run.** They print the format errors (the first 20) with their file and line; `cargo xtask backlog check` shows the rule violations.

[build-gradle]: https://github.com/apache/kafka/blob/4.4.0/build.gradle
