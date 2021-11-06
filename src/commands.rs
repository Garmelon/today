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
    Year(i32),
    Month(i32),
    Day(i32),
    Week(i32),
    Hour(i32),
    Minute(i32),
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
    YearLength,
    /// `yd`, day of the year
    YearDay,
    /// `m`
    Month,
    /// `ml`, length of the current month in days
    MonthLength,
    /// `d`, day of the month
    MonthDay,
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
    IsWeekday,
    IsWeekend,
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
