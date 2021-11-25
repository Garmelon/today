use chrono::NaiveDate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub hour: u8,
    pub min: u8,
}

impl Time {
    pub fn new(hour: u32, min: u32) -> Option<Self> {
        if hour < 24 && min < 60 || hour == 24 && min == 0 {
            Some(Self {
                hour: hour as u8,
                min: min as u8,
            })
        } else {
            None
        }
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

impl Weekday {
    pub fn name(&self) -> &'static str {
        match self {
            Weekday::Monday => "mon",
            Weekday::Tuesday => "tue",
            Weekday::Wednesday => "wed",
            Weekday::Thursday => "thu",
            Weekday::Friday => "fri",
            Weekday::Saturday => "sat",
            Weekday::Sunday => "sun",
        }
    }
}

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
            DeltaStep::Year(i) => *i,
            DeltaStep::Month(i) => *i,
            DeltaStep::MonthReverse(i) => *i,
            DeltaStep::Day(i) => *i,
            DeltaStep::Week(i) => *i,
            DeltaStep::Hour(i) => *i,
            DeltaStep::Minute(i) => *i,
            DeltaStep::Weekday(i, _) => *i,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DeltaStep::Year(_) => "y",
            DeltaStep::Month(_) => "m",
            DeltaStep::MonthReverse(_) => "M",
            DeltaStep::Day(_) => "d",
            DeltaStep::Week(_) => "w",
            DeltaStep::Hour(_) => "h",
            DeltaStep::Minute(_) => "min",
            DeltaStep::Weekday(_, wd) => wd.name(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Delta(pub Vec<DeltaStep>);

#[derive(Debug)]
pub struct Repeat {
    /// Start at the date when the latest `DONE` was created instead of the
    /// task's previous occurrence.
    pub start_at_done: bool,
    pub delta: Delta,
}

#[derive(Debug)]
pub struct DateSpec {
    pub start: NaiveDate,
    pub start_delta: Option<Delta>,
    pub start_time: Option<Time>,
    pub end: Option<NaiveDate>,
    pub end_delta: Option<Delta>,
    pub end_time: Option<Time>,
    pub repeat: Option<Repeat>,
}

#[derive(Debug)]
pub struct WeekdaySpec {
    pub start: Weekday,
    pub start_time: Option<Time>,
    pub end: Option<Weekday>,
    pub end_delta: Option<Delta>,
    pub end_time: Option<Time>,
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
    /// Equal to `((md - 1) / 7) + 1`
    MonthWeek,
    /// `mW`, 1 during the last 7 days of the month, 2 during the previous etc.
    ///
    /// Equal to `((mD - 1) / 7) + 1`
    MonthWeekReverse,
    /// `d`, day of the month
    Day,
    /// `D`, day of the month starting from the end
    ///
    /// Equal to `ml - md + 1`
    DayReverse,
    /// `iy`, ISO 8601 year
    IsoYear,
    /// `iyl`, length of current ISO 8601 year **in weeks**
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
}

impl Var {
    pub fn name(&self) -> &'static str {
        match self {
            // Constants
            Var::True => "true",
            Var::False => "false",
            Var::Monday => "mon",
            Var::Tuesday => "tue",
            Var::Wednesday => "wed",
            Var::Thursday => "thu",
            Var::Friday => "fri",
            Var::Saturday => "sat",
            Var::Sunday => "sun",
            // Variables
            Var::JulianDay => "j",
            Var::Year => "y",
            Var::YearLength => "yl",
            Var::YearDay => "yd",
            Var::YearDayReverse => "yD",
            Var::YearWeek => "yw",
            Var::YearWeekReverse => "yW",
            Var::Month => "m",
            Var::MonthLength => "ml",
            Var::MonthWeek => "mw",
            Var::MonthWeekReverse => "mW",
            Var::Day => "d",
            Var::DayReverse => "D",
            Var::IsoYear => "iy",
            Var::IsoYearLength => "iyl",
            Var::IsoWeek => "iw",
            Var::Weekday => "wd",
            Var::Easter => "e",
            // Variables with "boolean" values
            Var::IsWeekday => "isWeekday",
            Var::IsWeekend => "isWeekend",
            Var::IsLeapYear => "isLeapYear",
        }
    }
}

impl From<Weekday> for Var {
    fn from(wd: Weekday) -> Self {
        match wd {
            Weekday::Monday => Self::Monday,
            Weekday::Tuesday => Self::Tuesday,
            Weekday::Wednesday => Self::Wednesday,
            Weekday::Thursday => Self::Thursday,
            Weekday::Friday => Self::Friday,
            Weekday::Saturday => Self::Saturday,
            Weekday::Sunday => Self::Sunday,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(i64),
    Var(Var),
    Paren(Box<Expr>),
    // Integer-y operations
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    // Comparisons
    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    // Boolean-y operations
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub struct FormulaSpec {
    pub start: Option<Expr>, // None: *
    pub start_delta: Option<Delta>,
    pub start_time: Option<Time>,
    pub end_delta: Option<Delta>,
    pub end_time: Option<Time>,
}

#[derive(Debug)]
pub enum Spec {
    Date(DateSpec),
    Weekday(WeekdaySpec),
    Formula(FormulaSpec),
}

#[derive(Debug, Clone, Copy)]
pub enum DoneDate {
    Date {
        root: NaiveDate,
    },
    DateWithTime {
        root: NaiveDate,
        root_time: Time,
    },
    DateToDate {
        root: NaiveDate,
        other: NaiveDate,
    },
    DateToDateWithTime {
        root: NaiveDate,
        root_time: Time,
        other: NaiveDate,
        other_time: Time,
    },
}

impl DoneDate {
    pub fn root(&self) -> NaiveDate {
        match self {
            DoneDate::Date { root } => *root,
            DoneDate::DateWithTime { root, .. } => *root,
            DoneDate::DateToDate { root, .. } => *root,
            DoneDate::DateToDateWithTime { root, .. } => *root,
        }
    }
}

#[derive(Debug)]
pub struct Done {
    pub date: Option<DoneDate>,
    pub done_at: NaiveDate,
}

#[derive(Debug)]
pub struct Task {
    pub title: String,
    pub when: Vec<Spec>,
    pub from: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
    pub except: Vec<NaiveDate>,
    pub done: Vec<Done>,
    pub desc: Vec<String>,
}

#[derive(Debug)]
pub struct Note {
    pub title: String,
    pub when: Vec<Spec>, // Should not be empty?
    pub from: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
    pub except: Vec<NaiveDate>,
    pub desc: Vec<String>,
}

#[derive(Debug)]
pub struct BirthdaySpec {
    pub date: NaiveDate,
    pub year_known: bool, // If year is unknown, use NaiveDate of year 0
}

#[derive(Debug)]
pub struct Birthday {
    pub title: String,
    pub when: BirthdaySpec,
    pub desc: Vec<String>,
}

#[derive(Debug)]
pub enum Command {
    Task(Task),
    Note(Note),
    Birthday(Birthday),
}

#[derive(Debug)]
pub struct File {
    pub includes: Vec<String>,
    pub timezone: Option<String>,
    pub commands: Vec<Command>,
}
