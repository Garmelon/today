use chrono::NaiveDateTime;

use crate::eval::{DateRange, Entry};
use crate::files::Files;

use self::day::DayLayout;
use self::line::LineLayout;

mod day;
pub mod line;

pub fn layout(
    files: &Files,
    entries: &[Entry],
    range: DateRange,
    now: NaiveDateTime,
) -> LineLayout {
    let mut day_layout = DayLayout::new(range, now);
    day_layout.layout(entries);

    let mut line_layout = LineLayout::new();
    line_layout.render(files, entries, &day_layout);

    line_layout
}
