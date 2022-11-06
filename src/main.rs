extern crate termion;

pub mod analyzer;
pub mod args;
pub mod backends;
pub mod interface;
pub mod package;
pub mod util;

use args::{Command, CommandLineArgs};

use clap::Parser;
use std::path::Path;

fn main() {
    let args = CommandLineArgs::parse();
    match args.command {
        Command::Init => {
            let initial_config = package::Config::new();
            initial_config.write(Path::new("./Barrel.toml"));
            package::Config::setup_example();
        }
        Command::Build(build_command_args) => {
            let config = package::Config::from(Path::new("./Barrel.toml"));

            if config.is_none() {
                return;
            }

            if !backends::select_backend("lfc", config.unwrap().package).build(build_command_args.package) {
                println!("error has occured!")
            }
        }
        _ => todo!(),
    }
}
