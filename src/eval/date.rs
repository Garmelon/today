use chrono::NaiveDate;

use crate::files::commands::Time;

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
        end: NaiveDate,
        start_time: Time,
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
