use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;
use std::process::Command;
use std::{fmt, fs};

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
        lfc_command.arg(format!(
            "--json={}",
            LfcJsonArgs::new(app, compile_target_code)
        ));
        crate::util::execute_command_to_build_result(lfc_command)
    }
}

impl BatchBackend for LFC {
    fn execute_command(&mut self, command: &CommandSpec, results: &mut BatchBuildResults) {
        match command {
            CommandSpec::Build(options) => {
                LFC::do_parallel_lfc_codegen(options, results, options.compile_target_code)
            }
            CommandSpec::Update => todo!(),
            CommandSpec::Clean => {
                results.par_map(|app| {
                    crate::util::default_build_clean(&app.output_root)?;
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
    /// Other properties, mapped to CLI args by LFC.
    pub properties: &'a HashMap<String, serde_json::Value>,
    #[serde(skip)]
    no_compile: bool,
}

impl<'a> LfcJsonArgs<'a> {
    pub fn new(app: &'a App, compile_target_code: bool) -> Self {
        Self {
            src: &app.main_reactor,
            out: &app.output_root,
            properties: &app.properties,
            no_compile: !compile_target_code,
        }
    }

    fn to_properties(&self) -> serde_json::Result<serde_json::Value> {
        let mut value = serde_json::to_value(self)?;
        let properties = value
            .as_object_mut()
            .unwrap()
            .get_mut("properties")
            .unwrap()
            .as_object_mut()
            .unwrap();
        // lfc does not support no-compile:false
        if self.no_compile {
            properties.insert("no-compile".to_string(), 
                              self.no_compile.into());
        }
        Ok(value)
    }
}

impl<'a> Display for LfcJsonArgs<'a> {
    /// convert lfc properties to string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = self
            .to_properties()
            .and_then(|v| serde_json::to_string(&v))
            .map_err(|_| fmt::Error)?;
        write!(f, "{}", string)
    }
}
