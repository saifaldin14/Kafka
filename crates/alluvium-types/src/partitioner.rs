//! Kafka's key hash: `murmur2`, which producers use to choose a key's partition.
//!
//! [`murmur2`] is a close port of `Utils.murmur2` from Apache Kafka 4.4.0
//! (`clients/src/main/java/org/apache/kafka/common/utils/Utils.java`), licensed under
//! the Apache License 2.0, and modified: rewritten in Rust with unsigned wrapping
//! arithmetic in place of Java's `int` overflow. Its golden test data was produced by
//! running that Kafka code (`tests/golden/generate.sh`).

/// Kafka's 32-bit `murmur2` hash of `data`: the hash producers apply to a record's key.
///
/// Java's `int` arithmetic wraps, so this computes on `u32` with wrapping operations
/// and returns the bits as `i32`, exactly as Kafka returns them.
// Parity: clients/src/main/java/org/apache/kafka/common/utils/Utils.java@4.4.0 Utils.murmur2
pub fn murmur2(data: &[u8]) -> i32 {
    const SEED: u32 = 0x9747_b28c;
    // Mixing constants from the original MurmurHash2.
    const M: u32 = 0x5bd1_e995;
    const R: u32 = 24;

    // Kafka keys are at most 2^31 - 1 bytes (a Java array length), so the length always
    // fits; the conversion keeps the low 32 bits as Java's `int` would.
    let length = u32::try_from(data.len()).unwrap_or(u32::MAX);
    let mut h = SEED ^ length;
    let (blocks, tail) = data.as_chunks::<4>();
    for block in blocks {
        let mut k = u32::from_le_bytes(*block);
        k = k.wrapping_mul(M);
        k ^= k >> R;
        k = k.wrapping_mul(M);
        h = h.wrapping_mul(M);
        h ^= k;
    }
    // Kafka's `switch` falls through from 3 to 2 to 1.
    match *tail {
        [a, b, c] => {
            h ^= u32::from(c) << 16;
            h ^= u32::from(b) << 8;
            h ^= u32::from(a);
            h = h.wrapping_mul(M);
        }
        [a, b] => {
            h ^= u32::from(b) << 8;
            h ^= u32::from(a);
            h = h.wrapping_mul(M);
        }
        [a] => {
            h ^= u32::from(a);
            h = h.wrapping_mul(M);
        }
        _ => {}
    }
    h ^= h >> 13;
    h = h.wrapping_mul(M);
    h ^= h >> 15;
    h.cast_signed()
}

#[cfg(test)]
pub(crate) mod tests {
    use proptest::prelude::{any, prop_assert_eq, proptest};

    use super::murmur2;
    use crate::rng::splitmix64;

    /// The inputs `tests/golden/kafka_hashes.jsh` hashes: from a `SplitMix64` `state`, an
    /// input of at most `max` bytes has length `next() mod (max + 1)`, then one byte
    /// per position, the low 8 bits of `next()`.
    pub(crate) fn golden_input(state: &mut u64, max: u64) -> Vec<u8> {
        let length = splitmix64(state) % (max + 1);
        (0..length)
            .map(|_| splitmix64(state).to_le_bytes()[0])
            .collect()
    }

    /// The data lines of a golden file, without its `#` comments.
    pub(crate) fn golden_lines(file: &str) -> impl Iterator<Item = &str> {
        file.lines().filter(|line| !line.starts_with('#'))
    }

    // Ported from clients/src/test/java/org/apache/kafka/common/utils/UtilsTest.java@4.4.0
    // (Apache License 2.0), testMurmur2, unchanged apart from the language.
    #[test]
    fn kafka_test_vectors() {
        let cases: [(&[u8], i32); 6] = [
            (b"21", -973_932_308),
            (b"foobar", -790_332_482),
            (b"a-little-bit-long-string", -985_981_536),
            (b"a-little-bit-longer-string", -1_486_304_829),
            (
                b"lkjh234lh9fiuh90y23oiuhsafujhadof229phr9h19h89h8",
                -58_897_971,
            ),
            (b"abc", 479_470_107),
        ];
        for (data, expected) in cases {
            assert_eq!(murmur2(data), expected, "{data:?}");
        }
    }

    #[test]
    fn matches_kafka_on_10000_random_inputs() {
        let golden = include_str!("../tests/golden/murmur2.txt");
        let mut state = 1;
        let mut count = 0;
        for (index, line) in golden_lines(golden).enumerate() {
            let input = golden_input(&mut state, 1024);
            let expected: i32 = line.parse().unwrap();
            assert_eq!(
                murmur2(&input),
                expected,
                "input {index}, {} bytes",
                input.len()
            );
            count += 1;
        }
        assert_eq!(count, 10_000);
    }

    /// Kafka's Java, transliterated statement by statement on signed `i32` with
    /// explicit fall-through, as an independent check of the idiomatic version.
    fn java_murmur2(data: &[u8]) -> i32 {
        let length = i32::try_from(data.len()).unwrap();
        let seed = 0x9747_b28c_u32.cast_signed();
        let m = 0x5bd1_e995_i32;
        let mut h = seed ^ length;
        let length4 = length >> 2;
        for i in 0..length4 {
            let i4 = usize::try_from(i << 2).unwrap();
            let mut k = i32::from_le_bytes(data[i4..i4 + 4].try_into().unwrap());
            k = k.wrapping_mul(m);
            k ^= (k.cast_unsigned() >> 24).cast_signed();
            k = k.wrapping_mul(m);
            h = h.wrapping_mul(m);
            h ^= k;
        }
        let index = usize::try_from(length4 << 2).unwrap();
        let remaining = data.len() - index;
        if remaining == 3 {
            h ^= i32::from(data[index + 2]) << 16;
        }
        if remaining >= 2 {
            h ^= i32::from(data[index + 1]) << 8;
        }
        if remaining >= 1 {
            h ^= i32::from(data[index]);
            h = h.wrapping_mul(m);
        }
        h ^= (h.cast_unsigned() >> 13).cast_signed();
        h = h.wrapping_mul(m);
        h ^= (h.cast_unsigned() >> 15).cast_signed();
        h
    }

    proptest! {
        // Any input, covering every block count and tail length: no panic, and the
        // same hash as the transliteration.
        #[test]
        fn matches_the_java_transliteration(data in proptest::collection::vec(any::<u8>(), 0..300)) {
            prop_assert_eq!(murmur2(&data), java_murmur2(&data));
        }
    }
}
