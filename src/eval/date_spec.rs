use chrono::NaiveDate;

use crate::files::commands::{self, DoneDate, Time};
use crate::files::Source;

use super::delta::{Delta, DeltaStep};
use super::{Entry, Eval, Result};

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
            end_delta.steps.push(DeltaStep::Time(time));
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

impl Eval {
    pub fn eval_date_spec(
        &mut self,
        spec: DateSpec,
        last_done: Option<NaiveDate>,
        new_entry: impl Fn(Source, Option<DoneDate>) -> Entry,
    ) -> Result<()> {
        todo!()
    }
}
