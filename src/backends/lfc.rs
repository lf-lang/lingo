use std::fs;
use std::process::Command;

use crate::backends::{BatchBackend, BatchLingoCommand, BuildCommandOptions, CommandSpec};
use crate::package::App;
use crate::util::command_line::run_and_capture;
use crate::util::errors::{BuildResult, LingoError};

pub struct LFC;

impl LFC {
    fn wrap_command_execution(mut command: Command) -> BuildResult {
        match run_and_capture(&mut command) {
            Err(e) => Err(Box::new(e)),
            Ok((status, _, _)) if !status.success() => {
                Err(Box::new(LingoError::CommandFailed(command, status)))
            }
            _ => Ok(()),
        }
    }
    // todo need to identify which apps have failed and which haven't
    pub fn do_parallel_lfc_codegen(
        task: &BuildCommandOptions,
        apps: &Vec<&App>,
    ) -> BuildResult {
        let BuildCommandOptions {
            compile_target_code,
            lfc_exec_path,
            ..
        } = task;

        // todo this doesn't work as gradle locks and restricts parallelism.
        //  LFC should support parallel builds directly, or I shouldn't use gradle?

        use rayon::prelude::*;
        apps
            .par_iter()
            .map(|&app| {
                fs::create_dir_all(&app.output_root)?;

                let mut lfc_command = Command::new(lfc_exec_path);
                lfc_command.arg("-o");
                lfc_command.arg(&app.output_root);
                lfc_command.arg(&app.main_reactor);
                if !compile_target_code {
                    lfc_command.arg("--no-compile");
                }

                LFC::wrap_command_execution(lfc_command)
            })
            .reduce_with(crate::util::errors::merge)
            .unwrap_or(Ok(()))
    }
}

impl BatchBackend for LFC {
    fn execute_command(&mut self, command: BatchLingoCommand) -> BuildResult {
        match &command.task {
            CommandSpec::Build(options) => LFC::do_parallel_lfc_codegen(options, &command.apps),
            CommandSpec::Update => Ok(()),
            CommandSpec::Clean => {
                for &app in &command.apps {
                    fs::remove_dir_all(app.src_gen_dir())?
                }
                Ok(())
            }
        }
    }
}
