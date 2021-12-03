use std::cmp::Ordering;

use chrono::{Datelike, Duration, NaiveDate};

use crate::files::commands::{self, Span, Spanned, Time, Weekday};

use super::{Error, Result};

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
    start: NaiveDate,
    start_time: Option<Time>,
    curr: NaiveDate,
    curr_time: Option<Time>,
}

impl DeltaEval {
    fn new(start: NaiveDate, start_time: Option<Time>) -> Self {
        Self {
            start,
            start_time,
            curr: start,
            curr_time: start_time,
        }
    }

    fn err_step(&self, span: Span) -> Error {
        Error::DeltaInvalidStep {
            span,
            start: self.start,
            start_time: self.start_time,
            prev: self.curr,
            prev_time: self.curr_time,
        }
    }

    fn err_time(&self, span: Span) -> Error {
        Error::DeltaNoTime {
            span,
            start: self.start,
            prev: self.curr,
        }
    }

    fn apply(&mut self, step: &Spanned<DeltaStep>) -> Result<()> {
        match step.value {
            DeltaStep::Year(n) => self.step_year(step.span, n),
            DeltaStep::Month(n) => self.step_month(step.span, n),
            DeltaStep::MonthReverse(n) => self.step_month_reverse(step.span, n),
            DeltaStep::Day(n) => self.step_day(step.span, n),
            DeltaStep::Week(n) => self.step_week(step.span, n),
            DeltaStep::Hour(n) => self.step_hour(step.span, n),
            DeltaStep::Minute(n) => self.step_minute(step.span, n),
            DeltaStep::Weekday(n, wd) => self.step_weekday(step.span, n, wd),
            DeltaStep::Time(time) => self.step_time(step.span, time),
        }
    }

    fn step_year(&mut self, span: Span, amount: i32) -> Result<()> {
        let curr = self.curr;
        match NaiveDate::from_ymd_opt(curr.year() + amount, curr.month(), curr.day()) {
            None => Err(self.err_step(span)),
            Some(next) => {
                self.curr = next;
                Ok(())
            }
        }
    }

    fn step_month(&mut self, span: Span, amount: i32) -> Result<()> {
        let month0 = self.curr.month0() as i32 + amount;
        let year = self.curr.year() + month0.div_euclid(12);
        let month0 = month0.rem_euclid(12) as u32;
        match NaiveDate::from_ymd_opt(year, month0 + 1, self.curr.day()) {
            None => Err(self.err_step(span)),
            Some(next) => {
                self.curr = next;
                Ok(())
            }
        }
    }

    fn step_month_reverse(&mut self, span: Span, amount: i32) -> Result<()> {
        todo!()
    }

    fn step_day(&mut self, span: Span, amount: i32) -> Result<()> {
        let delta = Duration::days(amount.into());
        match self.curr.checked_add_signed(delta) {
            None => Err(self.err_step(span)),
            Some(next) => {
                self.curr = next;
                Ok(())
            }
        }
    }

    fn step_week(&mut self, span: Span, amount: i32) -> Result<()> {
        let delta = Duration::days((7 * amount).into());
        match self.curr.checked_add_signed(delta) {
            None => Err(self.err_step(span)),
            Some(next) => {
                self.curr = next;
                Ok(())
            }
        }
    }

    fn step_hour(&mut self, span: Span, amount: i32) -> Result<()> {
        todo!()
    }

    fn step_minute(&mut self, span: Span, amount: i32) -> Result<()> {
        todo!()
    }

    fn step_weekday(&mut self, span: Span, amount: i32, weekday: Weekday) -> Result<()> {
        todo!()
    }

    fn step_time(&mut self, span: Span, time: Time) -> Result<()> {
        todo!()
    }
}

impl Delta {
    pub fn lower_bound(&self) -> i32 {
        self.steps.iter().map(|step| step.value.lower_bound()).sum()
    }

    pub fn upper_bound(&self) -> i32 {
        self.steps.iter().map(|step| step.value.upper_bound()).sum()
    }

    pub fn apply(&self, start: (NaiveDate, Option<Time>)) -> Result<(NaiveDate, Option<Time>)> {
        let mut eval = DeltaEval::new(start.0, start.1);
        for step in &self.steps {
            eval.apply(step)?;
        }
        Ok((eval.curr, eval.curr_time))
    }
}
