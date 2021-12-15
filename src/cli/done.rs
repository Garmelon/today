use std::vec;

use chrono::NaiveDateTime;

use crate::eval::Entry;
use crate::files::commands::Done;
use crate::files::Files;

use super::error::{Error, Result};
use super::layout::line::LineLayout;

pub fn mark_done(
    files: &mut Files,
    entries: &[Entry],
    layout: &LineLayout,
    numbers: &[usize],
    now: NaiveDateTime,
) -> Result<()> {
    let mut not_tasks = vec![];
    for &number in numbers {
        let entry = &entries[layout.look_up_number(number)?];
        let done = Done {
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
