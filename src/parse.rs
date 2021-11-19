use std::path::Path;
use std::result;

use chrono::NaiveDate;
use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::{Parser, Span};

use crate::commands::{
    Birthday, BirthdaySpec, Command, DateSpec, Delta, DeltaStep, Done, Expr, File, FormulaSpec,
    Note, Spec, Task, Time, Var, Weekday, WeekdaySpec,
};

#[derive(pest_derive::Parser)]
#[grammar = "parse/todayfile.pest"]
struct TodayfileParser;

type Result<T> = result::Result<T, Error<Rule>>;

fn error<S: Into<String>>(span: Span, message: S) -> Error<Rule> {
    Error::new_from_span(
        ErrorVariant::CustomError {
            message: message.into(),
        },
        span,
    )
}

fn fail<S: Into<String>, T>(span: Span, message: S) -> Result<T> {
    Err(error(span, message))
}

fn parse_number(p: Pair<Rule>) -> i32 {
    assert_eq!(p.as_rule(), Rule::number);
    p.as_str().parse().unwrap()
}

fn parse_title(p: Pair<Rule>) -> String {
    assert_eq!(p.as_rule(), Rule::title);
    let p = p.into_inner().next().unwrap();
    assert_eq!(p.as_rule(), Rule::rest_some);
    p.as_str().to_string()
}

fn parse_datum(p: Pair<Rule>) -> Result<NaiveDate> {
    assert_eq!(p.as_rule(), Rule::datum);
    let span = p.as_span();
    let mut p = p.into_inner();

    let year = p.next().unwrap().as_str().parse().unwrap();
    let month = p.next().unwrap().as_str().parse().unwrap();
    let day = p.next().unwrap().as_str().parse().unwrap();

    assert_eq!(p.next(), None);

    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) => Ok(date),
        None => fail(span, "invalid date"),
    }
}

fn parse_time(p: Pair<Rule>) -> Result<Time> {
    assert_eq!(p.as_rule(), Rule::time);
    let span = p.as_span();
    let mut p = p.into_inner();

    let hour = p.next().unwrap().as_str().parse().unwrap();
    let min = p.next().unwrap().as_str().parse().unwrap();

    assert_eq!(p.next(), None);

    match Time::new(hour, min) {
        Some(time) => Ok(time),
        None => fail(span, "invalid time"),
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

fn parse_amount(p: Pair<Rule>) -> Amount {
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

fn parse_weekday(p: Pair<Rule>) -> Weekday {
    assert_eq!(p.as_rule(), Rule::weekday);
    match p.as_str() {
        "mon" => Weekday::Monday,
        "tue" => Weekday::Tuesday,
        "wed" => Weekday::Wednesday,
        "thu" => Weekday::Thursday,
        "fri" => Weekday::Friday,
        "sat" => Weekday::Saturday,
        "sun" => Weekday::Sunday,
        _ => unreachable!(),
    }
}

fn parse_delta_weekdays(p: Pair<Rule>, sign: &mut Option<Sign>) -> Result<DeltaStep> {
    assert_eq!(p.as_rule(), Rule::delta_weekdays);
    let span = p.as_span();
    let mut p = p.into_inner();

    let amount = parse_amount(p.next().unwrap()).with_prev_sign(*sign);
    let weekday = parse_weekday(p.next().unwrap());

    assert_eq!(p.next(), None);

    let value = amount
        .value()
        .ok_or_else(|| error(span, "ambiguous sign"))?;
    *sign = amount.sign;

    Ok(DeltaStep::Weekday(value, weekday))
}

fn parse_delta_step(
    p: Pair<Rule>,
    sign: &mut Option<Sign>,
    f: impl FnOnce(i32) -> DeltaStep,
) -> Result<DeltaStep> {
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

    let span = p.as_span();
    let amount = parse_amount(p.into_inner().next().unwrap()).with_prev_sign(*sign);
    let value = amount
        .value()
        .ok_or_else(|| error(span, "ambiguous sign"))?;

    *sign = amount.sign;
    Ok(f(value))
}

fn parse_delta(p: Pair<Rule>) -> Result<Delta> {
    assert_eq!(p.as_rule(), Rule::delta);

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

    Ok(Delta(steps))
}

fn parse_date_fixed_start(p: Pair<Rule>, spec: &mut DateSpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_fixed_start);

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::datum => spec.start = parse_datum(p)?,
            Rule::delta => spec.start_delta = Some(parse_delta(p)?),
            Rule::time => spec.start_time = Some(parse_time(p)?),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn parse_date_fixed_end(p: Pair<Rule>, spec: &mut DateSpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_fixed_end);

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::datum => spec.end = Some(parse_datum(p)?),
            Rule::delta => spec.end_delta = Some(parse_delta(p)?),
            Rule::time => spec.end_time = Some(parse_time(p)?),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn parse_date_fixed_repeat(p: Pair<Rule>, spec: &mut DateSpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_fixed_repeat);
    let mut p = p.into_inner();

    if let Some(p) = p.next() {
        spec.repeat = Some(parse_delta(p)?);
    }

    assert_eq!(p.next(), None);
    Ok(())
}

fn parse_date_fixed(p: Pair<Rule>) -> Result<DateSpec> {
    assert_eq!(p.as_rule(), Rule::date_fixed);

    let mut spec = DateSpec {
        start: NaiveDate::from_ymd(0, 1, 1),
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

fn parse_boolean(p: Pair<Rule>) -> Var {
    assert_eq!(p.as_rule(), Rule::boolean);
    match p.as_str() {
        "true" => Var::True,
        "false" => Var::False,
        _ => unreachable!(),
    }
}

fn parse_variable(p: Pair<Rule>) -> Var {
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
        _ => unreachable!(),
    }
}

fn parse_unop_expr(p: Pair<Rule>) -> Expr {
    assert_eq!(p.as_rule(), Rule::unop_expr);
    let mut p = p.into_inner();
    let p_op = p.next().unwrap();
    let p_expr = p.next().unwrap();
    assert_eq!(p.next(), None);

    let expr = parse_expr(p_expr);
    match p_op.as_rule() {
        Rule::unop_neg => Expr::Neg(Box::new(expr)),
        Rule::unop_not => Expr::Not(Box::new(expr)),
        _ => unreachable!(),
    }
}

fn parse_paren_expr(p: Pair<Rule>) -> Expr {
    assert_eq!(p.as_rule(), Rule::paren_expr);
    let inner = parse_expr(p.into_inner().next().unwrap());
    Expr::Paren(Box::new(inner))
}

fn parse_term(p: Pair<Rule>) -> Expr {
    assert_eq!(p.as_rule(), Rule::term);
    let p = p.into_inner().next().unwrap();
    match p.as_rule() {
        Rule::number => Expr::Lit(parse_number(p).into()),
        Rule::boolean => Expr::Var(parse_boolean(p)),
        Rule::variable => Expr::Var(parse_variable(p)),
        Rule::unop_expr => parse_unop_expr(p),
        Rule::paren_expr => parse_paren_expr(p),
        _ => unreachable!(),
    }
}

fn parse_op(l: Expr, p: Pair<Rule>, r: Expr) -> Expr {
    match p.as_rule() {
        // Integer-y operations
        Rule::op_add => Expr::Add(Box::new(l), Box::new(r)),
        Rule::op_sub => Expr::Sub(Box::new(l), Box::new(r)),
        Rule::op_mul => Expr::Mul(Box::new(l), Box::new(r)),
        Rule::op_div => Expr::Div(Box::new(l), Box::new(r)),
        Rule::op_mod => Expr::Mod(Box::new(l), Box::new(r)),

        // Comparisons
        Rule::op_eq => Expr::Eq(Box::new(l), Box::new(r)),
        Rule::op_neq => Expr::Neq(Box::new(l), Box::new(r)),
        Rule::op_lt => Expr::Lt(Box::new(l), Box::new(r)),
        Rule::op_lte => Expr::Lte(Box::new(l), Box::new(r)),
        Rule::op_gt => Expr::Gt(Box::new(l), Box::new(r)),
        Rule::op_gte => Expr::Gte(Box::new(l), Box::new(r)),

        // Boolean-y operations
        Rule::op_and => Expr::And(Box::new(l), Box::new(r)),
        Rule::op_or => Expr::Or(Box::new(l), Box::new(r)),
        Rule::op_xor => Expr::Xor(Box::new(l), Box::new(r)),

        _ => unreachable!(),
    }
}

fn parse_expr(p: Pair<Rule>) -> Expr {
    assert_eq!(p.as_rule(), Rule::expr);

    fn op(rule: Rule) -> Operator<Rule> {
        Operator::new(rule, Assoc::Left)
    }

    let climber = PrecClimber::new(vec![
        // Precedence from low to high
        op(Rule::op_or) | op(Rule::op_xor),
        op(Rule::op_and),
        op(Rule::op_eq) | op(Rule::op_neq),
        op(Rule::op_lt) | op(Rule::op_lte) | op(Rule::op_gt) | op(Rule::op_gte),
        op(Rule::op_mul) | op(Rule::op_div) | op(Rule::op_mod),
        op(Rule::op_add) | op(Rule::op_sub),
    ]);

    climber.climb(p.into_inner(), parse_term, parse_op)
}

fn parse_date_expr_start(p: Pair<Rule>, spec: &mut FormulaSpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_expr_start);

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::paren_expr => spec.start = Some(parse_expr(p.into_inner().next().unwrap())),
            Rule::delta => spec.start_delta = Some(parse_delta(p)?),
            Rule::time => spec.start_time = Some(parse_time(p)?),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn parse_date_expr_end(p: Pair<Rule>, spec: &mut FormulaSpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_expr_end);

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::delta => spec.end_delta = Some(parse_delta(p)?),
            Rule::time => spec.end_time = Some(parse_time(p)?),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn parse_date_expr(p: Pair<Rule>) -> Result<FormulaSpec> {
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

fn parse_date_weekday_start(p: Pair<Rule>, spec: &mut WeekdaySpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_weekday_start);

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::weekday => spec.start = parse_weekday(p),
            Rule::time => spec.start_time = Some(parse_time(p)?),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn parse_date_weekday_end(p: Pair<Rule>, spec: &mut WeekdaySpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_weekday_end);

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::weekday => spec.end = Some(parse_weekday(p)),
            Rule::delta => spec.end_delta = Some(parse_delta(p)?),
            Rule::time => spec.end_time = Some(parse_time(p)?),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn parse_date_weekday(p: Pair<Rule>) -> Result<WeekdaySpec> {
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

fn parse_date(p: Pair<Rule>) -> Result<Spec> {
    assert_eq!(p.as_rule(), Rule::date);
    let p = p.into_inner().next().unwrap();
    match p.as_rule() {
        Rule::date_fixed => parse_date_fixed(p).map(Spec::Date),
        Rule::date_expr => parse_date_expr(p).map(Spec::Formula),
        Rule::date_weekday => parse_date_weekday(p).map(Spec::Weekday),
        _ => unreachable!(),
    }
}

fn parse_from(p: Pair<Rule>) -> Result<NaiveDate> {
    assert_eq!(p.as_rule(), Rule::from);
    parse_datum(p.into_inner().next().unwrap())
}

fn parse_until(p: Pair<Rule>) -> Result<NaiveDate> {
    assert_eq!(p.as_rule(), Rule::until);
    parse_datum(p.into_inner().next().unwrap())
}

fn parse_except(p: Pair<Rule>) -> Result<NaiveDate> {
    assert_eq!(p.as_rule(), Rule::except);
    parse_datum(p.into_inner().next().unwrap())
}

fn parse_donedate(p: Pair<Rule>) -> Result<(NaiveDate, Time)> {
    assert_eq!(p.as_rule(), Rule::donedate);
    let mut p = p.into_inner();

    let date = parse_datum(p.next().unwrap())?;
    let time = parse_time(p.next().unwrap())?;

    assert_eq!(p.next(), None);

    Ok((date, time))
}

fn parse_done(p: Pair<Rule>) -> Result<Done> {
    assert_eq!(p.as_rule(), Rule::done);

    let mut refering_to = None;
    let mut created_at = None;

    for ele in p.into_inner() {
        match ele.as_rule() {
            Rule::datum => refering_to = Some(parse_datum(ele)?),
            Rule::donedate => created_at = Some(parse_donedate(ele)?),
            _ => unreachable!(),
        }
    }

    Ok(Done {
        refering_to,
        created_at,
    })
}

#[derive(Default)]
struct Options {
    when: Vec<Spec>,
    from: Option<NaiveDate>,
    until: Option<NaiveDate>,
    except: Vec<NaiveDate>,
    done: Vec<Done>,
}

fn parse_options(p: Pair<Rule>) -> Result<Options> {
    assert!(matches!(
        p.as_rule(),
        Rule::task_options | Rule::note_options
    ));

    let mut opts = Options::default();
    for opt in p.into_inner() {
        match opt.as_rule() {
            Rule::date => opts.when.push(parse_date(opt)?),
            Rule::from if opts.from.is_none() => opts.from = Some(parse_from(opt)?),
            Rule::from => fail(opt.as_span(), "FROM already defined earlier")?,
            Rule::until if opts.until.is_none() => opts.until = Some(parse_until(opt)?),
            Rule::until => fail(opt.as_span(), "UNTIL already defined earlier")?,
            Rule::except => opts.except.push(parse_except(opt)?),
            Rule::done => opts.done.push(parse_done(opt)?),
            _ => unreachable!(),
        }
    }

    Ok(opts)
}

fn parse_desc_line(p: Pair<Rule>) -> Result<String> {
    assert_eq!(p.as_rule(), Rule::desc_line);
    Ok(match p.into_inner().next() {
        None => "".to_string(),
        Some(p) => {
            assert_eq!(p.as_rule(), Rule::rest_any);
            p.as_str().to_string()
        }
    })
}

fn parse_description(p: Pair<Rule>) -> Result<Vec<String>> {
    assert_eq!(p.as_rule(), Rule::description);
    p.into_inner().map(parse_desc_line).collect()
}

fn parse_task(p: Pair<Rule>) -> Result<Task> {
    assert_eq!(p.as_rule(), Rule::task);
    let mut p = p.into_inner();

    let title = parse_title(p.next().unwrap());
    let opts = parse_options(p.next().unwrap())?;
    let desc = parse_description(p.next().unwrap())?;

    assert_eq!(p.next(), None);

    Ok(Task {
        title,
        when: opts.when,
        from: opts.from,
        until: opts.until,
        except: opts.except,
        done: opts.done,
        desc,
    })
}

fn parse_note(p: Pair<Rule>) -> Result<Note> {
    assert_eq!(p.as_rule(), Rule::note);
    let mut p = p.into_inner();

    let title = parse_title(p.next().unwrap());
    let opts = parse_options(p.next().unwrap())?;
    let desc = parse_description(p.next().unwrap())?;

    assert_eq!(p.next(), None);
    assert!(opts.done.is_empty());

    Ok(Note {
        title,
        when: opts.when,
        from: opts.from,
        until: opts.until,
        except: opts.except,
        desc,
    })
}

fn parse_bdatum(p: Pair<Rule>) -> Result<BirthdaySpec> {
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

fn parse_bdate(p: Pair<Rule>) -> Result<BirthdaySpec> {
    assert_eq!(p.as_rule(), Rule::bdate);
    parse_bdatum(p.into_inner().next().unwrap())
}

fn parse_birthday(p: Pair<Rule>) -> Result<Birthday> {
    assert_eq!(p.as_rule(), Rule::birthday);
    let mut p = p.into_inner();

    let title = parse_title(p.next().unwrap());
    let when = parse_bdate(p.next().unwrap())?;
    let desc = parse_description(p.next().unwrap())?;

    Ok(Birthday { title, when, desc })
}

fn parse_command(p: Pair<Rule>) -> Result<Command> {
    assert_eq!(p.as_rule(), Rule::command);

    let p = p.into_inner().next().unwrap();
    match p.as_rule() {
        Rule::task => parse_task(p).map(Command::Task),
        Rule::note => parse_note(p).map(Command::Note),
        Rule::birthday => parse_birthday(p).map(Command::Birthday),
        _ => unreachable!(),
    }
}

pub fn parse(path: &Path, input: &str) -> Result<File> {
    let path = path.to_string_lossy();
    let mut pairs = TodayfileParser::parse(Rule::file, input)?;
    let file = pairs.next().unwrap();
    let commands = file
        .into_inner()
        // For some reason, the EOI in `file` always gets captured
        .take_while(|p| p.as_rule() == Rule::command)
        .map(parse_command)
        .collect::<Result<_>>()
        .map_err(|e| e.with_path(&path))?;
    Ok(File { commands })
}
