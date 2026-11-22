//! The key hash space and ranges over it
//! ([ADR-0013](../../../docs/adr/0013-key-hash-space.md)).
//!
//! A key's hash is Kafka's `toPositive(murmur2(key))`, a value in `[0, 2^31)`. Elastic
//! topics map half-open ranges of that space to partitions. Null keys have no hash.

use std::fmt;

use crate::partitioner::{murmur2, to_positive};

/// The exclusive end of the hash space, `2^31`.
const SPACE_END: u32 = 1 << 31;

/// A key's position in the hash space: Kafka's `toPositive(murmur2(key))`, the value
/// every Java producer computes for the key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyHash(u32);

/// A half-open range `[start, end)` of the hash space, with `end` at most `2^31`.
///
/// Half-open so that the whole space is one range ([`KeyRange::FULL`]), adjacent ranges
/// share a boundary, and splitting needs no `+ 1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyRange {
    start: u32,
    end: u32,
}

/// Why a key hash or range was rejected, or a range operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum KeyRangeError {
    /// A hash at or above `2^31`.
    #[error("key hash {value} is outside the hash space [0, 2^31)")]
    HashOutsideSpace {
        /// The rejected value.
        value: u32,
    },
    /// A range that ends after `2^31`.
    #[error("key range end {end} is beyond the hash space end 2^31")]
    EndBeyondSpace {
        /// The rejected end.
        end: u32,
    },
    /// A range whose start is after its end.
    #[error("key range start {start} is after its end {end}")]
    Inverted {
        /// The requested start.
        start: u32,
        /// The requested end.
        end: u32,
    },
    /// A split point that isn't strictly inside the range, so one half would be empty.
    #[error("split point {at} is not strictly inside the key range [{start}, {end})")]
    SplitNotInside {
        /// The range's start.
        start: u32,
        /// The range's end.
        end: u32,
        /// The rejected split point.
        at: u32,
    },
    /// Two ranges that don't share a boundary.
    #[error(
        "key ranges [{first_start}, {first_end}) and [{second_start}, {second_end}) are not adjacent"
    )]
    NotAdjacent {
        /// The first range's start.
        first_start: u32,
        /// The first range's end.
        first_end: u32,
        /// The second range's start.
        second_start: u32,
        /// The second range's end.
        second_end: u32,
    },
}

impl KeyHash {
    /// The hash of a (non-null) key. Records with a null key have no hash.
    pub fn of(key: &[u8]) -> KeyHash {
        KeyHash(to_positive(murmur2(key)))
    }

    /// The hash value, below `2^31`.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for KeyHash {
    type Error = KeyRangeError;

    fn try_from(value: u32) -> Result<KeyHash, KeyRangeError> {
        if value < SPACE_END {
            Ok(KeyHash(value))
        } else {
            Err(KeyRangeError::HashOutsideSpace { value })
        }
    }
}

impl KeyRange {
    /// The whole hash space, `[0, 2^31)`.
    pub const FULL: KeyRange = KeyRange {
        start: 0,
        end: SPACE_END,
    };

    /// The range `[start, end)`.
    ///
    /// # Errors
    ///
    /// [`KeyRangeError::EndBeyondSpace`] if `end` is after `2^31`, and
    /// [`KeyRangeError::Inverted`] if `start` is after `end`.
    pub fn new(start: u32, end: u32) -> Result<KeyRange, KeyRangeError> {
        if end > SPACE_END {
            Err(KeyRangeError::EndBeyondSpace { end })
        } else if start > end {
            Err(KeyRangeError::Inverted { start, end })
        } else {
            Ok(KeyRange { start, end })
        }
    }

    /// The first hash in the range.
    pub fn start(self) -> u32 {
        self.start
    }

    /// The first hash after the range.
    pub fn end(self) -> u32 {
        self.end
    }

    /// True if the range has no hashes (`start == end`).
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// True if `hash` is in the range.
    pub fn contains(self, hash: KeyHash) -> bool {
        self.start <= hash.0 && hash.0 < self.end
    }

    /// Splits into `[start, at)` and `[at, end)`, both non-empty.
    ///
    /// # Errors
    ///
    /// [`KeyRangeError::SplitNotInside`] unless `start < at < end`.
    pub fn split_at(self, at: KeyHash) -> Result<(KeyRange, KeyRange), KeyRangeError> {
        if self.start < at.0 && at.0 < self.end {
            Ok((
                KeyRange {
                    start: self.start,
                    end: at.0,
                },
                KeyRange {
                    start: at.0,
                    end: self.end,
                },
            ))
        } else {
            Err(KeyRangeError::SplitNotInside {
                start: self.start,
                end: self.end,
                at: at.0,
            })
        }
    }

    /// True if one range ends where the other starts.
    pub fn is_adjacent(self, other: KeyRange) -> bool {
        self.end == other.start || other.end == self.start
    }

    /// The union of two adjacent ranges, in either order.
    ///
    /// # Errors
    ///
    /// [`KeyRangeError::NotAdjacent`] if the ranges don't share a boundary.
    pub fn merge(self, other: KeyRange) -> Result<KeyRange, KeyRangeError> {
        if self.end == other.start {
            Ok(KeyRange {
                start: self.start,
                end: other.end,
            })
        } else if other.end == self.start {
            Ok(KeyRange {
                start: other.start,
                end: self.end,
            })
        } else {
            Err(KeyRangeError::NotAdjacent {
                first_start: self.start,
                first_end: self.end,
                second_start: other.start,
                second_end: other.end,
            })
        }
    }
}

impl fmt::Display for KeyHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// `[start, end)`.
impl fmt::Display for KeyRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{prop_assert, prop_assert_eq, proptest};

    use super::{KeyHash, KeyRange, KeyRangeError, SPACE_END};
    use crate::partition_for_key;
    use crate::strategies::{key_hash, key_range};

    fn hash(value: u32) -> KeyHash {
        KeyHash::try_from(value).unwrap()
    }

    #[test]
    fn key_hash_is_kafkas_positive_murmur2() {
        // Kafka 4.4.0 hashes "21" to -973932308; clearing the sign bit gives this.
        assert_eq!(KeyHash::of(b"21").get(), 1_173_551_340);
        // The only 4-byte key hashing to i32::MIN (see the partitioner tests).
        assert_eq!(KeyHash::of(&[167, 214, 80, 108]).get(), 0);
        // So a hash modulo a partition count is Kafka's partition choice.
        let partitions = std::num::NonZeroU32::new(12).unwrap();
        assert_eq!(
            KeyHash::of(b"user-123").get() % 12,
            u32::try_from(partition_for_key(b"user-123", partitions).get()).unwrap()
        );
    }

    #[test]
    fn hashes_and_ranges_stay_inside_the_space() {
        assert_eq!(
            KeyHash::try_from(SPACE_END),
            Err(KeyRangeError::HashOutsideSpace { value: SPACE_END })
        );
        assert_eq!(
            KeyHash::try_from(SPACE_END - 1).map(KeyHash::get),
            Ok(SPACE_END - 1)
        );
        assert_eq!(KeyRange::new(0, SPACE_END), Ok(KeyRange::FULL));
        assert_eq!(
            KeyRange::new(0, SPACE_END + 1),
            Err(KeyRangeError::EndBeyondSpace { end: SPACE_END + 1 })
        );
        assert_eq!(
            KeyRange::new(5, 4),
            Err(KeyRangeError::Inverted { start: 5, end: 4 })
        );
        assert!(KeyRange::new(7, 7).unwrap().is_empty());
        assert_eq!(KeyRange::FULL.to_string(), "[0, 2147483648)");
        assert!(KeyRange::FULL.contains(hash(SPACE_END - 1)));
    }

    #[test]
    fn splits_must_leave_two_non_empty_halves() {
        let range = KeyRange::new(10, 20).unwrap();
        for at in [0, 10, 20, 25] {
            assert_eq!(
                range.split_at(hash(at)),
                Err(KeyRangeError::SplitNotInside {
                    start: 10,
                    end: 20,
                    at
                })
            );
        }
        let (low, high) = range.split_at(hash(15)).unwrap();
        assert_eq!(
            (low.to_string(), high.to_string()),
            ("[10, 15)".to_owned(), "[15, 20)".to_owned())
        );
    }

    #[test]
    fn only_adjacent_ranges_merge() {
        let low = KeyRange::new(0, 10).unwrap();
        let high = KeyRange::new(10, 20).unwrap();
        let far = KeyRange::new(11, 20).unwrap();
        assert!(low.is_adjacent(high) && high.is_adjacent(low));
        assert_eq!(low.merge(high), KeyRange::new(0, 20));
        assert_eq!(high.merge(low), KeyRange::new(0, 20));
        assert!(!low.is_adjacent(far));
        assert_eq!(
            low.merge(far),
            Err(KeyRangeError::NotAdjacent {
                first_start: 0,
                first_end: 10,
                second_start: 11,
                second_end: 20
            })
        );
    }

    proptest! {
        #[test]
        fn split_then_merge_is_the_identity(range in key_range(), at in key_hash()) {
            match range.split_at(at) {
                Ok((low, high)) => {
                    prop_assert!(!low.is_empty() && !high.is_empty());
                    prop_assert!(low.is_adjacent(high));
                    prop_assert_eq!(low.merge(high), Ok(range));
                    prop_assert_eq!(high.merge(low), Ok(range));
                }
                Err(_) => prop_assert!(at.get() <= range.start() || at.get() >= range.end()),
            }
        }

        #[test]
        fn exactly_one_half_contains_each_hash(range in key_range(), at in key_hash(), probe in key_hash()) {
            if let Ok((low, high)) = range.split_at(at) {
                prop_assert_eq!(range.contains(probe), low.contains(probe) || high.contains(probe));
                prop_assert!(!(low.contains(probe) && high.contains(probe)));
            }
        }

        #[test]
        fn merge_fails_exactly_for_non_adjacent_ranges(a in key_range(), b in key_range()) {
            prop_assert_eq!(a.merge(b).is_ok(), a.is_adjacent(b));
        }
    }
}
