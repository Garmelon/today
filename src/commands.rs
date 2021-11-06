use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

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
struct DateDelta {
    years: i32,
    months: i32,
    weeks: i32,
    days: i32,
}

#[derive(Debug)]
struct TimeDelta {
    hours: i32,
    minutes: i32,
}

#[derive(Debug)]
struct EndDateSpec {
    end: Option<NaiveDate>,
    delta: Option<Delta>,
}

#[derive(Debug)]
struct DateSpec {
    start: NaiveDate,
    start_time: Option<NaiveTime>,
    offset: Option<Delta>,
    end: Option<EndDateSpec>,
    repeat: Option<Delta>,
}

// #[derive(Debug)]
// struct FormulaSpec {
//     start: (), // TODO Formula
//     start_time: Option<NaiveTime>,
//     offset: Option<Delta>,
//     end: Option<Delta>,
// }

#[derive(Debug)]
enum Spec {
    Date(DateSpec),
    // Formula(FormulaSpec),
}

#[derive(Debug)]
struct Done {
    refering_to: Option<NaiveDate>,
    created_at: Option<NaiveDateTime>,
}

#[derive(Debug)]
struct Task {
    title: String,
    when: Option<Spec>,
    desc: Option<String>,
    dones: Vec<Done>,
}

#[derive(Debug)]
struct Note {
    title: String,
    when: Spec,
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
