// Portions of this file are ported from Apache Kafka 4.4.0 and modified by Alluvium;
// each is marked `// Ported from:`. Apache Kafka is Copyright The Apache Software
// Foundation and licensed under the Apache License 2.0 (LICENSES/Apache-2.0.txt).
// See NOTICE.

//! Version ranges in schemas: `N`, `N-M`, `N+` (up to `i16::MAX`) and `none`.

use std::fmt;

/// An inclusive range of API versions. Empty when `lowest > highest`; [`Versions::NONE`]
/// is the empty range Kafka writes as `none`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Versions {
    lowest: i16,
    highest: i16,
}

// Parity: generator/src/main/java/org/apache/kafka/message/Versions.java@4.4.0 Versions
impl Versions {
    /// Every version: `0+`.
    #[cfg(test)]
    pub(crate) const ALL: Versions = Versions {
        lowest: 0,
        highest: i16::MAX,
    };

    /// No version: `none`.
    pub(crate) const NONE: Versions = Versions {
        lowest: 0,
        highest: -1,
    };

    /// The range `lowest..=highest`. Bounds must not be negative.
    pub(crate) fn new(lowest: i16, highest: i16) -> Result<Versions, String> {
        if lowest < 0 || highest < 0 {
            return Err(format!("Invalid version range {lowest} to {highest}"));
        }
        Ok(Versions { lowest, highest })
    }

    /// Parses `input`, trimmed: `none`, `N`, `N-M` or `N+`. A missing or blank input
    /// gives `default`, which may itself be missing.
    pub(crate) fn parse(
        input: Option<&str>,
        default: Option<Versions>,
    ) -> Result<Option<Versions>, String> {
        let Some(input) = input.map(str::trim).filter(|input| !input.is_empty()) else {
            return Ok(default);
        };
        if input == "none" {
            return Ok(Some(Versions::NONE));
        }
        let versions = if let Some(lowest) = input.strip_suffix('+') {
            Versions::new(parse_short(lowest)?, i16::MAX)?
        } else if let Some((lowest, highest)) = input.split_once('-') {
            Versions::new(parse_short(lowest)?, parse_short(highest)?)?
        } else {
            let version = parse_short(input)?;
            Versions::new(version, version)?
        };
        Ok(Some(versions))
    }

    pub(crate) fn lowest(self) -> i16 {
        self.lowest
    }

    pub(crate) fn highest(self) -> i16 {
        self.highest
    }

    pub(crate) fn is_empty(self) -> bool {
        self.lowest > self.highest
    }

    /// The versions in both ranges.
    pub(crate) fn intersect(self, other: Versions) -> Versions {
        let lowest = self.lowest.max(other.lowest);
        let highest = self.highest.min(other.highest);
        if lowest > highest {
            Versions::NONE
        } else {
            Versions { lowest, highest }
        }
    }

    /// The versions in `self` but not in `other`, or `None` when that is two ranges.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "codegen's version conditions (PROTO-F05) use it")
    )]
    pub(crate) fn subtract(self, other: Versions) -> Option<Versions> {
        if other.lowest <= self.lowest {
            if other.highest >= self.highest {
                Some(Versions::NONE)
            } else if other.highest < self.lowest {
                Some(self)
            } else {
                // other.highest < self.highest <= i16::MAX, so this can't overflow.
                Some(Versions {
                    lowest: other.highest + 1,
                    highest: self.highest,
                })
            }
        } else if other.highest >= self.highest {
            // other.lowest > self.lowest >= 0, so this can't go below -1.
            let highest = other.lowest - 1;
            if highest < self.highest {
                Some(Versions {
                    lowest: self.lowest,
                    highest,
                })
            } else {
                Some(self)
            }
        } else {
            None
        }
    }

    /// Whether `version` is in the range.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "codegen's version conditions (PROTO-F05) use it")
    )]
    pub(crate) fn contains(self, version: i16) -> bool {
        self.lowest <= version && version <= self.highest
    }

    /// Whether every version of `other` is in this range. Every range contains `none`.
    pub(crate) fn contains_all(self, other: Versions) -> bool {
        other.is_empty() || (self.lowest <= other.lowest && other.highest <= self.highest)
    }
}

/// Java's `Short.parseShort`: an optional sign, then decimal digits, within `i16`.
fn parse_short(text: &str) -> Result<i16, String> {
    text.parse()
        .map_err(|_| format!("For input string: \"{text}\""))
}

impl fmt::Display for Versions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.write_str("none")
        } else if self.lowest == self.highest {
            write!(f, "{}", self.lowest)
        } else if self.highest == i16::MAX {
            write!(f, "{}+", self.lowest)
        } else {
            write!(f, "{}-{}", self.lowest, self.highest)
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, prop_assert, prop_assert_eq, prop_compose, proptest};

    use super::Versions;

    fn range(lowest: i16, highest: i16) -> Versions {
        Versions::new(lowest, highest).unwrap()
    }

    fn parse(text: &str) -> Versions {
        Versions::parse(Some(text), None).unwrap().unwrap()
    }

    // Ported from: generator/src/test/java/org/apache/kafka/message/VersionsTest.java@4.4.0.

    #[test]
    fn versions_parse() {
        assert_eq!(parse(" none "), Versions::NONE);
        assert_eq!(
            Versions::parse(None, Some(Versions::NONE)),
            Ok(Some(Versions::NONE))
        );
        assert_eq!(
            Versions::parse(Some(" "), Some(Versions::ALL)),
            Ok(Some(Versions::ALL))
        );
        assert_eq!(
            Versions::parse(Some(""), Some(Versions::ALL)),
            Ok(Some(Versions::ALL))
        );
        assert_eq!(parse(" 4-5 "), range(4, 5));
    }

    #[test]
    fn round_trips() {
        for (versions, text) in [
            (Versions::ALL, "0+"),
            (range(1, 3), "1-3"),
            (range(2, 2), "2"),
            (range(3, i16::MAX), "3+"),
            (Versions::NONE, "none"),
        ] {
            assert_eq!(versions.to_string(), text);
            assert_eq!(parse(&versions.to_string()), versions);
        }
    }

    #[test]
    fn intersections() {
        assert_eq!(range(1, 3).intersect(range(2, 4)), range(2, 3));
        assert_eq!(range(0, i16::MAX).intersect(range(3, 3)), range(3, 3));
        assert_eq!(range(9, i16::MAX).intersect(range(2, 8)), Versions::NONE);
        assert_eq!(Versions::NONE.intersect(Versions::NONE), Versions::NONE);
    }

    #[test]
    fn contains() {
        assert!(range(2, 3).contains(3));
        assert!(range(2, 3).contains(2));
        assert!(!range(0, 1).contains(2));
        assert!(range(0, i16::MAX).contains(100));
        assert!(!range(2, i16::MAX).contains(0));
        assert!(range(2, 3).contains_all(range(2, 3)));
        assert!(range(2, 3).contains_all(range(2, 2)));
        assert!(!range(2, 3).contains_all(range(2, 4)));
        assert!(range(2, 3).contains_all(Versions::NONE));
        assert!(Versions::ALL.contains_all(range(1, 2)));
    }

    #[test]
    fn subtract() {
        assert_eq!(
            Versions::NONE.subtract(Versions::NONE),
            Some(Versions::NONE)
        );
        assert_eq!(range(0, 0).subtract(Versions::NONE), Some(range(0, 0)));
        assert_eq!(range(1, 2).subtract(range(2, 2)), Some(range(1, 1)));
        assert_eq!(range(1, 2).subtract(range(1, 1)), Some(range(2, 2)));
        assert_eq!(range(0, i16::MAX).subtract(range(1, 100)), None);
        assert_eq!(range(1, 10).subtract(range(1, 9)), Some(range(10, 10)));
        assert_eq!(range(1, 10).subtract(range(2, 10)), Some(range(1, 1)));
        assert_eq!(
            range(2, i16::MAX).subtract(range(5, i16::MAX)),
            Some(range(2, 4))
        );
        assert_eq!(
            range(0, i16::MAX).subtract(range(0, 4)),
            Some(range(5, i16::MAX))
        );
    }

    // End of the ported tests.

    #[test]
    fn negative_and_malformed_bounds_are_rejected() {
        for text in ["-1", "-1+", "3--4", "1-x", "x", "40000", "1 - 2"] {
            assert!(Versions::parse(Some(text), None).is_err(), "{text}");
        }
        assert_eq!(Versions::parse(Some("2-1"), None), Ok(Some(range(2, 1))));
        assert!(range(2, 1).is_empty());
    }

    prop_compose! {
        fn versions()(lowest in 0..=i16::MAX, highest in -1..=i16::MAX) -> Versions {
            if highest < 0 { Versions::NONE } else { range(lowest, highest) }
        }
    }

    proptest! {
        #[test]
        fn intersect_is_commutative(a in versions(), b in versions()) {
            let (left, right) = (a.intersect(b), b.intersect(a));
            prop_assert!(left == right || (left.is_empty() && right.is_empty()));
        }

        #[test]
        fn subtract_stays_inside_the_range(a in versions(), b in versions(), version in any::<i16>()) {
            if let Some(difference) = a.subtract(b) {
                prop_assert!(a.contains_all(difference));
                prop_assert_eq!(
                    difference.contains(version),
                    a.contains(version) && !b.contains(version)
                );
            }
        }

        #[test]
        fn contains_agrees_with_the_bounds(a in versions(), version in any::<i16>()) {
            prop_assert_eq!(
                a.contains(version),
                a.lowest() <= version && version <= a.highest()
            );
        }
    }
}
