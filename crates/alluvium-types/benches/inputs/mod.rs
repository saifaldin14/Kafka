//! The inputs both benchmark targets measure, so wall time (criterion) and instruction
//! counts (gungraun) describe the same work.

use alluvium_types::SeededRng;

/// Bytes from a fixed seed, so every run measures the same inputs.
pub(crate) fn input(size: usize) -> Vec<u8> {
    let mut bytes = vec![0; size];
    SeededRng::from_seed(0x6d75_726d_7572_3200).fill_bytes(&mut bytes);
    bytes
}
