use std::result;

use chrono::NaiveDate;

use crate::files::primitives::{Span, Time};

#[derive(Debug)]
pub enum Error {
    /// A delta step resulted in an invalid date.
    DeltaInvalidStep {
        span: Span,
        start: NaiveDate,
        start_time: Option<Time>,
        prev: NaiveDate,
        prev_time: Option<Time>,
    },
    /// A time-based delta step was applied to a date without time.
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
    /// A division by zero has occurred.
    DivByZero { span: Span, date: NaiveDate },
    /// A modulo operation by zero has occurred.
    ModByZero { span: Span, date: NaiveDate },
    /// Easter calculation failed.
    Easter {
        span: Span,
        date: NaiveDate,
        msg: &'static str,
    },
}

pub type Result<T> = result::Result<T, Error>;
