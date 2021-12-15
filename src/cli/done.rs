use chrono::NaiveDateTime;

use crate::eval::Entry;
use crate::files::commands::Done;
use crate::files::Files;

use super::error::Result;
use super::layout::line::LineLayout;

pub fn mark_done(
    files: &mut Files,
    entries: &[Entry],
    layout: &LineLayout,
    number: usize,
    now: NaiveDateTime,
) -> Result<()> {
    let entry = &entries[layout.look_up_number(number)?];
    let done = Done {
        date: entry.dates.map(|dates| dates.into()),
        done_at: now.date(),
    };
    files.add_done(number, entry.source, done)?;
    Ok(())
}
