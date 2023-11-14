use crate::util::errors::LingoError;
use crate::util::execute_command_to_build_result;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Npm;

pub struct TypeScriptToolCommands {
    pub binary_name: &'static str,
    pub install_command: &'static str,
    pub release_build_argument: &'static str,
}

use crate::backends::{
    BatchBackend, BatchBuildResults, BuildCommandOptions, BuildProfile, CommandSpec,
};

pub fn do_typescript_build(
    results: &mut BatchBuildResults,
    options: &BuildCommandOptions,
    commands: TypeScriptToolCommands,
) {
    results.keep_going(options.keep_going);
    super::lfc::LFC::do_parallel_lfc_codegen(options, results, false);
    if !options.compile_target_code {
        return;
    }

    let extract_name = |path: &PathBuf| -> Result<String, Box<dyn Error + Send + Sync>> {
        match Path::new(&path).file_stem() {
            Some(value) => value
                .to_str()
                .map(String::from)
                .ok_or(Box::new(LingoError::InvalidMainReactor)),
            None => Err(Box::new(LingoError::InvalidMainReactor)),
        }
    };

    let extract_location = |main_reactor: &PathBuf,
                            root_path: &PathBuf|
     -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        let output_dir = main_reactor.strip_prefix(root_path)?;

        let src_index = output_dir
            .iter()
            .map(|x| x.to_os_string())
            .position(|element| element == *"src")
            .ok_or(LingoError::InvalidMainReactor)?;

        let mut path_copy: Vec<OsString> = output_dir.iter().map(|x| x.to_os_string()).collect();
        path_copy.drain(0..src_index + 1);

        let mut new_path_buf: PathBuf = PathBuf::new();

        for element in path_copy {
            new_path_buf.push(element);
        }

        new_path_buf.set_extension("");
        Ok(new_path_buf)
    };

    results
        .map(|app| {
            let src_postfix = extract_location(&app.main_reactor, &app.root_path)?;
            let path = app.output_root.join("src-gen").join(src_postfix);

            let mut npm_install = Command::new(commands.binary_name);
            npm_install.current_dir(path);
            npm_install.arg(commands.install_command);
            if options.profile == BuildProfile::Release {
                npm_install.arg(commands.release_build_argument);
            }

            execute_command_to_build_result(npm_install)?;
            Ok(())
        })
        .map(|app| {
            let src_postfix = extract_location(&app.main_reactor, &app.root_path)?; // path after src
            let path = app.output_root.join("src-gen").join(src_postfix);

            let mut npm_build = Command::new(commands.binary_name);
            npm_build.current_dir(path);
            npm_build.arg("run");
            npm_build.arg("build");

            if options.profile == BuildProfile::Release {
                npm_build.arg(commands.release_build_argument);
            }

            execute_command_to_build_result(npm_build)?;

            Ok(())
        })
        .map(|app| {
            fs::create_dir_all(app.output_root.join("bin"))?;
            let file_name = extract_name(&app.main_reactor)?;
            let src_postfix = extract_location(&app.main_reactor, &app.root_path)?; // path after src

            let path = app
                .output_root
                .join("src-gen")
                .join(src_postfix)
                .join("dist")
                .join(file_name + ".js");

            // cleanup: rename executable to match the app name
            fs::rename(path, app.executable_path())?;
            Ok(())
        });
}

impl BatchBackend for Npm {
    fn execute_command(&mut self, command: &CommandSpec, results: &mut BatchBuildResults) {
        match command {
            CommandSpec::Build(options) => do_typescript_build(
                results,
                options,
                TypeScriptToolCommands {
                    binary_name: "npm",
                    install_command: "install",
                    release_build_argument: "--production",
                },
            ),
            CommandSpec::Clean => {
                results.par_map(|app| {
                    crate::util::default_build_clean(&app.output_root)?;
                    crate::util::delete_subdirs(&app.output_root, &["node_modules", "dist"])?;
                    Ok(())
                });
            }
            _ => todo!(),
        }
    }
}
