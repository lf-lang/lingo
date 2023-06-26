
use std::process::Command;


use crate::backends::{BatchBackend, BatchLingoCommand, BuildCommandOptions, BuildResult, CommandSpec, LingoCommandCtx};



use crate::util::command_line::run_and_capture;
use crate::util::errors::LingoError;

pub struct LFC;

impl LFC {
    pub fn do_lfc_codegen(task: &BuildCommandOptions, command: &BatchLingoCommand, ctx: &mut LingoCommandCtx) -> BuildResult {
        let BuildCommandOptions { compile_target_code, .. } = task;
        for app in &command.apps { // todo this loop could be parallelized.
            let mut lfc_command = Command::new("lfc");
            lfc_command.arg("-o");
            lfc_command.arg(app.src_gen_dir());
            lfc_command.arg(&app.main_reactor);
            if !compile_target_code {
                lfc_command.arg("--no-compile");
            }

            match run_and_capture(&mut lfc_command) {
                Err(e) => {
                    ctx.notify_failed(app, &e)?;
                },
                Ok((status, _, _)) if !status.success() => {
                    ctx.notify_failed(app, &LingoError::CommandFailed(lfc_command))?;
                }
                _ => {
                    // ok
                }
            }
        }
        Ok(())
    }
}

impl BatchBackend for LFC {
    fn execute_command(&mut self, command: BatchLingoCommand, ctx: &mut LingoCommandCtx) -> BuildResult {
        match &command.task {
            CommandSpec::Build(options) => LFC::do_lfc_codegen(options, &command, ctx),
            CommandSpec::Update => {
                Ok(())
            }
            CommandSpec::Clean => {
                // todo
                Ok(())
            }
        }
    }
}
