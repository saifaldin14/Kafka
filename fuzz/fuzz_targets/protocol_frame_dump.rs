//! `dump_request_frame` on arbitrary input, and `dump_response_frame` on all but its
//! first two bytes, which pick the API and the version: no panic, and every byte of the
//! input is accounted for, in the size prefix, the frame read, or beyond it.
#![no_main]

use alluvium_protocol::{ApiKey, FrameDump, dump_request_frame, dump_response_frame};
use bytes::Bytes;
use libfuzzer_sys::fuzz_target;

/// Every byte of `input` is in the prefix, the frame or beyond it.
fn accounted(dump: &FrameDump, input: &[u8]) {
    assert_eq!(4 + dump.present + dump.beyond_frame, input.len());
    assert!(dump.present <= dump.declared_size);
}

fuzz_target!(|input: &[u8]| {
    let Some((&api, rest)) = input.split_first() else {
        return;
    };
    let Some((&version, frame)) = rest.split_first() else {
        return;
    };
    if let Ok(dump) = dump_request_frame(&Bytes::copy_from_slice(input)) {
        accounted(&dump, input);
    }
    let api = ApiKey::ALL[usize::from(api) % ApiKey::ALL.len()];
    if let Ok(dump) =
        dump_response_frame(&Bytes::copy_from_slice(frame), api, i16::from(version % 32))
    {
        accounted(&dump, frame);
    }
});
