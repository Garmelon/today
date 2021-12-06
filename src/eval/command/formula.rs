use chrono::{Datelike, NaiveDate};

use crate::files::commands::{self, Command, Expr, Var};
use crate::files::primitives::{Spanned, Time, Weekday};

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

impl Var {
    fn eval(self, date: NaiveDate) -> Result<i64> {
        Ok(match self {
            Var::True => 1,
            Var::False => 0,
            Var::Monday => 1,
            Var::Tuesday => 2,
            Var::Wednesday => 3,
            Var::Thursday => 4,
            Var::Friday => 5,
            Var::Saturday => 6,
            Var::Sunday => 7,
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
            Var::Easter => {
                let e = computus::gregorian(date.year()).map_err(|e| Error::Easter {
                    span: todo!(),
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

impl Expr {
    fn eval(&self, date: NaiveDate) -> Result<i64> {
        Ok(match self {
            Expr::Lit(l) => *l,
            Expr::Var(v) => v.eval(date)?,
            Expr::Paren(e) => e.eval(date)?,
            Expr::Neg(e) => -e.eval(date)?,
            Expr::Add(a, b) => a.eval(date)? + b.eval(date)?,
            Expr::Sub(a, b) => a.eval(date)? - b.eval(date)?,
            Expr::Mul(a, b) => a.eval(date)? * b.eval(date)?,
            Expr::Div(a, b) => {
                let b = b.eval(date)?;
                if b == 0 {
                    return Err(Error::DivByZero {
                        span: todo!(),
                        date,
                    });
                }
                a.eval(date)?.div_euclid(b)
            }
            Expr::Mod(a, b) => {
                let b = b.eval(date)?;
                if b == 0 {
                    return Err(Error::ModByZero {
                        span: todo!(),
                        date,
                    });
                }
                a.eval(date)?.rem_euclid(b)
            }
            Expr::Eq(a, b) => b2i(a.eval(date)? == b.eval(date)?),
            Expr::Neq(a, b) => b2i(a.eval(date)? != b.eval(date)?),
            Expr::Lt(a, b) => b2i(a.eval(date)? < b.eval(date)?),
            Expr::Lte(a, b) => b2i(a.eval(date)? <= b.eval(date)?),
            Expr::Gt(a, b) => b2i(a.eval(date)? > b.eval(date)?),
            Expr::Gte(a, b) => b2i(a.eval(date)? >= b.eval(date)?),
            Expr::Not(e) => b2i(!i2b(e.eval(date)?)),
            Expr::And(a, b) => b2i(i2b(a.eval(date)?) && i2b(b.eval(date)?)),
            Expr::Or(a, b) => b2i(i2b(a.eval(date)?) || i2b(b.eval(date)?)),
            Expr::Xor(a, b) => b2i(i2b(a.eval(date)?) ^ i2b(b.eval(date)?)),
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
        let start: Expr = spec.start.as_ref().cloned().unwrap_or(Expr::Lit(1));

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
            Box::new(Expr::Var(spec.start.into())),
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
            if let Some(last_done) = s.last_done() {
                range = range.with_from(last_done.succ())?;
            }
        }

        s.limit_from_until(range)
    }

    fn dates(&self, start: NaiveDate) -> Result<Dates> {
        let root = self.start_delta.apply_date(start)?;
        Ok(if let Some(root_time) = self.start_time {
            let (other, other_time) = self.end_delta.apply_date_time(root, root_time)?;
            Dates::new_with_time(root, root_time, other, other_time)
        } else {
            let other = self.end_delta.apply_date(root)?;
            Dates::new(root, other)
        })
    }

    fn eval(&self, date: NaiveDate) -> Result<bool> {
        Ok(i2b(self.start.eval(date)?))
    }
}

impl<'a> CommandState<'a> {
    pub fn eval_formula_spec(&mut self, spec: FormulaSpec) -> Result<()> {
        if let Some(range) = spec.range(self) {
            for day in range.days() {
                if spec.eval(day)? {
                    let dates = spec.dates(day)?;
                    self.add(self.kind(), Some(dates));
                }
            }
        }
        Ok(())
    }
}
