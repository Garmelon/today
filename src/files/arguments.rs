use std::str::FromStr;

use chrono::NaiveDate;
use pest::iterators::Pair;
use pest::Parser;

use super::commands::Delta;
use super::parse::{self, Error, Result, Rule, TodayfileParser};

#[derive(Debug)]
pub enum RangeDate {
    Date(NaiveDate),
    Today,
}

#[derive(Debug)]
pub struct Range {
    pub start: RangeDate,
    pub start_delta: Option<Delta>,
    pub end: Option<RangeDate>,
    pub end_delta: Option<Delta>,
}

/* Parsing */

fn parse_range_date(p: Pair<'_, Rule>) -> Result<RangeDate> {
    assert!(matches!(p.as_rule(), Rule::datum | Rule::today));
    Ok(match p.as_rule() {
        Rule::datum => RangeDate::Date(parse::parse_datum(p)?.value),
        Rule::today => RangeDate::Today,
        _ => unreachable!(),
    })
}

fn parse_range_start(p: Pair<'_, Rule>) -> Result<(RangeDate, Option<Delta>)> {
    assert_eq!(p.as_rule(), Rule::range_start);
    let mut p = p.into_inner();

    let start = parse_range_date(p.next().unwrap())?;
    let start_delta = match p.next() {
        None => None,
        Some(p) => Some(parse::parse_delta(p)?.value),
    };

    assert_eq!(p.next(), None);

    Ok((start, start_delta))
}

fn parse_range_end(p: Pair<'_, Rule>) -> Result<(Option<RangeDate>, Option<Delta>)> {
    assert_eq!(p.as_rule(), Rule::range_end);

    let mut end = None;
    let mut end_delta = None;

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::datum | Rule::today => end = Some(parse_range_date(p)?),
            Rule::delta => end_delta = Some(parse::parse_delta(p)?.value),
            _ => unreachable!(),
        }
    }

    Ok((end, end_delta))
}

fn parse_range(p: Pair<'_, Rule>) -> Result<Range> {
    assert_eq!(p.as_rule(), Rule::range);
    let mut p = p.into_inner();

    let (start, start_delta) = parse_range_start(p.next().unwrap())?;
    let (end, end_delta) = match p.next() {
        // For some reason, the EOI gets captured but the SOI doesn't.
        Some(p) if p.as_rule() != Rule::EOI => parse_range_end(p)?,
        _ => (None, None),
    };

    Ok(Range {
        start,
        start_delta,
        end,
        end_delta,
    })
}

impl FromStr for Range {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut pairs = TodayfileParser::parse(Rule::range, s)?;
        let p = pairs.next().unwrap();
        assert_eq!(pairs.next(), None);

        parse_range(p)
    }
}
