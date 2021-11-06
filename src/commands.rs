use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

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
enum EndDate {
    Fixed(NaiveDate),
    Delta(DateDelta),
}

#[derive(Debug)]
enum EndTime {
    Fixed(NaiveTime),
    Delta(TimeDelta),
}

#[derive(Debug)]
struct DateSpec {
    start: NaiveDate,
    end: Option<EndDate>,
    repeat: Option<DateDelta>,
}

#[derive(Debug)]
struct TimeSpec {
    start: NaiveTime,
    end: Option<EndTime>,
}

#[derive(Debug)]
struct WhenSpec {
    date: DateSpec,
    time: Option<TimeSpec>,
}

#[derive(Debug)]
struct Done {
    refering_to: Option<NaiveDate>,
    created_at: Option<NaiveDateTime>,
}

#[derive(Debug)]
struct Task {
    title: String,
    when: Option<WhenSpec>,
    desc: Option<String>,
    dones: Vec<Done>,
}

#[derive(Debug)]
struct Note {
    title: String,
    when: WhenSpec,
    desc: Option<String>,
}

#[derive(Debug)]
enum BirthdaySpec {
    Date(NaiveDate),
    DateWithoutYear { month: u8, day: u8 },
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
