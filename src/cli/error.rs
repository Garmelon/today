use std::result;

use crate::{eval, files};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Files(#[from] files::Error),
    #[error("{0}")]
    Eval(#[from] eval::Error),
    #[error("No entry with number {0}")]
    NoSuchEntry(usize),
}

pub type Result<T> = result::Result<T, Error>;
