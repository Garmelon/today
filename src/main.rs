#![warn(future_incompatible)]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]
#![warn(clippy::use_self)]

// TODO Switch to new format syntax project-wide

mod cli;
mod error;
mod eval;
mod files;

fn main() {
    cli::run();
}
