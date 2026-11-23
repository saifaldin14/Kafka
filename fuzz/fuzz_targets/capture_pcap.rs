//! `read_capture` on an arbitrary file, then `exchanges` on each connection it finds: no
//! panic, and each direction's frames tile its stream in order.
#![no_main]

use alluvium_capture::exchanges::exchanges;
use alluvium_capture::read_capture;
use alluvium_capture::streams::StreamFrame;
use libfuzzer_sys::fuzz_target;

fn assert_contiguous(frames: &[StreamFrame]) {
    let mut next = None;
    for frame in frames {
        if let Some(next) = next {
            assert_eq!(frame.offset, next, "frames follow each other");
        }
        next = u64::try_from(frame.bytes.len())
            .ok()
            .and_then(|len| frame.offset.checked_add(len));
    }
}

fuzz_target!(|input: &[u8]| {
    let Ok(capture) = read_capture(input) else {
        return;
    };
    for connection in &capture.connections {
        assert_contiguous(&connection.requests);
        assert_contiguous(&connection.responses);
        let paired = exchanges(connection);
        assert_eq!(
            paired.exchanges.len(),
            connection.requests.len(),
            "every request is an exchange"
        );
    }
});
