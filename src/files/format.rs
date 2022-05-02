use std::collections::HashSet;
use std::fmt;

use chrono::Datelike;

use crate::files::commands::DoneKind;

use super::commands::{
    BirthdaySpec, Command, DateSpec, Delta, DeltaStep, Done, DoneDate, Expr, File, FormulaSpec,
    Log, Note, Repeat, Spec, Statement, Task, Var, WeekdaySpec,
};
use super::primitives::{Spanned, Time, Weekday};

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

fn format_desc(f: &mut fmt::Formatter<'_>, desc: &[String]) -> fmt::Result {
    for line in desc {
        if line.is_empty() {
            writeln!(f, "#")?;
        } else {
            writeln!(f, "# {line}")?;
        }
    }
    Ok(())
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.min)
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

fn format_delta_step(f: &mut fmt::Formatter<'_>, step: &DeltaStep, sign: &mut i32) -> fmt::Result {
    let amount = step.amount();
    if *sign == 0 || (amount != 0 && *sign != amount.signum()) {
        write!(f, "{}", if amount >= 0 { "+" } else { "-" })?;
    }
    *sign = if amount >= 0 { 1 } else { -1 };
    if amount.abs() != 1 {
        write!(f, "{}", amount.abs())?;
    }
    write!(f, "{}", step.name())
}

impl fmt::Display for Delta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sign = 0;
        for step in &self.0 {
            format_delta_step(f, &step.value, &mut sign)?;
        }
        Ok(())
    }
}

impl fmt::Display for Repeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start_at_done {
            write!(f, "done ")?;
        }
        write!(f, "{}", self.delta)
    }
}

impl fmt::Display for DateSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Start
        write!(f, "{}", self.start)?;
        for delta in &self.start_delta {
            write!(f, " {delta}")?;
        }
        for time in &self.start_time {
            write!(f, " {time}")?;
        }

        // End
        if self.end.is_some() || self.end_delta.is_some() || self.end_time.is_some() {
            write!(f, " --")?;
            if let Some(date) = self.end {
                write!(f, " {date}")?;
            }
            if let Some(delta) = &self.end_delta {
                write!(f, " {delta}")?;
            }
            if let Some(time) = &self.end_time {
                write!(f, " {time}")?;
            }
        }

        // Repeat
        if let Some(repeat) = &self.repeat {
            write!(f, "; {repeat}")?;
        }

        Ok(())
    }
}

impl fmt::Display for WeekdaySpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Start
        write!(f, "{}", self.start)?;
        for time in &self.start_time {
            write!(f, " {time}")?;
        }

        // End
        if self.end.is_some() || self.end_delta.is_some() || self.end_time.is_some() {
            write!(f, " --")?;
            if let Some(wd) = self.end {
                write!(f, " {wd}")?;
            }
            if let Some(delta) = &self.end_delta {
                write!(f, " {delta}")?;
            }
            if let Some(time) = &self.end_time {
                write!(f, " {time}")?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Lit(i) => write!(f, "{i}"),
            Expr::Var(v) => write!(f, "{v}"),
            Expr::Paren(e) => write!(f, "({e})"),
            Expr::Neg(e) => write!(f, "-{e}"),
            Expr::Add(a, b) => write!(f, "{a} + {b}"),
            Expr::Sub(a, b) => write!(f, "{a} - {b}"),
            Expr::Mul(a, b) => write!(f, "{a} * {b}"),
            Expr::Div(a, b) => write!(f, "{a} / {b}"),
            Expr::Mod(a, b) => write!(f, "{a} % {b}"),
            Expr::Eq(a, b) => write!(f, "{a} = {b}"),
            Expr::Neq(a, b) => write!(f, "{a} != {b}"),
            Expr::Lt(a, b) => write!(f, "{a} < {b}"),
            Expr::Lte(a, b) => write!(f, "{a} <= {b}"),
            Expr::Gt(a, b) => write!(f, "{a} > {b}"),
            Expr::Gte(a, b) => write!(f, "{a} >= {b}"),
            Expr::Not(e) => write!(f, "!{e}"),
            Expr::And(a, b) => write!(f, "{a} & {b}"),
            Expr::Or(a, b) => write!(f, "{a} | {b}"),
            Expr::Xor(a, b) => write!(f, "{a} ^ {b}"),
        }
    }
}

impl fmt::Display for FormulaSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Start
        if let Some(expr) = &self.start {
            write!(f, "({expr})")?;
        } else {
            write!(f, "*")?;
        }
        for delta in &self.start_delta {
            write!(f, " {delta}")?;
        }
        for time in &self.start_time {
            write!(f, " {time}")?;
        }

        // End
        if self.end_delta.is_some() || self.end_time.is_some() {
            write!(f, " --")?;
            if let Some(delta) = &self.end_delta {
                write!(f, " {delta}")?;
            }
            if let Some(time) = &self.end_time {
                write!(f, " {time}")?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Spec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Spec::Date(spec) => write!(f, "{spec}"),
            Spec::Weekday(spec) => write!(f, "{spec}"),
            Spec::Formula(spec) => write!(f, "{spec}"),
        }
    }
}

impl fmt::Display for BirthdaySpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.year_known {
            write!(f, "{}", self.date)
        } else {
            write!(f, "?-{:02}-{:02}", self.date.month(), self.date.day())
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Date(spec) => writeln!(f, "DATE {spec}"),
            Statement::BDate(spec) => writeln!(f, "BDATE {spec}"),
            Statement::From(Some(date)) => writeln!(f, "FROM {date}"),
            Statement::From(None) => writeln!(f, "FROM *"),
            Statement::Until(Some(date)) => writeln!(f, "UNTIL {date}"),
            Statement::Until(None) => writeln!(f, "UNTIL *"),
            Statement::Except(date) => writeln!(f, "EXCEPT {date}"),
            Statement::Move {
                from, to, to_time, ..
            } => match (to, to_time) {
                (None, None) => unreachable!(),
                (Some(to), None) => writeln!(f, "MOVE {from} TO {to}"),
                (None, Some(to_time)) => writeln!(f, "MOVE {from} TO {to_time}"),
                (Some(to), Some(to_time)) => writeln!(f, "MOVE {from} TO {to} {to_time}"),
            },
            Statement::Remind(Some(delta)) => writeln!(f, "REMIND {delta}"),
            Statement::Remind(None) => writeln!(f, "REMIND *"),
        }
    }
}

impl fmt::Display for DoneDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.simplified() {
            DoneDate::Date { root } => write!(f, "{root}"),
            DoneDate::DateTime { root, root_time } => write!(f, "{root} {root_time}"),
            DoneDate::DateToDate { root, other } => write!(f, "{root} -- {other}"),
            DoneDate::DateTimeToTime {
                root,
                root_time,
                other_time,
            } => write!(f, "{root} {root_time} -- {other_time}"),
            DoneDate::DateTimeToDateTime {
                root,
                root_time,
                other,
                other_time,
            } => write!(f, "{root} {root_time} -- {other} {other_time}"),
        }
    }
}

impl fmt::Display for Done {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.kind {
            DoneKind::Done => "DONE",
            DoneKind::Canceled => "CANCELED",
        };
        write!(f, "{kind} [{}]", self.done_at)?;
        if let Some(date) = &self.date {
            write!(f, " {date}")?;
        }
        writeln!(f)
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "TASK {}", self.title)?;
        for statement in &self.statements {
            write!(f, "{statement}")?;
        }
        for done in &self.done {
            write!(f, "{done}")?;
        }
        format_desc(f, &self.desc)?;
        Ok(())
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "NOTE {}", self.title)?;
        for statement in &self.statements {
            write!(f, "{statement}")?;
        }
        format_desc(f, &self.desc)?;
        Ok(())
    }
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "LOG {}", self.date)?;
        format_desc(f, &self.desc)?;
        Ok(())
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Include(name) => writeln!(f, "INCLUDE {name}"),
            Command::Timezone(name) => writeln!(f, "TIMEZONE {name}"),
            Command::Capture => writeln!(f, "CAPTURE"),
            Command::Task(task) => write!(f, "{task}"),
            Command::Note(note) => write!(f, "{note}"),
            Command::Log(log) => write!(f, "{log}"),
        }
    }
}

impl File {
    fn sort(commands: &mut [&Command]) {
        // Order of commands in a file:
        // 1. Imports, sorted alphabetically
        // 2. Time zone(s)
        // 3. Captures
        // 4. Log entries, sorted by date (ascending)
        // 5. Tasks and notes, in original order

        // There should always be at most one time zone, so we don't care about
        // their order.

        // In the individual steps we must use a stable sort so the order of 4.
        // is not lost.

        // Order imports alphabetically
        commands.sort_by_key(|c| match c {
            Command::Include(path) => Some(&path.value),
            _ => None,
        });

        // Order log entries by date
        commands.sort_by_key(|c| match c {
            Command::Log(Log { date, .. }) => Some(date.value),
            _ => None,
        });

        // Order by type
        commands.sort_by_key(|c| match c {
            Command::Include(_) => 0,
            Command::Timezone(_) => 1,
            Command::Capture => 2,
            Command::Log(_) => 3,
            Command::Task(_) | Command::Note(_) => 4,
        });
    }

    pub fn format(&self, removed: &HashSet<usize>) -> String {
        let mut result = String::new();

        let mut commands = self
            .commands
            .iter()
            .enumerate()
            .filter(|(i, _)| !removed.contains(i))
            .map(|(_, c)| &c.value)
            .collect::<Vec<_>>();

        Self::sort(&mut commands);

        for i in 0..commands.len() {
            let curr = &commands[i];
            let next = commands.get(i + 1);

            result.push_str(&format!("{curr}"));

            match (curr, next) {
                (Command::Include(_), Some(Command::Include(_))) => {}
                (_, None) => {}
                _ => result.push('\n'),
            }
        }

        result
    }
}
