use crate::interface::Backend;
use crate::package::Package;

use crate::util::command_line::run_and_capture;
use std::process::Command;

pub struct LFC {
    package: Package
}

impl Backend for LFC {
    fn from_package(package: Package) -> Self {
        LFC { package }
    }

    fn build(&self) -> bool {
        match fs::create_dir_all("./target") {
            Ok(_) => {
                let mut command = Command::new("lfc");
                command.arg("--output");
                command.arg("./target");
                command.arg(format!("./src/{}.lf", &self.package.main_reactor));
                run_and_capture(&mut command).is_ok()
            },
            Err(e) => {
                println!("cannot create target directory because of {:?}", e);
            }
        }
    }

    fn check(&self) -> bool {
        true
    }

    fn update(&self) -> bool {
        true
    }

    fn run(&self) -> bool {
        true
    }

    fn clean(&self) -> bool {
        true
    }
}


