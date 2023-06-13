use crate::args::BuildArgs;
use crate::interface::Backend;
use crate::lfc::LFCProperties;
use crate::App;

use crate::util::command_line::run_and_capture;
use std::{env, io};
use std::fs;
use std::process::Command;

pub struct Cmake<'a> {
    app: &'a App,
    lfc: &'a LFCProperties,
}

impl<'a> Backend<'a> for Cmake<'a> {
    fn from_target(app: &'a App, lfc: &'a LFCProperties) -> Self {
        Cmake { app, lfc }
    }

    fn build(&self, config: &BuildArgs) -> io::Result<()> {
        fs::create_dir_all(format!("{}/build", self.lfc.out.display().to_string()))?;

        // cmake generation
        let mut cmake_command = Command::new("cmake");
        cmake_command.arg(format!(
            "-DCMAKE_BUILD_TYPE={}",
            if config.release { "RELEASE" } else { "DEBUG" }
        ));
        cmake_command.arg(format!("-DCMAKE_INSTALL_PREFIX={}", self.lfc.out.display()));
        cmake_command.arg(format!("-DCMAKE_INSTALL_BINDIR=bin"));
        cmake_command.arg(format!("-DREACTOR_CPP_VALIDATE=ON"));
        cmake_command.arg(format!("-DREACTOR_CPP_TRACE=OFF"));
        cmake_command.arg(format!("-DREACTOR_CPP_LOG_LEVEL=3"));
        cmake_command.arg(format!(
            "-DLF_SRC_PKG_PATH={}",
            self.app.root_path.display()
        ));
        cmake_command.arg(format!("{}/src-gen", self.lfc.out.display().to_string()));
        cmake_command.arg(format!("-B {}/build", self.lfc.out.display().to_string()));
        cmake_command.current_dir(format!("{}/build", self.lfc.out.display().to_string()));
        run_and_capture(&mut cmake_command)?;

        // compiling
        let mut cmake_build_command = Command::new("cmake");
        cmake_build_command.current_dir(format!("{}/build", self.lfc.out.display().to_string()));
        cmake_build_command.arg("--build");
        cmake_build_command.arg("./");
        run_and_capture(&mut cmake_build_command)?;

        // installing
        let mut cmake_install_command = Command::new("cmake");
        cmake_install_command.current_dir(format!("{}/build", self.lfc.out.display().to_string()));
        cmake_install_command.arg("--install");
        cmake_install_command.arg("./");
        run_and_capture(&mut cmake_install_command)?;

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
