use std::path::PathBuf;
use std::str::FromStr;
use std::{process, result};

use chrono::{NaiveDate, NaiveDateTime};
use clap::Parser;
use codespan_reporting::files::SimpleFile;
use directories::ProjectDirs;

use crate::eval::{self, DateRange, Entry, EntryMode};
use crate::files::cli::{CliDate, CliIdent, CliRange};
use crate::files::{self, Files, ParseError};

use self::error::{Error, Result};
use self::layout::line::LineLayout;

mod cancel;
mod done;
mod error;
mod layout;
mod log;
mod new;
mod print;
mod show;
mod util;

#[derive(Debug, clap::Parser)]
pub struct Opt {
    /// File to load
    #[clap(short, long)]
    file: Option<PathBuf>,
    /// Overwrite the current date
    #[clap(short, long, default_value = "t")]
    date: String,
    /// Range of days to focus on
    #[clap(short, long, default_value = "t-2d--t+2w")]
    range: String,
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Shows individual entries in detail
    #[clap(alias = "s")]
    Show {
        /// Entries and days to show
        #[clap(required = true)]
        identifiers: Vec<String>,
    },
    /// Create a new entry based on a template
    #[clap(alias = "n")]
    New {
        #[clap(subcommand)]
        template: Template,
    },
    /// Marks one or more entries as done
    #[clap(alias = "d")]
    Done {
        /// Entries to mark as done
        #[clap(required = true)]
        entries: Vec<usize>,
    },
    /// Marks one or more entries as canceled
    #[clap(alias = "c")]
    Cancel {
        /// Entries to mark as done
        #[clap(required = true)]
        entries: Vec<usize>,
    },
    /// Edits or creates a log entry
    #[clap(alias = "l")]
    Log {
        #[clap(default_value = "t")]
        date: String,
    },
    /// Reformats all loaded files
    Fmt,
}

#[derive(Debug, clap::Subcommand)]
pub enum Template {
    /// Adds a task
    #[clap(alias = "t")]
    Task {
        /// If specified, the task is dated to this date
        date: Option<String>,
    },
    /// Adds a note
    #[clap(alias = "n")]
    Note {
        /// If specified, the note is dated to this date
        date: Option<String>,
    },
    /// Adds an undated task marked as done today
    #[clap(alias = "d")]
    Done,
}

fn default_file() -> PathBuf {
    ProjectDirs::from("", "", "today")
        .expect("could not determine config dir")
        .config_dir()
        .join("main.today")
}

fn load_files(opt: &Opt, files: &mut Files) -> result::Result<(), files::Error> {
    let file = opt.file.clone().unwrap_or_else(default_file);
    files.load(&file)
}

fn find_entries(files: &Files, range: DateRange) -> Result<Vec<Entry>> {
    Ok(files.eval(EntryMode::Relevant, range)?)
}

fn find_layout(
    files: &Files,
    entries: &[Entry],
    range: DateRange,
    now: NaiveDateTime,
) -> LineLayout {
    layout::layout(files, entries, range, now)
}

fn parse_eval_arg<T, E, R>(name: &str, text: &str, eval: E) -> Result<R>
where
    T: FromStr<Err = ParseError<()>>,
    E: FnOnce(T) -> result::Result<R, eval::Error<()>>,
{
    let value = T::from_str(text).map_err(|error| Error::ArgumentParse {
        file: SimpleFile::new(name.to_string(), text.to_string()),
        error,
    })?;
    eval(value).map_err(|error| Error::ArgumentEval {
        file: SimpleFile::new(name.to_string(), text.to_string()),
        error,
    })
}

fn parse_eval_date(name: &str, text: &str, today: NaiveDate) -> Result<NaiveDate> {
    parse_eval_arg(name, text, |date: CliDate| date.eval((), today))
}

fn parse_show_idents(identifiers: &[String], today: NaiveDate) -> Result<Vec<show::Ident>> {
    let mut idents = vec![];
    for ident in identifiers {
        let ident = parse_eval_arg("identifier", ident, |ident: CliIdent| match ident {
            CliIdent::Number(n) => Ok(show::Ident::Number(n)),
            CliIdent::Date(d) => Ok(show::Ident::Date(d.eval((), today)?)),
        })?;
        idents.push(ident);
    }
    Ok(idents)
}

fn run_command(opt: &Opt, files: &mut Files, range: DateRange, now: NaiveDateTime) -> Result<()> {
    match &opt.command {
        None => {
            let entries = find_entries(files, range)?;
            let layout = find_layout(files, &entries, range, now);
            print::print(&layout);
        }
        Some(Command::Show { identifiers }) => {
            let entries = find_entries(files, range)?;
            let layout = find_layout(files, &entries, range, now);
            let idents = parse_show_idents(identifiers, now.date())?;
            show::show(files, &entries, &layout, &idents);
        }
        Some(Command::New { template }) => match template {
            Template::Task { date: Some(date) } => {
                let date = parse_eval_date("date", date, now.date())?;
                new::task(files, Some(date))?
            }
            Template::Task { date: None } => new::task(files, None)?,
            Template::Note { date: Some(date) } => {
                let date = parse_eval_date("date", date, now.date())?;
                new::note(files, Some(date))?
            }
            Template::Note { date: None } => new::note(files, None)?,
            Template::Done => new::done(files, now.date())?,
        },
        Some(Command::Done { entries: ns }) => {
            let entries = find_entries(files, range)?;
            let layout = find_layout(files, &entries, range, now);
            done::done(files, &entries, &layout, ns, now)?;
            let entries = find_entries(files, range)?;
            let layout = find_layout(files, &entries, range, now);
            print::print(&layout);
        }
        Some(Command::Cancel { entries: ns }) => {
            let entries = find_entries(files, range)?;
            let layout = find_layout(files, &entries, range, now);
            cancel::cancel(files, &entries, &layout, ns, now)?;
            let entries = find_entries(files, range)?;
            let layout = find_layout(files, &entries, range, now);
            print::print(&layout);
        }
        Some(Command::Log { date }) => {
            let date = parse_eval_arg("date", date, |date: CliDate| date.eval((), now.date()))?;
            log::log(files, date)?
        }
        Some(Command::Fmt) => files.mark_all_dirty(),
    }
    Ok(())
}

fn run_with_files(opt: Opt, files: &mut Files) -> Result<()> {
    let now = files.now().naive_local();
    let today = parse_eval_arg("--date", &opt.date, |date: CliDate| {
        date.eval((), now.date())
    })?;
    let now = today.and_time(now.time());

    let range = parse_eval_arg("--range", &opt.range, |range: CliRange| {
        range.eval((), now.date())
    })?;

    run_command(&opt, files, range, now)?;

    Ok(())
}

pub fn run() {
    let opt = Opt::parse();

    let mut files = Files::new();
    if let Err(e) = load_files(&opt, &mut files) {
        crate::error::eprint_error(&files, &e);
        process::exit(1);
    }

    if let Err(e) = run_with_files(opt, &mut files) {
        crate::error::eprint_error(&files, &e);
        process::exit(1);
    }

    if let Err(e) = files.save() {
        crate::error::eprint_error(&files, &e);
        process::exit(1);
    }
}
