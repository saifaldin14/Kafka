//! `PayloadSummary`: how record payloads appear in logs, panics and debug output.

use std::fmt;

/// Prints a payload as its length and CRC32C, never its contents:
/// `Bytes { len: 1048576, crc32c: 0x1a2b3c4d }`.
///
/// Record keys, values and headers may hold personal or secret data, so they never
/// appear in logs ([debuggability standard](../../../docs/standards/debuggability.md#printing-values)).
/// Wrap every payload-bearing field in log output with this type. The length and
/// checksum are still enough to tell two payloads apart, or to match a payload against
/// the one a client sent.
///
/// The checksum is computed only when the value is formatted, so building a summary
/// for a log line that is filtered out costs nothing. Formatting costs one CRC32C pass
/// over the payload, which uses the CPU's CRC instructions: the budget is under a
/// millisecond for a 1 MiB payload, and the `types/payload_summary` benchmark measures
/// it.
#[derive(Clone, Copy)]
pub struct PayloadSummary<'a>(pub &'a [u8]);

impl fmt::Debug for PayloadSummary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Parity: clients/src/main/java/org/apache/kafka/common/utils/internals/Crc32C.java@4.4.0 Crc32C.compute
        // (the checksum Kafka uses for record batches).
        let crc = crc32c::crc32c(self.0);
        write!(f, "Bytes {{ len: {}, crc32c: {crc:#010x} }}", self.0.len())
    }
}

impl fmt::Display for PayloadSummary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use proptest::collection::vec;
    use proptest::prelude::{Strategy, any, prop_assert, prop_assert_eq, prop_oneof, proptest};

    use super::PayloadSummary;

    /// Standard base64 with padding; the URL-safe form swaps `+/` for `-_`.
    fn base64(bytes: &[u8]) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let group = chunk
                .iter()
                .enumerate()
                .fold(0_usize, |group, (index, byte)| {
                    group | usize::from(*byte) << (16 - 8 * index)
                });
            for index in 0..4 {
                if index <= chunk.len() {
                    out.push(char::from(ALPHABET[(group >> (18 - 6 * index)) & 0x3f]));
                } else {
                    out.push('=');
                }
            }
        }
        out
    }

    /// ASCII text, any UTF-8 and arbitrary bytes, at least 12 bytes long: every
    /// encoding of them is then longer than anything variable in a summary (8 hex
    /// digits of CRC, a length), so it could only appear by leaking.
    fn payload() -> impl Strategy<Value = Vec<u8>> {
        prop_oneof![
            vec(0x20_u8..0x7f, 12..256),
            any::<String>()
                .prop_filter("at least 12 bytes", |text| text.len() >= 12)
                .prop_map(String::into_bytes),
            vec(any::<u8>(), 12..256),
        ]
    }

    proptest! {
        #[test]
        fn the_summary_reveals_only_length_and_checksum(payload in payload()) {
            let shown = PayloadSummary(&payload).to_string();
            // Fully determined by the length and the CRC, so the contents can't reach it.
            prop_assert_eq!(
                &shown,
                &format!("Bytes {{ len: {}, crc32c: {:#010x} }}", payload.len(), crc32c::crc32c(&payload))
            );
            let hex: String = payload
                .iter()
                .flat_map(|byte| [byte >> 4, byte & 0xf])
                .filter_map(|nibble| char::from_digit(u32::from(nibble), 16))
                .collect();
            let standard = base64(&payload);
            let url_safe = standard.replace('+', "-").replace('/', "_");
            for encoding in [
                String::from_utf8_lossy(&payload).into_owned(),
                hex.clone(),
                hex.to_uppercase(),
                standard.trim_end_matches('=').to_owned(),
                url_safe.trim_end_matches('=').to_owned(),
            ] {
                prop_assert!(!shown.contains(&encoding), "{:?} leaked into {:?}", encoding, shown);
            }
        }
    }

    #[test]
    fn the_base64_helper_matches_rfc_4648() {
        // RFC 4648 section 10 test vectors.
        for (input, expected) in [
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(input.as_bytes()), expected);
        }
    }

    #[test]
    fn the_crc_is_kafkas_crc32c() {
        // Parity: clients/src/main/java/org/apache/kafka/common/utils/internals/Crc32C.java@4.4.0 Crc32C.compute
        // The standard CRC-32C check value, which java.util.zip.CRC32C also gives.
        assert_eq!(
            PayloadSummary(b"123456789").to_string(),
            "Bytes { len: 9, crc32c: 0xe3069283 }"
        );
    }

    #[test]
    fn the_empty_payload_prints_length_zero() {
        assert_eq!(
            format!("{:?}", PayloadSummary(&[])),
            "Bytes { len: 0, crc32c: 0x00000000 }"
        );
    }
}
