mod cli;
mod weaver;
mod wrapper;

use cli::{Args, Command as CliCommand};
use weaver::{Config};
use wrapper::run_and_capture;

use clap::Parser;
extern crate termion;

use std::path::Path;
use std::fs::{read_to_string, write};
use std::process::Command;
use std::io::Read;
use termion::{color, style};


fn generate_code() {
    let config = Config::from(Path::new("./Weaver.toml"));
    let mut command = Command::new("git");
    command.arg("add");
    command.arg("./nix-build/*");
    run_and_capture(&mut command);

    config.write_nix_code();
}

fn build() {
    generate_code();
    let mut command = Command::new("nix");
    command.arg("build");
    command.arg("./nix-build");
    command.arg("-L");
    run_and_capture(&mut command);
}

fn run() {
    build();
    let mut command = Command::new("nix");
    command.arg("run");
    command.arg("./nix-build");
    command.arg("-L");
    run_and_capture(&mut command);
}

fn main() {
    let args = Args::parse();
    match args.command {
        CliCommand::Init { } => {
            let initial_config = Config::new();
            initial_config.write(Path::new("./Weaver.toml"));
        }
        CliCommand::Generate {} => {
            generate_code();
        }
        CliCommand::Check {} => {
            let mut command = Command::new("nix");
            command.arg("flake");
            command.arg("check");
            command.arg("./nix-build");
            command.arg("-L");
            run_and_capture(&mut command);
        }
        CliCommand::Build {} => {
            build()
        }
        CliCommand::Update {} => {
            let mut command = Command::new("nix");
            command.arg("flake");
            command.arg("update");
            command.arg("./nix-build");
            command.arg("-L");
            run_and_capture(&mut command);
        }
        CliCommand::Run {} => {
            run();
        }
        CliCommand::Search { package } => {
            let mut command = Command::new("nix");
            command.arg("search");
            command.arg("./nix-build");
            command.arg(package);
            run_and_capture(&mut command);
        }
        CliCommand::Clean {} => {
            std::fs::remove_dir_all(Path::new("./result"));
        }
        CliCommand::CollectGarbage {} => {
            println!("{}Warning this will collect the garbage from the entire nix store. Do you want to continue ? [Y/n]{}",
                color::Fg(color::Red), color::Fg(color::White));

            let mut stdin = std::io::stdin();
            let mut buffer = [0;1];

            stdin.read_exact(&mut buffer).unwrap();

            if buffer[0] as char == 'n' || buffer[0] as char == 'N' {
                return;
            }

            let mut command = Command::new("nix-collect-garbage");
            command.arg("-d");
            run_and_capture(&mut command);

        }
    };

}
