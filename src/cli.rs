use std::path::PathBuf;

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
    /// Reformat the file
    #[structopt(short, long)]
    reformat: bool,
    /// Reformat the file and all imports
    #[structopt(short = "R", long)]
    reformat_all: bool,
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
    print!("{}", render.display());

    if opt.reformat_all {
        files.mark_all_dirty();
    } else if opt.reformat {
        files.mark_main_dirty();
    }
    files.save()?;
    Ok(())
}
