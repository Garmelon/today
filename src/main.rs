use std::path::PathBuf;

use structopt::StructOpt;

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
    let files = Files::load(&opt.file)?;
    println!("{:#?}", files);
    Ok(())
}
