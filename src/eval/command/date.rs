use chrono::NaiveDate;

use crate::eval::date::Dates;
use crate::eval::DateRange;
use crate::files::commands::{self, Command, Spanned, Time};

use super::super::command::CommandState;
use super::super::delta::{Delta, DeltaStep};
use super::super::{Error, Result};

pub struct DateSpec {
    pub start: NaiveDate,
    pub start_delta: Delta,
    pub start_time: Option<Time>,
    pub end_delta: Delta,
    pub repeat: Option<Delta>,
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
        if let Some(time) = spec.end_time {
            end_delta
                .steps
                .push(Spanned::new(time.span, DeltaStep::Time(time.value)));
        }

        let repeat: Option<Delta> = spec.repeat.as_ref().map(|repeat| (&repeat.delta).into());
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
    fn start_and_range(&self, s: &CommandState<'_>) -> Option<(NaiveDate, DateRange)> {
        let (start, range) = match s.command.command {
            Command::Task(_) => {
                let last_done = s.last_done();
                let start = last_done
                    .filter(|_| self.start_at_done)
                    .unwrap_or(self.start);
                let range_from = last_done.map(|date| date.succ()).unwrap_or(self.start);
                let range = s
                    .range
                    .expand_by(&self.end_delta)
                    .move_by(&self.start_delta)
                    .with_from(range_from)?;
                (start, range)
            }
            Command::Note(_) => {
                let start = self.start;
                let range = s
                    .range
                    .expand_by(&self.end_delta)
                    .move_by(&self.start_delta);
                (start, range)
            }
        };
        let range = s.limit_from_until(range)?;
        Some((start, range))
    }

    fn step(from: NaiveDate, repeat: &Delta) -> Result<NaiveDate> {
        let to = repeat.apply_date(from)?;
        if to > from {
            Ok(to)
        } else {
            Err(Error::RepeatDidNotMoveForwards {
                span: todo!(),
                from,
                to,
            })
        }
    }

    fn dates(&self, start: NaiveDate) -> Result<Dates> {
        let root = self.start_delta.apply_date(start)?;
        Ok(if let Some(root_time) = self.start_time {
            let (other, other_time) = self.end_delta.apply_date_time(root, root_time)?;
            Dates::new_with_time(root, root_time, other, other_time)
        } else {
            let other = self.end_delta.apply_date(root)?;
            Dates::new(root, other)
        })
    }
}

impl<'a> CommandState<'a> {
    pub fn eval_date_spec(&mut self, spec: DateSpec) -> Result<()> {
        if let Some(repeat) = &spec.repeat {
            if let Some((mut start, range)) = spec.start_and_range(self) {
                while start < range.from() {
                    start = DateSpec::step(start, repeat)?;
                }
                while start <= range.until() {
                    let dates = spec.dates(start)?;
                    self.add(self.kind(), Some(dates));
                    start = DateSpec::step(start, repeat)?;
                }
            }
        } else {
            let dates = spec.dates(spec.start)?;
            self.add(self.kind(), Some(dates));
        }
        Ok(())
    }
}
