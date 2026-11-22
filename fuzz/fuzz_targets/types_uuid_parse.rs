//! `Uuid::from_str` on arbitrary text: no panic, and a parsed id prints as text that
//! parses back to the same id.
#![no_main]

use std::str::FromStr;

use alluvium_types::Uuid;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &[u8]| {
    let Ok(text) = std::str::from_utf8(input) else {
        return;
    };
    if let Ok(id) = Uuid::from_str(text) {
        assert_eq!(Uuid::from_str(&id.to_string()), Ok(id));
    }
});
