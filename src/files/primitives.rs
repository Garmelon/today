use std::cmp::{self, Ordering};
use std::fmt;

use chrono::{NaiveTime, Timelike};

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl<'a> From<&pest::Span<'a>> for Span {
    fn from(pspan: &pest::Span<'a>) -> Self {
        Self {
            start: pspan.start(),
            end: pspan.end(),
        }
    }
}

impl Span {
    pub fn join(self, other: Self) -> Self {
        Self {
            start: cmp::min(self.start, other.start),
            end: cmp::max(self.end, other.end),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

impl<T: fmt::Debug> fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.value.fmt(f)
    }
}

impl<T> Spanned<T> {
    pub fn new(span: Span, value: T) -> Self {
        Self { span, value }
    }
}

// I don't know how one would write this. It works as a polymorphic standalone
// function, but not in an impl block.
// impl<S, T: Into<S>> Spanned<T> {
//     pub fn convert(&self) -> Spanned<S> {
//         Self::new(self.span, self.value.into())
//     }
// }

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    pub hour: u8,
    pub min: u8,
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.min)
    }
}

impl From<NaiveTime> for Time {
    fn from(t: NaiveTime) -> Self {
        Self::new(t.hour(), t.minute())
    }
}

impl Time {
    pub fn new(hour: u32, min: u32) -> Self {
        Self {
            hour: hour as u8,
            min: min as u8,
        }
    }

    /// Whether this time is within the normal range for times. This means that
    /// the minutes are always smaller than 60 and the whole time is between
    /// `00:00` and `24:00` (inclusive).
    ///
    /// In cases like leap seconds or daylight savings time, it is possible that
    /// times outside of this range occur.
    pub fn in_normal_range(&self) -> bool {
        if self.min >= 60 {
            return false;
        }
        if self.hour > 24 {
            return false;
        }
        if self.hour == 24 && self.min != 0 {
            return false;
        }
        true
    }

    /// How many minutes into the day this time is.
    fn minutes(&self) -> i64 {
        (self.hour as i64) * 60 + (self.min as i64)
    }

    pub fn add_minutes(&self, amount: i64) -> (i64, Self) {
        match amount.cmp(&0) {
            Ordering::Less => {
                let mut mins = self.minutes() + amount;

                let days = mins.div_euclid(60 * 24);
                mins = mins.rem_euclid(60 * 24);

                let hour = mins.div_euclid(60) as u32;
                let min = mins.rem_euclid(60) as u32;
                (days, Self::new(hour, min))
            }
            Ordering::Greater => {
                let mut mins = self.minutes() + amount;

                let mut days = mins.div_euclid(60 * 24);
                mins = mins.rem_euclid(60 * 24);

                // Correct days and minutes so we get 24:00 instead of 00:00
                if mins == 0 {
                    days -= 1;
                    mins = 60 * 24;
                }

                let hour = mins.div_euclid(60) as u32;
                let min = mins.rem_euclid(60) as u32;
                (days, Self::new(hour, min))
            }
            Ordering::Equal => (0, *self),
        }
    }

    pub fn add_hours(&self, amount: i64) -> (i64, Self) {
        self.add_minutes(amount * 60)
    }

    /// `a.minutes_to(b)` returns the minutes from `a` to `b`, meaning it is
    /// greater than 0 if `a` is earlier than `b`.
    ///
    /// May return weird amounts if [`Self::in_normal_range`] is not true for
    /// both.
    pub fn minutes_to(&self, other: Self) -> i64 {
        other.minutes() - self.minutes()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<chrono::Weekday> for Weekday {
    fn from(wd: chrono::Weekday) -> Self {
        match wd {
            chrono::Weekday::Mon => Self::Monday,
            chrono::Weekday::Tue => Self::Tuesday,
            chrono::Weekday::Wed => Self::Wednesday,
            chrono::Weekday::Thu => Self::Thursday,
            chrono::Weekday::Fri => Self::Friday,
            chrono::Weekday::Sat => Self::Saturday,
            chrono::Weekday::Sun => Self::Sunday,
        }
    }
}

impl Weekday {
    /// Short name (`mon`, `tue`, `wed`, `fri`, `sat`, `sun`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Monday => "mon",
            Self::Tuesday => "tue",
            Self::Wednesday => "wed",
            Self::Thursday => "thu",
            Self::Friday => "fri",
            Self::Saturday => "sat",
            Self::Sunday => "sun",
        }
    }

    /// Full name (`Monday`, `Tuesday`, `Wednesday`, `Thursday`, `Friday`,
    /// `Saturday`, `Sunday`).
    pub fn full_name(self) -> &'static str {
        match self {
            Weekday::Monday => "Monday",
            Weekday::Tuesday => "Tuesday",
            Weekday::Wednesday => "Wednesday",
            Weekday::Thursday => "Thursday",
            Weekday::Friday => "Friday",
            Weekday::Saturday => "Saturday",
            Weekday::Sunday => "Sunday",
        }
    }

    pub fn num(self) -> u8 {
        match self {
            Self::Monday => 1,
            Self::Tuesday => 2,
            Self::Wednesday => 3,
            Self::Thursday => 4,
            Self::Friday => 5,
            Self::Saturday => 6,
            Self::Sunday => 7,
        }
    }

    pub fn is_weekend(self) -> bool {
        matches!(self, Self::Saturday | Self::Sunday)
    }

    /// How many days from now until the other weekday.
    pub fn until(self, other: Self) -> u8 {
        let num_self = self.num();
        let num_other = other.num();
        if num_self <= num_other {
            num_other - num_self
        } else {
            num_other + 7 - num_self
        }
    }
}
