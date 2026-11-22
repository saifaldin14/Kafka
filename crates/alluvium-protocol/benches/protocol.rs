//! Wall-time benchmarks for `alluvium-protocol`'s codecs: `cargo bench -p
//! alluvium-protocol --bench protocol`. The gated instruction counts of the same work are
//! in `protocol_instructions.rs`.

use alluvium_protocol::{Framing, Reader, Writer};
use bytes::Bytes;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

mod inputs;

use inputs::{
    VALUES, VARINTS, byte_arrays, compact_bytes, compact_strings, strings, varint_values, varints,
};

fn elements(count: usize) -> Throughput {
    Throughput::Elements(u64::try_from(count).unwrap_or(u64::MAX))
}

fn unsigned_varints(c: &mut Criterion) {
    let mut read = c.benchmark_group("protocol/read_unsigned_varint");
    read.throughput(elements(VARINTS));
    for width in [1, 3, 5] {
        let input = varints(width);
        read.bench_with_input(BenchmarkId::from_parameter(width), &input, |b, input| {
            b.iter(|| {
                let mut reader = Reader::new(Bytes::clone(input));
                while reader.read_unsigned_varint().is_ok() {}
            });
        });
    }
    read.finish();
    let mut write = c.benchmark_group("protocol/write_unsigned_varint");
    write.throughput(elements(VARINTS));
    for width in [1, 3, 5] {
        let values = varint_values(width);
        write.bench_with_input(BenchmarkId::from_parameter(width), &values, |b, values| {
            b.iter(|| {
                let mut writer = Writer::new();
                for &value in values {
                    writer.write_unsigned_varint(value);
                }
                writer.into_bytes()
            });
        });
    }
    write.finish();
}

fn compact_strings_and_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol/read_compact_string");
    group.throughput(elements(VALUES));
    for length in [16, 4096] {
        let input = compact_strings(length);
        group.bench_with_input(BenchmarkId::from_parameter(length), &input, |b, input| {
            b.iter(|| {
                let mut reader = Reader::new(Bytes::clone(input));
                while reader.read_string(Framing::Compact).is_ok() {}
            });
        });
    }
    group.finish();
    let mut group = c.benchmark_group("protocol/write_compact_string");
    group.throughput(elements(VALUES));
    for length in [16, 4096] {
        let values = strings(length);
        group.bench_with_input(BenchmarkId::from_parameter(length), &values, |b, values| {
            b.iter(|| {
                let mut writer = Writer::new();
                for text in values {
                    if writer.write_string(Framing::Compact, text).is_err() {
                        break;
                    }
                }
                writer.into_bytes()
            });
        });
    }
    group.finish();
    let mut group = c.benchmark_group("protocol/read_compact_bytes");
    group.throughput(elements(VALUES));
    for length in [16, 4096] {
        let input = compact_bytes(length);
        group.bench_with_input(BenchmarkId::from_parameter(length), &input, |b, input| {
            b.iter(|| {
                let mut reader = Reader::new(Bytes::clone(input));
                while reader.read_bytes(Framing::Compact).is_ok() {}
            });
        });
    }
    group.finish();
    let mut group = c.benchmark_group("protocol/write_compact_bytes");
    group.throughput(elements(VALUES));
    for length in [16, 4096] {
        let values = byte_arrays(length);
        group.bench_with_input(BenchmarkId::from_parameter(length), &values, |b, values| {
            b.iter(|| {
                let mut writer = Writer::new();
                for bytes in values {
                    if writer.write_bytes(Framing::Compact, bytes).is_err() {
                        break;
                    }
                }
                writer.into_bytes()
            });
        });
    }
    group.finish();
}

criterion_group!(benches, unsigned_varints, compact_strings_and_bytes);
criterion_main!(benches);
