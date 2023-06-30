pub mod args;
pub mod backends;
pub mod interface;
pub mod lfc;
pub mod package;
pub mod util;

use args::{BuildArgs, Command as ConsoleCommand, CommandLineArgs, Platform};
use package::App;

use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

fn build(args: &BuildArgs, config: &package::Config) {
    let build_target = |app: &App| -> bool {
        println!("--- Building app `{0}`", app.name);
        
        match app.platform {
            Platform::Zephyr => {
                println!("--- Skipping app `{0}` which is Zephyr-based. Use `west lf-build` to build instead", app.name);
                true
            }
            _ => {
                // path to the main reactor
                let mut main_reactor_path = app.root_path.clone();
                main_reactor_path.push(app.main_reactor.clone());

                let code_generator = lfc::CodeGenerator::new(
                    PathBuf::from(format!("{}/{}", app.root_path.display(), app.main_reactor)),
                    PathBuf::from(format!("{}/", app.root_path.display())),
                    args.lfc.clone().map(PathBuf::from),
                    app.properties.clone(),
                );

                if let Err(e) = code_generator.clone().generate_code(app) {
                    //TODO: optimize
                    eprintln!("--- Cannot generate code {:?}", e);
                    return false;
                }

                let backend = backends::select_backend(
                    &args.build_system.clone().unwrap_or(args::BuildSystem::LFC),
                    app,
                    &code_generator.properties,
                );

                if !backend.build(args) {
                    println!("--- An error has occured!");
                    return false;
                }
                true
            }
        }


    };
    util::invoke_on_selected(&args.apps, config.apps.clone(), build_target);
}

fn main() {
    const PACKAGE_FILE: &str = "./";

    // finds Lingo.toml recurisvely inside the parent directories.
    let lingo_path = util::find_toml(&PathBuf::from(PACKAGE_FILE));

    // parses command line arguments
    let args = CommandLineArgs::parse();

    // tries to read Lingo.toml
    let wrapped_config = if lingo_path.is_none() {
        None
    } else {
        package::ConfigFile::from(&lingo_path.clone().unwrap())
    };

    // we match on a tuple here
    match (wrapped_config, args.command) {
        (_, ConsoleCommand::Init(init_config)) => {
            let initial_config = package::ConfigFile::new(init_config);
            let toml_path = format!("{}/Lingo.toml", PACKAGE_FILE);
            initial_config.write(Path::new(&toml_path));
            initial_config.setup_example();
        }
        (Some(file_config), ConsoleCommand::Build(build_command_args)) => {
            let mut working_path = lingo_path.unwrap();
            working_path.pop();
            let config = file_config.to_config(working_path);
            build(&build_command_args, &config)
        }
        (Some(file_config), ConsoleCommand::Run(build_command_args)) => {
            let mut working_path = lingo_path.unwrap();
            working_path.pop();
            let config = file_config.to_config(working_path);

            build(&build_command_args, &config);
            let execute_binary = |app: &App| -> bool {
                let mut command = Command::new(format!("./bin/{}", app.name));
                // FIXME: When we execute the program we want to forward all outputs to stdout, stderr
                util::command_line::run_and_capture(&mut command).is_ok()
            };

            util::invoke_on_selected(&build_command_args.apps, config.apps, execute_binary);
        }
        (Some(_config), ConsoleCommand::Clean) => todo!(),
        _ => todo!(),
    }
}
