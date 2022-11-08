extern crate termion;

pub mod analyzer;
pub mod args;
pub mod backends;
pub mod interface;
pub mod package;
pub mod util;

use args::{BuildArgs, Command as ConsoleCommand, CommandLineArgs};

use clap::Parser;
use std::path::Path;
use std::process::Command;

fn build(args: &BuildArgs) {
    let config = package::Config::from(Path::new("./Barrel.toml"));

    if config.is_none() {
        return;
    }

    if !backends::select_backend("lfc", config.unwrap().package).build(args.package.clone()) {
        println!("error has occured!")
    }
}

fn main() {
    let args = CommandLineArgs::parse();
    let config = package::Config::from(Path::new("./Barrel.toml"));

    match args.command {
        ConsoleCommand::Init => {
            let initial_config = package::Config::new();
            initial_config.write(Path::new("./Barrel.toml"));
            package::Config::setup_example();
        }
        ConsoleCommand::Build(build_command_args) => build(&build_command_args),
        ConsoleCommand::Run(build_command_args) => {
            if config.is_none() {
                return;
            }

            build(&build_command_args);

            let execute_binary = |binary: &String| -> bool {
                let mut command = Command::new(format!("./bin/{}", binary));
                util::command_line::run_and_capture(&mut command).is_ok()
            };

            util::invoke_on_selected(
                build_command_args.package,
                config.unwrap().package.main_reactor,
                execute_binary,
            );
        }
        ConsoleCommand::Clean => {
            let config = package::Config::from(Path::new("./Barrel.toml"));

            if config.is_none() {
                return;
            }

            if !backends::select_backend("lfc", config.unwrap().package).clean() {
                println!("error has occured!")
            }
        }
        _ => todo!(),
    }
}
