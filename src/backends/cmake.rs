use crate::args::BuildArgs;
use crate::interface::{Backend, TargetInformation};

use crate::util::command_line::run_and_capture;
use std::env;
use std::fs;
use std::process::Command;

pub struct Cmake {
    target: TargetInformation,
}

impl Backend for Cmake {
    fn from_target(target: TargetInformation) -> Self {
        Cmake { target: target }
    }

    fn build(&self, _config: &BuildArgs) -> bool {
        let reactor_copy = self.target.app.main_reactor.clone();

        let build_lambda = |main_reactor: &String| -> bool {
            println!("building main reactor: {}", &main_reactor);
            let mut cmake_command = Command::new("cmake");
            cmake_command.arg("-DCMAKE_BUILD_TYPE=DEBUG"); // TODO: progagate from cli args
            cmake_command.arg(format!(
                "-DCMAKE_INSTALL_PREFIX={}",
                self.target.lfc.out.display()
            )); // TODO: I Think this is actually the root_dir
            cmake_command.arg(format!("-DCMAKE_INSTALL_BINDIR=bin"));
            cmake_command.arg(format!("-DREACTOR_CPP_VALIDATE=ON"));
            cmake_command.arg(format!("-DREACTOR_CPP_TRACE=OFF"));
            cmake_command.arg(format!("-DREACTOR_CPP_LOG_LEVEL=3"));
            cmake_command.arg(format!(
                "-DLF_SRC_PKG_PATH={}",
                self.target.app.root_path.display()
            ));
            cmake_command.arg(self.target.lfc.out.display().to_string());
            cmake_command.arg(format!(
                "-B {}/build",
                self.target.lfc.out.display().to_string()
            ));
            run_and_capture(&mut cmake_command).is_ok();

            let mut cmake_build_command = Command::new("cmake");
            cmake_build_command.arg("--build");
            cmake_build_command.arg("--target");
            run_and_capture(&mut cmake_build_command).is_ok()
        };

        if !build_lambda(&reactor_copy) {
            println!("calling cmake returned an error can cmake be found in $PATH ?");
            return false;
        }

        true
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
