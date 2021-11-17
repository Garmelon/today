use std::error;

#[derive(Debug, thiserror::Error)]
#[error("line {line}: {reason}")]
pub struct ParseError {
    line: usize,
    reason: Box<dyn error::Error>,
}

impl ParseError {
    #[must_use]
    pub fn new(line: usize, reason: impl error::Error + 'static) -> Self {
        Self {
            line,
            reason: Box::new(reason),
        }
    }

    #[must_use]
    pub fn pack<T>(line: usize, reason: impl error::Error + 'static) -> Result<T, Self> {
        Err(Self::new(line, reason))
    }
}

pub trait ToParseError: error::Error + 'static + Sized {
    #[must_use]
    fn at(self, line: usize) -> ParseError {
        ParseError::new(line, self)
    }
}

impl<E: error::Error + 'static> ToParseError for E {}
