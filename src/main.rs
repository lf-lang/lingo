extern crate termion;

pub mod analyzer;
pub mod args;
pub mod backends;
pub mod interface;
pub mod util;
pub mod package;

use args::{CommandLineArgs, Command};

use clap::Parser;
use std::path::Path;

fn main() {
    let args = CommandLineArgs::parse();
    match args.command {
        Command::Init {} => {
            let initial_config = package::Config::new();
            initial_config.write(Path::new("./Barrel.toml"));
            package::Config::setup_example();
        },
        Command::Build {} => {
            let config = package::Config::from(Path::new("./Barrel.toml"));
            if !backends::select_backend("lfc", config.package).build() {
                println!("error has occured!")
            }
        },
        _ => todo!()
    }
}


