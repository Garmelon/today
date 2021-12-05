use chrono::{Datelike, NaiveDate};

use crate::eval::date::Dates;
use crate::files::commands::BirthdaySpec;

use super::super::command::CommandState;
use super::super::EntryKind;

impl<'a> CommandState<'a> {
    pub fn eval_birthday_spec(&mut self, spec: &BirthdaySpec) {
        // This could be optimized by restricting the range via FROM and UNTIL,
        // but I don't think that kind of optimization will be necessary any
        // time soon.
        for year in self.range.years() {
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
                self.add(EntryKind::Birthday(age), Some(Dates::new(date, date)));
            } else {
                assert_eq!(spec.date.month(), 2);
                assert_eq!(spec.date.day(), 29);

                let first = NaiveDate::from_ymd(year, 2, 28);
                let second = NaiveDate::from_ymd(year, 3, 1);
                self.add(kind, Some(Dates::new(first, second)));
            }
        }
    }
}
