use std::result;

use chrono::{Datelike, NaiveDate};

use crate::eval::entry::EntryDate;
use crate::files::commands::{Birthday, Command};
use crate::files::{Files, Source};

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
    pub fn new(range: DateRange, source: Source) -> Self {
        Self {
            map: EntryMap::new(range),
            source,
        }
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

    pub fn eval(&mut self, command: &Command) -> Result<Vec<Entry>> {
        // This function fills the entry map and then drains it again, so in
        // theory, the struct can even be reused afterwards.
        match command {
            Command::Task(task) => todo!(),
            Command::Note(note) => todo!(),
            Command::Birthday(birthday) => self.eval_birthday(birthday),
        }
        Ok(self.map.drain())
    }
}

impl Files {
    pub fn eval(&self, range: DateRange) -> Result<Vec<Entry>> {
        let mut result = vec![];
        for command in self.commands() {
            result.append(&mut Eval::new(range, command.source).eval(command.command)?);
        }
        Ok(result)
    }
}
