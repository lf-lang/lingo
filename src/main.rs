use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;
use std::{env, io};

use clap::Parser;

use args::{BuildArgs, Command as ConsoleCommand, CommandLineArgs};
use package::App;

use crate::backends::{BatchLingoCommand, BuildCommandOptions, CommandSpec, LingoCommandCtx};
use crate::package::{Config, ConfigFile};
use crate::util::errors::BuildResult;

pub mod args;
pub mod backends;
pub mod interface;
pub mod lfc;
pub mod package;
pub(crate) mod util;

fn build(args: &BuildArgs, config: &Config) -> BuildResult {
    run_command(
        CommandSpec::Build(BuildCommandOptions {
            profile: args.build_profile(),
            compile_target_code: !args.no_compile,
        }),
        config,
        args.keep_going,
    )
}

fn run_command(task: CommandSpec, config: &Config, fail_at_end: bool) -> BuildResult {
    let command = BatchLingoCommand {
        apps: config.apps.iter().collect(),
        task,
    };
    let mut ctx = LingoCommandCtx::new(fail_at_end);
    backends::execute_command(command, &mut ctx)
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
            println!("{}", errs);
        }
    }
}

fn execute_command(config: Option<Config>, command: ConsoleCommand) -> BuildResult {
    match (config, command) {
        (_, ConsoleCommand::Init(init_config)) => {
            let initial_config = ConfigFile::new_for_init_task(init_config).map_err(Box::new)?;
            initial_config.write(Path::new("./Lingo.toml"));
            initial_config.setup_example();
            Ok(())
        }
        (None, _) => Err(Box::new(io::Error::new(
            ErrorKind::NotFound,
            "Error: Missing Lingo.toml file",
        ))),
        (Some(mut config), ConsoleCommand::Build(build_command_args)) => {
            config.filter_apps(&build_command_args.apps);
            println!("Building ...");
            build(&build_command_args, &config)
        }
        (Some(mut config), ConsoleCommand::Run(build_command_args)) => {
            config.filter_apps(&build_command_args.apps);
            build(&build_command_args, &config)?;
            for app in config.apps {
                let mut command = Command::new(app.executable_path());
                util::command_line::run_and_capture(&mut command)
                    .map(|_| ())
                    .map_err(Box::new)?
            }
            Ok(()) // todo
        }
        (Some(config), ConsoleCommand::Clean) => run_command(CommandSpec::Clean, &config, true),
        _ => todo!(),
    }
}
