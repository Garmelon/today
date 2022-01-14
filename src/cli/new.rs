use std::result;
use std::str::FromStr;

use chrono::NaiveDate;
use codespan_reporting::files::SimpleFile;

use crate::files::cli::CliCommand;
use crate::files::commands::{Command, DateSpec, Done, DoneKind, Note, Spec, Statement, Task};
use crate::files::{Files, ParseError};

use super::error::{Error, Result};
use super::util;

fn edit<R, F>(name: &str, mut text: String, validate: F) -> Result<Option<R>>
where
    R: FromStr<Err = ParseError<()>>,
    F: Fn(&R) -> result::Result<(), &str>,
{
    Ok(loop {
        text = util::edit(&text)?;
        match text.parse() {
            Ok(command) => match validate(&command) {
                Ok(()) => break Some(command),
                Err(msg) => eprintln!("{msg}"),
            },
            Err(e) => crate::error::eprint_error(&SimpleFile::new(name, &text), &e),
        }
        if !matches!(
            promptly::prompt_default("Continue editing?", true),
            Ok(true)
        ) {
            println!("Aborting");
            break None;
        }
    })
}

fn is_task_or_note(command: &CliCommand) -> result::Result<(), &str> {
    match command.0 {
        Command::Task(_) | Command::Note(_) => Ok(()),
        _ => Err("Only TASK and NOTE are allowed"),
    }
}

fn new_command(files: &mut Files, command: Command) -> Result<()> {
    let capture = files.capture().ok_or(Error::NoCaptureFile)?;

    let command = edit("new command", format!("{command}"), is_task_or_note)?;
    if let Some(command) = command {
        files.insert(capture, command.0)
    }

    Ok(())
}

pub fn task(files: &mut Files, date: Option<NaiveDate>) -> Result<()> {
    let statements = match date {
        Some(date) => vec![Statement::Date(Spec::Date(DateSpec {
            start: date,
            start_delta: None,
            start_time: None,
            end: None,
            end_delta: None,
            end_time: None,
            repeat: None,
        }))],
        None => vec![],
    };
    let command = Command::Task(Task {
        title: String::new(),
        statements,
        done: vec![],
        desc: vec![],
    });

    new_command(files, command)
}

pub fn note(files: &mut Files, date: Option<NaiveDate>) -> Result<()> {
    let statements = match date {
        Some(date) => vec![Statement::Date(Spec::Date(DateSpec {
            start: date,
            start_delta: None,
            start_time: None,
            end: None,
            end_delta: None,
            end_time: None,
            repeat: None,
        }))],
        None => vec![],
    };
    let command = Command::Note(Note {
        title: String::new(),
        statements,
        desc: vec![],
    });

    new_command(files, command)
}

pub fn done(files: &mut Files, date: NaiveDate) -> Result<()> {
    let command = Command::Task(Task {
        title: String::new(),
        statements: vec![],
        done: vec![Done {
            kind: DoneKind::Done,
            date: None,
            done_at: date,
        }],
        desc: vec![],
    });

    new_command(files, command)
}
