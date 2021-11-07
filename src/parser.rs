use std::cmp::min;
use std::process::Command;

use crate::source::{SourceFile, SourceSpan};

#[derive(Debug)]
pub struct ParseError(SourceSpan, String);

type ParseResult<T> = Result<T, ParseError>;

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

    fn peek(&self, amount: usize) -> &'a str {
        let end = min(self.content.len(), self.offset + amount);
        &self.content[self.offset..end]
    }

    fn peek_rest(&self) -> &'a str {
        &self.content[self.offset..]
    }

    fn at_eof(&self) -> bool {
        self.offset >= self.content.len()
    }

    fn take(&mut self, amount: usize) {
        self.offset += amount;
    }

    fn error(&self, at: usize, error: impl ToString) -> ParseError {
        ParseError(self.file.span(at..at), error.to_string())
    }

    fn uncritical(&mut self, at: usize, error: impl ToString) {
        self.errors.push(self.error(at, error));
    }

    fn critical(&self, at: usize, error: impl ToString) -> ParseResult<()> {
        Err(self.error(at, error))
    }

    pub fn parse(&mut self) -> Result<Vec<Command>, Vec<ParseError>> {
        match self.parse_commands() {
            Ok(commands) => {
                if self.errors.is_empty() {
                    return Ok(commands);
                }
            }
            Err(error) => {
                self.errors.push(error);
            }
        };

        Err(self.errors.split_off(0))
    }

    fn parse_commands(&mut self) -> ParseResult<Vec<Command>> {
        let mut commands = vec![];

        self.skip_empty_lines();
        while !self.at_eof() {
            commands.push(self.parse_command()?);
        }

        if !self.at_eof() {
            self.uncritical(self.offset, "expected EOF");
        }

        Ok(commands)
    }

    fn skip_empty_lines(&mut self) {
        loop {
            if let Some(i) = self.peek_rest().find('\n') {
                if self.peek(i).chars().all(|c| c.is_whitespace()) {
                    self.take(i + 1); // Include the newline
                } else {
                    break;
                }
            } else if self.peek_rest().chars().all(|c| c.is_whitespace()) {
                self.take(self.peek_rest().len());
            } else {
                break;
            }
        }
    }

    fn parse_command(&mut self) -> ParseResult<Command> {
        todo!()
    }
}
