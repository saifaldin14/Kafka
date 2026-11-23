//! The checks every generated fuzz target runs (`fuzz/generated/`, written by `cargo
//! xtask codegen`), so that each target is one call.
//!
//! Each decodes the input within a decode budget of [`BUDGET_PER_INPUT_BYTE`] per input
//! byte, and must not panic. The allocation oracle: the heap the decode uses at its
//! peak, which this crate's global allocator counts, must stay within that budget, so a
//! decoder that allocates without charging the budget fails. When the input decodes,
//! encoding the value and decoding that again must give the same bytes once more, and
//! `encoded_size` must equal their length. Bytes rather than values are compared,
//! because a decoded `NaN` never equals itself. Some values decode but don't encode, in
//! Kafka as here, and those encode errors end the check ([`kafka_cannot_write`]).

use std::fmt::Debug;

use alluvium_protocol::{
    DecodeLimits, EncodeError, HeaderError, Message, Reader, Writer, decode_request_header,
    decode_version_prefixed, encode_request_header, encode_version_prefixed,
};
use bytes::Bytes;
use peak_alloc::PeakAlloc;

/// The decode budget per input byte, as in `protocol_wire_primitives`: a correct decoder
/// allocates at most a small multiple of its input.
pub const BUDGET_PER_INPUT_BYTE: usize = 64;

/// What a decode may allocate beyond its budget: the error a failed decode returns,
/// whose field path grows with the nesting, which is at most 8 levels.
pub const ERROR_ALLOWANCE: usize = 1024;

#[global_allocator]
static HEAP: PeakAlloc = PeakAlloc;

/// Runs `decode` on `input` within the budget for its length, failing if the heap it
/// uses at its peak exceeds that budget.
fn decode_within_budget<T, E>(
    input: &[u8],
    decode: impl FnOnce(&mut Reader) -> Result<T, E>,
) -> Option<T> {
    let budget = input.len().saturating_mul(BUDGET_PER_INPUT_BYTE);
    let limits = DecodeLimits {
        allocation_budget: budget,
        ..DecodeLimits::default()
    };
    let mut reader = Reader::with_limits(Bytes::copy_from_slice(input), limits);
    let before = HEAP.current_usage();
    HEAP.reset_peak_usage();
    let decoded = decode(&mut reader);
    let used = HEAP.peak_usage().saturating_sub(before);
    let allowed = budget.saturating_add(ERROR_ALLOWANCE);
    assert!(
        used <= allowed,
        "decoding {} input bytes used {used} bytes of heap; the budget allows {allowed}",
        input.len()
    );
    decoded.ok()
}

/// Whether an encode error is one a decoded value may hit, because Kafka reads what it
/// can't write: a string whose invalid UTF-8 became more than 32767 bytes of U+FFFD, or
/// unknown tagged fields out of order or repeated, which Kafka keeps as they came.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java@4.4.0 Readable.readUnknownTaggedField
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/types/RawTaggedFieldWriter.java@4.4.0 RawTaggedFieldWriter.writeRawTags
fn kafka_cannot_write(error: &EncodeError) -> bool {
    matches!(
        error,
        EncodeError::StringTooLong { .. } | EncodeError::UnsortedRawTags { .. }
    )
}

/// `M` after its `int16` version, as Kafka's `MessageUtil.toVersionPrefixedBytes`
/// writes the data types a client sends. Requests and responses are read the same way,
/// so that every golden vector is a seed with its version in front.
pub fn message<M: Message + Debug>(input: &[u8]) {
    let Some((version, decoded)) = decode_within_budget(input, decode_version_prefixed::<M>)
    else {
        return;
    };
    let encoded = match encode_version_prefixed(&decoded, version) {
        Ok(encoded) => encoded,
        Err(error) if kafka_cannot_write(&error) => return,
        Err(error) => panic!("a decoded {decoded:?} doesn't encode: {error}"),
    };
    let size = decoded.encoded_size(version).expect("it just encoded").total;
    assert_eq!(size.saturating_add(2), encoded.len(), "encoded_size");
    let (_, again) = decode_version_prefixed::<M>(&mut Reader::new(encoded.clone()))
        .expect("what was encoded decodes");
    assert_eq!(
        encode_version_prefixed(&again, version).ok(),
        Some(encoded),
        "re-encoding"
    );
}

/// A request header, at the version its API key and version select, as a broker reads
/// it.
pub fn request_header(input: &[u8]) {
    let Some(decoded) = decode_within_budget(input, decode_request_header) else {
        return;
    };
    let mut writer = Writer::new();
    match encode_request_header(&decoded.header, &mut writer) {
        Ok(()) => {}
        Err(HeaderError::Encode(error)) if kafka_cannot_write(&error) => return,
        Err(error) => panic!("a decoded {decoded:?} doesn't encode: {error}"),
    }
    // The size may shrink: varints in the input need not be minimal, as in Kafka.
    let again = decode_request_header(&mut Reader::new(writer.into_bytes()))
        .expect("what was encoded decodes");
    assert_eq!(
        (again.api, again.header_version, &again.header),
        (decoded.api, decoded.header_version, &decoded.header),
        "re-decoding"
    );
}

#[cfg(test)]
mod tests {
    use super::decode_within_budget;

    // One test, so that no other test allocates while it measures. A decode over its
    // budget is the fixture for a decoder that allocates without charging the budget
    // (PROTO-0134).
    #[test]
    fn the_oracle_fails_a_decode_over_its_budget() {
        let decoded = decode_within_budget(&[0; 16], |_| Ok::<_, ()>(vec![0_u8; 16 * 64]));
        assert_eq!(decoded.map(|bytes| bytes.len()), Some(1024));
        let overrun = std::panic::catch_unwind(|| {
            decode_within_budget(&[0; 16], |_| Ok::<_, ()>(vec![0_u8; 64 * 1024]))
        });
        let message = overrun.expect_err("the oracle fails an overrun");
        let message = message.downcast_ref::<String>().expect("a formatted panic");
        assert!(
            message.starts_with("decoding 16 input bytes used 65536 bytes of heap"),
            "{message}"
        );
    }
}
