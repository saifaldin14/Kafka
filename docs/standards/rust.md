# Rust standards

These rules apply to all Rust code in Alluvium. Most are enforced by the compiler, clippy or `xtask`. The rest are enforced in review. Each rule says why, so you can apply its intent to cases it doesn't name.

Above all of them: more code is not better. Every line, item, feature flag, config key and dependency must serve the end goal and be the best available choice ([more is not better](../../.agents/skills/alluvium-engineering/SKILL.md#more-is-not-better-braess-paradox)). In Rust terms: no `pub` item without a caller, no Cargo feature or generic without a current need, and no dependency feature left enabled that isn't used.

## Toolchain

- The toolchain is pinned in `rust-toolchain.toml` (currently **1.99.0**, edition **2024**). Upgrade it in a dedicated PR that fixes any new lints.
- Use rustup's `cargo`. Homebrew and distribution builds ignore `rust-toolchain.toml`.
- Alluvium is an application, so its minimum supported Rust version is simply the pinned toolchain.

## Workspace configuration

The root `Cargo.toml` contains exactly this lint table, and every crate has `[lints] workspace = true`; the `manifests` step checks both, and checks `clippy.toml` against its block below. CI runs clippy with `-D warnings`, so `warn` fails CI but doesn't block local iteration. A lint that is misspelled or removed fails silently, so the `lint-canary` step proves the lints fire on fixtures in [`xtask/lint-canary`](../../xtask/lint-canary/src/lib.rs).

```toml
[workspace.lints.rust]
unsafe_code = "forbid"                  # exceptions need an ADR (standards/security.md)
missing_docs = "warn"
missing_debug_implementations = "warn"  # every public type must be printable when debugging
unreachable_pub = "warn"
unused_qualifications = "warn"
rust_2018_idioms = { level = "warn", priority = -1 }

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"       # `storage::StorageError` reads fine
must_use_candidate = "allow"            # too noisy; add #[must_use] where ignoring is a bug
allow_attributes = "warn"               # use #[expect(..., reason = "...")]
allow_attributes_without_reason = "warn"
await_holding_lock = "deny"
await_holding_refcell_ref = "deny"
dbg_macro = "deny"
exit = "deny"
expect_used = "warn"                    # allowed only as expect("invariant: ...") with #[expect]
large_futures = "warn"
mem_forget = "deny"
panic = "warn"
print_stderr = "deny"                   # use tracing
print_stdout = "deny"                   # use tracing (the CLI's output module is the exception)
todo = "deny"
unimplemented = "deny"
unwrap_used = "deny"
undocumented_unsafe_blocks = "deny"
rc_buffer = "warn"
string_slice = "warn"                   # byte slicing of str panics on char boundaries
```

Decoder code must check input-derived indexing and arithmetic. Cargo doesn't let a crate combine `workspace = true` with extra lints, so the decoder crates (`alluvium-protocol`, `alluvium-records`) put `#![deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)]` at the top of `lib.rs`. `alluvium-storage` puts the same attribute on its decoding modules. The `manifests` step checks every file listed in `DECODER_FILES` (`xtask/src/check_manifests.rs`) whose crate exists; a new decoding module is added to that list in the change that creates it.

The root `clippy.toml` contains exactly this:

```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
allow-dbg-in-tests = true
allow-print-in-tests = true
disallowed-methods = [
  { path = "std::time::Instant::now", reason = "inject time through alluvium_types::Clock" },
  { path = "std::time::SystemTime::now", reason = "inject time through alluvium_types::Clock" },
  { path = "std::time::Instant::elapsed", reason = "read time through alluvium_types::Clock" },
  { path = "std::time::SystemTime::elapsed", reason = "read time through alluvium_types::Clock" },
  { path = "std::thread::sleep", reason = "never block a thread to wait; async code waits through alluvium_types::Timer" },
  { path = "std::thread::spawn", reason = "spawn through the task supervisor" },
  { path = "tokio::spawn", reason = "spawn through the task supervisor so tasks are named, traced and shut down", allow-invalid = true },
  { path = "tokio::time::sleep", reason = "wait through alluvium_types::Timer so simulation controls time", allow-invalid = true },
  { path = "tokio::time::sleep_until", reason = "wait through alluvium_types::Timer so simulation controls time", allow-invalid = true },
  { path = "tokio::time::timeout", reason = "wait through alluvium_types::Timer so simulation controls time", allow-invalid = true },
  { path = "tokio::time::timeout_at", reason = "wait through alluvium_types::Timer so simulation controls time", allow-invalid = true },
  { path = "tokio::time::interval", reason = "wait through alluvium_types::Timer so simulation controls time", allow-invalid = true },
  { path = "tokio::time::interval_at", reason = "wait through alluvium_types::Timer so simulation controls time", allow-invalid = true },
  { path = "tokio::time::Instant::now", reason = "inject time through alluvium_types::Clock", allow-invalid = true },
  { path = "tokio::sync::mpsc::unbounded_channel", reason = "all queues are bounded; use mpsc::channel with a configured capacity", allow-invalid = true },
  { path = "rand::rng", reason = "use the injected seeded RNG", allow-invalid = true },
  { path = "rand::random", reason = "use the injected seeded RNG", allow-invalid = true },
  { path = "rand::random_iter", reason = "use the injected seeded RNG", allow-invalid = true },
  { path = "rand::random_range", reason = "use the injected seeded RNG", allow-invalid = true },
  { path = "rand::random_bool", reason = "use the injected seeded RNG", allow-invalid = true },
  { path = "rand::random_ratio", reason = "use the injected seeded RNG", allow-invalid = true },
  { path = "rand::fill", reason = "use the injected seeded RNG", allow-invalid = true },
  { path = "rand::make_rng", reason = "use the injected seeded RNG; OS entropy is read only by SeededRng::from_os_entropy", allow-invalid = true },
  { path = "getrandom::fill", reason = "use the injected seeded RNG; OS entropy is read only by SeededRng::from_os_entropy", allow-invalid = true },
  { path = "getrandom::fill_uninit", reason = "use the injected seeded RNG; OS entropy is read only by SeededRng::from_os_entropy", allow-invalid = true },
  { path = "getrandom::u32", reason = "use the injected seeded RNG; OS entropy is read only by SeededRng::from_os_entropy", allow-invalid = true },
  { path = "getrandom::u64", reason = "use the injected seeded RNG; OS entropy is read only by SeededRng::from_os_entropy", allow-invalid = true },
  { path = "std::collections::HashMap::new", reason = "RandomState breaks deterministic replay; use BTreeMap or alluvium_types::collections::SeededHashMap" },
  { path = "std::collections::HashMap::with_capacity", reason = "RandomState breaks deterministic replay; use BTreeMap or alluvium_types::collections::SeededHashMap" },
  { path = "std::collections::HashSet::new", reason = "RandomState breaks deterministic replay; use BTreeSet or alluvium_types::collections::SeededHashSet" },
  { path = "std::collections::HashSet::with_capacity", reason = "RandomState breaks deterministic replay; use BTreeSet or alluvium_types::collections::SeededHashSet" },
  { path = "std::process::exit", reason = "only the binary's main may exit; return errors" },
]
disallowed-types = [
  { path = "rand::rngs::ThreadRng", reason = "use the injected seeded RNG", allow-invalid = true },
  { path = "rand::rngs::SysRng", reason = "use the injected seeded RNG; OS entropy is read only by SeededRng::from_os_entropy", allow-invalid = true },
  { path = "std::collections::hash_map::RandomState", reason = "breaks deterministic replay; seed hashers from the injected RNG" },
]
```

Each disallowed item has exactly one sanctioned wrapper, and the wrapper carries a single `#[expect(clippy::disallowed_methods, reason = "…")]`. For example, `SystemClock` is the one place that calls `Instant::now`. [`xtask/tests/disallowed_exceptions.rs`](../../xtask/tests/disallowed_exceptions.rs) lists every such exception by file, so a new one fails the tests until it is reviewed and added there. The seeded hash maps take their seed from the injected RNG: random per process in production, so attacker-chosen keys can't cause hash flooding, and fixed in simulation, so runs replay exactly.

Clippy matches functions, not paths, so one entry bans every path to a function: `tokio::spawn` also covers `tokio::task::spawn`. A second entry for the same function would silently replace the first one's reason. Clippy also can't ban a trait method by path (`ThreadRng::default`), so such cases ban the type in `disallowed-types`. The `tokio` and `rand` entries carry `allow-invalid = true`: their functions exist only with some of the crates' features (rand's `thread_rng`, tokio's `rt` and `time`), and without it clippy warns in every crate that uses fewer features. A misspelled entry is still caught, because the lint canary compiles them all and requires every entry to fire. Every entry needs a fixture in the lint canary, which also checks that clippy reports the configured reason.

Release profile:

```toml
[profile.release]
panic = "abort"              # a panic is a bug; never continue with corrupted state (ADR-0004)
overflow-checks = true       # correctness over a few percent of speed; revisit only with benchmarks
debug = "line-tables-only"   # usable backtraces in production
lto = "thin"
```

## Errors

Defined by [ADR-0004](../adr/0004-error-model-and-panic-policy.md).

- **Each library crate has one or a few `thiserror` enums**, named for what failed (`StorageError`, `DecodeError`), plus `type Result<T, E = XError> = std::result::Result<T, E>`.
- **Variants carry context as fields**, not as formatted strings. Use `SegmentCorrupt { topic_partition, base_offset, position, reason: CorruptionReason }`, not `Other(String)`.
- **`Display` messages** are lowercase sentence fragments without trailing punctuation, and don't repeat the source error's text. The chain is printed through `source()`.
- **Kafka error codes are mapped in one place per crate**: `impl From<&XError> for ErrorCode`, as an exhaustive `match` with no `_ =>` arm. A new variant must force a decision. Error codes that depend on the request version are chosen in the handler, next to the version logic.
- **Log an error once**, at the place that handles it (decides to retry, respond or crash). Code that passes an error upward does not log it.
- **No silent discards.** `let _ = x;` and `.ok()` on a `Result` need a `// Ignored because …` comment.
- `anyhow` is allowed in the binary, `xtask` and tests only.

## Panics

- A panic means an Alluvium bug. Nothing reachable from network or disk input may panic.
- Use `expect` only to state an invariant the code itself guarantees, with the message `"invariant: <what holds and why>"`. Mark it `#[expect(clippy::expect_used, reason = "invariant")]`.
- State machines check invariants with `debug_assert!` in hot paths and `assert!` where a violation would corrupt data. ([architecture §8](../architecture.md#8-invariants))

## Types and naming

- **Newtypes for identifiers and positions**: `Offset(i64)`, `LeaderEpoch(i32)`, `BrokerId(i32)`, `PartitionIndex(i32)`, `ProducerId(i64)`, `ProducerEpoch(i16)`, `Sequence(i32)`. The compiler then catches swapped arguments, which are a classic bug in this domain.
- **Validated construction** via `TryFrom`. For example, `TopicName` enforces Kafka's rules: legal characters `[a-zA-Z0-9._-]`, at most 249 characters, not `.` or `..`. Once you hold a `TopicName`, it is valid.
- **Enums instead of booleans** in signatures: `IsolationLevel::ReadCommitted`, not `read_committed: bool`.
- **Make illegal states unrepresentable**: a leader-only field lives in the `Leader` variant, not in an `Option` on a shared struct.
- **Kafka's names, exactly**: `high_watermark`, `log_end_offset`, `last_stable_offset`, `log_start_offset`, `isr`, `leader_epoch`. The [glossary](../glossary.md) is the authority. Inventing a synonym is a review failure.
- Units in names where ambiguous: `timeout_ms`, `segment_bytes`, `retention: Duration`.

## Modules and documentation

- Each crate's `lib.rs` (or `main.rs` for a binary-only crate) starts with a `//!` doc covering: responsibility, layer, Kafka counterpart (with path at tag `4.4.0`), key invariants, and an entry-point type. The layer and counterpart are lines of their own, which `cargo xtask check-manifests` checks against the layer table:

  ```rust
  //! Layer: 0 ([ADR-0009](../../../docs/adr/0009-crate-layering-and-dependency-rules.md)).
  //! Kafka counterpart: `clients/.../common/Uuid.java` at 4.4.0, or `none`.
  ```
- Each non-trivial module starts with a `//!` doc covering what it owns, its invariants, and its lock or ownership rules.
- **One responsibility per module.** No `utils`, `helpers`, `common` or `misc`. A module over ~600 lines (excluding tests) is a signal to split by responsibility.
- Use `foo.rs` with `foo/` for submodules, not `mod.rs`.
- **Default to `pub(crate)`.** A crate's `pub` items are its contract, documented with `missing_docs`.
- Comments explain *why*. Use these standard tags:
  - `// Parity: <kafka path>@4.4.0 <Class.method> (KIP-n)` for code mirroring Kafka behavior.
  - `// Ported from: <kafka path>@4.4.0 <Class.method>` in its place for code or tests closely ported from Kafka, which also need the [attribution header](../../CONTRIBUTING.md#attributing-kafka-material).
  - `// Deviation: ADR-NNNN` where behavior intentionally differs.
  - `// INVARIANT: …` for a property the surrounding code relies on.
  - `// SAFETY: …` for any approved `unsafe`.
  - `// TODO(#issue): …`. TODOs without an issue are rejected.
- No commented-out code. Git remembers it.

## Async and concurrency

- **Layers 0–2 are synchronous and runtime-free** ([architecture §5](../architecture.md#5-layering-rules)).
- **Never block a Tokio worker**: no `std::fs`, no `std::thread::sleep`, no CPU work over ~100 µs (large decompression, CRC over big batches) outside the designated blocking pools.
- **Never hold a lock across `.await`**. This is enforced by clippy for std locks. For async locks, prefer restructuring over `tokio::sync::Mutex`.
- **Prefer single ownership plus messages to shared locks.** If a lock is needed, the module doc lists what it protects and the lock order. Acquiring locks in any other order is a bug.
- **Bounded channels only.** Capacity comes from config and is documented next to the channel, along with what happens when it is full.
- **Every network wait has a timeout** derived from config (Kafka's `request.timeout.ms`, `connections.max.idle.ms` and so on).
- **Cancellation safety**: every `select!` branch documents whether its future is cancel-safe. Non-cancel-safe work is moved into a task that owns it.
- **Tasks** are spawned only via the supervisor in `alluvium-server` (`supervisor.spawn(name, future)`). It attaches a span, registers the task for shutdown and diagnostics, and escalates panics.

## Dependencies

- All versions and sources live in `[workspace.dependencies]`, including path dependencies between workspace crates. Crates use `dep = { workspace = true }`, adding only `features` or `optional`. The `manifests` step enforces this.
- Enable minimal features (`default-features = false` where practical).
- A new dependency needs the justification in [security.md](security.md#dependencies).
- Preferred building blocks: `bytes`, `thiserror`, `tracing`, `tokio`, `rustls`, `crc32c`, `proptest`, `prometheus-client`. Introduce alternatives only with an ADR.

## Generated code

- Protocol and metadata types are generated by `cargo xtask codegen` from the vendored Kafka schemas ([ADR-0008](../adr/0008-protocol-code-generated-from-kafka-schemas.md)).
- Output is **committed** under `src/generated/`, starts with an `@generated by cargo xtask codegen` comment naming its schema and saying not to edit it, and is formatted with rustfmt (with `use_small_heuristics = "Max"`; `cargo fmt` skips the generated modules, which are declared with `#[rustfmt::skip]`). That way it can be read, grepped, stepped through in a debugger, and reviewed in diffs. How each schema feature maps to Rust is in [docs/protocol/codegen.md](../protocol/codegen.md).
- To change generated code, change the schema or the generator and run `cargo xtask codegen`. The `codegen` step fails CI when the committed output is stale.

## Performance

- Correctness and clarity first. An optimization that makes code harder to understand needs a benchmark showing the gain on a realistic workload, and the PR must include the numbers.
- Hot paths (record batch handling, fetch, produce append, request decode) avoid per-record allocation. Payloads stay as `bytes::Bytes` and are never copied into `Vec<u8>`.
- Hot paths have gated benchmarks in each crate's `benches/`, and an instruction-count regression above the threshold in `benches/thresholds.toml` fails the `bench` job ([ADR-0019](../adr/0019-benchmark-regression-gating.md), [testing standard](testing.md#benchmarks)).

## Tooling commands (contract implemented in M0)

`cargo xtask ci` runs every check CI runs, as named steps in a fixed order. CI jobs pick steps with `--step`, so each check is defined once, in `xtask/src/ci.rs`. Each job in [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) runs its steps through the [`xtask-ci`](../../.github/actions/xtask-ci/action.yml) action, which installs the pinned toolchain, restores the cache (saved only by pushes to main, so a PR can't poison it) and installs only the tools those steps need, and runs them with `--keep-going` so a job reports every failing step. There are two exceptions. ci.yml's `fuzz` jobs run `cargo xtask fuzz` on each target for 60 s, one job per target ([fuzz/README.md](../../fuzz/README.md#in-ci)). Its `bench` job runs `cargo xtask bench --base <base commit>`, which needs valgrind ([ADR-0019](../adr/0019-benchmark-regression-gating.md)).

| Command | Does |
|---|---|
| `cargo xtask ci` | Runs every step below in order, except the on-demand ones; stops at the first failure unless `--keep-going` is given. Run it before pushing. |
| `cargo xtask ci --list` | Lists the steps in run order |
| `cargo xtask ci --step <name>` | Runs only the named steps (repeatable), still in run order. On-demand steps run only this way |
| `cargo xtask doctor` | Checks the local setup: rustup's `cargo` first on `PATH`, the pinned toolchain and components, and the pinned tools the CI steps need (required); Docker, and the nightly in `fuzz/rust-toolchain.toml` and the pinned cargo-fuzz (optional). Prints the fix for each problem; exits 1 only when a required check fails |
| `cargo xtask install-hooks [--uninstall]` | Optional: installs (or removes) a pre-push hook that runs `cargo xtask ci --step fmt --step clippy`. Never installed automatically; a hook xtask didn't write is left alone, with a warning |
| `cargo xtask tools install [--step <name>]... [--tool <name>]...` | Installs the pinned versions of the external tools the CI steps need, or only those the named steps need, plus any tool named with `--tool`, such as `cargo-fuzz`, which no step needs (`xtask/src/tools.rs` is the only place versions appear). Rust tools come from crates.io; others (actionlint, shellcheck) from release archives whose SHA-256 must match the table |
| `cargo xtask fuzz <target> [--time <duration>]`, `fuzz <target> <input>`, `fuzz --list` | Builds a fuzz target with the pinned cargo-fuzz on the nightly in `fuzz/rust-toolchain.toml`, with debug assertions, and runs its binary with the libFuzzer limits in `xtask/src/fuzz.rs`: for the duration (`90s`, `5m`, `1h`; 60 s by default) over `fuzz/corpus/<target>/` and `fuzz/seeds/<target>/`, or once on one input such as a crash artifact. An unknown target fails with the list of targets; a missing nightly or cargo-fuzz fails with the command that installs it ([fuzz/README.md](../../fuzz/README.md)) |
| `cargo xtask bench [--base <ref>]` | Runs the gated benchmarks (bench targets named `*_instructions`) under valgrind with the pinned gungraun-runner, and prints each one's instruction count and estimated cycles. With `--base`, it also runs them at `<ref>` in a worktree under `target/bench/`, prints the comparison, and fails when an instruction count grows beyond its threshold in `benches/thresholds.toml` ([ADR-0019](../adr/0019-benchmark-regression-gating.md)). Linux only. A missing valgrind or gungraun-runner fails with the install command |
| `cargo xtask fuzz corpus pull [<target>]...`, `fuzz corpus minimize <target>` | Downloads, with `gh`, each target's newest unexpired corpus artifact from a run on `main` of this repository ([ADR-0018](../adr/0018-fuzz-corpus-storage.md)); shrinks a corpus with libFuzzer's `-merge=1`, replacing it only if the merge completed without losing coverage, and fails if it is over 50 MiB |
| `cargo xtask codegen [--check]` | Regenerates `crates/alluvium-protocol/src/generated/` (the message types from the vendored Kafka schemas, and `ErrorCode` from `Errors.java`), `crates/alluvium-protocol/src/test_messages/` (from Kafka's generator test schemas) and [docs/errors.md](../errors.md), writing only changed files and removing files it no longer generates, and prints the module and error-code counts and the most deeply nested message ([docs/protocol/codegen.md](../protocol/codegen.md)). `--check` writes nothing and fails, naming each stale file and its first differing line, if the committed files differ. Both first check the vendored directory against its manifest |
| `cargo xtask schemas verify` | Every `schemas/kafka-<tag>/` directory holds exactly the files its `MANIFEST.sha256` lists, with those SHA-256s ([ADR-0008](../adr/0008-protocol-code-generated-from-kafka-schemas.md)) |
| `cargo xtask schemas vendor --tag <tag> [--dry-run]` | Sparse-clones the apache/kafka tag into `target/`, copies the schema directories, the error sources and Kafka's `LICENSE` and `NOTICE` into `schemas/kafka-<tag>/` (keeping its README.md) and writes the manifest; `--dry-run` only prints the added, removed and changed files. Needs the network, so never runs in CI |
| `cargo xtask golden <dump>` | Records what Kafka 4.4.0's own classes do, with the Java harness in `tools/kafka-golden/`: downloads the client jar and its runtime dependencies from Maven Central (pinned SHA-256s), compiles and runs the harness in a JDK image pinned by digest, under Docker, without network access, and moves its output into the repository. `errors` writes `Errors.values()` to `xtask/tests/fixtures/kafka-4.4.0-errors.json`, the golden file the generated `ErrorCode` is tested against; `api-keys` writes the `ApiKeys` registry and the API version lists Kafka's broker and controller listeners advertise to `tests/golden/protocol/registry/`. Needs the network and Docker, so never runs in per-PR CI |
| `cargo xtask deps-diff <base-ref>` | Lists the registry crates added, removed and upgraded in `Cargo.lock` since `<base-ref>`, with license (from `cargo metadata`) and transitive count (from the lockfile), as a Markdown table. Offline; a reviewer aid that exits 1 only when it can't read its inputs ([dependency questions](security.md#dependencies)) |
| `cargo xtask docs check-crates`, `new-crate-doc <crate>` | Every workspace crate's deep dive exists, with the stubs listed; and a stub deep dive for a new crate from the [template](../crates/template.md) ([crate deep dives](../crates/README.md)) |
| `cargo xtask backlog check`, `next [--all] [-n N]`, `show <ID>`, `stats [--write]` | The backlog rules, the next ready tasks, one item with its dependencies, and the totals in the backlog README ([backlog rules](../backlog/README.md#choosing-the-next-task)) |

| Step | Runs |
|---|---|
| `fmt` | `cargo fmt --all --check` (default rustfmt settings) |
| `clippy` | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| `doc` | `cargo doc --workspace --no-deps --document-private-items --locked` with `RUSTDOCFLAGS="-D warnings"`: docs build with no broken intra-doc links or invalid HTML. Private items are documented too, because most items are `pub(crate)` and their docs must stay correct |
| `layers` | `cargo xtask check-layers`: crate dependency layering ([architecture §5](../architecture.md#5-layering-rules)), and no layer 0–2 crate other than `alluvium-types` names `SystemClock` or `SeededRng::from_os_entropy` outside comments; `--print` shows the layer table |
| `manifests` | `cargo xtask check-manifests`: `[workspace.lints]` and `clippy.toml` equal the blocks in [workspace configuration](#workspace-configuration), compared as parsed TOML, and every member has `[lints] workspace = true` and `license-file.workspace = true` and inherits every dependency with `{ workspace = true }`; every advisory ignore in `deny.toml` has a `reason` and an `expiry`; every crate root's `//!` doc has `Layer:` (matching the layer table) and `Kafka counterpart:` lines; every decoder file in `DECODER_FILES` whose crate exists denies `clippy::indexing_slicing` and `clippy::arithmetic_side_effects` |
| `schemas` | `cargo xtask schemas verify`: each vendored Kafka schema directory matches its manifest, and every schema in it loads as the code generator reads it (`xtask/src/message_spec.rs`), with each API key having both a request and a response; a modified, added or removed file, or one that doesn't load, fails, named. It prints an inventory: schemas, request and response APIs, keyed struct arrays |
| `codegen` | `cargo xtask codegen --check`: the generated code and docs/errors.md are exactly what `cargo xtask codegen` writes from the vendored Kafka files, and those match their manifest. A hand-edited, stale, missing or leftover generated file fails, named with its first differing line |
| `crate-docs` | `cargo xtask docs check-crates`: every workspace member has `docs/crates/<crate>.md`; deep dives that are still stubs are listed but don't fail. That every member is in [architecture §4](../architecture.md#4-code-map) is the `layers` step's rule |
| `workflows` | `cargo xtask check-workflows`: every `uses:` in `.github/workflows/` and `.github/actions/` is pinned to a commit SHA with a `# v…` comment, no workflow uses `pull_request_target`, and every job sets `timeout-minutes` and `permissions`; the `steps:` inputs of `ci.yml`'s jobs together run every step that isn't on demand, and name no other; then the pinned actionlint checks the workflows' syntax, expressions and runner labels, and the pinned shellcheck checks every `run:` script |
| `backlog` | `cargo xtask backlog check`: the backlog's format, dependency rules, feature statuses and README totals ([backlog rules](../backlog/README.md)) |
| `lint-canary` | `cargo xtask lint-canary`: clippy on the fixtures in [`xtask/lint-canary`](../../xtask/lint-canary/src/lib.rs) must report exactly the lints their `expect-lint:` comments name, so a removed or weakened lint fails. Every `clippy.toml` entry must fire with its configured reason, and every lint `[workspace.lints]` sets one by one, at a level other than `allow`, must have a fixture |
| `deny` | `cargo deny --locked check bans licenses sources`: banned and duplicate crates, licenses, sources ([`deny.toml`](../../deny.toml)) |
| `fuzz-deny` | The same as `deny`, for the fuzz workspace (`--manifest-path fuzz/Cargo.toml`), which has its own lockfile. Its advisories aren't checked: nothing it builds ships |
| `advisories` | `cargo deny --locked check advisories`: security advisories against `Cargo.lock`. Separate from `deny` because a new advisory depends on the date, not the change, so CI reports it without blocking unrelated PRs |
| `links` | `lychee --offline --include-fragments` over README.md, AGENTS.md, CLAUDE.md, CONTRIBUTING.md, `docs/`, `.agents/` and `.github/`: relative links and `#anchors` resolve. External URLs are not checked here |
| `links-external` | On demand: `lychee` online, with retries and a neutral user agent, over the same files as `links`. External sites can fail without a change here, so no PR runs it; [`links.yml`](../../.github/workflows/links.yml) runs it weekly and opens or updates a tracking issue listing each dead link's `file:line` and URL |
| `test` | `cargo nextest run --workspace --locked --profile ci` ([`.config/nextest.toml`](../../.config/nextest.toml)) |
| `doctest` | `cargo test --workspace --doc --locked`: the examples in doc comments compile and pass, and `compile_fail` examples fail to compile. A separate step because nextest does not run doctests |

Steps and commands still to come, each added by its backlog task:

| Planned | Does |
|---|---|
| `cargo xtask compat` | Real-client and differential tests (Docker) |
