use std::fs;
use std::path::PathBuf;

use chrono::NaiveDate;
use structopt::StructOpt;

use crate::eval::entries::DateRange;

mod commands;
mod eval;
mod format;
mod parse;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let content = fs::read_to_string(&opt.file)?;
    let file = parse::parse(&opt.file, &content)?;
    let entries = eval::eval(
        &file,
        DateRange::new(
            NaiveDate::from_ymd(2021, 1, 1),
            NaiveDate::from_ymd(2021, 12, 31),
        ),
    )?;
    println!("{:#?}", entries);
    Ok(())
}
