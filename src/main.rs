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

fn build(args: &BuildArgs, config: &package::Config) {
    if let Some(backend) = backends::select_backend("lfc", &config.package) {
        if !backend.build(args.package.clone()) {
            println!("error has occured!")
        }
    }
}

fn main() {
    const PACKAGE_FILE: &str = "./Barrel.toml";

    let args = CommandLineArgs::parse();
    let wrapped_config = package::Config::from(Path::new(PACKAGE_FILE));

    // we match on a tuple here
    match (wrapped_config, args.command) {
        (_, ConsoleCommand::Init) => {
            let initial_config = package::Config::new();
            initial_config.write(Path::new(PACKAGE_FILE));
            package::Config::setup_example();
        }
        (Some(config), ConsoleCommand::Build(build_command_args)) => {
            build(&build_command_args, &config)
        }
        (Some(config), ConsoleCommand::Run(build_command_args)) => {
            build(&build_command_args, &config);

            let execute_binary = |binary: &String| -> bool {
                let mut command = Command::new(format!("./bin/{}", binary));
                util::command_line::run_and_capture(&mut command).is_ok()
            };

            util::invoke_on_selected(
                build_command_args.package,
                config.package.main_reactor,
                execute_binary,
            );
        }
        (Some(config), ConsoleCommand::Clean) => {
            if let Some(backend) = backends::select_backend("lfc", &config.package) {
                if !backend.clean() {
                    println!("error has occured!")
                }
            }
        }
        _ => todo!(),
    }
}
