use std::path::PathBuf;
use std::process;

use chrono::{Duration, NaiveDate};
use directories::ProjectDirs;
use structopt::StructOpt;

use crate::eval::{DateRange, EntryMode};
use crate::files::Files;

use self::error::Result;

mod error;
mod layout;
mod show;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// File to load
    #[structopt(short, long, parse(from_os_str))]
    file: Option<PathBuf>,
    /// Overwrite the current date
    #[structopt(short, long)]
    date: Option<NaiveDate>,
    /// How many days to include before the current date
    #[structopt(short, long, default_value = "3")]
    before: u32,
    /// How many days to include after the current date
    #[structopt(short, long, default_value = "13")]
    after: u32,
    /// Number of the entry to view or edit
    // TODO Select multiple entries at once
    entry: Option<usize>,
    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[allow(rustdoc::broken_intra_doc_links)]
    /// Shows entries in a range, or a single entry if one is specified
    /// [default]
    Show,
    /// Marks an entry as done (requires entry)
    Done,
    /// Reformat all loaded files
    Fmt,
}

fn default_file() -> PathBuf {
    ProjectDirs::from("", "", "today")
        .expect("could not determine config dir")
        .config_dir()
        .join("main.today")
}

pub fn run() -> Result<()> {
    let opt = Opt::from_args();

    let file = opt.file.unwrap_or_else(default_file);
    let mut files = Files::load(&file)?;
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
        None | Some(Command::Show) => match opt.entry {
            None => print!("{}", show::show_all(&layout)),
            Some(n) => show::show_entry(&files, &entries, &layout, n)?,
        },
        Some(Command::Done) => match opt.entry {
            None => {
                println!("Please specify an entry. See `today --help` for more details.");
                process::exit(1);
            }
            Some(i) => todo!(),
        },
        Some(Command::Fmt) => files.mark_all_dirty(),
    }

    files.save()?;
    Ok(())
}
