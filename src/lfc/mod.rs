use crate::util::command_line::run_and_capture;
use crate::App;

use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::write;
use std::path::{Path, PathBuf};
use std::process::Command;

///
/// taken from: https://www.lf-lang.org/docs/handbook/target-declaration?target=c
///
#[derive(Serialize, Deserialize, Clone)]
pub struct Properties {}

///
/// struct which is given to lfc for code generation
///
#[derive(Serialize, Deserialize, Clone)]
pub struct LFCProperties {
    pub src: PathBuf,
    pub out: PathBuf,
    pub properties: HashMap<String, serde_json::Value>,
}

///
/// this struct contains everything that is required to invoke lfc
///
pub struct CodeGenerator {
    pub lfc: PathBuf,
    pub properties: LFCProperties,
}

impl LFCProperties {
    /// root points to root of project
    pub fn new(
        src: PathBuf,
        out: PathBuf,
        properties: HashMap<String, serde_json::Value>,
    ) -> LFCProperties {
        LFCProperties {
            src,
            out,
            properties,
        }
    }

    /// write lfc properties to file
    pub fn write(&self, path: &Path) -> std::io::Result<()> {
        write(path, self.to_string())
    }

    /// convert lfc properties to string
    pub fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl CodeGenerator {
    pub fn new(
        src: PathBuf,
        out: PathBuf,
        lfc: Option<PathBuf>,
        properties: HashMap<String, serde_json::Value>,
    ) -> CodeGenerator {
        CodeGenerator {
            lfc: lfc.unwrap_or(PathBuf::from("/")),
            properties: LFCProperties::new(src, out, properties),
        }
    }

    pub fn generate_code(self, app: &App) -> std::io::Result<()> {
        // path to the src-gen directory
        let mut src_gen_directory = app.root_path.clone();
        src_gen_directory.push(PathBuf::from("./src-gen"));
        println!(
            "generating code ... {}/bin/lfc --json={}",
            &self.lfc.display(),
            self.properties.to_string()
        );

        let mut command = Command::new(format!("{}/bin/lfc", &self.lfc.display()));
        command.arg(format!("--json={}", self.properties.to_string()));

        match run_and_capture(&mut command) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("error while generating code {:?}", e);
                Err(e)
            }
        }
    }
}
