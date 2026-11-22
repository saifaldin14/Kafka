# Fuzz targets

Every decoder of untrusted bytes, from the network or from disk, has a fuzz target here ([testing standard](../docs/standards/testing.md), [security standard](../docs/standards/security.md#untrusted-input)). libFuzzer feeds each target arbitrary bytes and reports any panic, hang or runaway allocation as a crash.

This directory is its own Cargo workspace, which the main workspace excludes. That keeps libFuzzer out of every other build, and lets the targets use the nightly toolchain that cargo-fuzz needs. The nightly is pinned by date in [`rust-toolchain.toml`](rust-toolchain.toml), so a crash reproduces with the compiler that found it. Its dependencies are checked by the `fuzz-deny` CI step against the main [`deny.toml`](../deny.toml).

## Running a target

```sh
rustup toolchain install nightly-2026-11-20   # the date in rust-toolchain.toml
cargo install --locked cargo-fuzz
cargo +nightly-2026-11-20 fuzz run --fuzz-dir fuzz --debug-assertions types_uuid_parse -- -max_total_time=60
```

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
