use chrono::NaiveDate;

use crate::eval::{Entry, EntryKind};
use crate::files::commands::{Command, Log};
use crate::files::{Files, Sourced};

use super::error::Error;
use super::layout::line::LineLayout;

fn show_command(command: &Command) {
    for line in format!("{}", command).lines() {
        println!("| {}", line);
    }
}

fn show_entry(files: &Files, entry: &Entry) {
    let command = files.command(entry.source);

    let kind = match entry.kind {
        EntryKind::Task | EntryKind::TaskDone(_) | EntryKind::TaskCanceled(_) => "TASK",
        EntryKind::Note => "NOTE",
        EntryKind::Birthday(_) => "BIRTHDAY",
    };
    println!("{} {}", kind, entry.title);

    if let Some(dates) = entry.dates {
        println!("DATE {}", dates.sorted());
    } else {
        println!("NO DATE");
    }

    match entry.kind {
        EntryKind::TaskDone(when) => println!("DONE {}", when),
        EntryKind::TaskCanceled(when) => println!("CANCELED {}", when),
        EntryKind::Birthday(Some(age)) => println!("AGE {}", age),
        _ => {}
    }

    println!("FROM COMMAND");
    show_command(&command.value.value);
}

fn show_log(files: &Files, log: Sourced<'_, Log>) {
    let command = files.command(log.source);

    println!("LOG {}", log.value.date);

    println!("FROM COMMAND");
    show_command(&command.value.value);
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
        show_ident(files, entries, layout, ident);
    }
}
