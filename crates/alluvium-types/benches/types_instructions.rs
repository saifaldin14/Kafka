//! Gated benchmarks for `alluvium-types`: instruction counts under callgrind, compared
//! with the base of every pull request by `cargo xtask bench` (ADR-0019). Linux only.
//!
//! Ids are `<area>/<operation>/<case>`: the group, the function and the case below.
#![expect(
    unused_qualifications,
    missing_docs,
    clippy::disallowed_methods,
    reason = "gungraun's macros expand to fully qualified paths, undocumented items, and \
              the benchmark binary's main, which exits with process::exit (ADR-0019)"
)]

use std::hint::black_box;

use alluvium_types::PayloadSummary;
use gungraun::{library_benchmark, library_benchmark_group, main};

mod inputs;

use inputs::input;

// The partitioner's key hash, over the key sizes of the criterion benchmark. (The
// benchmark macros don't accept doc comments.)
#[library_benchmark]
#[bench::size_16(input(16))]
#[bench::size_256(input(256))]
#[bench::size_4096(input(4 * 1024))]
#[bench::size_65536(input(64 * 1024))]
fn murmur2(key: Vec<u8>) -> i32 {
    black_box(alluvium_types::murmur2(black_box(&key)))
}

// Printing a payload in a log line: one CRC32C pass.
#[library_benchmark]
#[bench::size_1024(input(1024))]
#[bench::size_1048576(input(1024 * 1024))]
fn payload_summary(payload: Vec<u8>) -> String {
    black_box(PayloadSummary(black_box(&payload)).to_string())
}

library_benchmark_group!(name = types, benchmarks = [murmur2, payload_summary]);
main!(library_benchmark_groups = types);
