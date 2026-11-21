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

Decoder code must check input-derived indexing and arithmetic. Cargo doesn't let a crate combine `workspace = true` with extra lints, so the decoder crates (`alluvium-protocol`, `alluvium-records`) put `#![deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)]` at the top of `lib.rs`. `alluvium-storage` puts the same attribute on its decoding modules.

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
  { path = "std::thread::sleep", reason = "use the Clock; never block a thread to wait" },
  { path = "std::thread::spawn", reason = "spawn through the task supervisor" },
  { path = "tokio::spawn", reason = "spawn through the task supervisor so tasks are named, traced and shut down" },
  { path = "tokio::time::sleep", reason = "use the Clock so simulation controls time" },
  { path = "tokio::time::Instant::now", reason = "inject time through alluvium_types::Clock" },
  { path = "tokio::sync::mpsc::unbounded_channel", reason = "all queues are bounded; use mpsc::channel with a configured capacity" },
  { path = "rand::rng", reason = "use the injected seeded RNG" },
  { path = "rand::random", reason = "use the injected seeded RNG" },
  { path = "rand::random_iter", reason = "use the injected seeded RNG" },
  { path = "rand::random_range", reason = "use the injected seeded RNG" },
  { path = "rand::random_bool", reason = "use the injected seeded RNG" },
  { path = "rand::random_ratio", reason = "use the injected seeded RNG" },
  { path = "rand::fill", reason = "use the injected seeded RNG" },
  { path = "rand::make_rng", reason = "use the injected seeded RNG; OS entropy is read only by ProcessRng::from_os_entropy" },
  { path = "std::collections::HashMap::new", reason = "RandomState breaks deterministic replay; use BTreeMap or alluvium_types::collections::SeededHashMap" },
  { path = "std::collections::HashMap::with_capacity", reason = "RandomState breaks deterministic replay; use BTreeMap or alluvium_types::collections::SeededHashMap" },
  { path = "std::collections::HashSet::new", reason = "RandomState breaks deterministic replay; use BTreeSet or alluvium_types::collections::SeededHashSet" },
  { path = "std::collections::HashSet::with_capacity", reason = "RandomState breaks deterministic replay; use BTreeSet or alluvium_types::collections::SeededHashSet" },
  { path = "std::process::exit", reason = "only the binary's main may exit; return errors" },
]
disallowed-types = [
  { path = "rand::rngs::ThreadRng", reason = "use the injected seeded RNG" },
  { path = "rand::rngs::SysRng", reason = "use the injected seeded RNG; OS entropy is read only by ProcessRng::from_os_entropy" },
  { path = "std::collections::hash_map::RandomState", reason = "breaks deterministic replay; seed hashers from the injected RNG" },
]
```

Each disallowed item has exactly one sanctioned wrapper, and the wrapper carries a single `#[expect(clippy::disallowed_methods, reason = "…")]`. For example, `SystemClock` is the one place that calls `Instant::now`. The seeded hash maps take their seed from the injected RNG: random per process in production, so attacker-chosen keys can't cause hash flooding, and fixed in simulation, so runs replay exactly.

Clippy matches functions, not paths, so one entry bans every path to a function: `tokio::spawn` also covers `tokio::task::spawn`. A second entry for the same function would silently replace the first one's reason. Clippy also can't ban a trait method by path (`ThreadRng::default`), so such cases ban the type in `disallowed-types`. Every entry needs a fixture in the lint canary, which also checks that clippy reports the configured reason.

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

- Each crate's `lib.rs` starts with a `//!` doc covering: responsibility, layer, Kafka counterpart (with path at tag `4.4.0`), key invariants, and an entry-point type.
- Each non-trivial module starts with a `//!` doc covering what it owns, its invariants, and its lock or ownership rules.
- **One responsibility per module.** No `utils`, `helpers`, `common` or `misc`. A module over ~600 lines (excluding tests) is a signal to split by responsibility.
- Use `foo.rs` with `foo/` for submodules, not `mod.rs`.
- **Default to `pub(crate)`.** A crate's `pub` items are its contract, documented with `missing_docs`.
- Comments explain *why*. Use these standard tags:
  - `// Parity: <kafka path>@4.4.0 <Class.method> (KIP-n)` for code mirroring Kafka behavior.
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
- Output is **committed** under `src/generated/`, starts with `// @generated by xtask codegen from schemas/kafka-4.4.0 — do not edit`, and is formatted with rustfmt. That way it can be read, grepped, stepped through in a debugger, and reviewed in diffs.
- CI fails if regenerating changes anything. To change generated code, change the generator.

## Performance

- Correctness and clarity first. An optimization that makes code harder to understand needs a benchmark showing the gain on a realistic workload, and the PR must include the numbers.
- Hot paths (record batch handling, fetch, produce append, request decode) avoid per-record allocation. Payloads stay as `bytes::Bytes` and are never copied into `Vec<u8>`.
- Criterion benchmarks for hot paths live in each crate's `benches/`. Regressions above the agreed threshold fail the benchmark job.

## Tooling commands (contract implemented in M0)

`cargo xtask ci` runs every check CI runs, as named steps in a fixed order. CI jobs pick steps with `--step`, so each check is defined once, in `xtask/src/ci.rs`.

| Command | Does |
|---|---|
| `cargo xtask ci` | Runs every step below in order; stops at the first failure unless `--keep-going` is given. Run it before pushing. |
| `cargo xtask ci --list` | Lists the steps in run order |
| `cargo xtask ci --step <name>` | Runs only the named steps (repeatable), still in run order |
| `cargo xtask tools install` | Installs the pinned versions of the external tools (`xtask/src/tools.rs` is the only place versions appear) |
| `cargo xtask backlog check`, `next [--all] [-n N]`, `show <ID>`, `stats [--write]` | The backlog rules, the next ready tasks, one item with its dependencies, and the totals in the backlog README ([backlog rules](../backlog/README.md#choosing-the-next-task)); the same output as `backlog.py`, which FND-0045 retires |

| Step | Runs |
|---|---|
| `fmt` | `cargo fmt --all --check` (default rustfmt settings) |
| `clippy` | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| `doc` | `cargo doc --workspace --no-deps --document-private-items --locked` with `RUSTDOCFLAGS="-D warnings"`: docs build with no broken intra-doc links or invalid HTML. Private items are documented too, because most items are `pub(crate)` and their docs must stay correct |
| `layers` | `cargo xtask check-layers`: crate dependency layering ([architecture §5](../architecture.md#5-layering-rules)); `--print` shows the layer table |
| `manifests` | `cargo xtask check-manifests`: `[workspace.lints]` and `clippy.toml` equal the blocks in [workspace configuration](#workspace-configuration), compared as parsed TOML, and every member has `[lints] workspace = true` and inherits every dependency with `{ workspace = true }`; every advisory ignore in `deny.toml` has a `reason` and an `expiry` |
| `lint-canary` | `cargo xtask lint-canary`: clippy on the fixtures in [`xtask/lint-canary`](../../xtask/lint-canary/src/lib.rs) must report exactly the lints their `expect-lint:` comments name, so a removed or weakened lint fails. Every `clippy.toml` entry must fire with its configured reason |
| `deny` | `cargo deny --locked check`: advisories, licenses, banned and duplicate crates, sources ([`deny.toml`](../../deny.toml)) |
| `links` | `lychee --offline --include-fragments` over README.md, AGENTS.md, CLAUDE.md, CONTRIBUTING.md, `docs/`, `.agents/` and `.github/`: relative links and `#anchors` resolve. External URLs are not checked here |
| `test` | `cargo nextest run --workspace --locked --profile ci` ([`.config/nextest.toml`](../../.config/nextest.toml)) |

Steps and commands still to come, each added by its backlog task:

| Planned | Does |
|---|---|
| `cargo xtask codegen --check` | Generated code is up to date |
| `cargo xtask compat` | Real-client and differential tests (Docker) |
| `cargo xtask fuzz <target> [--time 60s]` | Run a fuzz target |
