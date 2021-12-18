use chrono::NaiveDate;

use crate::files::commands::{self, Command};
use crate::files::primitives::{Spanned, Time};

use super::super::command::CommandState;
use super::super::date::Dates;
use super::super::delta::{Delta, DeltaStep};
use super::super::{DateRange, Error, Result};

pub struct DateSpec {
    pub start: NaiveDate,
    pub start_delta: Delta,
    pub start_time: Option<Time>,
    pub end_delta: Delta,
    pub repeat: Option<Spanned<Delta>>,
    pub start_at_done: bool,
}

impl From<&commands::DateSpec> for DateSpec {
    fn from(spec: &commands::DateSpec) -> Self {
        let start_delta: Delta = spec
            .start_delta
            .as_ref()
            .map(|delta| delta.into())
            .unwrap_or_default();

        let mut end_delta: Delta = spec
            .end_delta
            .as_ref()
            .map(|delta| delta.into())
            .unwrap_or_default();
        if let Some(date) = spec.end {
            // Strictly speaking, this could be out of range, but that would
            // require a delta of about 6 million years. I'm not too worried...
            let days = (date.value - spec.start).num_days() as i32;
            end_delta
                .steps
                .insert(0, Spanned::new(date.span, DeltaStep::Day(days)));
        }
        if let Some(time) = spec.end_time {
            end_delta
                .steps
                .push(Spanned::new(time.span, DeltaStep::Time(time.value)));
        }

        let repeat: Option<Spanned<Delta>> = spec
            .repeat
            .as_ref()
            .map(|repeat| Spanned::new(repeat.delta.span, (&repeat.delta.value).into()));
        let start_at_done = spec
            .repeat
            .as_ref()
            .map(|repeat| repeat.start_at_done)
            .unwrap_or(false);

        Self {
            start: spec.start,
            start_delta,
            start_time: spec.start_time,
            end_delta,
            repeat,
            start_at_done,
        }
    }
}

impl DateSpec {
    /// Find the start date and range for the date spec calculation.
    ///
    /// Returns a tuple `(start, skip, range)` where `skip` is `true` if the
    /// `start` date itself should be skipped (and thus not result in an entry).
    /// This may be necessary if [`Self::start_at_done`] is set.
    fn start_and_range(&self, s: &CommandState<'_>) -> Option<(NaiveDate, bool, DateRange)> {
        let (start, skip, range) = match s.command.command {
            Command::Task(_) => {
                let (start, skip) = s
                    .last_done_completion()
                    .map(|start| (start, true))
                    .filter(|_| self.start_at_done)
                    .unwrap_or((self.start, false));
                let range_from = s
                    .last_done_root()
                    .map(|date| date.succ())
                    .unwrap_or(self.start);
                let range = s
                    .range
                    .expand_by(&self.end_delta)
                    .move_by(&self.start_delta)
                    .with_from(range_from)?;
                (start, skip, range)
            }
            Command::Note(_) => {
                let start = self.start;
                let range = s
                    .range
                    .expand_by(&self.end_delta)
                    .move_by(&self.start_delta);
                (start, false, range)
            }
        };
        let range = s.limit_from_until(range)?;
        Some((start, skip, range))
    }

    fn step(index: usize, from: NaiveDate, repeat: &Spanned<Delta>) -> Result<NaiveDate> {
        let to = repeat.value.apply_date(index, from)?;
        if to > from {
            Ok(to)
        } else {
            Err(Error::RepeatDidNotMoveForwards {
                index,
                span: repeat.span,
                from,
                to,
            })
        }
    }

    fn dates(&self, index: usize, start: NaiveDate) -> Result<Dates> {
        let root = self.start_delta.apply_date(index, start)?;
        Ok(if let Some(root_time) = self.start_time {
            let (other, other_time) = self.end_delta.apply_date_time(index, root, root_time)?;
            Dates::new_with_time(root, root_time, other, other_time)
        } else {
            let other = self.end_delta.apply_date(index, root)?;
            Dates::new(root, other)
        })
    }
}

impl<'a> CommandState<'a> {
    pub fn eval_date_spec(&mut self, spec: DateSpec) -> Result<()> {
        let index = self.command.source.file();
        if let Some(repeat) = &spec.repeat {
            if let Some((mut start, skip, range)) = spec.start_and_range(self) {
                if skip {
                    start = DateSpec::step(index, start, repeat)?;
                }
                while start < range.from() {
                    start = DateSpec::step(index, start, repeat)?;
                }
                while start <= range.until() {
                    let dates = spec.dates(index, start)?;
                    self.add(self.kind(), Some(dates));
                    start = DateSpec::step(index, start, repeat)?;
                }
            }
        } else {
            let dates = spec.dates(index, spec.start)?;
            self.add(self.kind(), Some(dates));
        }
        Ok(())
    }
}
