//! Organize a list of entries into a specified range of days.
//!
//! This includes adding reminders and ordering everything so it will be nicer
//! to display later.

use std::collections::HashMap;

use chrono::{NaiveDate, NaiveDateTime};

use crate::eval::{DateRange, Dates, Entry, EntryKind};
use crate::files::commands::Command;
use crate::files::primitives::Time;
use crate::files::Files;

#[derive(Debug)]
pub enum DayEntry {
    End(usize),
    Now(Time),
    TimedEnd(usize, Time),
    TimedAt(usize, Time, Option<Time>),
    TimedStart(usize, Time),
    ReminderSince(usize, i64),
    At(usize),
    ReminderWhile(usize, i64),
    Undated(usize),
    Start(usize),
    ReminderUntil(usize, i64),
}

#[derive(Debug)]
pub struct DayLayout {
    pub range: DateRange,
    pub today: NaiveDate,
    pub time: Time,
    /// Entries that are required to draw brackets correctly.
    pub earlier: Vec<DayEntry>,
    pub days: HashMap<NaiveDate, Vec<DayEntry>>,
}

impl DayLayout {
    pub fn new(range: DateRange, now: NaiveDateTime) -> Self {
        Self {
            range,
            today: now.date(),
            time: now.time().into(),
            earlier: vec![],
            days: range.days().map(|d| (d, vec![])).collect(),
        }
    }

    pub fn layout(&mut self, files: &Files, entries: &[Entry]) {
        self.insert(self.today, DayEntry::Now(self.time));

        let mut commands = entries
            .iter()
            .enumerate()
            .map(|(i, e)| (i, e, files.command(e.source)))
            .collect::<Vec<_>>();

        Self::sort_entries(&mut commands);

        for (index, entry, _) in commands {
            self.layout_entry(index, entry);
        }

        for (_, day) in self.days.iter_mut() {
            Self::sort_day(day);
        }

        self.combine_times();
    }

    fn layout_entry(&mut self, index: usize, entry: &Entry) {
        match entry.kind {
            EntryKind::Task => self.layout_task(index, entry),
            EntryKind::TaskDone(at) => self.layout_task_done(index, entry, at),
            EntryKind::Note | EntryKind::Birthday(_) => self.layout_note(index, entry),
        }
    }

    fn layout_task(&mut self, index: usize, entry: &Entry) {
        if let Some(dates) = entry.dates {
            let (start, end) = dates.start_end();
            if self.today < start && (start - self.today).num_days() < 7 {
                // TODO Make this adjustable, maybe even per-command
                let days = (start - self.today).num_days();
                self.insert(self.today, DayEntry::ReminderUntil(index, days));
            } else if start < self.today && self.today < end {
                let days = (end - self.today).num_days();
                self.insert(self.today, DayEntry::ReminderWhile(index, days));
            } else if end < self.today {
                let days = (self.today - end).num_days();
                self.insert(self.today, DayEntry::ReminderSince(index, days));
            }
            self.layout_dated_entry(index, dates);
        } else {
            self.insert(self.today, DayEntry::Undated(index));
        }
    }

    fn layout_task_done(&mut self, index: usize, entry: &Entry, at: NaiveDate) {
        if let Some(dates) = entry.dates {
            if at > dates.end() {
                let days = (at - dates.end()).num_days();
                self.insert(at, DayEntry::ReminderSince(index, days));
            }
            self.layout_dated_entry(index, dates);
        } else {
            // Treat the task as if its date was its completion time
            self.layout_dated_entry(index, Dates::new(at, at));
        }
    }

    fn layout_note(&mut self, index: usize, entry: &Entry) {
        if let Some(dates) = entry.dates {
            let (start, end) = dates.start_end();
            if start < self.range.from() && self.range.until() < end {
                // This note applies to the current day, but it won't appear if
                // we just layout it as a dated entry, so instead we add it as a
                // reminder. Since we are usually more interested in when
                // something ends than when it starts, we count the days until
                // the end.
                let days = (end - self.today).num_days();
                self.insert(self.today, DayEntry::ReminderWhile(index, days));
            } else {
                self.layout_dated_entry(index, dates);
            }
        } else {
            self.insert(self.today, DayEntry::Undated(index));
        }
    }

    fn layout_dated_entry(&mut self, index: usize, dates: Dates) {
        let (start, end) = dates.start_end();
        #[allow(clippy::if_same_then_else)] // Makes the code easier to read
        if let Some((date, time)) = dates.point_in_time() {
            let entry = match time {
                Some(time) => DayEntry::TimedAt(index, time, None),
                None => DayEntry::At(index),
            };
            self.insert(date, entry);
        } else if end < self.range.from() || self.range.until() < start {
            // Since the entry is entirely outside the range, there is no point
            // in adding the start or end entry anywhere. If the entry was
            // entirely before the range and we didn't filter it out, we would
            // get a bracket without any visible start or end.
        } else if start < self.range.from() && self.range.until() < end {
            // Since neither the start nor end entries are visible directly, we
            // omit them both. Otherwise, we would get a bracket without any
            // visible start or end.
        } else {
            let (start_entry, end_entry) = match dates.start_end_time() {
                Some((start_time, end_time)) => (
                    DayEntry::TimedStart(index, start_time),
                    DayEntry::TimedEnd(index, end_time),
                ),
                None => (DayEntry::Start(index), DayEntry::End(index)),
            };
            self.insert(start, start_entry);
            self.insert(end, end_entry);
        }
    }

    fn insert(&mut self, date: NaiveDate, e: DayEntry) {
        if date < self.range.from() {
            self.earlier.push(e);
        } else if let Some(es) = self.days.get_mut(&date) {
            match e {
                // Insert end entries in reverse order so that the brackets
                // connecting them to their start entry look nicer and less
                // convoluted.
                DayEntry::End(_) | DayEntry::TimedEnd(_, _) => es.insert(0, e),
                _ => es.push(e),
            }
        }
    }

    fn sort_entries(entries: &mut Vec<(usize, &Entry, &Command)>) {
        // Entries should be sorted by these factors, in descending order of
        // significance:
        // 1. Their start date, if any
        // 2. Their end date in reverse, if any
        // 3. Their kind
        // 4. Their title

        // 4.
        entries.sort_by_key(|(_, _, c)| c.title());

        // 3.
        entries.sort_by_key(|(_, e, _)| match e.kind {
            EntryKind::Task => 0,
            EntryKind::TaskDone(_) => 1,
            EntryKind::Birthday(_) => 2,
            EntryKind::Note => 3,
        });

        // 2.
        entries.sort_by(|(_, e1, _), (_, e2, _)| {
            let d1 = e1.dates.map(|d| (d.end(), d.end_time()));
            let d2 = e2.dates.map(|d| (d.end(), d.end_time()));
            d2.cmp(&d1)
        });

        // 1.
        entries.sort_by_key(|(_, e, _)| e.dates.map(|d| (d.start(), d.start_time())));
    }

    fn sort_day(day: &mut Vec<DayEntry>) {
        // In a day, entries should be sorted into these categories:
        // 1. Untimed entries that end at the current day
        // 2. Timed entries, based on
        //   2.1. Their time
        //   2.2. Their type (ending, at, starting)
        // 3. Reminders for overdue entries
        // 4. Untimed entries occurring today
        // 5. Reminders for entries ending soon
        // 6. Undated entries occurring today
        // 7. Untimed entries starting today
        // 8. Reminders for entries starting soon
        //
        // Entries within a single category should already be ordered based on
        // their kind and title since the order they are layouted in takes these
        // into account.

        // Ensure timed entries for a single time occur in the correct order
        day.sort_by_key(|e| match e {
            DayEntry::Now(_) => 1,
            DayEntry::TimedEnd(_, _) => 2,
            DayEntry::TimedAt(_, _, _) => 3,
            DayEntry::TimedStart(_, _) => 4,
            _ => 0,
        });

        // Ensure timed entries for different times occur in the correct order
        day.sort_by_key(|e| match e {
            DayEntry::Now(t) => Some(*t),
            DayEntry::TimedEnd(_, t) => Some(*t),
            DayEntry::TimedAt(_, t, _) => Some(*t),
            DayEntry::TimedStart(_, t) => Some(*t),
            _ => None,
        });

        // Ensure categories occur in the correct order
        day.sort_by_key(|e| match e {
            DayEntry::End(_) => 0,
            DayEntry::Now(_) => 1,
            DayEntry::TimedEnd(_, _) => 1,
            DayEntry::TimedAt(_, _, _) => 1,
            DayEntry::TimedStart(_, _) => 1,
            DayEntry::ReminderSince(_, _) => 2,
            DayEntry::At(_) => 3,
            DayEntry::ReminderWhile(_, _) => 4,
            DayEntry::Undated(_) => 5,
            DayEntry::Start(_) => 6,
            DayEntry::ReminderUntil(_, _) => 7,
        })
    }

    /// Combine matching [`DayEntry::TimedStart`]s and [`DayEntry::TimedEnd`]s
    /// with nothing in-between into [`DayEntry::TimedAt`]s.
    fn combine_times(&mut self) {
        for day in self.days.values_mut() {
            let mut i = 0;
            while i + 1 < day.len() {
                match (&day[i], &day[i + 1]) {
                    (DayEntry::TimedStart(i1, t1), DayEntry::TimedEnd(i2, t2)) if i1 == i2 => {
                        day[i] = DayEntry::TimedAt(*i1, *t1, Some(*t2));
                        day.remove(i + 1);
                    }
                    _ => {}
                }
                i += 1;
            }
        }
    }
}
