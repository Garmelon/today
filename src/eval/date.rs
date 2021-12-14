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
        let (start, end) = self.start_end();
        match self.start_end_time() {
            Some((start_time, end_time)) if start == end && start_time == end_time => {
                write!(f, "{} {}", start, start_time)
            }
            Some((start_time, end_time)) if start == end => {
                write!(f, "{} {} -- {}", start, start_time, end_time)
            }
            Some((start_time, end_time)) => {
                write!(f, "{} {} -- {} {}", start, start_time, end, end_time)
            }
            None if start == end => write!(f, "{}", start),
            None => write!(f, "{} -- {}", start, end),
        }
    }
}

impl From<Dates> for DoneDate {
    fn from(dates: Dates) -> Self {
        match dates.times {
            Some(times) if dates.root == dates.other && times.root == times.other => {
                DoneDate::DateWithTime {
                    root: dates.root,
                    root_time: times.root,
                }
            }
            Some(times) => DoneDate::DateToDateWithTime {
                root: dates.root,
                root_time: times.root,
                other: dates.other,
                other_time: times.other,
            },
            None if dates.root == dates.other => DoneDate::Date { root: dates.root },
            None => DoneDate::DateToDate {
                root: dates.root,
                other: dates.other,
            },
        }
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

    pub fn root(&self) -> NaiveDate {
        self.root
    }

    pub fn other(&self) -> NaiveDate {
        self.other
    }

    pub fn root_time(&self) -> Option<Time> {
        self.times.map(|times| times.root)
    }

    pub fn other_time(&self) -> Option<Time> {
        self.times.map(|times| times.other)
    }

    pub fn start_end(&self) -> (NaiveDate, NaiveDate) {
        if self.root <= self.other {
            (self.root, self.other)
        } else {
            (self.other, self.root)
        }
    }

    pub fn start(&self) -> NaiveDate {
        self.start_end().0
    }

    pub fn end(&self) -> NaiveDate {
        self.start_end().1
    }

    pub fn start_end_time(&self) -> Option<(Time, Time)> {
        if let Some(times) = self.times {
            if self.root < self.other || (self.root == self.other && times.root <= times.other) {
                Some((times.root, times.other))
            } else {
                Some((times.other, times.root))
            }
        } else {
            None
        }
    }

    pub fn start_time(&self) -> Option<Time> {
        self.start_end_time().map(|times| times.0)
    }

    pub fn end_time(&self) -> Option<Time> {
        self.start_end_time().map(|times| times.1)
    }

    pub fn point_in_time(&self) -> Option<(NaiveDate, Option<Time>)> {
        if self.root != self.other {
            return None;
        }
        match self.times {
            Some(times) if times.root == times.other => Some((self.root, Some(times.root))),
            Some(_) => None,
            None => Some((self.root, None)),
        }
    }

    pub fn move_by(&self, delta: Duration) -> Self {
        Self {
            root: self.root + delta,
            other: self.other + delta,
            times: self.times,
        }
    }
}

impl From<DoneDate> for Dates {
    fn from(date: DoneDate) -> Self {
        match date {
            DoneDate::Date { root } => Self::new(root, root),
            DoneDate::DateWithTime { root, root_time } => {
                Self::new_with_time(root, root_time, root, root_time)
            }
            DoneDate::DateToDate { root, other } => {
                if root <= other {
                    Self::new(root, other)
                } else {
                    Self::new(other, root)
                }
            }
            DoneDate::DateToDateWithTime {
                root,
                root_time,
                other,
                other_time,
            } => {
                if root < other || (root == other && root_time <= other_time) {
                    Self::new_with_time(root, root_time, other, other_time)
                } else {
                    Self::new_with_time(other, other_time, root, root_time)
                }
            }
        }
    }
}
