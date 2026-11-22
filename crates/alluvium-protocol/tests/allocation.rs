//! Peak heap use while decoding hostile input, measured by a counting global allocator.
//!
//! Everything runs in one test so that no other test allocates at the same time, under
//! nextest or `cargo test` alike.

use std::mem::size_of;

use alluvium_protocol::{Framing, Reader, Writer};
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
}
