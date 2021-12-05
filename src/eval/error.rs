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
    /// A `DATE`'s repeat delta did not move the date forwards in time. Instead,
    /// it either remained at the current date (`to == from`) or moved backwards
    /// in time (`to < from`).
    RepeatDidNotMoveForwards {
        span: Span,
        from: NaiveDate,
        to: NaiveDate,
    },
    /// A `MOVE a TO b` statement was executed, but there was no entry at the
    /// date `a`.
    MoveWithoutSource { span: Span },
}

pub type Result<T> = result::Result<T, Error>;
