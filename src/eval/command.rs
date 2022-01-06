use std::collections::HashMap;

use chrono::{Duration, NaiveDate};

use crate::files::commands::{
    self, BirthdaySpec, Command, Done, DoneDate, DoneKind, Note, Spec, Statement, Task,
};
use crate::files::primitives::{Span, Spanned, Time};
use crate::files::{FileSource, Source};

use super::date::Dates;
use super::delta::Delta;
use super::{DateRange, Entry, EntryKind, Error};

mod birthday;
mod date;
mod formula;

/// A command that can be evaluated.
pub enum EvalCommand<'a> {
    Task(&'a Task),
    Note(&'a Note),
}

impl<'a> EvalCommand<'a> {
    pub fn new(command: &'a Command) -> Option<Self> {
        match command {
            Command::Task(task) => Some(Self::Task(task)),
            Command::Note(note) => Some(Self::Note(note)),
            _ => None,
        }
    }

    fn statements(&self) -> &[Statement] {
        match self {
            Self::Task(task) => &task.statements,
            Self::Note(note) => &note.statements,
        }
    }

    fn kind(&self) -> EntryKind {
        match self {
            Self::Task(_) => EntryKind::Task,
            Self::Note(_) => EntryKind::Note,
        }
    }

    fn title(&self) -> String {
        match self {
            Self::Task(task) => task.title.clone(),
            Self::Note(note) => note.title.clone(),
        }
    }

    fn has_description(&self) -> bool {
        match self {
            Self::Task(task) => !task.desc.is_empty(),
            Self::Note(note) => !note.desc.is_empty(),
        }
    }

    /// Last root date mentioned in any `DONE`.
    fn last_done_root(&self) -> Option<NaiveDate> {
        match self {
            Self::Task(task) => task
                .done
                .iter()
                .filter_map(|done| done.date.map(DoneDate::root))
                .max(),
            Self::Note(_) => None,
        }
    }

    /// Last completion date mentioned in any `DONE`.
    fn last_done_completion(&self) -> Option<NaiveDate> {
        match self {
            Self::Task(task) => task.done.iter().map(|done| done.done_at).max(),
            Self::Note(_) => None,
        }
    }
}

pub struct CommandState<'a> {
    command: EvalCommand<'a>,
    source: Source,
    range: DateRange,

    from: Option<NaiveDate>,
    until: Option<NaiveDate>,
    remind: Option<Spanned<Delta>>,

    dated: HashMap<NaiveDate, Entry>,
    undated: Vec<Entry>,
}

impl<'a> CommandState<'a> {
    pub fn new(command: EvalCommand<'a>, source: Source, mut range: DateRange) -> Self {
        // If we don't calculate entries for the source of the move command, it
        // fails even though the user did nothing wrong. Also, move commands (or
        // chains thereof) may move an initially out-of-range entry into range.
        //
        // To fix this, we just expand the range to contain all move command
        // sources. This is a quick fix, but until it becomes a performance
        // issue (if ever), it's probably fine.
        for statement in command.statements() {
            if let Statement::Move { from, .. } = statement {
                range = range.containing(*from)
            }
        }

        Self {
            command,
            source,
            range,
            from: None,
            until: None,
            remind: None,
            dated: HashMap::new(),
            undated: Vec::new(),
        }
    }

    pub fn eval(mut self) -> Result<Self, Error<FileSource>> {
        match self.command {
            EvalCommand::Task(task) => self.eval_task(task)?,
            EvalCommand::Note(note) => self.eval_note(note)?,
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

    fn range_with_remind(&self) -> DateRange {
        match &self.remind {
            None => self.range,
            Some(delta) => self.range.expand_by(&delta.value),
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

    fn entry_with_remind(
        &self,
        kind: EntryKind,
        dates: Option<Dates>,
    ) -> Result<Entry, Error<FileSource>> {
        let remind = if let (Some(dates), Some(delta)) = (dates, &self.remind) {
            let index = self.source.file();
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

        Ok(Entry::new(
            self.source,
            kind,
            self.command.title(),
            self.command.has_description(),
            dates,
            remind,
        ))
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

    fn eval_task(&mut self, task: &Task) -> Result<(), Error<FileSource>> {
        if Self::has_date_stmt(&task.statements) {
            for statement in &task.statements {
                self.eval_statement(statement)?;
            }
        } else if task.done.is_empty() {
            self.add(self.entry_with_remind(self.command.kind(), None)?);
        }

        for done in &task.done {
            self.eval_done(done)?;
        }

        Ok(())
    }

    fn eval_note(&mut self, note: &Note) -> Result<(), Error<FileSource>> {
        if Self::has_date_stmt(&note.statements) {
            for statement in &note.statements {
                self.eval_statement(statement)?;
            }
        } else {
            self.add(self.entry_with_remind(self.command.kind(), None)?);
        }

        Ok(())
    }

    fn eval_statement(&mut self, statement: &Statement) -> Result<(), Error<FileSource>> {
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

    fn eval_date(&mut self, spec: &Spec) -> Result<(), Error<FileSource>> {
        match spec {
            Spec::Date(spec) => self.eval_date_spec(spec.into()),
            Spec::Weekday(spec) => self.eval_formula_spec(spec.into()),
            Spec::Formula(spec) => self.eval_formula_spec(spec.into()),
        }
    }

    fn eval_bdate(&mut self, spec: &BirthdaySpec) -> Result<(), Error<FileSource>> {
        self.eval_birthday_spec(spec)
    }

    fn eval_except(&mut self, date: NaiveDate) {
        // TODO Error if nothing is removed?
        self.dated.remove(&date);
    }

    fn eval_move(
        &mut self,
        span: Span,
        from: NaiveDate,
        to: Option<NaiveDate>,
        to_time: Option<Spanned<Time>>,
    ) -> Result<(), Error<FileSource>> {
        if let Some(mut entry) = self.dated.remove(&from) {
            let mut dates = entry.dates.expect("comes from self.dated");

            // Determine delta
            let mut delta = Duration::zero();
            if let Some(to) = to {
                delta = delta + (to - dates.root());
            }
            if let Some(to_time) = to_time {
                if let Some((root, _)) = dates.times() {
                    delta = delta + Duration::minutes(root.minutes_to(to_time.value));
                } else {
                    return Err(Error::TimedMoveWithoutTime {
                        index: self.source.file(),
                        span: to_time.span,
                    });
                }
            }

            dates = dates.move_by(delta);
            entry.dates = Some(dates);
            self.dated.insert(dates.root(), entry);

            Ok(())
        } else {
            Err(Error::MoveWithoutSource {
                index: self.source.file(),
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

    fn eval_done(&mut self, done: &Done) -> Result<(), Error<FileSource>> {
        let kind = match done.kind {
            DoneKind::Done => EntryKind::TaskDone(done.done_at),
            DoneKind::Canceled => EntryKind::TaskCanceled(done.done_at),
        };
        let dates = done.date.map(|date| date.into());
        self.add_forced(self.entry_with_remind(kind, dates)?);
        Ok(())
    }
}
