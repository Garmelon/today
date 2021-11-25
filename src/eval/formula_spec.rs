use crate::files::commands::{self, DoneDate, Expr, Time, Var};
use crate::files::Source;

use super::delta::{Delta, DeltaStep};
use super::{Entry, Eval, Result};

pub struct FormulaSpec {
    // TODO Implement more efficient exprs and expr evaluation
    pub start: Expr,
    pub start_delta: Delta,
    pub start_time: Option<Time>,
    pub end_delta: Delta,
}

impl From<&commands::FormulaSpec> for FormulaSpec {
    fn from(spec: &commands::FormulaSpec) -> Self {
        let start: Expr = spec.start.as_ref().cloned().unwrap_or(Expr::Lit(1));

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

        Self {
            start,
            start_delta,
            start_time: spec.start_time,
            end_delta,
        }
    }
}

impl From<&commands::WeekdaySpec> for FormulaSpec {
    fn from(spec: &commands::WeekdaySpec) -> Self {
        let start = Expr::Eq(
            Box::new(Expr::Var(Var::Weekday)),
            Box::new(Expr::Var(spec.start.into())),
        );

        let mut end_delta = Delta::default();
        if let Some(wd) = spec.end {
            end_delta.steps.push(DeltaStep::Weekday(1, wd));
        }
        if let Some(delta) = &spec.end_delta {
            for step in &delta.0 {
                end_delta.steps.push((*step).into());
            }
        }
        if let Some(time) = spec.end_time {
            end_delta.steps.push(DeltaStep::Time(time));
        }

        Self {
            start,
            start_delta: Default::default(),
            start_time: spec.start_time,
            end_delta,
        }
    }
}

impl Eval {
    pub fn eval_formula_spec(
        &mut self,
        spec: FormulaSpec,
        new_entry: impl Fn(Source, Option<DoneDate>) -> Entry,
    ) -> Result<()> {
        todo!()
    }
}
