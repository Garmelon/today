use std::result;

use chrono::NaiveDate;

use crate::files::commands::{Span, Time};

#[derive(Debug)]
pub enum Error {
    DeltaInvalidStep {
        span: Span,
        start: NaiveDate,
        start_time: Option<Time>,
        prev: NaiveDate,
        prev_time: Option<Time>,
    },
    DeltaNoTime {
        span: Span,
        start: NaiveDate,
        prev: NaiveDate,
    },
    /// A `MOVE a TO b` statement was executed, but there was no entry at the
    /// date `a`.
    MoveWithoutSource { span: Span },
}

pub type Result<T> = result::Result<T, Error>;
