use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde_derive::Serialize;

use crate::backends::{BatchBackend, BatchBuildResults, BuildCommandOptions, CommandSpec};
use crate::package::App;
use crate::util::errors::BuildResult;

pub struct LFC;

impl LFC {
    /// Do codegen for all apps in the batch result in parallel.
    pub fn do_parallel_lfc_codegen(
        options: &BuildCommandOptions,
        results: &mut BatchBuildResults,
        compile_target_code: bool,
    ) {
        results.par_map(|app| LFC::do_lfc_codegen(app, options, compile_target_code));
    }

    /// Do codegen for a single app.
    fn do_lfc_codegen(
        app: &App,
        options: &BuildCommandOptions,
        compile_target_code: bool,
    ) -> BuildResult {
        fs::create_dir_all(&app.output_root)?;

        let mut lfc_command = Command::new(&options.lfc_exec_path);
        lfc_command.arg(format!("--json='{}'", LfcJsonArgs::new(app)));
        if !compile_target_code {
            lfc_command.arg("--no-compile");
        }
        crate::util::execute_command_to_build_result(lfc_command)
    }
}

impl BatchBackend for LFC {
    fn execute_command<'a>(&mut self, command: &CommandSpec, results: &mut BatchBuildResults<'a>) {
        match command {
            CommandSpec::Build(options) => {
                LFC::do_parallel_lfc_codegen(options, results, options.compile_target_code)
            }
            CommandSpec::Update => todo!(),
            CommandSpec::Clean => {
                results.par_map(|app| {
                    fs::remove_dir_all(app.src_gen_dir())?;
                    Ok(())
                });
            }
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
