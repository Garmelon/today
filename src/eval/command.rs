use std::collections::HashMap;

use chrono::NaiveDate;

use crate::files::commands::{BirthdaySpec, Command, Done, Note, Span, Spec, Statement, Task};
use crate::files::{Source, SourcedCommand};

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
        self.dated
            .into_values()
            .chain(self.undated.into_iter())
            .collect()
    }

    // Helper functions

    fn title(&self) -> String {
        self.command.command.title().to_string()
    }

    fn desc(&self) -> Vec<String> {
        self.command.command.desc().to_vec()
    }

    fn source(&self) -> Source {
        self.command.source
    }

    /// Add an entry, respecting [`Self::from`] and [`Self::until`]. Does not
    /// overwrite existing entries if a root date is specified.
    fn add(&mut self, entry: Entry) {
        if let Some(root) = entry.root {
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
    fn add_forced(&mut self, entry: Entry) {
        if let Some(root) = entry.root {
            self.dated.insert(root, entry);
        } else {
            self.undated.push(entry);
        }
    }

    // Actual evaluation

    fn eval_task(&mut self, task: &Task) -> Result<()> {
        for statement in &task.statements {
            self.eval_statement(statement)?;
        }
        for done in &task.done {
            self.eval_done(done);
        }
        Ok(())
    }

    fn eval_note(&mut self, note: &Note) -> Result<()> {
        for statement in &note.statements {
            self.eval_statement(statement)?;
        }
        Ok(())
    }

    fn eval_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Date(spec) => self.eval_date(spec)?,
            Statement::BDate(spec) => self.eval_bdate(spec)?,
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

    fn eval_bdate(&mut self, spec: &BirthdaySpec) -> Result<()> {
        self.eval_birthday_spec(spec)
    }

    fn eval_except(&mut self, date: NaiveDate) {
        self.dated.remove(&date);
    }

    fn eval_move(&mut self, span: Span, from: NaiveDate, to: NaiveDate) -> Result<()> {
        if let Some(entry) = self.dated.remove(&from) {
            self.dated.insert(to, entry);
            Ok(())
        } else {
            Err(Error::MoveWithoutSource { span })
        }
    }

    fn eval_done(&mut self, done: &Done) {
        self.add_forced(Entry {
            kind: EntryKind::TaskDone(done.done_at),
            title: self.title(),
            desc: self.desc(),
            source: self.source(),
            dates: done.date.map(|date| date.into()),
            root: done.date.map(|date| date.root()),
        });
    }
}
