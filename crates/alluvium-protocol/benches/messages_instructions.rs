//! Gated benchmarks for decoding and encoding whole generated messages: instruction
//! counts under callgrind, compared with the base of every pull request by `cargo xtask
//! bench` (ADR-0019). Linux only. Ids are `messages/<operation>/<case>`.
//!
//! They are a target of their own, apart from the wire primitives in
//! `protocol_instructions.rs`: each target is one thin-LTO unit, so linking the generated
//! codecs into the primitives' binary would shift how those are inlined, and their
//! counts, with every change to the generator.
#![expect(
    unused_qualifications,
    missing_docs,
    clippy::disallowed_methods,
    reason = "gungraun's macros expand to fully qualified paths, undocumented items, and \
              the benchmark binary's main, which exits with process::exit (ADR-0019)"
)]

use std::hint::black_box;

use alluvium_protocol::generated::fetch_response::FetchResponse;
use alluvium_protocol::generated::produce_request::ProduceRequest;
use alluvium_protocol::{DecodeError, EncodeError, Message, Reader, Writer};
use bytes::Bytes;
use gungraun::{library_benchmark, library_benchmark_group, main};

mod messages_inputs;

use messages_inputs::{
    FETCH_RESPONSE_VERSION, PRODUCE_REQUEST_VERSION, encoded, fetch_response, produce_request,
};

// Each benchmark only calls an `#[inline(never)]` function that does the work, as in
// protocol_instructions.rs. (The benchmark macros don't accept doc comments.)
#[inline(never)]
fn decode<M: Message>(input: Bytes, version: i16) -> Result<M, DecodeError> {
    M::decode(&mut Reader::new(black_box(input)), version)
}

// The message is returned beside its encoding so that dropping it isn't counted.
#[inline(never)]
fn encode<M: Message>(message: M, version: i16) -> (M, Result<Bytes, EncodeError>) {
    let mut writer = Writer::new();
    let encoded = black_box(&message)
        .encode(&mut writer, version)
        .map(|()| writer.into_bytes());
    (message, encoded)
}

// A ProduceRequest v13 with one record batch for each partition of one topic.
#[library_benchmark]
#[bench::partitions_1(encoded(&produce_request(1), PRODUCE_REQUEST_VERSION))]
#[bench::partitions_100(encoded(&produce_request(100), PRODUCE_REQUEST_VERSION))]
#[bench::partitions_1000(encoded(&produce_request(1000), PRODUCE_REQUEST_VERSION))]
fn decode_produce_request_v13(input: Bytes) -> Result<ProduceRequest, DecodeError> {
    decode(black_box(input), PRODUCE_REQUEST_VERSION)
}

#[library_benchmark]
#[bench::partitions_1(produce_request(1))]
#[bench::partitions_100(produce_request(100))]
#[bench::partitions_1000(produce_request(1000))]
fn encode_produce_request_v13(
    message: ProduceRequest,
) -> (ProduceRequest, Result<Bytes, EncodeError>) {
    encode(black_box(message), PRODUCE_REQUEST_VERSION)
}

// A FetchResponse v18 with one record batch for each partition of one topic.
#[library_benchmark]
#[bench::partitions_1(encoded(&fetch_response(1), FETCH_RESPONSE_VERSION))]
#[bench::partitions_100(encoded(&fetch_response(100), FETCH_RESPONSE_VERSION))]
#[bench::partitions_1000(encoded(&fetch_response(1000), FETCH_RESPONSE_VERSION))]
fn decode_fetch_response_v18(input: Bytes) -> Result<FetchResponse, DecodeError> {
    decode(black_box(input), FETCH_RESPONSE_VERSION)
}

#[library_benchmark]
#[bench::partitions_1(fetch_response(1))]
#[bench::partitions_100(fetch_response(100))]
#[bench::partitions_1000(fetch_response(1000))]
fn encode_fetch_response_v18(
    message: FetchResponse,
) -> (FetchResponse, Result<Bytes, EncodeError>) {
    encode(black_box(message), FETCH_RESPONSE_VERSION)
}

library_benchmark_group!(
    name = messages,
    benchmarks = [
        decode_produce_request_v13,
        encode_produce_request_v13,
        decode_fetch_response_v18,
        encode_fetch_response_v18
    ]
);
main!(library_benchmark_groups = messages);
