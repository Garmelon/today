use std::collections::HashMap;

use chrono::NaiveDate;

use crate::files::commands::{DoneDate, Time};
use crate::files::Source;

use super::range::DateRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Task,
    TaskDone(NaiveDate),
    Note,
    Birthday,
}

#[derive(Debug)]
pub struct Entry {
    pub kind: EntryKind,
    pub title: String,
    pub desc: Vec<String>,

    pub source: Source,
    pub date: Option<DoneDate>,
}

pub struct EntryMap {
    pub range: DateRange,
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

    pub fn block(&mut self, date: NaiveDate) {
        if self.range.contains(date) {
            self.map.entry(date).or_insert(None);
        }
    }

    pub fn insert(&mut self, entry: Entry) {
        if let Some(date) = entry.date {
            let date = date.root();
            if self.range.contains(date) {
                self.map.entry(date).or_insert(Some(entry));
            } else if let EntryKind::TaskDone(done_date) = entry.kind {
                if self.range.contains(done_date) {
                    self.map.entry(date).or_insert(Some(entry));
                }
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
