use crate::eval::{Entry, EntryKind};
use crate::files::Files;

use super::error::Error;
use super::layout::line::LineLayout;

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
    for line in format!("{}", command.command).lines() {
        println!("| {}", line);
    }
}

pub fn show<S>(
    files: &Files,
    entries: &[Entry],
    layout: &LineLayout,
    numbers: &[usize],
) -> Result<(), Error<S>> {
    if numbers.is_empty() {
        // Nothing to do
        return Ok(());
    }

    let indices = numbers
        .iter()
        .map(|n| layout.look_up_number(*n))
        .collect::<Result<Vec<usize>, _>>()?;

    show_entry(files, &entries[indices[0]]);
    for &index in indices.iter().skip(1) {
        println!();
        show_entry(files, &entries[index]);
    }

    Ok(())
}
