use std::path::PathBuf;
use std::{io, result};

use chrono::NaiveDate;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::Config;

use crate::error::Eprint;

use super::primitives::Span;
use super::{parse, FileSource, Files};

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
    #[error("{0}")]
    Parse(#[from] parse::Error),
    #[error("Conflicting time zones {tz1} and {tz2}")]
    TzConflict {
        file1: FileSource,
        span1: Span,
        tz1: String,
        file2: FileSource,
        span2: Span,
        tz2: String,
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
            // TODO Format using codespan-reporting as well
            Error::Parse(error) => eprintln!("{}", error),
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
