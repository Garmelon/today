use std::cmp;

use chrono::{Datelike, NaiveDate};
use colored::{ColoredString, Colorize};

use crate::files::primitives::{Time, Weekday};

use super::layout::line::{LineEntry, LineKind, LineLayout, SpanSegment, SpanStyle, Times};

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
            LineEntry::Day {
                spans,
                date,
                today,
                has_log,
            } => self.display_line_date(spans, *date, *today, *has_log),
            LineEntry::Now { spans, time } => self.display_line_now(spans, *time),
            LineEntry::Entry {
                number,
                spans,
                time,
                kind,
                text,
                has_desc,
                extra,
            } => self.display_line_entry(*number, spans, *time, *kind, text, *has_desc, extra),
        }
    }

    fn display_line_date(
        &mut self,
        spans: &[Option<SpanSegment>],
        date: NaiveDate,
        today: bool,
        has_log: bool,
    ) {
        let weekday: Weekday = date.weekday().into();
        let weekday = weekday.full_name();

        let styled = |s: &str| {
            if today {
                s.bright_cyan().bold()
            } else {
                s.cyan()
            }
        };

        // '=' symbols before the spans start
        let p1 = format!("{:=<w$}=", "", w = self.num_width);

        // Spans and filler '=' symbols
        let p2 = self.display_spans(spans, styled("="));

        // The rest of the line until after the date
        let p3 = format!("===  {:9}  {}", weekday, date,);

        // The "has log" marker (if any)
        let p4 = Self::display_marker(has_log, " ");

        // The rest of the line
        let p5 = format!(" ===={:=<w$}", "", w = self.num_width + self.span_width);

        self.push(&format!(
            "{}{}{}{}{}\n",
            styled(&p1),
            p2,
            styled(&p3),
            p4,
            styled(&p5)
        ));
    }

    fn display_line_now(&mut self, spans: &[Option<SpanSegment>], time: Time) {
        self.push(&format!(
            "{:>nw$} {}  {}\n",
            "now".bright_cyan().bold(),
            self.display_spans(spans, " ".into()),
            Self::display_time(Times::At(time)),
            nw = self.num_width,
        ));
    }

    fn display_line_entry(
        &mut self,
        number: Option<usize>,
        spans: &[Option<SpanSegment>],
        time: Times,
        kind: LineKind,
        text: &str,
        has_desc: bool,
        extra: &Option<String>,
    ) {
        let num = match number {
            Some(n) => format!("{}", n),
            None => "".to_string(),
        };

        self.push(&format!(
            "{:>nw$} {} {}{} {}{}{}\n",
            num.bright_black(),
            self.display_spans(spans, " ".into()),
            Self::display_kind(kind),
            Self::display_time(time),
            text,
            Self::display_marker(has_desc, ""),
            Self::display_extra(extra),
            nw = self.num_width,
        ))
    }

    fn display_spans(&self, spans: &[Option<SpanSegment>], empty: ColoredString) -> String {
        let mut result = String::new();
        for i in 0..self.span_width {
            if let Some(Some(segment)) = spans.get(i) {
                let colored_str = match segment {
                    SpanSegment::Start(_) => "┌".bright_black(),
                    SpanSegment::Middle(SpanStyle::Solid) => "│".bright_black(),
                    SpanSegment::Middle(SpanStyle::Dashed) => "╎".bright_black(),
                    SpanSegment::Middle(SpanStyle::Dotted) => "┊".bright_black(),
                    SpanSegment::End(_) => "└".bright_black(),
                };
                result.push_str(&format!("{}", colored_str));
            } else {
                result.push_str(&format!("{}", empty));
            }
        }
        result
    }

    fn display_time(time: Times) -> ColoredString {
        match time {
            Times::Untimed => "".into(),
            Times::At(t) => format!(" {}", t).bright_black(),
            Times::FromTo(t1, t2) => format!(" {}--{}", t1, t2).bright_black(),
        }
    }

    fn display_kind(kind: LineKind) -> ColoredString {
        match kind {
            LineKind::Task => "T".magenta().bold(),
            LineKind::Done => "D".green().bold(),
            LineKind::Canceled => "C".red().bold(),
            LineKind::Note => "N".blue().bold(),
            LineKind::Birthday => "B".yellow().bold(),
        }
    }

    fn display_marker(marker: bool, otherwise: &str) -> ColoredString {
        if marker {
            "*".bright_yellow()
        } else {
            otherwise.into()
        }
    }

    fn display_extra(extra: &Option<String>) -> ColoredString {
        match extra {
            None => "".into(),
            Some(extra) => format!(" ({})", extra).bright_black(),
        }
    }

    fn push(&mut self, line: &str) {
        self.result.push_str(line);
    }

    fn result(self) -> String {
        self.result
    }
}

pub fn print(layout: &LineLayout) {
    let num_width = cmp::max(layout.num_width(), 3); // `now` is 3 chars wide
    let mut show_lines = ShowLines::new(num_width, layout.span_width());
    for line in layout.lines() {
        show_lines.display_line(line);
    }
    print!("{}", show_lines.result());
}
