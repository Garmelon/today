use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Debug)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Delta {
    pub years: i32,
    pub months: i32,
    pub weeks: i32,
    pub days: i32,
    pub hours: i32,
    pub minutes: i32,
}

#[derive(Debug)]
pub struct DateEndSpec {
    pub end: Option<NaiveDate>,
    pub delta: Option<Delta>,
    pub end_time: Option<NaiveTime>,
}

#[derive(Debug)]
pub struct DateSpec {
    pub start: NaiveDate,
    pub delta: Option<Delta>,
    pub start_time: Option<NaiveTime>,
    pub end: Option<DateEndSpec>,
    pub repeat: Option<Delta>,
}

#[derive(Debug)]
pub struct WeekdayEndSpec {
    pub end: Option<Weekday>,
    pub delta: Option<Delta>,
    pub end_time: Option<NaiveTime>,
}

#[derive(Debug)]
pub struct WeekdaySpec {
    pub start: Weekday,
    pub start_time: Option<NaiveTime>,
    pub end: Option<WeekdayEndSpec>,
}

#[derive(Debug)]
pub enum IntVar {
    /// `j`, see https://en.wikipedia.org/wiki/Julian_day
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
    /// `yw`, 1 during the last 7 days of the year, 2 during the previous etc.
    ///
    /// Equal to `((yD - 1) / 7) + 1`
    YearWeekReverse,
    /// `m`
    Month,
    /// `ml`, length of the current month in days
    MonthLength,
    /// `d` or `md`, day of the month
    MonthDay,
    /// `D` or `mD`, day of the month starting from the end
    ///
    /// Equal to `ml - md + 1`
    MonthDayReverse,
    /// `mw`, 1 during the first 7 days of the month, 2 during the next etc.
    ///
    /// Equal to `((md - 1) / 7) + 1`
    MonthWeek,
    /// `mW`, 1 during the last 7 days of the month, 2 during the previous etc.
    ///
    /// Equal to `((mD - 1) / 7) + 1`
    MonthWeekReverse,
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
}

#[derive(Debug)]
pub enum IntExpr {
    Lit(i64),
    Var(IntVar),
    Paren(Box<IntVar>),
    Neg(Box<IntExpr>),
    Add(Box<IntExpr>, Box<IntExpr>),
    Sub(Box<IntExpr>, Box<IntExpr>),
    Mul(Box<IntExpr>, Box<IntExpr>),
    Div(Box<IntExpr>, Box<IntExpr>),
    Mod(Box<IntExpr>, Box<IntExpr>),
    Ternary(Box<BoolExpr>, Box<IntExpr>, Box<IntExpr>),
}

#[derive(Debug)]
pub enum BoolVar {
    /// `isWeekday`, whether the current day is one of mon-fri
    IsWeekday,
    /// `isWeekend`, whether the current day is one of sat-sun
    IsWeekend,
    /// `isLeapYear`, whether the current year is a leap year
    IsLeapYear,
}

#[derive(Debug)]
pub enum BoolExpr {
    Lit(bool),
    Var(BoolVar),
    Paren(Box<BoolVar>),
    Eq(Box<IntExpr>, Box<IntExpr>),
    Neq(Box<IntExpr>, Box<IntExpr>),
    Lt(Box<IntExpr>, Box<IntExpr>),
    Lte(Box<IntExpr>, Box<IntExpr>),
    Gt(Box<IntExpr>, Box<IntExpr>),
    Gte(Box<IntExpr>, Box<IntExpr>),
    Not(Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Xor(Box<BoolExpr>, Box<BoolExpr>),
    BEq(Box<BoolExpr>, Box<BoolExpr>),
    BNeq(Box<BoolExpr>, Box<BoolExpr>),
}

#[derive(Debug)]
pub struct FormulaSpec {
    pub start: Option<BoolExpr>, // None: *
    pub start_time: Option<NaiveTime>,
    pub offset: Option<Delta>,
    pub end: Option<Delta>,
}

#[derive(Debug)]
pub enum Spec {
    Date(DateSpec),
    Weekday(WeekdaySpec),
    Formula(FormulaSpec),
}

#[derive(Debug)]
pub struct Done {
    pub refering_to: Option<NaiveDate>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug)]
pub struct Task {
    pub title: String,
    pub when: Vec<Spec>,
    pub done: Vec<Done>,
    pub desc: Option<String>,
}

#[derive(Debug)]
pub struct Note {
    pub title: String,
    pub when: Vec<Spec>, // Should not be empty?
    pub desc: Option<String>,
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
    pub desc: Option<String>,
}

#[derive(Debug)]
pub enum Command {
    Task(Task),
    Note(Note),
    Birthday(Birthday),
}
