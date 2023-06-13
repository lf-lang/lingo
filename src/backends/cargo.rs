use crate::args::BuildArgs;
use crate::interface::Backend;
use crate::lfc::LFCProperties;
use crate::App;

use crate::util::command_line::run_and_capture;
use std::{env, io};
use std::fs;
use std::process::Command;

pub struct CargoBackend<'a> {
    app: &'a App,
    lfc: &'a LFCProperties,
}

impl<'a> Backend<'a> for CargoBackend<'a> {
    fn from_target(app: &'a App, lfc: &'a LFCProperties) -> Self {
        CargoBackend { app, lfc }
    }

    fn build(&self, config: &BuildArgs) -> io::Result<()> {
        fs::create_dir_all(format!("{}/build", self.lfc.out.display().to_string()))?;

        // Cargo command
        let mut cargo = Command::new("cargo");
        if config.release {
            cargo.arg("--release")
        }
        // todo custom profile
        println!("{:?}", self.lfc.properties);

        cargo.arg(format!("{}/src-gen", self.lfc.out.display()));
        cargo.arg(format!("-B {}/build", self.lfc.out.display()));
        cargo.current_dir(format!("{}/build", self.lfc.out.display().to_string()));
        run_and_capture(&mut cargo)?;

        // todo copy binaries

        Ok(())
    }

    fn update(&self) -> bool {
        true
    }

    fn clean(&self) -> bool {
        println!("removing build artifacts in {:?}", env::current_dir());
        // just removes all the lingua-franca build artifacts
        fs::remove_dir_all("./bin").is_ok()
            && fs::remove_dir_all("./include").is_ok()
            && fs::remove_dir_all("./src-gen").is_ok()
            && fs::remove_dir_all("./lib64").is_ok()
            && fs::remove_dir_all("./share").is_ok()
            && fs::remove_dir_all("./build").is_ok()
    }
}