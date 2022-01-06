//! Organize layouted entries into a list of lines to display.
//!
//! Additional information, such as the mapping from numbers to entries, are
//! collected along the way. The lines are not yet rendered into strings.

use std::collections::HashMap;

use chrono::NaiveDate;

use crate::eval::{Entry, EntryKind};
use crate::files::primitives::Time;

use super::super::error::Error;
use super::day::{DayEntry, DayLayout};

#[derive(Debug, Clone, Copy)]
pub enum SpanStyle {
    Solid,
    Dashed,
    Dotted,
}

impl SpanStyle {
    fn from_indentation(index: usize) -> Self {
        match index % 3 {
            0 => Self::Solid,
            1 => Self::Dashed,
            2 => Self::Dotted,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpanSegment {
    Start(SpanStyle),
    Middle(SpanStyle),
    End(SpanStyle),
}

impl SpanSegment {
    fn style(&self) -> SpanStyle {
        match self {
            SpanSegment::Start(s) => *s,
            SpanSegment::Middle(s) => *s,
            SpanSegment::End(s) => *s,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Times {
    Untimed,
    At(Time),
    FromTo(Time, Time),
}

#[derive(Debug, Clone, Copy)]
pub enum LineKind {
    Task,
    Done,
    Canceled,
    Note,
    Birthday,
}

pub enum LineEntry {
    Day {
        spans: Vec<Option<SpanSegment>>,
        date: NaiveDate,
        today: bool,
    },
    Now {
        spans: Vec<Option<SpanSegment>>,
        time: Time,
    },
    Entry {
        number: Option<usize>,
        spans: Vec<Option<SpanSegment>>,
        time: Times,
        kind: LineKind,
        text: String,
        extra: Option<String>,
    },
}

pub struct LineLayout {
    /// Map from entry indices to their corresponding display numbers.
    ///
    /// Display numbers start at 1, not 0.
    numbers: HashMap<usize, usize>,
    /// The last number that was used as a display number.
    ///
    /// Is set to 0 initially, which is fine since display numbers start at 1.
    last_number: usize,
    spans: Vec<Option<(usize, SpanSegment)>>,
    lines: Vec<LineEntry>,
}

impl LineLayout {
    pub fn new() -> Self {
        Self {
            numbers: HashMap::new(),
            last_number: 0,
            spans: vec![],
            lines: vec![],
        }
    }

    pub fn render(&mut self, entries: &[Entry], layout: &DayLayout) {
        // Make sure spans for visible `*End`s are drawn
        for entry in &layout.earlier {
            match entry {
                DayEntry::TimedStart(i, _) | DayEntry::Start(i) => self.start_span(*i),
                _ => {}
            }
        }
        self.step_spans();

        for day in layout.range.days() {
            let spans = self.spans_for_line();
            self.line(LineEntry::Day {
                spans,
                date: day,
                today: day == layout.today,
            });

            let layout_entries = layout.days.get(&day).expect("got nonexisting day");
            for layout_entry in layout_entries {
                self.render_layout_entry(entries, layout_entry);
            }
        }
    }

    pub fn num_width(&self) -> usize {
        format!("{}", self.last_number).len()
    }

    pub fn span_width(&self) -> usize {
        self.spans.len()
    }

    pub fn lines(&self) -> &[LineEntry] {
        &self.lines
    }

    pub fn look_up_number<S>(&self, number: usize) -> Result<usize, Error<S>> {
        self.numbers
            .iter()
            .filter(|(_, n)| **n == number)
            .map(|(i, _)| *i)
            .next()
            .ok_or(Error::NoSuchEntry(number))
    }

    fn render_layout_entry(&mut self, entries: &[Entry], l_entry: &DayEntry) {
        match l_entry {
            DayEntry::End(i) => {
                self.stop_span(*i);
                self.line_entry(entries, *i, Times::Untimed, None);
            }
            DayEntry::Now(t) => self.line(LineEntry::Now {
                spans: self.spans_for_line(),
                time: *t,
            }),
            DayEntry::TimedEnd(i, t) => {
                self.stop_span(*i);
                self.line_entry(entries, *i, Times::At(*t), None);
            }
            DayEntry::TimedAt(i, t, t2) => {
                let time = t2
                    .map(|t2| Times::FromTo(*t, t2))
                    .unwrap_or_else(|| Times::At(*t));
                self.line_entry(entries, *i, time, None);
            }
            DayEntry::TimedStart(i, t) => {
                self.start_span(*i);
                self.line_entry(entries, *i, Times::At(*t), None);
            }
            DayEntry::ReminderSince(i, d) => {
                let extra = if *d == 1 {
                    "yesterday".to_string()
                } else {
                    format!("{} days ago", d)
                };
                self.line_entry(entries, *i, Times::Untimed, Some(extra));
            }
            DayEntry::At(i) => {
                self.line_entry(entries, *i, Times::Untimed, None);
            }
            DayEntry::ReminderWhile(i, d) => {
                let plural = if *d == 1 { "" } else { "s" };
                let extra = format!("{} day{} left", d, plural);
                self.line_entry(entries, *i, Times::Untimed, Some(extra));
            }
            DayEntry::Undated(i) => {
                self.line_entry(entries, *i, Times::Untimed, None);
            }
            DayEntry::Start(i) => {
                self.start_span(*i);
                self.line_entry(entries, *i, Times::Untimed, None);
            }
            DayEntry::ReminderUntil(i, d) => {
                let extra = if *d == 1 {
                    "tomorrow".to_string()
                } else {
                    format!("in {} days", d)
                };
                self.line_entry(entries, *i, Times::Untimed, Some(extra));
            }
        }
    }

    fn entry_kind(entry: &Entry) -> LineKind {
        match entry.kind {
            EntryKind::Task => LineKind::Task,
            EntryKind::TaskDone(_) => LineKind::Done,
            EntryKind::TaskCanceled(_) => LineKind::Canceled,
            EntryKind::Note => LineKind::Note,
            EntryKind::Birthday(_) => LineKind::Birthday,
        }
    }

    fn entry_title(entry: &Entry) -> String {
        match entry.kind {
            EntryKind::Birthday(Some(age)) => format!("{} ({})", entry.title, age),
            _ => entry.title.clone(),
        }
    }

    fn start_span(&mut self, index: usize) {
        for (i, span) in self.spans.iter_mut().enumerate() {
            if span.is_none() {
                let style = SpanStyle::from_indentation(i);
                *span = Some((index, SpanSegment::Start(style)));
                return;
            }
        }

        // Not enough space, we need another column
        let style = SpanStyle::from_indentation(self.spans.len());
        self.spans.push(Some((index, SpanSegment::Start(style))));
    }

    fn stop_span(&mut self, index: usize) {
        for span in self.spans.iter_mut() {
            match span {
                Some((i, s)) if *i == index => *s = SpanSegment::End(s.style()),
                _ => {}
            }
        }
    }

    fn step_spans(&mut self) {
        for span in self.spans.iter_mut() {
            match span {
                Some((_, s @ SpanSegment::Start(_))) => *s = SpanSegment::Middle(s.style()),
                Some((_, SpanSegment::End(_))) => *span = None,
                _ => {}
            }
        }
    }

    fn spans_for_line(&self) -> Vec<Option<SpanSegment>> {
        self.spans
            .iter()
            .map(|span| span.as_ref().map(|(_, s)| *s))
            .collect()
    }

    fn line(&mut self, line: LineEntry) {
        self.lines.push(line);
        self.step_spans();
    }

    fn line_entry(&mut self, entries: &[Entry], index: usize, time: Times, extra: Option<String>) {
        let entry = &entries[index];

        let number = match self.numbers.get(&index) {
            Some(number) => *number,
            None => {
                self.last_number += 1;
                self.numbers.insert(index, self.last_number);
                self.last_number
            }
        };

        self.line(LineEntry::Entry {
            number: Some(number),
            spans: self.spans_for_line(),
            time,
            kind: Self::entry_kind(entry),
            text: Self::entry_title(entry),
            extra,
        });
    }
}
