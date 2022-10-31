use chrono::{Datelike, Duration, NaiveDate};

use crate::files::commands;
use crate::files::primitives::{Span, Spanned, Time, Weekday};
use crate::files::FileSource;

use super::super::command::CommandState;
use super::super::date::Dates;
use super::super::delta::{Delta, DeltaStep};
use super::super::{util, DateRange, Error};
use super::EvalCommand;

fn b2i(b: bool) -> i64 {
    if b {
        1
    } else {
        0
    }
}

fn i2b(i: i64) -> bool {
    i != 0
}

#[derive(Debug, Clone, Copy)]
pub enum Var {
    JulianDay,
    Year,
    YearLength,
    YearDay,
    YearDayReverse,
    YearWeek,
    YearWeekReverse,
    Month,
    MonthLength,
    MonthWeek,
    MonthWeekReverse,
    Day,
    DayReverse,
    IsoYear,
    IsoYearLength,
    IsoWeek,
    Weekday,
    Easter(Span),
    IsWeekday,
    IsWeekend,
    IsLeapYear,
    IsIsoLeapYear,
}

impl Var {
    fn eval<S>(self, index: S, date: NaiveDate) -> Result<i64, Error<S>> {
        Ok(match self {
            Self::JulianDay => date.num_days_from_ce().into(),
            Self::Year => date.year().into(),
            Self::YearLength => util::year_length(date.year()).into(),
            Self::YearDay => date.ordinal().into(),
            Self::YearDayReverse => (util::year_length(date.year()) - date.ordinal0()).into(),
            Self::YearWeek => (date.ordinal0().div_euclid(7) + 1).into(),
            Self::YearWeekReverse => {
                #[allow(non_snake_case)]
                let yD = util::year_length(date.year()) - date.ordinal();
                (yD.div_euclid(7) + 1).into()
            }
            Self::Month => date.month().into(),
            Self::MonthLength => util::month_length(date.year(), date.month()).into(),
            Self::MonthWeek => (date.day0().div_euclid(7) + 1).into(),
            Self::MonthWeekReverse => {
                #[allow(non_snake_case)]
                let mD = util::month_length(date.year(), date.month()) - date.day();
                (mD.div_euclid(7) + 1).into()
            }
            Self::Day => date.day().into(),
            Self::DayReverse => {
                let ml = util::month_length(date.year(), date.month());
                (ml - date.day0()).into()
            }
            Self::IsoYear => date.iso_week().year().into(),
            Self::IsoYearLength => util::iso_year_length(date.iso_week().year()).into(),
            Self::IsoWeek => date.iso_week().week().into(),
            Self::Weekday => {
                let wd: Weekday = date.weekday().into();
                wd.num().into()
            }
            Self::Easter(span) => {
                let e = computus::gregorian(date.year()).map_err(|e| Error::Easter {
                    index,
                    span,
                    date,
                    msg: e,
                })?;
                NaiveDate::from_ymd(e.year, e.month, e.day).ordinal().into()
            }
            Self::IsWeekday => {
                let wd: Weekday = date.weekday().into();
                b2i(!wd.is_weekend())
            }
            Self::IsWeekend => {
                let wd: Weekday = date.weekday().into();
                b2i(wd.is_weekend())
            }
            Self::IsLeapYear => b2i(util::is_leap_year(date.year())),
            Self::IsIsoLeapYear => b2i(util::is_iso_leap_year(date.year())),
        })
    }
}

#[derive(Debug)]
pub enum Expr {
    Lit(i64),
    Var(Var),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>, Span),
    Mod(Box<Expr>, Box<Expr>, Span),
    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
}

impl From<&Spanned<commands::Expr>> for Expr {
    fn from(expr: &Spanned<commands::Expr>) -> Self {
        fn conv(expr: &Spanned<commands::Expr>) -> Box<Expr> {
            Box::new(expr.into())
        }

        match &expr.value {
            commands::Expr::Lit(l) => Self::Lit(*l),
            commands::Expr::Var(v) => match v {
                commands::Var::True => Self::Lit(1),
                commands::Var::False => Self::Lit(0),
                commands::Var::Monday => Self::Lit(1),
                commands::Var::Tuesday => Self::Lit(2),
                commands::Var::Wednesday => Self::Lit(3),
                commands::Var::Thursday => Self::Lit(4),
                commands::Var::Friday => Self::Lit(5),
                commands::Var::Saturday => Self::Lit(6),
                commands::Var::Sunday => Self::Lit(7),
                commands::Var::JulianDay => Self::Var(Var::JulianDay),
                commands::Var::Year => Self::Var(Var::Year),
                commands::Var::YearLength => Self::Var(Var::YearLength),
                commands::Var::YearDay => Self::Var(Var::YearDay),
                commands::Var::YearDayReverse => Self::Var(Var::YearDayReverse),
                commands::Var::YearWeek => Self::Var(Var::YearWeek),
                commands::Var::YearWeekReverse => Self::Var(Var::YearWeekReverse),
                commands::Var::Month => Self::Var(Var::Month),
                commands::Var::MonthLength => Self::Var(Var::MonthLength),
                commands::Var::MonthWeek => Self::Var(Var::MonthWeek),
                commands::Var::MonthWeekReverse => Self::Var(Var::MonthWeekReverse),
                commands::Var::Day => Self::Var(Var::Day),
                commands::Var::DayReverse => Self::Var(Var::DayReverse),
                commands::Var::IsoYear => Self::Var(Var::IsoYear),
                commands::Var::IsoYearLength => Self::Var(Var::IsoYearLength),
                commands::Var::IsoWeek => Self::Var(Var::IsoWeek),
                commands::Var::Weekday => Self::Var(Var::Weekday),
                commands::Var::Easter => Self::Var(Var::Easter(expr.span)),
                commands::Var::IsWeekday => Self::Var(Var::IsWeekday),
                commands::Var::IsWeekend => Self::Var(Var::IsWeekend),
                commands::Var::IsLeapYear => Self::Var(Var::IsLeapYear),
                commands::Var::IsIsoLeapYear => Self::Var(Var::IsIsoLeapYear),
            },
            commands::Expr::Paren(i) => i.as_ref().into(),
            commands::Expr::Neg(i) => Self::Neg(conv(i)),
            commands::Expr::Add(a, b) => Self::Add(conv(a), conv(b)),
            commands::Expr::Sub(a, b) => Self::Sub(conv(a), conv(b)),
            commands::Expr::Mul(a, b) => Self::Mul(conv(a), conv(b)),
            commands::Expr::Div(a, b) => Self::Div(conv(a), conv(b), expr.span),
            commands::Expr::Mod(a, b) => Self::Mod(conv(a), conv(b), expr.span),
            commands::Expr::Eq(a, b) => Self::Eq(conv(a), conv(b)),
            commands::Expr::Neq(a, b) => Self::Neq(conv(a), conv(b)),
            commands::Expr::Lt(a, b) => Self::Lt(conv(a), conv(b)),
            commands::Expr::Lte(a, b) => Self::Lte(conv(a), conv(b)),
            commands::Expr::Gt(a, b) => Self::Gt(conv(a), conv(b)),
            commands::Expr::Gte(a, b) => Self::Gte(conv(a), conv(b)),
            commands::Expr::Not(i) => Self::Not(conv(i)),
            commands::Expr::And(a, b) => Self::And(conv(a), conv(b)),
            commands::Expr::Or(a, b) => Self::Or(conv(a), conv(b)),
            commands::Expr::Xor(a, b) => Self::Xor(conv(a), conv(b)),
        }
    }
}

impl From<Weekday> for Expr {
    fn from(wd: Weekday) -> Self {
        match wd {
            Weekday::Monday => Self::Lit(1),
            Weekday::Tuesday => Self::Lit(2),
            Weekday::Wednesday => Self::Lit(3),
            Weekday::Thursday => Self::Lit(4),
            Weekday::Friday => Self::Lit(5),
            Weekday::Saturday => Self::Lit(6),
            Weekday::Sunday => Self::Lit(7),
        }
    }
}

impl Expr {
    fn eval<S: Copy>(&self, index: S, date: NaiveDate) -> Result<i64, Error<S>> {
        Ok(match self {
            Self::Lit(l) => *l,
            Self::Var(v) => v.eval(index, date)?,
            Self::Neg(e) => -e.eval(index, date)?,
            Self::Add(a, b) => a.eval(index, date)? + b.eval(index, date)?,
            Self::Sub(a, b) => a.eval(index, date)? - b.eval(index, date)?,
            Self::Mul(a, b) => a.eval(index, date)? * b.eval(index, date)?,
            Self::Div(a, b, span) => {
                let b = b.eval(index, date)?;
                if b == 0 {
                    return Err(Error::DivByZero {
                        index,
                        span: *span,
                        date,
                    });
                }
                a.eval(index, date)?.div_euclid(b)
            }
            Self::Mod(a, b, span) => {
                let b = b.eval(index, date)?;
                if b == 0 {
                    return Err(Error::ModByZero {
                        index,
                        span: *span,
                        date,
                    });
                }
                a.eval(index, date)?.rem_euclid(b)
            }
            Self::Eq(a, b) => b2i(a.eval(index, date)? == b.eval(index, date)?),
            Self::Neq(a, b) => b2i(a.eval(index, date)? != b.eval(index, date)?),
            Self::Lt(a, b) => b2i(a.eval(index, date)? < b.eval(index, date)?),
            Self::Lte(a, b) => b2i(a.eval(index, date)? <= b.eval(index, date)?),
            Self::Gt(a, b) => b2i(a.eval(index, date)? > b.eval(index, date)?),
            Self::Gte(a, b) => b2i(a.eval(index, date)? >= b.eval(index, date)?),
            Self::Not(e) => b2i(!i2b(e.eval(index, date)?)),
            Self::And(a, b) => b2i(i2b(a.eval(index, date)?) && i2b(b.eval(index, date)?)),
            Self::Or(a, b) => b2i(i2b(a.eval(index, date)?) || i2b(b.eval(index, date)?)),
            Self::Xor(a, b) => b2i(i2b(a.eval(index, date)?) ^ i2b(b.eval(index, date)?)),
        })
    }
}

pub struct FormulaSpec {
    pub start: Expr,
    pub start_delta: Delta,
    pub start_time: Option<Time>,
    pub end_delta: Delta,
}

impl From<&commands::FormulaSpec> for FormulaSpec {
    fn from(spec: &commands::FormulaSpec) -> Self {
        let start: Expr = match &spec.start {
            Some(expr) => expr.into(),
            None => Expr::Lit(1), // Always true
        };

        let start_delta: Delta = spec
            .start_delta
            .as_ref()
            .map(|delta| delta.into())
            .unwrap_or_default();

        let mut end_delta: Delta = spec
            .end_delta
            .as_ref()
            .map(|delta| delta.into())
            .unwrap_or_default();
        if let Some(time) = spec.end_time {
            end_delta
                .steps
                .push(Spanned::new(time.span, DeltaStep::Time(time.value)));
        }

        Self {
            start,
            start_delta,
            start_time: spec.start_time,
            end_delta,
        }
    }
}

impl From<&commands::WeekdaySpec> for FormulaSpec {
    fn from(spec: &commands::WeekdaySpec) -> Self {
        let start = Expr::Eq(
            Box::new(Expr::Var(Var::Weekday)),
            Box::new(spec.start.into()),
        );

        let mut end_delta = Delta::default();
        if let Some(wd) = spec.end {
            end_delta
                .steps
                .push(Spanned::new(wd.span, DeltaStep::Weekday(1, wd.value)));
        }
        if let Some(delta) = &spec.end_delta {
            for step in &delta.0 {
                end_delta
                    .steps
                    .push(Spanned::new(step.span, step.value.into()));
            }
        }
        if let Some(time) = spec.end_time {
            end_delta
                .steps
                .push(Spanned::new(time.span, DeltaStep::Time(time.value)));
        }

        Self {
            start,
            start_delta: Default::default(),
            start_time: spec.start_time,
            end_delta,
        }
    }
}

impl FormulaSpec {
    fn range(&self, s: &CommandState<'_>) -> Option<DateRange> {
        let mut range = s
            .range_with_remind()
            .expand_by(&self.end_delta)
            .move_by(&self.start_delta);

        if let EvalCommand::Task(_) = s.command {
            if let Some(last_done_root) = s.command.last_done_root() {
                range = range.with_from(last_done_root.succ())?;
            } else if let Some(from) = s.from {
                range = range.with_from(from)?;
            } else if matches!(s.command, EvalCommand::Task(_)) {
                // We have no idea if we missed any tasks since the user hasn't
                // specified a `FROM`, so we just just look back one year. Any
                // task older than a year is probably not important anyways...
                range = range.with_from(range.from() - Duration::days(365))?;
            }
        }

        s.limit_from_until(range)
    }

    fn dates(&self, index: FileSource, start: NaiveDate) -> Result<Dates, Error<FileSource>> {
        let root = self.start_delta.apply_date(index, start)?;
        Ok(if let Some(root_time) = self.start_time {
            let (other, other_time) = self.end_delta.apply_date_time(index, root, root_time)?;
            Dates::new_with_time(root, root_time, other, other_time)
        } else {
            let other = self.end_delta.apply_date(index, root)?;
            Dates::new(root, other)
        })
    }

    fn eval(&self, index: FileSource, date: NaiveDate) -> Result<bool, Error<FileSource>> {
        Ok(i2b(self.start.eval(index, date)?))
    }
}

impl CommandState<'_> {
    pub fn eval_formula_spec(&mut self, spec: FormulaSpec) -> Result<(), Error<FileSource>> {
        if let Some(range) = spec.range(self) {
            let index = self.source.file();
            for day in range.days() {
                if spec.eval(index, day)? {
                    let dates = spec.dates(index, day)?;
                    self.add(self.entry_with_remind(self.command.kind(), Some(dates))?);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::panic;

    use chrono::{Datelike, Duration, NaiveDate};

    use crate::files::primitives::Span;

    use super::{Expr, Var};

    fn expr(expr: &Expr, date: NaiveDate, target: i64) {
        if let Ok(result) = expr.eval((), date) {
            assert_eq!(result, target);
        } else {
            panic!("formula produced error for day {date}");
        }
    }

    #[test]
    fn julian_day() {
        let e = Expr::Var(Var::JulianDay);

        for delta in -1000..1000 {
            let d1 = NaiveDate::from_ymd(2021, 12, 19);
            let d2 = d1 + Duration::days(delta);
            assert_eq!(e.eval((), d2).unwrap() - e.eval((), d1).unwrap(), delta);
        }
    }

    #[test]
    fn year() {
        let e = Expr::Var(Var::Year);

        for y in -3000..=3000 {
            expr(&e, NaiveDate::from_ymd(y, 2, 19), y.into());
        }

        expr(&e, NaiveDate::from_ymd(2021, 1, 1), 2021);
        expr(&e, NaiveDate::from_ymd(2021, 12, 31), 2021);
    }

    #[test]
    fn year_length() {
        let e = Expr::Var(Var::YearLength);

        expr(&e, NaiveDate::from_ymd(2000, 12, 19), 366);
        expr(&e, NaiveDate::from_ymd(2019, 12, 19), 365);
        expr(&e, NaiveDate::from_ymd(2020, 12, 19), 366);
        expr(&e, NaiveDate::from_ymd(2021, 12, 19), 365);
    }

    #[test]
    fn year_day() {
        let e = Expr::Var(Var::YearDay);

        for i in 1..=365 {
            expr(&e, NaiveDate::from_yo(2020, i), i.into());
            expr(&e, NaiveDate::from_yo(2021, i), i.into());
        }

        expr(&e, NaiveDate::from_yo(2020, 366), 366);
    }

    #[test]
    fn year_day_reverse() {
        let e = Expr::Var(Var::YearDayReverse);

        for i in 1..=365 {
            expr(&e, NaiveDate::from_yo(2020, i), (366 - i + 1).into());
            expr(&e, NaiveDate::from_yo(2021, i), (365 - i + 1).into());
        }

        expr(&e, NaiveDate::from_ymd(2020, 1, 1), 366);
        expr(&e, NaiveDate::from_ymd(2021, 1, 1), 365);
        expr(&e, NaiveDate::from_ymd(2020, 12, 31), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 31), 1);
    }

    #[test]
    fn year_week() {
        let e = Expr::Var(Var::YearWeek);

        for y in 1000..3000 {
            expr(&e, NaiveDate::from_ymd(y, 1, 1), 1);
            expr(&e, NaiveDate::from_ymd(y, 1, 2), 1);
            expr(&e, NaiveDate::from_ymd(y, 1, 3), 1);
            expr(&e, NaiveDate::from_ymd(y, 1, 4), 1);
            expr(&e, NaiveDate::from_ymd(y, 1, 5), 1);
            expr(&e, NaiveDate::from_ymd(y, 1, 6), 1);
            expr(&e, NaiveDate::from_ymd(y, 1, 7), 1);
            expr(&e, NaiveDate::from_ymd(y, 1, 8), 2);
            expr(&e, NaiveDate::from_ymd(y, 1, 9), 2);
            expr(&e, NaiveDate::from_ymd(y, 1, 10), 2);
            expr(&e, NaiveDate::from_ymd(y, 1, 11), 2);
            expr(&e, NaiveDate::from_ymd(y, 1, 12), 2);
            expr(&e, NaiveDate::from_ymd(y, 1, 13), 2);
            expr(&e, NaiveDate::from_ymd(y, 1, 14), 2);
            expr(&e, NaiveDate::from_ymd(y, 1, 15), 3);
        }

        expr(&e, NaiveDate::from_ymd(2020, 12, 28), 52);
        expr(&e, NaiveDate::from_ymd(2020, 12, 29), 52);
        expr(&e, NaiveDate::from_ymd(2020, 12, 30), 53);
        expr(&e, NaiveDate::from_ymd(2020, 12, 31), 53);

        expr(&e, NaiveDate::from_ymd(2021, 12, 28), 52);
        expr(&e, NaiveDate::from_ymd(2021, 12, 29), 52);
        expr(&e, NaiveDate::from_ymd(2021, 12, 30), 52);
        expr(&e, NaiveDate::from_ymd(2021, 12, 31), 53);
    }

    #[test]
    fn year_week_reverse() {
        let e = Expr::Var(Var::YearWeekReverse);

        for y in 1000..3000 {
            expr(&e, NaiveDate::from_ymd(y, 12, 31), 1);
            expr(&e, NaiveDate::from_ymd(y, 12, 30), 1);
            expr(&e, NaiveDate::from_ymd(y, 12, 29), 1);
            expr(&e, NaiveDate::from_ymd(y, 12, 28), 1);
            expr(&e, NaiveDate::from_ymd(y, 12, 27), 1);
            expr(&e, NaiveDate::from_ymd(y, 12, 26), 1);
            expr(&e, NaiveDate::from_ymd(y, 12, 25), 1);
            expr(&e, NaiveDate::from_ymd(y, 12, 24), 2);
            expr(&e, NaiveDate::from_ymd(y, 12, 23), 2);
            expr(&e, NaiveDate::from_ymd(y, 12, 22), 2);
            expr(&e, NaiveDate::from_ymd(y, 12, 21), 2);
            expr(&e, NaiveDate::from_ymd(y, 12, 20), 2);
            expr(&e, NaiveDate::from_ymd(y, 12, 19), 2);
            expr(&e, NaiveDate::from_ymd(y, 12, 18), 2);
            expr(&e, NaiveDate::from_ymd(y, 12, 17), 3);
        }

        expr(&e, NaiveDate::from_ymd(2020, 1, 1), 53);
        expr(&e, NaiveDate::from_ymd(2020, 1, 2), 53);
        expr(&e, NaiveDate::from_ymd(2020, 1, 3), 52);
        expr(&e, NaiveDate::from_ymd(2020, 1, 4), 52);

        expr(&e, NaiveDate::from_ymd(2021, 1, 1), 53);
        expr(&e, NaiveDate::from_ymd(2021, 1, 2), 52);
        expr(&e, NaiveDate::from_ymd(2021, 1, 3), 52);
        expr(&e, NaiveDate::from_ymd(2021, 1, 4), 52);
    }

    #[test]
    fn month() {
        let e = Expr::Var(Var::Month);
        for y in -1000..=3000 {
            for m in 1..=12 {
                expr(&e, NaiveDate::from_ymd(y, m, 13), m.into());
            }
        }
    }

    #[test]
    fn month_length() {
        let e = Expr::Var(Var::MonthLength);

        expr(&e, NaiveDate::from_ymd(2021, 1, 5), 31);
        expr(&e, NaiveDate::from_ymd(2021, 2, 5), 28);
        expr(&e, NaiveDate::from_ymd(2021, 3, 5), 31);
        expr(&e, NaiveDate::from_ymd(2021, 4, 5), 30);
        expr(&e, NaiveDate::from_ymd(2021, 5, 5), 31);
        expr(&e, NaiveDate::from_ymd(2021, 6, 5), 30);
        expr(&e, NaiveDate::from_ymd(2021, 7, 5), 31);
        expr(&e, NaiveDate::from_ymd(2021, 8, 5), 31);
        expr(&e, NaiveDate::from_ymd(2021, 9, 5), 30);
        expr(&e, NaiveDate::from_ymd(2021, 10, 5), 31);
        expr(&e, NaiveDate::from_ymd(2021, 11, 5), 30);
        expr(&e, NaiveDate::from_ymd(2021, 12, 5), 31);

        expr(&e, NaiveDate::from_ymd(2020, 2, 5), 29);
        expr(&e, NaiveDate::from_ymd(2019, 2, 5), 28);
        expr(&e, NaiveDate::from_ymd(2000, 2, 5), 29);
    }

    #[test]
    fn month_week() {
        let e = Expr::Var(Var::MonthWeek);

        expr(&e, NaiveDate::from_ymd(2021, 12, 1), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 2), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 3), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 4), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 5), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 6), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 7), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 8), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 9), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 10), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 11), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 12), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 13), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 14), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 15), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 16), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 17), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 18), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 19), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 20), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 21), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 22), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 23), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 24), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 25), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 26), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 27), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 28), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 29), 5);
        expr(&e, NaiveDate::from_ymd(2021, 12, 30), 5);
        expr(&e, NaiveDate::from_ymd(2021, 12, 31), 5);
    }

    #[test]
    fn month_week_reverse() {
        let e = Expr::Var(Var::MonthWeekReverse);

        expr(&e, NaiveDate::from_ymd(2021, 12, 1), 5);
        expr(&e, NaiveDate::from_ymd(2021, 12, 2), 5);
        expr(&e, NaiveDate::from_ymd(2021, 12, 3), 5);
        expr(&e, NaiveDate::from_ymd(2021, 12, 4), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 5), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 6), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 7), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 8), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 9), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 10), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 11), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 12), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 13), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 14), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 15), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 16), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 17), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 18), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 19), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 20), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 21), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 22), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 23), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 24), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 25), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 26), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 27), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 28), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 29), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 30), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 31), 1);
    }

    #[test]
    fn day() {
        let e = Expr::Var(Var::Day);

        for d in 1..=31 {
            expr(&e, NaiveDate::from_ymd(2020, 1, d), d.into());
            expr(&e, NaiveDate::from_ymd(2020, 3, d), d.into());
            expr(&e, NaiveDate::from_ymd(2020, 5, d), d.into());
            expr(&e, NaiveDate::from_ymd(2020, 7, d), d.into());
            expr(&e, NaiveDate::from_ymd(2020, 8, d), d.into());
            expr(&e, NaiveDate::from_ymd(2020, 10, d), d.into());
            expr(&e, NaiveDate::from_ymd(2020, 12, d), d.into());

            expr(&e, NaiveDate::from_ymd(2021, 1, d), d.into());
            expr(&e, NaiveDate::from_ymd(2021, 3, d), d.into());
            expr(&e, NaiveDate::from_ymd(2021, 5, d), d.into());
            expr(&e, NaiveDate::from_ymd(2021, 7, d), d.into());
            expr(&e, NaiveDate::from_ymd(2021, 8, d), d.into());
            expr(&e, NaiveDate::from_ymd(2021, 10, d), d.into());
            expr(&e, NaiveDate::from_ymd(2021, 12, d), d.into());
        }

        for d in 1..=30 {
            expr(&e, NaiveDate::from_ymd(2020, 4, d), d.into());
            expr(&e, NaiveDate::from_ymd(2020, 6, d), d.into());
            expr(&e, NaiveDate::from_ymd(2020, 9, d), d.into());
            expr(&e, NaiveDate::from_ymd(2020, 11, d), d.into());

            expr(&e, NaiveDate::from_ymd(2021, 4, d), d.into());
            expr(&e, NaiveDate::from_ymd(2021, 6, d), d.into());
            expr(&e, NaiveDate::from_ymd(2021, 9, d), d.into());
            expr(&e, NaiveDate::from_ymd(2021, 11, d), d.into());
        }

        for d in 1..=28 {
            expr(&e, NaiveDate::from_ymd(2020, 2, d), d.into());
            expr(&e, NaiveDate::from_ymd(2021, 2, d), d.into());
        }

        expr(&e, NaiveDate::from_ymd(2020, 2, 29), 29);
    }

    #[test]
    fn day_reverse() {
        let e = Expr::Var(Var::DayReverse);

        expr(&e, NaiveDate::from_ymd(2021, 12, 1), 31);
        expr(&e, NaiveDate::from_ymd(2021, 12, 2), 30);
        expr(&e, NaiveDate::from_ymd(2021, 12, 3), 29);
        expr(&e, NaiveDate::from_ymd(2021, 12, 4), 28);
        expr(&e, NaiveDate::from_ymd(2021, 12, 5), 27);
        expr(&e, NaiveDate::from_ymd(2021, 12, 6), 26);
        expr(&e, NaiveDate::from_ymd(2021, 12, 7), 25);
        expr(&e, NaiveDate::from_ymd(2021, 12, 8), 24);
        expr(&e, NaiveDate::from_ymd(2021, 12, 9), 23);
        expr(&e, NaiveDate::from_ymd(2021, 12, 10), 22);
        expr(&e, NaiveDate::from_ymd(2021, 12, 11), 21);
        expr(&e, NaiveDate::from_ymd(2021, 12, 12), 20);
        expr(&e, NaiveDate::from_ymd(2021, 12, 13), 19);
        expr(&e, NaiveDate::from_ymd(2021, 12, 14), 18);
        expr(&e, NaiveDate::from_ymd(2021, 12, 15), 17);
        expr(&e, NaiveDate::from_ymd(2021, 12, 16), 16);
        expr(&e, NaiveDate::from_ymd(2021, 12, 17), 15);
        expr(&e, NaiveDate::from_ymd(2021, 12, 18), 14);
        expr(&e, NaiveDate::from_ymd(2021, 12, 19), 13);
        expr(&e, NaiveDate::from_ymd(2021, 12, 20), 12);
        expr(&e, NaiveDate::from_ymd(2021, 12, 21), 11);
        expr(&e, NaiveDate::from_ymd(2021, 12, 22), 10);
        expr(&e, NaiveDate::from_ymd(2021, 12, 23), 9);
        expr(&e, NaiveDate::from_ymd(2021, 12, 24), 8);
        expr(&e, NaiveDate::from_ymd(2021, 12, 25), 7);
        expr(&e, NaiveDate::from_ymd(2021, 12, 26), 6);
        expr(&e, NaiveDate::from_ymd(2021, 12, 27), 5);
        expr(&e, NaiveDate::from_ymd(2021, 12, 28), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 29), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 30), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 31), 1);
    }

    #[test]
    fn iso_year() {
        let e = Expr::Var(Var::IsoYear);

        // From https://en.wikipedia.org/wiki/ISO_week_date

        expr(&e, NaiveDate::from_ymd(1977, 1, 1), 1976);
        expr(&e, NaiveDate::from_ymd(1977, 1, 2), 1976);
        expr(&e, NaiveDate::from_ymd(1977, 1, 3), 1977);

        expr(&e, NaiveDate::from_ymd(1977, 12, 31), 1977);
        expr(&e, NaiveDate::from_ymd(1978, 1, 1), 1977);
        expr(&e, NaiveDate::from_ymd(1978, 1, 2), 1978);

        expr(&e, NaiveDate::from_ymd(1978, 12, 31), 1978);
        expr(&e, NaiveDate::from_ymd(1979, 1, 1), 1979);

        expr(&e, NaiveDate::from_ymd(1979, 12, 30), 1979);
        expr(&e, NaiveDate::from_ymd(1979, 12, 31), 1980);
        expr(&e, NaiveDate::from_ymd(1980, 1, 1), 1980);

        expr(&e, NaiveDate::from_ymd(1980, 12, 28), 1980);
        expr(&e, NaiveDate::from_ymd(1980, 12, 29), 1981);
        expr(&e, NaiveDate::from_ymd(1980, 12, 30), 1981);
        expr(&e, NaiveDate::from_ymd(1980, 12, 31), 1981);
        expr(&e, NaiveDate::from_ymd(1981, 1, 1), 1981);

        expr(&e, NaiveDate::from_ymd(1981, 12, 31), 1981);
        expr(&e, NaiveDate::from_ymd(1982, 1, 1), 1981);
        expr(&e, NaiveDate::from_ymd(1982, 1, 2), 1981);
        expr(&e, NaiveDate::from_ymd(1982, 1, 3), 1981);
        expr(&e, NaiveDate::from_ymd(1982, 1, 4), 1982);
    }

    #[test]
    fn iso_year_length() {
        let e = Expr::Var(Var::IsoYearLength);

        // August 1st is definitely in the same year in both systems
        expr(&e, NaiveDate::from_ymd(2000, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2001, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2002, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2003, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2004, 8, 1), 52 * 7 + 7);
        expr(&e, NaiveDate::from_ymd(2005, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2006, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2007, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2008, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2009, 8, 1), 52 * 7 + 7);
        expr(&e, NaiveDate::from_ymd(2010, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2011, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2012, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2013, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2014, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2015, 8, 1), 52 * 7 + 7);
        expr(&e, NaiveDate::from_ymd(2016, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2017, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2018, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2019, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2020, 8, 1), 52 * 7 + 7);
        expr(&e, NaiveDate::from_ymd(2021, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2022, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2023, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2024, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2025, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2026, 8, 1), 52 * 7 + 7);
        expr(&e, NaiveDate::from_ymd(2027, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2028, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2029, 8, 1), 52 * 7);
        expr(&e, NaiveDate::from_ymd(2030, 8, 1), 52 * 7);
    }

    #[test]
    fn iso_week() {
        let e = Expr::Var(Var::IsoWeek);

        // From https://en.wikipedia.org/wiki/ISO_week_date

        expr(&e, NaiveDate::from_ymd(1977, 1, 1), 53);
        expr(&e, NaiveDate::from_ymd(1977, 1, 2), 53);
        expr(&e, NaiveDate::from_ymd(1977, 1, 3), 1);

        expr(&e, NaiveDate::from_ymd(1977, 12, 31), 52);
        expr(&e, NaiveDate::from_ymd(1978, 1, 1), 52);
        expr(&e, NaiveDate::from_ymd(1978, 1, 2), 1);

        expr(&e, NaiveDate::from_ymd(1978, 12, 31), 52);
        expr(&e, NaiveDate::from_ymd(1979, 1, 1), 1);

        expr(&e, NaiveDate::from_ymd(1979, 12, 30), 52);
        expr(&e, NaiveDate::from_ymd(1979, 12, 31), 1);
        expr(&e, NaiveDate::from_ymd(1980, 1, 1), 1);

        expr(&e, NaiveDate::from_ymd(1980, 12, 28), 52);
        expr(&e, NaiveDate::from_ymd(1980, 12, 29), 1);
        expr(&e, NaiveDate::from_ymd(1980, 12, 30), 1);
        expr(&e, NaiveDate::from_ymd(1980, 12, 31), 1);
        expr(&e, NaiveDate::from_ymd(1981, 1, 1), 1);

        expr(&e, NaiveDate::from_ymd(1981, 12, 31), 53);
        expr(&e, NaiveDate::from_ymd(1982, 1, 1), 53);
        expr(&e, NaiveDate::from_ymd(1982, 1, 2), 53);
        expr(&e, NaiveDate::from_ymd(1982, 1, 3), 53);
        expr(&e, NaiveDate::from_ymd(1982, 1, 4), 1);
    }

    #[test]
    fn weekday() {
        let e = Expr::Var(Var::Weekday);

        expr(&e, NaiveDate::from_ymd(2021, 12, 18), 6);
        expr(&e, NaiveDate::from_ymd(2021, 12, 19), 7);
        expr(&e, NaiveDate::from_ymd(2021, 12, 20), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 21), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 22), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 23), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 24), 5);
        expr(&e, NaiveDate::from_ymd(2021, 12, 25), 6);
        expr(&e, NaiveDate::from_ymd(2021, 12, 26), 7);
        expr(&e, NaiveDate::from_ymd(2021, 12, 27), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 28), 2);
        expr(&e, NaiveDate::from_ymd(2021, 12, 29), 3);
        expr(&e, NaiveDate::from_ymd(2021, 12, 30), 4);
        expr(&e, NaiveDate::from_ymd(2021, 12, 31), 5);
    }

    #[test]
    fn easter() {
        let e = Expr::Var(Var::Easter(Span { start: 0, end: 0 }));

        // From https://en.wikipedia.org/wiki/List_of_dates_for_Easter
        #[rustfmt::skip]
        let dates = [
            (2041,4,21), (2040,4, 1),
            (2039,4,10), (2038,4,25), (2037,4, 5), (2036,4,13), (2035,3,25), (2034,4, 9), (2033,4,17), (2032,3,28), (2031,4,13), (2030,4,21),
            (2029,4, 1), (2028,4,16), (2027,3,28), (2026,4, 5), (2025,4,20), (2024,3,31), (2023,4, 9), (2022,4,17), (2021,4, 4), (2020,4,12),
            (2019,4,21), (2018,4, 1), (2017,4,16), (2016,3,27), (2015,4, 5), (2014,4,20), (2013,3,31), (2012,4, 8), (2011,4,24), (2010,4, 4),
            (2009,4,12), (2008,3,23), (2007,4, 8), (2006,4,16), (2005,3,27), (2004,4,11), (2003,4,20), (2002,3,31), (2001,4,15), (2000,4,23),
            (1999,4, 4), (1998,4,12), (1997,3,30), (1996,4, 7), (1995,4,16), (1994,4, 3), (1993,4,11), (1992,4,19), (1991,3,31), (1990,4,15),
            (1989,3,26), (1988,4, 3), (1987,4,19), (1986,3,30), (1985,4, 7), (1984,4,22), (1983,4, 3), (1982,4,11), (1981,4,19), (1980,4, 6),
            (1979,4,15), (1978,3,26), (1977,4,10), (1976,4,18), (1975,3,30), (1974,4,14), (1973,4,22), (1972,4, 2), (1971,4,11), (1970,3,29),
            (1969,4, 6), (1968,4,14), (1967,3,26), (1966,4,10), (1965,4,18), (1964,3,29), (1963,4,14), (1962,4,22), (1961,4, 2), (1960,4,17),
            (1959,3,29), (1958,4, 6), (1957,4,21), (1956,4, 1), (1955,4,10), (1954,4,18), (1953,4, 5), (1952,4,13), (1951,3,25), (1950,4, 9),
            (1949,4,17), (1948,3,28), (1947,4, 6), (1946,4,21), (1945,4, 1), (1944,4, 9), (1943,4,25), (1942,4, 5), (1941,4,13), (1940,3,24),
            (1939,4, 9), (1938,4,17), (1937,3,28), (1936,4,12), (1935,4,21), (1934,4, 1), (1933,4,16), (1932,3,27), (1931,4, 5), (1930,4,20),
            (1929,3,31), (1928,4, 8), (1927,4,17), (1926,4, 4), (1925,4,12), (1924,4,20), (1923,4, 1), (1922,4,16), (1921,3,27), (1920,4, 4),
            (1919,4,20), (1918,3,31), (1917,4, 8), (1916,4,23), (1915,4, 4), (1914,4,12), (1913,3,23), (1912,4, 7), (1911,4,16), (1910,3,27),
            (1909,4,11), (1908,4,19), (1907,3,31), (1906,4,15), (1905,4,23), (1904,4, 3), (1903,4,12), (1902,3,30), (1901,4, 7), (1900,4,15),
            (1899,4, 2), (1898,4,10), (1897,4,18), (1896,4, 5), (1895,4,14), (1894,3,25), (1893,4, 2), (1892,4,17), (1891,3,29), (1890,4, 6),
            (1889,4,21), (1888,4, 1), (1887,4,10), (1886,4,25), (1885,4, 5), (1884,4,13), (1883,3,25), (1882,4, 9), (1881,4,17), (1880,3,28),
            (1879,4,13), (1878,4,21), (1877,4, 1), (1876,4,16), (1875,3,28), (1874,4, 5), (1873,4,13), (1872,3,31), (1871,4, 9), (1870,4,17),
            (1869,3,28), (1868,4,12), (1867,4,21), (1866,4, 1), (1865,4,16), (1864,3,27), (1863,4, 5), (1862,4,20), (1861,3,31), (1860,4, 8),
            (1859,4,24), (1858,4, 4), (1857,4,12), (1856,3,23), (1855,4, 8), (1854,4,16), (1853,3,27), (1852,4,11), (1851,4,20), (1850,3,31),
            (1849,4, 8), (1848,4,23), (1847,4, 4), (1846,4,12), (1845,3,23), (1844,4, 7), (1843,4,16), (1842,3,27), (1841,4,11), (1840,4,19),
            (1839,3,31), (1838,4,15), (1837,3,26), (1836,4, 3), (1835,4,19), (1834,3,30), (1833,4, 7), (1832,4,22), (1831,4, 3), (1830,4,11),
            (1829,4,19), (1828,4, 6), (1827,4,15), (1826,3,26), (1825,4, 3), (1824,4,18), (1823,3,30), (1822,4, 7), (1821,4,22), (1820,4, 2),
            (1819,4,11), (1818,3,22), (1817,4, 6), (1816,4,14), (1815,3,26), (1814,4,10), (1813,4,18), (1812,3,29), (1811,4,14), (1810,4,22),
            (1809,4, 2), (1808,4,17), (1807,3,29), (1806,4, 6), (1805,4,14), (1804,4, 1), (1803,4,10), (1802,4,18), (1801,4, 5), (1800,4,13),
            (1799,3,24), (1798,4, 8), (1797,4,16), (1796,3,27), (1795,4, 5), (1794,4,20), (1793,3,31), (1792,4, 8), (1791,4,24), (1790,4, 4),
            (1789,4,12), (1788,3,23), (1787,4, 8), (1786,4,16), (1785,3,27), (1784,4,11), (1783,4,20), (1782,3,31), (1781,4,15), (1780,3,26),
            (1779,4, 4), (1778,4,19), (1777,3,30), (1776,4, 7), (1775,4,16), (1774,4, 3), (1773,4,11), (1772,4,19), (1771,3,31), (1770,4,15),
            (1769,3,26), (1768,4, 3), (1767,4,19), (1766,3,30), (1765,4, 7), (1764,4,22), (1763,4, 3), (1762,4,11), (1761,3,22), (1760,4, 6),
            (1759,4,15), (1758,3,26), (1757,4,10), (1756,4,18), (1755,3,30), (1754,4,14), (1753,4,22), (1752,4, 2), (1751,4,11), (1750,3,29),
            (1749,4, 6), (1748,4,14), (1747,4, 2), (1746,4,10), (1745,4,18), (1744,4, 5), (1743,4,14), (1742,3,25), (1741,4, 2), (1740,4,17),
            (1739,3,29), (1738,4, 6), (1737,4,21), (1736,4, 1), (1735,4,10), (1734,4,25), (1733,4, 5), (1732,4,13), (1731,3,25), (1730,4, 9),
            (1729,4,17), (1728,3,28), (1727,4,13), (1726,4,21), (1725,4, 1), (1724,4,16), (1723,3,28), (1722,4, 5), (1721,4,13), (1720,3,31),
            (1719,4, 9), (1718,4,17), (1717,3,28), (1716,4,12), (1715,4,21), (1714,4, 1), (1713,4,16), (1712,3,27), (1711,4, 5), (1710,4,20),
            (1709,3,31), (1708,4, 8), (1707,4,24), (1706,4, 4), (1705,4,12), (1704,3,23), (1703,4, 8), (1702,4,16), (1701,3,27), (1700,4,11),
            (1699,4,19), (1698,3,30), (1697,4, 7), (1696,4,22), (1695,4, 3), (1694,4,11), (1693,3,22), (1692,4, 6), (1691,4,15), (1690,3,26),
            (1689,4,10), (1688,4,18), (1687,3,30), (1686,4,14), (1685,4,22), (1684,4, 2), (1683,4,18), (1682,3,29), (1681,4, 6), (1680,4,21),
            (1679,4, 2), (1678,4,10), (1677,4,18), (1676,4, 5), (1675,4,14), (1674,3,25), (1673,4, 2), (1672,4,17), (1671,3,29), (1670,4 ,6),
            (1669,4,21), (1668,4, 1), (1667,4,10), (1666,4,25), (1665,4, 5), (1664,4,13), (1663,3,25), (1662,4, 9), (1661,4,17), (1660,3,28),
            (1659,4,13), (1658,4,21), (1657,4, 1), (1656,4,16), (1655,3,28), (1654,4, 5), (1653,4,13), (1652,3,31), (1651,4, 9), (1650,4,17),
            (1649,4, 4), (1648,4,12), (1647,4,21), (1646,4, 1), (1645,4,16), (1644,3,27), (1643,4, 5), (1642,4,20), (1641,3,31), (1640,4, 8),
            (1639,4,24), (1638,4, 4), (1637,4,12), (1636,3,23), (1635,4, 8), (1634,4,16), (1633,3,27), (1632,4,11), (1631,4,20), (1630,3,31),
            (1629,4,15), (1628,4,23), (1627,4, 4), (1626,4,12), (1625,3,30), (1624,4, 7), (1623,4,16), (1622,3,27), (1621,4,11), (1620,4,19),
            (1619,3,31), (1618,4,15), (1617,3,26), (1616,4, 3), (1615,4,19), (1614,3,30), (1613,4, 7), (1612,4,22), (1611,4, 3), (1610,4,11),
            (1609,4,19), (1608,4, 6), (1607,4,15), (1606,3,26), (1605,4,10), (1604,4,18), (1603,3,30), (1602,4, 7), (1601,4,22), (1600,4, 2),
            (1599,4,11), (1598,3,22), (1597,4, 6), (1596,4,14), (1595,3,26), (1594,4,10), (1593,4,18), (1592,3,29), (1591,4,14), (1590,4,22),
            (1589,4, 2), (1588,4,17), (1587,3,29), (1586,4, 6), (1585,4,21), (1584,4, 1), (1583,4,10),
        ];

        for (y, m, d) in dates {
            expr(
                &e,
                NaiveDate::from_ymd(y, 1, 1),
                NaiveDate::from_ymd(y, m, d).ordinal().into(),
            );
        }
    }

    #[test]
    fn is_weekday() {
        let e = Expr::Var(Var::IsWeekday);

        expr(&e, NaiveDate::from_ymd(2021, 12, 18), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 19), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 20), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 21), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 22), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 23), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 24), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 25), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 26), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 27), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 28), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 29), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 30), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 31), 1);
    }

    #[test]
    fn is_weekend() {
        let e = Expr::Var(Var::IsWeekend);

        expr(&e, NaiveDate::from_ymd(2021, 12, 18), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 19), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 20), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 21), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 22), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 23), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 24), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 25), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 26), 1);
        expr(&e, NaiveDate::from_ymd(2021, 12, 27), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 28), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 29), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 30), 0);
        expr(&e, NaiveDate::from_ymd(2021, 12, 31), 0);
    }

    #[test]
    fn is_leap_year() {
        let e = Expr::Var(Var::IsLeapYear);

        expr(&e, NaiveDate::from_ymd(2000, 1, 1), 1);
        expr(&e, NaiveDate::from_ymd(2001, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2002, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2003, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2004, 1, 1), 1);
        expr(&e, NaiveDate::from_ymd(2005, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2006, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2007, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2008, 1, 1), 1);
        expr(&e, NaiveDate::from_ymd(2009, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2010, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2011, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2012, 1, 1), 1);
        expr(&e, NaiveDate::from_ymd(2013, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2014, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2015, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2016, 1, 1), 1);
        expr(&e, NaiveDate::from_ymd(2017, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2018, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2019, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2020, 1, 1), 1);
        expr(&e, NaiveDate::from_ymd(2021, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2022, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2023, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2024, 1, 1), 1);
        expr(&e, NaiveDate::from_ymd(2025, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2026, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2027, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2028, 1, 1), 1);
        expr(&e, NaiveDate::from_ymd(2029, 1, 1), 0);
        expr(&e, NaiveDate::from_ymd(2030, 1, 1), 0);
    }

    #[test]
    fn is_iso_leap_year() {
        let e = Expr::Var(Var::IsIsoLeapYear);

        // August 1st is definitely in the same year in both systems
        expr(&e, NaiveDate::from_ymd(2000, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2001, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2002, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2003, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2004, 8, 1), 1);
        expr(&e, NaiveDate::from_ymd(2005, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2006, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2007, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2008, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2009, 8, 1), 1);
        expr(&e, NaiveDate::from_ymd(2010, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2011, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2012, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2013, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2014, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2015, 8, 1), 1);
        expr(&e, NaiveDate::from_ymd(2016, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2017, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2018, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2019, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2020, 8, 1), 1);
        expr(&e, NaiveDate::from_ymd(2021, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2022, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2023, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2024, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2025, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2026, 8, 1), 1);
        expr(&e, NaiveDate::from_ymd(2027, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2028, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2029, 8, 1), 0);
        expr(&e, NaiveDate::from_ymd(2030, 8, 1), 0);
    }
}
