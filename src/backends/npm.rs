use crate::util::execute_command_to_build_result;
use std::fs;
use std::path::Path;
use std::process::Command;
use which::which;

pub struct Npm;

use crate::backends::{
    BatchBackend, BatchBuildResults, BuildCommandOptions, BuildProfile, BuildResult, CommandSpec,
};

fn do_npm_build(results: &mut BatchBuildResults, options: &BuildCommandOptions) {
    super::lfc::LFC::do_parallel_lfc_codegen(options, results, false);
    if !options.compile_target_code {
        return;
    }
    results
        .map(|app| {
            // check if pnpm is available
            let mut cmd = "npm";
            let mut prod: &str = "--production";
            if which("pnpm").is_ok() {
                cmd = "pnpm";
                prod = "--prod";
            } 
            let mut npm_install = Command::new(cmd);
            npm_install.arg("install");
            if options.profile == BuildProfile::Release {
                npm_install.arg(prod);
            }
            npm_install.current_dir(self.lfc.out.display().to_string());
            execute_command_to_build_result(npm_install)
        })
        .map(|app| {
            let mut npm_build = Command::new(cmd);
            npm_build.arg("run");
            npm_build.arg("build");
            let reactor_path = Path::new("./node_modules/@lf_lang/reactor-ts");
            npm_build.current_dir(reactor_path.display().to_string());
            execute_command_to_build_result(npm_build)
        });
}

impl BatchBackend for Npm {
    fn execute_command(&mut self, command: &CommandSpec, results: &mut BatchBuildResults) {
        match command {
            CommandSpec::Build(options) => do_npm_build(results, options),
            CommandSpec::Clean => {
                results.par_map(|app| {
                    crate::util::default_build_clean(&app.output_root)?;
                    fs::remove_dir_all("./node_modules").is_ok()
                        && fs::remove_dir_all("./dist").is_ok();
                    Ok(())
                });
            }
            _ => todo!(),
        }
    }
}

