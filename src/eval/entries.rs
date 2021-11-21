use std::collections::HashMap;
use std::ops::RangeInclusive;

use chrono::{Datelike, NaiveDate};

use crate::files::commands::Time;

#[derive(Debug, PartialEq, Eq)]
pub enum EntryKind {
    Task,
    DoneTask,
    Note,
    Birthday,
}

impl EntryKind {
    pub fn done(&mut self) {
        if matches!(self, Self::Task) {
            *self = Self::DoneTask;
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub kind: EntryKind,
    pub title: String,
    pub desc: Vec<String>,

    /// Index in the source file
    pub source: usize,

    pub start: Option<NaiveDate>,
    pub start_time: Option<Time>,
    pub end: Option<NaiveDate>,
    pub end_time: Option<Time>,
}

impl Entry {
    pub fn new(source: usize, kind: EntryKind, title: String) -> Self {
        Self {
            kind,
            title,
            desc: vec![],
            source,
            start: None,
            start_time: None,
            end: None,
            end_time: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DateRange {
    pub from: NaiveDate,
    pub until: NaiveDate,
}

impl DateRange {
    pub fn new(from: NaiveDate, until: NaiveDate) -> Self {
        assert!(from <= until);
        Self { from, until }
    }

    pub fn years(&self) -> RangeInclusive<i32> {
        self.from.year()..=self.until.year()
    }
}

pub struct EntryMap {
    pub range: DateRange,
    pub from: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
    pub map: HashMap<NaiveDate, Option<Entry>>,
}

impl EntryMap {
    pub fn new(range: DateRange) -> Self {
        Self {
            range,
            from: None,
            until: None,
            map: HashMap::new(),
        }
    }

    pub fn block(&mut self, date: NaiveDate) {
        self.map.entry(date).or_insert(None);
    }

    pub fn insert(&mut self, date: NaiveDate, entry: Entry) {
        self.map.entry(date).or_insert(Some(entry));
    }

    pub fn drain(&mut self) -> Vec<Entry> {
        self.map.drain().filter_map(|(_, entry)| entry).collect()
    }
}
