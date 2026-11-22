//! Peak heap use while decoding hostile input, measured by a counting global allocator.
//!
//! Everything runs in one test so that no other test allocates at the same time, under
//! nextest or `cargo test` alike.

use std::mem::size_of;

use alluvium_protocol::generated::response_header::ResponseHeader;
use alluvium_protocol::{
    DecodeErrorKind, DecodeLimits, Framing, Limit, Message, RawTaggedField, Reader, Writer,
};
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
fn declared_sizes_do_not_drive_allocation() {
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
    let empties = 1_000_000 * size_of::<Empty>();
    let charged = holders * size_of::<Holder>() + 4 * empties;
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
}
