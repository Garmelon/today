#![warn(future_incompatible)]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]
#![warn(clippy::use_self)]

use std::path::PathBuf;

use chrono::NaiveDate;
use structopt::StructOpt;

use crate::eval::{DateRange, EntryMode};
use crate::files::Files;

mod eval;
mod files;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let mut files = Files::load(&opt.file)?;
    println!("{}", files.now().format("%F %T %Z"));

    let range = DateRange::new(
        NaiveDate::from_ymd(2021, 1, 1),
        NaiveDate::from_ymd(2022, 12, 31),
    )
    .unwrap();
    for entry in files.eval(EntryMode::Relevant, range)? {
        let title = files.command(entry.source).title();
        if let Some(dates) = entry.dates {
            println!("{:36} - {}", format!("{}", dates), title);
        } else {
            println!("{:36} - {}", "undated", title);
        }
    }

    files.mark_all_dirty();
    files.save()?;
    Ok(())
}
