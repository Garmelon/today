use colored::{ColoredString, Colorize};

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
