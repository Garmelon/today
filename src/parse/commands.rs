use crate::commands::Command;

use super::{ParseResult, Parser};

pub fn parse(p: &mut Parser<'_>) -> ParseResult<Vec<Command>> {
    let mut commands = vec![];

    skip_empty_lines(p);
    while !p.at_eof() {
        // Commands consume all their trailing lines, including empty ones
        commands.push(parse_command(p)?);
    }

    Ok(commands)
}

fn skip_empty_lines(p: &mut Parser<'_>) {
    while p.peek_line().chars().all(|c| c.is_whitespace()) {
        p.take_line();
    }
}

fn parse_command(p: &mut Parser<'_>) -> ParseResult<Command> {
    let rest = p.peek_rest();
    if rest.starts_with("TASK") {
        todo!() // TODO Implement parsing TASK command
    } else if rest.starts_with("NOTE") {
        todo!() // TODO Implement parsing NOTE command
    } else if rest.starts_with("BIRTHDAY") {
        todo!() // TODO Implement parsing BIRTHDAY command
    } else {
        p.critical(p.at(), "Expected command")
    }
}
