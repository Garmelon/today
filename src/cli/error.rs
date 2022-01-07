use chrono::NaiveDate;
use codespan_reporting::files::Files;
use codespan_reporting::term::Config;

use crate::error::Eprint;
use crate::eval;

#[derive(Debug, thiserror::Error)]
pub enum Error<S> {
    #[error("{0}")]
    Eval(#[from] eval::Error<S>),
    #[error("No entry with number {0}")]
    NoSuchEntry(usize),
    #[error("No log for {0}")]
    NoSuchLog(NaiveDate),
    #[error("Not a task")]
    NotATask(Vec<usize>),
}

impl<'a, F: Files<'a>> Eprint<'a, F> for Error<F::FileId> {
    fn eprint<'f: 'a>(&self, files: &'f F, config: &Config) {
        match self {
            Error::Eval(e) => e.eprint(files, config),
            Error::NoSuchEntry(n) => eprintln!("No entry with number {}", n),
            Error::NoSuchLog(date) => eprintln!("No log for {}", date),
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
        }
    }
}
