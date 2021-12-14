use std::cmp;

use chrono::{Datelike, NaiveDate};

use crate::eval::{Entry, EntryKind};
use crate::files::primitives::{Time, Weekday};
use crate::files::Files;

use super::error::Result;
use super::layout::line::{LineEntry, LineLayout, SpanSegment, Times};

struct ShowLines {
    num_width: usize,
    span_width: usize,
    result: String,
}

impl ShowLines {
    fn new(num_width: usize, span_width: usize) -> Self {
        Self {
            num_width,
            span_width,
            result: String::new(),
        }
    }

    fn display_line(&mut self, line: &LineEntry) {
        match line {
            LineEntry::Day { spans, date } => self.display_line_date(spans, *date),
            LineEntry::Now { spans, time } => self.display_line_now(spans, *time),
            LineEntry::Entry {
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
            "{:<nw$} {:sw$} {}\n",
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
        time: Times,
        text: &str,
    ) {
        let num = match number {
            Some(n) => format!("{}", n),
            None => "".to_string(),
        };

        let time = match time {
            Times::Untimed => "".to_string(),
            Times::At(t) => format!("{} ", t),
            Times::FromTo(t1, t2) => format!("{}--{} ", t1, t2),
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

pub fn show_all(layout: &LineLayout) {
    let num_width = cmp::max(layout.num_width(), 3); // `now` is 3 chars wide
    let mut show_lines = ShowLines::new(num_width, layout.span_width());
    for line in layout.lines() {
        show_lines.display_line(line);
    }
    print!("{}", show_lines.result());
}

pub fn show_entry(files: &Files, entry: &Entry) -> Result<()> {
    let command = files.command(entry.source);

    match entry.kind {
        EntryKind::Task => println!("TASK {}", command.title()),
        EntryKind::TaskDone(when) => {
            println!("DONE {}", command.title());
            println!("DONE AT {}", when);
        }
        EntryKind::Note => println!("NOTE {}", command.title()),
        EntryKind::Birthday(Some(age)) => {
            println!("BIRTHDAY {}", command.title());
            println!("AGE {}", age);
        }
        EntryKind::Birthday(None) => {
            println!("BIRTHDAY {}", command.title());
            println!("AGE UNKNOWN");
        }
    }

    if let Some(dates) = entry.dates {
        let (start, end) = dates.start_end();
        if start == end {
            match dates.start_end_time() {
                Some((s, e)) if s == e => println!("DATE {} {}", start, s),
                Some((s, e)) => println!("DATE {} {} -- {}", start, s, e),
                None => println!("DATE {}", start),
            }
        } else {
            match dates.start_end_time() {
                Some((s, e)) => println!("DATE {} {} -- {} {}", start, s, end, e),
                None => println!("DATE {} -- {}", start, end),
            }
        }
    } else {
        println!("NO DATE");
    };

    for line in command.desc() {
        println!("# {}", line);
    }

    Ok(())
}
