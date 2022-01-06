use chrono::NaiveDateTime;

use crate::eval::{DateRange, Entry};

use self::day::DayLayout;
use self::line::LineLayout;

mod day;
pub mod line;

pub fn layout(entries: &[Entry], range: DateRange, now: NaiveDateTime) -> LineLayout {
    let mut day_layout = DayLayout::new(range, now);
    day_layout.layout(entries);

    let mut line_layout = LineLayout::new();
    line_layout.render(entries, &day_layout);

    line_layout
}
