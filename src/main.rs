pub mod analyzer;
pub mod args;
pub mod backends;
pub mod interface;
pub mod package;
pub mod util;

use args::{BuildArgs, Command as ConsoleCommand, CommandLineArgs};
use package::App;

use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

fn build(args: &BuildArgs, config: &package::Config) {
    let build_target = |app: &App| -> bool {
        if let Some(backend) = backends::select_backend("lfc", app) {
            if !backend.build(args) {
                println!("error has occured!");
                return false;
            }
        }
        true
    };

    util::invoke_on_selected(args.target.clone(), config.apps.clone(), build_target);
}

fn main() {
    const PACKAGE_FILE: &str = "./";

    // finds Lingo.toml recurisvely inside the parent directories.
    let barrel_path = util::find_toml(&PathBuf::from(PACKAGE_FILE));

    // parses command line arguments
    let args = CommandLineArgs::parse();

    // tries to read barrel toml
    let wrapped_config = if barrel_path.is_none() {
        None
    } else {
        package::ConfigFile::from(&barrel_path.clone().unwrap())
    };

    // we match on a tuple here
    match (wrapped_config, args.command) {
        (_, ConsoleCommand::Init) => {
            let initial_config = package::ConfigFile::new();
            let toml_path = format!("{}/Barrel.toml", PACKAGE_FILE);
            initial_config.write(Path::new(&toml_path));
            package::ConfigFile::setup_example();
        }
        (Some(file_config), ConsoleCommand::Build(build_command_args)) => {
            let mut working_path = barrel_path.unwrap();
            working_path.pop();
            let config = file_config.to_config(working_path);

            build(&build_command_args, &config)
        }
        (Some(file_config), ConsoleCommand::Run(build_command_args)) => {
            let mut working_path = barrel_path.unwrap();
            working_path.pop();
            let config = file_config.to_config(working_path);

            build(&build_command_args, &config);

            let execute_binary = |app: &App| -> bool {
                let mut command = Command::new(format!("./bin/{}", app.name));
                util::command_line::run_and_capture(&mut command).is_ok()
            };

            util::invoke_on_selected(build_command_args.target, config.apps, execute_binary);
        }
        (Some(_config), ConsoleCommand::Clean) => todo!(),
        _ => todo!(),
    }
}
