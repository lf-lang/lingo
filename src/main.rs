extern crate termion;

pub mod analyzer;
pub mod args;
pub mod backends;
pub mod interface;
pub mod package;
pub mod util;

use args::{BuildArgs, Command as ConsoleCommand, CommandLineArgs};
use package::App;

use clap::Parser;
use std::path::Path;
use std::process::Command;

fn build(args: &BuildArgs, config: &package::Config) {
    let build_target = |app: &App| -> bool {
        if let Some(backend) = backends::select_backend("lfc", &app) {
            if !backend.build(args) {
                println!("error has occured!");
                return false;
            }
        }
        true
    };

    util::invoke_on_selected(args.target.clone(), config.app.clone(), build_target);
}

fn main() {
    const PACKAGE_FILE: &str = "./Barrel.toml";

    let args = CommandLineArgs::parse();

    //let test_wrapped_config = package::ConfigFile::from(Path::new(PACKAGE_FILE)).unwrap().to_config();

    //return;
    let wrapped_config = package::ConfigFile::from(Path::new(PACKAGE_FILE));

    // we match on a tuple here
    match (wrapped_config, args.command) {
        (_, ConsoleCommand::Init) => {
            let initial_config = package::ConfigFile::new();
            initial_config.write(Path::new(PACKAGE_FILE));
            package::ConfigFile::setup_example();
        }
        (Some(config), ConsoleCommand::Build(build_command_args)) => {
            build(&build_command_args, &config.to_config())
        }
        (Some(file_config), ConsoleCommand::Run(build_command_args)) => {
            let config = file_config.to_config();

            build(&build_command_args, &config);

            let execute_binary = |app: &App| -> bool {
                let mut command = Command::new(format!("./bin/{}", app.name));
                util::command_line::run_and_capture(&mut command).is_ok()
            };

            util::invoke_on_selected(build_command_args.target, config.app, execute_binary);
        }
        (Some(_config), ConsoleCommand::Clean) => {
            /*if let Some(backend) = backends::select_backend("lfc", &config.package) {
                if !backend.clean() {
                    println!("error has occured!")
                }
            }*/
        }
        _ => todo!(),
    }
}
