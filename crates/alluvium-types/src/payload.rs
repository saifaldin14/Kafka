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
/// for a log line that is filtered out costs nothing.
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
    use super::PayloadSummary;

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

    #[test]
    fn the_contents_never_appear() {
        let printed = format!("{:?}", PayloadSummary(b"card=4111111111111111"));
        assert!(!printed.contains("4111"), "{printed}");
        assert!(
            printed.starts_with("Bytes { len: 21, crc32c: 0x"),
            "{printed}"
        );
    }
}

fn   badly_formatted( ){}
