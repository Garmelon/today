use std::result;

use chrono::NaiveDate;

use crate::commands::{BirthdaySpec, Done, Spec};

#[derive(Debug)]
pub enum Line {
    Empty,
    Indented(String),

    Task(String),
    Note(String),
    Birthday(String),
    Date(Spec),
    BDate(BirthdaySpec),
    From(NaiveDate),
    Until(NaiveDate),
    Except(NaiveDate),
    Done(Done),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("line {line}: unknown command {name:?}")]
    UnknownCommand { line: usize, name: String },
    #[error("line {line}: unknown format")]
    UnknownFormat { line: usize },
}

type Result<T> = result::Result<T, Error>;

pub fn parse_lines(content: &str) -> Result<Vec<Line>> {
    content
        .lines()
        .enumerate()
        .map(|(line, content)| parse_line(line, content))
        .collect()
}

fn parse_line(line: usize, content: &str) -> Result<Line> {
    println!("Parsing line {:?}", content);

    if content.is_empty() {
        Ok(Line::Empty)
    } else if content.starts_with('\t') || content.starts_with(' ') {
        Ok(Line::Indented(content.to_string()))
    } else if let Some((name, rest)) = parse_command(content) {
        match name {
            "TASK" => Ok(Line::Task(rest.to_string())),
            "NOTE" => Ok(Line::Note(rest.to_string())),
            "BIRTHDAY" => Ok(Line::Birthday(rest.to_string())),
            "DATE" => parse_date(rest),
            "BDATE" => parse_bdate(rest),
            "FROM" => parse_datum(rest).map(Line::From),
            "UNTIL" => parse_datum(rest).map(Line::Until),
            "EXCEPT" => parse_datum(rest).map(Line::Except),
            "DONE" => parse_done(rest),
            _ => Err(Error::UnknownCommand {
                line,
                name: name.to_string(),
            }),
        }
    } else {
        Err(Error::UnknownFormat { line })
    }
}

fn parse_command(line: &str) -> Option<(&str, &str)> {
    if let Some(space) = line.find(' ') {
        let name = &line[..space];
        let content = &line[space + 1..];
        Some((name, content))
    } else {
        None
    }
}

fn parse_date(s: &str) -> Result<Line> {
    println!("  parsing date from {:?}", s);
    Ok(Line::Empty) // TODO
}

fn parse_bdate(s: &str) -> Result<Line> {
    println!("  parsing bdate from {:?}", s);
    Ok(Line::Empty) // TODO
}

fn parse_datum(s: &str) -> Result<NaiveDate> {
    println!("  parsing datum from {:?}", s);
    Ok(NaiveDate::from_ymd(2015, 3, 14)) // TODO
}

fn parse_done(s: &str) -> Result<Line> {
    println!("  parsing done from {:?}", s);
    Ok(Line::Empty) // TODO
}
