
use std::process::Command;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use super::{run_and_capture};

pub fn debian_install() {
    let mut command = Command::new("sudo");
    command.arg("apt");
    command.arg("install");
    command.arg("nix");
    run_and_capture(&mut command).ok();
}

pub fn arch_install() {
    let mut command = Command::new("sudo");
    command.arg("pacman");
    command.arg("-S");
    command.arg("nix");
    run_and_capture(&mut command).ok();
}

pub fn default_install() {
    let mut command = Command::new("sudo");
    command.arg("sh");
    command.arg("<(curl -L https://nixos.org/nix/install)");
    command.arg("--daemon");

    run_and_capture(&mut command).ok();
}

pub fn edit_config() {
    let filepath = "/etc/nix/nix.conf";
    let features = "experimental-features = nix-command flakes";
    let mut fh = OpenOptions::new()
        .read(true)
        .write(true)
        .open(filepath)
        .expect("could not open nix.conf");

    let mut data: String = String::new();
    fh.read_to_string(&mut data).ok();

    if data.contains(&features) {
        fh.write_all(features.as_bytes()).expect("could not write nix.conf");
    }
}
