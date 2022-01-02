use chrono::{Datelike, NaiveDate};

use crate::files::commands::BirthdaySpec;
use crate::files::FileSource;

use super::super::command::CommandState;
use super::super::date::Dates;
use super::super::error::Error;
use super::super::EntryKind;

impl<'a> CommandState<'a> {
    pub fn eval_birthday_spec(&mut self, spec: &BirthdaySpec) -> Result<(), Error<FileSource>> {
        let range = match self.limit_from_until(self.range_with_remind()) {
            Some(range) => range,
            None => return Ok(()),
        };

        for year in range.years() {
            let age = if spec.year_known {
                let age = year - spec.date.year();
                if age < 0 {
                    continue;
                }
                Some(age)
            } else {
                None
            };
            let kind = EntryKind::Birthday(age);

            if let Some(date) = spec.date.with_year(year) {
                self.add(
                    self.entry_with_remind(EntryKind::Birthday(age), Some(Dates::new(date, date)))?,
                );
            } else {
                assert_eq!(spec.date.month(), 2);
                assert_eq!(spec.date.day(), 29);

                let first = NaiveDate::from_ymd(year, 2, 28);
                let second = NaiveDate::from_ymd(year, 3, 1);
                self.add(self.entry_with_remind(kind, Some(Dates::new(first, second)))?);
            }
        }

        Ok(())
    }
}
