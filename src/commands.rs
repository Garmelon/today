use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Debug)]
enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug)]
enum DeltaStep {
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
struct DateEndSpec {
    end: Option<NaiveDate>,
    delta: Option<Delta>,
    end_time: Option<NaiveTime>,
}

#[derive(Debug)]
struct DateSpec {
    start: NaiveDate,
    delta: Option<Delta>,
    start_time: Option<NaiveTime>,
    end: Option<DateEndSpec>,
    repeat: Option<Delta>,
}

#[derive(Debug)]
struct WeekdayEndSpec {
    end: Option<Weekday>,
    delta: Option<Delta>,
    end_time: Option<NaiveTime>,
}

#[derive(Debug)]
struct WeekdaySpec {
    start: Weekday,
    start_time: Option<NaiveTime>,
    end: Option<WeekdayEndSpec>,
}

#[derive(Debug)]
enum IntVar {
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
enum IntExpr {
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
enum BoolVar {
    /// `isWeekday`, whether the current day is one of mon-fri
    IsWeekday,
    /// `isWeekend`, whether the current day is one of sat-sun
    IsWeekend,
    /// `isLeapYear`, whether the current year is a leap year
    IsLeapYear,
}

#[derive(Debug)]
enum BoolExpr {
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
struct FormulaSpec {
    start: Option<BoolExpr>, // None: *
    start_time: Option<NaiveTime>,
    offset: Option<Delta>,
    end: Option<Delta>,
}

#[derive(Debug)]
enum Spec {
    Date(DateSpec),
    Weekday(WeekdaySpec),
    Formula(FormulaSpec),
}

#[derive(Debug)]
struct Done {
    refering_to: Option<NaiveDate>,
    created_at: Option<NaiveDateTime>,
}

#[derive(Debug)]
struct Task {
    title: String,
    when: Vec<Spec>,
    done: Vec<Done>,
    desc: Option<String>,
}

#[derive(Debug)]
struct Note {
    title: String,
    when: Vec<Spec>, // Not empty
    desc: Option<String>,
}

#[derive(Debug)]
struct BirthdaySpec {
    date: NaiveDate,
    year_known: bool, // If year is unknown, use NaiveDate of year 0
}

#[derive(Debug)]
struct Birthday {
    title: String,
    when: BirthdaySpec,
    desc: Option<String>,
}

#[derive(Debug)]
enum Command {
    Task(Task),
    Note(Note),
    Birthday(Birthday),
}
