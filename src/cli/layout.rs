use std::collections::HashMap;

use chrono::NaiveDate;

use crate::eval::{DateRange, Dates, Entry, EntryKind};
use crate::files::primitives::Time;
use crate::files::Files;

#[derive(Debug)]
pub struct TimedLayout {
    pub ending: Vec<usize>,
    pub at: Vec<usize>,
    pub starting: Vec<usize>,
}

impl TimedLayout {
    pub fn new() -> Self {
        Self {
            ending: vec![],
            at: vec![],
            starting: vec![],
        }
    }
}

#[derive(Debug)]
pub struct DayLayout {
    pub ending: Vec<usize>,
    pub timed: HashMap<Time, TimedLayout>,
    pub at: Vec<usize>,
    pub other: Vec<usize>,
    pub starting: Vec<usize>,
}

impl DayLayout {
    pub fn new() -> Self {
        Self {
            ending: vec![],
            timed: HashMap::new(),
            at: vec![],
            other: vec![],
            starting: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Layout {
    pub range: DateRange,
    pub today: NaiveDate,
    pub days: HashMap<NaiveDate, DayLayout>,
}

impl Layout {
    pub fn new(range: DateRange, today: NaiveDate) -> Self {
        let mut days = HashMap::new();
        for day in range.days() {
            days.insert(day, DayLayout::new());
        }
        Self { range, today, days }
    }

    pub fn layout(&mut self, files: &Files, entries: &[Entry]) {
        let mut commands = entries
            .iter()
            .enumerate()
            .map(|(i, e)| (i, e, files.command(e.source)))
            .collect::<Vec<_>>();

        // Sort entries (maintaining the keys) so the output is more deterministic
        commands.sort_by_key(|(_, _, c)| c.title());
        commands.sort_by_key(|(_, e, _)| e.dates.map(|d| (d.end(), d.end_time())));
        commands.sort_by_key(|(_, e, _)| e.dates.map(|d| (d.start(), d.start_time())));

        for (index, entry, _) in commands {
            self.insert(index, entry);
        }
    }

    fn insert(&mut self, index: usize, entry: &Entry) {
        if let Some(dates) = entry.dates {
            self.insert_dated(index, dates);
            if let EntryKind::TaskDone(at) = entry.kind {
                self.insert_other(at, index);
            }
        } else {
            self.insert_other(self.today, index);
        }
    }

    fn insert_dated(&mut self, index: usize, dates: Dates) {
        let (start, end) = dates.start_end();
        if start < self.range.from() && self.range.until() < end {
            self.insert_other(self.today, index);
        } else if let Some((date, time)) = dates.point_in_time() {
            self.insert_at(date, time, index);
        } else {
            let (start_time, end_time) = match dates.start_end_time() {
                Some((s, e)) => (Some(s), Some(e)),
                None => (None, None),
            };
            self.insert_start(start, start_time, index);
            self.insert_end(end, end_time, index);
        }
    }

    fn insert_f(&mut self, date: NaiveDate, f: impl FnOnce(&mut DayLayout)) {
        if let Some(l) = self.days.get_mut(&date) {
            f(l);
        }
    }

    fn insert_timed_f(&mut self, date: NaiveDate, time: Time, f: impl FnOnce(&mut TimedLayout)) {
        if let Some(l) = self.days.get_mut(&date) {
            let tl = l.timed.entry(time).or_insert_with(TimedLayout::new);
            f(tl);
        }
    }

    fn insert_start(&mut self, date: NaiveDate, time: Option<Time>, index: usize) {
        if let Some(time) = time {
            self.insert_timed_f(date, time, |tl| tl.starting.push(index));
        } else {
            self.insert_f(date, |dl| dl.starting.push(index));
        }
    }

    fn insert_at(&mut self, date: NaiveDate, time: Option<Time>, index: usize) {
        if let Some(time) = time {
            self.insert_timed_f(date, time, |tl| tl.at.push(index));
        } else {
            self.insert_f(date, |dl| dl.at.push(index));
        }
    }

    fn insert_end(&mut self, date: NaiveDate, time: Option<Time>, index: usize) {
        if let Some(time) = time {
            self.insert_timed_f(date, time, |tl| tl.ending.push(index));
        } else {
            self.insert_f(date, |dl| dl.ending.push(index));
        }
    }

    fn insert_other(&mut self, date: NaiveDate, index: usize) {
        self.insert_f(date, |dl| dl.other.push(index));
    }
}
