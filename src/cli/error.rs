use std::result;

use crate::eval;
use crate::files::Files;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Eval(#[from] eval::Error),
    #[error("No entry with number {0}")]
    NoSuchEntry(usize),
    #[error("Not a task")]
    NotATask(Vec<usize>),
}

impl Error {
    pub fn print(&self, files: &Files) {
        match self {
            Error::Eval(e) => e.print(files),
            Error::NoSuchEntry(n) => eprintln!("No entry with number {}", n),
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

pub type Result<T> = result::Result<T, Error>;
