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
}

pub type Result<T> = result::Result<T, Error>;
