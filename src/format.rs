use std::fmt;

use chrono::Datelike;

use crate::commands::{
    Birthday, BirthdaySpec, Command, DateSpec, Delta, DeltaStep, Done, Expr, File, FormulaSpec,
    Note, Spec, Task, Time, Var, Weekday, WeekdaySpec,
};

fn format_desc(f: &mut fmt::Formatter<'_>, desc: &[String]) -> fmt::Result {
    for line in desc {
        if line.is_empty() {
            writeln!(f, "#")?;
        } else {
            writeln!(f, "# {}", line)?;
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
            format_delta_step(f, step, &mut sign)?;
        }
        Ok(())
    }
}

impl fmt::Display for DateSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Start
        write!(f, "{}", self.start)?;
        for delta in &self.start_delta {
            write!(f, " {}", delta)?;
        }
        for time in &self.start_time {
            write!(f, " {}", time)?;
        }

        // End
        if self.end.is_some() || self.end_delta.is_some() || self.end_time.is_some() {
            write!(f, " --")?;
            if let Some(date) = self.end {
                write!(f, " {}", date)?;
            }
            if let Some(delta) = &self.end_delta {
                write!(f, " {}", delta)?;
            }
            if let Some(time) = &self.end_time {
                write!(f, " {}", time)?;
            }
        }

        // Repeat
        if let Some(repeat) = &self.repeat {
            write!(f, "; {}", repeat)?;
        }

        Ok(())
    }
}

impl fmt::Display for WeekdaySpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Start
        write!(f, "{}", self.start)?;
        for time in &self.start_time {
            write!(f, " {}", time)?;
        }

        // End
        if self.end.is_some() || self.end_delta.is_some() || self.end_time.is_some() {
            write!(f, " --")?;
            if let Some(wd) = self.end {
                write!(f, " {}", wd)?;
            }
            if let Some(delta) = &self.end_delta {
                write!(f, " {}", delta)?;
            }
            if let Some(time) = &self.end_time {
                write!(f, " {}", time)?;
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
            Expr::Lit(i) => write!(f, "{}", i),
            Expr::Var(v) => write!(f, "{}", v),
            Expr::Paren(e) => write!(f, "({})", e),
            Expr::Neg(e) => write!(f, "-{}", e),
            Expr::Add(a, b) => write!(f, "{} + {}", a, b),
            Expr::Sub(a, b) => write!(f, "{} - {}", a, b),
            Expr::Mul(a, b) => write!(f, "{} * {}", a, b),
            Expr::Div(a, b) => write!(f, "{} / {}", a, b),
            Expr::Mod(a, b) => write!(f, "{} % {}", a, b),
            Expr::Eq(a, b) => write!(f, "{} = {}", a, b),
            Expr::Neq(a, b) => write!(f, "{} != {}", a, b),
            Expr::Lt(a, b) => write!(f, "{} < {}", a, b),
            Expr::Lte(a, b) => write!(f, "{} <= {}", a, b),
            Expr::Gt(a, b) => write!(f, "{} > {}", a, b),
            Expr::Gte(a, b) => write!(f, "{} >= {}", a, b),
            Expr::Not(e) => write!(f, "!{}", e),
            Expr::And(a, b) => write!(f, "{} & {}", a, b),
            Expr::Or(a, b) => write!(f, "{} | {}", a, b),
            Expr::Xor(a, b) => write!(f, "{} ^ {}", a, b),
        }
    }
}

impl fmt::Display for FormulaSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Start
        if let Some(expr) = &self.start {
            write!(f, "({})", expr)?;
        } else {
            write!(f, "*")?;
        }
        for delta in &self.start_delta {
            write!(f, " {}", delta)?;
        }
        for time in &self.start_time {
            write!(f, " {}", time)?;
        }

        // End
        if self.end_delta.is_some() || self.end_time.is_some() {
            write!(f, " --")?;
            if let Some(delta) = &self.end_delta {
                write!(f, " {}", delta)?;
            }
            if let Some(time) = &self.end_time {
                write!(f, " {}", time)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Spec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DATE ")?;
        match self {
            Spec::Date(spec) => write!(f, "{}", spec)?,
            Spec::Weekday(spec) => write!(f, "{}", spec)?,
            Spec::Formula(spec) => write!(f, "{}", spec)?,
        }
        writeln!(f)
    }
}

impl fmt::Display for Done {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DONE")?;
        if let Some(date) = &self.refering_to {
            write!(f, " {}", date)?;
        }
        if let Some((date, time)) = &self.created_at {
            write!(f, " ({} {})", date, time)?;
        }
        writeln!(f)
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "TASK {}", self.title)?;
        for spec in &self.when {
            write!(f, "{}", spec)?;
        }
        if let Some(date) = self.from {
            writeln!(f, "FROM {}", date)?;
        }
        if let Some(date) = self.until {
            writeln!(f, "UNTIL {}", date)?;
        }
        for date in &self.except {
            writeln!(f, "EXCEPT {}", date)?;
        }
        for done in &self.done {
            write!(f, "{}", done)?;
        }
        format_desc(f, &self.desc)?;
        Ok(())
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "NOTE {}", self.title)?;
        for spec in &self.when {
            write!(f, "{}", spec)?;
        }
        if let Some(date) = self.from {
            writeln!(f, "FROM {}", date)?;
        }
        if let Some(date) = self.until {
            writeln!(f, "UNTIL {}", date)?;
        }
        for date in &self.except {
            writeln!(f, "EXCEPT {}", date)?;
        }
        format_desc(f, &self.desc)?;
        Ok(())
    }
}

impl fmt::Display for BirthdaySpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.year_known {
            writeln!(f, "BDATE {}", self.date)
        } else {
            writeln!(f, "BDATE ?-{:02}-{:02}", self.date.month(), self.date.day())
        }
    }
}

impl fmt::Display for Birthday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BIRTHDAY {}", self.title)?;
        write!(f, "{}", self.when)?;
        format_desc(f, &self.desc)?;
        Ok(())
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Task(task) => write!(f, "{}", task),
            Command::Note(note) => write!(f, "{}", note),
            Command::Birthday(birthday) => write!(f, "{}", birthday),
        }
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut commands = self.commands.iter();
        if let Some(command) = commands.next() {
            write!(f, "{}", command)?;
            for command in commands {
                writeln!(f)?;
                write!(f, "{}", command)?;
            }
        }
        Ok(())
    }
}
