use std::path::PathBuf;
use std::{io, result};

use chrono::NaiveDate;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::Config;
use pest::error::{ErrorVariant, InputLocation};

use crate::error::Eprint;

use super::primitives::Span;
use super::{parse, FileSource, Files};

#[derive(Debug, thiserror::Error)]
#[error("{error}")]
pub struct ParseError<S> {
    file: S,
    error: parse::Error,
}

impl<S> ParseError<S> {
    pub fn new(file: S, error: parse::Error) -> Self {
        Self { file, error }
    }

    fn rule_name(rule: parse::Rule) -> String {
        // TODO Rename rules to be more readable?
        format!("{:?}", rule)
    }

    fn enumerate(rules: &[parse::Rule]) -> String {
        match rules.len() {
            0 => "something".to_string(),
            1 => Self::rule_name(rules[0]),
            n => {
                let except_last = rules
                    .iter()
                    .take(n - 1)
                    .map(|rule| Self::rule_name(*rule))
                    .collect::<Vec<_>>()
                    .join(", ");
                let last = Self::rule_name(rules[n - 1]);
                format!("{} or {}", except_last, last)
            }
        }
    }

    fn notes(&self) -> Vec<String> {
        match &self.error.variant {
            ErrorVariant::ParsingError {
                positives,
                negatives,
            } => {
                let mut notes = vec![];
                if !positives.is_empty() {
                    notes.push(format!("expected {}", Self::enumerate(positives)))
                }
                if !negatives.is_empty() {
                    notes.push(format!("unexpected {}", Self::enumerate(negatives)))
                }
                notes
            }
            ErrorVariant::CustomError { message } => vec![message.clone()],
        }
    }
}

impl<'a, F> Eprint<'a, F> for ParseError<F::FileId>
where
    F: codespan_reporting::files::Files<'a>,
{
    fn eprint<'f: 'a>(&self, files: &'f F, config: &Config) {
        let range = match self.error.location {
            InputLocation::Pos(at) => at..at,
            InputLocation::Span((from, to)) => from..to,
        };
        let name = files.name(self.file).expect("file exists");
        let diagnostic = Diagnostic::error()
            .with_message(format!("Could not parse {}", name))
            .with_labels(vec![Label::primary(self.file, range)])
            .with_notes(self.notes());
        Self::eprint_diagnostic(files, config, &diagnostic);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not resolve {path}: {error}")]
    ResolvePath { path: PathBuf, error: io::Error },
    #[error("Could not load {file}: {error}")]
    ReadFile { file: PathBuf, error: io::Error },
    #[error("Could not write {file}: {error}")]
    WriteFile { file: PathBuf, error: io::Error },
    #[error("Could not resolve timezone {tz}: {error}")]
    ResolveTz {
        file: FileSource,
        span: Span,
        tz: String,
        error: io::Error,
    },
    #[error("Could not determine local timezone: {error}")]
    LocalTz { error: io::Error },
    #[error("{error}")]
    Parse {
        file: FileSource,
        error: parse::Error,
    },
    #[error("Conflicting time zones {tz1} and {tz2}")]
    TzConflict {
        file1: FileSource,
        span1: Span,
        tz1: String,
        file2: FileSource,
        span2: Span,
        tz2: String,
    },
    #[error("Multiple capture commands")]
    MultipleCapture {
        file1: FileSource,
        span1: Span,
        file2: FileSource,
        span2: Span,
    },
    #[error("Duplicate logs for {date}")]
    LogConflict {
        file1: FileSource,
        span1: Span,
        file2: FileSource,
        span2: Span,
        date: NaiveDate,
    },
}

impl<'a> Eprint<'a, Files> for Error {
    fn eprint<'f: 'a>(&self, files: &'f Files, config: &Config) {
        match self {
            Error::ResolvePath { path, error } => {
                eprintln!("Could not resolve path {:?}:", path);
                eprintln!("  {}", error);
            }
            Error::ReadFile { file, error } => {
                eprintln!("Could not read file {:?}:", file);
                eprintln!("  {}", error);
            }
            Error::WriteFile { file, error } => {
                eprintln!("Could not write file {:?}:", file);
                eprintln!("  {}", error);
            }
            Error::ResolveTz {
                file,
                span,
                tz,
                error,
            } => {
                let diagnostic = Diagnostic::error()
                    .with_message(format!("Could not resolve time zone {}", tz))
                    .with_labels(vec![
                        Label::primary(*file, span).with_message("Time zone defined here")
                    ])
                    .with_notes(vec![format!("{}", error)]);
                Self::eprint_diagnostic(files, config, &diagnostic);
            }
            Error::LocalTz { error } => {
                eprintln!("Could not determine local timezone:");
                eprintln!("  {}", error);
            }
            Error::Parse { file, error } => {
                ParseError::new(*file, error.clone()).eprint(files, config)
            }
            Error::TzConflict {
                file1,
                span1,
                tz1,
                file2,
                span2,
                tz2,
            } => {
                let diagnostic = Diagnostic::error()
                    .with_message(format!("Time zone conflict between {} and {}", tz1, tz2))
                    .with_labels(vec![
                        Label::primary(*file1, span1).with_message("Time zone defined here"),
                        Label::primary(*file2, span2).with_message("Time zone defined here"),
                    ])
                    .with_notes(vec![
                        "All TIMEZONE commands must set the same time zone.".to_string()
                    ]);
                Self::eprint_diagnostic(files, config, &diagnostic);
            }
            Error::MultipleCapture {
                file1,
                span1,
                file2,
                span2,
            } => {
                let diagnostic = Diagnostic::error()
                    .with_message("Multiple capture commands")
                    .with_labels(vec![
                        Label::primary(*file1, span1),
                        Label::primary(*file2, span2),
                    ])
                    .with_notes(vec![
                        "There must be at most one CAPTURE command.".to_string()
                    ]);
                Self::eprint_diagnostic(files, config, &diagnostic);
            }
            Error::LogConflict {
                file1,
                span1,
                file2,
                span2,
                date,
            } => {
                let diagnostic = Diagnostic::error()
                    .with_message(format!("Duplicate log entries for {}", date))
                    .with_labels(vec![
                        Label::primary(*file1, span1).with_message("Log defined here"),
                        Label::primary(*file2, span2).with_message("Log defined here"),
                    ])
                    .with_notes(vec!["A day can have at most one LOG entry.".to_string()]);
                Self::eprint_diagnostic(files, config, &diagnostic);
            }
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
