//! `input::detect` and `input::parse` on arbitrary input, in every format: no panic, and
//! the bytes a dump holds, written as hex, read back the same.
#![no_main]

use std::fmt::Write as _;

use alluvium_capture::input::{InputFormat, detect, parse};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &[u8]| {
    let _ = detect(input);
    for format in [InputFormat::Raw, InputFormat::Hex, InputFormat::Xxd] {
        if let Ok(bytes) = parse(input, format) {
            let hex = bytes.iter().fold(String::new(), |mut hex, byte| {
                let _ = write!(hex, "{byte:02x}");
                hex
            });
            assert_eq!(parse(hex.as_bytes(), InputFormat::Hex), Ok(bytes));
        }
    }
});
