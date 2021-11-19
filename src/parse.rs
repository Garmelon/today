use std::result;

use chrono::NaiveDate;
use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;
use pest::{Parser, Span};

use crate::commands::{Birthday, BirthdaySpec, Command, Done, Note, Spec, Task};

#[derive(pest_derive::Parser)]
#[grammar = "parse/todayfile.pest"]
struct TodayfileParser;

type Result<T> = result::Result<T, Error<Rule>>;

#[must_use]
fn fail<S: Into<String>, T>(span: Span, message: S) -> Result<T> {
    Err(Error::new_from_span(
        ErrorVariant::CustomError {
            message: message.into(),
        },
        span,
    ))
}

fn parse_title(p: Pair<Rule>) -> Result<String> {
    assert_eq!(p.as_rule(), Rule::title);
    let p = p.into_inner().next().unwrap();
    assert_eq!(p.as_rule(), Rule::rest_some);
    Ok(p.as_str().to_string())
}

fn parse_datum(p: Pair<Rule>) -> Result<NaiveDate> {
    assert_eq!(p.as_rule(), Rule::datum);
    let date_span = p.as_span();
    let mut p = p.into_inner();

    let year = p.next().unwrap().as_str().parse().unwrap();
    let month = p.next().unwrap().as_str().parse().unwrap();
    let day = p.next().unwrap().as_str().parse().unwrap();

    assert_eq!(p.next(), None);

    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) => Ok(date),
        None => fail(date_span, "invalid date"),
    }
}

fn parse_date(p: Pair<Rule>) -> Result<Spec> {
    dbg!(p);
    todo!()
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

fn parse_done(p: Pair<Rule>) -> Result<Done> {
    dbg!(p);
    todo!()
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

    let title = parse_title(p.next().unwrap())?;
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
    assert_eq!(p.as_rule(), Rule::task);
    let mut p = p.into_inner();

    let title = parse_title(p.next().unwrap())?;
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

    let title = parse_title(p.next().unwrap())?;
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

pub fn parse(input: &str) -> Result<Vec<Command>> {
    let mut pairs = TodayfileParser::parse(Rule::file, input)?;
    let file = pairs.next().unwrap();
    file.into_inner()
        // For some reason, the EOI in `file` always gets captured
        .take_while(|p| p.as_rule() == Rule::command)
        .map(parse_command)
        .collect()
}
