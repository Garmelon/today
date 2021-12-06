use std::collections::HashMap;

use chrono::NaiveDate;

use crate::files::commands::{BirthdaySpec, Command, Done, Note, Spec, Statement, Task};
use crate::files::primitives::Span;
use crate::files::SourcedCommand;

use super::date::Dates;
use super::{DateRange, Entry, EntryKind, Error, Result};

mod birthday;
mod date;
mod formula;

pub struct CommandState<'a> {
    command: SourcedCommand<'a>,
    range: DateRange,

    from: Option<NaiveDate>,
    until: Option<NaiveDate>,

    dated: HashMap<NaiveDate, Entry>,
    undated: Vec<Entry>,
}

impl<'a> CommandState<'a> {
    pub fn new(command: SourcedCommand<'a>, range: DateRange) -> Self {
        Self {
            range,
            command,
            from: None,
            until: None,
            dated: HashMap::new(),
            undated: Vec::new(),
        }
    }

    pub fn eval(mut self) -> Result<Self> {
        match self.command.command {
            Command::Task(task) => self.eval_task(task)?,
            Command::Note(note) => self.eval_note(note)?,
        }
        Ok(self)
    }

    pub fn entries(self) -> Vec<Entry> {
        self.undated
            .into_iter()
            .chain(self.dated.into_values())
            .collect()
    }

    // Helper functions

    fn kind(&self) -> EntryKind {
        match self.command.command {
            Command::Task(_) => EntryKind::Task,
            Command::Note(_) => EntryKind::Note,
        }
    }

    fn last_done(&self) -> Option<NaiveDate> {
        match self.command.command {
            Command::Task(task) => task.done.iter().map(|done| done.done_at).max(),
            Command::Note(_) => None,
        }
    }

    fn limit_from_until(&self, range: DateRange) -> Option<DateRange> {
        let range_from = range.from();
        let from = self
            .from
            .filter(|&from| from > range_from)
            .unwrap_or(range_from);

        let range_until = range.until();
        let until = self
            .until
            .filter(|&until| until < range_until)
            .unwrap_or(range_until);

        DateRange::new(from, until)
    }

    /// Add an entry, respecting [`Self::from`] and [`Self::until`]. Does not
    /// overwrite existing entries if a root date is specified.
    fn add(&mut self, kind: EntryKind, dates: Option<Dates>) {
        let entry = Entry::new(self.command.source, kind, dates);
        if let Some(dates) = dates {
            let root = dates.root();
            if let Some(from) = self.from {
                if root < from {
                    return;
                }
            }
            if let Some(until) = self.until {
                if until < root {
                    return;
                }
            }
            self.dated.entry(root).or_insert(entry);
        } else {
            self.undated.push(entry);
        }
    }

    /// Add an entry, ignoring [`Self::from`] and [`Self::until`]. Always
    /// overwrites existing entries if a root date is specified.
    fn add_forced(&mut self, kind: EntryKind, dates: Option<Dates>) {
        let entry = Entry::new(self.command.source, kind, dates);
        if let Some(dates) = dates {
            self.dated.insert(dates.root(), entry);
        } else {
            self.undated.push(entry);
        }
    }

    // Actual evaluation

    fn has_date_stmt(statements: &[Statement]) -> bool {
        statements
            .iter()
            .any(|s| matches!(s, Statement::Date(_) | Statement::BDate(_)))
    }

    fn eval_task(&mut self, task: &Task) -> Result<()> {
        if Self::has_date_stmt(&task.statements) {
            for statement in &task.statements {
                self.eval_statement(statement)?;
            }
        } else {
            self.add(self.kind(), None);
        }

        for done in &task.done {
            self.eval_done(done);
        }

        Ok(())
    }

    fn eval_note(&mut self, note: &Note) -> Result<()> {
        if Self::has_date_stmt(&note.statements) {
            for statement in &note.statements {
                self.eval_statement(statement)?;
            }
        } else {
            self.add(self.kind(), None);
        }

        Ok(())
    }

    fn eval_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Date(spec) => self.eval_date(spec)?,
            Statement::BDate(spec) => self.eval_bdate(spec),
            Statement::From(date) => self.from = *date,
            Statement::Until(date) => self.until = *date,
            Statement::Except(date) => self.eval_except(*date),
            Statement::Move { span, from, to } => self.eval_move(*span, *from, *to)?,
        }
        Ok(())
    }

    fn eval_date(&mut self, spec: &Spec) -> Result<()> {
        match spec {
            Spec::Date(spec) => self.eval_date_spec(spec.into()),
            Spec::Weekday(spec) => self.eval_formula_spec(spec.into()),
            Spec::Formula(spec) => self.eval_formula_spec(spec.into()),
        }
    }

    fn eval_bdate(&mut self, spec: &BirthdaySpec) {
        self.eval_birthday_spec(spec);
    }

    fn eval_except(&mut self, date: NaiveDate) {
        self.dated.remove(&date);
    }

    fn eval_move(&mut self, span: Span, from: NaiveDate, to: NaiveDate) -> Result<()> {
        if let Some(mut entry) = self.dated.remove(&from) {
            if let Some(dates) = entry.dates {
                let delta = to - from;
                entry.dates = Some(dates.move_by(delta));
            }
            self.dated.insert(to, entry);
            Ok(())
        } else {
            Err(Error::MoveWithoutSource { span })
        }
    }

    fn eval_done(&mut self, done: &Done) {
        self.add_forced(
            EntryKind::TaskDone(done.done_at),
            done.date.map(|date| date.into()),
        );
    }
}
