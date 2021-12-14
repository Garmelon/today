use chrono::NaiveDateTime;

use crate::eval::Entry;
use crate::files::commands::Done;
use crate::files::Files;

use super::error::Result;

pub fn mark_done(files: &mut Files, entry: &Entry, now: NaiveDateTime) -> Result<()> {
    let done = Done {
        date: entry.dates.map(|dates| dates.into()),
        done_at: now.date(),
    };
    files.add_done(entry.source, done)?;
    Ok(())
}
