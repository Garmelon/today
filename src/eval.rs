use std::result;

use chrono::{Datelike, NaiveDate};

use crate::eval::entry::EntryDate;
use crate::files::commands::{Birthday, Command, Note, Spec, Task};
use crate::files::{Files, Source, SourcedCommand};

use self::entry::{Entry, EntryKind, EntryMap};
pub use self::range::DateRange;

mod delta;
mod entry;
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
    fn eval_spec(
        &mut self,
        spec: &Spec,
        new_entry: impl Fn(Source, EntryDate) -> Entry,
    ) -> Result<()> {
        todo!()
    }

    fn eval_note(&mut self, note: &Note) -> Result<()> {
        if note.when.is_empty() {
            let entry = Entry {
                kind: EntryKind::Note,
                title: note.title.clone(),
                desc: note.desc.clone(),
                source: self.source,
                date: EntryDate::None,
            };
            self.map.insert(entry);
        } else {
            if let Some(from) = note.from {
                if self.map.range().until() < from {
                    return Ok(());
                }
                if self.map.range().from() < from {
                    self.map.set_from(from);
                }
            }
            if let Some(until) = note.until {
                if until < self.map.range().from() {
                    return Ok(());
                }
                if until < self.map.range().until() {
                    self.map.set_until(until);
                }
            }
            for except in &note.except {
                self.map.block(*except);
            }
            for spec in &note.when {
                self.eval_spec(spec, |source, date| Entry {
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
        for year in self.map.range().years() {
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
                    date: EntryDate::Date { root: date },
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
                    date: EntryDate::Date { root: date },
                };
                self.map.insert(entry);

                let date = NaiveDate::from_ymd(year, 3, 1);
                let entry = Entry {
                    kind: EntryKind::Birthday,
                    title: format!("{} (second half)", title),
                    desc: birthday.desc.clone(),
                    source: self.source,
                    date: EntryDate::Date { root: date },
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
