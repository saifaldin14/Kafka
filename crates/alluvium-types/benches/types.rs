//! Benchmarks for `alluvium-types`: `cargo bench -p alluvium-types`.
//!
//! Conventions are in docs/standards/testing.md#benchmarks: names are
//! `<area>/<operation>/<size>`, inputs are deterministic, and every benchmark declares
//! its throughput.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

use alluvium_types::{SeededRng, murmur2};

/// Kafka keys are usually short, but nothing stops a client sending a large one.
const KEY_SIZES: [usize; 4] = [16, 256, 4 * 1024, 64 * 1024];

/// Bytes from a fixed seed, so every run hashes the same inputs.
fn input(size: usize) -> Vec<u8> {
    let mut bytes = vec![0; size];
    SeededRng::from_seed(0x6d75_726d_7572_3200).fill_bytes(&mut bytes);
    bytes
}

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

criterion_group!(benches, key_hash);
criterion_main!(benches);
