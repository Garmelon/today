use chrono::NaiveDate;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::Files;
use codespan_reporting::term::Config;

use crate::error::Eprint;
use crate::files::primitives::{Span, Time};

#[derive(Debug, thiserror::Error)]
pub enum Error<S> {
    /// A delta step resulted in an invalid date.
    #[error("delta step resulted in invalid date")]
    DeltaInvalidStep {
        index: S,
        span: Span,
        start: NaiveDate,
        start_time: Option<Time>,
        prev: NaiveDate,
        prev_time: Option<Time>,
    },
    /// A time-based delta step was applied to a date without time.
    #[error("time-based delta step applied to date without time")]
    DeltaNoTime {
        index: S,
        span: Span,
        start: NaiveDate,
        prev: NaiveDate,
    },
    /// A `DATE`'s repeat delta did not move the date forwards in time. Instead,
    /// it either remained at the current date (`to == from`) or moved backwards
    /// in time (`to < from`).
    #[error("repeat delta did not move forwards")]
    RepeatDidNotMoveForwards {
        index: S,
        span: Span,
        from: NaiveDate,
        to: NaiveDate,
    },
    /// A `REMIND`'s delta did not move backwards in time from the entry's start
    /// date. Instead, it either remained at the start date (`to == from`) or
    /// moved forwards in time (`from < to`).
    #[error("remind delta did not move backwards")]
    RemindDidNotMoveBackwards {
        index: S,
        span: Span,
        from: NaiveDate,
        to: NaiveDate,
    },
    /// A `MOVE a TO b` statement was executed, but there was no entry at the
    /// date `a`.
    #[error("tried to move nonexisting entry")]
    MoveWithoutSource { index: S, span: Span },
    /// A `MOVE a TO b` statement was executed where `b` contains a time but `a`
    /// doesn't was executed.
    #[error("tried to move un-timed entry to new time")]
    TimedMoveWithoutTime { index: S, span: Span },
    /// A division by zero has occurred.
    #[error("tried to divide by zero")]
    DivByZero {
        index: S,
        span: Span,
        date: NaiveDate,
    },
    /// A modulo operation by zero has occurred.
    #[error("tried to modulo by zero")]
    ModByZero {
        index: S,
        span: Span,
        date: NaiveDate,
    },
    /// Easter calculation failed.
    #[error("easter calculation failed")]
    Easter {
        index: S,
        span: Span,
        date: NaiveDate,
        msg: &'static str,
    },
}

impl<S> Error<S> {
    fn fmt_date_time(date: NaiveDate, time: Option<Time>) -> String {
        match time {
            None => format!("{}", date),
            Some(time) => format!("{} {}", date, time),
        }
    }
}

impl<'a, F: Files<'a>> Eprint<'a, F> for Error<F::FileId> {
    fn eprint<'f: 'a>(&self, files: &'f F, config: &Config) {
        let diagnostic = match self {
            Error::DeltaInvalidStep {
                index,
                span,
                start,
                start_time,
                prev,
                prev_time,
            } => {
                let start_str = Self::fmt_date_time(*start, *start_time);
                let prev_str = Self::fmt_date_time(*prev, *prev_time);
                Diagnostic::error()
                    .with_message("Delta step resulted in invalid date")
                    .with_labels(vec![
                        Label::primary(*index, span).with_message("At this step")
                    ])
                    .with_notes(vec![
                        format!("Date before applying delta: {}", start_str),
                        format!("Date before applying this step: {}", prev_str),
                    ])
            }
            Error::DeltaNoTime {
                index,
                span,
                start,
                prev,
            } => Diagnostic::error()
                .with_message("Time-based delta step applied to date without time")
                .with_labels(vec![
                    Label::primary(*index, span).with_message("At this step")
                ])
                .with_notes(vec![
                    format!("Date before applying delta: {}", start),
                    format!("Date before applying this step: {}", prev),
                ]),
            Error::RepeatDidNotMoveForwards {
                index,
                span,
                from,
                to,
            } => Diagnostic::error()
                .with_message("Repeat delta did not move forwards")
                .with_labels(vec![Label::primary(*index, span).with_message("This delta")])
                .with_notes(vec![format!("Moved from {} to {}", from, to)]),
            Error::RemindDidNotMoveBackwards {
                index,
                span,
                from,
                to,
            } => Diagnostic::error()
                .with_message("Remind delta did not move backwards")
                .with_labels(vec![Label::primary(*index, span).with_message("This delta")])
                .with_notes(vec![format!("Moved from {} to {}", from, to)]),
            Error::MoveWithoutSource { index, span } => Diagnostic::error()
                .with_message("Tried to move nonexistent entry")
                .with_labels(vec![Label::primary(*index, span).with_message("Here")]),
            Error::TimedMoveWithoutTime { index, span } => Diagnostic::error()
                .with_message("Tried to move un-timed entry to new time")
                .with_labels(vec![Label::primary(*index, span).with_message("Here")]),
            Error::DivByZero { index, span, date } => Diagnostic::error()
                .with_message("Tried to divide by zero")
                .with_labels(vec![
                    Label::primary(*index, span).with_message("This expression")
                ])
                .with_notes(vec![format!("At date: {}", date)]),
            Error::ModByZero { index, span, date } => Diagnostic::error()
                .with_message("Tried to modulo by zero")
                .with_labels(vec![
                    Label::primary(*index, span).with_message("This expression")
                ])
                .with_notes(vec![format!("At date: {}", date)]),
            Error::Easter {
                index,
                span,
                date,
                msg,
            } => Diagnostic::error()
                .with_message("Failed to calculate easter")
                .with_labels(vec![
                    Label::primary(*index, span).with_message("This expression")
                ])
                .with_notes(vec![
                    format!("At date: {}", date),
                    format!("Reason: {}", msg),
                ]),
        };
        Self::eprint_diagnostic(files, config, &diagnostic);
    }
}
