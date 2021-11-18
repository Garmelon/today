use std::fs;
use std::path::PathBuf;

use structopt::StructOpt;

mod commands;
mod parse;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let content = fs::read_to_string(&opt.file)?;
    let commands = parse::parse(&content)?;
    println!("{:#?}", commands);
    Ok(())
}
