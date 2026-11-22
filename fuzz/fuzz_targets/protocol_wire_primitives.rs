//! Every primitive reader in `alluvium-protocol`, in a sequence the input chooses: no
//! panic, no allocation beyond the decode budget of 64 bytes per input byte, and every
//! value read encodes and then decodes to itself.
//!
//! Each step reads an op byte: bit 7 chooses the framing, the rest which reader runs on
//! the bytes that follow. The run stops at the first error.
#![no_main]

use alluvium_protocol::{DecodeError, DecodeLimits, EncodeError, Framing, Reader, Writer};
use bytes::Bytes;
use libfuzzer_sys::fuzz_target;

/// The decode budget per input byte. A correct reader allocates at most a small
/// multiple of its input, so the budget, not libFuzzer's malloc limit, bounds a run.
const BUDGET_PER_INPUT_BYTE: usize = 64;

fuzz_target!(|input: &[u8]| {
    let limits = DecodeLimits {
        allocation_budget: input.len().saturating_mul(BUDGET_PER_INPUT_BYTE),
        ..DecodeLimits::default()
    };
    let mut reader = Reader::with_limits(Bytes::copy_from_slice(input), limits);
    while let Ok(op) = reader.read_i8() {
        let framing = if op < 0 {
            Framing::Compact
        } else {
            Framing::Classic
        };
        if step(op & 0x7f, framing, &mut reader).is_err() {
            break;
        }
    }
});

/// Reads one value with the reader `op` selects, and checks it round-trips.
fn step(op: i8, framing: Framing, reader: &mut Reader) -> Result<(), DecodeError> {
    match op % 20 {
        0 => round_trip(reader.read_i8()?, Writer::write_i8, Reader::read_i8),
        1 => round_trip(reader.read_i16()?, Writer::write_i16, Reader::read_i16),
        2 => round_trip(reader.read_i32()?, Writer::write_i32, Reader::read_i32),
        3 => round_trip(reader.read_i64()?, Writer::write_i64, Reader::read_i64),
        4 => round_trip(reader.read_u16()?, Writer::write_u16, Reader::read_u16),
        5 => round_trip(reader.read_u32()?, Writer::write_u32, Reader::read_u32),
        6 => round_trip(
            reader.read_f64()?.to_bits(),
            |w, bits| w.write_f64(f64::from_bits(bits)),
            |r| r.read_f64().map(f64::to_bits),
        ),
        7 => round_trip(reader.read_bool()?, Writer::write_bool, Reader::read_bool),
        8 => round_trip(reader.read_uuid()?, Writer::write_uuid, Reader::read_uuid),
        9 => round_trip(
            reader.read_unsigned_varint()?,
            Writer::write_unsigned_varint,
            Reader::read_unsigned_varint,
        ),
        10 => round_trip(
            reader.read_varint()?,
            Writer::write_varint,
            Reader::read_varint,
        ),
        11 => round_trip(
            reader.read_varlong()?,
            Writer::write_varlong,
            Reader::read_varlong,
        ),
        12 => round_trip(
            reader.read_string(framing)?,
            |w, value| {
                w.write_string(framing, &value)
                    .expect("a decoded string encodes")
            },
            |r| r.read_string(framing),
        ),
        13 => round_trip(
            reader.read_nullable_string(framing)?,
            |w, value| {
                w.write_nullable_string(framing, value.as_deref())
                    .expect("a decoded string encodes");
            },
            |r| r.read_nullable_string(framing),
        ),
        14 => round_trip(
            reader.read_bytes(framing)?,
            |w, value| {
                w.write_bytes(framing, &value)
                    .expect("decoded bytes encode")
            },
            |r| r.read_bytes(framing),
        ),
        15 => round_trip(
            reader.read_nullable_bytes(framing)?,
            |w, value| {
                w.write_nullable_bytes(framing, value.as_deref())
                    .expect("decoded bytes encode");
            },
            |r| r.read_nullable_bytes(framing),
        ),
        16 => round_trip(
            reader.read_records(framing)?,
            |w, value| {
                w.write_records(framing, value.as_ref())
                    .expect("decoded records encode")
            },
            |r| r.read_records(framing),
        ),
        17 => round_trip(
            reader.read_array(framing, Reader::read_i32)?,
            |w, value| {
                w.write_array_length(framing, value.len())
                    .expect("a decoded array length encodes");
                for &element in &value {
                    w.write_i32(element);
                }
            },
            |r| r.read_array(framing, Reader::read_i32),
        ),
        18 => round_trip(
            reader.read_nullable_array(framing, Reader::read_i32)?,
            |w, value| {
                w.write_nullable_array_length(framing, value.as_ref().map(Vec::len))
                    .expect("a decoded array length encodes");
                for &element in value.iter().flatten() {
                    w.write_i32(element);
                }
            },
            |r| r.read_nullable_array(framing, Reader::read_i32),
        ),
        _ => {
            let fields = reader.read_tagged_fields()?;
            if fields
                .windows(2)
                .all(|pair| matches!(pair, [a, b] if a.tag < b.tag))
            {
                round_trip(
                    fields,
                    |w, value| {
                        w.write_tagged_fields(&value)
                            .expect("ascending tags encode")
                    },
                    Reader::read_tagged_fields,
                );
            } else {
                // Kafka reads tags in any order but writes only ascending ones, so a
                // section read out of order is refused, never written differently.
                assert!(matches!(
                    Writer::new().write_tagged_fields(&fields),
                    Err(EncodeError::UnsortedRawTags { .. })
                ));
            }
        }
    }
    Ok(())
}

/// Encodes `value` with `write`, decodes it with `read`, and asserts it is unchanged.
fn round_trip<T: PartialEq + std::fmt::Debug + Clone>(
    value: T,
    write: impl FnOnce(&mut Writer, T),
    read: impl FnOnce(&mut Reader) -> Result<T, DecodeError>,
) {
    let mut writer = Writer::new();
    write(&mut writer, value.clone());
    let mut encoded = Reader::new(writer.into_bytes());
    assert_eq!(read(&mut encoded), Ok(value));
    assert_eq!(
        encoded.remaining(),
        0,
        "the encoding has bytes the decoder left"
    );
}
