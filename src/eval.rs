use std::collections::HashSet;
use std::result;

use chrono::{Datelike, NaiveDate};

use crate::files::commands::{Birthday, Command, File, Note, Spec, Task};

use self::entries::{DateRange, Entry, EntryKind, EntryMap};

pub mod entries;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("TODO")]
    Todo,
}

type Result<T> = result::Result<T, Error>;

fn eval_spec(spec: &Spec, index: usize, map: &mut EntryMap) -> Result<()> {
    Ok(())
}

fn eval_task(task: &Task, index: usize, range: DateRange) -> Result<Vec<Entry>> {
    let mut map = EntryMap::new(range);

    map.from = task.from;
    map.until = task.until;

    for date in &task.except {
        map.block(*date);
    }

    for spec in &task.when {
        eval_spec(spec, index, &mut map)?;
    }

    let done: HashSet<NaiveDate> = task
        .done
        .iter()
        .filter_map(|done| done.refering_to)
        .collect();

    for (date, entry) in map.map.iter_mut() {
        if let Some(entry) = entry {
            if done.contains(date) {
                entry.kind.done();
            }
        }
    }

    Ok(map.drain())
}

fn eval_note(task: &Note, index: usize, range: DateRange) -> Result<Vec<Entry>> {
    let mut map = EntryMap::new(range);

    map.from = task.from;
    map.until = task.until;

    for date in &task.except {
        map.block(*date);
    }

    for spec in &task.when {
        eval_spec(spec, index, &mut map)?;
    }

    Ok(map.drain())
}

fn eval_birthday(bd: &Birthday, index: usize, range: DateRange) -> Result<Vec<Entry>> {
    let mut map = EntryMap::new(range);

    for year in range.years() {
        if bd.when.year_known && year < bd.when.date.year() {
            continue;
        }

        let title = if bd.when.year_known {
            let age = year - bd.when.date.year();
            format!("{} ({})", bd.title, age)
        } else {
            bd.title.to_string()
        };

        match bd.when.date.with_year(year) {
            Some(date) => {
                let mut entry = Entry::new(index, EntryKind::Birthday, title);
                entry.start = Some(date);
                map.insert(date, entry);
            }
            None => {
                // We must've hit a non-leapyear
                assert_eq!(bd.when.date.month(), 2);
                assert_eq!(bd.when.date.day(), 29);

                let first_date = NaiveDate::from_ymd(year, 2, 28);
                let first_title = format!("{} (first half)", title);
                let mut first_entry = Entry::new(index, EntryKind::Birthday, first_title);
                first_entry.start = Some(first_date);
                map.insert(first_date, first_entry);

                let second_date = NaiveDate::from_ymd(year, 3, 1);
                let second_title = format!("{} (second half)", title);
                let mut second_entry = Entry::new(index, EntryKind::Birthday, second_title);
                second_entry.start = Some(second_date);
                map.insert(second_date, second_entry);
            }
        }
    }

    Ok(map.drain())
}

fn eval_command(command: &Command, index: usize, range: DateRange) -> Result<Vec<Entry>> {
    match command {
        Command::Task(task) => eval_task(task, index, range),
        Command::Note(note) => eval_note(note, index, range),
        Command::Birthday(birthday) => eval_birthday(birthday, index, range),
    }
}

pub fn eval(file: &File, range: DateRange) -> Result<Vec<Entry>> {
    let mut result = vec![];
    for (index, command) in file.commands.iter().enumerate() {
        result.append(&mut eval_command(command, index, range)?);
    }
    Ok(result)
}
