use std::cmp::Ordering;

use chrono::{Datelike, Duration, NaiveDate};

use crate::files::commands;
use crate::files::primitives::{Span, Spanned, Time, Weekday};

use super::{util, Error, Result};

/// Like [`commands::DeltaStep`] but includes a new constructor,
/// [`DeltaStep::Time`].
#[derive(Debug, Clone, Copy)]
pub enum DeltaStep {
    Year(i32),
    Month(i32),
    MonthReverse(i32),
    Day(i32),
    Week(i32),
    Hour(i32),
    Minute(i32),
    Weekday(i32, Weekday),
    /// Set the time to the next occurrence of the specified time. Useful to
    /// unify the end delta and end time for different specs.
    Time(Time),
}

impl From<commands::DeltaStep> for DeltaStep {
    fn from(step: commands::DeltaStep) -> Self {
        match step {
            commands::DeltaStep::Year(n) => Self::Year(n),
            commands::DeltaStep::Month(n) => Self::Month(n),
            commands::DeltaStep::MonthReverse(n) => Self::MonthReverse(n),
            commands::DeltaStep::Day(n) => Self::Day(n),
            commands::DeltaStep::Week(n) => Self::Week(n),
            commands::DeltaStep::Hour(n) => Self::Hour(n),
            commands::DeltaStep::Minute(n) => Self::Minute(n),
            commands::DeltaStep::Weekday(n, wd) => Self::Weekday(n, wd),
        }
    }
}

impl DeltaStep {
    /// A lower bound on days
    fn lower_bound(&self) -> i32 {
        match self {
            DeltaStep::Year(n) => {
                if *n < 0 {
                    *n * 366
                } else {
                    *n * 365
                }
            }
            DeltaStep::Month(n) | DeltaStep::MonthReverse(n) => {
                if *n < 0 {
                    *n * 31
                } else {
                    *n * 28
                }
            }
            DeltaStep::Day(n) => *n,
            DeltaStep::Week(n) => *n * 7,
            DeltaStep::Hour(n) => {
                if *n < 0 {
                    *n / 24 + (*n % 24).signum()
                } else {
                    *n / 24
                }
            }
            DeltaStep::Minute(n) => {
                if *n < 0 {
                    *n / (24 * 60) + (*n % (24 * 60)).signum()
                } else {
                    *n / (24 * 60)
                }
            }
            DeltaStep::Weekday(n, _) => match n.cmp(&0) {
                Ordering::Less => *n * 7 - 1,
                Ordering::Equal => 0,
                Ordering::Greater => *n * 7 - 7,
            },
            DeltaStep::Time(_) => 0,
        }
    }

    /// An upper bound on days
    fn upper_bound(&self) -> i32 {
        match self {
            DeltaStep::Year(n) => {
                if *n > 0 {
                    *n * 366
                } else {
                    *n * 365
                }
            }
            DeltaStep::Month(n) | DeltaStep::MonthReverse(n) => {
                if *n > 0 {
                    *n * 31
                } else {
                    *n * 28
                }
            }
            DeltaStep::Day(n) => *n,
            DeltaStep::Week(n) => *n * 7,
            DeltaStep::Hour(n) => {
                if *n > 0 {
                    *n / 24 + (*n % 24).signum()
                } else {
                    *n / 24
                }
            }
            DeltaStep::Minute(n) => {
                if *n > 0 {
                    *n / (24 * 60) + (*n % (24 * 60)).signum()
                } else {
                    *n / (24 * 60)
                }
            }
            DeltaStep::Weekday(n, _) => match n.cmp(&0) {
                Ordering::Less => *n * 7 - 7,
                Ordering::Equal => 0,
                Ordering::Greater => *n * 7 - 1,
            },
            DeltaStep::Time(_) => 1,
        }
    }
}

#[derive(Debug, Default)]
pub struct Delta {
    pub steps: Vec<Spanned<DeltaStep>>,
}

impl From<&commands::Delta> for Delta {
    fn from(delta: &commands::Delta) -> Self {
        Self {
            steps: delta
                .0
                .iter()
                .map(|step| Spanned::new(step.span, step.value.into()))
                .collect(),
        }
    }
}

struct DeltaEval {
    index: usize,
    start: NaiveDate,
    start_time: Option<Time>,
    curr: NaiveDate,
    curr_time: Option<Time>,
}

impl DeltaEval {
    fn new(index: usize, start: NaiveDate, start_time: Option<Time>) -> Self {
        Self {
            index,
            start,
            start_time,
            curr: start,
            curr_time: start_time,
        }
    }

    fn err_step(&self, span: Span) -> Error {
        Error::DeltaInvalidStep {
            index: self.index,
            span,
            start: self.start,
            start_time: self.start_time,
            prev: self.curr,
            prev_time: self.curr_time,
        }
    }

    fn err_time(&self, span: Span) -> Error {
        Error::DeltaNoTime {
            index: self.index,
            span,
            start: self.start,
            prev: self.curr,
        }
    }

    fn apply(&mut self, step: &Spanned<DeltaStep>) -> Result<()> {
        match step.value {
            DeltaStep::Year(n) => self.step_year(step.span, n)?,
            DeltaStep::Month(n) => self.step_month(step.span, n)?,
            DeltaStep::MonthReverse(n) => self.step_month_reverse(step.span, n)?,
            DeltaStep::Day(n) => self.step_day(n),
            DeltaStep::Week(n) => self.step_week(n),
            DeltaStep::Hour(n) => self.step_hour(step.span, n)?,
            DeltaStep::Minute(n) => self.step_minute(step.span, n)?,
            DeltaStep::Weekday(n, wd) => self.step_weekday(n, wd),
            DeltaStep::Time(time) => self.step_time(step.span, time)?,
        }
        Ok(())
    }

    fn step_year(&mut self, span: Span, amount: i32) -> Result<()> {
        let year = self.curr.year() + amount;
        match NaiveDate::from_ymd_opt(year, self.curr.month(), self.curr.day()) {
            None => Err(self.err_step(span)),
            Some(next) => {
                self.curr = next;
                Ok(())
            }
        }
    }

    fn step_month(&mut self, span: Span, amount: i32) -> Result<()> {
        let (year, month) = util::add_months(self.curr.year(), self.curr.month(), amount);
        match NaiveDate::from_ymd_opt(year, month, self.curr.day()) {
            None => Err(self.err_step(span)),
            Some(next) => {
                self.curr = next;
                Ok(())
            }
        }
    }

    fn step_month_reverse(&mut self, span: Span, amount: i32) -> Result<()> {
        // Calculate offset from the last day of the month
        let month_length = util::month_length(self.curr.year(), self.curr.month()) as i32;
        let end_offset = self.curr.day() as i32 - month_length;

        let (year, month) = util::add_months(self.curr.year(), self.curr.month(), amount);

        // Calculate day based on the offset from earlier
        let month_length = util::month_length(year, month) as i32;
        let day = if end_offset + month_length > 0 {
            (end_offset + month_length) as u32
        } else {
            return Err(self.err_step(span));
        };

        match NaiveDate::from_ymd_opt(year, month, day) {
            None => Err(self.err_step(span)),
            Some(next) => {
                self.curr = next;
                Ok(())
            }
        }
    }

    fn step_day(&mut self, amount: i32) {
        let delta = Duration::days(amount.into());
        self.curr += delta;
    }

    fn step_week(&mut self, amount: i32) {
        let delta = Duration::days((7 * amount).into());
        self.curr += delta;
    }

    fn step_hour(&mut self, span: Span, amount: i32) -> Result<()> {
        let time = match self.curr_time {
            Some(time) => time,
            None => return Err(self.err_time(span)),
        };

        let (days, time) = time.add_hours(amount.into());
        self.curr += Duration::days(days);
        self.curr_time = Some(time);
        Ok(())
    }

    fn step_minute(&mut self, span: Span, amount: i32) -> Result<()> {
        let time = match self.curr_time {
            Some(time) => time,
            None => return Err(self.err_time(span)),
        };

        let (days, time) = time.add_minutes(amount.into());
        self.curr += Duration::days(days);
        self.curr_time = Some(time);
        Ok(())
    }

    fn step_weekday(&mut self, amount: i32, weekday: Weekday) {
        let curr_wd: Weekday = self.curr.weekday().into();
        #[allow(clippy::comparison_chain)] // The if looks better in this case
        if amount > 0 {
            let rest: i32 = curr_wd.until(weekday).into();
            let days = rest + (amount - 1) * 7;
            self.curr += Duration::days(days.into());
        } else if amount < 0 {
            let rest: i32 = weekday.until(curr_wd).into();
            let days = rest + (amount - 1) * 7;
            self.curr -= Duration::days(days.into());
        }
    }

    fn step_time(&mut self, span: Span, time: Time) -> Result<()> {
        let curr_time = match self.curr_time {
            Some(time) => time,
            None => return Err(self.err_time(span)),
        };

        if time < curr_time {
            self.curr = self.curr.succ();
        }
        self.curr_time = Some(time);
        Ok(())
    }
}

impl Delta {
    pub fn lower_bound(&self) -> i32 {
        self.steps.iter().map(|step| step.value.lower_bound()).sum()
    }

    pub fn upper_bound(&self) -> i32 {
        self.steps.iter().map(|step| step.value.upper_bound()).sum()
    }

    fn apply(
        &self,
        index: usize,
        start: (NaiveDate, Option<Time>),
    ) -> Result<(NaiveDate, Option<Time>)> {
        let mut eval = DeltaEval::new(index, start.0, start.1);
        for step in &self.steps {
            eval.apply(step)?;
        }
        Ok((eval.curr, eval.curr_time))
    }

    pub fn apply_date(&self, index: usize, date: NaiveDate) -> Result<NaiveDate> {
        Ok(self.apply(index, (date, None))?.0)
    }

    pub fn apply_date_time(
        &self,
        index: usize,
        date: NaiveDate,
        time: Time,
    ) -> Result<(NaiveDate, Time)> {
        let (date, time) = self.apply(index, (date, Some(time)))?;
        Ok((date, time.expect("time was not preserved")))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::files::primitives::{Span, Spanned, Time};

    use super::super::Result;
    use super::{Delta, DeltaStep as Step};

    const SPAN: Span = Span { start: 12, end: 34 };

    fn delta(step: Step) -> Delta {
        Delta {
            steps: vec![Spanned::new(SPAN, step)],
        }
    }

    fn apply_d(step: Step, from: (i32, u32, u32)) -> Result<NaiveDate> {
        delta(step).apply_date(0, NaiveDate::from_ymd(from.0, from.1, from.2))
    }

    fn test_d(step: Step, from: (i32, u32, u32), expected: (i32, u32, u32)) {
        assert_eq!(
            apply_d(step, from).unwrap(),
            NaiveDate::from_ymd(expected.0, expected.1, expected.2)
        );
    }

    fn apply_dt(step: Step, from: (i32, u32, u32, u32, u32)) -> Result<(NaiveDate, Time)> {
        delta(step).apply_date_time(
            0,
            NaiveDate::from_ymd(from.0, from.1, from.2),
            Time::new(from.3, from.4),
        )
    }

    #[allow(clippy::too_many_arguments)] // This is just for writing tests
    fn test_dt(step: Step, from: (i32, u32, u32, u32, u32), expected: (i32, u32, u32, u32, u32)) {
        assert_eq!(
            apply_dt(step, from).unwrap(),
            (
                NaiveDate::from_ymd(expected.0, expected.1, expected.2),
                Time::new(expected.3, expected.4)
            )
        );
    }

    #[test]
    fn delta_year() {
        test_d(Step::Year(-10000), (2021, 7, 3), (-7979, 7, 3));
        test_d(Step::Year(-100), (2021, 7, 3), (1921, 7, 3));
        test_d(Step::Year(-10), (2021, 7, 3), (2011, 7, 3));
        test_d(Step::Year(-2), (2021, 7, 3), (2019, 7, 3));
        test_d(Step::Year(-1), (2021, 7, 3), (2020, 7, 3));
        test_d(Step::Year(0), (2021, 7, 3), (2021, 7, 3));
        test_d(Step::Year(1), (2021, 7, 3), (2022, 7, 3));
        test_d(Step::Year(2), (2021, 7, 3), (2023, 7, 3));
        test_d(Step::Year(10), (2021, 7, 3), (2031, 7, 3));
        test_d(Step::Year(100), (2021, 7, 3), (2121, 7, 3));
        test_d(Step::Year(10000), (2021, 7, 3), (12021, 7, 3));

        // Leap year shenanigans
        test_d(Step::Year(4), (2020, 2, 29), (2024, 2, 29));
        test_d(Step::Year(2), (2020, 2, 28), (2022, 2, 28));
        test_d(Step::Year(2), (2020, 3, 1), (2022, 3, 1));
        test_d(Step::Year(-2), (2022, 2, 28), (2020, 2, 28));
        test_d(Step::Year(-2), (2022, 3, 1), (2020, 3, 1));
        assert!(apply_d(Step::Year(1), (2020, 2, 29)).is_err());

        // Doesn't touch time
        test_dt(Step::Year(1), (2021, 7, 3, 12, 34), (2022, 7, 3, 12, 34));
    }

    #[test]
    fn delta_month() {
        test_d(Step::Month(-48), (2021, 7, 3), (2017, 7, 3));
        test_d(Step::Month(-12), (2021, 7, 3), (2020, 7, 3));
        test_d(Step::Month(-2), (2021, 7, 3), (2021, 5, 3));
        test_d(Step::Month(-1), (2021, 7, 3), (2021, 6, 3));
        test_d(Step::Month(0), (2021, 7, 3), (2021, 7, 3));
        test_d(Step::Month(1), (2021, 7, 3), (2021, 8, 3));
        test_d(Step::Month(2), (2021, 7, 3), (2021, 9, 3));
        test_d(Step::Month(12), (2021, 7, 3), (2022, 7, 3));

        // At end of months
        test_d(Step::Month(2), (2021, 1, 31), (2021, 3, 31));
        test_d(Step::Month(3), (2021, 1, 30), (2021, 4, 30));
        assert!(apply_d(Step::Month(1), (2021, 1, 31)).is_err());

        // Leap year shenanigans
        test_d(Step::Month(1), (2020, 1, 29), (2020, 2, 29));
        assert!(apply_d(Step::Month(1), (2021, 1, 29)).is_err());

        // Doesn't touch time
        test_dt(Step::Month(1), (2021, 7, 3, 12, 34), (2021, 8, 3, 12, 34));
    }

    #[test]
    fn delta_month_reverse() {
        test_d(Step::MonthReverse(-48), (2021, 7, 31), (2017, 7, 31));
        test_d(Step::MonthReverse(-12), (2021, 7, 31), (2020, 7, 31));
        test_d(Step::MonthReverse(-2), (2021, 7, 31), (2021, 5, 31));
        test_d(Step::MonthReverse(-1), (2021, 7, 31), (2021, 6, 30));
        test_d(Step::MonthReverse(0), (2021, 7, 31), (2021, 7, 31));
        test_d(Step::MonthReverse(1), (2021, 7, 31), (2021, 8, 31));
        test_d(Step::MonthReverse(2), (2021, 7, 31), (2021, 9, 30));
        test_d(Step::MonthReverse(12), (2021, 7, 31), (2022, 7, 31));

        // At start of months
        test_d(Step::MonthReverse(2), (2021, 1, 1), (2021, 3, 1));
        test_d(Step::MonthReverse(3), (2021, 1, 2), (2021, 4, 1));
        assert!(apply_d(Step::MonthReverse(1), (2021, 1, 1)).is_err());

        // Leap year shenanigans
        test_d(Step::MonthReverse(1), (2020, 1, 30), (2020, 2, 28));
        test_d(Step::MonthReverse(-1), (2020, 2, 28), (2020, 1, 30));
        test_d(Step::MonthReverse(1), (2021, 1, 31), (2021, 2, 28));
        test_d(Step::MonthReverse(-1), (2021, 2, 28), (2021, 1, 31));

        // Doesn't touch time
        test_dt(
            Step::MonthReverse(1),
            (2021, 7, 3, 12, 34),
            (2021, 8, 3, 12, 34),
        );
    }

    #[test]
    fn delta_day() {
        test_d(Step::Day(-365), (2021, 7, 3), (2020, 7, 3));
        test_d(Step::Day(-30), (2021, 7, 3), (2021, 6, 3));
        test_d(Step::Day(-2), (2021, 7, 3), (2021, 7, 1));
        test_d(Step::Day(-1), (2021, 7, 3), (2021, 7, 2));
        test_d(Step::Day(0), (2021, 7, 3), (2021, 7, 3));
        test_d(Step::Day(1), (2021, 7, 3), (2021, 7, 4));
        test_d(Step::Day(2), (2021, 7, 3), (2021, 7, 5));
        test_d(Step::Day(31), (2021, 7, 3), (2021, 8, 3));
        test_d(Step::Day(365), (2021, 7, 3), (2022, 7, 3));

        // Leap year shenanigans
        test_d(Step::Day(1), (2020, 2, 28), (2020, 2, 29));
        test_d(Step::Day(1), (2020, 2, 29), (2020, 3, 1));
        test_d(Step::Day(1), (2021, 2, 28), (2021, 3, 1));
        test_d(Step::Day(-1), (2020, 3, 1), (2020, 2, 29));
        test_d(Step::Day(-1), (2020, 2, 29), (2020, 2, 28));
        test_d(Step::Day(-1), (2021, 3, 1), (2021, 2, 28));

        // Doesn't touch time
        test_dt(Step::Day(1), (2021, 7, 3, 12, 34), (2021, 7, 4, 12, 34));
    }

    #[test]
    fn delta_week() {
        test_d(Step::Week(-2), (2021, 7, 3), (2021, 6, 19));
        test_d(Step::Week(-1), (2021, 7, 3), (2021, 6, 26));
        test_d(Step::Week(0), (2021, 7, 3), (2021, 7, 3));
        test_d(Step::Week(1), (2021, 7, 3), (2021, 7, 10));
        test_d(Step::Week(2), (2021, 7, 3), (2021, 7, 17));

        // Leap year shenanigans
        test_d(Step::Week(1), (2020, 2, 25), (2020, 3, 3));
        test_d(Step::Week(1), (2021, 2, 25), (2021, 3, 4));

        // Doesn't touch time
        test_dt(Step::Week(1), (2021, 7, 3, 12, 34), (2021, 7, 10, 12, 34));
    }

    #[test]
    fn delta_hour() {
        test_dt(Step::Hour(-24), (2021, 7, 3, 12, 34), (2021, 7, 2, 12, 34));
        test_dt(Step::Hour(-12), (2021, 7, 3, 12, 34), (2021, 7, 3, 0, 34));
        test_dt(Step::Hour(-2), (2021, 7, 3, 12, 34), (2021, 7, 3, 10, 34));
        test_dt(Step::Hour(-1), (2021, 7, 3, 12, 34), (2021, 7, 3, 11, 34));
        test_dt(Step::Hour(0), (2021, 7, 3, 12, 34), (2021, 7, 3, 12, 34));
        test_dt(Step::Hour(1), (2021, 7, 3, 12, 34), (2021, 7, 3, 13, 34));
        test_dt(Step::Hour(2), (2021, 7, 3, 12, 34), (2021, 7, 3, 14, 34));
        test_dt(Step::Hour(12), (2021, 7, 3, 12, 34), (2021, 7, 4, 0, 34));
        test_dt(Step::Hour(24), (2021, 7, 3, 12, 34), (2021, 7, 4, 12, 34));

        // 24:00 != 00:00
        test_dt(Step::Hour(1), (2021, 7, 3, 23, 0), (2021, 7, 3, 24, 0));
        test_dt(Step::Hour(2), (2021, 7, 3, 23, 0), (2021, 7, 4, 1, 0));
        test_dt(Step::Hour(-1), (2021, 7, 3, 1, 0), (2021, 7, 3, 0, 0));
        test_dt(Step::Hour(-2), (2021, 7, 3, 1, 0), (2021, 7, 2, 23, 0));

        // Requires time
        assert!(apply_d(Step::Hour(0), (2021, 7, 3)).is_err());
    }

    #[test]
    fn delta_minute() {
        test_dt(
            Step::Minute(-60 * 24),
            (2021, 7, 3, 12, 34),
            (2021, 7, 2, 12, 34),
        );
        test_dt(
            Step::Minute(-60),
            (2021, 7, 3, 12, 34),
            (2021, 7, 3, 11, 34),
        );
        test_dt(Step::Minute(-2), (2021, 7, 3, 12, 34), (2021, 7, 3, 12, 32));
        test_dt(Step::Minute(-1), (2021, 7, 3, 12, 34), (2021, 7, 3, 12, 33));
        test_dt(Step::Minute(0), (2021, 7, 3, 12, 34), (2021, 7, 3, 12, 34));
        test_dt(Step::Minute(1), (2021, 7, 3, 12, 34), (2021, 7, 3, 12, 35));
        test_dt(Step::Minute(2), (2021, 7, 3, 12, 34), (2021, 7, 3, 12, 36));
        test_dt(Step::Minute(60), (2021, 7, 3, 12, 34), (2021, 7, 3, 13, 34));
        test_dt(
            Step::Minute(60 * 24),
            (2021, 7, 3, 12, 34),
            (2021, 7, 4, 12, 34),
        );

        // 24:00 != 00:00
        test_dt(Step::Minute(1), (2021, 7, 3, 23, 59), (2021, 7, 3, 24, 0));
        test_dt(Step::Minute(2), (2021, 7, 3, 23, 59), (2021, 7, 4, 0, 1));
        test_dt(Step::Minute(-1), (2021, 7, 3, 0, 1), (2021, 7, 3, 0, 0));
        test_dt(Step::Minute(-2), (2021, 7, 3, 0, 1), (2021, 7, 2, 23, 59));

        // Requires time
        assert!(apply_d(Step::Minute(0), (2021, 7, 3)).is_err());
    }

    #[test]
    fn delta_time() {
        test_dt(
            Step::Time(Time::new(12, 33)),
            (2021, 7, 3, 12, 34),
            (2021, 7, 4, 12, 33),
        );
        test_dt(
            Step::Time(Time::new(12, 34)),
            (2021, 7, 3, 12, 34),
            (2021, 7, 3, 12, 34),
        );
        test_dt(
            Step::Time(Time::new(12, 35)),
            (2021, 7, 3, 12, 34),
            (2021, 7, 3, 12, 35),
        );

        // 24:00 != 00:00
        test_dt(
            Step::Time(Time::new(24, 0)),
            (2021, 7, 3, 12, 0),
            (2021, 7, 3, 24, 0),
        );
        test_dt(
            Step::Time(Time::new(0, 0)),
            (2021, 7, 3, 12, 0),
            (2021, 7, 4, 0, 0),
        );

        // Requires time
        assert!(apply_d(Step::Time(Time::new(12, 34)), (2021, 7, 3)).is_err());
    }
}
