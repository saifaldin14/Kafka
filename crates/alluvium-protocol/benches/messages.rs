//! Wall-time benchmarks for decoding and encoding whole generated messages: `cargo bench
//! -p alluvium-protocol --bench messages`. The gated instruction counts of the same work
//! are in `messages_instructions.rs`.

use alluvium_protocol::generated::fetch_response::FetchResponse;
use alluvium_protocol::generated::produce_request::ProduceRequest;
use alluvium_protocol::{Message, Reader, Writer};
use bytes::Bytes;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

mod messages_inputs;

use messages_inputs::{
    FETCH_RESPONSE_VERSION, PRODUCE_REQUEST_VERSION, encoded, fetch_response, produce_request,
};

fn elements(count: usize) -> Throughput {
    Throughput::Elements(u64::try_from(count).unwrap_or(u64::MAX))
}

/// Decoding and encoding `M` at `version`, for each partition count, with `message`
/// building the message of that many partitions.
fn message<M: Message + PartialEq>(
    c: &mut Criterion,
    name: &str,
    version: i16,
    message: impl Fn(usize) -> M,
) {
    let mut decode = c.benchmark_group(format!("messages/decode_{name}_v{version}"));
    for partitions in [1, 100, 1000] {
        decode.throughput(elements(partitions));
        let input = encoded(&message(partitions), version);
        decode.bench_with_input(
            BenchmarkId::from_parameter(partitions),
            &input,
            |b, input| {
                b.iter(|| M::decode(&mut Reader::new(Bytes::clone(input)), version));
            },
        );
    }
    decode.finish();
    let mut encode = c.benchmark_group(format!("messages/encode_{name}_v{version}"));
    for partitions in [1, 100, 1000] {
        encode.throughput(elements(partitions));
        let value = message(partitions);
        encode.bench_with_input(
            BenchmarkId::from_parameter(partitions),
            &value,
            |b, value| {
                b.iter(|| {
                    let mut writer = Writer::new();
                    value
                        .encode(&mut writer, version)
                        .map(|()| writer.into_bytes())
                });
            },
        );
    }
    encode.finish();
}

fn messages(c: &mut Criterion) {
    message::<ProduceRequest>(
        c,
        "produce_request",
        PRODUCE_REQUEST_VERSION,
        produce_request,
    );
    message::<FetchResponse>(c, "fetch_response", FETCH_RESPONSE_VERSION, fetch_response);
}

criterion_group!(benches, messages);
criterion_main!(benches);
