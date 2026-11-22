//! Log offsets: positions in a partition, half-open ranges of them, and the offset
//! specifications a `ListOffsets` request can ask for.

use std::fmt;

/// A position in a partition's log. Non-negative; `-1` (for example
/// `OffsetFetchResponse.INVALID_OFFSET`, no committed offset) reads as `None`.
///
/// Arithmetic is checked: a result outside `[0, i64::MAX]` is an error, never a wrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Offset(i64);

/// A half-open range of offsets, `[start, end)`.
///
/// Half-open because the empty range is `start == end`, `len` is `end - start`, and the
/// range up to the end of the log needs no `end + 1`, which could overflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OffsetRange {
    start: Offset,
    end: Offset,
}

/// What a `ListOffsets` request asks for: a timestamp, or one of Kafka's special values.
///
/// Which values each API version may send is checked by the request handler (API
/// epic), not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OffsetSpec {
    /// `-1`: the log end offset.
    Latest,
    /// `-2`: the log start offset.
    Earliest,
    /// `-3`: the offset of the record with the largest timestamp.
    MaxTimestamp,
    /// `-4`: the first offset still on local storage (tiered storage).
    EarliestLocal,
    /// `-5`: the last offset already in remote storage (tiered storage).
    LatestTiered,
    /// `-6`: the first offset not yet uploaded to remote storage (tiered storage).
    EarliestPendingUpload,
    /// A timestamp in milliseconds, `>= 0`: the first offset at or after it.
    Timestamp(i64),
}

/// Why an offset operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum OffsetError {
    /// Offsets are non-negative; `-1` is accepted only by nullable fields.
    #[error("negative offset {value}")]
    Negative {
        /// The rejected value.
        value: i64,
    },
    /// The result of `offset + delta` falls outside `[0, i64::MAX]`.
    #[error("offset {offset} plus {delta} is out of range")]
    Overflow {
        /// The starting offset.
        offset: i64,
        /// The signed change that was applied. `i128` so that subtracting `i64::MIN`
        /// is reported exactly.
        delta: i128,
    },
    /// A range whose start is after its end.
    #[error("offset range start {start} is after its end {end}")]
    InvertedRange {
        /// The requested start.
        start: i64,
        /// The requested end.
        end: i64,
    },
    /// A split point outside the range it should split.
    #[error("split point {at} is outside the offset range [{start}, {end})")]
    SplitOutsideRange {
        /// The range's start.
        start: i64,
        /// The range's end.
        end: i64,
        /// The rejected split point.
        at: i64,
    },
    /// A negative `ListOffsets` timestamp that is not one of Kafka's special values.
    #[error("unknown ListOffsets timestamp {value}")]
    UnknownSpec {
        /// The rejected value.
        value: i64,
    },
}

impl Offset {
    /// Offset 0, the first offset of every partition.
    pub const ZERO: Offset = Offset(0);

    /// The offset as Kafka writes it.
    pub fn get(self) -> i64 {
        self.0
    }

    /// Reads a nullable offset field, where `-1` means "no offset".
    ///
    /// # Errors
    ///
    /// [`OffsetError::Negative`] for a negative value other than `-1`.
    // Parity: clients/src/main/java/org/apache/kafka/common/requests/OffsetFetchResponse.java@4.4.0 INVALID_OFFSET
    pub fn from_wire_nullable(value: i64) -> Result<Option<Offset>, OffsetError> {
        if value == -1 {
            Ok(None)
        } else {
            Offset::try_from(value).map(Some)
        }
    }

    /// `self + delta`.
    ///
    /// # Errors
    ///
    /// [`OffsetError::Overflow`] if the result is outside `[0, i64::MAX]`.
    pub fn checked_add(self, delta: i64) -> Result<Offset, OffsetError> {
        self.0
            .checked_add(delta)
            .filter(|value| *value >= 0)
            .map(Offset)
            .ok_or(OffsetError::Overflow {
                offset: self.0,
                delta: delta.into(),
            })
    }

    /// `self - delta`.
    ///
    /// # Errors
    ///
    /// [`OffsetError::Overflow`] if the result is outside `[0, i64::MAX]`, reporting the
    /// change as `-delta`.
    pub fn checked_sub(self, delta: i64) -> Result<Offset, OffsetError> {
        self.0
            .checked_sub(delta)
            .filter(|value| *value >= 0)
            .map(Offset)
            .ok_or(OffsetError::Overflow {
                offset: self.0,
                delta: -i128::from(delta),
            })
    }

    /// The next offset.
    ///
    /// # Errors
    ///
    /// [`OffsetError::Overflow`] at `i64::MAX`.
    pub fn next(self) -> Result<Offset, OffsetError> {
        self.checked_add(1)
    }
}

impl TryFrom<i64> for Offset {
    type Error = OffsetError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < 0 {
            Err(OffsetError::Negative { value })
        } else {
            Ok(Offset(value))
        }
    }
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl OffsetRange {
    /// The range `[start, end)`.
    ///
    /// # Errors
    ///
    /// [`OffsetError::InvertedRange`] if `start > end`.
    pub fn new(start: Offset, end: Offset) -> Result<OffsetRange, OffsetError> {
        if start > end {
            return Err(OffsetError::InvertedRange {
                start: start.0,
                end: end.0,
            });
        }
        Ok(OffsetRange { start, end })
    }

    /// The first offset in the range (or where it would be, if empty).
    pub fn start(self) -> Offset {
        self.start
    }

    /// The first offset after the range.
    pub fn end(self) -> Offset {
        self.end
    }

    /// True if `offset` is in `[start, end)`.
    pub fn contains(self, offset: Offset) -> bool {
        self.start <= offset && offset < self.end
    }

    /// The number of offsets in the range.
    pub fn len(self) -> u64 {
        // `end >= start >= 0`, so the difference is non-negative and fits in `u64`.
        self.end.0.abs_diff(self.start.0)
    }

    /// True if the range holds no offsets.
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Splits into `[start, at)` and `[at, end)`: adjacent ranges whose union is `self`.
    ///
    /// # Errors
    ///
    /// [`OffsetError::SplitOutsideRange`] unless `start <= at <= end`.
    pub fn split_at(self, at: Offset) -> Result<(OffsetRange, OffsetRange), OffsetError> {
        if at < self.start || at > self.end {
            return Err(OffsetError::SplitOutsideRange {
                start: self.start.0,
                end: self.end.0,
                at: at.0,
            });
        }
        Ok((
            OffsetRange {
                start: self.start,
                end: at,
            },
            OffsetRange {
                start: at,
                end: self.end,
            },
        ))
    }

    /// The offsets in both ranges, or `None` if they share none.
    pub fn intersect(self, other: OffsetRange) -> Option<OffsetRange> {
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        (start < end).then_some(OffsetRange { start, end })
    }
}

impl fmt::Display for OffsetRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

// Parity: clients/src/main/java/org/apache/kafka/common/requests/ListOffsetsRequest.java@4.4.0
// LATEST_TIMESTAMP (-1), EARLIEST_TIMESTAMP (-2), MAX_TIMESTAMP (-3), EARLIEST_LOCAL_TIMESTAMP (-4),
// LATEST_TIERED_TIMESTAMP (-5), EARLIEST_PENDING_UPLOAD_TIMESTAMP (-6)
impl OffsetSpec {
    /// Reads the `timestamp` field of a `ListOffsets` partition.
    ///
    /// # Errors
    ///
    /// [`OffsetError::UnknownSpec`] for a negative value that is not a special value.
    pub fn from_wire(value: i64) -> Result<OffsetSpec, OffsetError> {
        match value {
            -1 => Ok(OffsetSpec::Latest),
            -2 => Ok(OffsetSpec::Earliest),
            -3 => Ok(OffsetSpec::MaxTimestamp),
            -4 => Ok(OffsetSpec::EarliestLocal),
            -5 => Ok(OffsetSpec::LatestTiered),
            -6 => Ok(OffsetSpec::EarliestPendingUpload),
            _ if value >= 0 => Ok(OffsetSpec::Timestamp(value)),
            _ => Err(OffsetError::UnknownSpec { value }),
        }
    }

    /// The wire value.
    pub fn to_wire(self) -> i64 {
        match self {
            OffsetSpec::Latest => -1,
            OffsetSpec::Earliest => -2,
            OffsetSpec::MaxTimestamp => -3,
            OffsetSpec::EarliestLocal => -4,
            OffsetSpec::LatestTiered => -5,
            OffsetSpec::EarliestPendingUpload => -6,
            OffsetSpec::Timestamp(ms) => ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};

    use super::{Offset, OffsetError, OffsetRange, OffsetSpec};
    use crate::strategies;

    fn offset(value: i64) -> Offset {
        Offset::try_from(value).unwrap_or_else(|error| panic!("{error}"))
    }

    fn range(start: i64, end: i64) -> OffsetRange {
        OffsetRange::new(offset(start), offset(end)).unwrap_or_else(|error| panic!("{error}"))
    }

    /// The reference result of `offset + delta`, in `i128`, if it is a valid offset.
    fn reference(offset: i64, delta: i128) -> Option<i64> {
        let sum = i128::from(offset) + delta;
        (0..=i128::from(i64::MAX))
            .contains(&sum)
            .then(|| i64::try_from(sum).unwrap_or_default())
    }

    proptest! {
        #[test]
        fn checked_arithmetic_agrees_with_i128(o in strategies::offset(), delta in any::<i64>()) {
            prop_assert_eq!(o.checked_add(delta).ok().map(Offset::get), reference(o.get(), i128::from(delta)));
            prop_assert_eq!(o.checked_sub(delta).ok().map(Offset::get), reference(o.get(), -i128::from(delta)));
        }

        #[test]
        fn range_operations_match_a_naive_model(a in 0_i64..40, b in 0_i64..40, c in 0_i64..40, d in 0_i64..40, at in 0_i64..40, probe in 0_i64..40) {
            let (start, end) = (a.min(b), a.max(b));
            let r = range(start, end);
            let model: Vec<i64> = (start..end).collect();
            prop_assert_eq!(r.contains(offset(probe)), model.contains(&probe));
            prop_assert_eq!(r.len(), u64::try_from(model.len()).unwrap_or_default());
            prop_assert_eq!(r.is_empty(), model.is_empty());
            let other = range(c.min(d), c.max(d));
            let both: Vec<i64> = model.iter().copied().filter(|x| other.contains(offset(*x))).collect();
            match r.intersect(other) {
                Some(i) => prop_assert_eq!((i.start().get()..i.end().get()).collect::<Vec<_>>(), both),
                None => prop_assert!(both.is_empty()),
            }
            match r.split_at(offset(at)) {
                Ok((left, right)) => {
                    prop_assert!((start..=end).contains(&at));
                    prop_assert_eq!(left.end(), right.start());
                    prop_assert_eq!((left.start(), right.end()), (r.start(), r.end()));
                    prop_assert_eq!(left.len() + right.len(), r.len());
                }
                Err(error) => {
                    prop_assert!(!(start..=end).contains(&at));
                    prop_assert_eq!(error, OffsetError::SplitOutsideRange { start, end, at });
                }
            }
        }

        #[test]
        fn offset_specs_round_trip_and_reject_unknown_negatives(value in any::<i64>()) {
            match OffsetSpec::from_wire(value) {
                Ok(spec) => {
                    prop_assert!(value >= -6);
                    prop_assert_eq!(spec.to_wire(), value);
                }
                Err(error) => {
                    prop_assert!(value < -6);
                    prop_assert_eq!(error, OffsetError::UnknownSpec { value });
                }
            }
        }
    }

    #[test]
    fn next_stops_at_the_top_and_nullable_minus_one_is_none() {
        assert_eq!(offset(41).next(), Ok(offset(42)));
        assert_eq!(
            offset(i64::MAX).next(),
            Err(OffsetError::Overflow {
                offset: i64::MAX,
                delta: 1
            })
        );
        assert_eq!(Offset::from_wire_nullable(-1), Ok(None));
        assert_eq!(
            Offset::from_wire_nullable(-2),
            Err(OffsetError::Negative { value: -2 })
        );
        assert_eq!(offset(7).to_string(), "7");
    }

    #[test]
    fn subtraction_errors_report_the_exact_change() {
        assert_eq!(
            offset(0).checked_sub(i64::MIN),
            Err(OffsetError::Overflow {
                offset: 0,
                delta: 1_i128 << 63
            })
        );
        assert_eq!(
            offset(3).checked_sub(4).map_err(|error| error.to_string()),
            Err("offset 3 plus -4 is out of range".to_owned())
        );
    }

    #[test]
    fn ranges_reject_inversion_and_display_half_open() {
        assert_eq!(
            OffsetRange::new(offset(5), offset(4)),
            Err(OffsetError::InvertedRange { start: 5, end: 4 })
        );
        assert_eq!(range(3, 9).to_string(), "[3, 9)");
        assert!(range(3, 3).is_empty());
    }

    #[test]
    fn every_special_offset_spec_matches_kafka() {
        // Parity: clients/src/main/java/org/apache/kafka/common/requests/ListOffsetsRequest.java@4.4.0
        let specs = [
            (OffsetSpec::Latest, -1),
            (OffsetSpec::Earliest, -2),
            (OffsetSpec::MaxTimestamp, -3),
            (OffsetSpec::EarliestLocal, -4),
            (OffsetSpec::LatestTiered, -5),
            (OffsetSpec::EarliestPendingUpload, -6),
            (OffsetSpec::Timestamp(0), 0),
        ];
        for (spec, wire) in specs {
            assert_eq!(OffsetSpec::from_wire(wire), Ok(spec));
            assert_eq!(spec.to_wire(), wire);
        }
    }
}
