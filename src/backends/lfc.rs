use crate::interface::Backend;
use crate::package::Package;

use crate::util;
use crate::util::command_line::run_and_capture;
use std::env;
use std::fs;
use std::process::Command;

pub struct LFC {
    package: Package,
}

impl Backend for LFC {
    fn from_package(package: &Package) -> Self {
        LFC { package: package.clone() }
    }

    fn build(&self, binary: Option<String>) -> bool {
        let reactor_copy = self.package.main_reactor.clone();

        let build_lambda = |main_reactor: &String| -> bool {
            println!("building main reactor: {}", &main_reactor);
            let mut command = Command::new("lfc");
            command.arg("--output");
            command.arg("./");
            command.arg(format!("./src/{}.lf", &main_reactor));
            run_and_capture(&mut command).is_ok()
        };

        util::invoke_on_selected(binary, reactor_copy, build_lambda)
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
