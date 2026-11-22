//! Seeded randomness ([ADR-0012](../../../docs/adr/0012-seeded-randomness-and-hashing.md)).
//!
//! [`SeededRng`] is the workspace's only random number generator. Simulation and tests
//! seed it with a `u64`, so a printed seed replays a run exactly; production seeds it
//! once from the operating system and forks a stream for each component.
//!
//! Every output is built here from raw `ChaCha12` keystream words, so it depends only on
//! the `ChaCha12` algorithm and this file, never on `rand`'s distributions or
//! `rand_core`'s helpers, which may change between releases. The golden tests pin
//! the first outputs; a change that alters any of them breaks every stored seed.

use std::fmt;
use std::num::NonZeroU64;

use chacha20::ChaCha12Rng;
use chacha20::rand_core::{Rng, SeedableRng};
use siphasher::sip::SipHasher13;

/// A deterministic, cryptographically secure random number generator: the `ChaCha12`
/// keystream under a 256-bit key.
///
/// It is not `Clone`: two copies would draw the same values. Use [`SeededRng::fork`]
/// to give a component its own stream. `Debug` prints `SeededRng { .. }`, never the key
/// or position, because knowing them would let an attacker predict every later value.
pub struct SeededRng {
    keystream: ChaCha12Rng,
}

/// The operating system couldn't provide entropy. Startup fails; there is no weaker
/// fallback (ADR-0012).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the operating system could not provide entropy")]
pub struct EntropyError {
    #[source]
    source: getrandom::Error,
}

/// The `SipHash` key that maps a fork label to a `ChaCha` stream number. It is public and
/// fixed on purpose: it only spreads labels over streams, and a stream number reveals
/// nothing without the parent's key.
const FORK_STREAM_KEY: (u64, u64) = (
    u64::from_le_bytes(*b"alluvium"),
    u64::from_le_bytes(*b"rngforks"),
);

impl SeededRng {
    /// A generator that replays exactly from `seed`. For simulation and tests; print the
    /// seed with any failure.
    ///
    /// The seed is expanded to the 256-bit key with four `SplitMix64` outputs, written
    /// little-endian. The expansion is fixed here rather than taken from `rand_core`,
    /// whose `seed_from_u64` may change.
    pub fn from_seed(seed: u64) -> SeededRng {
        let mut state = seed;
        let mut key = [0; 32];
        for chunk in key.as_chunks_mut::<8>().0 {
            *chunk = splitmix64(&mut state).to_le_bytes();
        }
        SeededRng::from_key(key)
    }

    /// A generator keyed from the operating system's entropy: the production root, read
    /// once at startup and forked for every component. Layer 0–2 code never calls it;
    /// it receives a generator, or values drawn from one, from its driver.
    ///
    /// # Errors
    ///
    /// [`EntropyError`] if the operating system can't provide entropy.
    pub fn from_os_entropy() -> Result<SeededRng, EntropyError> {
        let mut key = [0; 32];
        os_entropy(&mut key).map_err(|source| EntropyError { source })?;
        Ok(SeededRng::from_key(key))
    }

    fn from_key(key: [u8; 32]) -> SeededRng {
        SeededRng {
            keystream: ChaCha12Rng::from_seed(key),
        }
    }

    /// The next 32 bits of the keystream.
    pub fn next_u32(&mut self) -> u32 {
        self.keystream.next_u32()
    }

    /// The next two keystream words, the first as the low half.
    pub fn next_u64(&mut self) -> u64 {
        let low = self.next_u32();
        let high = self.next_u32();
        u64::from(low) | (u64::from(high) << 32)
    }

    /// Fills `bytes` with keystream words, little-endian. A final partial word uses its
    /// first bytes and discards the rest.
    pub fn fill_bytes(&mut self, bytes: &mut [u8]) {
        for chunk in bytes.chunks_mut(4) {
            let word = self.next_u32().to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }
    }

    /// A uniformly distributed value in `0..bound`, without modulo bias.
    ///
    /// Lemire's multiply-and-reject method ("Fast Random Integer Generation in an
    /// Interval", 2019): the high half of `x * bound` is the result, and draws whose low
    /// half falls in the biased region are rejected.
    pub fn below(&mut self, bound: NonZeroU64) -> u64 {
        let bound = bound.get();
        let (mut high, mut low) = wide_mul(self.next_u64(), bound);
        if low < bound {
            // 2^64 mod bound: how many low halves would make some results more likely.
            let biased = bound.wrapping_neg() % bound;
            while low < biased {
                (high, low) = wide_mul(self.next_u64(), bound);
            }
        }
        high
    }

    /// An independent generator for the component named `label`, such as `"raft"` or
    /// `"group-coordinator/payments"`.
    ///
    /// The child's key is the first 32 bytes of `ChaCha12` under this generator's key, on
    /// the stream that `SipHash-1-3` picks for the label (never stream 0, this
    /// generator's own). It depends only on this generator's key and the label: forking
    /// draws nothing from this generator, so adding or removing a fork never shifts
    /// another stream. `ChaCha` is a pseudorandom function of its key, so a child's output
    /// reveals nothing about its parent or siblings.
    #[must_use]
    pub fn fork(&self, label: &str) -> SeededRng {
        let mut derivation = SeededRng::from_key(self.keystream.get_seed());
        derivation.keystream.set_stream(fork_stream(label));
        let mut key = [0; 32];
        derivation.fill_bytes(&mut key);
        SeededRng::from_key(key)
    }
}

impl fmt::Debug for SeededRng {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SeededRng").finish_non_exhaustive()
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "SeededRng::from_os_entropy is the one sanctioned OS-entropy read (ADR-0012)"
)]
fn os_entropy(key: &mut [u8; 32]) -> Result<(), getrandom::Error> {
    getrandom::fill(key)
}

/// One `SplitMix64` step (Steele, Lea and Flood, 2014).
pub(crate) fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn fork_stream(label: &str) -> u64 {
    let (key0, key1) = FORK_STREAM_KEY;
    SipHasher13::new_with_keys(key0, key1)
        .hash(label.as_bytes())
        .max(1)
}

/// The high and low halves of the 128-bit product `a * b`.
#[expect(
    clippy::cast_possible_truncation,
    reason = "each half of a 128-bit product is exactly 64 bits"
)]
fn wide_mul(a: u64, b: u64) -> (u64, u64) {
    let product = u128::from(a) * u128::from(b);
    ((product >> 64) as u64, product as u64)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use proptest::prelude::{any, prop_assert, proptest};

    use super::{SeededRng, splitmix64};

    // The goldens were computed by an independent Python implementation of ChaCha12,
    // SplitMix64, SipHash-1-3 and Lemire's method, itself checked against the RFC 7539
    // ChaCha20 block vector, the SipHash-2-4 paper vector and SplitMix64's published
    // first output. CI runs them on Linux x86_64 and macOS aarch64, which use different
    // ChaCha backends.

    fn first_u64s(rng: &mut SeededRng, count: usize) -> Vec<u64> {
        (0..count).map(|_| rng.next_u64()).collect()
    }

    fn bound(value: u64) -> NonZeroU64 {
        NonZeroU64::new(value).unwrap()
    }

    #[test]
    fn splitmix64_matches_its_published_first_output() {
        let mut state = 0;
        assert_eq!(splitmix64(&mut state), 0xe220_a839_7b1d_cdaf);
    }

    #[test]
    fn fixed_seeds_give_fixed_streams() {
        let cases: [(u64, [u64; 4]); 3] = [
            (
                0,
                [
                    0xd18c_9d7b_82b6_7bca,
                    0x73f1_688a_dd8c_2eb1,
                    0x65b1_6a72_2bbe_7197,
                    0x5445_15e3_ab5c_eb0a,
                ],
            ),
            (
                42,
                [
                    0x280b_7b79_f392_fa12,
                    0x4dad_ef83_bc93_1d07,
                    0xc195_c99b_a537_5e5f,
                    0x7e65_7f1b_6bdc_3bfd,
                ],
            ),
            (
                u64::MAX,
                [
                    0xfaad_820e_1019_8c2a,
                    0xcbe4_ff9d_a3a9_3d15,
                    0x1787_2c99_9978_ada3,
                    0xb06d_cc25_cfc7_66f4,
                ],
            ),
        ];
        for (seed, expected) in cases {
            assert_eq!(
                first_u64s(&mut SeededRng::from_seed(seed), 4),
                expected,
                "seed {seed}"
            );
        }
    }

    #[test]
    fn words_bytes_and_u64s_come_from_one_keystream() {
        let mut rng = SeededRng::from_seed(1);
        assert_eq!(
            [rng.next_u32(), rng.next_u32(), rng.next_u32()],
            [0x6892_9713, 0x200c_5d91, 0xe5b1_3971]
        );

        // Seven bytes use two words and discard the last byte of the second.
        let mut rng = SeededRng::from_seed(1);
        let mut bytes = [0; 7];
        rng.fill_bytes(&mut bytes);
        assert_eq!(bytes, [0x13, 0x97, 0x92, 0x68, 0x91, 0x5d, 0x0c]);
        assert_eq!(rng.next_u32(), 0xe5b1_3971);

        let mut rng = SeededRng::from_seed(1);
        assert_eq!(rng.next_u64(), 0x200c_5d91_6892_9713);
    }

    #[test]
    fn forks_give_fixed_streams() {
        let parent = SeededRng::from_seed(42);
        let cases = [
            (
                "raft",
                [
                    0xa46a_b12e_d407_64c0,
                    0x0d54_fe7f_796b_1cfb,
                    0x0930_fc20_0302_36e6,
                ],
            ),
            (
                "group-coordinator/payments",
                [
                    0x6a41_f9c4_94ee_b2a9,
                    0x9702_e2a0_5d40_4918,
                    0xcdd5_c530_d34b_867a,
                ],
            ),
            (
                "",
                [
                    0x3fbb_5576_10d0_16a2,
                    0x5959_c4b1_b86f_4a4f,
                    0x31d8_b363_2857_aa6d,
                ],
            ),
        ];
        for (label, expected) in cases {
            assert_eq!(
                first_u64s(&mut parent.fork(label), 3),
                expected,
                "{label:?}"
            );
        }
        assert_eq!(
            first_u64s(&mut parent.fork("raft").fork("node-1"), 2),
            [0x1bcf_6b17_c87b_ed6e, 0x2059_c645_3a9b_66d0]
        );
    }

    #[test]
    fn forking_never_shifts_any_other_stream() {
        let untouched = first_u64s(&mut SeededRng::from_seed(42), 8);

        let mut parent = SeededRng::from_seed(42);
        let raft_before = first_u64s(&mut parent.fork("raft"), 4);
        let mut sibling = parent.fork("a-new-component");
        sibling.next_u64();
        // The parent's own stream and an existing fork are unchanged by a new fork,
        // and by the parent having drawn values.
        assert_eq!(first_u64s(&mut parent, 8), untouched);
        assert_eq!(first_u64s(&mut parent.fork("raft"), 4), raft_before);
        assert_ne!(
            first_u64s(&mut parent.fork("raft"), 4),
            first_u64s(&mut parent.fork("raft2"), 4)
        );
    }

    #[test]
    fn below_matches_lemires_method() {
        let mut rng = SeededRng::from_seed(7);
        let dice: Vec<u64> = (0..12).map(|_| rng.below(bound(6))).collect();
        assert_eq!(dice, [5, 0, 3, 3, 3, 2, 5, 2, 4, 1, 0, 3]);

        // Just over 2^63, so nearly half of all draws are rejected: these six values
        // take ten draws, which exercises the rejection loop.
        let mut rng = SeededRng::from_seed(7);
        let large: Vec<u64> = (0..6).map(|_| rng.below(bound((1 << 63) + 1))).collect();
        assert_eq!(
            large,
            [
                967_241_939_031_942_422,
                5_346_606_906_638_044_685,
                4_389_916_092_635_768_821,
                4_488_099_587_092_549_083,
                7_656_584_569_222_592_979,
                2_350_995_564_465_163_445,
            ]
        );

        let mut rng = SeededRng::from_seed(7);
        let widest: Vec<u64> = (0..3).map(|_| rng.below(bound(u64::MAX))).collect();
        assert_eq!(
            widest,
            [
                16_181_896_596_862_253_296,
                1_934_483_878_063_884_844,
                9_444_936_788_922_062_768
            ]
        );
    }

    #[test]
    fn debug_prints_no_state() {
        assert_eq!(
            format!("{:?}", SeededRng::from_seed(42)),
            "SeededRng { .. }"
        );
    }

    #[test]
    fn os_entropy_gives_different_generators() {
        let mut first = SeededRng::from_os_entropy().unwrap();
        let mut second = SeededRng::from_os_entropy().unwrap();
        // Equal only if the OS returned the same 256-bit key twice.
        assert_ne!(first_u64s(&mut first, 2), first_u64s(&mut second, 2));
    }

    proptest! {
        #[test]
        fn below_stays_below_the_bound(seed in any::<u64>(), limit in any::<u64>()) {
            let limit = bound(limit.max(1));
            let mut rng = SeededRng::from_seed(seed);
            for _ in 0..8 {
                prop_assert!(rng.below(limit) < limit.get());
            }
        }
    }
}
