use std::path::PathBuf;
use std::{io, result};

use super::parse;

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
    #[error("{file1} has time zone {tz1} but {file2} has time zone {tz2}")]
    TzConflict {
        file1: PathBuf,
        tz1: String,
        file2: PathBuf,
        tz2: String,
    },
}

impl Error {
    pub fn print(self) {
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
            Error::TzConflict {
                file1,
                tz1,
                file2,
                tz2,
            } => {
                eprintln!("Time zone conflict:");
                eprintln!("  {:?} has time zone {}", file1, tz1);
                eprintln!("  {:?} has time zone {}", file2, tz2);
            }
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
