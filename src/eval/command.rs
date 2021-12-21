use std::collections::HashMap;

use chrono::{Duration, NaiveDate};

use crate::files::commands::{
    self, BirthdaySpec, Command, Done, DoneDate, Note, Spec, Statement, Task,
};
use crate::files::primitives::{Span, Spanned, Time};
use crate::files::SourcedCommand;

use super::date::Dates;
use super::delta::Delta;
use super::{DateRange, Entry, EntryKind, Error, Result};

mod birthday;
mod date;
mod formula;

pub struct CommandState<'a> {
    command: SourcedCommand<'a>,
    range: DateRange,

    from: Option<NaiveDate>,
    until: Option<NaiveDate>,
    remind: Option<Spanned<Delta>>,

    dated: HashMap<NaiveDate, Entry>,
    undated: Vec<Entry>,
}

impl<'a> CommandState<'a> {
    pub fn new(command: SourcedCommand<'a>, mut range: DateRange) -> Self {
        // If we don't calculate entries for the source of the move command, it
        // fails even though the user did nothing wrong. Also, move commands (or
        // chains thereof) may move an initially out-of-range entry into range.
        //
        // To fix this, we just expand the range to contain all move command
        // sources. This is a quick fix, but until it becomes a performance
        // issue (if ever), it's probably fine.
        for statement in command.command.statements() {
            if let Statement::Move { from, .. } = statement {
                range = range.containing(*from)
            }
        }

        Self {
            range,
            command,
            from: None,
            until: None,
            remind: None,
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

    fn range_with_remind(&self) -> DateRange {
        match &self.remind {
            None => self.range,
            Some(delta) => self.range.expand_by(&delta.value),
        }
    }

    /// Last root date mentioned in any `DONE`.
    fn last_done_root(&self) -> Option<NaiveDate> {
        match self.command.command {
            Command::Task(task) => task
                .done
                .iter()
                .filter_map(|done| done.date.map(DoneDate::root))
                .max(),
            Command::Note(_) => None,
        }
    }

    /// Last completion date mentioned in any `DONE`.
    fn last_done_completion(&self) -> Option<NaiveDate> {
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

        if from <= until {
            Some(DateRange::new(from, until))
        } else {
            None
        }
    }

    fn entry_with_remind(&self, kind: EntryKind, dates: Option<Dates>) -> Result<Entry> {
        let remind = if let (Some(dates), Some(delta)) = (dates, &self.remind) {
            let index = self.command.source.file();
            let start = dates.sorted().root();
            let remind = delta.value.apply_date(index, dates.sorted().root())?;
            if remind >= start {
                return Err(Error::RemindDidNotMoveBackwards {
                    index,
                    span: delta.span,
                    from: start,
                    to: remind,
                });
            }
            Some(remind)
        } else {
            None
        };

        Ok(Entry::new(self.command.source, kind, dates, remind))
    }

    /// Add an entry, respecting [`Self::from`] and [`Self::until`]. Does not
    /// overwrite existing entries if a root date is specified.
    fn add(&mut self, entry: Entry) {
        if let Some(dates) = entry.dates {
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
    fn add_forced(&mut self, entry: Entry) {
        if let Some(dates) = entry.dates {
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
        } else if task.done.is_empty() {
            self.add(self.entry_with_remind(self.kind(), None)?);
        }

        for done in &task.done {
            self.eval_done(done)?;
        }

        Ok(())
    }

    fn eval_note(&mut self, note: &Note) -> Result<()> {
        if Self::has_date_stmt(&note.statements) {
            for statement in &note.statements {
                self.eval_statement(statement)?;
            }
        } else {
            self.add(self.entry_with_remind(self.kind(), None)?);
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
            Statement::Move {
                span,
                from,
                to,
                to_time,
            } => self.eval_move(*span, *from, *to, *to_time)?,
            Statement::Remind(delta) => self.eval_remind(delta),
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

    fn eval_move(
        &mut self,
        span: Span,
        from: NaiveDate,
        to: Option<NaiveDate>,
        to_time: Option<Time>,
    ) -> Result<()> {
        if let Some(mut entry) = self.dated.remove(&from) {
            let mut dates = entry.dates.expect("comes from self.dated");

            // Determine delta
            let mut delta = Duration::zero();
            if let Some(to) = to {
                delta = delta + (to - dates.root());
            }
            if let Some(to_time) = to_time {
                if let Some((root, _)) = dates.times() {
                    delta = delta + Duration::minutes(root.minutes_to(to_time));
                }
            }

            dates = dates.move_by(delta);
            entry.dates = Some(dates);
            self.dated.insert(dates.root(), entry);

            Ok(())
        } else {
            Err(Error::MoveWithoutSource {
                index: self.command.source.file(),
                span,
            })
        }
    }

    fn eval_remind(&mut self, delta: &Option<Spanned<commands::Delta>>) {
        if let Some(delta) = delta {
            self.remind = Some(Spanned::new(delta.span, (&delta.value).into()));
        } else {
            self.remind = None;
        }
    }

    fn eval_done(&mut self, done: &Done) -> Result<()> {
        self.add_forced(self.entry_with_remind(
            EntryKind::TaskDone(done.done_at),
            done.date.map(|date| date.into()),
        )?);
        Ok(())
    }
}
