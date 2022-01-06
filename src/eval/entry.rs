use chrono::NaiveDate;

use crate::files::Source;

use super::date::Dates;
use super::range::DateRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Task,
    TaskDone(NaiveDate),
    TaskCanceled(NaiveDate),
    Note,
    Birthday(Option<i32>),
}

/// A single instance of a command.
#[derive(Debug, Clone)]
pub struct Entry {
    pub source: Source,
    pub kind: EntryKind,
    pub title: String,
    pub has_description: bool,
    pub dates: Option<Dates>,
    /// Remind the user of an entry before it occurs. This date should always be
    /// before the entry's start date, or `None` if there is no start date.
    pub remind: Option<NaiveDate>,
}

impl Entry {
    pub fn new(
        source: Source,
        kind: EntryKind,
        title: String,
        has_description: bool,
        dates: Option<Dates>,
        remind: Option<NaiveDate>,
    ) -> Self {
        if let Some(dates) = dates {
            if let Some(remind) = remind {
                assert!(remind < dates.sorted().root());
            }
        } else {
            assert!(remind.is_none());
        }

        Self {
            source,
            kind,
            title,
            has_description,
            dates,
            remind,
        }
    }

    pub fn root(&self) -> Option<NaiveDate> {
        self.dates.map(|dates| dates.root())
    }
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
        match entry.root() {
            Some(date) => self.range.contains(date),
            None => false,
        }
    }

    fn is_touching(&self, entry: &Entry) -> bool {
        if let Some(dates) = entry.dates {
            let (start, end) = dates.sorted().dates();
            // Inside the range or overlapping it
            start <= self.range.until() && self.range.from() <= end
        } else {
            false
        }
    }

    fn is_relevant(&self, entry: &Entry) -> bool {
        if entry.dates.is_none() {
            return true;
        }

        // Anything close to the range
        if self.is_touching(entry) {
            return true;
        }

        if let (Some(remind), Some(dates)) = (entry.remind, entry.dates) {
            let (_, end) = dates.sorted().dates();
            let remind_before = remind <= self.range.until();
            let entry_before = end < self.range.from();
            if remind_before && !entry_before {
                return true;
            }
        }

        // Tasks that were finished inside the range
        if let EntryKind::TaskDone(done) | EntryKind::TaskCanceled(done) = entry.kind {
            if self.range.contains(done) {
                return true;
            }
        }

        // Unfinished tasks before or inside the range
        if let EntryKind::Task = entry.kind {
            if let Some(dates) = entry.dates {
                let (start, _) = dates.sorted().dates();
                if start <= self.range.until() {
                    return true;
                }
            }
        }

        false
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
