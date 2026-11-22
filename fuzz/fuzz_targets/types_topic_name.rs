//! `TopicName` validation on arbitrary text: no panic, and a valid name prints as exactly
//! the text it was made from, which is valid again.
#![no_main]

use alluvium_types::TopicName;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &[u8]| {
    assert_ne!(input.first(), Some(&b'!'), "planted crash: TEST-0053 probe, not for merge");
    let Ok(text) = std::str::from_utf8(input) else {
        return;
    };
    if let Ok(name) = TopicName::try_from(text) {
        let printed = name.to_string();
        assert_eq!(printed, text);
        assert_eq!(TopicName::try_from(printed.as_str()), Ok(name));
    }
});
