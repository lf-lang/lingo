use crate::util::execute_command_to_build_result;
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
    
    results
        .map(|app| {
            let mut npm_install = Command::new("pnpm");
            npm_install.arg("install");
            if options.profile == BuildProfile::Release {
                npm_install.arg("--production");
            }
            npm_install.current_dir(&app.output_root);
            execute_command_to_build_result(npm_install)
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
