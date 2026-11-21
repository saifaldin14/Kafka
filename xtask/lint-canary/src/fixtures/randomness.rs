//! Unseeded randomness, including the random hash seeds of `std`'s hash maps, which
//! make iteration order differ between runs and break deterministic replay.

use std::collections::{HashMap, HashSet};

/// Taking the thread-local RNG instead of the injected seeded one.
pub fn rand_rng() -> rand::rngs::ThreadRng {
    rand::rng() // expect-lint: clippy::disallowed_methods
}

/// A hash map with a random seed.
pub fn hash_map_new() -> HashMap<u8, u8> {
    HashMap::new() // expect-lint: clippy::disallowed_methods
}

/// A hash map with a random seed and a capacity.
pub fn hash_map_with_capacity() -> HashMap<u8, u8> {
    HashMap::with_capacity(1) // expect-lint: clippy::disallowed_methods
}

/// A hash set with a random seed.
pub fn hash_set_new() -> HashSet<u8> {
    HashSet::new() // expect-lint: clippy::disallowed_methods
}

/// A hash set with a random seed and a capacity.
pub fn hash_set_with_capacity() -> HashSet<u8> {
    HashSet::with_capacity(1) // expect-lint: clippy::disallowed_methods
}

/// Naming the random hasher state directly. Any mention fires the lint, including a
/// `use` of it.
pub type Hasher = std::collections::hash_map::RandomState; // expect-lint: clippy::disallowed_types
