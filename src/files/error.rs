use std::path::PathBuf;
use std::{io, result};

use chrono::NaiveDate;

use super::parse;

// TODO Format TzConflict and LogConflict errors better

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not resolve {path}: {error}")]
    ResolvePath { path: PathBuf, error: io::Error },
    #[error("Could not load {file}: {error}")]
    ReadFile { file: PathBuf, error: io::Error },
    #[error("Could not write {file}: {error}")]
    WriteFile { file: PathBuf, error: io::Error },
    #[error("Could not resolve timezone {timezone}: {error}")]
    ResolveTz { timezone: String, error: io::Error },
    #[error("Could not determine local timezone: {error}")]
    LocalTz { error: io::Error },
    #[error("{0}")]
    Parse(#[from] parse::Error),
    #[error("Conflicting time zones {tz1} and {tz2}")]
    TzConflict { tz1: String, tz2: String },
    #[error("Duplicate logs for {0}")]
    LogConflict(NaiveDate),
}

impl Error {
    pub fn print(&self) {
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
            Error::ResolveTz { timezone, error } => {
                eprintln!("Could not resolve time zone {}:", timezone);
                eprintln!("  {}", error);
            }
            Error::LocalTz { error } => {
                eprintln!("Could not determine local timezone:");
                eprintln!("  {}", error);
            }
            Error::Parse(error) => eprintln!("{}", error),
            Error::TzConflict { tz1, tz2 } => {
                eprintln!("Time zone conflict:");
                eprintln!("  Both {} and {} are specified", tz1, tz2);
            }
            Error::LogConflict(date) => {
                eprintln!("Log conflict:");
                eprintln!("  More than one entry exists for {}", date);
            }
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
