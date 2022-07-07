mod cli;
mod weaver;
mod wrapper;
mod analyzer;
mod install;

use cli::{Args, Command as CliCommand};
use weaver::{Config};
use wrapper::run_and_capture;
pub use analyzer::search;
use install::{debian_install, arch_install, edit_config};

use clap::Parser;
use git2::Repository;
use termion::color;
use os_version::OsVersion;

extern crate termion;

use std::path::Path;
use std::process::Command;
use std::io::Read;



fn generate_code() {
    let config = Config::from(Path::new("./Weaver.toml"));
    let mut command = Command::new("git");
    command.arg("add");
    command.arg("./nix-build/*");
    run_and_capture(&mut command).ok();

    config.write_nix_code();
}

fn build() {
    generate_code();
    let mut command = Command::new("nix");
    command.arg("build");
    command.arg("./nix-build");
    command.arg("-L");
    command.arg("--impure");
    run_and_capture(&mut command).ok();
}

fn run() {
    build();
    let mut command = Command::new("nix");
    command.arg("run");
    command.arg("./nix-build");
    command.arg("-L");
    run_and_capture(&mut command).ok();
}

fn main() {
    let args = Args::parse();
    match args.command {
        CliCommand::Init { } => {

            let _ = match Repository::init("./") {
                Ok(repo) => repo,
                Err(e) => panic!("failed to init: {}", e),
            };

            let initial_config = Config::new();
            initial_config.write(Path::new("./Weaver.toml"));
            Config::setup_example();
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
            run_and_capture(&mut command).ok();
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
            run_and_capture(&mut command).ok();
        }
        CliCommand::Run {} => {
            run();
        }
        CliCommand::Search { package } => {
            let mut command = Command::new("nix");
            command.arg("search");
            command.arg("./nix-build");
            command.arg(package);
            run_and_capture(&mut command).ok();
        }
        CliCommand::Clean {} => {
            std::fs::remove_dir_all(Path::new("./result")).ok();
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
            run_and_capture(&mut command).ok();

        }
        CliCommand::Publish {} => {
            let config = Config::from(Path::new("./Weaver.toml"));
            let mut command = Command::new("git");
            command.arg("add");
            command.arg("./nix-build/*");
            run_and_capture(&mut command).ok();
            //color::Fg(color::Red), color::Fg(color::White));

            println!("{} pkgs/{}/{}.nix\n{}\n{}\n{} pkgs/root.nix{}\n{}", 
                    color::Fg(color::Green), 
                    &config.package.language, 
                    &config.package.name,
                    color::Fg(color::White), 
                    config.publish_nix(),
                    color::Fg(color::Green),
                    color::Fg(color::White), 
                    config.root_nix_publish()
            );

        }
        CliCommand::Install {} => {
            let distro = os_version::detect();

            match distro.unwrap() {
                OsVersion::Linux(linux) => {
                    println!("Detected Linux: {} Version: {}, Version Name: {}", 
                             &linux.distro, 
                             &linux.version.unwrap(), 
                             &linux.version_name.unwrap()
                    );

                    match linux.distro.as_str() {
                        "nixos" => {

                        }
                        "ubuntu" => {
                            debian_install();
                            edit_config();
                        }
                        "debian" => {
                            debian_install();
                            edit_config();
                        }
                        "arch" => {
                            arch_install();
                            edit_config();
                        }
                        "manjaro" => {
                            arch_install();
                            edit_config();
                        }
                        &_ => todo!()
                    }
                }
                OsVersion::MacOS(macos) => {
                    println!("Detected MacOs Version: {}", &macos.version);

                }
                _ => {
                    println!("Unsupported Version");
                }

            }

        }
    };

}
