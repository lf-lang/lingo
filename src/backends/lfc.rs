use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde_derive::Serialize;

use crate::backends::{
    BatchBackend, BatchBuildResults, BatchLingoCommand, BuildCommandOptions, CommandSpec,
};
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

    pub fn do_parallel_lfc_codegen<'a, 'b>(
        options: &'b BuildCommandOptions,
        results: BatchBuildResults<'a>,
    ) -> BatchBuildResults<'a> {
        results.par_map(|app| LFC::do_lfc_codegen(app, options))
    }

    pub fn do_lfc_codegen(app: &App, options: &BuildCommandOptions) -> BuildResult {
        fs::create_dir_all(&app.output_root)?;

        let mut lfc_command = Command::new(&options.lfc_exec_path);
        lfc_command.arg(format!("--json={}", LfcJsonArgs::new(app)));
        if !options.compile_target_code {
            lfc_command.arg("--no-compile");
        }
        LFC::wrap_command_execution(lfc_command)
    }
}

impl BatchBackend for LFC {
    fn execute_command<'a>(&mut self, command: BatchLingoCommand<'a>) -> BatchBuildResults<'a> {
        match &command.task {
            CommandSpec::Build(options) => {
                LFC::do_parallel_lfc_codegen(options, command.new_results())
            }
            CommandSpec::Update => todo!(),
            CommandSpec::Clean => command.new_results().par_map(|app| {
                fs::remove_dir_all(app.src_gen_dir())?;
                Ok(())
            }),
        }
    }
}

/// Formats LFC arguments to JSON.
#[derive(Serialize, Clone)]
struct LfcJsonArgs<'a> {
    /// Path to the LF source file containing the main reactor.
    pub src: &'a Path,
    /// Path to the directory into which build artifacts like
    /// the src-gen and bin directory are generated.
    pub out: &'a Path,
    pub properties: &'a HashMap<String, serde_json::Value>,
}

impl<'a> LfcJsonArgs<'a> {
    pub fn new(app: &'a App) -> Self {
        Self {
            src: &app.main_reactor,
            out: &app.output_root,
            properties: &app.properties,
        }
    }
}

impl<'a> Display for LfcJsonArgs<'a> {
    /// convert lfc properties to string
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let string = serde_json::to_string(&self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", string)
    }
}
