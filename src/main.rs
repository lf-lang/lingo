use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;
use std::{env, io};

use clap::Parser;

use args::{BuildArgs, Command as ConsoleCommand, CommandLineArgs};
use package::App;
use crate::args::InitArgs;

use crate::backends::{BatchBuildResults, BatchLingoCommand, BuildCommandOptions, CommandSpec};
use crate::package::{Config, ConfigFile};
use crate::util::errors::{BuildResult, LingoError};

pub mod args;
pub mod backends;
pub mod lfc;
pub mod package;
pub(crate) mod util;

fn main() {
    // parses command line arguments
    let args = CommandLineArgs::parse();

    // Finds Lingo.toml recursively inside the parent directories.
    // If it exists the returned path is absolute.
    let lingo_path = util::find_toml(&env::current_dir().unwrap());

    // tries to read Lingo.toml
    let mut wrapped_config = lingo_path.as_ref().and_then(|path| {
        ConfigFile::from(path)
            .map_err(|err| println!("Error while reading Lingo.toml: {}", err))
            .ok()
            .map(|cf| cf.to_config(path.parent().unwrap()))
    });

    let result: BuildResult = validate(&mut wrapped_config, &args.command);
    if result.is_err() {
        print_res(result)
    }

    let result = execute_command(wrapped_config.as_ref(), args.command);

    match result {
        CommandResult::Batch(_) => {
            todo!()
        }
        CommandResult::Single(res) => {
            print_res(res)
        }
    }
}

fn print_res(result: BuildResult) {
    match result {
        Ok(_) => {}
        Err(errs) => {
            println!("{}", errs);
        }
    }
}

fn validate(config: &mut Option<Config>, command: &ConsoleCommand) -> BuildResult {
    match (config, command) {
        (Some(config), ConsoleCommand::Build(build))
        | (Some(config), ConsoleCommand::Run(build)) => {
            let unknown_names = build
                .apps
                .iter()
                .filter(|&name| !config.apps.iter().any(|app| &app.name == name))
                .map(|s| s.clone())
                .collect::<Vec<_>>();
            if !unknown_names.is_empty() {
                return Err(Box::new(LingoError::UnknownAppNames(unknown_names)));
            }
            config.filter_apps(&build.apps);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn execute_command(config: Option<&Config>, command: ConsoleCommand) -> CommandResult {
    match (config, command) {
        (_, ConsoleCommand::Init(init_config)) => {
            CommandResult::Single(do_init(init_config))
        }
        (None, _) => CommandResult::Single(Err(Box::new(io::Error::new(
            ErrorKind::NotFound,
            "Error: Missing Lingo.toml file",
        )))),
        (Some(config), ConsoleCommand::Build(build_command_args)) => {
            println!("Building ...");
            CommandResult::Batch(build(&build_command_args, &config))
        }
        (Some(config), ConsoleCommand::Run(build_command_args)) => {
            let res =
                build(&build_command_args, &config)
                    .map(|app| {
                        let mut command = Command::new(app.executable_path());
                        util::command_line::run_and_capture(&mut command)?;
                        Ok(())
                    });
            CommandResult::Batch(res)
        }
        (Some(config), ConsoleCommand::Clean) => CommandResult::Batch(run_command(CommandSpec::Clean, &config, true)),
        _ => todo!(),
    }
}

fn do_init(init_config: InitArgs) -> BuildResult {
    let initial_config = ConfigFile::new_for_init_task(init_config)?;
    initial_config.write(Path::new("./Lingo.toml"))?;
    initial_config.setup_example();
    Ok(())
}

fn build<'a>(args: &BuildArgs, config: &'a Config) -> BatchBuildResults<'a> {
    run_command(
        CommandSpec::Build(BuildCommandOptions {
            profile: args.build_profile(),
            compile_target_code: !args.no_compile,
            lfc_exec_path: util::find_lfc_exec(args).expect("TODO replace me"),
        }),
        config,
        args.keep_going,
    )
}

fn run_command(task: CommandSpec, config: &Config, _fail_at_end: bool) -> BatchBuildResults {
    let command = BatchLingoCommand {
        apps: config.apps.iter().collect(),
        task,
    };
    backends::execute_command(command)
}

enum CommandResult<'a> {
    Batch(BatchBuildResults<'a>),
    Single(BuildResult),
}
