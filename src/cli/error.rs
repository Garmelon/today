use std::result;

use crate::{eval, files};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Files(#[from] files::Error),
    #[error("{0}")]
    Eval(#[from] eval::Error),
    #[error("No number specified")]
    NoNumber,
    #[error("No entry with number {0}")]
    NoSuchEntry(usize),
    #[error("Not a task")]
    NotATask(Vec<usize>),
}

pub type Result<T> = result::Result<T, Error>;
