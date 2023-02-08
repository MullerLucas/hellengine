// crate-config: start
#![deny(warnings)]
// crate-config: end

mod args;
mod crap;

// use args::CrapArgs;
// use clap::Parser;

fn main() {
    // let args = cli::Args::parse();
    crap::run().unwrap();
}
