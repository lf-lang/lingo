use regex::Regex;

use crate::interface::Backend;
use crate::package::Package;

use crate::util::command_line::run_and_capture;
use std::fs;
use std::process::Command;

pub struct LFC {
    package: Package,
}

impl Backend for LFC {
    fn from_package(package: Package) -> Self {
        LFC { package }
    }

    fn build(&self, binary: Option<String>) -> bool {
        let mut reactor_copy = self.package.main_reactor.clone();

        // a package can define multiple binaries the binary argument defines which one to
        // build if the argument is not specified all binaries are build
        if binary.is_some() {
            reactor_copy.retain(|x| match Regex::new(&binary.as_ref().unwrap()) {
                Ok(result) => result.is_match(x),
                Err(_) => false,
            });
        }

        let mut success = true;

        for main_reactor in reactor_copy {
            println!("building main reactor: {}", &main_reactor);
            let mut command = Command::new("lfc");
            command.arg("--output");
            command.arg("./target");
            command.arg(format!("./src/{}.lf", &main_reactor));
            success = success && run_and_capture(&mut command).is_ok();
        }

        success
    }

    fn update(&self) -> bool {
        true
    }

    fn clean(&self) -> bool {
        // just removes all the lingua-franca build artifacts
        fs::remove_dir_all("./bin").is_ok()
            && fs::remove_dir_all("./include").is_ok()
            && fs::remove_dir_all("./src-gen").is_ok()
            && fs::remove_dir_all("./lib64").is_ok()
            && fs::remove_dir_all("./share").is_ok()
            && fs::remove_dir_all("./build").is_ok()
    }
}


