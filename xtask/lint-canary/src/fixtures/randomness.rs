//! Unseeded randomness, including the random hash seeds of `std`'s hash maps, which
//! make iteration order differ between runs and break deterministic replay.

use std::collections::{HashMap, HashSet};

/// Taking the thread-local RNG instead of the injected seeded one.
pub fn rand_rng() -> impl rand::Rng {
    rand::rng() // expect-lint: clippy::disallowed_methods
}

/// A random value from the thread-local RNG.
pub fn rand_random() -> u64 {
    rand::random() // expect-lint: clippy::disallowed_methods
}

/// An endless stream from the thread-local RNG.
pub fn rand_random_iter() -> impl Iterator<Item = u8> {
    rand::random_iter() // expect-lint: clippy::disallowed_methods
}

/// A value in a range from the thread-local RNG.
pub fn rand_random_range() -> u8 {
    rand::random_range(0..10) // expect-lint: clippy::disallowed_methods
}

/// A coin flip from the thread-local RNG.
pub fn rand_random_bool() -> bool {
    rand::random_bool(0.5) // expect-lint: clippy::disallowed_methods
}

/// A ratio check from the thread-local RNG.
pub fn rand_random_ratio() -> bool {
    rand::random_ratio(1, 2) // expect-lint: clippy::disallowed_methods
}

/// Filling a buffer from the thread-local RNG.
pub fn rand_fill(buffer: &mut [u8]) {
    rand::fill(buffer); // expect-lint: clippy::disallowed_methods
}

/// An RNG seeded from OS entropy.
pub fn rand_make_rng() -> rand::rngs::StdRng {
    rand::make_rng() // expect-lint: clippy::disallowed_methods
}

/// The thread-local RNG through its `Default` impl. Clippy cannot ban a trait method
/// by path, so the type itself is disallowed.
pub fn thread_rng_default() -> impl rand::Rng {
    rand::rngs::ThreadRng::default() // expect-lint: clippy::disallowed_types
}

/// The OS RNG itself.
pub type OsEntropy = rand::rngs::SysRng; // expect-lint: clippy::disallowed_types

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
