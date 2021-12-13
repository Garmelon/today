//! Organize layouted entries into a list of lines to display.
//!
//! Additional information, such as the mapping from numbers to entries, are
//! collected along the way. The lines are not yet rendered into strings.

use std::collections::HashMap;

use chrono::NaiveDate;

use crate::eval::{Entry, EntryKind};
use crate::files::primitives::Time;
use crate::files::Files;

use super::super::error::{Error, Result};
use super::day::{DayEntry, DayLayout};

#[derive(Debug, Clone, Copy)]
pub enum SpanSegment {
    Start,
    Middle,
    End,
}

pub enum LineEntry {
    Day {
        spans: Vec<Option<SpanSegment>>,
        date: NaiveDate,
    },
    Now {
        spans: Vec<Option<SpanSegment>>,
        time: Time,
    },
    Entry {
        number: Option<usize>,
        spans: Vec<Option<SpanSegment>>,
        time: Option<Time>,
        text: String,
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

    pub fn render(&mut self, files: &Files, entries: &[Entry], layout: &DayLayout) {
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
            self.line(LineEntry::Day { spans, date: day });

            let layout_entries = layout.days.get(&day).expect("got nonexisting day");
            for layout_entry in layout_entries {
                self.render_layout_entry(files, entries, layout_entry);
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

    /// Return a map from entry indices to their corresponding display numbers.
    ///
    /// If you need to resolve a display number into an entry index, use
    /// [`look_up_number`] instead.
    pub fn numbers(&self) -> &HashMap<usize, usize> {
        &self.numbers
    }

    pub fn look_up_number(&self, number: usize) -> Result<usize> {
        self.numbers
            .iter()
            .filter(|(_, n)| **n == number)
            .map(|(i, _)| *i)
            .next()
            .ok_or_else(|| Error::NoSuchEntry(number))
    }

    fn render_layout_entry(&mut self, files: &Files, entries: &[Entry], l_entry: &DayEntry) {
        match l_entry {
            DayEntry::End(i) => {
                self.stop_span(*i);
                self.line_entry(Some(*i), None, Self::format_entry(files, entries, *i));
            }
            DayEntry::Now(t) => self.line(LineEntry::Now {
                spans: self.spans_for_line(),
                time: *t,
            }),
            DayEntry::TimedEnd(i, t) => {
                self.stop_span(*i);
                self.line_entry(Some(*i), Some(*t), Self::format_entry(files, entries, *i));
            }
            DayEntry::TimedAt(i, t) => {
                self.line_entry(Some(*i), Some(*t), Self::format_entry(files, entries, *i));
            }
            DayEntry::TimedStart(i, t) => {
                self.start_span(*i);
                self.line_entry(Some(*i), Some(*t), Self::format_entry(files, entries, *i));
            }
            DayEntry::ReminderSince(i, d) => {
                let text = Self::format_entry(files, entries, *i);
                let text = if *d == 1 {
                    format!("{} (yesterday)", text)
                } else {
                    format!("{} ({} days ago)", text, d)
                };
                self.line_entry(Some(*i), None, text);
            }
            DayEntry::At(i) => {
                self.line_entry(Some(*i), None, Self::format_entry(files, entries, *i))
            }
            DayEntry::ReminderWhile(i, d) => {
                let text = Self::format_entry(files, entries, *i);
                let plural = if *d == 1 { "" } else { "s" };
                let text = format!("{} ({} day{} left)", text, i, plural);
                self.line_entry(Some(*i), None, text);
            }
            DayEntry::Undated(i) => {
                self.line_entry(Some(*i), None, Self::format_entry(files, entries, *i));
            }
            DayEntry::Start(i) => {
                self.start_span(*i);
                self.line_entry(Some(*i), None, Self::format_entry(files, entries, *i));
            }
            DayEntry::ReminderUntil(i, d) => {
                let text = Self::format_entry(files, entries, *i);
                let text = if *d == 1 {
                    format!("{} (tomorrow)", text)
                } else {
                    format!("{} (in {} days)", text, d)
                };
                self.line_entry(Some(*i), None, text);
            }
        }
    }

    fn format_entry(files: &Files, entries: &[Entry], index: usize) -> String {
        let entry = entries[index];
        let command = files.command(entry.source);
        match entry.kind {
            EntryKind::Task => format!("T {}", command.title()),
            EntryKind::TaskDone(_) => format!("D {}", command.title()),
            EntryKind::Note => format!("N {}", command.title()),
            EntryKind::Birthday(Some(age)) => format!("B {} ({})", command.title(), age),
            EntryKind::Birthday(None) => format!("B {}", command.title()),
        }
    }

    fn start_span(&mut self, index: usize) {
        for span in self.spans.iter_mut() {
            if span.is_none() {
                *span = Some((index, SpanSegment::Start));
                return;
            }
        }

        // Not enough space, we need another column
        self.spans.push(Some((index, SpanSegment::Start)));
    }

    fn stop_span(&mut self, index: usize) {
        for span in self.spans.iter_mut() {
            match span {
                Some((i, s)) if *i == index => *s = SpanSegment::End,
                _ => {}
            }
        }
    }

    fn step_spans(&mut self) {
        for span in self.spans.iter_mut() {
            match span {
                Some((_, s @ SpanSegment::Start)) => *s = SpanSegment::Middle,
                Some((_, SpanSegment::End)) => *span = None,
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

    fn line_entry(&mut self, index: Option<usize>, time: Option<Time>, text: String) {
        let number = match index {
            Some(index) => Some(match self.numbers.get(&index) {
                Some(number) => *number,
                None => {
                    self.last_number += 1;
                    self.numbers.insert(index, self.last_number);
                    self.last_number
                }
            }),
            None => None,
        };

        self.line(LineEntry::Entry {
            number,
            spans: self.spans_for_line(),
            time,
            text,
        });
    }
}
