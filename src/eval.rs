use crate::files::Files;

use self::command::CommandState;
use self::date::Dates;
use self::entry::Entries;
pub use self::entry::{Entry, EntryKind, EntryMode};
pub use self::error::{Error, Result};
pub use self::range::DateRange;

mod command;
mod date;
mod delta;
mod entry;
mod error;
mod range;
mod util;

impl Files {
    pub fn eval(&self, mode: EntryMode, range: DateRange) -> Result<Vec<Entry>> {
        let mut entries = Entries::new(mode, range);
        for command in self.commands() {
            for entry in CommandState::new(command, range).eval()?.entries() {
                entries.add(entry);
            }
        }
        let mut entries = entries.entries();
        entries.sort_by_key(|e| {
            e.dates.map(|d| {
                let (start, end) = d.start_end();
                if let Some((start_time, end_time)) = d.start_end_time() {
                    (start, Some(start_time), end, Some(end_time))
                } else {
                    (start, None, end, None)
                }
            })
        });
        Ok(entries)
    }
}
