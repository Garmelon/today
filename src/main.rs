use std::path::PathBuf;

use structopt::StructOpt;

mod commands;
mod parse;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);

    let commands = parse::parse(&opt.file);
    println!("{:#?}", commands);
}
