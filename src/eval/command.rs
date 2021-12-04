use std::collections::HashMap;

use chrono::NaiveDate;

use crate::files::SourcedCommand;

use super::{DateRange, Entry, Result};

pub struct CommandState<'a> {
    command: SourcedCommand<'a>,
    range: DateRange,

    from: Option<NaiveDate>,
    until: Option<NaiveDate>,
    entries: HashMap<NaiveDate, Entry>,
}

impl<'a> CommandState<'a> {
    pub fn new(command: SourcedCommand<'a>, range: DateRange) -> Self {
        Self {
            range,
            command,
            from: None,
            until: None,
            entries: HashMap::new(),
        }
    }

    pub fn eval(self) -> Result<Self> {
        todo!()
    }

    pub fn entries(self) -> Vec<Entry> {
        self.entries.into_values().collect()
    }
}
