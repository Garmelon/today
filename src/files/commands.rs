use chrono::NaiveDate;

use super::primitives::{Span, Spanned, Time, Weekday};

#[derive(Debug, Clone, Copy)]
pub enum DeltaStep {
    /// `y`, move by a year, keeping the same month and day
    Year(i32),
    /// `m`, move by a month, keeping the same day `d`
    Month(i32),
    /// `M`, move by a month, keeping the same day `D`
    MonthReverse(i32),
    /// `d`
    Day(i32),
    /// `w`, move by 7 days
    Week(i32),
    /// `h`
    Hour(i32),
    /// `m`
    Minute(i32),
    /// `mon`, `tue`, `wed`, `thu`, `fri`, `sat`, `sun`
    ///
    /// Move to the next occurrence of the specified weekday
    Weekday(i32, Weekday),
}

impl DeltaStep {
    pub fn amount(&self) -> i32 {
        match self {
            Self::Year(i) => *i,
            Self::Month(i) => *i,
            Self::MonthReverse(i) => *i,
            Self::Day(i) => *i,
            Self::Week(i) => *i,
            Self::Hour(i) => *i,
            Self::Minute(i) => *i,
            Self::Weekday(i, _) => *i,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Year(_) => "y",
            Self::Month(_) => "m",
            Self::MonthReverse(_) => "M",
            Self::Day(_) => "d",
            Self::Week(_) => "w",
            Self::Hour(_) => "h",
            Self::Minute(_) => "min",
            Self::Weekday(_, wd) => wd.name(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Delta(pub Vec<Spanned<DeltaStep>>);

#[derive(Debug)]
pub struct Repeat {
    /// Start at the date when the latest `DONE` was created instead of the
    /// task's previous occurrence.
    pub start_at_done: bool,
    pub delta: Spanned<Delta>,
}

#[derive(Debug)]
pub struct DateSpec {
    pub start: NaiveDate,
    pub start_delta: Option<Delta>,
    pub start_time: Option<Time>,
    pub end: Option<Spanned<NaiveDate>>,
    pub end_delta: Option<Delta>,
    pub end_time: Option<Spanned<Time>>,
    pub repeat: Option<Repeat>,
    // TODO Allow specifying amount of repetitions
}

#[derive(Debug)]
pub struct WeekdaySpec {
    pub start: Weekday,
    pub start_time: Option<Time>,
    pub end: Option<Spanned<Weekday>>,
    pub end_delta: Option<Delta>,
    pub end_time: Option<Spanned<Time>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Var {
    /// `true`, always 1
    True,
    /// `false`, always 0
    False,
    /// `mon`, always 1
    Monday,
    /// `tue`, always 2
    Tuesday,
    /// `wed`, always 3
    Wednesday,
    /// `thu`, always 4
    Thursday,
    /// `fri`, always 5
    Friday,
    /// `sat`, always 6
    Saturday,
    /// `sun`, always 7
    Sunday,
    /// `j`, see <https://en.wikipedia.org/wiki/Julian_day>
    JulianDay,
    /// `y`
    Year,
    /// `yl`, length of the current year in days
    ///
    /// Equal to `isLeapYear ? 366 : 365`
    YearLength,
    /// `yd`, day of the year
    YearDay,
    /// `yD`, day of the year starting from the end
    ///
    /// Equal to `yl - yd + 1`
    YearDayReverse,
    /// `yw`, 1 during the first 7 days of the year, 2 during the next etc.
    ///
    /// Equal to `((yd - 1) / 7) + 1`
    YearWeek,
    /// `yW`, 1 during the last 7 days of the year, 2 during the previous etc.
    ///
    /// Equal to `((yD - 1) / 7) + 1`
    YearWeekReverse,
    /// `m`
    Month,
    /// `ml`, length of the current month in days
    MonthLength,
    /// `mw`, 1 during the first 7 days of the month, 2 during the next etc.
    ///
    /// Equal to `((d - 1) / 7) + 1`
    MonthWeek,
    /// `mW`, 1 during the last 7 days of the month, 2 during the previous etc.
    ///
    /// Equal to `((D - 1) / 7) + 1`
    MonthWeekReverse,
    /// `d`, day of the month
    Day,
    /// `D`, day of the month starting from the end
    ///
    /// Equal to `ml - d + 1`
    DayReverse,
    /// `iy`, ISO 8601 year
    IsoYear,
    /// `iyl`, length of current ISO 8601 year in days
    IsoYearLength,
    /// `iw`, ISO 8601 week
    IsoWeek,
    /// `wd`, day of the week, starting at monday with 1
    Weekday,
    /// `e`, day of the year that easter falls on
    Easter,
    /// `isWeekday`, whether the current day is one of mon-fri
    IsWeekday,
    /// `isWeekend`, whether the current day is one of sat-sun
    IsWeekend,
    /// `isLeapYear`, whether the current year is a leap year
    IsLeapYear,
    /// `isIsoLeapYear`, whether the current year is a long year in the ISO week system
    IsIsoLeapYear,
}

impl Var {
    pub fn name(&self) -> &'static str {
        match self {
            // Constants
            Self::True => "true",
            Self::False => "false",
            Self::Monday => "mon",
            Self::Tuesday => "tue",
            Self::Wednesday => "wed",
            Self::Thursday => "thu",
            Self::Friday => "fri",
            Self::Saturday => "sat",
            Self::Sunday => "sun",
            // Variables
            Self::JulianDay => "j",
            Self::Year => "y",
            Self::YearLength => "yl",
            Self::YearDay => "yd",
            Self::YearDayReverse => "yD",
            Self::YearWeek => "yw",
            Self::YearWeekReverse => "yW",
            Self::Month => "m",
            Self::MonthLength => "ml",
            Self::MonthWeek => "mw",
            Self::MonthWeekReverse => "mW",
            Self::Day => "d",
            Self::DayReverse => "D",
            Self::IsoYear => "iy",
            Self::IsoYearLength => "iyl",
            Self::IsoWeek => "iw",
            Self::Weekday => "wd",
            Self::Easter => "e",
            // Variables with "boolean" values
            Self::IsWeekday => "isWeekday",
            Self::IsWeekend => "isWeekend",
            Self::IsLeapYear => "isLeapYear",
            Self::IsIsoLeapYear => "isIsoLeapYear",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(i64),
    Var(Var),
    Paren(Box<Spanned<Expr>>),
    // Integer-y operations
    Neg(Box<Spanned<Expr>>),
    Add(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Sub(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Mul(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Div(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Mod(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    // Comparisons
    Eq(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Neq(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Lt(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Lte(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Gt(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Gte(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    // Boolean-y operations
    Not(Box<Spanned<Expr>>),
    And(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Or(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Xor(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
}

#[derive(Debug)]
pub struct FormulaSpec {
    pub start: Option<Spanned<Expr>>, // None: *
    pub start_delta: Option<Delta>,
    pub start_time: Option<Time>,
    pub end_delta: Option<Delta>,
    pub end_time: Option<Spanned<Time>>,
}

#[derive(Debug)]
pub enum Spec {
    Date(DateSpec),
    Weekday(WeekdaySpec),
    Formula(FormulaSpec),
}

#[derive(Debug)]
pub struct BirthdaySpec {
    pub date: NaiveDate,
    pub year_known: bool, // If year is unknown, use NaiveDate of year 0
}

#[derive(Debug)]
pub enum Statement {
    Date(Spec),
    BDate(BirthdaySpec),
    // TODO Allow specifying delta and repetitions for FROM and UNTIL
    From(Option<NaiveDate>),
    Until(Option<NaiveDate>),
    // TODO Allow excluding ranges (maybe with --range syntax?)
    Except(NaiveDate),
    Move {
        span: Span,
        from: NaiveDate,
        to: Option<NaiveDate>,
        to_time: Option<Spanned<Time>>,
    },
    Remind(Option<Spanned<Delta>>),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy)]
pub enum DoneDate {
    Date {
        root: NaiveDate,
    },
    DateTime {
        root: NaiveDate,
        root_time: Time,
    },
    DateToDate {
        root: NaiveDate,
        other: NaiveDate,
    },
    DateTimeToTime {
        root: NaiveDate,
        root_time: Time,
        other_time: Time,
    },
    DateTimeToDateTime {
        root: NaiveDate,
        root_time: Time,
        other: NaiveDate,
        other_time: Time,
    },
}

impl DoneDate {
    pub fn root(self) -> NaiveDate {
        match self {
            Self::Date { root } => root,
            Self::DateTime { root, .. } => root,
            Self::DateToDate { root, .. } => root,
            Self::DateTimeToTime { root, .. } => root,
            Self::DateTimeToDateTime { root, .. } => root,
        }
    }

    /// Remove redundancies like the same date or time specified twice.
    pub fn simplified(self) -> Self {
        let result = match self {
            Self::DateToDate { root, other } if root == other => Self::Date { root },
            Self::DateTimeToDateTime {
                root,
                root_time,
                other,
                other_time,
            } if root == other => Self::DateTimeToTime {
                root,
                root_time,
                other_time,
            },
            other => other,
        };

        match result {
            Self::DateTimeToTime {
                root,
                root_time,
                other_time,
            } if root_time == other_time => Self::DateTime { root, root_time },
            other => other,
        }
    }
}

#[derive(Debug)]
pub enum DoneKind {
    Done,
    Canceled,
}

#[derive(Debug)]
pub struct Done {
    pub kind: DoneKind,
    /// The date of the task the DONE refers to.
    pub date: Option<DoneDate>,
    /// When the task was actually completed.
    pub done_at: NaiveDate,
}

#[derive(Debug)]
pub struct Task {
    pub title: String,
    pub statements: Vec<Statement>,
    pub done: Vec<Done>,
    pub desc: Vec<String>,
}

#[derive(Debug)]
pub struct Note {
    pub title: String,
    pub statements: Vec<Statement>,
    pub desc: Vec<String>,
}

#[derive(Debug)]
pub struct Log {
    pub date: Spanned<NaiveDate>,
    pub desc: Vec<String>,
}

#[derive(Debug)]
pub enum Command {
    Include(Spanned<String>),
    Timezone(Spanned<String>),
    Capture, // TODO Set capture file by template?
    Task(Task),
    Note(Note),
    Log(Log),
}

#[derive(Debug)]
pub struct File {
    pub commands: Vec<Spanned<Command>>,
}

impl File {
    /// Create an empty dummy file. This file should only be used as a
    /// placeholder value.
    pub fn dummy() -> Self {
        Self { commands: vec![] }
    }
}
