use crate::util::execute_command_to_build_result;
use std::process::Command;
use which::which;

pub struct Npm;

use crate::backends::{
    BatchBackend, BatchBuildResults, BuildCommandOptions, BuildProfile, CommandSpec,
};

fn do_npm_build(results: &mut BatchBuildResults, options: &BuildCommandOptions) {
    super::lfc::LFC::do_parallel_lfc_codegen(options, results, false);
    if !options.compile_target_code {
        return;
    }
    // check if pnpm is available
    let mut cmd = "npm";
    let mut prod: &str = "--production";
    if which("pnpm").is_ok() {
        cmd = "pnpm";
        prod = "--prod";
    }
    results
        .map(|app| {
            let mut npm_install = Command::new(cmd);
            npm_install.arg("install");
            if options.profile == BuildProfile::Release {
                npm_install.arg(prod);
            }
            npm_install.current_dir(&app.output_root);
            execute_command_to_build_result(npm_install)
        })
        .map(|app| {
            if options.profile == BuildProfile::Debug && cmd == "npm" {
                // pnpm does this automatically
                let mut npm_build = Command::new("npm");
                npm_build.arg("run");
                npm_build.arg("build");
                let reactor_path = app.output_root.join("node_modules/@lf_lang/reactor-ts");
                npm_build.current_dir(reactor_path.display().to_string());
                execute_command_to_build_result(npm_build)
            } else {
                Ok(())
            }
        });
}

impl BatchBackend for Npm {
    fn execute_command(&mut self, command: &CommandSpec, results: &mut BatchBuildResults) {
        match command {
            CommandSpec::Build(options) => do_npm_build(results, options),
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
