use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;
use std::{env, io};

use clap::Parser;

use args::{BuildArgs, Command as ConsoleCommand, CommandLineArgs, Platform};
use package::App;

use crate::lfc::LFCProperties;
use crate::package::{Config, ConfigFile};

pub mod args;
pub mod backends;
pub mod interface;
pub mod lfc;
pub mod package;
pub(crate) mod util;

fn build(args: &BuildArgs, config: &Config) -> Result<(), Vec<io::Error>> {
    util::invoke_on_selected(&args.apps, &config.apps, |app: &App| {
        // TODO: Support using lingo as a thin wrapper around west
        if app.platform == Platform::Zephyr {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "Error: Use `west lf-build` to build and run Zephyr programs.",
            ));
        }

        // TODO remove LFCProperties?
        let lfc_props = LFCProperties::new(
            app.main_reactor.clone(),
            app.root_path.clone(),
            app.properties.clone(),
        );

        let lfc_exec = util::find_lfc_exec(args)?;
        lfc::invoke_code_generator(&lfc_exec, &lfc_props, app)?;

        backends::run_build(
            args.build_system.unwrap_or(args::BuildSystem::LFC),
            app,
            &lfc_props,
            &args,
        )
    })
}

fn main() {
    // parses command line arguments
    let args = CommandLineArgs::parse();

    // Finds Lingo.toml recursively inside the parent directories.
    // If it exists the returned path is absolute.
    let lingo_path = util::find_toml(&env::current_dir().unwrap());

    // tries to read Lingo.toml
    let wrapped_config = lingo_path.as_ref().and_then(|path| {
        ConfigFile::from(path)
            .map_err(|err| println!("Error while reading Lingo.toml: {}", err))
            .ok()
            .map(|cf| cf.to_config(path.parent().unwrap()))
    });

    // we match on a tuple here
    let result = execute_command(wrapped_config, args.command);
    match result {
        Ok(_) => {}
        Err(errs) => {
            if errs.len() == 1 {
                println!("An error occurred: {}", errs[0]);
            } else {
                println!("{} errors occurred:", errs.len());
                for err in errs {
                    println!("{}", err)
                }
            }
        }
    }
}

fn execute_command(config: Option<Config>, command: ConsoleCommand) -> Result<(), Vec<io::Error>> {
    match (config, command) {
        (_, ConsoleCommand::Init(init_config)) => {
            let initial_config = ConfigFile::new_for_init_task(init_config).map_err(|e| vec![e])?;
            initial_config.write(Path::new("./Lingo.toml"));
            initial_config.setup_example();
            Ok(())
        }
        (None, _) => Err(vec![io::Error::new(
            ErrorKind::NotFound,
            "Error: Missing Lingo.toml file",
        )]),
        (Some(config), ConsoleCommand::Build(build_command_args)) => {
            println!("Building ...");
            build(&build_command_args, &config)
        }
        (Some(config), ConsoleCommand::Run(build_command_args)) => {
            build(&build_command_args, &config).and_then(|_| {
                // the run command
                util::invoke_on_selected(&build_command_args.apps, &config.apps, |app: &App| {
                    let mut command = Command::new(format!("./bin/{}", app.name));
                    util::command_line::run_and_capture(&mut command).map(|_| ())
                })
            })
        }
        (Some(_config), ConsoleCommand::Clean) => todo!(),
        _ => todo!(),
    }
}
