use chrono::NaiveDate;
use codespan_reporting::files::Files as CsFiles;
use colored::Colorize;

use crate::eval::{Entry, EntryKind};
use crate::files::commands::{Command, Log};
use crate::files::primitives::Spanned;
use crate::files::{Files, Sourced};

use super::error::Error;
use super::layout::line::LineLayout;
use super::util;

fn fmt_where(files: &Files, command: &Sourced<'_, Spanned<Command>>) -> String {
    let name = files.name(command.source.file()).expect("file exists");
    let line = files
        .line_number(command.source.file(), command.value.span.start)
        .expect("file exists and line is valid");
    format!("Line {} in {}", line, name)
}

fn print_desc(command: &Sourced<'_, Spanned<Command>>) {
    let desc: &[String] = match &command.value.value {
        Command::Task(task) => &task.desc,
        Command::Note(note) => &note.desc,
        Command::Log(log) => &log.desc,
        _ => &[],
    };
    if !desc.is_empty() {
        println!();
        for line in desc {
            println!("{}", line);
        }
    }
}

fn show_entry(files: &Files, entry: &Entry) {
    let command = files.command(entry.source);

    let kind = util::display_kind(LineLayout::entry_kind(entry));
    println!("{} {} {}", "Title:".bright_black(), kind, entry.title);

    let what = match entry.kind {
        EntryKind::Task => "Task".to_string(),
        EntryKind::TaskDone(date) => format!("Task, done {}", date),
        EntryKind::TaskCanceled(date) => format!("Task, canceled {}", date),
        EntryKind::Note => "Note".to_string(),
        EntryKind::Birthday(None) => "Birthday, age unknown".to_string(),
        EntryKind::Birthday(Some(age)) => format!("Birthday, age {}", age),
    };
    println!("{}  {}", "What:".bright_black(), what);

    let when = match entry.dates {
        None => "no date".to_string(),
        Some(date) => format!("{}", date.sorted()),
    };
    println!("{}  {}", "When:".bright_black(), when);

    println!("{} {}", "Where:".bright_black(), fmt_where(files, &command));

    print_desc(&command);
}

fn show_log(files: &Files, log: Sourced<'_, Log>) {
    let command = files.command(log.source);

    println!("{}  Log entry", "What:".bright_black());
    println!("{}  {}", "When:".bright_black(), log.value.date);

    println!("{} {}", "Where:".bright_black(), fmt_where(files, &command));

    print_desc(&command);
}

fn show_ident(files: &Files, entries: &[Entry], layout: &LineLayout, ident: Ident) {
    match ident {
        Ident::Number(n) => match layout.look_up_number::<()>(n) {
            Ok(index) => show_entry(files, &entries[index]),
            Err(e) => println!("{}", e),
        },
        Ident::Date(date) => match files.log(date) {
            Some(log) => show_log(files, log),
            None => println!("{}", Error::NoSuchLog::<()>(date)),
        },
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Ident {
    Number(usize),
    Date(NaiveDate),
}

pub fn show(files: &Files, entries: &[Entry], layout: &LineLayout, idents: &[Ident]) {
    if idents.is_empty() {
        // Nothing to do
        return;
    }

    show_ident(files, entries, layout, idents[0]);
    for &ident in idents.iter().skip(1) {
        println!();
        println!();
        println!();
        show_ident(files, entries, layout, ident);
    }
}
