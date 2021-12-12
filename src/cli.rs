use std::path::PathBuf;

use chrono::NaiveDate;
use structopt::StructOpt;

use crate::eval::{DateRange, EntryMode};
use crate::files::Files;

use self::layout::Layout;
use self::render::Render;

mod layout;
mod render;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

pub fn run() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let files = Files::load(&opt.file)?;
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

    files.save()?;
    Ok(())
}
