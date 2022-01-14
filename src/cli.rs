use std::path::PathBuf;
use std::str::FromStr;
use std::{process, result};

use chrono::{NaiveDate, NaiveDateTime};
use codespan_reporting::files::SimpleFile;
use directories::ProjectDirs;
use structopt::StructOpt;

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
mod print;
mod show;
mod util;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// File to load
    #[structopt(short, long, parse(from_os_str))]
    file: Option<PathBuf>,
    /// Overwrite the current date
    #[structopt(short, long, default_value = "t")]
    date: String,
    /// Range of days to focus on
    #[structopt(short, long, default_value = "t-2d--t+13d")]
    range: String,
    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Shows individual entries in detail
    #[structopt(alias = "s")]
    Show {
        /// Entries and days to show
        #[structopt(required = true)]
        identifiers: Vec<String>,
    },
    /// Marks one or more entries as done
    #[structopt(alias = "d")]
    Done {
        /// Entries to mark as done
        #[structopt(required = true)]
        entries: Vec<usize>,
    },
    /// Marks one or more entries as canceled
    #[structopt(alias = "c")]
    Cancel {
        /// Entries to mark as done
        #[structopt(required = true)]
        entries: Vec<usize>,
    },
    /// Edits or creates a log entry
    #[structopt(alias = "l")]
    Log {
        #[structopt(default_value = "t")]
        date: String,
    },
    /// Reformats all loaded files
    Fmt,
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
    let opt = Opt::from_args();

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
