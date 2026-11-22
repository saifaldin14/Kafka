//! Hash maps and sets whose hashing is keyed from the injected RNG
//! ([ADR-0012](../../../docs/adr/0012-seeded-randomness-and-hashing.md)).
//!
//! `std`'s `HashMap::new` seeds its hasher randomly, so iteration order changes from run
//! to run and a simulation can't replay. These types hash with SipHash-1-3, the same
//! algorithm and hash-flooding resistance as `std`, under a [`HashSeed`] drawn from a
//! [`SeededRng`]: random per process in production, so attacker-chosen keys such as
//! topic names or client ids can't be made to collide, and fixed in simulation, so runs
//! replay exactly.
//!
//! Iteration order is still an implementation detail of `std`'s hash table. It must
//! never reach output or decisions, such as a response, a log line, or which partition
//! to move first. Where order is observable, use `BTreeMap` or `BTreeSet`.
//!
//! ```
//! use alluvium_types::SeededRng;
//! use alluvium_types::collections::{HashSeed, SeededHashMap};
//!
//! let mut rng = SeededRng::from_seed(42);
//! let mut leaders: SeededHashMap<&str, i32> = SeededHashMap::with_hasher(HashSeed::from_rng(&mut rng));
//! leaders.insert("payments", 3);
//! assert_eq!(leaders.get("payments"), Some(&3));
//! ```

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::BuildHasher;

use siphasher::sip::SipHasher13;

use crate::SeededRng;

/// A `HashMap` keyed by a [`HashSeed`]. Build it with `with_hasher(seed)` or
/// `with_capacity_and_hasher(capacity, seed)`; there is no unseeded `new` or `default`.
pub type SeededHashMap<K, V> = HashMap<K, V, HashSeed>;

/// A `HashSet` keyed by a [`HashSeed`]. Build it with `with_hasher(seed)` or
/// `with_capacity_and_hasher(capacity, seed)`; there is no unseeded `new` or `default`.
pub type SeededHashSet<T> = HashSet<T, HashSeed>;

/// The 128-bit `SipHash` key of a seeded map or set.
///
/// It is the map's `BuildHasher`. It has no `Default`, so every map is built from a
/// seed drawn from the injected RNG. `Debug` prints `HashSeed(***)`: a production seed
/// lets anyone who learns it build keys that all collide.
#[derive(Clone)]
pub struct HashSeed {
    key0: u64,
    key1: u64,
}

impl HashSeed {
    /// Draws a seed from `rng`.
    pub fn from_rng(rng: &mut SeededRng) -> HashSeed {
        HashSeed {
            key0: rng.next_u64(),
            key1: rng.next_u64(),
        }
    }
}

impl BuildHasher for HashSeed {
    type Hasher = SipHasher13;

    fn build_hasher(&self) -> SipHasher13 {
        SipHasher13::new_with_keys(self.key0, self.key1)
    }
}

impl fmt::Debug for HashSeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HashSeed(***)")
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{BuildHasher, Hasher};

    use super::{HashSeed, SeededHashMap, SeededHashSet};
    use crate::SeededRng;

    fn seed(value: u64) -> HashSeed {
        HashSeed::from_rng(&mut SeededRng::from_seed(value))
    }

    fn sip13(seed: &HashSeed, bytes: &[u8]) -> u64 {
        let mut hasher = seed.build_hasher();
        hasher.write(bytes);
        hasher.finish()
    }

    #[test]
    fn hashes_are_siphash_1_3_under_the_drawn_key() {
        // The key is SeededRng::from_seed(42)'s first two u64s. The expected hashes come
        // from the independent Python SipHash used for the SeededRng goldens.
        let seed = seed(42);
        assert_eq!(sip13(&seed, b"topic-a"), 0x9b4e_0818_bde4_8115);
        assert_eq!(sip13(&seed, b""), 0x0bdf_d745_8d5f_da6a);
        assert_eq!(
            sip13(&seed, b"group-coordinator/payments"),
            0x2ac3_4526_2b23_3636
        );
    }

    #[test]
    fn different_seeds_hash_the_same_key_differently() {
        assert_ne!(sip13(&seed(1), b"topic-a"), sip13(&seed(2), b"topic-a"));
    }

    #[test]
    fn the_same_seed_and_operations_give_the_same_iteration_order() {
        // What replay needs: two runs that do the same inserts under the same seed see
        // the same order. (The order also depends on insertion order, which is the
        // hash table's business and another reason it must never be observable.)
        let keys: Vec<String> = (0..200).map(|index| format!("topic-{index}")).collect();
        let map_order = |value: u64| {
            let mut map = SeededHashMap::with_capacity_and_hasher(16, seed(value));
            for key in &keys {
                map.insert(key.clone(), key.len());
            }
            map.into_keys().collect::<Vec<_>>()
        };
        assert_eq!(map_order(7), map_order(7));
        assert_ne!(map_order(7), map_order(8));
        assert_ne!(
            map_order(7),
            keys,
            "a hash map should not keep insertion order"
        );

        let set_order = |value: u64| {
            let mut set = SeededHashSet::with_hasher(seed(value));
            set.extend(keys.iter());
            set.into_iter().cloned().collect::<Vec<_>>()
        };
        assert_eq!(set_order(7), set_order(7));
        assert_ne!(set_order(7), set_order(8));
    }

    #[test]
    fn debug_hides_the_seed() {
        assert_eq!(format!("{:?}", seed(42)), "HashSeed(***)");
    }
}
