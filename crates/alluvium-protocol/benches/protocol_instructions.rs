//! Gated benchmarks for `alluvium-protocol`'s codecs: instruction counts under callgrind,
//! compared with the base of every pull request by `cargo xtask bench` (ADR-0019). Linux
//! only. Ids are `protocol/<operation>/<case>`.
//!
//! Each benchmark only calls an `#[inline(never)]` function that does the work. callgrind
//! counts from each entry to the benchmark function, and it takes a loop that jumps back
//! to a function's first instruction for a recursive call. So a loop compiled into the
//! benchmark function itself would turn counting off and on, and most of its work would
//! go uncounted.
#![expect(
    unused_qualifications,
    missing_docs,
    clippy::disallowed_methods,
    reason = "gungraun's macros expand to fully qualified paths, undocumented items, and \
              the benchmark binary's main, which exits with process::exit (ADR-0019)"
)]

use std::hint::black_box;

use alluvium_protocol::{Framing, Reader, Writer};
use bytes::Bytes;
use gungraun::{library_benchmark, library_benchmark_group, main};

mod inputs;

use inputs::{byte_arrays, compact_bytes, compact_strings, strings, varint_values, varints};

// 1000 unsigned varints of one width: the length prefix of every compact field and
// tagged field. (The benchmark macros don't accept doc comments.)
#[library_benchmark]
#[bench::width_1(varints(1))]
#[bench::width_3(varints(3))]
#[bench::width_5(varints(5))]
fn read_unsigned_varint(input: Bytes) -> u32 {
    read_unsigned_varint_all(black_box(input))
}

#[inline(never)]
fn read_unsigned_varint_all(input: Bytes) -> u32 {
    let mut reader = Reader::new(black_box(input));
    let mut sum = 0_u32;
    while let Ok(value) = reader.read_unsigned_varint() {
        sum = sum.wrapping_add(value);
    }
    sum
}

#[library_benchmark]
#[bench::width_1(varint_values(1))]
#[bench::width_3(varint_values(3))]
#[bench::width_5(varint_values(5))]
fn write_unsigned_varint(values: Vec<u32>) -> Bytes {
    write_unsigned_varint_all(black_box(values))
}

#[inline(never)]
fn write_unsigned_varint_all(values: Vec<u32>) -> Bytes {
    let mut writer = Writer::new();
    for value in black_box(values) {
        writer.write_unsigned_varint(value);
    }
    writer.into_bytes()
}

// 100 compact strings of one length, decoded with UTF-8 validation and charged to the
// decode budget.
#[library_benchmark]
#[bench::length_16(compact_strings(16))]
#[bench::length_4096(compact_strings(4096))]
fn read_compact_string(input: Bytes) -> usize {
    read_compact_string_all(black_box(input))
}

#[inline(never)]
fn read_compact_string_all(input: Bytes) -> usize {
    let mut reader = Reader::new(black_box(input));
    let mut total = 0;
    while let Ok(text) = reader.read_string(Framing::Compact) {
        total += black_box(text).len();
    }
    total
}

#[library_benchmark]
#[bench::length_16(strings(16))]
#[bench::length_4096(strings(4096))]
fn write_compact_string(values: Vec<String>) -> Bytes {
    write_compact_string_all(black_box(values))
}

#[inline(never)]
fn write_compact_string_all(values: Vec<String>) -> Bytes {
    let mut writer = Writer::new();
    for text in black_box(values) {
        if writer.write_string(Framing::Compact, &text).is_err() {
            break;
        }
    }
    writer.into_bytes()
}

// 100 compact byte arrays of one length, decoded as slices of the input.
#[library_benchmark]
#[bench::length_16(compact_bytes(16))]
#[bench::length_4096(compact_bytes(4096))]
fn read_compact_bytes(input: Bytes) -> usize {
    read_compact_bytes_all(black_box(input))
}

#[inline(never)]
fn read_compact_bytes_all(input: Bytes) -> usize {
    let mut reader = Reader::new(black_box(input));
    let mut total = 0;
    while let Ok(bytes) = reader.read_bytes(Framing::Compact) {
        total += black_box(bytes).len();
    }
    total
}

#[library_benchmark]
#[bench::length_16(byte_arrays(16))]
#[bench::length_4096(byte_arrays(4096))]
fn write_compact_bytes(values: Vec<Vec<u8>>) -> Bytes {
    write_compact_bytes_all(black_box(values))
}

#[inline(never)]
fn write_compact_bytes_all(values: Vec<Vec<u8>>) -> Bytes {
    let mut writer = Writer::new();
    for bytes in black_box(values) {
        if writer.write_bytes(Framing::Compact, &bytes).is_err() {
            break;
        }
    }
    writer.into_bytes()
}

library_benchmark_group!(
    name = protocol,
    benchmarks = [
        read_unsigned_varint,
        write_unsigned_varint,
        read_compact_string,
        write_compact_string,
        read_compact_bytes,
        write_compact_bytes
    ]
);
main!(library_benchmark_groups = protocol);
