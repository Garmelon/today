use std::result;

use chrono::NaiveDate;

use crate::files::primitives::{Span, Time};
use crate::files::Files;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A delta step resulted in an invalid date.
    #[error("delta step resulted in invalid date")]
    DeltaInvalidStep {
        file: usize,
        span: Span,
        start: NaiveDate,
        start_time: Option<Time>,
        prev: NaiveDate,
        prev_time: Option<Time>,
    },
    /// A time-based delta step was applied to a date without time.
    #[error("time-based delta step applied to date without time")]
    DeltaNoTime {
        file: usize,
        span: Span,
        start: NaiveDate,
        prev: NaiveDate,
    },
    /// A `DATE`'s repeat delta did not move the date forwards in time. Instead,
    /// it either remained at the current date (`to == from`) or moved backwards
    /// in time (`to < from`).
    #[error("repeat delta did not move forwards")]
    RepeatDidNotMoveForwards {
        file: usize,
        span: Span,
        from: NaiveDate,
        to: NaiveDate,
    },
    /// A `MOVE a TO b` statement was executed, but there was no entry at the
    /// date `a`.
    #[error("tried to move nonexisting entry")]
    MoveWithoutSource { file: usize, span: Span },
    /// A division by zero has occurred.
    #[error("tried to divide by zero")]
    DivByZero {
        file: usize,
        span: Span,
        date: NaiveDate,
    },
    /// A modulo operation by zero has occurred.
    #[error("tried to modulo by zero")]
    ModByZero {
        file: usize,
        span: Span,
        date: NaiveDate,
    },
    /// Easter calculation failed.
    #[error("easter calculation failed")]
    Easter {
        file: usize,
        span: Span,
        date: NaiveDate,
        msg: &'static str,
    },
}

impl Error {
    fn print_at(files: &Files, file: &usize, span: &Span, message: String) {
        use pest::error as pe;
        let (name, content) = files.file(*file).expect("file index is valid");
        let span = pest::Span::new(content, span.start, span.end).expect("span is valid");
        let variant = pe::ErrorVariant::<()>::CustomError { message };
        let error = pe::Error::new_from_span(variant, span).with_path(&name.to_string_lossy());
        eprintln!("{}", error);
    }

    fn fmt_date_time(date: NaiveDate, time: Option<Time>) -> String {
        match time {
            None => format!("{}", date),
            Some(time) => format!("{} {}", date, time),
        }
    }

    pub fn print(&self, files: &Files) {
        match self {
            Error::DeltaInvalidStep {
                file,
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
                Self::print_at(files, file, span, msg);
            }
            Error::DeltaNoTime {
                file,
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
                Self::print_at(files, file, span, msg);
            }
            Error::RepeatDidNotMoveForwards {
                file,
                span,
                from,
                to,
            } => {
                let msg = format!(
                    "Repeat delta did not move forwards\
                    \nMoved from {} to {}",
                    from, to
                );
                Self::print_at(files, file, span, msg);
            }
            Error::MoveWithoutSource { file, span } => {
                let msg = "Tried to move nonexisting entry".to_string();
                Self::print_at(files, file, span, msg);
            }
            Error::DivByZero { file, span, date } => {
                let msg = format!(
                    "Tried to divide by zero\
                    \nAt date: {}",
                    date
                );
                Self::print_at(files, file, span, msg);
            }
            Error::ModByZero { file, span, date } => {
                let msg = format!(
                    "Tried to modulo by zero\
                    \nAt date: {}",
                    date
                );
                Self::print_at(files, file, span, msg);
            }
            Error::Easter {
                file,
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
                Self::print_at(files, file, span, msg);
            }
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
