use std::result;

use chrono::NaiveDate;

use crate::files::primitives::{Span, Time};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A delta step resulted in an invalid date.
    #[error("delta step resulted in invalid date")]
    DeltaInvalidStep {
        index: usize,
        span: Span,
        start: NaiveDate,
        start_time: Option<Time>,
        prev: NaiveDate,
        prev_time: Option<Time>,
    },
    /// A time-based delta step was applied to a date without time.
    #[error("time-based delta step applied to date without time")]
    DeltaNoTime {
        index: usize,
        span: Span,
        start: NaiveDate,
        prev: NaiveDate,
    },
    /// A `DATE`'s repeat delta did not move the date forwards in time. Instead,
    /// it either remained at the current date (`to == from`) or moved backwards
    /// in time (`to < from`).
    #[error("repeat delta did not move forwards")]
    RepeatDidNotMoveForwards {
        index: usize,
        span: Span,
        from: NaiveDate,
        to: NaiveDate,
    },
    /// A `REMIND`'s delta did not move backwards in time from the entry's start
    /// date. Instead, it either remained at the start date (`to == from`) or
    /// moved forwards in time (`from < to`).
    #[error("remind delta did not move backwards")]
    RemindDidNotMoveBackwards {
        index: usize,
        span: Span,
        from: NaiveDate,
        to: NaiveDate,
    },
    /// A `MOVE a TO b` statement was executed, but there was no entry at the
    /// date `a`.
    #[error("tried to move nonexisting entry")]
    MoveWithoutSource { index: usize, span: Span },
    /// A division by zero has occurred.
    #[error("tried to divide by zero")]
    DivByZero {
        index: usize,
        span: Span,
        date: NaiveDate,
    },
    /// A modulo operation by zero has occurred.
    #[error("tried to modulo by zero")]
    ModByZero {
        index: usize,
        span: Span,
        date: NaiveDate,
    },
    /// Easter calculation failed.
    #[error("easter calculation failed")]
    Easter {
        index: usize,
        span: Span,
        date: NaiveDate,
        msg: &'static str,
    },
}

pub struct SourceInfo<'a> {
    pub name: Option<String>,
    pub content: &'a str,
}

impl Error {
    fn print_at<'a>(sources: &[SourceInfo<'a>], index: &usize, span: &Span, message: String) {
        use pest::error as pe;

        let source = sources.get(*index).expect("index is valid");
        let span = pest::Span::new(source.content, span.start, span.end).expect("span is valid");
        let variant = pe::ErrorVariant::<()>::CustomError { message };
        let mut error = pe::Error::new_from_span(variant, span);
        if let Some(name) = &source.name {
            error = error.with_path(name);
        }

        eprintln!("{}", error);
    }

    fn fmt_date_time(date: NaiveDate, time: Option<Time>) -> String {
        match time {
            None => format!("{}", date),
            Some(time) => format!("{} {}", date, time),
        }
    }

    pub fn print<'a>(&self, sources: &[SourceInfo<'a>]) {
        match self {
            Error::DeltaInvalidStep {
                index,
                span,
                start,
                start_time,
                prev,
                prev_time,
            } => {
                let msg = format!(
                    "Delta step resulted in invalid date\
                    \nInitial start: {}\
                    \nPrevious step: {}",
                    Self::fmt_date_time(*start, *start_time),
                    Self::fmt_date_time(*prev, *prev_time),
                );
                Self::print_at(sources, index, span, msg);
            }
            Error::DeltaNoTime {
                index,
                span,
                start,
                prev,
            } => {
                let msg = format!(
                    "Time-based delta step applied to date without time\
                    \nInitial start: {}\
                    \nPrevious step: {}",
                    start, prev
                );
                Self::print_at(sources, index, span, msg);
            }
            Error::RepeatDidNotMoveForwards {
                index,
                span,
                from,
                to,
            } => {
                let msg = format!(
                    "Repeat delta did not move forwards\
                    \nMoved from {} to {}",
                    from, to
                );
                Self::print_at(sources, index, span, msg);
            }
            Error::RemindDidNotMoveBackwards {
                index,
                span,
                from,
                to,
            } => {
                let msg = format!(
                    "Remind delta did not move backwards\
                    \nMoved from {} to {}",
                    from, to
                );
                Self::print_at(sources, index, span, msg);
            }
            Error::MoveWithoutSource { index, span } => {
                let msg = "Tried to move nonexisting entry".to_string();
                Self::print_at(sources, index, span, msg);
            }
            Error::DivByZero { index, span, date } => {
                let msg = format!(
                    "Tried to divide by zero\
                    \nAt date: {}",
                    date
                );
                Self::print_at(sources, index, span, msg);
            }
            Error::ModByZero { index, span, date } => {
                let msg = format!(
                    "Tried to modulo by zero\
                    \nAt date: {}",
                    date
                );
                Self::print_at(sources, index, span, msg);
            }
            Error::Easter {
                index,
                span,
                date,
                msg,
            } => {
                let msg = format!(
                    "Failed to calculate easter\
                    \nAt date: {}\
                    \nReason: {}",
                    date, msg
                );
                Self::print_at(sources, index, span, msg);
            }
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
