use std::path::PathBuf;
use std::process;

use structopt::StructOpt;

use crate::parse::Parser;
use crate::source::SourceFiles;

mod commands;
mod parse;
mod source;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let mut files = SourceFiles::new();

    let (file, content) = match files.load(&opt.file) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed to load file: {}", e);
            process::exit(1);
        }
    };

    let mut parser = Parser::new(file, content);

    let commands = match parser.parse(parse::parse_commands) {
        Ok(result) => result,
        Err(es) => {
            files.emit_all(&es);
            process::exit(1);
        }
    };

    println!("{:#?}", commands);
}
