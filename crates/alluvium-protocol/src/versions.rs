//! [`Versions`]: a range of a message's versions, as Kafka's schemas write them.

/// The versions `lowest` to `highest`, inclusive, or none. An open-ended range such as
/// `3+` ends at `i16::MAX`.
// Parity: generator/src/main/java/org/apache/kafka/message/Versions.java@4.4.0 Versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Versions {
    lowest: i16,
    highest: i16,
}

impl Versions {
    /// No versions.
    pub const NONE: Versions = Versions {
        lowest: 0,
        highest: -1,
    };

    /// The versions `lowest` to `highest`, or [`Versions::NONE`] if `lowest` is above
    /// `highest`.
    pub const fn new(lowest: i16, highest: i16) -> Versions {
        if lowest > highest {
            Versions::NONE
        } else {
            Versions { lowest, highest }
        }
    }

    /// Whether `version` is in the range.
    pub const fn contains(self, version: i16) -> bool {
        self.lowest <= version && version <= self.highest
    }

    /// The lowest version, or `None` for no versions.
    pub const fn lowest(self) -> Option<i16> {
        if self.lowest > self.highest {
            None
        } else {
            Some(self.lowest)
        }
    }

    /// The versions in both ranges.
    // Parity: generator/src/main/java/org/apache/kafka/message/Versions.java@4.4.0 Versions.intersect
    #[must_use]
    pub const fn intersect(self, other: Versions) -> Versions {
        let lowest = if self.lowest > other.lowest {
            self.lowest
        } else {
            other.lowest
        };
        let highest = if self.highest < other.highest {
            self.highest
        } else {
            other.highest
        };
        Versions::new(lowest, highest)
    }

    /// The highest version, or `None` for no versions.
    pub const fn highest(self) -> Option<i16> {
        if self.lowest > self.highest {
            None
        } else {
            Some(self.highest)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Versions;

    #[test]
    fn ranges_hold_their_ends_and_none_holds_nothing() {
        let range = Versions::new(3, 13);
        assert!(range.contains(3) && range.contains(13));
        assert!(!range.contains(2) && !range.contains(14));
        assert_eq!((range.lowest(), range.highest()), (Some(3), Some(13)));
        assert_eq!(Versions::new(5, 4), Versions::NONE);
        assert!(!Versions::NONE.contains(0) && !Versions::NONE.contains(-1));
        assert_eq!(
            (Versions::NONE.lowest(), Versions::NONE.highest()),
            (None, None)
        );
        assert!(Versions::new(9, i16::MAX).contains(i16::MAX));
    }
}
