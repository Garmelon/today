use std::vec;

use chrono::NaiveDateTime;

use crate::eval::Entry;
use crate::files::commands::{Done, DoneKind};
use crate::files::Files;

use super::error::Error;
use super::layout::line::LineLayout;

pub fn cancel<S>(
    files: &mut Files,
    entries: &[Entry],
    layout: &LineLayout,
    numbers: &[usize],
    now: NaiveDateTime,
) -> Result<(), Error<S>> {
    let mut not_tasks = vec![];
    for &number in numbers {
        let entry = &entries[layout.look_up_number(number)?];
        let done = Done {
            kind: DoneKind::Canceled,
            date: entry.dates.map(|dates| dates.into()),
            done_at: now.date(),
        };
        if !files.add_done(entry.source, done) {
            not_tasks.push(number);
        }
    }

    if not_tasks.is_empty() {
        Ok(())
    } else {
        Err(Error::NotATask(not_tasks))
    }
}
