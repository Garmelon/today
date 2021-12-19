use chrono::{Datelike, NaiveDate, Weekday};

pub fn is_leap_year(year: i32) -> bool {
    NaiveDate::from_ymd_opt(year, 2, 29).is_some()
}

pub fn is_iso_leap_year(year: i32) -> bool {
    NaiveDate::from_isoywd_opt(year, 53, Weekday::Sun).is_some()
}

pub fn year_length(year: i32) -> u32 {
    NaiveDate::from_ymd(year, 12, 31).ordinal()
}

pub fn month_length(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd(year + 1, 1, 1))
        .pred()
        .day()
}

// Length of an ISO week year in days.
pub fn iso_year_length(year: i32) -> u32 {
    if is_iso_leap_year(year) {
        53 * 7
    } else {
        52 * 7
    }
}

pub fn add_months(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let month0 = (month as i32) - 1 + delta;
    let year = year + month0.div_euclid(12);
    let month = month0.rem_euclid(12) as u32 + 1;
    (year, month)
}
