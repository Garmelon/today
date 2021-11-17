pub struct Parser<'d> {
    data: &'d str,
    index: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum Reason {
    #[error("expected character {expected:?} at {rest:?}")]
    ExpectedChar { expected: char, rest: String },
    #[error("expected string {expected:?} at {rest:?}")]
    ExpectedStr { expected: String, rest: String },
    #[error("expected whitespace at {rest:?}")]
    ExpectedWhitespace { rest: String },
}

impl<'d> Parser<'d> {
    pub fn new(data: &'d str) -> Self {
        Self { data, index: 0 }
    }

    fn rest(&self) -> &'d str {
        &self.data[self.index..]
    }

    pub fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    pub fn take(&mut self) -> Option<char> {
        if let Some(c) = self.peek() {
            self.index += c.len_utf8();
            Some(c)
        } else {
            None
        }
    }

    pub fn take_exact(&mut self, c: char) -> Result<(), Reason> {
        if self.peek() == Some(c) {
            self.take();
            Ok(())
        } else {
            Err(Reason::ExpectedChar {
                expected: c,
                rest: self.rest().to_string(),
            })
        }
    }

    pub fn take_any_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.take();
            } else {
                break;
            }
        }
    }

    pub fn take_some_whitespace(&mut self) -> Result<(), Reason> {
        match self.peek() {
            Some(c) if c.is_whitespace() => {
                self.take();
                self.take_any_whitespace();
                Ok(())
            }
            _ => Err(Reason::ExpectedWhitespace {
                rest: self.rest().to_string(),
            }),
        }
    }

    pub fn starts_with(&self, pattern: &str) -> bool {
        self.data.starts_with(pattern)
    }

    pub fn take_starting_with(&mut self, pattern: &str) -> Result<(), Reason> {
        if self.starts_with(pattern) {
            self.index += pattern.len();
            Ok(())
        } else {
            Err(Reason::ExpectedStr {
                expected: pattern.to_string(),
                rest: self.rest().to_string(),
            })
        }
    }
}
