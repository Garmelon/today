use std::path::PathBuf;
use std::{process, result};

use chrono::{Duration, NaiveDate};
use directories::ProjectDirs;
use structopt::StructOpt;

use crate::eval::{DateRange, EntryMode};
use crate::files::{self, Files};

use self::error::Result;

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

pub fn run() -> Result<()> {
    let opt = Opt::from_args();

    let mut files = match load_files(&opt) {
        Ok(result) => result,
        Err(e) => {
            e.print();
            process::exit(1);
        }
    };

    let now = files.now().naive_local();

    let range_date = opt.date.unwrap_or_else(|| now.date());
    let range = DateRange::new(
        range_date - Duration::days(opt.before.into()),
        range_date + Duration::days(opt.after.into()),
    )
    .expect("determine range");

    let entries = files.eval(EntryMode::Relevant, range)?;
    let layout = layout::layout(&files, &entries, range, now);

    match opt.command {
        None => print::print(&layout),
        Some(Command::Show { entries: numbers }) => {
            show::show(&files, &entries, &layout, &numbers)?
        }
        Some(Command::Done { entries: numbers }) => {
            done::done(&mut files, &entries, &layout, &numbers, now)?
        }
        Some(Command::Fmt) => files.mark_all_dirty(),
    }

    if let Err(e) = files.save() {
        e.print();
        process::exit(1);
    }

    Ok(())
}
