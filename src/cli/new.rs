use std::str::FromStr;

use chrono::NaiveDateTime;
use codespan_reporting::files::SimpleFile;

use crate::files::cli::CliCommand;
use crate::files::commands::{Done, DoneKind, Task};
use crate::files::Files;

use super::error::{Error, Result};
use super::util;

pub fn done(files: &mut Files, now: NaiveDateTime) -> Result<()> {
    let capture = files.capture().ok_or(Error::NoCaptureFile)?;

    let command = Task {
        title: String::new(),
        statements: vec![],
        done: vec![Done {
            kind: DoneKind::Done,
            date: None,
            done_at: now.date(),
        }],
        desc: vec![],
    };

    let mut text = format!("{command}");
    let command = loop {
        text = util::edit(&text)?;
        match CliCommand::from_str(&text) {
            Ok(command) => break command.0,
            Err(e) => crate::error::eprint_error(&SimpleFile::new("new command", &text), &e),
        }
        if !matches!(
            promptly::prompt_default("Continue editing?", true),
            Ok(true)
        ) {
            println!("Aborting");
            return Ok(());
        }
    };

    files.insert(capture, command);

    Ok(())
}
