use std::ops::RangeInclusive;

use chrono::{Datelike, NaiveDate};

#[derive(Debug, Clone, Copy)]
pub struct DateRange {
    from: NaiveDate,
    until: NaiveDate,
}

impl DateRange {
    pub fn new(from: NaiveDate, until: NaiveDate) -> Self {
        assert!(from <= until);
        Self { from, until }
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

    pub fn years(&self) -> RangeInclusive<i32> {
        self.from.year()..=self.until.year()
    }
}
