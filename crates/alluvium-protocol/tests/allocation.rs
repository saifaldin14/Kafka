//! Peak heap use while decoding hostile input, and while encoding records into a frame,
//! measured by a counting global allocator.
//!
//! Everything runs in one test so that no other test allocates at the same time, under
//! nextest or `cargo test` alike.

use std::mem::size_of;

use alluvium_protocol::generated::create_topics_request::CreateTopicsRequest;
use alluvium_protocol::generated::fetch_response::{
    FetchResponse, FetchableTopicResponse, PartitionData,
};
use alluvium_protocol::generated::metadata_request::MetadataRequest;
use alluvium_protocol::generated::response_header::ResponseHeader;
use alluvium_protocol::generated::sasl_authenticate_request::SaslAuthenticateRequest;
use alluvium_protocol::{
    DecodeError, DecodeErrorKind, DecodeLimits, Framing, HeaderError, Limit, Message,
    RawTaggedField, Reader, RecordsField, Writer, decode_request_header,
};
use bytes::Bytes;
use peak_alloc::PeakAlloc;

#[global_allocator]
static HEAP: PeakAlloc = PeakAlloc;

/// The most bytes in use at once while `decode` ran, beyond those in use before it.
fn peak_during(decode: impl FnOnce()) -> usize {
    let before = HEAP.current_usage();
    HEAP.reset_peak_usage();
    decode();
    HEAP.peak_usage().saturating_sub(before)
}

/// A struct with no fields in the version decoded, as a generated decoder holds it. On
/// the wire it takes no bytes at all.
struct Empty {
    _unknown_tagged_fields: Vec<RawTaggedField>,
}

/// A struct holding an array of [`Empty`].
struct Holder {
    _empties: Vec<Empty>,
}

#[test]
fn heap_use_is_bounded() {
    // PROTO-0107: a million declared string elements preallocate a thousand. Each is a
    // null, which fails to decode, so only the preallocation is ever made.
    let mut writer = Writer::new();
    writer
        .write_array_length(Framing::Classic, 1_000_000)
        .unwrap();
    for _ in 0..1_000_000 {
        writer.write_i16(-1);
    }
    let input = writer.into_bytes();
    let peak = peak_during(|| {
        let result = Reader::new(input).read_array(Framing::Classic, |element| {
            element.read_string(Framing::Classic)
        });
        assert!(result.is_err());
    });
    assert!(
        peak <= 1000 * size_of::<String>(),
        "peak {peak} bytes for a declared 1,000,000-element array"
    );

    // PROTO-0111: a 1 MiB frame of holders, each declaring a million empty structs.
    // Every count passes Kafka's checks, and the empty structs cost no input, so only
    // the budget stops the decode.
    let holders = (1 << 20) / 4 - 1;
    let mut writer = Writer::new();
    writer
        .write_array_length(Framing::Classic, holders)
        .unwrap();
    for _ in 0..holders {
        writer
            .write_array_length(Framing::Classic, 1_000_000)
            .unwrap();
    }
    let frame = writer.into_bytes();
    assert_eq!(frame.len(), 1 << 20);
    let budget = DecodeLimits::default().allocation_budget;
    let mut error = None;
    let peak = peak_during(|| {
        let holders = Reader::new(frame).read_array(Framing::Classic, |holder| {
            let empties = holder.read_array(Framing::Classic, |_| {
                Ok(Empty {
                    _unknown_tagged_fields: Vec::new(),
                })
            })?;
            Ok(Holder { _empties: empties })
        });
        error = holders.err();
    });
    // The holders, then four arrays of empties, fit in the budget; the fifth doesn't.
    // Each array is charged for its 1000 first elements too, which it outgrows.
    let empties = (1_000_000 + 1000) * size_of::<Empty>();
    let charged = (holders + 1000) * size_of::<Holder>() + 4 * empties;
    let error = error.expect("the decode exhausts the budget");
    assert_eq!(
        (error.offset, error.kind),
        (
            4 + 4 * 4,
            DecodeErrorKind::LimitExceeded {
                limit: Limit::AllocationBudget,
                requested: empties,
                allowed: budget - charged,
            }
        )
    );
    assert!(
        peak <= budget,
        "peak {peak} bytes over the {budget}-byte budget"
    );

    // An unknown tagged field kept under a budget of one slot allocates one slot, not
    // the four `Vec` would allocate on its first push: capacity is charged before it is
    // allocated. The rest of the peak is the shared header slicing the input may make.
    let slot = size_of::<RawTaggedField>();
    let mut writer = Writer::new();
    writer.write_i32(1);
    writer.write_unsigned_varint(1);
    writer.write_unsigned_varint(0);
    writer.write_unsigned_varint(0);
    let input = writer.into_bytes();
    let limits = DecodeLimits {
        allocation_budget: slot,
        ..DecodeLimits::default()
    };
    let peak = peak_during(|| {
        let header = ResponseHeader::decode(&mut Reader::with_limits(input, limits), 1).unwrap();
        assert_eq!(header.unknown_tagged_fields.len(), 1);
    });
    assert!(
        peak < 2 * slot,
        "peak {peak} bytes for one kept tagged field"
    );
    encoding_shares_records().unwrap();
    hostile_messages_fail_small().unwrap();
    growing_collections_stay_within_the_budget().unwrap();
}

/// Whether a decode from the reader succeeds.
type Decodes = fn(Reader) -> bool;

/// PROTO-0134: a decode never holds more than it was charged, even while a collection
/// grows and its old and new blocks are both live. Each case decodes under the smallest
/// budget it succeeds with, which must then bound its peak.
fn growing_collections_stay_within_the_budget() -> Result<(), String> {
    // A response header keeping 33 unknown tagged fields, grown by doubling to 64.
    let mut writer = Writer::new();
    writer.write_i32(1);
    writer.write_unsigned_varint(33);
    for tag in 0..33 {
        writer.write_unsigned_varint(tag);
        writer.write_unsigned_varint(0);
    }
    let tagged = writer.into_bytes();
    // An array of 1001 int8, past the 1000 elements allocated up front.
    let mut writer = Writer::new();
    writer
        .write_array_length(Framing::Classic, 1001)
        .map_err(|error| error.to_string())?;
    for _ in 0..1001 {
        writer.write_i8(0);
    }
    let array = writer.into_bytes();
    let cases: [(&str, Bytes, Decodes); 2] = [
        ("33 unknown tagged fields", tagged, |mut reader| {
            ResponseHeader::decode(&mut reader, 1).is_ok()
        }),
        ("an array of 1001 elements", array, |mut reader| {
            reader.read_array(Framing::Classic, Reader::read_i8).is_ok()
        }),
    ];
    for (name, input, decodes) in cases {
        // Sharing the input allocates its shared header now, not during the decode.
        let input = input.clone();
        let with_budget = |allocation_budget| {
            let limits = DecodeLimits {
                allocation_budget,
                ..DecodeLimits::default()
            };
            Reader::with_limits(input.clone(), limits)
        };
        let (mut low, mut high) = (0_usize, 1 << 20);
        while low < high {
            let middle = low + (high - low) / 2;
            if decodes(with_budget(middle)) {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        let reader = with_budget(low);
        let peak = peak_during(|| assert!(decodes(reader)));
        if peak > low {
            return Err(format!(
                "{name}: peak {peak} bytes under the smallest budget it decodes with, {low}"
            ));
        }
    }
    Ok(())
}

/// A decode that fails, and the error it must fail with.
type Decode = Box<dyn Fn() -> Option<DecodeError>>;
type Expected = fn(&DecodeErrorKind) -> bool;

/// PROTO-0113: hostile sizes in tiny inputs, through generated decoders and the request
/// header, each failing as Kafka's checks require with under 64 KiB allocated.
fn hostile_messages_fail_small() -> Result<(), String> {
    let cases: [(&str, Decode, Expected); 4] = [
        (
            "an int32 array length of 2^31 - 1",
            Box::new(|| {
                let input = Bytes::from_static(&[0x7f, 0xff, 0xff, 0xff, 1]);
                MetadataRequest::decode(&mut Reader::new(input), 4).err()
            }),
            |kind| {
                matches!(
                    kind,
                    DecodeErrorKind::ArrayLongerThanInput {
                        length: 0x7fff_ffff,
                        ..
                    }
                )
            },
        ),
        (
            "a compact length of 0xFFFFFFFF, which Java reads as null",
            Box::new(|| {
                let input = Bytes::from_static(&[0xff, 0xff, 0xff, 0xff, 0x0f, 0, 0]);
                SaslAuthenticateRequest::decode(&mut Reader::new(input), 2).err()
            }),
            |kind| matches!(kind, DecodeErrorKind::UnexpectedNull),
        ),
        (
            "10,001 tagged fields in a request header",
            Box::new(|| {
                let mut writer = Writer::new();
                writer.write_i16(10);
                writer.write_i16(4);
                writer.write_i32(1);
                writer.write_i16(0);
                writer.write_unsigned_varint(10_001);
                let mut input = writer.into_bytes().to_vec();
                input.resize(input.len() + 10_001 * 2, 0);
                match decode_request_header(&mut Reader::new(Bytes::from(input))) {
                    Err(HeaderError::Malformed { error, .. }) => Some(error),
                    _ => None,
                }
            }),
            |kind| {
                matches!(
                    kind,
                    DecodeErrorKind::LimitExceeded {
                        limit: Limit::TaggedFieldCount,
                        requested: 10_001,
                        ..
                    }
                )
            },
        ),
        (
            "nested arrays each declaring every byte left",
            Box::new(|| {
                // CreateTopicsRequest v5: 63 topics, the first with 55 assignments, the
                // first of those with 50 broker ids, in 64 bytes.
                let mut input = vec![0x40, 0x01, 0, 0, 0, 1, 0, 1, 0x38, 0, 0, 0, 0, 0x33];
                input.resize(64, 0xab);
                CreateTopicsRequest::decode(&mut Reader::new(Bytes::from(input)), 5).err()
            }),
            |kind| matches!(kind, DecodeErrorKind::Truncated { .. }),
        ),
    ];
    for (name, decode, expected) in cases {
        let mut error = None;
        let peak = peak_during(|| error = decode());
        let error = error.ok_or_else(|| format!("{name}: decoded"))?;
        if !expected(&error.kind) {
            return Err(format!("{name}: {error}"));
        }
        if peak >= 64 << 10 {
            return Err(format!("{name}: peak {peak} bytes"));
        }
    }
    Ok(())
}

/// PROTO-0102: a `FetchResponse` of 100 partitions, each with 64 KiB of records, encodes
/// into a frame that shares every batch: the encode allocates the response's own few
/// kilobytes, never the 6.4 MiB of records.
fn encoding_shares_records() -> Result<(), Box<dyn std::error::Error>> {
    let batch = Bytes::from(vec![7_u8; 64 << 10]);
    let response = FetchResponse {
        responses: vec![FetchableTopicResponse {
            partitions: (0..100)
                .map(|partition_index| PartitionData {
                    partition_index,
                    records: Some(RecordsField::new(batch.clone())),
                    ..PartitionData::default()
                })
                .collect(),
            ..FetchableTopicResponse::default()
        }],
        ..FetchResponse::default()
    };
    let mut encoded = None;
    let peak = peak_during(|| {
        let mut writer = Writer::sharing();
        encoded = Some(
            response
                .encode(&mut writer, 18)
                .map(|()| writer.into_frame()),
        );
    });
    let frame = encoded.ok_or("the encode didn't run")??;
    assert!(
        peak < 64 << 10,
        "peak {peak} bytes to encode 6.4 MiB of records"
    );
    assert_eq!(frame.len(), response.encoded_size(18)?.total);
    let shared: Vec<_> = frame
        .segments()
        .iter()
        .filter(|segment| segment.as_ptr() == batch.as_ptr())
        .collect();
    assert_eq!(shared.len(), 100, "every batch is its own shared segment");
    assert_eq!(
        frame.segments().len(),
        201,
        "owned bytes between batches are one segment"
    );
    Ok(())
}
