use crate::backends::npm::{do_typescript_build, TypeScriptToolCommands};

pub struct Pnpm;

use crate::backends::{BatchBackend, BatchBuildResults, CommandSpec};

impl BatchBackend for Pnpm {
    fn execute_command(&mut self, command: &CommandSpec, results: &mut BatchBuildResults) {
        match command {
            CommandSpec::Build(options) => do_typescript_build(
                results,
                options,
                TypeScriptToolCommands {
                    binary_name: "pnpm",
                    install_command: "install",
                    release_build_argument: "--prod",
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
