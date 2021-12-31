use std::path::PathBuf;
use std::str::FromStr;
use std::{process, result};

use chrono::{NaiveDate, NaiveDateTime};
use directories::ProjectDirs;
use structopt::StructOpt;

use crate::eval::{DateRange, Entry, EntryMode, SourceInfo};
use crate::files::arguments::Range;
use crate::files::{self, Files};

use self::error::Result;
use self::layout::line::LineLayout;

mod cancel;
mod done;
mod error;
mod layout;
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
    /// The range days to focus on
    #[structopt(short, long, default_value = "today-2d--today+13d")]
    range: String,
    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[allow(rustdoc::broken_intra_doc_links)]
    /// Shows individual entries in detail
    Show {
        /// Entries to show
        #[structopt(required = true)]
        entries: Vec<usize>,
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
    /// Reformat all loaded files
    Fmt,
}

fn default_file() -> PathBuf {
    ProjectDirs::from("", "", "today")
        .expect("could not determine config dir")
        .config_dir()
        .join("main.today")
}

fn load_files(opt: &Opt) -> result::Result<Files, files::Error> {
    let file = opt.file.clone().unwrap_or_else(default_file);
    Files::load(&file)
}

fn find_now(opt: &Opt, files: &Files) -> NaiveDateTime {
    let now = files.now().naive_local();
    if let Some(date) = opt.date {
        date.and_time(now.time())
    } else {
        now
    }
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

fn run_command(opt: &Opt, files: &mut Files, range: DateRange, now: NaiveDateTime) -> Result<()> {
    match &opt.command {
        None => {
            let entries = find_entries(files, range)?;
            let layout = find_layout(files, &entries, range, now);
            print::print(&layout);
        }
        Some(Command::Show { entries: ns }) => {
            let entries = find_entries(files, range)?;
            let layout = find_layout(files, &entries, range, now);
            show::show(files, &entries, &layout, ns)?;
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
        Some(Command::Fmt) => files.mark_all_dirty(),
    }
    Ok(())
}

pub fn run() {
    let opt = Opt::from_args();

    let mut files = match load_files(&opt) {
        Ok(result) => result,
        Err(e) => {
            e.print();
            process::exit(1);
        }
    };

    let now = find_now(&opt, &files);

    // Kinda ugly, but it can stay for now (until it grows at least).
    let range = match Range::from_str(&opt.range) {
        Ok(range) => match range.eval(0, now.date()) {
            Ok(range) => range,
            Err(e) => {
                eprintln!("Failed to evaluate --range:");
                e.print(&[SourceInfo {
                    name: Some("--range".to_string()),
                    content: &opt.range,
                }]);
                process::exit(1)
            }
        },
        Err(e) => {
            eprintln!("Failed to parse --range:\n{}", e.with_path("--range"));
            process::exit(1)
        }
    };

    if let Err(e) = run_command(&opt, &mut files, range, now) {
        e.print(&files.sources());
        process::exit(1);
    }

    if let Err(e) = files.save() {
        e.print();
        process::exit(1);
    }
}
