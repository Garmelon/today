use std::path::PathBuf;
use std::str::FromStr;
use std::{process, result};

use chrono::{NaiveDate, NaiveDateTime};
use codespan_reporting::files::SimpleFile;
use directories::ProjectDirs;
use structopt::StructOpt;

use crate::eval::{self, DateRange, Entry, EntryMode};
use crate::files::arguments::{CliDate, CliIdent, CliRange};
use crate::files::{self, FileSource, Files, ParseError};

use self::error::Error;
use self::layout::line::LineLayout;

mod cancel;
mod done;
mod error;
mod layout;
mod log;
mod print;
mod show;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// File to load
    #[structopt(short, long, parse(from_os_str))]
    file: Option<PathBuf>,
    /// Overwrite the current date
    #[structopt(short, long)]
    date: Option<NaiveDate>,
    /// Range of days to focus on
    #[structopt(short, long, default_value = "t-2d--t+13d")]
    range: String,
    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[allow(rustdoc::broken_intra_doc_links)]
    /// Shows individual entries in detail
    Show {
        /// Entries and days to show
        #[structopt(required = true)]
        identifiers: Vec<String>,
    },
    /// Marks one or more entries as done
    Done {
        /// Entries to mark as done
        #[structopt(required = true)]
        entries: Vec<usize>,
    },
    /// Marks one or more entries as canceled
    Cancel {
        /// Entries to mark as done
        #[structopt(required = true)]
        entries: Vec<usize>,
    },
    /// Edits or creates a log entry
    Log {
        #[structopt(default_value = "today")]
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

fn find_now(opt: &Opt, files: &Files) -> NaiveDateTime {
    let now = files.now().naive_local();
    if let Some(date) = opt.date {
        date.and_time(now.time())
    } else {
        now
    }
}

fn find_entries(files: &Files, range: DateRange) -> Result<Vec<Entry>, Error<FileSource>> {
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

fn parse_eval_arg<T, R>(
    name: &str,
    text: &str,
    eval: impl FnOnce(T) -> Result<R, eval::Error<()>>,
) -> Option<R>
where
    T: FromStr<Err = ParseError<()>>,
{
    match T::from_str(text) {
        Ok(value) => match eval(value) {
            Ok(result) => return Some(result),
            Err(e) => crate::error::eprint_error(&SimpleFile::new(name, text), &e),
        },
        Err(e) => crate::error::eprint_error(&SimpleFile::new(name, text), &e),
    }
    None
}

fn parse_show_idents(identifiers: &[String], today: NaiveDate) -> Vec<show::Ident> {
    let mut idents = vec![];
    for ident in identifiers {
        let ident = match parse_eval_arg("identifier", ident, |ident: CliIdent| match ident {
            CliIdent::Number(n) => Ok(show::Ident::Number(n)),
            CliIdent::Date(d) => Ok(show::Ident::Date(d.eval((), today)?)),
        }) {
            Some(ident) => ident,
            None => process::exit(1),
        };
        idents.push(ident);
    }
    idents
}

fn run_command(
    opt: &Opt,
    files: &mut Files,
    range: DateRange,
    now: NaiveDateTime,
) -> Result<(), Error<FileSource>> {
    match &opt.command {
        None => {
            let entries = find_entries(files, range)?;
            let layout = find_layout(files, &entries, range, now);
            print::print(&layout);
        }
        Some(Command::Show { identifiers }) => {
            let entries = find_entries(files, range)?;
            let layout = find_layout(files, &entries, range, now);
            let idents = parse_show_idents(identifiers, now.date());
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
            match parse_eval_arg("date", date, |date: CliDate| date.eval((), now.date())) {
                Some(date) => log::log(files, date)?,
                None => process::exit(1),
            };
        }
        Some(Command::Fmt) => files.mark_all_dirty(),
    }
    Ok(())
}

pub fn run() {
    let opt = Opt::from_args();

    let mut files = Files::new();
    if let Err(e) = load_files(&opt, &mut files) {
        crate::error::eprint_error(&files, &e);
        process::exit(1);
    }

    let now = find_now(&opt, &files);

    let range = match parse_eval_arg("--range", &opt.range, |range: CliRange| {
        range.eval((), now.date())
    }) {
        Some(range) => range,
        None => process::exit(1),
    };

    if let Err(e) = run_command(&opt, &mut files, range, now) {
        crate::error::eprint_error(&files, &e);
        process::exit(1);
    }

    if let Err(e) = files.save() {
        crate::error::eprint_error(&files, &e);
        process::exit(1);
    }
}
