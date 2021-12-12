use std::cmp;
use std::ops::RangeInclusive;

use chrono::{Datelike, Duration, NaiveDate};

use super::delta::Delta;

#[derive(Debug, Clone, Copy)]
pub struct DateRange {
    from: NaiveDate,
    until: NaiveDate,
}

impl DateRange {
    pub fn new(from: NaiveDate, until: NaiveDate) -> Option<Self> {
        if from <= until {
            Some(Self { from, until })
        } else {
            None
        }
    }

    pub fn with_from(&self, from: NaiveDate) -> Option<Self> {
        Self::new(from, self.until)
    }

    pub fn with_until(&self, until: NaiveDate) -> Option<Self> {
        Self::new(self.from, until)
    }

    pub fn containing(&self, date: NaiveDate) -> Self {
        if date < self.from {
            Self {
                from: date,
                until: self.until,
            }
        } else if self.until < date {
            Self {
                from: self.from,
                until: date,
            }
        } else {
            *self
        }
    }

    pub fn contains(&self, date: NaiveDate) -> bool {
        self.from <= date && date <= self.until
    }

    pub fn from(&self) -> NaiveDate {
        self.from
    }

    pub fn until(&self) -> NaiveDate {
        self.until
    }

    pub fn days(&self) -> impl Iterator<Item = NaiveDate> {
        (self.from.num_days_from_ce()..=self.until.num_days_from_ce())
            .map(|n| NaiveDate::from_num_days_from_ce(n))
    }

    pub fn years(&self) -> RangeInclusive<i32> {
        self.from.year()..=self.until.year()
    }

    /// Expand the range so that it contains at least all dates from which the
    /// original range could be reached using `delta`. This new range will
    /// always contain the old range.
    pub fn expand_by(&self, delta: &Delta) -> Self {
        let expand_lower = cmp::min(-delta.upper_bound(), 0);
        let expand_upper = cmp::max(-delta.lower_bound(), 0);
        Self::new(
            self.from + Duration::days(expand_lower.into()),
            self.until + Duration::days(expand_upper.into()),
        )
        // The range is never shrunk, so the new range should always be valid.
        .expect("expanded range shrunk")
    }

    /// Return a new range that contains at least all dates from which the
    /// original range could be reached using `delta`. This new range might not
    /// contain the old range.
    pub fn move_by(&self, delta: &Delta) -> Self {
        let move_lower = -delta.upper_bound();
        let move_upper = -delta.lower_bound();
        Self::new(
            self.from + Duration::days(move_lower.into()),
            self.until + Duration::days(move_upper.into()),
        )
        // The delta's upper bound is greater or equal than its lower bound, so
        // the range should never become smaller. It can only move and expand.
        .expect("moved range shrunk")
    }
}
