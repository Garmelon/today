use std::path::PathBuf;
use std::process;

use chrono::NaiveDate;
use directories::ProjectDirs;
use structopt::StructOpt;

use crate::eval::{DateRange, EntryMode};
use crate::files::Files;

use self::layout::Layout;
use self::render::Render;

mod layout;
mod render;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// File to load
    #[structopt(short, long, parse(from_os_str))]
    file: Option<PathBuf>,
    /// Number of the entry to view or edit
    entry: Option<usize>,
    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Shows entries in a range, or a single entry if one is specified
    /// (default)
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

pub fn run() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let file = opt.file.unwrap_or_else(default_file);
    let mut files = Files::load(&file)?;
    let now = files.now().naive_local();

    let range = DateRange::new(
        NaiveDate::from_ymd(2021, 12, 12 - 3),
        NaiveDate::from_ymd(2021, 12, 12 + 13),
    )
    .unwrap();

    let entries = files.eval(EntryMode::Relevant, range)?;

    let mut layout = Layout::new(range, now);
    layout.layout(&files, &entries);

    let mut render = Render::new();
    render.render(&files, &entries, &layout);

    match opt.command {
        None | Some(Command::Show) => match opt.entry {
            None => print!("{}", render.display()),
            // Some(i) => print!("{}", render::render_entry(&files, &entries, &layout, i)),
            Some(i) => todo!(),
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
