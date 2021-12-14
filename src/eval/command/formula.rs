use chrono::{Datelike, NaiveDate};

use crate::files::commands::{self, Command};
use crate::files::primitives::{Span, Spanned, Time, Weekday};

use super::super::command::CommandState;
use super::super::date::Dates;
use super::super::delta::{Delta, DeltaStep};
use super::super::{util, DateRange, Error, Result};

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
}

impl Var {
    fn eval(self, file: usize, date: NaiveDate) -> Result<i64> {
        Ok(match self {
            Var::JulianDay => date.num_days_from_ce().into(),
            Var::Year => date.year().into(),
            Var::YearLength => util::year_length(date.year()).into(),
            Var::YearDay => date.ordinal().into(),
            Var::YearDayReverse => (util::year_length(date.year()) - date.ordinal0()).into(),
            Var::YearWeek => (date.ordinal0().div_euclid(7) + 1).into(),
            Var::YearWeekReverse => {
                #[allow(non_snake_case)]
                let yD = util::year_length(date.year()) - date.ordinal();
                (yD.div_euclid(7) + 1).into()
            }
            Var::Month => date.month().into(),
            Var::MonthLength => util::month_length(date.year(), date.month()).into(),
            Var::MonthWeek => (date.month0().div_euclid(7) + 1).into(),
            Var::MonthWeekReverse => {
                #[allow(non_snake_case)]
                let mD = util::month_length(date.year(), date.month()) - date.month0();
                (mD.div_euclid(7) + 1).into()
            }
            Var::Day => date.day().into(),
            Var::DayReverse => {
                let ml = util::month_length(date.year(), date.month());
                (ml - date.month0()).into()
            }
            Var::IsoYear => date.iso_week().year().into(),
            Var::IsoYearLength => util::iso_year_length(date.iso_week().year()).into(),
            Var::IsoWeek => date.iso_week().week().into(),
            Var::Weekday => {
                let wd: Weekday = date.weekday().into();
                wd.num().into()
            }
            Var::Easter(span) => {
                let e = computus::gregorian(date.year()).map_err(|e| Error::Easter {
                    file,
                    span,
                    date,
                    msg: e,
                })?;
                NaiveDate::from_ymd(e.year, e.month, e.day).ordinal().into()
            }
            Var::IsWeekday => {
                let wd: Weekday = date.weekday().into();
                b2i(!wd.is_weekend())
            }
            Var::IsWeekend => {
                let wd: Weekday = date.weekday().into();
                b2i(wd.is_weekend())
            }
            Var::IsLeapYear => b2i(util::is_leap_year(date.year())),
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
    fn eval(&self, file: usize, date: NaiveDate) -> Result<i64> {
        Ok(match self {
            Expr::Lit(l) => *l,
            Expr::Var(v) => v.eval(file, date)?,
            Expr::Neg(e) => -e.eval(file, date)?,
            Expr::Add(a, b) => a.eval(file, date)? + b.eval(file, date)?,
            Expr::Sub(a, b) => a.eval(file, date)? - b.eval(file, date)?,
            Expr::Mul(a, b) => a.eval(file, date)? * b.eval(file, date)?,
            Expr::Div(a, b, span) => {
                let b = b.eval(file, date)?;
                if b == 0 {
                    return Err(Error::DivByZero {
                        file,
                        span: *span,
                        date,
                    });
                }
                a.eval(file, date)?.div_euclid(b)
            }
            Expr::Mod(a, b, span) => {
                let b = b.eval(file, date)?;
                if b == 0 {
                    return Err(Error::ModByZero {
                        file,
                        span: *span,
                        date,
                    });
                }
                a.eval(file, date)?.rem_euclid(b)
            }
            Expr::Eq(a, b) => b2i(a.eval(file, date)? == b.eval(file, date)?),
            Expr::Neq(a, b) => b2i(a.eval(file, date)? != b.eval(file, date)?),
            Expr::Lt(a, b) => b2i(a.eval(file, date)? < b.eval(file, date)?),
            Expr::Lte(a, b) => b2i(a.eval(file, date)? <= b.eval(file, date)?),
            Expr::Gt(a, b) => b2i(a.eval(file, date)? > b.eval(file, date)?),
            Expr::Gte(a, b) => b2i(a.eval(file, date)? >= b.eval(file, date)?),
            Expr::Not(e) => b2i(!i2b(e.eval(file, date)?)),
            Expr::And(a, b) => b2i(i2b(a.eval(file, date)?) && i2b(b.eval(file, date)?)),
            Expr::Or(a, b) => b2i(i2b(a.eval(file, date)?) || i2b(b.eval(file, date)?)),
            Expr::Xor(a, b) => b2i(i2b(a.eval(file, date)?) ^ i2b(b.eval(file, date)?)),
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
            .range
            .expand_by(&self.end_delta)
            .move_by(&self.start_delta);

        if let Command::Task(_) = s.command.command {
            if let Some(last_done_root) = s.last_done_root() {
                range = range.with_from(last_done_root.succ())?;
            }
            // TODO Otherwise, go back one year or so if no FROM is specified
        }

        s.limit_from_until(range)
    }

    fn dates(&self, file: usize, start: NaiveDate) -> Result<Dates> {
        let root = self.start_delta.apply_date(file, start)?;
        Ok(if let Some(root_time) = self.start_time {
            let (other, other_time) = self.end_delta.apply_date_time(file, root, root_time)?;
            Dates::new_with_time(root, root_time, other, other_time)
        } else {
            let other = self.end_delta.apply_date(file, root)?;
            Dates::new(root, other)
        })
    }

    fn eval(&self, file: usize, date: NaiveDate) -> Result<bool> {
        Ok(i2b(self.start.eval(file, date)?))
    }
}

impl<'a> CommandState<'a> {
    pub fn eval_formula_spec(&mut self, spec: FormulaSpec) -> Result<()> {
        if let Some(range) = spec.range(self) {
            let file = self.command.source.file();
            for day in range.days() {
                if spec.eval(file, day)? {
                    let dates = spec.dates(file, day)?;
                    self.add(self.kind(), Some(dates));
                }
            }
        }
        Ok(())
    }
}
