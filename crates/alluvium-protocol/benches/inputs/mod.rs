//! The inputs both benchmark targets measure, so wall time (criterion) and instruction
//! counts (gungraun) describe the same work.

use alluvium_protocol::{Framing, Writer};
use alluvium_types::SeededRng;
use bytes::Bytes;

/// How many varints one input holds.
pub(crate) const VARINTS: usize = 1000;

/// How many strings or byte arrays one input holds.
pub(crate) const VALUES: usize = 100;

/// `VARINTS` values that each take `width` bytes as unsigned varints (1 to 5).
pub(crate) fn varint_values(width: u32) -> Vec<u32> {
    // The smallest value that needs `width` bytes: 7 bits per byte.
    let value = 1_u32.checked_shl(7 * (width - 1)).unwrap_or(u32::MAX);
    vec![value; VARINTS]
}

/// [`varint_values`], encoded.
pub(crate) fn varints(width: u32) -> Bytes {
    let mut writer = Writer::new();
    for value in varint_values(width) {
        writer.write_unsigned_varint(value);
    }
    writer.into_bytes()
}

/// `VALUES` lowercase ASCII strings of `length` bytes, from a fixed seed.
pub(crate) fn strings(length: usize) -> Vec<String> {
    let mut rng = SeededRng::from_seed(0x7072_6f74_6f63_6f6c);
    (0..VALUES)
        .map(|_| {
            let mut bytes = vec![0; length];
            rng.fill_bytes(&mut bytes);
            bytes
                .iter()
                .map(|byte| char::from(b'a' + byte % 26))
                .collect()
        })
        .collect()
}

/// [`strings`], encoded as compact strings.
pub(crate) fn compact_strings(length: usize) -> Bytes {
    let mut writer = Writer::new();
    for text in strings(length) {
        // The strings are shorter than Kafka's 32767-byte limit, so this can't fail.
        let _ = writer.write_string(Framing::Compact, &text);
    }
    writer.into_bytes()
}

/// `VALUES` byte arrays of `length` bytes, from a fixed seed.
pub(crate) fn byte_arrays(length: usize) -> Vec<Vec<u8>> {
    let mut rng = SeededRng::from_seed(0x6279_7465_7300_0000);
    (0..VALUES)
        .map(|_| {
            let mut bytes = vec![0; length];
            rng.fill_bytes(&mut bytes);
            bytes
        })
        .collect()
}

/// [`byte_arrays`], encoded as compact bytes.
pub(crate) fn compact_bytes(length: usize) -> Bytes {
    let mut writer = Writer::new();
    for bytes in byte_arrays(length) {
        // The arrays are far shorter than an int32 length, so this can't fail.
        let _ = writer.write_bytes(Framing::Compact, &bytes);
    }
    writer.into_bytes()
}
