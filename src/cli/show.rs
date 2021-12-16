use crate::eval::{Entry, EntryKind};
use crate::files::Files;

use super::error::Result;
use super::layout::line::LineLayout;

fn show_entry(files: &Files, entry: &Entry) {
    let command = files.command(entry.source);

    match entry.kind {
        EntryKind::Task => println!("TASK {}", command.title()),
        EntryKind::TaskDone(when) => {
            println!("DONE {}", command.title());
            println!("DONE AT {}", when);
        }
        EntryKind::Note => println!("NOTE {}", command.title()),
        EntryKind::Birthday(Some(age)) => {
            println!("BIRTHDAY {}", command.title());
            println!("AGE {}", age);
        }
        EntryKind::Birthday(None) => {
            println!("BIRTHDAY {}", command.title());
            println!("AGE UNKNOWN");
        }
    }

    if let Some(dates) = entry.dates {
        println!("DATE {}", dates.sorted());
    } else {
        println!("NO DATE");
    }

    for line in command.desc() {
        println!("# {}", line);
    }
}

pub fn show(
    files: &Files,
    entries: &[Entry],
    layout: &LineLayout,
    numbers: &[usize],
) -> Result<()> {
    if numbers.is_empty() {
        // Nothing to do
        return Ok(());
    }

    let indices = numbers
        .iter()
        .map(|n| layout.look_up_number(*n))
        .collect::<Result<Vec<usize>>>()?;

    show_entry(files, &entries[indices[0]]);
    for &index in indices.iter().skip(1) {
        println!();
        show_entry(files, &entries[index]);
    }

    Ok(())
}
