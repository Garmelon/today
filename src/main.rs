#![warn(future_incompatible)]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]
#![warn(clippy::use_self)]

use std::path::PathBuf;

use chrono::NaiveDate;
use structopt::StructOpt;

// use crate::eval::DateRange;
use crate::files::Files;

// mod eval;
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

    // let range = DateRange::new(
    //     NaiveDate::from_ymd(2021, 11, 20),
    //     NaiveDate::from_ymd(2021, 11, 26),
    // );
    // println!("{:#?}", files.eval(range));

    files.mark_all_dirty();
    files.save()?;

    Ok(())
}
