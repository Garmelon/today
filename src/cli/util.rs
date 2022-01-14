use colored::{ColoredString, Colorize};

use super::error::{Error, Result};
use super::layout::line::LineKind;

pub fn display_kind(kind: LineKind) -> ColoredString {
    match kind {
        LineKind::Task => "T".magenta().bold(),
        LineKind::Done => "D".green().bold(),
        LineKind::Canceled => "C".red().bold(),
        LineKind::Note => "N".blue().bold(),
        LineKind::Birthday => "B".yellow().bold(),
    }
}

pub fn edit(input: &str) -> Result<String> {
    edit::edit(input).map_err(Error::EditingIo)
}

pub fn edit_with_suffix(input: &str, suffix: &str) -> Result<String> {
    let mut builder = edit::Builder::new();
    builder.suffix(suffix);
    edit::edit_with_builder(input, &builder).map_err(Error::EditingIo)
}
