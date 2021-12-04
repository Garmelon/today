use chrono::NaiveDate;

use crate::files::commands::{DoneDate, Time};

#[derive(Debug, Clone, Copy)]
pub struct Times {
    start: Time,
    end: Time,
}

impl Times {
    pub fn start(&self) -> Time {
        self.start
    }

    pub fn end(&self) -> Time {
        self.end
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Dates {
    start: NaiveDate,
    end: NaiveDate,
    times: Option<Times>,
}

impl Dates {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        assert!(start <= end);
        Self {
            start,
            end,
            times: None,
        }
    }

    pub fn new_with_time(
        start: NaiveDate,
        start_time: Time,
        end: NaiveDate,
        end_time: Time,
    ) -> Self {
        assert!(start <= end);
        if start == end {
            assert!(start_time <= end_time);
        }
        Self {
            start,
            end,
            times: Some(Times {
                start: start_time,
                end: end_time,
            }),
        }
    }

    pub fn start(&self) -> NaiveDate {
        self.start
    }

    pub fn end(&self) -> NaiveDate {
        self.start
    }

    pub fn times(&self) -> Option<Times> {
        self.times
    }

    pub fn start_time(&self) -> Option<Time> {
        self.times.as_ref().map(Times::start)
    }

    pub fn end_time(&self) -> Option<Time> {
        self.times.as_ref().map(Times::end)
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
