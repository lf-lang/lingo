use std::fs;
use std::process::Command;

use crate::backends::{BatchBackend, BatchLingoCommand, BuildCommandOptions, CommandSpec};
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
    pub fn do_parallel_lfc_codegen(
        task: &BuildCommandOptions,
        command: &BatchLingoCommand,
    ) -> BuildResult {
        let BuildCommandOptions {
            compile_target_code,
            lfc_exec_path,
            ..
        } = task;
        use rayon::prelude::*;
        command
            .apps
            .par_iter()
            .map(|&app| {
                let mut lfc_command = Command::new(lfc_exec_path);
                lfc_command.arg("-o");
                lfc_command.arg(app.src_gen_dir());
                lfc_command.arg(&app.main_reactor);
                if !compile_target_code {
                    lfc_command.arg("--no-compile");
                }

                LFC::wrap_command_execution(lfc_command)
            })
            .reduce(|| Ok(()), crate::util::errors::merge)
    }
}

impl BatchBackend for LFC {
    fn execute_command(&mut self, command: BatchLingoCommand) -> BuildResult {
        match &command.task {
            CommandSpec::Build(options) => LFC::do_parallel_lfc_codegen(options, &command),
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
