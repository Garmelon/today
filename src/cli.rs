use std::path::PathBuf;

use chrono::NaiveDate;
use structopt::StructOpt;

use crate::eval::{DateRange, EntryMode};
use crate::files::Files;

use self::layout::Layout;

mod layout;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

pub fn run() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let files = Files::load(&opt.file)?;
    let now = files.now();
    let today = now.date().naive_local();

    let range = DateRange::new(
        NaiveDate::from_ymd(2021, 1, 1),
        NaiveDate::from_ymd(2022, 12, 31),
    )
    .unwrap();

    let entries = files.eval(EntryMode::Relevant, range)?;
    println!("{:#?}", entries);

    let mut layout = Layout::new(range, today);
    layout.layout(&files, &entries);
    println!("{:#?}", layout);

    files.save()?;
    Ok(())
}
