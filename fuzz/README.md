# Fuzz targets

Every decoder of untrusted bytes, from the network or from disk, has a fuzz target here ([testing standard](../docs/standards/testing.md), [security standard](../docs/standards/security.md#untrusted-input)). libFuzzer feeds each target arbitrary bytes and reports any panic, hang or runaway allocation as a crash.

This directory is its own Cargo workspace, which the main workspace excludes. That keeps libFuzzer out of every other build, and lets the targets use the nightly toolchain that cargo-fuzz needs. The nightly is pinned by date in [`rust-toolchain.toml`](rust-toolchain.toml), so a crash reproduces with the compiler that found it. Its dependencies are checked by the `fuzz-deny` CI step against the main [`deny.toml`](../deny.toml).

## Running a target

```sh
cargo xtask tools install --tool cargo-fuzz             # the pinned cargo-fuzz
rustup toolchain install nightly-2026-11-20 --profile minimal   # the date in rust-toolchain.toml
cargo xtask fuzz --list                                 # the targets
cargo xtask fuzz types_uuid_parse --time 5m             # fuzz for 5 minutes (default 60s)
cargo xtask fuzz types_uuid_parse fuzz/artifacts/types_uuid_parse/crash-…   # rerun one input
```

`cargo xtask fuzz` builds the target with `cargo fuzz build` and debug assertions, so integer overflow is a crash. Then it runs the target's binary itself, with the limits below, so that every run uses them. cargo-fuzz's own `run` reruns a crashing input without them.

A run reads the corpus in `corpus/<target>/` and the committed seeds in `seeds/<target>/`, and adds new inputs to the corpus only. A crash leaves its input in `artifacts/<target>/`. `corpus/`, `artifacts/` and `target/` are ignored by git.

## Corpora and seeds

The nightly fuzz run on `main` grows each target's corpus and stores it as an Actions artifact ([ADR-0018](../docs/adr/0018-fuzz-corpus-storage.md)). Only CI writes corpora.

```sh
cargo xtask fuzz corpus pull                     # every target's newest corpus (needs `gh auth login`)
cargo xtask fuzz corpus pull types_uuid_parse    # one target's
cargo xtask fuzz corpus minimize types_uuid_parse
```

`corpus minimize` keeps only the inputs that add coverage, using libFuzzer's merge. It replaces the corpus only if the merge completed and the coverage and feature counts didn't drop, and it fails if the result is over the 50 MiB budget.

`seeds/<target>/` holds small hand-picked inputs, and the input of every fixed crash, so that a run starts from known-interesting cases even with no corpus. Name a seed after what it shows (`zero`, `longest`, `crash-<issue>`). Each is at most 64 KiB, and a target's seeds stay under 1 MiB.

## Limits

Every run uses these libFuzzer limits, from `LIMITS` in [`xtask/src/fuzz.rs`](../xtask/src/fuzz.rs); a test keeps this table equal to it. Exceeding one is reported as a crash, so a hang or a runaway allocation fails a run like a panic does.

| Flag | Value | Why |
|---|---|---|
| `-max_len` | 65536 | The largest input, in bytes. Big enough for a request with thousands of array elements; larger inputs slow every run without reaching new code. |
| `-timeout` | 10 | Seconds for one input. Decoding 64 KiB takes microseconds even under the address sanitizer, so 10 s on a slow shared runner is a hang, not slowness. |
| `-malloc_limit_mb` | 64 | The largest single allocation, in MiB. From a 64 KiB input a correct decoder allocates at most the input times a small constant (each declared element is backed by input bytes and charged to the decode budget). An allocation sized from an unchecked length, up to 2³¹ elements, fails here at once. |
| `-rss_limit_mb` | 2048 | The process's memory, in MiB: libFuzzer's default. It catches memory that accumulates across inputs, such as a leak, and stays well under a CI runner's memory. |

## In CI

Every pull request fuzzes each target for 60 s, one job per target, in [ci.yml](../.github/workflows/ci.yml). Each job starts from the corpus the nightly run grew. A crash fails the PR and uploads its input as the artifact `fuzz-crash-<target>`, kept 14 days. To reproduce it, download it and run `cargo xtask fuzz <target> <input>`.

## Writing a target

A target is named `<crate>_<decoder>`, after the crate without its `alluvium-` prefix and the decoder it feeds, such as `types_uuid_parse` or `protocol_wire_primitives`. Its file is `fuzz_targets/<name>.rs`, with a `[[bin]]` entry of the same name in `Cargo.toml`.

Each target checks two things:

1. **No panic, on any input.** The decoder returns an error for bad input; libFuzzer reports a panic as a crash. Targets build with debug assertions, so integer overflow is a crash too.
2. **What decodes, round-trips.** When decoding succeeds, encoding the value and decoding it again gives the same value. For text, printing it and parsing it again does. This catches a decoder that accepts input it then misreads.

A target's `//!` doc states what it feeds and what it asserts:

```rust
//! `Uuid::from_str` on arbitrary text: no panic, and a parsed id prints as text that
//! parses back to the same id.
#![no_main]

use std::str::FromStr;

use alluvium_types::Uuid;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &[u8]| {
    let Ok(text) = std::str::from_utf8(input) else {
        return;
    };
    if let Ok(id) = Uuid::from_str(text) {
        assert_eq!(Uuid::from_str(&id.to_string()), Ok(id));
    }
});
```

Asserting that a printed value equals the input only holds where the format has one spelling for each value. A valid topic name has one, so `types_topic_name` asserts it. Kafka's uuid parser accepts several spellings of the same id, so `types_uuid_parse` doesn't.
