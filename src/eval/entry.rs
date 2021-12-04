use chrono::NaiveDate;

use crate::files::commands::DoneDate;
use crate::files::Source;

use super::range::DateRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Task,
    TaskDone(NaiveDate),
    Note,
    Birthday,
}

/// A single instance of a command.
#[derive(Debug)]
pub struct Entry {
    pub kind: EntryKind,
    pub title: String,
    pub desc: Vec<String>,

    pub source: Source,
    pub root: Option<DoneDate>,
}

/// Mode that determines how entries are filtered when they are added to
/// an [`Entries`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryMode {
    /// The entry's root date must be contained in the range.
    Rooted,
    /// The entry must overlap the range.
    Touching,
    /// The entry must be in some way relevant to the range. It may
    /// - touch the range,
    /// - be an unfinished task that lies before the range,
    /// - be a finished task that was completed inside the range, or
    /// - have no root date.
    Relevant,
}

pub struct Entries {
    mode: EntryMode,
    range: DateRange,
    entries: Vec<Entry>,
}

impl Entries {
    pub fn new(mode: EntryMode, range: DateRange) -> Self {
        Self {
            mode,
            range,
            entries: vec![],
        }
    }

    fn is_rooted(&self, entry: &Entry) -> bool {
        match entry.root {
            Some(date) => self.range.contains(date.root()),
            None => false,
        }
    }

    fn is_touching(&self, entry: &Entry) -> bool {
        if let Some(date) = entry.root {
            // Inside the range or overlapping it
            date.first() <= self.range.until() && self.range.from() <= date.last()
        } else {
            false
        }
    }

    fn is_relevant(&self, entry: &Entry) -> bool {
        if entry.root.is_none() {
            return true;
        }

        // Anything close to the range
        if self.is_touching(entry) {
            return true;
        }

        // Tasks that were finished inside the range
        if let EntryKind::TaskDone(done) = entry.kind {
            if self.range.contains(done) {
                return true;
            }
        }

        // Unfinished tasks before or inside the range
        if let EntryKind::Task = entry.kind {
            if let Some(date) = entry.root {
                if date.first() <= self.range.until() {
                    return true;
                }
            }
        }

        return false;
    }

    pub fn add(&mut self, entry: Entry) {
        let keep = match self.mode {
            EntryMode::Rooted => self.is_rooted(&entry),
            EntryMode::Touching => self.is_touching(&entry),
            EntryMode::Relevant => self.is_relevant(&entry),
        };
        if keep {
            self.entries.push(entry);
        }
    }

    pub fn entries(self) -> Vec<Entry> {
        self.entries
    }
}
