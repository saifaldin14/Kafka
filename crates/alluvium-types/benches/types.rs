//! Wall-time benchmarks for `alluvium-types`: `cargo bench -p alluvium-types --bench types`.
//! The gated instruction counts of the same work are in `types_instructions.rs`.
//!
//! Conventions are in docs/standards/testing.md#benchmarks: names are
//! `<area>/<operation>/<size>`, inputs are deterministic, and every benchmark declares
//! its throughput.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

use alluvium_types::{PayloadSummary, murmur2};

mod inputs;

use inputs::input;

/// Kafka keys are usually short, but nothing stops a client sending a large one. The
/// gated benchmarks in `types_instructions.rs` use the same sizes.
const KEY_SIZES: [usize; 4] = [16, 256, 4 * 1024, 64 * 1024];

fn key_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("types/murmur2");
    for size in KEY_SIZES {
        let key = input(size);
        group.throughput(Throughput::Bytes(u64::try_from(size).unwrap_or(u64::MAX)));
        group.bench_with_input(BenchmarkId::from_parameter(size), &key, |b, key| {
            b.iter(|| murmur2(std::hint::black_box(key)));
        });
    }
    group.finish();
}

/// Printing a payload in a log line: one CRC32C pass. Budget: under 1 ms for 1 MiB
/// (see `PayloadSummary`).
fn payload_summary(c: &mut Criterion) {
    let mut group = c.benchmark_group("types/payload_summary");
    for size in [1024, 1024 * 1024] {
        let payload = input(size);
        group.throughput(Throughput::Bytes(u64::try_from(size).unwrap_or(u64::MAX)));
        group.bench_with_input(BenchmarkId::from_parameter(size), &payload, |b, payload| {
            b.iter(|| PayloadSummary(std::hint::black_box(payload)).to_string());
        });
    }
    group.finish();
}

criterion_group!(benches, key_hash, payload_summary);
criterion_main!(benches);
