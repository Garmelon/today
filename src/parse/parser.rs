use std::cmp::min;

use codespan_reporting::diagnostic::{Diagnostic, Label};

use crate::source::{SourceFile, SourceSpan};

// TODO Add warnings for things like trailing whitespace

#[derive(Debug)]
pub struct ParseError(SourceSpan, String);

impl From<&ParseError> for Diagnostic<usize> {
    fn from(e: &ParseError) -> Self {
        Self::error()
            .with_message(&e.1)
            .with_labels(vec![Label::primary(e.0.file_id(), e.0.range())])
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct Parser<'a> {
    file: SourceFile,
    content: &'a str,
    offset: usize,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(file: SourceFile, content: &'a str) -> Self {
        Self {
            file,
            content,
            offset: 0,
            errors: vec![],
        }
    }

    pub fn peek(&self, amount: usize) -> &'a str {
        // self.offset is always a valid start since take() ensures it is in the
        // range 0..self.content.len()
        let end = min(self.content.len(), self.offset + amount);
        &self.content[self.offset..end]
    }

    pub fn peek_rest(&self) -> &'a str {
        // self.offset is always a valid start since take() ensures it is in the
        // range 0..self.content.len()
        &self.content[self.offset..]
    }

    pub fn peek_line(&self) -> &'a str {
        if let Some(i) = self.peek_rest().find('\n') {
            self.peek(i)
        } else {
            self.peek_rest()
        }
    }

    pub fn at(&self) -> usize {
        self.offset
    }

    pub fn at_eof(&self) -> bool {
        self.offset >= self.content.len()
    }

    pub fn take(&mut self, amount: usize) {
        // Ensure the offset always stays in the range 0..self.content.len()
        self.offset = min(self.content.len(), self.offset + amount);
    }

    pub fn take_line(&mut self) {
        self.take(self.peek_line().len() + 1);
    }

    fn error(&self, at: usize, error: impl ToString) -> ParseError {
        ParseError(self.file.span(at..at), error.to_string())
    }

    pub fn uncritical(&mut self, at: usize, error: impl ToString) {
        self.errors.push(self.error(at, error));
    }

    pub fn critical<T>(&self, at: usize, error: impl ToString) -> ParseResult<T> {
        Err(self.error(at, error))
    }

    pub fn parse<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> ParseResult<T>,
    ) -> Result<T, Vec<ParseError>> {
        match f(self) {
            Ok(result) => {
                if self.errors.is_empty() {
                    return Ok(result);
                }
            }
            Err(error) => {
                self.errors.push(error);
            }
        };

        Err(self.errors.split_off(0))
    }
}
