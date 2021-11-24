use std::collections::HashMap;

use chrono::NaiveDate;

use crate::files::commands::Time;
use crate::files::Source;

use super::range::DateRange;

#[derive(Debug, PartialEq, Eq)]
pub enum EntryKind {
    Task,
    DoneTask,
    Note,
    Birthday,
}

#[derive(Debug)]
pub enum EntryDate {
    None,
    Date {
        root: NaiveDate,
    },
    DateWithTime {
        root: NaiveDate,
        root_time: Time,
    },
    DateToDate {
        root: NaiveDate,
        other: NaiveDate,
    },
    DateToDateWithTime {
        root: NaiveDate,
        root_time: Time,
        other: NaiveDate,
        other_time: Time,
    },
}

impl EntryDate {
    pub fn root(&self) -> Option<NaiveDate> {
        match self {
            EntryDate::None => None,
            EntryDate::Date { root, .. } => Some(*root),
            EntryDate::DateWithTime { root, .. } => Some(*root),
            EntryDate::DateToDate { root, .. } => Some(*root),
            EntryDate::DateToDateWithTime { root, .. } => Some(*root),
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub kind: EntryKind,
    pub title: String,
    pub desc: Vec<String>,

    pub source: Source,
    pub date: EntryDate,
}

pub struct EntryMap {
    range: DateRange,
    map: HashMap<NaiveDate, Option<Entry>>,
    undated: Vec<Entry>,
}

impl EntryMap {
    pub fn new(range: DateRange) -> Self {
        Self {
            range,
            map: HashMap::new(),
            undated: vec![],
        }
    }

    pub fn range(&self) -> DateRange {
        self.range
    }

    pub fn set_from(&mut self, from: NaiveDate) {
        self.range = DateRange::new(from, self.range.until());
    }

    pub fn set_until(&mut self, until: NaiveDate) {
        self.range = DateRange::new(self.range.from(), until);
    }

    pub fn block(&mut self, date: NaiveDate) {
        if self.range.contains(date) {
            self.map.entry(date).or_insert(None);
        }
    }

    pub fn insert(&mut self, entry: Entry) {
        if let Some(date) = entry.date.root() {
            if self.range.contains(date) {
                self.map.entry(date).or_insert(Some(entry));
            }
        } else {
            self.undated.push(entry);
        }
    }

    pub fn drain(&mut self) -> Vec<Entry> {
        self.map
            .drain()
            .filter_map(|(_, entry)| entry)
            .chain(self.undated.drain(..))
            .collect()
    }
}
