use std::fmt;

use chrono::{Duration, NaiveDate};

use crate::files::commands::DoneDate;
use crate::files::primitives::Time;

#[derive(Debug, Clone, Copy)]
struct Times {
    root: Time,
    other: Time,
}

#[derive(Debug, Clone, Copy)]
pub struct Dates {
    root: NaiveDate,
    other: NaiveDate,
    times: Option<Times>,
}

impl fmt::Display for Dates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let done_date: DoneDate = (*self).into();
        write!(f, "{}", done_date)
    }
}

impl Dates {
    pub fn new(root: NaiveDate, other: NaiveDate) -> Self {
        Self {
            root,
            other,
            times: None,
        }
    }

    pub fn new_with_time(
        root: NaiveDate,
        root_time: Time,
        other: NaiveDate,
        other_time: Time,
    ) -> Self {
        Self {
            root,
            other,
            times: Some(Times {
                root: root_time,
                other: other_time,
            }),
        }
    }

    pub fn root(self) -> NaiveDate {
        self.root
    }

    pub fn root_with_time(self) -> (NaiveDate, Option<Time>) {
        (self.root, self.times.map(|t| t.root))
    }

    pub fn other_with_time(self) -> (NaiveDate, Option<Time>) {
        (self.other, self.times.map(|t| t.other))
    }

    pub fn dates(self) -> (NaiveDate, NaiveDate) {
        (self.root, self.other)
    }

    pub fn times(self) -> Option<(Time, Time)> {
        self.times.map(|times| (times.root, times.other))
    }

    /// Flip `root` and `other`.
    fn flip(self) -> Self {
        Self {
            root: self.other,
            other: self.root,
            times: self.times.map(|times| Times {
                root: times.other,
                other: times.root,
            }),
        }
    }

    /// Return a new [`Dates`] where `root` is the earlier and `other` the later
    /// date.
    pub fn sorted(self) -> Self {
        match self.times {
            _ if self.root < self.other => self,
            None if self.root <= self.other => self,
            Some(times) if self.root <= self.other && times.root <= times.other => self,
            _ => self.flip(),
        }
    }

    pub fn point_in_time(self) -> Option<(NaiveDate, Option<Time>)> {
        let done_date: DoneDate = self.into();
        match done_date {
            DoneDate::Date { root } => Some((root, None)),
            DoneDate::DateTime { root, root_time } => Some((root, Some(root_time))),
            _ => None,
        }
    }

    pub fn move_by(&self, delta: Duration) -> Self {
        let mut result = *self;

        // Modify dates
        result.root += delta;
        result.other += delta;

        // Modify times if necessary (may further modify dates)
        const MINUTES_PER_DAY: i64 = 24 * 60;
        let minutes = delta.num_minutes() % MINUTES_PER_DAY; // May be negative
        if let Some(times) = self.times {
            let (root_days, root) = times.root.add_minutes(minutes);
            let (other_days, other) = times.other.add_minutes(minutes);
            result.root += Duration::days(root_days);
            result.other += Duration::days(other_days);
            result.times = Some(Times { root, other });
        }

        result
    }
}

impl From<DoneDate> for Dates {
    fn from(date: DoneDate) -> Self {
        match date {
            DoneDate::Date { root } => Self::new(root, root),
            DoneDate::DateTime { root, root_time } => {
                Self::new_with_time(root, root_time, root, root_time)
            }
            DoneDate::DateToDate { root, other } => Self::new(root, other),
            DoneDate::DateTimeToTime {
                root,
                root_time,
                other_time,
            } => Self::new_with_time(root, root_time, root, other_time),
            DoneDate::DateTimeToDateTime {
                root,
                root_time,
                other,
                other_time,
            } => Self::new_with_time(root, root_time, other, other_time),
        }
    }
}

impl From<Dates> for DoneDate {
    fn from(dates: Dates) -> Self {
        let (root, other) = dates.dates();
        match dates.times() {
            Some((root_time, other_time)) => Self::DateTimeToDateTime {
                root,
                root_time,
                other,
                other_time,
            },
            None => Self::DateToDate { root, other },
        }
        .simplified()
    }
}
