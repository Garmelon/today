use std::result;
use std::str::FromStr;

use chrono::NaiveDate;
use pest::iterators::Pair;
use pest::Parser;

use super::commands::{Command, Delta};
use super::parse::{self, Result, Rule, TodayfileParser};
use super::ParseError;

fn from_str_via_parse<P, R>(s: &str, rule: Rule, parse: P) -> result::Result<R, ParseError<()>>
where
    P: FnOnce(Pair<'_, Rule>) -> Result<R>,
{
    let mut pairs = TodayfileParser::parse(rule, s).map_err(|e| ParseError::new((), e))?;
    let p = pairs.next().unwrap();
    assert_eq!(pairs.next(), None);

    parse(p).map_err(|e| ParseError::new((), e))
}

#[derive(Debug)]
pub enum CliDatum {
    Date(NaiveDate),
    Today,
}

fn parse_cli_datum(p: Pair<'_, Rule>) -> Result<CliDatum> {
    assert_eq!(p.as_rule(), Rule::cli_datum);
    let p = p.into_inner().next().unwrap();
    Ok(match p.as_rule() {
        Rule::datum => CliDatum::Date(parse::parse_datum(p)?.value),
        Rule::today => CliDatum::Today,
        _ => unreachable!(),
    })
}

#[derive(Debug)]
pub struct CliDate {
    pub datum: CliDatum,
    pub delta: Option<Delta>,
}

fn parse_cli_date(p: Pair<'_, Rule>) -> Result<CliDate> {
    assert_eq!(p.as_rule(), Rule::cli_date);
    let mut p = p.into_inner();

    let datum = parse_cli_datum(p.next().unwrap())?;
    let delta = match p.next() {
        Some(p) => Some(parse::parse_delta(p)?.value),
        None => None,
    };

    assert_eq!(p.next(), None);

    Ok(CliDate { datum, delta })
}

impl FromStr for CliDate {
    type Err = ParseError<()>;

    fn from_str(s: &str) -> result::Result<Self, ParseError<()>> {
        from_str_via_parse(s, Rule::cli_date, parse_cli_date)
    }
}

#[derive(Debug)]
pub enum CliIdent {
    Number(usize),
    Date(CliDate),
}

fn parse_cli_ident(p: Pair<'_, Rule>) -> Result<CliIdent> {
    assert_eq!(p.as_rule(), Rule::cli_ident);
    let p = p.into_inner().next().unwrap();
    Ok(match p.as_rule() {
        Rule::number => CliIdent::Number(parse::parse_number(p) as usize),
        Rule::cli_date => CliIdent::Date(parse_cli_date(p)?),
        _ => unreachable!(),
    })
}

impl FromStr for CliIdent {
    type Err = ParseError<()>;

    fn from_str(s: &str) -> result::Result<Self, ParseError<()>> {
        from_str_via_parse(s, Rule::cli_ident, parse_cli_ident)
    }
}

#[derive(Debug)]
pub struct CliRange {
    pub start: CliDatum,
    pub start_delta: Option<Delta>,
    pub end: Option<CliDatum>,
    pub end_delta: Option<Delta>,
}

fn parse_cli_range_start(p: Pair<'_, Rule>) -> Result<(CliDatum, Option<Delta>)> {
    assert_eq!(p.as_rule(), Rule::cli_range_start);
    let mut p = p.into_inner();

    let start = parse_cli_datum(p.next().unwrap())?;
    let start_delta = match p.next() {
        None => None,
        Some(p) => Some(parse::parse_delta(p)?.value),
    };

    assert_eq!(p.next(), None);

    Ok((start, start_delta))
}

fn parse_cli_range_end(p: Pair<'_, Rule>) -> Result<(Option<CliDatum>, Option<Delta>)> {
    assert_eq!(p.as_rule(), Rule::cli_range_end);

    let mut end = None;
    let mut end_delta = None;

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::cli_datum => end = Some(parse_cli_datum(p)?),
            Rule::delta => end_delta = Some(parse::parse_delta(p)?.value),
            _ => unreachable!(),
        }
    }

    Ok((end, end_delta))
}

fn parse_cli_range(p: Pair<'_, Rule>) -> Result<CliRange> {
    assert_eq!(p.as_rule(), Rule::cli_range);
    let mut p = p.into_inner();

    let (start, start_delta) = parse_cli_range_start(p.next().unwrap())?;
    let (end, end_delta) = match p.next() {
        // For some reason, the EOI gets captured but the SOI doesn't.
        Some(p) if p.as_rule() != Rule::EOI => parse_cli_range_end(p)?,
        _ => (None, None),
    };

    Ok(CliRange {
        start,
        start_delta,
        end,
        end_delta,
    })
}

impl FromStr for CliRange {
    type Err = ParseError<()>;

    fn from_str(s: &str) -> result::Result<Self, ParseError<()>> {
        from_str_via_parse(s, Rule::cli_range, parse_cli_range)
    }
}

#[derive(Debug)]
pub struct CliCommand(pub Command);

fn parse_cli_command(p: Pair<'_, Rule>) -> Result<CliCommand> {
    assert_eq!(p.as_rule(), Rule::cli_command);
    let p = p.into_inner().next().unwrap();
    Ok(CliCommand(parse::parse_command(p)?.value))
}

impl FromStr for CliCommand {
    type Err = ParseError<()>;

    fn from_str(s: &str) -> result::Result<Self, ParseError<()>> {
        from_str_via_parse(s, Rule::cli_command, parse_cli_command)
    }
}
