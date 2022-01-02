use chrono::NaiveDate;

use crate::files::arguments::{Range, RangeDate};
use crate::files::{FileSource, Files};

use self::command::{CommandState, EvalCommand};
pub use self::date::Dates;
use self::delta::Delta;
use self::entry::Entries;
pub use self::entry::{Entry, EntryKind, EntryMode};
pub use self::error::Error;
pub use self::range::DateRange;

mod command;
mod date;
mod delta;
mod entry;
mod error;
mod range;
mod util;

impl Files {
    pub fn eval(&self, mode: EntryMode, range: DateRange) -> Result<Vec<Entry>, Error<FileSource>> {
        let mut entries = Entries::new(mode, range);
        for command in self.commands() {
            let source = command.source;
            if let Some(command) = EvalCommand::new(command.command) {
                for entry in CommandState::new(command, source, range).eval()?.entries() {
                    entries.add(entry);
                }
            }
        }
        Ok(entries.entries())
    }
}

impl Range {
    pub fn eval<S: Copy>(&self, index: S, today: NaiveDate) -> Result<DateRange, Error<S>> {
        let mut start = match self.start {
            RangeDate::Date(d) => d,
            RangeDate::Today => today,
        };

        if let Some(delta) = &self.start_delta {
            let delta: Delta = delta.into();
            start = delta.apply_date(index, start)?;
        }

        let mut end = start;

        match self.end {
            Some(RangeDate::Date(d)) => end = d,
            Some(RangeDate::Today) => end = today,
            None => {}
        }

        if let Some(delta) = &self.end_delta {
            let delta: Delta = delta.into();
            end = delta.apply_date(index, end)?;
        }

        Ok(DateRange::new(start, end))
    }
}
