use std::path::Path;
use std::result;

use chrono::NaiveDate;
use pest::error::ErrorVariant;
use pest::iterators::Pair;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::{Parser, Span};

use super::commands::{
    Birthday, BirthdaySpec, Command, DateSpec, Delta, DeltaStep, Done, DoneDate, Expr, File,
    FormulaSpec, Note, Spec, Task, Time, Var, Weekday, WeekdaySpec,
};

#[derive(pest_derive::Parser)]
#[grammar = "files/grammar.pest"]
struct TodayfileParser;

pub type Error = pest::error::Error<Rule>;
pub type Result<T> = result::Result<T, Error>;

fn error<S: Into<String>>(span: Span<'_>, message: S) -> Error {
    Error::new_from_span(
        ErrorVariant::CustomError {
            message: message.into(),
        },
        span,
    )
}

fn fail<S: Into<String>, T>(span: Span<'_>, message: S) -> Result<T> {
    Err(error(span, message))
}

fn parse_include(p: Pair<'_, Rule>) -> String {
    assert_eq!(p.as_rule(), Rule::include);
    p.into_inner().next().unwrap().as_str().to_string()
}

fn parse_timezone(p: Pair<'_, Rule>) -> String {
    assert_eq!(p.as_rule(), Rule::timezone);
    p.into_inner().next().unwrap().as_str().trim().to_string()
}

fn parse_number(p: Pair<'_, Rule>) -> i32 {
    assert_eq!(p.as_rule(), Rule::number);
    p.as_str().parse().unwrap()
}

fn parse_title(p: Pair<'_, Rule>) -> String {
    assert_eq!(p.as_rule(), Rule::title);
    let p = p.into_inner().next().unwrap();
    assert_eq!(p.as_rule(), Rule::rest_some);
    p.as_str().trim().to_string()
}

fn parse_datum(p: Pair<'_, Rule>) -> Result<NaiveDate> {
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

fn parse_time(p: Pair<'_, Rule>) -> Result<Time> {
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

fn parse_weekday(p: Pair<'_, Rule>) -> Weekday {
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

fn parse_delta_weekdays(p: Pair<'_, Rule>, sign: &mut Option<Sign>) -> Result<DeltaStep> {
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
    p: Pair<'_, Rule>,
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

fn parse_delta(p: Pair<'_, Rule>) -> Result<Delta> {
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

fn parse_date_fixed_start(p: Pair<'_, Rule>, spec: &mut DateSpec) -> Result<()> {
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

fn parse_date_fixed_end(p: Pair<'_, Rule>, spec: &mut DateSpec) -> Result<()> {
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

fn parse_date_fixed_repeat(p: Pair<'_, Rule>, spec: &mut DateSpec) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::date_fixed_repeat);
    let mut p = p.into_inner();

    if let Some(p) = p.next() {
        spec.repeat = Some(parse_delta(p)?);
    }

    assert_eq!(p.next(), None);
    Ok(())
}

fn parse_date_fixed(p: Pair<'_, Rule>) -> Result<DateSpec> {
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
        _ => unreachable!(),
    }
}

fn parse_unop_expr(p: Pair<'_, Rule>) -> Expr {
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

fn parse_paren_expr(p: Pair<'_, Rule>) -> Expr {
    assert_eq!(p.as_rule(), Rule::paren_expr);
    let inner = parse_expr(p.into_inner().next().unwrap());
    Expr::Paren(Box::new(inner))
}

fn parse_term(p: Pair<'_, Rule>) -> Expr {
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

fn parse_op(l: Expr, p: Pair<'_, Rule>, r: Expr) -> Expr {
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

fn parse_expr(p: Pair<'_, Rule>) -> Expr {
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

fn parse_date_expr_start(p: Pair<'_, Rule>, spec: &mut FormulaSpec) -> Result<()> {
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

fn parse_date_expr_end(p: Pair<'_, Rule>, spec: &mut FormulaSpec) -> Result<()> {
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
            Rule::weekday => spec.start = parse_weekday(p),
            Rule::time => spec.start_time = Some(parse_time(p)?),
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
            Rule::delta => spec.end_delta = Some(parse_delta(p)?),
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

fn parse_date(p: Pair<'_, Rule>) -> Result<Spec> {
    assert_eq!(p.as_rule(), Rule::date);
    let p = p.into_inner().next().unwrap();
    match p.as_rule() {
        Rule::date_fixed => parse_date_fixed(p).map(Spec::Date),
        Rule::date_expr => parse_date_expr(p).map(Spec::Formula),
        Rule::date_weekday => parse_date_weekday(p).map(Spec::Weekday),
        _ => unreachable!(),
    }
}

fn parse_from(p: Pair<'_, Rule>) -> Result<NaiveDate> {
    assert_eq!(p.as_rule(), Rule::from);
    parse_datum(p.into_inner().next().unwrap())
}

fn parse_until(p: Pair<'_, Rule>) -> Result<NaiveDate> {
    assert_eq!(p.as_rule(), Rule::until);
    parse_datum(p.into_inner().next().unwrap())
}

fn parse_except(p: Pair<'_, Rule>) -> Result<NaiveDate> {
    assert_eq!(p.as_rule(), Rule::except);
    parse_datum(p.into_inner().next().unwrap())
}

fn parse_donedate(p: Pair<'_, Rule>) -> Result<DoneDate> {
    assert_eq!(p.as_rule(), Rule::donedate);
    let mut ps = p.into_inner().collect::<Vec<_>>();

    // Popping the elements off of the vector in reverse so I don't have to
    // shuffle them around weirdly. In Haskell, I would've just pattern-matched
    // the list ;-;
    Ok(match ps.len() {
        1 => DoneDate::Date {
            root: parse_datum(ps.pop().unwrap())?,
        },
        2 => match ps[1].as_rule() {
            Rule::time => DoneDate::DateWithTime {
                root_time: parse_time(ps.pop().unwrap())?,
                root: parse_datum(ps.pop().unwrap())?,
            },
            Rule::datum => DoneDate::DateToDate {
                other: parse_datum(ps.pop().unwrap())?,
                root: parse_datum(ps.pop().unwrap())?,
            },
            _ => unreachable!(),
        },
        4 => DoneDate::DateToDateWithTime {
            other_time: parse_time(ps.pop().unwrap())?,
            other: parse_datum(ps.pop().unwrap())?,
            root_time: parse_time(ps.pop().unwrap())?,
            root: parse_datum(ps.pop().unwrap())?,
        },
        _ => unreachable!(),
    })
}

fn parse_done(p: Pair<'_, Rule>) -> Result<Done> {
    assert_eq!(p.as_rule(), Rule::done);
    let mut p = p.into_inner();

    let done_at = parse_datum(p.next().unwrap())?;
    let date = if let Some(p) = p.next() {
        Some(parse_donedate(p)?)
    } else {
        None
    };

    assert_eq!(p.next(), None);

    Ok(Done { date, done_at })
}

#[derive(Default)]
struct Options {
    when: Vec<Spec>,
    from: Option<NaiveDate>,
    until: Option<NaiveDate>,
    except: Vec<NaiveDate>,
    done: Vec<Done>,
}

fn parse_options(p: Pair<'_, Rule>) -> Result<Options> {
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

fn parse_note(p: Pair<'_, Rule>) -> Result<Note> {
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

fn parse_bdate(p: Pair<'_, Rule>) -> Result<BirthdaySpec> {
    assert_eq!(p.as_rule(), Rule::bdate);
    parse_bdatum(p.into_inner().next().unwrap())
}

fn parse_birthday(p: Pair<'_, Rule>) -> Result<Birthday> {
    assert_eq!(p.as_rule(), Rule::birthday);
    let mut p = p.into_inner();

    let title = parse_title(p.next().unwrap());
    let when = parse_bdate(p.next().unwrap())?;
    let desc = parse_description(p.next().unwrap())?;

    Ok(Birthday { title, when, desc })
}

fn parse_command(p: Pair<'_, Rule>, file: &mut File) -> Result<()> {
    assert_eq!(p.as_rule(), Rule::command);

    let p = p.into_inner().next().unwrap();
    match p.as_rule() {
        Rule::include => file.includes.push(parse_include(p)),
        Rule::timezone => match file.timezone {
            None => file.timezone = Some(parse_timezone(p)),
            Some(_) => fail(p.as_span(), "cannot set timezone multiple times")?,
        },
        Rule::task => file.commands.push(Command::Task(parse_task(p)?)),
        Rule::note => file.commands.push(Command::Note(parse_note(p)?)),
        Rule::birthday => file.commands.push(Command::Birthday(parse_birthday(p)?)),
        _ => unreachable!(),
    }

    Ok(())
}

pub fn parse_file(p: Pair<'_, Rule>) -> Result<File> {
    assert_eq!(p.as_rule(), Rule::file);

    let mut file = File {
        includes: vec![],
        timezone: None,
        commands: vec![],
    };

    for p in p.into_inner() {
        // For some reason, the EOI in `file` always gets captured
        if p.as_rule() == Rule::EOI {
            break;
        }

        parse_command(p, &mut file)?;
    }

    Ok(file)
}

pub fn parse(path: &Path, input: &str) -> Result<File> {
    let pathstr = path.to_string_lossy();

    let mut pairs = TodayfileParser::parse(Rule::file, input).map_err(|e| e.with_path(&pathstr))?;
    let file_pair = pairs.next().unwrap();
    assert_eq!(pairs.next(), None);

    parse_file(file_pair).map_err(|e| e.with_path(&pathstr))
}
