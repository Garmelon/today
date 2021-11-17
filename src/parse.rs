use std::fs;
use std::path::Path;

use crate::commands::Command;

use self::line::parse_lines;

mod line;

pub fn parse(file: &Path) -> anyhow::Result<Vec<Command>> {
    let content = fs::read_to_string(file)?;
    let lines = parse_lines(&content)?;

    println!("{:#?}", lines);
    todo!()
}
