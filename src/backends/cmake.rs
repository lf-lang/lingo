use std::fs;

use std::process::Command;




use crate::util::command_line::run_and_capture;
use crate::App;

use crate::backends::{BatchBackend, BatchLingoCommand, BuildCommandOptions, BuildProfile, BuildResult, CommandSpec, LingoCommandCtx};
use std::borrow::Borrow;

pub struct Cmake;

fn build_single_app(app: &App, options: &BuildCommandOptions) -> BuildResult {
    let build_dir = app.output_root.join("build");
    fs::create_dir_all(&build_dir)?;

    // cmake generation
    let mut cmake_command = Command::new("cmake");
    cmake_command.arg(format!(
        "-DCMAKE_BUILD_TYPE={}",
        if options.profile == BuildProfile::Release { "RELEASE" } else { "DEBUG" }
    ));
    cmake_command.arg(format!("-DCMAKE_INSTALL_PREFIX={}", app.output_root.display()));
    cmake_command.arg(format!("-DCMAKE_INSTALL_BINDIR=bin"));
    cmake_command.arg(format!("-DREACTOR_CPP_VALIDATE=ON"));
    cmake_command.arg(format!("-DREACTOR_CPP_TRACE=OFF"));
    cmake_command.arg(format!("-DREACTOR_CPP_LOG_LEVEL=3"));
    cmake_command.arg(format!("-DLF_SRC_PKG_PATH={}", app.root_path.display()));
    cmake_command.arg(app.src_gen_dir());
    cmake_command.arg(format!("-B {}", build_dir.display()));
    cmake_command.current_dir(&build_dir);
    run_and_capture(&mut cmake_command)?;

    // compiling
    let mut cmake_build_command = Command::new("cmake");
    cmake_build_command.current_dir(&build_dir);
    cmake_build_command.args(["--build", "."]);
    run_and_capture(&mut cmake_build_command)?;

    // installing
    let mut cmake_install_command = Command::new("cmake");
    cmake_install_command.current_dir(&build_dir);
    cmake_build_command.args(["--install", "."]);
    run_and_capture(&mut cmake_install_command)?;
    Ok(())
}

impl BatchBackend for Cmake {
    fn execute_command(&mut self, command: BatchLingoCommand, ctx: &mut LingoCommandCtx) -> BuildResult {
        match &command.task {
            CommandSpec::Build(options) => {
                for app in &command.apps {
                    match build_single_app(app, options) {
                        Err(error) => ctx.notify_failed(app, error.borrow())?,
                        _ => {}
                    }
                }
                Ok(())
            }
            _ => {
                Ok(())
            }
        }
    }
}
