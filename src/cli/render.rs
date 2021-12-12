use std::cmp;
use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};

use crate::eval::{Entry, EntryKind};
use crate::files::primitives::{Time, Weekday};
use crate::files::Files;

use super::layout::{Layout, LayoutEntry};

#[derive(Debug, Clone, Copy)]
enum SpanSegment {
    Start,
    Middle,
    End,
}

enum Line {
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

pub struct Render {
    numbers: HashMap<usize, usize>,
    last_number: usize,
    spans: Vec<Option<(usize, SpanSegment)>>,
    lines: Vec<Line>,
}

impl Render {
    pub fn new() -> Self {
        Self {
            numbers: HashMap::new(),
            last_number: 0,
            spans: vec![],
            lines: vec![],
        }
    }

    pub fn render(&mut self, files: &Files, entries: &[Entry], layout: &Layout) {
        // Make sure spans for visible `*End`s are drawn
        for entry in &layout.earlier {
            match entry {
                LayoutEntry::TimedStart(i, _) | LayoutEntry::Start(i) => self.start_span(*i),
                _ => {}
            }
        }
        self.step_spans();

        for day in layout.range.days() {
            let spans = self.spans_for_line();
            self.line(Line::Day { spans, date: day });

            let layout_entries = layout.days.get(&day).expect("got nonexisting day");
            for layout_entry in layout_entries {
                self.render_layout_entry(files, entries, layout_entry);
            }
        }
    }

    pub fn display(&self) -> String {
        let num_width = format!("{}", self.last_number).len();
        let num_width = cmp::max(num_width, 3); // for a "now" in the first column
        let span_width = self.spans.len();

        let mut ctx = DisplayContext::new(num_width, span_width);
        for line in &self.lines {
            ctx.display_line(line);
        }

        ctx.result()
    }

    fn render_layout_entry(&mut self, files: &Files, entries: &[Entry], l_entry: &LayoutEntry) {
        match l_entry {
            LayoutEntry::End(i) => {
                self.stop_span(*i);
                self.line_entry(Some(*i), None, Self::format_entry(files, entries, *i));
            }
            LayoutEntry::Now(t) => self.line(Line::Now {
                spans: self.spans_for_line(),
                time: *t,
            }),
            LayoutEntry::TimedEnd(i, t) => {
                self.stop_span(*i);
                self.line_entry(Some(*i), Some(*t), Self::format_entry(files, entries, *i));
            }
            LayoutEntry::TimedAt(i, t) => {
                self.line_entry(Some(*i), Some(*t), Self::format_entry(files, entries, *i));
            }
            LayoutEntry::TimedStart(i, t) => {
                self.start_span(*i);
                self.line_entry(Some(*i), Some(*t), Self::format_entry(files, entries, *i));
            }
            LayoutEntry::ReminderSince(i, d) => {
                let text = Self::format_entry(files, entries, *i);
                let text = if *d == 1 {
                    format!("{} (yesterday)", text)
                } else {
                    format!("{} ({} days ago)", text, d)
                };
                self.line_entry(Some(*i), None, text);
            }
            LayoutEntry::At(i) => {
                self.line_entry(Some(*i), None, Self::format_entry(files, entries, *i))
            }
            LayoutEntry::ReminderWhile(i, d) => {
                let text = Self::format_entry(files, entries, *i);
                let plural = if *d == 1 { "" } else { "s" };
                let text = format!("{} ({} day{} left)", text, i, plural);
                self.line_entry(Some(*i), None, text);
            }
            LayoutEntry::Undated(i) => {
                self.line_entry(Some(*i), None, Self::format_entry(files, entries, *i));
            }
            LayoutEntry::Start(i) => {
                self.start_span(*i);
                self.line_entry(Some(*i), None, Self::format_entry(files, entries, *i));
            }
            LayoutEntry::ReminderUntil(i, d) => {
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

    fn line(&mut self, line: Line) {
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

        self.line(Line::Entry {
            number,
            spans: self.spans_for_line(),
            time,
            text,
        });
    }
}

struct DisplayContext {
    num_width: usize,
    span_width: usize,
    result: String,
}

impl DisplayContext {
    fn new(num_width: usize, span_width: usize) -> Self {
        Self {
            num_width,
            span_width,
            result: String::new(),
        }
    }

    fn display_line(&mut self, line: &Line) {
        match line {
            Line::Day { spans, date } => self.display_line_date(spans, *date),
            Line::Now { spans, time } => self.display_line_now(spans, *time),
            Line::Entry {
                number,
                spans,
                time,
                text,
            } => self.display_line_entry(*number, spans, *time, text),
        }
    }

    fn display_line_date(&mut self, spans: &[Option<SpanSegment>], date: NaiveDate) {
        let weekday: Weekday = date.weekday().into();
        let weekday = weekday.full_name();
        self.push(&format!(
            "{:=>nw$}={:=<sw$}===  {:9}  {}  ==={:=<sw$}={:=>nw$}\n",
            "",
            Self::display_spans(spans, '='),
            weekday,
            date,
            "",
            "",
            nw = self.num_width,
            sw = self.span_width
        ));
    }

    fn display_line_now(&mut self, spans: &[Option<SpanSegment>], time: Time) {
        self.push(&format!(
            "{:>nw$} {:sw$} {}\n",
            "now",
            Self::display_spans(spans, ' '),
            time,
            nw = self.num_width,
            sw = self.span_width
        ));
    }

    fn display_line_entry(
        &mut self,
        number: Option<usize>,
        spans: &[Option<SpanSegment>],
        time: Option<Time>,
        text: &str,
    ) {
        let num = match number {
            Some(n) => format!("{}", n),
            None => "".to_string(),
        };

        let time = match time {
            Some(t) => format!("{} ", t),
            None => "".to_string(),
        };

        self.push(&format!(
            "{:>nw$} {:sw$} {}{}\n",
            num,
            Self::display_spans(spans, ' '),
            time,
            text,
            nw = self.num_width,
            sw = self.span_width
        ))
    }

    fn display_spans(spans: &[Option<SpanSegment>], empty: char) -> String {
        let mut result = String::new();
        for segment in spans {
            result.push(match segment {
                Some(SpanSegment::Start) => '┌',
                Some(SpanSegment::Middle) => '│',
                Some(SpanSegment::End) => '└',
                None => empty,
            });
        }
        result
    }

    fn push(&mut self, line: &str) {
        self.result.push_str(line);
    }

    fn result(self) -> String {
        self.result
    }
}
