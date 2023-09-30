use crate::util::errors::LingoError;
use crate::util::execute_command_to_build_result;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Pnpm;

use crate::backends::{
    BatchBackend, BatchBuildResults, BuildCommandOptions, BuildProfile, CommandSpec,
};

fn do_pnpm_build(results: &mut BatchBuildResults, options: &BuildCommandOptions) {
    super::lfc::LFC::do_parallel_lfc_codegen(options, results, false);
    if !options.compile_target_code {
        return;
    }

    let extract_name = |path: &PathBuf| -> Result<String, Box<dyn Error + Send + Sync>> {
        match Path::new(&path).file_stem() {
            Some(value) => value
                .to_str()
                .map(|x| String::from(x))
                .ok_or(Box::new(LingoError::InvalidMainReactor)),
            None => Err(Box::new(LingoError::InvalidMainReactor)),
        }
    };

    results
        .map(|app| {
            let file_name = extract_name(&app.main_reactor)?;
            let path = app.output_root.join("src-gen").join(file_name);

            let mut npm_install = Command::new("pnpm");
            npm_install.current_dir(path);
            npm_install.arg("install");
            if options.profile == BuildProfile::Release {
                npm_install.arg("--prod"); // different to npm
            }
            execute_command_to_build_result(npm_install)?;
            Ok(())
        })
        .map(|app| {
            let file_name = extract_name(&app.main_reactor)?;
            let path = app.output_root.join("src-gen").join(file_name);

            let mut npm_build = Command::new("pnpm");
            npm_build.current_dir(path);
            npm_build.arg("run");
            npm_build.arg("build");

            if options.profile == BuildProfile::Release {
                npm_build.arg("--production");
            }
            execute_command_to_build_result(npm_build)?;

            Ok(())
        })
        .map(|app| {
            fs::create_dir_all(&app.output_root.join("bin"))?;

            let file_name = extract_name(&app.main_reactor)?;
            let path = app
                .output_root
                .join("src-gen")
                .join(&file_name)
                .join("dist")
                .join(file_name + ".js");

            // cleanup: rename executable to match the app name
            fs::rename(path, app.executable_path())?;
            Ok(())
        });
}

impl BatchBackend for Pnpm {
    fn execute_command(&mut self, command: &CommandSpec, results: &mut BatchBuildResults) {
        match command {
            CommandSpec::Build(options) => do_pnpm_build(results, options),
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
