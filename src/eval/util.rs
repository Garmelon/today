use chrono::{Datelike, NaiveDate};

pub fn is_leap_year(year: i32) -> bool {
    NaiveDate::from_ymd_opt(year, 2, 29).is_some()
}

pub fn add_months(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let month0 = (month as i32) - 1 + delta;
    let year = year + month0.div_euclid(12);
    let month = month0.rem_euclid(12) as u32 + 1;
    (year, month)
}

pub fn month_length(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd(year + 1, 1, 1))
        .pred()
        .day()
}
