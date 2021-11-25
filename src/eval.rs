use std::cmp;
use std::result;

use chrono::{Datelike, NaiveDate};

use crate::files::commands::DateSpec;
use crate::files::commands::{Birthday, Command, DoneDate, Note, Spec, Task};
use crate::files::{Files, Source, SourcedCommand};

use self::entry::EntryMap;
pub use self::entry::{Entry, EntryKind};
use self::formula_spec::FormulaSpec;
pub use self::range::DateRange;

mod delta;
mod entry;
mod formula_spec;
mod range;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("TODO")]
    Todo,
}

type Result<T> = result::Result<T, Error>;

struct Eval {
    map: EntryMap,
    source: Source,
}

impl Eval {
    fn eval_date_spec(
        &mut self,
        spec: &DateSpec,
        last_done: Option<NaiveDate>,
        new_entry: impl Fn(Source, Option<DoneDate>) -> Entry,
    ) -> Result<()> {
        todo!()
    }

    fn eval_formula_spec(
        &mut self,
        spec: FormulaSpec,
        new_entry: impl Fn(Source, Option<DoneDate>) -> Entry,
    ) -> Result<()> {
        todo!()
    }

    fn eval_spec(
        &mut self,
        spec: &Spec,
        last_done: Option<NaiveDate>,
        new_entry: impl Fn(Source, Option<DoneDate>) -> Entry,
    ) -> Result<()> {
        match spec {
            Spec::Date(spec) => self.eval_date_spec(spec, last_done, new_entry),
            Spec::Weekday(spec) => self.eval_formula_spec(spec.into(), new_entry),
            Spec::Formula(spec) => self.eval_formula_spec(spec.into(), new_entry),
        }
    }

    fn eval_dones(&mut self, task: &Task) {
        for done in &task.done {
            let entry = Entry {
                kind: EntryKind::TaskDone(done.done_at),
                title: task.title.clone(),
                desc: task.desc.clone(),
                source: self.source,
                date: done.date,
            };
            self.map.insert(entry);
        }
    }

    fn determine_last_done(task: &Task) -> Option<NaiveDate> {
        task.done
            .iter()
            .flat_map(|done| done.date.iter())
            .map(|d| d.root())
            .max()
    }

    fn determine_task_range(&self, task: &Task, last_done: Option<NaiveDate>) -> Option<DateRange> {
        let mut from = self.map.range.from();
        let mut until = self.map.range.until();

        if let Some(last_done) = last_done {
            from = cmp::min(from, last_done);
        }

        if let Some(task_from) = task.from {
            from = cmp::max(from, task_from);
        }
        if let Some(task_until) = task.until {
            until = cmp::min(until, task_until);
        }

        if from <= until {
            Some(DateRange::new(from, until))
        } else {
            None
        }
    }

    fn eval_task(&mut self, task: &Task) -> Result<()> {
        self.eval_dones(task);

        if task.done.iter().any(|done| done.date.is_none()) {
            return Ok(());
        }

        if task.when.is_empty() {
            self.map.insert(Entry {
                kind: EntryKind::Task,
                title: task.title.clone(),
                desc: task.desc.clone(),
                source: self.source,
                date: None,
            });
        } else {
            let last_done = Self::determine_last_done(task);

            if let Some(range) = self.determine_task_range(task, last_done) {
                self.map.range = range;
            } else {
                return Ok(());
            }

            for except in &task.except {
                self.map.block(*except);
            }

            for spec in &task.when {
                self.eval_spec(spec, last_done, |source, date| Entry {
                    kind: EntryKind::Note,
                    title: task.title.clone(),
                    desc: task.desc.clone(),
                    source,
                    date,
                })?;
            }
        }
        Ok(())
    }

    fn determine_note_range(&self, note: &Note) -> Option<DateRange> {
        let mut from = self.map.range.from();
        let mut until = self.map.range.until();

        if let Some(task_from) = note.from {
            from = cmp::max(from, task_from);
        }
        if let Some(task_until) = note.until {
            until = cmp::max(until, task_until);
        }

        if from <= until {
            Some(DateRange::new(from, until))
        } else {
            None
        }
    }

    fn eval_note(&mut self, note: &Note) -> Result<()> {
        if note.when.is_empty() {
            self.map.insert(Entry {
                kind: EntryKind::Note,
                title: note.title.clone(),
                desc: note.desc.clone(),
                source: self.source,
                date: None,
            });
        } else {
            if let Some(range) = self.determine_note_range(note) {
                self.map.range = range;
            } else {
                return Ok(());
            }

            for except in &note.except {
                self.map.block(*except);
            }

            for spec in &note.when {
                self.eval_spec(spec, None, |source, date| Entry {
                    kind: EntryKind::Note,
                    title: note.title.clone(),
                    desc: note.desc.clone(),
                    source,
                    date,
                })?;
            }
        }
        Ok(())
    }

    fn eval_birthday(&mut self, birthday: &Birthday) {
        for year in self.map.range.years() {
            let when = &birthday.when;
            let mut title = birthday.title.clone();

            if when.year_known {
                let age = year - when.date.year();
                if age < 0 {
                    continue;
                }
                title.push_str(&format!(" ({})", age));
            }

            if let Some(date) = when.date.with_year(year) {
                let entry = Entry {
                    kind: EntryKind::Birthday,
                    title: title.clone(),
                    desc: birthday.desc.clone(),
                    source: self.source,
                    date: Some(DoneDate::Date { root: date }),
                };
                self.map.insert(entry);
            } else {
                assert_eq!(when.date.month(), 2);
                assert_eq!(when.date.day(), 29);

                let date = NaiveDate::from_ymd(year, 2, 28);
                let entry = Entry {
                    kind: EntryKind::Birthday,
                    title: format!("{} (first half)", title),
                    desc: birthday.desc.clone(),
                    source: self.source,
                    date: Some(DoneDate::Date { root: date }),
                };
                self.map.insert(entry);

                let date = NaiveDate::from_ymd(year, 3, 1);
                let entry = Entry {
                    kind: EntryKind::Birthday,
                    title: format!("{} (second half)", title),
                    desc: birthday.desc.clone(),
                    source: self.source,
                    date: Some(DoneDate::Date { root: date }),
                };
                self.map.insert(entry);
            }
        }
    }

    pub fn eval(range: DateRange, command: &SourcedCommand<'_>) -> Result<Vec<Entry>> {
        let mut map = Self {
            map: EntryMap::new(range),
            source: command.source,
        };
        match command.command {
            Command::Task(task) => map.eval_task(task)?,
            Command::Note(note) => map.eval_note(note)?,
            Command::Birthday(birthday) => map.eval_birthday(birthday),
        }
        Ok(map.map.drain())
    }
}

impl Files {
    pub fn eval(&self, range: DateRange) -> Result<Vec<Entry>> {
        let mut result = vec![];
        for command in self.commands() {
            result.append(&mut Eval::eval(range, &command)?);
        }
        Ok(result)
    }
}
