use std::fs;
use std::io::Write;

use std::process::Command;

use crate::package::App;
use crate::util::execute_command_to_build_result;

use crate::backends::{
    BatchBackend, BatchBuildResults, BuildCommandOptions, BuildProfile, BuildResult, CommandSpec,
};

pub struct CmakeCpp;

fn gen_cmake_files(app: &App, options: &BuildCommandOptions) -> BuildResult {
    let build_dir = app.output_root.join("build");
    fs::create_dir_all(&build_dir)?;

    // location of the cmake file
    let app_build_folder = app.src_gen_dir().join(&app.main_reactor_name);
    let cmake_file = app_build_folder.clone().join("CMakeLists.txt");

    // create potential files that come from the target properties
    app.properties.write_artifacts(&app_build_folder)?;

    // we need to modify the cmake file here to include our generated cmake files
    let src_gen_dir = app.src_gen_dir();

    // read file and append cmake include to generated cmake file
    let mut content = fs::read_to_string(&cmake_file)?;
    let include_statement = format!(
        "\ninclude({}/aggregated_cmake_include.cmake)",
        app_build_folder.display()
    );
    content += &*include_statement;

    // overwrite cmake file
    let mut f = fs::OpenOptions::new().write(true).open(&cmake_file)?;
    f.write_all(content.as_ref())?;
    f.flush()?;

    // cmake args
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
    cmake.arg("-DCMAKE_INSTALL_BINDIR=bin");
    cmake.arg("-DREACTOR_CPP_VALIDATE=ON");
    cmake.arg("-DREACTOR_CPP_TRACE=OFF");
    cmake.arg("-DREACTOR_CPP_LOG_LEVEL=3");
    cmake.arg(format!(
        "-DLF_SRC_PKG_PATH={}",
        app.src_dir_path()
            .expect("not a valid main reactor path")
            .display()
    ));
    cmake.arg(src_gen_dir);
    cmake.arg(format!("-B {}", build_dir.display()));
    cmake.current_dir(&build_dir);

    execute_command_to_build_result(cmake)
}

fn do_cmake_build(results: &mut BatchBuildResults, options: &BuildCommandOptions) {
    // configure keep going parameter
    results.keep_going(options.keep_going);

    // start code-generation
    super::lfc::LFC::do_parallel_lfc_codegen(options, results, false);

    // stop process if the user request code-generation only
    if !options.compile_target_code {
        return;
    }

    results
        // generate all CMake files ahead of time
        .map(|app| gen_cmake_files(app, options))
        // Run cmake to build everything.
        .gather(|apps| {
            let build_dir = apps[0].output_root.join("build");

            // compile everything
            let mut cmake = Command::new("cmake");
            cmake.current_dir(&build_dir);
            cmake.args(["--build", "."]);
            for app in apps {
                // add one target arg for each app
                let name = app.main_reactor.file_stem().unwrap();
                cmake.arg("--target");
                cmake.arg(name);
            }
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
            fs::rename(bin_dir.join(cmake_binary_name), app.executable_path())?;
            Ok(())
        });
}

impl BatchBackend for CmakeCpp {
    fn execute_command(&mut self, command: &CommandSpec, results: &mut BatchBuildResults) {
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
