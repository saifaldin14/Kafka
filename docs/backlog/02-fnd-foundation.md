# FND: Workspace, tooling and CI

> Crates: workspace root, `xtask`, `.github/` · Kafka counterpart: Gradle build, checkstyle, CI · Milestones: M0-M1

Everything needed before the first line of product code: the Cargo workspace, pinned toolchain, lint and dependency policy, the `xtask` automation entry point, CI, and backlog tooling. M0 is done when `cargo xtask ci` passes in CI and enforces every machine-checkable rule in the standards.

## [Done] FND-F01 Cargo workspace and toolchain pin
P0 · M0 · Feature · deps: none
Scope: Root `Cargo.toml` workspace, `rust-toolchain.toml`, release/dev profiles, `.cargo/config.toml`, committed `Cargo.lock`.

### [Done] FND-0001 Create the Cargo workspace: root `Cargo.toml` with `[workspace]` (resolver 3) and the release profile, `rust-toolchain.toml` pinned to 1.99.0 with rustfmt and clippy, and a committed `Cargo.lock`
P0 · M0 · Task · deps: none · ref: docs/standards/rust.md
Done when: `Cargo.toml` has `[workspace]` with `resolver = "3"` and `members = []`, `[workspace.package]` (edition 2024, `rust-version = "1.99"`, `publish = false`) and an empty `[workspace.dependencies]`; `rustup show active-toolchain` in the repo reports 1.99.0 with rustfmt and clippy installed; `cargo generate-lockfile` produces the committed `Cargo.lock` (format version 4) with no diff; `[profile.release]` has exactly `panic = "abort"`, `overflow-checks = true`, `debug = "line-tables-only"`, `lto = "thin"`; members are listed explicitly because Cargo rejects a `crates/*` glob that matches nothing (the first crate switches to the glob); `.gitignore` still ignores `/target`.

## [In Progress] FND-F02 Lint configuration
P0 · M0 · Feature · deps: FND-F01
Scope: `[workspace.lints]`, `clippy.toml` and rustfmt exactly as in docs/standards/rust.md, plus smoke tests proving each disallowed item is rejected.

### [Done] FND-0002 Add `[workspace.lints.rust]` and `[workspace.lints.clippy]` to the root `Cargo.toml`, copied verbatim from docs/standards/rust.md
P0 · M0 · Task · deps: none · labels: nfr · ref: docs/standards/rust.md#workspace-configuration
Done when: Both tables match the rust.md block key for key, value for value and priority for priority (including `rust_2018_idioms` and `all`/`pedantic` at priority -1 and `unsafe_code = "forbid"`); the inline comments are kept; `cargo metadata` parses the manifest once the first member exists; no member crate exists yet, so enforcement is proven by the lint canary tasks.

### [Done] FND-0003 Add the root `clippy.toml` verbatim from docs/standards/rust.md (test allowances, 16 `disallowed-methods`, 1 `disallowed-types`, each with its reason)
P0 · M0 · Task · deps: FND-0002, FND-0018 · labels: nfr · ref: docs/standards/rust.md#workspace-configuration
Done when: File content equals the rust.md block; `cargo clippy --workspace --all-targets -- -D warnings` passes on the xtask crate with no clippy configuration warnings, even though `tokio` and `rand` are not in the dependency graph; if clippy 1.99 warns about unresolvable paths, the fix (for example `allow-invalid = true` on those entries) is applied to clippy.toml and rust.md together in this PR.

### [Done] FND-0004 `cargo xtask lint-canary`: annotated fixture crate proving the forbid/deny lints fire
P0 · M0 · Task · deps: FND-0003 · labels: test · ref: docs/standards/rust.md#workspace-configuration
Done when: A dev-layer `xtask/lint-canary` crate holds fixture files with `// expect-lint: <lint>` annotations; the command compiles it with clippy JSON output and compares diagnostics with the annotations; the first fixtures cover `unsafe_code`, `unwrap_used`, `todo`, `unimplemented`, `dbg_macro`, `print_stdout`, `print_stderr`, `exit` and `mem_forget`. Fixture code is compiled only with the crate's `canary` feature, so normal workspace builds stay clean; the command fails if an annotated lint does not fire, or if an unannotated diagnostic appears, and prints file:line with the expected and actual lint; removing one lint from `[workspace.lints]` makes the command fail (shown in the PR); `lint-canary` is listed as a dev crate in the layer table.

### [Done] FND-0005 Lint canary fixtures for every `disallowed-methods` entry and for `RandomState`
P0 · M0 · Task · deps: FND-0004 · labels: test · ref: docs/standards/rust.md#workspace-configuration
Done when: Fixtures cover the std time and thread functions, `tokio::spawn`, `tokio::task::spawn`, `tokio::time::sleep`, `tokio::time::Instant::now`, `tokio::sync::mpsc::unbounded_channel`, `rand::rng`, `rand::thread_rng`, the four `HashMap`/`HashSet` constructors and `std::process::exit`. All 17 fixtures fire `clippy::disallowed_methods` or `clippy::disallowed_types`, and each assertion also checks the configured reason text; `tokio` and `rand` are added to `[workspace.dependencies]` as optional canary-only dependencies, with the justification from docs/standards/security.md#dependencies; `cargo deny check` still passes.
Notes: rand 0.10 removed `rand::thread_rng`, and `tokio::task::spawn` is the same function as `tokio::spawn` (clippy matches functions, so that second entry only replaced the first's reason). Both entries were removed from clippy.toml and rust.md, leaving 14 methods and 1 type, covered by 16 fixtures (both tokio spawn spellings included). FND-0063 covers rand 0.10's other unseeded entry points.

### [Done] FND-0006 Lint canary fixtures for the suppression rules and the remaining warn-level lints
P0 · M0 · Task · deps: FND-0004 · labels: test · ref: docs/standards/rust.md#panics
Done when: Covered: a bare `#[allow]` fires `allow_attributes`; `#[expect]` without a reason fires `allow_attributes_without_reason`; the sanctioned forms pass (`#[expect(clippy::disallowed_methods, reason = "…")]` wrapper, `#[expect(clippy::expect_used, reason = "invariant")]` with `expect("invariant: …")`); `await_holding_lock`, `string_slice`, `missing_docs` and `unreachable_pub` fire. Negative fixtures fire exactly the named lint and positive fixtures produce zero diagnostics; `await_holding_lock` is exercised with a `std::sync::Mutex` held across `.await` in an `async fn` without any runtime dependency; the sanctioned-wrapper pattern is documented in the canary crate's `//!` doc with a link to rust.md.

### [Done] FND-0063 clippy.toml bans rand 0.10's other unseeded entry points, not only `rand::rng`
P1 · M0 · Task · deps: FND-0005 · labels: test · ref: docs/standards/rust.md#workspace-configuration
Done when: Every public function in rand 0.10 (the version in `Cargo.lock`) that draws from the thread-local or OS RNG without a caller-supplied seed is listed, from reading its source (for example `rand::random`, `random_iter`, `random_range`, `random_bool`, `random_ratio`, `fill` and OS-seeded constructors); each is added to clippy.toml and the rust.md block with the reason "use the injected seeded RNG"; each has a lint canary fixture; `cargo xtask lint-canary` passes.

### [Done] FND-0064 clippy.toml bans the remaining clock, timer and entropy entry points from ADR-0011 and ADR-0012
P1 · M0 · Task · deps: FND-0004 · labels: test · ref: ADR-0011, ADR-0012
Done when: `disallowed-methods` gains `std::time::Instant::elapsed`, `std::time::SystemTime::elapsed`, `tokio::time::sleep_until`, `tokio::time::timeout`, `tokio::time::timeout_at`, `tokio::time::interval`, `tokio::time::interval_at`, `getrandom::fill`, `getrandom::fill_uninit`, `getrandom::u32` and `getrandom::u64`, in clippy.toml and the rust.md block together; reasons point to `Clock`, `Timer` and `SeededRng::from_os_entropy`; every new entry has a lint canary fixture; `xtask`'s `Stopwatch` carries the matching `#[expect]`; `cargo xtask ci` passes.

### [Done] FND-0007 `cargo xtask check-manifests` (first rules): `[workspace.lints]` and `clippy.toml` must equal the TOML blocks in docs/standards/rust.md (parsed and compared semantically), and every workspace member must declare `[lints] workspace = true`
P0 · M0 · Task · deps: FND-0003, FND-0018 · labels: docs · ref: docs/standards/rust.md#workspace-configuration
Done when: Unit tests with fixture manifests cover: a changed lint level, an added or removed disallowed method, and a member without `[lints] workspace = true`; each fails with a message naming the file and the differing key; the real repository passes; the command is registered as the `manifests` step of `cargo xtask ci`.

### [To Do] FND-0008 `check-manifests` rule: `alluvium-protocol` and `alluvium-records` `lib.rs`, and `alluvium-storage` decoding modules listed in the rule, start with `#![deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)]`
P2 · M1 · Task · deps: FND-0007 · labels: decoder, security · ref: docs/standards/rust.md#workspace-configuration
Done when: The rule applies only to crates present in the workspace; fixture tests show a decoder crate without the attribute is rejected, with a message pointing to rust.md; the rule passes once `alluvium-protocol` exists (M1).

## [In Progress] FND-F03 Dependency policy
P0 · M0 · Feature · deps: FND-F01
Scope: `deny.toml` (advisories, licenses, bans, sources), `cargo deny` wiring, dependency review checklist.

### [Done] FND-0009 `deny.toml` `[advisories]`: deny vulnerabilities, `unmaintained = "all"`, `yanked = "deny"`, empty `ignore` list; `[graph]` targets limited to the supported platforms
P0 · M0 · Task · deps: FND-0018 · labels: security · ref: docs/standards/security.md#dependencies
Done when: `cargo deny check advisories` passes on the workspace; the file header comment links to docs/standards/security.md#dependencies; the `[graph]` targets list x86_64 and aarch64 Linux and aarch64 macOS (the NFR-F10 support matrix replaces it if it differs).

### [Done] FND-0010 `check-manifests` rule: every `[advisories] ignore` entry has a `reason` that ends in `(expires YYYY-MM-DD)`, and expired entries fail
P1 · M0 · Task · deps: FND-0009, FND-0007 · labels: security · ref: docs/standards/security.md#dependencies
Done when: Fixture tests cover: a missing reason, a missing expiry, a malformed date, and an expired date (evaluated against an injected date, so the test is deterministic); the failure message names the advisory id and the expiry; security.md documents the reason format.
Notes: Reduced to requiring `reason` and `expiry` on every entry. Checked with cargo-deny 0.22 against RUSTSEC-2026-0374: it reports an ignore whose `expiry` has passed (2000-01-01) again, rejects a malformed date (2099-13-45) as a config error, and accepts entries without `reason` or `expiry`. So xtask checks only that both fields are present, and parses no dates and reads no clock.

### [Done] FND-0011 `deny.toml` `[licenses]`: the exact allow-list from security.md (MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, Zlib), `confidence-threshold = 0.93`, workspace crates ignored as private (`publish = false`), no exceptions
P0 · M0 · Task · deps: FND-0009 · labels: security · ref: docs/standards/security.md#dependencies
Done when: `cargo deny check licenses` passes; adding a fixture dependency with a GPL-3.0 license fails with the license named; any exception added later needs a comment with the crate, the license and the reason.

### [Done] FND-0012 `deny.toml` `[bans]`: `multiple-versions = "warn"`, `wildcards = "deny"`, deny `openssl`, `openssl-sys` and `native-tls` with the reason "TLS is rustls only (ADR-0006)"
P0 · M0 · Task · deps: FND-0009 · labels: security · ref: ADR-0006
Done when: `cargo deny check bans` passes; an offline fixture workspace under `xtask/tests/fixtures/deny-bans/` with a path crate named `openssl-sys` is rejected by `cargo deny --manifest-path … check bans`, and an xtask test asserts the banned-crate diagnostic; a wildcard version in the fixture is rejected.

### [Done] FND-0013 `deny.toml` `[sources]`: `unknown-registry = "deny"`, `unknown-git = "deny"`, crates.io as the only allowed registry, no allowed git sources
P0 · M0 · Task · deps: FND-0009 · labels: security · ref: docs/standards/security.md#dependencies
Done when: `cargo deny check sources` passes; a fixture with a git dependency fails with the source named; security.md's "crates.io only" rule links to this section.

### [Done] FND-0014 Register `cargo deny check` as the `deny` step of `cargo xtask ci`, using the cargo-deny version pinned in the xtask tool table
P0 · M0 · Task · deps: FND-0012, FND-0011, FND-0013, FND-0020, FND-0021 · labels: security · ref: docs/standards/rust.md#tooling-commands-contract-implemented-in-m0
Done when: `cargo xtask ci --step deny` runs all four checks and fails with the cargo-deny diagnostics on any violation; a missing or wrong-version cargo-deny gives an error with the install command; the rust.md tooling table is still accurate.

### [Done] FND-0015 `check-manifests` rule: member crates declare every dependency as `{ workspace = true }` (optionally adding `features`/`optional`), never with an inline version, path or git source
P1 · M0 · Task · deps: FND-0007 · labels: security · ref: docs/standards/rust.md#dependencies
Done when: Fixture tests cover inline versions, inline paths for non-workspace crates, and git sources, each rejected with the crate and dependency named; workspace-internal path dependencies are declared in `[workspace.dependencies]` too, so the rule has no exceptions.

### [To Do] FND-0016 Dependency review aid: `cargo xtask deps-diff <base-ref>` and a PR template link to the review questions
P2 · M1 · Task · deps: FND-0014 · labels: security, docs · ref: docs/standards/security.md#dependencies
Done when: The command lists added, removed and upgraded crates from the two `Cargo.lock` files with license and transitive count; the PR template's dependency checkbox links to the five questions in security.md#dependencies. The command works offline from the two lockfiles plus `cargo metadata`; a unit test on two fixture lockfiles checks the diff; CI writes the output to the job summary when `Cargo.lock` changes; the questions are linked, not copied.

### [Done] FND-0017 `.github/dependabot.yml` for the `cargo` ecosystem: weekly, one PR per dependency, lockfile updates labelled `dependencies`
P1 · M0 · Task · deps: none · labels: security · ref: docs/standards/security.md#dependencies
Done when: Dependabot's config validation passes on GitHub; the policy "dependency updates are separate PRs" in security.md is now automated and the doc says so.
Notes: Verified after the push of c9fa497: GitHub's `.github/dependabot.yml` check run succeeded, and two Dependabot `cargo in /.` update runs completed successfully.

## [Done] FND-F04 `xtask` framework
P0 · M0 · Feature · deps: FND-F01
Scope: `xtask` crate, command dispatch, `cargo xtask ci` orchestration with clear step output and exit codes.

### [Done] FND-0018 Create the `xtask` crate with command dispatch, typed usage errors and exit codes, and the `cargo xtask` alias
P0 · M0 · Task · deps: FND-0002 · ref: ADR-0009
Done when: The crate is a workspace member with `[lints] workspace = true`; `.cargo/config.toml` has `[alias] xtask = "run --package xtask --"`; exit codes are 0 (success), 1 (check failed) and 2 (usage error). `cargo xtask help` lists the available commands; an unknown command exits 2 with the usage; arguments are parsed with std or one minimal crate justified per security.md#dependencies; the `//!` doc states the responsibility and explains how to add a command and a ci step (later commands such as `codegen`, `fuzz` and `compat` arrive with their own epics; nothing is stubbed); unit tests cover dispatch and the exit codes.
Notes: exit code 1 is documented in the crate doc and first produced by the step runner (FND-0019); no check exists before it, and an unused variant would be dead code.

### [Done] FND-0019 xtask step runner: run an external command as a named step with a `==> [n/N] <step>` header, inherited output, elapsed time and exit status, then print a summary table of all steps
P0 · M0 · Task · deps: FND-0018 · ref: ADR-0009
Done when: Each step returns a typed result (passed, failed with exit code, or could not start with the OS error); the summary marks each step with ok/FAILED and its duration; the overall exit code is 1 if any step failed; `--keep-going` runs the remaining steps after a failure (by default the runner stops at the first one); unit tests use a step that spawns `cargo --version` and one that cannot start.

### [Done] FND-0020 One table of pinned external tool versions in xtask (cargo-nextest, cargo-deny, plus later tools added there), with `cargo xtask tools install` (`cargo install --locked --version …`) and a version check before steps that need a tool
P0 · M0 · Task · deps: FND-0019 · ref: docs/standards/rust.md#tooling-commands-contract-implemented-in-m0
Done when: The tool table is the only place tool versions appear (CI and `doctor` read it); a step whose tool is missing or the wrong version fails before running, with the exact install command; a unit test parses `--version` output for each listed tool.

### [Done] FND-0021 `cargo xtask ci` orchestration with steps `fmt` (`cargo fmt --all --check`), `clippy` (`--workspace --all-targets --locked -- -D warnings`) and `test` (`cargo nextest run --workspace --locked --profile ci`), plus `--step <name>` (repeatable) and `--list`
P0 · M0 · Task · deps: FND-0019, FND-0020 · ref: docs/standards/rust.md#tooling-commands-contract-implemented-in-m0
Done when: `cargo xtask ci` runs the steps in the documented order and exits non-zero if any step fails; `--list` prints every registered step in order; later tasks register `doc`, `deny`, `layers`, `manifests`, `lint-canary`, `links`, `workflows` and `backlog` in the same table; the rust.md tooling table and CONTRIBUTING describe the commands as they now exist.

### [Done] FND-0022 `.config/nextest.toml` with `default` and `ci` profiles: no retries (flaky tests are bugs), `fail-fast = false` in ci, slow-test warning at 60 s and termination at 300 s, JUnit output at `target/nextest/ci/junit.xml`
P0 · M0 · Task · deps: FND-0021 · labels: test · ref: docs/standards/testing.md#rules
Done when: `cargo nextest run --profile ci` writes the JUnit file; a deliberately slow test (not committed) is reported as slow and then terminated, as shown in the PR; testing.md links to the profile and explains why retries are 0.

## [In Progress] FND-F05 Crate layer checker
P0 · M0 · Feature · deps: FND-F04
Scope: `cargo xtask check-layers`: layer table, forbidden runtime dependencies for layers 0-2, allow-list with ADR references, sync check with docs/architecture.md.

### [Done] FND-0023 Layer table in `xtask/src/layers.rs`: one constant mapping every crate from architecture §4 (layers 0-5 and dev, including `alluvium-testkit`, `xtask` and `lint-canary`) to a `Layer` enum, printed by `cargo xtask check-layers --print`
P0 · M0 · Task · deps: none · ref: ADR-0009
Done when: The table lists all target crates, including ones not created yet (it is data, not code paths); `--print` shows crate, layer and whether it exists in the workspace; the module `//!` doc states that this table and architecture §4 must agree and that `check-layers` enforces it.

### [Done] FND-0024 Load the dependency graph for `check-layers` from `cargo metadata --format-version 1 --locked` (with `--manifest-path` for fixtures): workspace members, resolved nodes, and edges with their dependency kinds (normal, build, dev)
P0 · M0 · Task · deps: FND-0023 · ref: ADR-0009
Done when: The parsed model is unit-tested against a committed metadata JSON fixture; the parsing crate (`cargo_metadata` or `serde_json`) is justified per security.md#dependencies; a missing `cargo` or a failed `cargo metadata` produces an error that includes cargo's stderr.

### [Done] FND-0025 check-layers rule: every workspace member must appear in the layer table
P0 · M0 · Task · deps: FND-0024 · ref: ADR-0009
Done when: A member that is not in the table fails with `crate <name> has no layer: add it to xtask/src/layers.rs and docs/architecture.md §4`; covered by a fixture test.

### [Done] FND-0026 check-layers rules for edges between workspace crates: layer order, same-layer allow-list, dev crates, binary
P0 · M0 · Task · deps: FND-0024 · ref: ADR-0009
Done when: Rules: normal and build dependencies must point to a strictly lower layer; same-layer edges only through the allow-list; dev crates only as dev-dependencies; nothing depends on the layer 5 binary. Each violation prints `<from> (layer a) -> <to> (layer b): <rule>` and the command exits 1; every violation is reported in one run, not just the first; unit tests on synthetic graphs cover each rule.

### [Done] FND-0027 check-layers rule: no async runtime or networking crate under layers 0-2, directly or transitively
P0 · M0 · Task · deps: FND-0024 · labels: security · ref: ADR-0009
Done when: Deny-list: `tokio`, `tokio-util`, `mio`, `async-std`, `smol`, `async-io`, `async-executor`, `futures-executor`, `hyper`, `h2`, `reqwest`, `tonic`, `axum`, `tower`, `socket2`; only normal and build edges count. The violation message prints the full path (for example `alluvium-records -> foo -> mio`); dev-dependencies of layer 0-2 crates are not flagged; the deny-list is one constant, and each entry has a reason; unit tests cover direct and three-hop transitive cases.

### [Done] FND-0028 check-layers same-layer allow-list: entries `{ from, to, adr }`, where the ADR file must exist in docs/adr/ with `Status: Accepted`; unused entries are errors
P1 · M0 · Task · deps: FND-0026 · ref: ADR-0009
Done when: The allow-list ships empty (architecture §5 says there are no exceptions today); unit tests use injected entries to show: an allowed same-layer edge passes, an entry with a missing ADR fails, a Proposed ADR fails, and an unused entry fails.

### [Done] FND-0029 Deliberately wrong dependency tests: offline fixture workspaces for every check-layers rule
P0 · M0 · Task · deps: FND-0025, FND-0026, FND-0027 · labels: test · ref: ADR-0009
Done when: Fixtures in `xtask/tests/fixtures/layers/` use path dependencies only and are each their own `[workspace]` root: `upward` (types -> server), `same-layer` (raft -> storage), `runtime-in-core` (records -> local stub -> a local crate named `tokio`), `dev-as-normal` (server -> testkit), `unknown-crate`, plus one passing fixture. An xtask integration test runs real `cargo metadata` plus check-layers on each fixture and asserts the exact violation lines and exit code 1 (exit code 0 for the passing fixture); the tests run in `cargo xtask ci` without network access; this is the M0 exit evidence that "the layer check rejects a deliberately wrong dependency".

### [Done] FND-0030 check-layers sync with docs/architecture.md: parse the §4 code-map table (layer and crate columns) and require the same crate set and layers as the xtask table; register `check-layers` as the `layers` step of `cargo xtask ci`
P0 · M0 · Task · deps: FND-0023, FND-0021 · labels: docs · ref: ADR-0009
Done when: Moving a crate to another layer in only one of the two places fails with a diff that lists crates missing on each side and layer mismatches; a test runs against a copy of the real architecture.md; `cargo xtask ci --step layers` runs on the real workspace; architecture §5 and ADR-0009 point to the command.

### [To Do] FND-0065 `check-layers` source rule: no layer 1–2 crate names `SystemClock` or `SeededRng::from_os_entropy` (ADR-0011, ADR-0012)
P2 · M1 · Task · deps: FND-0030 · ref: ADR-0011
Done when: The rule scans the `src/` of layer 1–2 workspace crates (layer 0 is `alluvium-types`, which defines them) and reports each use with file:line and the ADR; fixture crates show a use rejected and a doc comment mentioning the names accepted; the real workspace passes.

## [In Progress] FND-F06 CI pipeline
P0 · M0 · Feature · deps: FND-F02, FND-F03, FND-F04, FND-F05
Scope: GitHub Actions running `cargo xtask ci` on Linux and macOS, caching, SHA-pinned actions, required checks.

### [Done] FND-0031 `.github/workflows/ci.yml` skeleton with least-privilege defaults and the `lint` job
P0 · M0 · Task · deps: FND-0021 · labels: security · ref: docs/standards/security.md#dependencies
Done when: Triggers are `pull_request`, `push` to main and `merge_group`; workflow-level `permissions: {}` and per-job `contents: read`; per-ref concurrency with cancel-in-progress for PRs; `actions/checkout` with `persist-credentials: false`; `timeout-minutes` on every job; the `lint` job runs `cargo xtask ci --step fmt --step clippy` on ubuntu-24.04. All actions are pinned to full commit SHAs with a `# vX.Y.Z` comment; the toolchain comes from `rust-toolchain.toml` through the runner's rustup, with no third-party toolchain action; a PR with a formatting error fails the `lint` job.
Notes: Verified on GitHub once the repository was made public (the private repository's Actions were blocked by billing). PR #1's run [44934033847](https://github.com/saifaldin14/Kafka/actions/runs/44934033847) passed every job, and a PR with a formatting error ([#3](https://github.com/saifaldin14/Kafka/pull/3), run [44934858823](https://github.com/saifaldin14/Kafka/actions/runs/44934858823)) failed `lint`. actionlint 1.7.12 reports nothing on the workflows.

### [Done] FND-0032 CI caching: SHA-pinned `Swatinem/rust-cache` keyed on OS, toolchain and `Cargo.lock`, which also caches the tools installed by `cargo xtask tools install`; only pushes to main save the cache, PRs restore only
P0 · M0 · Task · deps: FND-0031 · labels: perf
Done when: A second run of an unchanged PR restores the cache and skips tool installation (cold and warm times are in the PR description); PR runs cannot write the cache that main uses (`save-if` on main only), which protects against cache poisoning.
Notes: Cold times on PR #1, with tools built from source: lint 0.4 min, policy 0.4, deny 2.3, advisories 1.9, docs 3.6, test 6.2 (ubuntu-24.04) and 7.0 (macos-15). After main's first run saved the cache, PR #10's run [44938949556](https://github.com/saifaldin14/Kafka/actions/runs/44938949556) restored it with a full key match in every job, and "All tools are installed at their pinned versions". Warm times: lint 0.3, policy 0.4, deny 0.4, advisories 0.3, docs 0.4, test 0.4 (ubuntu-24.04) and 0.6 (macos-15). Every cache entry is on `refs/heads/main`; none came from the 11 PR runs (`save-if: false`).

### [Done] FND-0033 CI `test` job: matrix of ubuntu-24.04 and macos-15 (arm64), `fail-fast: false`, running `cargo xtask ci --step test` and uploading the nextest JUnit report as an artifact
P0 · M0 · Task · deps: FND-0031, FND-0022 · labels: test · ref: docs/standards/testing.md#rules
Done when: Both matrix legs run on every PR; a failing test fails only its leg and the report artifact is downloadable; runner images are pinned by label (not `-latest`), and the labels are recorded in the NFR-F10 portability matrix once it exists.
Notes: Both legs ran on every PR. A test failing only on macOS ([#7](https://github.com/saifaldin14/Kafka/pull/7), run [44934882076](https://github.com/saifaldin14/Kafka/actions/runs/44934882076)) failed `test (macos-15)` while `test (ubuntu-24.04)` passed, and both `junit-<os>` artifacts were downloadable. The first CI run found a real bug: the cargo-deny fixture tests depended on terminal colour, fixed in fdc054f. NFR-F10's matrix doesn't exist yet; it should record ubuntu-24.04 and macos-15.

### [Done] FND-0034 CI `docs` job running `cargo xtask ci --step doc --step links`
P0 · M0 · Task · deps: FND-0031, FND-0054, FND-0055 · labels: docs
Done when: An intentionally broken intra-doc link or Markdown anchor in a test PR fails the job; the job runs on Linux only, with a documented reason (rustdoc output does not depend on the platform).
Notes: A PR with a broken intra-doc link and a broken Markdown anchor ([#5](https://github.com/saifaldin14/Kafka/pull/5), run [44934873977](https://github.com/saifaldin14/Kafka/actions/runs/44934873977)) failed both steps: rustdoc reported "unresolved link to `NoSuchType`" and lychee "Cannot find fragment".

### [Done] FND-0035 CI dependency jobs: `deny` (bans, licenses, sources) as a required check, plus a separate `advisories` job that runs on PRs without blocking them and on a daily schedule that opens or updates a tracking issue when it fails
P0 · M0 · Task · deps: FND-0031, FND-0014 · labels: security · ref: docs/standards/security.md#dependencies
Done when: A newly published advisory cannot block unrelated PRs, yet it is surfaced within a day as an issue with the advisory id; the scheduled job has `issues: write` only; a banned crate fails the required `deny` job.
Notes: A banned `openssl-sys` ([#6](https://github.com/saifaldin14/Kafka/pull/6), run [44934876168](https://github.com/saifaldin14/Kafka/actions/runs/44934876168)) failed `deny` with "explicitly banned". `advisories` ran on every PR without failing the run. advisories.yml dispatched from main succeeded (run [44938924572](https://github.com/saifaldin14/Kafka/actions/runs/44938924572); no advisories today). Its issue step ran once accidentally against GitHub, opening issue #2 (since deleted) with deduplicated advisory ids, and was tested against a stand-in `gh` for the comment path.

### [Done] FND-0036 CI `policy` job: `cargo xtask ci --step layers --step manifests --step lint-canary` (later policy steps such as `backlog` and `workflows` are added to this job when they land)
P0 · M0 · Task · deps: FND-0031, FND-0030, FND-0007, FND-0004 · ref: ADR-0009
Done when: A PR that adds an upward crate dependency fails this job with the check-layers message; a PR that weakens a lint in `[workspace.lints]` fails `manifests` and `lint-canary`.
Notes: An upward crate dependency ([#8](https://github.com/saifaldin14/Kafka/pull/8), run [44934887613](https://github.com/saifaldin14/Kafka/actions/runs/44934887613)) failed `policy` with "alluvium-types (layer 0) -> alluvium-storage (layer 1): a crate may depend only on lower layers". Setting `todo` to `allow` ([#4](https://github.com/saifaldin14/Kafka/pull/4), run [44935062251](https://github.com/saifaldin14/Kafka/actions/runs/44935062251)) failed both `manifests` and `lint-canary`.

### [Done] FND-0037 CI `ci-success` aggregator job (`needs:` every required job, `if: always()`) as the single required status check
P0 · M0 · Task · deps: FND-0033, FND-0034, FND-0035, FND-0036
Done when: The job fails when any needed job failed, was cancelled or was skipped unexpectedly, and passes only when all succeeded; adding a new required job means adding it to `needs:`, which the workflow comment states; FND-F11's repository settings name `ci-success` as the required check.
Notes: `ci-success` failed in all six demonstration PRs ([#3](https://github.com/saifaldin14/Kafka/pull/3) to [#8](https://github.com/saifaldin14/Kafka/pull/8)), each with exactly one other failing job, and passed on [44934033847](https://github.com/saifaldin14/Kafka/actions/runs/44934033847). Making it a required check is a repository setting that is now possible because the repository is public; FND-0061 (repository settings and ruleset) does it.

### [Done] FND-0038 `cargo xtask check-workflows` (SHA pinning, no `pull_request_target`, timeouts, permissions) and Dependabot for actions
P1 · M0 · Task · deps: FND-0031, FND-0007 · labels: security · ref: docs/standards/security.md#dependencies
Done when: Every `uses:` in `.github/workflows/*.yml` must be pinned to a 40-hex SHA with a `# v…` comment; every job sets `timeout-minutes` and explicit `permissions`; Dependabot covers the `github-actions` ecosystem weekly. Fixture workflows (tag-pinned action, missing timeout, missing permissions, `pull_request_target`) are each rejected with file:line; the real workflows pass; registered as the `workflows` step in the `policy` job; Dependabot opens SHA-bump PRs that keep the version comment.
Notes: check-workflows is verified locally and runs in `policy` on every PR. Dependabot validated the new `github-actions` entry and ran it from main ([run 44937154825](https://github.com/saifaldin14/Kafka/actions/runs/44937154825), success, no updates). Closed at the user's request: the last criterion, a Dependabot PR that bumps a SHA and keeps the version comment, can only be observed after an action publishes a new release, since all of them are at their latest release today. Check the first such PR when it arrives.

### [Done] FND-0039 Lint workflow files with actionlint (version pinned in the xtask tool table) as part of the `workflows` step
P2 · M0 · Task · deps: FND-0038
Done when: A workflow with an invalid expression or an unknown runner label fails the step locally and in CI; shellcheck findings in `run:` blocks are reported (actionlint's integration); the current workflows pass.

### [Done] FND-0066 CI `doctest` step: `cargo test --workspace --doc --locked`, run in the `test` job on both OSes, because nextest does not run doctests
P0 · M0 · Task · deps: FND-0021, FND-0033
Done when: `cargo xtask ci --list` shows `doctest` after `test`; a failing doctest and a `compile_fail` doctest that compiles each fail the step; docs/standards/rust.md lists the step; the `test` job runs `test doctest`.
Notes: Proved locally: a doctest asserting `1 == 2` fails the step ("FAILED", exit 1), a `compile_fail` example that compiles fails it ("Test compiled successfully, but it's marked `compile_fail`"), and the workspace passes. TYPE-0024's compile-fail test is the first real doctest.

### [To Do] FND-0040 M0 exit: `cargo xtask ci` is green in CI on main for Linux and macOS, a draft PR with a deliberately wrong crate dependency is rejected by the `policy` job, and the phase docs are updated
P0 · M0 · Task · deps: FND-0037, FND-0038, FND-0045 · labels: docs
Done when: Both runs are linked in the PR; the "Current phase" section of the engineering skill, AGENTS.md ("design phase"), CONTRIBUTING.md ("contract that milestone M0 implements") and the M0 row of docs/parity.md describe the new state; the remaining open M0 tasks (if any) are listed in the PR as non-blocking for starting M1 work.

## [Done] FND-F07 Backlog tooling
P1 · M0 · Feature · deps: FND-F04
Scope: Port `docs/backlog/tools/backlog.py` (check, next, stats) to `cargo xtask backlog` and run `backlog check` in CI.

### [Done] FND-0041 `cargo xtask backlog` parser: read `docs/backlog/*.md` into typed items (`Status`, `Priority`, `Milestone`, item type, deps, labels, ref, `Done when`, trailers) following the README item format, with file:line on every parse error
P1 · M0 · Task · deps: none · labels: docs · ref: docs/backlog/README.md
Done when: Parsing the real backlog gives the same item count and fields as `backlog.py` (checked by a test that compares against a committed JSON golden produced once by `backlog.py`); malformed headings, metadata lines and missing `Done when` are reported with file:line; unit tests use the README examples.
Notes: The committed goldens cover fixture backlogs in `xtask/tests/fixtures/backlog/`, because a golden of the real backlog would go stale with every status change. On the real backlog, both parsers gave the same 7,836 items field for field, checked once when the port landed.

### [Done] FND-0042 `cargo xtask backlog check [--allow-empty-features] [--epic CODE]` with every rule of `backlog.py check`
P1 · M0 · Task · deps: FND-0041 · labels: docs · ref: docs/backlog/README.md
Done when: Rules: id formats, duplicates, unknown deps, cross-epic deps only on features, no deps on later-milestone tasks, feature milestone = earliest task, feature status derived from tasks, Blocked/Won't Do trailers, cycles, README epic table and totals. For each rule, a fixture backlog triggers exactly the error text that `backlog.py` produces (committed goldens); the real backlog gives identical output from both tools; exit code 1 on any error.

### [Done] FND-0043 `cargo xtask backlog next [--all] [-n N]` and `show <ID>`: readiness with feature-dependency expansion by milestone, ordered by milestone, priority, epic table order and id
P1 · M0 · Task · deps: FND-0041 · labels: docs · ref: docs/backlog/README.md
Done when: Golden tests on fixture backlogs cover: a blocked task, a feature dependency satisfied only up to the task's milestone, claimed tasks skipped, and working ahead when the current milestone has nothing ready; the output on the real backlog matches `backlog.py next --all`.

### [Done] FND-0044 `cargo xtask backlog stats [--write]`: per-epic and per-milestone totals rewritten between the README `backlog-stats` markers
P2 · M0 · Task · deps: FND-0041 · labels: docs · ref: docs/backlog/README.md
Done when: The generated table is byte-identical to `backlog.py stats` on the real backlog; `--write` changes only the text between the markers; `check` (without `--allow-empty-features`) fails when the totals are stale.
Notes: `stats` and `--write` landed first, byte-identical on the real backlog. The stale-totals failure arrived with `check` (FND-0042).

### [Done] FND-0045 Run `cargo xtask backlog check` as the `backlog` step in the CI `policy` job, delete `docs/backlog/tools/backlog.py`, and point the README, the engineering skill and CONTRIBUTING to `cargo xtask backlog`
P1 · M0 · Task · deps: FND-0042, FND-0043, FND-0044, FND-0036 · labels: docs · ref: docs/backlog/README.md
Done when: CI fails a PR that breaks the backlog format or a dependency rule; no file references `backlog.py` (grep is clean); the golden tests no longer need Python.
Notes: A PR breaking the backlog format ([#9](https://github.com/saifaldin14/Kafka/pull/9), run [44936714614](https://github.com/saifaldin14/Kafka/actions/runs/44936714614)) failed `policy` at the `backlog` step: "01-type-domain-types.md:19: malformed heading", plus stale totals. `git grep backlog.py` matches only FND-F07's own task descriptions, which record what was ported; every instruction, doc, code comment and error message now points to `cargo xtask backlog`. The goldens are the specification now, so their two Python-specific details changed: Rust string quoting for a malformed metadata line, and the item count for a backlog with invalid items.

## [To Do] FND-F08 Licensing and attribution
P1 · M0 · Feature · deps: none
Scope: Project license decision, `NOTICE` file, Apache-2.0 attribution for vendored Kafka material.

### [To Do] FND-0046 Spike: choose the project license and contributor terms (Apache-2.0, MIT OR Apache-2.0, a copyleft or source-available license, or staying proprietary; DCO or CLA), with compatibility with included Apache Kafka material as a constraint
P1 · M0 · Spike · deps: none · labels: docs · ref: ADR-0008
Done when: An accepted ADR "Project license and contributor terms" records the license, the contributor terms, whether files carry SPDX headers, and the consequences for ADR-0008's attribution rules; README.md's license statement matches it.

### [To Do] FND-0047 Apply the license decision: `LICENSE` file, the `license` field in `[workspace.package]`, and the `[licenses]` private-crate handling in `deny.toml` adjusted if the ADR requires it
P1 · M0 · Task · deps: FND-0046, FND-0001 · labels: docs
Done when: `cargo metadata` shows the license on every member; `cargo deny check licenses` passes; if the ADR requires SPDX headers, a `check-manifests` rule enforces them, with a fixture test.

### [To Do] FND-0048 Root `NOTICE` file stating that Alluvium includes material from Apache Kafka (Copyright The Apache Software Foundation, Apache-2.0), with the content Apache-2.0 §4(d) requires carried over from Kafka 4.4.0's `NOTICE`
P1 · M0 · Task · deps: FND-0046 · labels: docs · ref: ADR-0008
Done when: NOTICE names the included material (vendored schemas under `schemas/kafka-4.4.0/`, closely ported tests and code); its text is compared with Kafka 4.4.0's `NOTICE` at tag 4.4.0 in the PR; ADR-0008 links to it.

### [To Do] FND-0049 Attribution procedure in CONTRIBUTING.md: the exact header for closely ported code or tests (source path at 4.4.0, copyright, Apache-2.0, "modified by Alluvium") and the `// Ported from:` test comment from testing.md
P1 · M0 · Task · deps: FND-0046 · labels: docs · ref: ADR-0008
Done when: The header template appears once (CONTRIBUTING) and is linked from ADR-0008 and docs/standards/testing.md rule 10; it states when attribution is required (close port) and when not (`// Parity:` references); the PR template's Kafka-reference section asks whether code was closely ported.

## [In Progress] FND-F09 Developer environment
P2 · M0 · Feature · deps: FND-F04
Scope: `cargo xtask doctor` (toolchain, rustup and tool checks), editor settings, optional git hooks, devcontainer.

### [Done] FND-0050 `cargo xtask doctor`: check the local toolchain and tools, with fix commands
P2 · M0 · Task · deps: FND-0020 · labels: good-first-task
Done when: Required checks: `cargo` is rustup's proxy (not Homebrew's or a distribution's), the active toolchain matches `rust-toolchain.toml` with rustfmt and clippy, and every tool in the xtask tool table is installed at its pinned version; optional checks: Docker (compat tests), a nightly toolchain and cargo-fuzz (fuzzing). Each check prints `ok` or `missing` with the exact fix command; the exit code is 1 only when a required check fails; a unit test feeds fake `rustup which cargo` and version outputs for the Homebrew-cargo case and the wrong-toolchain case; CONTRIBUTING §2 tells new engineers to run it.

### [Done] FND-0051 Editor settings: `.editorconfig` (UTF-8, LF, final newline, 4-space Rust, 2-space YAML/TOML/JSON) and shared `.vscode/settings.json` and `extensions.json` (rust-analyzer with clippy as the check command)
P2 · M0 · Task · deps: none · labels: good-first-task
Done when: `cargo fmt` and the editor settings agree (no reformatting churn on save in a sample edit); no user-specific paths are committed; CONTRIBUTING mentions the files.

### [Done] FND-0052 Optional git hooks: `cargo xtask install-hooks` writes a pre-push hook that runs `cargo xtask ci --step fmt --step clippy`, and `--uninstall` removes it
P3 · M0 · Task · deps: FND-0021 · labels: good-first-task
Done when: Hooks are never installed automatically; installing twice is idempotent; an existing hook that xtask did not write is left alone, with a warning.

### [To Do] FND-0053 `.devcontainer/` with a base image pinned by digest, rustup honouring `rust-toolchain.toml`, tools from `cargo xtask tools install`, and Docker-in-Docker for compatibility tests
P3 · M0 · Task · deps: FND-0020
Done when: In the container, `cargo xtask doctor` reports all required checks ok and `cargo xtask ci` passes; the image digest is updated by Dependabot (`devcontainers` ecosystem).

## [In Progress] FND-F10 Documentation build checks
P1 · M0 · Feature · deps: FND-F04
Scope: `cargo doc` with `-D warnings`, Markdown link and anchor checking in CI.

### [Done] FND-0054 Register the `doc` step: `cargo doc --workspace --no-deps --document-private-items --locked` with `RUSTDOCFLAGS="-D warnings"`
P1 · M0 · Task · deps: FND-0021 · labels: docs · ref: docs/standards/rust.md#tooling-commands-contract-implemented-in-m0
Done when: A broken intra-doc link, in either a public or a `pub(crate)` item, fails the step; the rust.md tooling table states that private items are documented too and why (most items are `pub(crate)`).

### [Done] FND-0055 Register the `links` step: check relative links and `#anchors` in README.md, AGENTS.md, CONTRIBUTING.md, `docs/**`, `.agents/**` and `.github/**` offline with lychee (version pinned in the xtask tool table, `--offline --include-fragments`)
P1 · M0 · Task · deps: FND-0021, FND-0020 · labels: docs
Done when: A test PR with a broken relative link and a wrong heading anchor fails the step, naming both; all existing broken links are fixed in this PR, so main starts clean; external URLs are excluded from this step.

### [To Do] FND-0056 Weekly scheduled workflow that checks external URLs in the same files (lychee online, with retries) and opens or updates one tracking issue when links are dead
P2 · M0 · Task · deps: FND-0055, FND-0031 · labels: docs
Done when: The workflow never runs on PRs; a dead link creates an issue listing file:line and URL; permissions are `contents: read` and `issues: write` only.

### [Done] FND-0057 `check-manifests` rule: each library crate's `lib.rs` starts with a `//!` doc that has `Layer:` (matching the xtask layer table) and `Kafka counterpart:` lines
P2 · M0 · Task · deps: FND-0023, FND-0007, TYPE-F01 · labels: docs · ref: docs/standards/rust.md#modules-and-documentation
Done when: Fixture crates with a missing doc, a missing line or a wrong layer are rejected; rust.md's "Modules and documentation" section shows the required lines; `alluvium-types` and `xtask` pass.
Notes: The rule reads only the crate root's text, so its fixtures are inline `lib.rs` texts in the unit test (missing doc, missing `Layer:`, wrong layer, missing or empty `Kafka counterpart:`, lines outside the leading doc, unknown crate) rather than fixture crates. Binary-only crates are checked through `src/main.rs`, which is how `xtask` passes. Also proved on the real workspace by changing `alluvium-types`' doc.

## [In Progress] FND-F11 Repository hygiene
P2 · M0 · Feature · deps: none
Scope: CODEOWNERS, issue templates, labels, branch protection settings documented.

### [To Do] FND-0058 `.github/CODEOWNERS`: an owner for every path, with security-sensitive paths listed explicitly (`deny.toml`, `Cargo.lock`, `.github/workflows/`, `docs/adr/`, `docs/standards/`, `.agents/`)
P2 · M0 · Task · deps: none · labels: security
Done when: GitHub validates the file without errors (CODEOWNERS errors view is clean); a PR that touches `deny.toml` requests review from its owner.

### [Done] FND-0059 Issue forms in `.github/ISSUE_TEMPLATE/` for bugs, parity gaps, design proposals and flaky tests
P2 · M0 · Task · deps: none · labels: docs
Done when: Bug: version, reproduction or simulation seed with the replay command, Kafka 4.4.0 behavior compared, redacted logs; parity gap: Kafka path at 4.4.0 or KIP, client and version; design/ADR proposal; flaky test: test name, seed, quarantine issue; `config.yml` disables blank issues. Each form renders on GitHub with its required fields marked; the bug form tells reporters to never paste record payloads or credentials (debuggability.md); the flaky form matches testing.md rule 7.

### [Done] FND-0060 Labels as code: `.github/labels.yml` (the backlog labels such as `security`, `nfr`, `decoder`, `perf`, `docs`, `test`, `good-first-task`, plus `bug`, `flaky`, `parity`, `dependencies`) synced by a workflow on changes to main
P3 · M0 · Task · deps: none · labels: docs
Done when: The sync workflow creates and updates labels and never deletes labels it doesn't manage; its permissions are `issues: write` only; the backlog README's label list and labels.yml agree.
Notes: The backlog README now links to `.github/labels.yml` instead of keeping its own list, so the two can't disagree; every label used in the backlog is in the file (two `debuggability` labels became `observability`, and the README example's `storage` label was dropped). The sync loop was run once locally with the same `gh label create --force` commands: it added 12 labels, updated `bug`, and removed none.

### [To Do] FND-0061 Document repository settings in `docs/guides/repository-settings.md` and commit the main-branch ruleset export
P2 · M0 · Task · deps: FND-0037 · labels: docs, security
Done when: Settings: required check `ci-success`; pull requests with one approval and code-owner review; dismissal of stale approvals; linear history; merge queue; no force pushes or deletions; Actions token read-only by default and approval for fork PRs; Dependabot alerts, secret scanning and push protection on; the ruleset is exported to `.github/rulesets/main.json`. The live settings match the document (screenshots or the `gh api` output are in the PR); the ruleset JSON imports cleanly; CONTRIBUTING links to the page.

### [Done] FND-0062 `.gitattributes`: `* text=auto eol=lf`, `**/src/generated/** linguist-generated=true`, `schemas/kafka-4.4.0/** linguist-vendored=true`, `Cargo.lock linguist-generated=true`
P2 · M0 · Task · deps: none · ref: ADR-0008
Done when: Generated code stays diffable (no `-diff`), as ADR-0008 requires, but is collapsed by default in PR views; checking out on a CRLF system produces LF files.
