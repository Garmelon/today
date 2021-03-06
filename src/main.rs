#![warn(future_incompatible)]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]
#![warn(clippy::use_self)]

mod cli;
mod error;
mod eval;
mod files;

fn main() {
    cli::run();
}
