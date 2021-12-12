use std::collections::HashMap;

use crate::eval::{Entry, EntryKind};
use crate::files::commands::Command;
use crate::files::primitives::Time;
use crate::files::Files;

use super::layout::{Layout, LayoutEntry};

#[derive(Debug, Clone, Copy)]
enum SpanSegment {
    Start,
    Middle,
    End,
}

struct Line {
    number: Option<usize>,
    spans: Vec<Option<SpanSegment>>,
    time: Option<Time>,
    text: String,
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
            self.line(None, None, format!("{}", day));

            let layout_entries = layout.days.get(&day).expect("got nonexisting day");
            for layout_entry in layout_entries {
                self.render_layout_entry(files, entries, layout_entry);
            }
        }
    }

    pub fn display(&self) -> String {
        let mut result = String::new();

        let num_width = format!("{}", self.last_number).len();
        let span_width = self.spans.len();

        for line in &self.lines {
            let num = match line.number {
                Some(n) => format!("{}", n),
                None => "".to_string(),
            };

            let mut span = String::new();
            for s in &line.spans {
                span.push(match s {
                    Some(SpanSegment::Start) => '┏',
                    Some(SpanSegment::Middle) => '┃',
                    Some(SpanSegment::End) => '┗',
                    None => ' ',
                })
            }

            let time = match line.time {
                Some(t) => format!(" {}", t),
                None => "".to_string(),
            };

            result.push_str(&format!(
                "{:nw$} {:sw$}{} {}\n",
                num,
                span,
                time,
                line.text,
                nw = num_width,
                sw = span_width
            ))
        }

        result
    }

    fn render_layout_entry(&mut self, files: &Files, entries: &[Entry], l_entry: &LayoutEntry) {
        match l_entry {
            LayoutEntry::End(i) => {
                self.stop_span(*i);
                self.line(Some(*i), None, Self::format_entry(files, entries, *i));
            }
            LayoutEntry::Now(t) => self.line(None, Some(*t), "now".to_string()),
            LayoutEntry::TimedEnd(i, t) => {
                self.stop_span(*i);
                self.line(Some(*i), Some(*t), Self::format_entry(files, entries, *i));
            }
            LayoutEntry::TimedAt(i, t) => {
                self.line(Some(*i), Some(*t), Self::format_entry(files, entries, *i));
            }
            LayoutEntry::TimedStart(i, t) => {
                self.start_span(*i);
                self.line(Some(*i), Some(*t), Self::format_entry(files, entries, *i));
            }
            LayoutEntry::ReminderSince(i, d) => {
                let text = Self::format_entry(files, entries, *i);
                let text = if *d == 1 {
                    format!("{} (yesterday)", text)
                } else {
                    format!("{} ({} days ago)", text, d)
                };
                self.line(Some(*i), None, text);
            }
            LayoutEntry::At(i) => self.line(Some(*i), None, Self::format_entry(files, entries, *i)),
            LayoutEntry::ReminderWhile(i, d) => {
                let text = Self::format_entry(files, entries, *i);
                let plural = if *d == 1 { "" } else { "s" };
                let text = format!("{} ({} day{} left)", text, i, plural);
                self.line(Some(*i), None, text);
            }
            LayoutEntry::Undated(i) => {
                self.line(Some(*i), None, Self::format_entry(files, entries, *i));
            }
            LayoutEntry::Start(i) => {
                self.start_span(*i);
                self.line(Some(*i), None, Self::format_entry(files, entries, *i));
            }
            LayoutEntry::ReminderUntil(i, d) => {
                let text = Self::format_entry(files, entries, *i);
                let text = if *d == 1 {
                    format!("{} (tomorrow)", text)
                } else {
                    format!("{} (in {} days)", text, d)
                };
                self.line(Some(*i), None, text);
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

    fn line(&mut self, index: Option<usize>, time: Option<Time>, text: String) {
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

        let spans = self
            .spans
            .iter()
            .map(|span| span.as_ref().map(|(_, s)| *s))
            .collect();

        self.lines.push(Line {
            number,
            spans,
            time,
            text,
        });

        self.step_spans();
    }
}
