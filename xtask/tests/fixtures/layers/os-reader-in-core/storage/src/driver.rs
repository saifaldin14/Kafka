//! A layer 1 crate reading OS entropy: rejected.

pub fn seed() {
    let _rng = SeededRng::from_os_entropy();
}
