//! Sizes in bytes.
//!
//! Configs keep Kafka's plain byte numbers (`segment.bytes=1073741824`), with no unit
//! suffixes, so existing configs and tools work unchanged. Only the printed form adds
//! a readable approximation.

use std::fmt;

/// A non-negative number of bytes.
///
/// Built from Kafka's signed size fields with `TryFrom<i32>` and `TryFrom<i64>`, which
/// reject negative values; arithmetic is checked. `Display` prints the exact bytes and
/// an IEC approximation: `1048576 (1 MiB)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteSize(u64);

/// Why a size was rejected or an operation on one failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ByteSizeError {
    /// Sizes are never negative. Kafka fields that use `-1` as "unlimited" or "unset"
    /// map that sentinel before converting.
    #[error("negative size {value}")]
    Negative {
        /// The rejected value.
        value: i64,
    },
    /// `bytes + added` doesn't fit in `u64`.
    #[error("{bytes} bytes plus {added} overflows")]
    Overflow {
        /// The size.
        bytes: u64,
        /// What was added.
        added: u64,
    },
    /// `bytes - subtracted` is below zero.
    #[error("{bytes} bytes minus {subtracted} is negative")]
    Underflow {
        /// The size.
        bytes: u64,
        /// What was subtracted.
        subtracted: u64,
    },
}

const UNITS: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

impl ByteSize {
    /// Zero bytes.
    pub const ZERO: ByteSize = ByteSize(0);

    /// A size of `bytes`.
    pub const fn from_bytes(bytes: u64) -> ByteSize {
        ByteSize(bytes)
    }

    /// The number of bytes.
    pub const fn as_bytes(self) -> u64 {
        self.0
    }

    /// `self + other`.
    ///
    /// # Errors
    ///
    /// [`ByteSizeError::Overflow`] if the sum doesn't fit in `u64`.
    pub fn checked_add(self, other: ByteSize) -> Result<ByteSize, ByteSizeError> {
        self.0
            .checked_add(other.0)
            .map(ByteSize)
            .ok_or(ByteSizeError::Overflow {
                bytes: self.0,
                added: other.0,
            })
    }

    /// `self - other`.
    ///
    /// # Errors
    ///
    /// [`ByteSizeError::Underflow`] if `other` is larger.
    pub fn checked_sub(self, other: ByteSize) -> Result<ByteSize, ByteSizeError> {
        self.0
            .checked_sub(other.0)
            .map(ByteSize)
            .ok_or(ByteSizeError::Underflow {
                bytes: self.0,
                subtracted: other.0,
            })
    }
}

impl TryFrom<i64> for ByteSize {
    type Error = ByteSizeError;

    fn try_from(value: i64) -> Result<ByteSize, ByteSizeError> {
        u64::try_from(value)
            .map(ByteSize)
            .map_err(|_| ByteSizeError::Negative { value })
    }
}

impl TryFrom<i32> for ByteSize {
    type Error = ByteSizeError;

    fn try_from(value: i32) -> Result<ByteSize, ByteSizeError> {
        ByteSize::try_from(i64::from(value))
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The largest unit that is at most the size, then tenths of it, rounded half
        // up. Rounding can reach 1024 of a unit (1023.96 KiB), which is then shown as
        // 1 of the next unit.
        let bytes = u128::from(self.0);
        let mut exponent = 0;
        while exponent + 1 < UNITS.len() && bytes >= 1_u128 << (10 * (exponent + 1)) {
            exponent += 1;
        }
        let unit = 1_u128 << (10 * exponent);
        let mut tenths = (bytes * 10 + unit / 2) / unit;
        if tenths >= 10_240 && exponent + 1 < UNITS.len() {
            exponent += 1;
            tenths = (bytes * 10 + (unit << 10) / 2) / (unit << 10);
        }
        write!(f, "{} ({}", self.0, tenths / 10)?;
        if !tenths.is_multiple_of(10) {
            write!(f, ".{}", tenths % 10)?;
        }
        write!(f, " {})", UNITS[exponent])
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};

    use super::{ByteSize, ByteSizeError};

    #[test]
    fn display_shows_exact_bytes_and_an_iec_approximation() {
        let cases = [
            (0, "0 (0 B)"),
            (512, "512 (512 B)"),
            (1023, "1023 (1023 B)"),
            (1024, "1024 (1 KiB)"),
            (1536, "1536 (1.5 KiB)"),
            (1_048_575, "1048575 (1 MiB)"),
            (1_048_576, "1048576 (1 MiB)"),
            (1_073_741_824, "1073741824 (1 GiB)"),
            (1_610_612_736, "1610612736 (1.5 GiB)"),
            (u64::MAX, "18446744073709551615 (16 EiB)"),
        ];
        for (bytes, shown) in cases {
            assert_eq!(ByteSize::from_bytes(bytes).to_string(), shown);
        }
    }

    #[test]
    fn arithmetic_is_checked() {
        let max = ByteSize::from_bytes(u64::MAX);
        assert_eq!(
            max.checked_add(ByteSize::from_bytes(1)),
            Err(ByteSizeError::Overflow {
                bytes: u64::MAX,
                added: 1
            })
        );
        assert_eq!(
            ByteSize::ZERO.checked_sub(ByteSize::from_bytes(1)),
            Err(ByteSizeError::Underflow {
                bytes: 0,
                subtracted: 1
            })
        );
        assert_eq!(
            ByteSize::from_bytes(5).checked_sub(ByteSize::from_bytes(5)),
            Ok(ByteSize::ZERO)
        );
    }

    proptest! {
        #[test]
        fn conversions_reject_exactly_negative_values(wide in any::<i64>(), narrow in any::<i32>()) {
            match ByteSize::try_from(wide) {
                Ok(size) => prop_assert_eq!(i128::from(size.as_bytes()), i128::from(wide)),
                Err(error) => {
                    prop_assert!(wide < 0);
                    prop_assert_eq!(error, ByteSizeError::Negative { value: wide });
                }
            }
            prop_assert_eq!(ByteSize::try_from(narrow).is_ok(), narrow >= 0);
        }

        #[test]
        fn arithmetic_matches_wide_integers(a in any::<u64>(), b in any::<u64>()) {
            let (x, y) = (ByteSize::from_bytes(a), ByteSize::from_bytes(b));
            let sum = u128::from(a) + u128::from(b);
            prop_assert_eq!(x.checked_add(y).ok().map(|s| u128::from(s.as_bytes())), (sum <= u128::from(u64::MAX)).then_some(sum));
            prop_assert_eq!(x.checked_sub(y).ok().map(ByteSize::as_bytes), a.checked_sub(b));
        }

        #[test]
        fn display_starts_with_the_exact_bytes(bytes in any::<u64>()) {
            let shown = ByteSize::from_bytes(bytes).to_string();
            let exact = format!("{bytes} (");
            prop_assert!(shown.starts_with(&exact), "{}", shown);
        }
    }
}
