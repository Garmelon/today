use std::result;

use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;

use crate::commands::{Command, Task};

#[derive(pest_derive::Parser)]
#[grammar = "parse/todayfile.pest"]
struct TodayfileParser;

type Result<T> = result::Result<T, Error<Rule>>;

pub fn parse(input: &str) -> Result<Vec<Command>> {
    let mut pairs = TodayfileParser::parse(Rule::file, input)?;
    let file = pairs.next().unwrap();
    let commands = file.into_inner();
    commands.map(parse_command).collect()
}

fn parse_command(p: Pair<Rule>) -> Result<Command> {
    match p.as_rule() {
        Rule::task => parse_task(p).map(Command::Task),
        Rule::note => todo!(),
        Rule::birthday => todo!(),
        _ => unreachable!(),
    }
}

fn parse_task(p: Pair<Rule>) -> Result<Task> {
    dbg!(p);
    todo!()
}
