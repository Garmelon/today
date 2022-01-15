use std::{io, result};

use chrono::NaiveDate;
use codespan_reporting::files::{Files, SimpleFile};
use codespan_reporting::term::Config;

use crate::error::Eprint;
use crate::files::FileSource;
use crate::{eval, files};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Eval(#[from] eval::Error<FileSource>),
    #[error("{error}")]
    ArgumentParse {
        file: SimpleFile<String, String>,
        error: files::ParseError<()>,
    },
    #[error("{error}")]
    ArgumentEval {
        file: SimpleFile<String, String>,
        error: eval::Error<()>,
    },
    #[error("No entry with number {0}")]
    NoSuchEntry(usize),
    #[error("No log for {0}")]
    NoSuchLog(NaiveDate),
    #[error("Not a task")]
    NotATask(Vec<usize>),
    #[error("No capture file found")]
    NoCaptureFile,
    #[error("Error editing: {0}")]
    EditingIo(io::Error),
}

pub type Result<T> = result::Result<T, Error>;

impl<'a, F> Eprint<'a, F> for Error
where
    F: Files<'a, FileId = FileSource>,
{
    fn eprint<'f: 'a>(&self, files: &'f F, config: &Config) {
        match self {
            Error::Eval(e) => e.eprint(files, config),
            Error::ArgumentParse { file, error } => error.eprint(file, config),
            Error::ArgumentEval { file, error } => error.eprint(file, config),
            Error::NoSuchEntry(n) => eprintln!("No entry with number {n}"),
            Error::NoSuchLog(date) => eprintln!("No log for {date}"),
            Error::NotATask(ns) => {
                if ns.is_empty() {
                    eprintln!("Not a task.");
                } else if ns.len() == 1 {
                    eprintln!("{} is not a task.", ns[0]);
                } else {
                    let ns = ns.iter().map(|n| n.to_string()).collect::<Vec<_>>();
                    eprintln!("{} are not tasks.", ns.join(", "));
                }
            }
            Error::NoCaptureFile => eprintln!("No capture file found"),
            Error::EditingIo(error) => {
                eprintln!("Error while editing:");
                eprintln!("  {error}");
            }
        }
    }
}
