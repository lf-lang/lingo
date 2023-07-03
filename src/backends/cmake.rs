use std::ffi::OsString;
use std::fs;

use std::process::Command;

use crate::util::execute_command_to_build_result;
use crate::App;

use crate::backends::{
    BatchBackend, BatchBuildResults, BatchLingoCommand, BuildCommandOptions, BuildProfile,
    BuildResult, CommandSpec,
};

pub struct Cmake;

fn gen_cmake_files(app: &App, options: &BuildCommandOptions) -> BuildResult {
    let build_dir = app.output_root.join("build");
    fs::create_dir_all(&build_dir)?;

    let mut cmake = Command::new("cmake");
    cmake.arg(format!(
        "-DCMAKE_BUILD_TYPE={}",
        if options.profile == BuildProfile::Release {
            "RELEASE"
        } else {
            "DEBUG"
        }
    ));
    cmake.arg(format!(
        "-DCMAKE_INSTALL_PREFIX={}",
        app.output_root.display()
    ));
    cmake.arg(format!("-DCMAKE_INSTALL_BINDIR=bin"));
    cmake.arg(format!("-DREACTOR_CPP_VALIDATE=ON"));
    cmake.arg(format!("-DREACTOR_CPP_TRACE=OFF"));
    cmake.arg(format!("-DREACTOR_CPP_LOG_LEVEL=3"));
    cmake.arg(format!("-DLF_SRC_PKG_PATH={}", app.root_path.display()));
    cmake.arg(app.src_gen_dir());
    cmake.arg(format!("-B {}", build_dir.display()));
    cmake.current_dir(&build_dir);

    execute_command_to_build_result(cmake)
}

fn do_cmake_build<'a>(
    results: &mut BatchBuildResults<'a>,
    options: &BuildCommandOptions,
) {
    super::lfc::LFC::do_parallel_lfc_codegen(options, results, false);
    if !options.compile_target_code {
        return;
    }
    results
        // generate all CMake files ahead of time
        .map(|app| gen_cmake_files(app, options))
        // Run cmake to build everything.
        .gather(|apps| {
            let build_dir = apps[0].output_root.join("build");
            let target_names: OsString = apps
                .iter()
                .map(|&app| app.main_reactor.file_stem().unwrap())
                .collect::<Vec<_>>()
                .join(&OsString::from(","));

            // compile everything
            let mut cmake = Command::new("cmake");
            cmake.current_dir(&build_dir);
            cmake.args(["--build", "."]);
            cmake.arg("--target");
            cmake.arg(target_names);
            // note: by parsing CMake stderr we would know which specific targets have failed.
            execute_command_to_build_result(cmake)
        })
        .map(|app| {
            let build_dir = app.output_root.join("build");
            // installing
            let mut cmake = Command::new("cmake");
            cmake.current_dir(&build_dir);
            cmake.args(["--install", "."]);
            execute_command_to_build_result(cmake)
        })
        .map(|app| {
            let cmake_binary_name = app.main_reactor.file_stem().unwrap();
            // cleanup: rename executable to match the app name
            let bin_dir = app.output_root.join("bin");
            fs::rename(bin_dir.join(cmake_binary_name), bin_dir.join(&app.name))?;
            Ok(())
        });
}

impl BatchBackend for Cmake {
    fn execute_command<'a>(&mut self, command: &CommandSpec, results: &mut BatchBuildResults<'a>) {
        match command {
            CommandSpec::Build(options) => do_cmake_build(results, options),
            CommandSpec::Clean => {
                results.par_map(|app| {
                    crate::util::default_build_clean(&app.output_root)?;
                    Ok(())
                });
            }
            _ => todo!(),
        }
    }
}
