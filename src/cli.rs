use std::path::PathBuf;
use std::{process, result};

use chrono::{Duration, NaiveDate, NaiveDateTime};
use directories::ProjectDirs;
use structopt::StructOpt;

use crate::eval::{DateRange, Entry, EntryMode};
use crate::files::{self, Files};

use self::error::Result;
use self::layout::line::LineLayout;

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
    // TODO Allow negative numbers for `before` and `after`
    // TODO Or just allow any Delta
    /// How many days to include before the current date
    #[structopt(short, long, default_value = "3")]
    before: u32,
    /// How many days to include after the current date
    #[structopt(short, long, default_value = "13")]
    after: u32,
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

fn find_range(opt: &Opt, now: NaiveDateTime) -> DateRange {
    let range_date = opt.date.unwrap_or_else(|| now.date());
    DateRange::new(
        range_date - Duration::days(opt.before.into()),
        range_date + Duration::days(opt.after.into()),
    )
    .expect("determine range")
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
    let range = find_range(&opt, now);

    if let Err(e) = run_command(&opt, &mut files, range, now) {
        e.print(&files.sources());
        process::exit(1);
    }

    if let Err(e) = files.save() {
        e.print();
        process::exit(1);
    }
}
