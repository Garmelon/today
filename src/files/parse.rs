use std::path::Path;
use std::result;

use chrono::NaiveDate;
use pest::error::ErrorVariant;
use pest::iterators::Pair;
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::{Parser, Span};

use super::commands::{
    BirthdaySpec, Command, DateSpec, Delta, DeltaStep, Done, DoneDate, DoneKind, Expr, File,
    FormulaSpec, Log, Note, Repeat, Spec, Statement, Task, Var, WeekdaySpec,
};
use super::primitives::{Spanned, Time, Weekday};

#[derive(pest_derive::Parser)]
#[grammar = "files/grammar.pest"]
pub struct TodayfileParser;

pub type Error = pest::error::Error<Rule>;
pub type Result<T> = result::Result<T, Box<Error>>;

fn error<S: Into<String>>(span: Span<'_>, message: S) -> Error {
    Error::new_from_span(
        ErrorVariant::CustomError {
            message: message.into(),
        },
        span,
    )
}

fn fail<S: Into<String>, T>(span: Span<'_>, message: S) -> Result<T> {
    Err(Box::new(error(span, message)))
}

fn parse_include(p: Pair<'_, Rule>) -> Spanned<String> {
    assert_eq!(p.as_rule(), Rule::include);
    let p = p.into_inner().next().unwrap();
    let span = (&p.as_span()).into();
    let name = p.as_str().to_string();
    Spanned::new(span, name)
}

fn parse_timezone(p: Pair<'_, Rule>) -> Spanned<String> {
    assert_eq!(p.as_rule(), Rule::timezone);
    let p = p.into_inner().next().unwrap();
    let span = (&p.as_span()).into();
    let name = p.as_str().to_string();
    Spanned::new(span, name)
}

pub fn parse_number(p: Pair<'_, Rule>) -> i32 {
    assert_eq!(p.as_rule(), Rule::number);
    p.as_str().parse().unwrap()
}

fn parse_title(p: Pair<'_, Rule>) -> String {
    assert_eq!(p.as_rule(), Rule::title);
    let p = p.into_inner().next().unwrap();
    assert_eq!(p.as_rule(), Rule::rest_some);
    p.as_str().trim().to_string()
}

pub fn parse_datum(p: Pair<'_, Rule>) -> Result<Spanned<NaiveDate>> {
    assert_eq!(p.as_rule(), Rule::datum);
    let pspan = p.as_span();
    let span = (&pspan).into();
    let mut p = p.into_inner();

    let year = p.next().unwrap().as_str().parse().unwrap();
    let month = p.next().unwrap().as_str().parse().unwrap();
    let day = p.next().unwrap().as_str().parse().unwrap();

    assert_eq!(p.next(), None);

    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) => Ok(Spanned::new(span, date)),
        None => fail(pspan, "invalid date"),
    }
}

fn parse_time(p: Pair<'_, Rule>) -> Result<Spanned<Time>> {
    assert_eq!(p.as_rule(), Rule::time);
    let pspan = p.as_span();
    let span = (&pspan).into();
    let mut p = p.into_inner();

    let hour = p.next().unwrap().as_str().parse().unwrap();
    let min = p.next().unwrap().as_str().parse().unwrap();

    assert_eq!(p.next(), None);

    let time = Time::new(hour, min);
    if time.in_normal_range() {
        Ok(Spanned::new(span, time))
    } else {
        fail(pspan, "invalid time")
    }
}

#[derive(Clone, Copy)]
pub enum Sign {
    Positive,
    Negative,
}
pub struct Amount {
    sign: Option<Sign>,
    value: i32,
}

impl Amount {
    pub fn with_prev_sign(mut self, prev: Option<Sign>) -> Self {
        if self.sign.is_none() {
            self.sign = prev;
        }
        self
    }

    pub fn value(&self) -> Option<i32> {
        match self.sign {
            None => None,
            Some(Sign::Positive) => Some(self.value),
            Some(Sign::Negative) => Some(-self.value),
        }
    }
}

fn parse_amount(p: Pair<'_, Rule>) -> Amount {
    assert_eq!(p.as_rule(), Rule::amount);

    let mut sign = None;
    let mut value = 1;
    for p in p.into_inner() {
        match p.as_rule() {
            Rule::amount_sign => {
                sign = Some(match p.as_str() {
                    "+" => Sign::Positive,
                    "-" => Sign::Negative,
                    _ => unreachable!(),
                })
            }
            Rule::number => value = parse_number(p),
            _ => unreachable!(),
        }
    }

    Amount { sign, value }
}

fn parse_weekday(p: Pair<'_, Rule>) -> Spanned<Weekday> {
    assert_eq!(p.as_rule(), Rule::weekday);
    let span = (&p.as_span()).into();
    let wd = match p.as_str() {
        "mon" => Weekday::Monday,
        "tue" => Weekday::Tuesday,
        "wed" => Weekday::Wednesday,
        "thu" => Weekday::Thursday,
        "fri" => Weekday::Friday,
        "sat" => Weekday::Saturday,
        "sun" => Weekday::Sunday,
        _ => unreachable!(),
    };
    Spanned::new(span, wd)
}

fn parse_delta_weekdays(p: Pair<'_, Rule>, sign: &mut Option<Sign>) -> Result<Spanned<DeltaStep>> {
    assert_eq!(p.as_rule(), Rule::delta_weekdays);
    let pspan = p.as_span();
    let span = (&pspan).into();
    let mut p = p.into_inner();

    let amount = parse_amount(p.next().unwrap()).with_prev_sign(*sign);
    let weekday = parse_weekday(p.next().unwrap()).value;

    assert_eq!(p.next(), None);

    let value = amount
        .value()
        .ok_or_else(|| error(pspan, "ambiguous sign"))?;
    *sign = amount.sign;

    Ok(Spanned::new(span, DeltaStep::Weekday(value, weekday)))
}

fn parse_delta_step(
    p: Pair<'_, Rule>,
    sign: &mut Option<Sign>,
    f: impl FnOnce(i32) -> DeltaStep,
) -> Result<Spanned<DeltaStep>> {
    assert!(matches!(
        p.as_rule(),
        Rule::delta_years
            | Rule::delta_months
            | Rule::delta_months_reverse
            | Rule::delta_days
            | Rule::delta_weeks
            | Rule::delta_hours
            | Rule::delta_minutes
    ));

    let pspan = p.as_span();
    let span = (&pspan).into();
    let amount = parse_amount(p.into_inner().next().unwrap()).with_prev_sign(*sign);
    let value = amount
        .value()
        .ok_or_else(|| error(pspan, "ambiguous sign"))?;

    *sign = amount.sign;
    Ok(Spanned::new(span, f(value)))
}

pub fn parse_delta(p: Pair<'_, Rule>) -> Result<Spanned<Delta>> {
    assert_eq!(p.as_rule(), Rule::delta);
    let span = (&p.as_span()).into();

    let mut sign = None;
    let mut steps = vec![];

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::delta_weekdays => steps.push(parse_delta_weekdays(p, &mut sign)?),
            Rule::delta_minutes => steps.push(parse_delta_step(p, &mut sign, DeltaStep::Minute)?),
            Rule::delta_years => steps.push(parse_delta_step(p, &mut sign, DeltaStep::Year)?),
            Rule::delta_months => steps.push(parse_delta_step(p, &mut sign, DeltaStep::Month)?),
            Rule::delta_months_reverse => {
                steps.push(parse_delta_step(p, &mut sign, DeltaStep::MonthReverse)?)
            }
            Rule::delta_days => steps.push(parse_delta_step(p, &mut sign, DeltaStep::Day)?),
            Rule::delta_weeks => steps.push(parse_delta_step(p, &mut sign, DeltaStep::Week)?),
            Rule::delta_hours => steps.push(parse_delta_step(p, &mut sign, DeltaStep::Hour)?),
            _ => unreachable!(),
        }
    }

    Ok(Spanned::new(span, Delta(steps)))
}

fn parse_date_fixed_start(p: Pair<'_, Rule>, spec: &mut DateSpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_fixed_start);

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::datum => spec.start = parse_datum(p)?.value,
            Rule::delta => spec.start_delta = Some(parse_delta(p)?.value),
            Rule::time => spec.start_time = Some(parse_time(p)?.value),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn parse_date_fixed_end(p: Pair<'_, Rule>, spec: &mut DateSpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_fixed_end);

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::datum => spec.end = Some(parse_datum(p)?),
            Rule::delta => spec.end_delta = Some(parse_delta(p)?.value),
            Rule::time => spec.end_time = Some(parse_time(p)?),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn parse_date_fixed_repeat(p: Pair<'_, Rule>, spec: &mut DateSpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_fixed_repeat);
    let mut ps = p.into_inner().collect::<Vec<_>>();

    let repeat = match ps.len() {
        1 => Repeat {
            start_at_done: false,
            delta: parse_delta(ps.pop().unwrap())?,
        },
        2 => {
            assert_eq!(ps[0].as_rule(), Rule::repeat_done);
            Repeat {
                start_at_done: true,
                delta: parse_delta(ps.pop().unwrap())?,
            }
        }
        _ => unreachable!(),
    };

    spec.repeat = Some(repeat);

    Ok(())
}

fn parse_date_fixed(p: Pair<'_, Rule>) -> Result<DateSpec> {
    assert_eq!(p.as_rule(), Rule::date_fixed);

    let mut spec = DateSpec {
        start: NaiveDate::from_ymd_opt(0, 1, 1).unwrap(),
        start_delta: None,
        start_time: None,
        end: None,
        end_delta: None,
        end_time: None,
        repeat: None,
    };

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::date_fixed_start => parse_date_fixed_start(p, &mut spec)?,
            Rule::date_fixed_end => parse_date_fixed_end(p, &mut spec)?,
            Rule::date_fixed_repeat => parse_date_fixed_repeat(p, &mut spec)?,
            _ => unreachable!(),
        }
    }

    Ok(spec)
}

fn parse_boolean(p: Pair<'_, Rule>) -> Var {
    assert_eq!(p.as_rule(), Rule::boolean);
    match p.as_str() {
        "true" => Var::True,
        "false" => Var::False,
        _ => unreachable!(),
    }
}

fn parse_variable(p: Pair<'_, Rule>) -> Var {
    assert_eq!(p.as_rule(), Rule::variable);
    match p.as_str() {
        "j" => Var::JulianDay,
        "y" => Var::Year,
        "yl" => Var::YearLength,
        "yd" => Var::YearDay,
        "yD" => Var::YearDayReverse,
        "yw" => Var::YearWeek,
        "yW" => Var::YearWeekReverse,
        "m" => Var::Month,
        "ml" => Var::MonthLength,
        "mw" => Var::MonthWeek,
        "mW" => Var::MonthWeekReverse,
        "d" => Var::Day,
        "D" => Var::DayReverse,
        "iy" => Var::IsoYear,
        "iyl" => Var::IsoYearLength,
        "iw" => Var::IsoWeek,
        "wd" => Var::Weekday,
        "e" => Var::Easter,
        "mon" => Var::Monday,
        "tue" => Var::Tuesday,
        "wed" => Var::Wednesday,
        "thu" => Var::Thursday,
        "fri" => Var::Friday,
        "sat" => Var::Saturday,
        "sun" => Var::Sunday,
        "isWeekday" => Var::IsWeekday,
        "isWeekend" => Var::IsWeekend,
        "isLeapYear" => Var::IsLeapYear,
        "isIsoLeapYear" => Var::IsIsoLeapYear,
        _ => unreachable!(),
    }
}

fn parse_paren_expr(p: Pair<'_, Rule>) -> Spanned<Expr> {
    assert_eq!(p.as_rule(), Rule::paren_expr);
    let span = (&p.as_span()).into();
    let inner = parse_expr(p.into_inner().next().unwrap());
    Spanned::new(span, Expr::Paren(Box::new(inner)))
}

fn parse_term(p: Pair<'_, Rule>) -> Spanned<Expr> {
    assert_eq!(p.as_rule(), Rule::term);
    let span = (&p.as_span()).into();
    let p = p.into_inner().next().unwrap();
    match p.as_rule() {
        Rule::number => Spanned::new(span, Expr::Lit(parse_number(p).into())),
        Rule::boolean => Spanned::new(span, Expr::Var(parse_boolean(p))),
        Rule::variable => Spanned::new(span, Expr::Var(parse_variable(p))),
        Rule::paren_expr => parse_paren_expr(p),
        _ => unreachable!(),
    }
}

fn parse_prefix(p: Pair<'_, Rule>, s: Spanned<Expr>) -> Spanned<Expr> {
    let span = s.span.join((&p.as_span()).into());
    let expr = match p.as_rule() {
        Rule::prefix_neg => Expr::Neg(Box::new(s)),
        Rule::prefix_not => Expr::Not(Box::new(s)),
        _ => unreachable!(),
    };
    Spanned::new(span, expr)
}

fn parse_infix(l: Spanned<Expr>, p: Pair<'_, Rule>, r: Spanned<Expr>) -> Spanned<Expr> {
    let span = l.span.join(r.span);
    let expr = match p.as_rule() {
        // Integer-y operations
        Rule::infix_add => Expr::Add(Box::new(l), Box::new(r)),
        Rule::infix_sub => Expr::Sub(Box::new(l), Box::new(r)),
        Rule::infix_mul => Expr::Mul(Box::new(l), Box::new(r)),
        Rule::infix_div => Expr::Div(Box::new(l), Box::new(r)),
        Rule::infix_mod => Expr::Mod(Box::new(l), Box::new(r)),

        // Comparisons
        Rule::infix_eq => Expr::Eq(Box::new(l), Box::new(r)),
        Rule::infix_neq => Expr::Neq(Box::new(l), Box::new(r)),
        Rule::infix_lt => Expr::Lt(Box::new(l), Box::new(r)),
        Rule::infix_lte => Expr::Lte(Box::new(l), Box::new(r)),
        Rule::infix_gt => Expr::Gt(Box::new(l), Box::new(r)),
        Rule::infix_gte => Expr::Gte(Box::new(l), Box::new(r)),

        // Boolean-y operations
        Rule::infix_and => Expr::And(Box::new(l), Box::new(r)),
        Rule::infix_or => Expr::Or(Box::new(l), Box::new(r)),
        Rule::infix_xor => Expr::Xor(Box::new(l), Box::new(r)),

        _ => unreachable!(),
    };
    Spanned::new(span, expr)
}

fn parse_expr(p: Pair<'_, Rule>) -> Spanned<Expr> {
    assert_eq!(p.as_rule(), Rule::expr);

    PrattParser::new()
        .op(Op::infix(Rule::infix_or, Assoc::Left) | Op::infix(Rule::infix_xor, Assoc::Left))
        .op(Op::infix(Rule::infix_and, Assoc::Left))
        .op(Op::infix(Rule::infix_eq, Assoc::Left) | Op::infix(Rule::infix_neq, Assoc::Left))
        .op(Op::infix(Rule::infix_lt, Assoc::Left)
            | Op::infix(Rule::infix_lte, Assoc::Left)
            | Op::infix(Rule::infix_gt, Assoc::Left)
            | Op::infix(Rule::infix_gte, Assoc::Left))
        .op(Op::infix(Rule::infix_mul, Assoc::Left)
            | Op::infix(Rule::infix_div, Assoc::Left)
            | Op::infix(Rule::infix_mod, Assoc::Left))
        .op(Op::infix(Rule::infix_add, Assoc::Left) | Op::infix(Rule::infix_sub, Assoc::Left))
        .op(Op::prefix(Rule::prefix_neg) | Op::prefix(Rule::prefix_not))
        .map_primary(parse_term)
        .map_prefix(parse_prefix)
        .map_infix(parse_infix)
        .parse(p.into_inner())
}

fn parse_date_expr_start(p: Pair<'_, Rule>, spec: &mut FormulaSpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_expr_start);

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::paren_expr => spec.start = Some(parse_expr(p.into_inner().next().unwrap())),
            Rule::delta => spec.start_delta = Some(parse_delta(p)?.value),
            Rule::time => spec.start_time = Some(parse_time(p)?.value),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn parse_date_expr_end(p: Pair<'_, Rule>, spec: &mut FormulaSpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_expr_end);

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::delta => spec.end_delta = Some(parse_delta(p)?.value),
            Rule::time => spec.end_time = Some(parse_time(p)?),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn parse_date_expr(p: Pair<'_, Rule>) -> Result<FormulaSpec> {
    assert_eq!(p.as_rule(), Rule::date_expr);

    let mut spec = FormulaSpec {
        start: None,
        start_delta: None,
        start_time: None,
        end_delta: None,
        end_time: None,
    };

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::date_expr_start => parse_date_expr_start(p, &mut spec)?,
            Rule::date_expr_end => parse_date_expr_end(p, &mut spec)?,
            _ => unreachable!(),
        }
    }

    Ok(spec)
}

fn parse_date_weekday_start(p: Pair<'_, Rule>, spec: &mut WeekdaySpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_weekday_start);

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::weekday => spec.start = parse_weekday(p).value,
            Rule::time => spec.start_time = Some(parse_time(p)?.value),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn parse_date_weekday_end(p: Pair<'_, Rule>, spec: &mut WeekdaySpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_weekday_end);

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::weekday => spec.end = Some(parse_weekday(p)),
            Rule::delta => spec.end_delta = Some(parse_delta(p)?.value),
            Rule::time => spec.end_time = Some(parse_time(p)?),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn parse_date_weekday(p: Pair<'_, Rule>) -> Result<WeekdaySpec> {
    assert_eq!(p.as_rule(), Rule::date_weekday);

    let mut spec = WeekdaySpec {
        start: Weekday::Monday,
        start_time: None,
        end: None,
        end_delta: None,
        end_time: None,
    };

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::date_weekday_start => parse_date_weekday_start(p, &mut spec)?,
            Rule::date_weekday_end => parse_date_weekday_end(p, &mut spec)?,
            _ => unreachable!(),
        }
    }

    Ok(spec)
}

fn parse_stmt_date(p: Pair<'_, Rule>) -> Result<Statement> {
    assert_eq!(p.as_rule(), Rule::stmt_date);
    let p = p.into_inner().next().unwrap();
    let spec = match p.as_rule() {
        Rule::date_fixed => Spec::Date(parse_date_fixed(p)?),
        Rule::date_expr => Spec::Formula(parse_date_expr(p)?),
        Rule::date_weekday => Spec::Weekday(parse_date_weekday(p)?),
        _ => unreachable!(),
    };
    Ok(Statement::Date(spec))
}

fn parse_bdatum(p: Pair<'_, Rule>) -> Result<BirthdaySpec> {
    assert_eq!(p.as_rule(), Rule::bdatum);
    let span = p.as_span();
    let p = p.into_inner().collect::<Vec<_>>();
    assert!(p.len() == 2 || p.len() == 3);

    let (y, m, d, year_known) = if p.len() == 3 {
        let y = p[0].as_str().parse().unwrap();
        let m = p[1].as_str().parse().unwrap();
        let d = p[2].as_str().parse().unwrap();
        (y, m, d, true)
    } else {
        let m = p[0].as_str().parse().unwrap();
        let d = p[1].as_str().parse().unwrap();
        (0, m, d, false)
    };

    let date = match NaiveDate::from_ymd_opt(y, m, d) {
        Some(date) => Ok(date),
        None => fail(span, "invalid date"),
    }?;

    Ok(BirthdaySpec { date, year_known })
}

fn parse_stmt_bdate(p: Pair<'_, Rule>) -> Result<Statement> {
    assert_eq!(p.as_rule(), Rule::stmt_bdate);
    let spec = parse_bdatum(p.into_inner().next().unwrap())?;
    Ok(Statement::BDate(spec))
}

fn parse_stmt_from(p: Pair<'_, Rule>) -> Result<Statement> {
    assert_eq!(p.as_rule(), Rule::stmt_from);
    let mut p = p.into_inner();
    let datum = match p.next() {
        Some(p) => Some(parse_datum(p)?.value),
        None => None,
    };
    assert_eq!(p.next(), None);
    Ok(Statement::From(datum))
}

fn parse_stmt_until(p: Pair<'_, Rule>) -> Result<Statement> {
    assert_eq!(p.as_rule(), Rule::stmt_until);
    let mut p = p.into_inner();
    let datum = match p.next() {
        Some(p) => Some(parse_datum(p)?.value),
        None => None,
    };
    assert_eq!(p.next(), None);
    Ok(Statement::Until(datum))
}

fn parse_stmt_except(p: Pair<'_, Rule>) -> Result<Statement> {
    assert_eq!(p.as_rule(), Rule::stmt_except);
    let datum = parse_datum(p.into_inner().next().unwrap())?.value;
    Ok(Statement::Except(datum))
}

fn parse_stmt_move(p: Pair<'_, Rule>) -> Result<Statement> {
    assert_eq!(p.as_rule(), Rule::stmt_move);
    let span = (&p.as_span()).into();
    let mut p = p.into_inner();
    let from = parse_datum(p.next().unwrap())?.value;

    let mut to = None;
    let mut to_time = None;
    for p in p {
        match p.as_rule() {
            Rule::datum => to = Some(parse_datum(p)?.value),
            Rule::time => to_time = Some(parse_time(p)?),
            _ => unreachable!(),
        }
    }

    Ok(Statement::Move {
        span,
        from,
        to,
        to_time,
    })
}

fn parse_stmt_remind(p: Pair<'_, Rule>) -> Result<Statement> {
    assert_eq!(p.as_rule(), Rule::stmt_remind);
    let mut p = p.into_inner();
    let delta = match p.next() {
        Some(p) => Some(parse_delta(p)?),
        None => None,
    };
    assert_eq!(p.next(), None);
    Ok(Statement::Remind(delta))
}

fn parse_statements(p: Pair<'_, Rule>, task: bool) -> Result<Vec<Statement>> {
    assert_eq!(p.as_rule(), Rule::statements);
    let mut statements = vec![];
    for p in p.into_inner() {
        statements.push(match p.as_rule() {
            Rule::stmt_date => parse_stmt_date(p)?,
            Rule::stmt_bdate if task => fail(p.as_span(), "BDATE not allowed in TASKs")?,
            Rule::stmt_bdate => parse_stmt_bdate(p)?,
            Rule::stmt_from => parse_stmt_from(p)?,
            Rule::stmt_until => parse_stmt_until(p)?,
            Rule::stmt_except => parse_stmt_except(p)?,
            Rule::stmt_move => parse_stmt_move(p)?,
            Rule::stmt_remind => parse_stmt_remind(p)?,
            _ => unreachable!(),
        });
    }
    Ok(statements)
}

fn parse_donedate(p: Pair<'_, Rule>) -> Result<DoneDate> {
    assert_eq!(p.as_rule(), Rule::donedate);
    let mut ps = p.into_inner().collect::<Vec<_>>();

    // Popping the elements off of the vector in reverse so I don't have to
    // shuffle them around weirdly. In Haskell, I would've just pattern-matched
    // the list ;-;
    Ok(match ps.len() {
        1 => DoneDate::Date {
            root: parse_datum(ps.pop().unwrap())?.value,
        },
        2 => match ps[1].as_rule() {
            Rule::time => DoneDate::DateTime {
                root_time: parse_time(ps.pop().unwrap())?.value,
                root: parse_datum(ps.pop().unwrap())?.value,
            },
            Rule::datum => DoneDate::DateToDate {
                other: parse_datum(ps.pop().unwrap())?.value,
                root: parse_datum(ps.pop().unwrap())?.value,
            },
            _ => unreachable!(),
        },
        3 => DoneDate::DateTimeToTime {
            other_time: parse_time(ps.pop().unwrap())?.value,
            root_time: parse_time(ps.pop().unwrap())?.value,
            root: parse_datum(ps.pop().unwrap())?.value,
        },
        4 => DoneDate::DateTimeToDateTime {
            other_time: parse_time(ps.pop().unwrap())?.value,
            other: parse_datum(ps.pop().unwrap())?.value,
            root_time: parse_time(ps.pop().unwrap())?.value,
            root: parse_datum(ps.pop().unwrap())?.value,
        },
        _ => unreachable!(),
    })
}

fn parse_done_kind(p: Pair<'_, Rule>) -> DoneKind {
    assert_eq!(p.as_rule(), Rule::done_kind);
    match p.as_str() {
        "DONE" => DoneKind::Done,
        "CANCELED" => DoneKind::Canceled,
        _ => unreachable!(),
    }
}

fn parse_done(p: Pair<'_, Rule>) -> Result<Done> {
    assert_eq!(p.as_rule(), Rule::done);
    let mut p = p.into_inner();

    let kind = parse_done_kind(p.next().unwrap());
    let done_at = parse_datum(p.next().unwrap())?.value;
    let date = if let Some(p) = p.next() {
        Some(parse_donedate(p)?)
    } else {
        None
    };

    assert_eq!(p.next(), None);

    Ok(Done {
        kind,
        date,
        done_at,
    })
}

fn parse_dones(p: Pair<'_, Rule>) -> Result<Vec<Done>> {
    assert_eq!(p.as_rule(), Rule::dones);
    let mut dones = vec![];
    for p in p.into_inner() {
        dones.push(parse_done(p)?);
    }
    Ok(dones)
}

fn parse_desc_line(p: Pair<'_, Rule>) -> Result<String> {
    assert_eq!(p.as_rule(), Rule::desc_line);
    Ok(match p.into_inner().next() {
        None => "".to_string(),
        Some(p) => {
            assert_eq!(p.as_rule(), Rule::rest_any);
            p.as_str().trim_end().to_string()
        }
    })
}

fn parse_description(p: Pair<'_, Rule>) -> Result<Vec<String>> {
    assert_eq!(p.as_rule(), Rule::description);
    p.into_inner().map(parse_desc_line).collect()
}

fn parse_task(p: Pair<'_, Rule>) -> Result<Task> {
    assert_eq!(p.as_rule(), Rule::task);
    let mut p = p.into_inner();

    let title = parse_title(p.next().unwrap());
    let statements = parse_statements(p.next().unwrap(), true)?;
    let done = parse_dones(p.next().unwrap())?;
    let desc = parse_description(p.next().unwrap())?;

    assert_eq!(p.next(), None);

    Ok(Task {
        title,
        statements,
        done,
        desc,
    })
}

fn parse_note(p: Pair<'_, Rule>) -> Result<Note> {
    assert_eq!(p.as_rule(), Rule::note);
    let mut p = p.into_inner();

    let title = parse_title(p.next().unwrap());
    let statements = parse_statements(p.next().unwrap(), false)?;
    let desc = parse_description(p.next().unwrap())?;

    assert_eq!(p.next(), None);

    Ok(Note {
        title,
        statements,
        desc,
    })
}

fn parse_log_head(p: Pair<'_, Rule>) -> Result<Spanned<NaiveDate>> {
    assert_eq!(p.as_rule(), Rule::log_head);
    parse_datum(p.into_inner().next().unwrap())
}

fn parse_log(p: Pair<'_, Rule>) -> Result<Log> {
    assert_eq!(p.as_rule(), Rule::log);
    let mut p = p.into_inner();

    let date = parse_log_head(p.next().unwrap())?;
    let desc = parse_description(p.next().unwrap())?;

    assert_eq!(p.next(), None);

    Ok(Log { date, desc })
}

pub fn parse_command(p: Pair<'_, Rule>) -> Result<Spanned<Command>> {
    assert_eq!(p.as_rule(), Rule::command);

    let p = p.into_inner().next().unwrap();
    let span = (&p.as_span()).into();
    let command = match p.as_rule() {
        Rule::include => Command::Include(parse_include(p)),
        Rule::timezone => Command::Timezone(parse_timezone(p)),
        Rule::capture => Command::Capture,
        Rule::task => Command::Task(parse_task(p)?),
        Rule::note => Command::Note(parse_note(p)?),
        Rule::log => Command::Log(parse_log(p)?),
        _ => unreachable!(),
    };
    Ok(Spanned::new(span, command))
}

pub fn parse_file(p: Pair<'_, Rule>) -> Result<File> {
    assert_eq!(p.as_rule(), Rule::file);

    let mut commands = vec![];
    for p in p.into_inner() {
        // For some reason, the EOI in `file` always gets captured
        if p.as_rule() == Rule::EOI {
            break;
        }

        commands.push(parse_command(p)?);
    }

    Ok(File { commands })
}

pub fn parse(path: &Path, input: &str) -> Result<File> {
    let pathstr = path.to_string_lossy();

    let mut pairs = TodayfileParser::parse(Rule::file, input).map_err(|e| e.with_path(&pathstr))?;
    let file_pair = pairs.next().unwrap();
    assert_eq!(pairs.next(), None);

    parse_file(file_pair).map_err(|e| Box::new(e.with_path(&pathstr)))
}
