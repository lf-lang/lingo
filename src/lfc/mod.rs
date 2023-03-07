use crate::App;
use crate::util::command_line::run_and_capture;

use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::process::Command;
use std::collections::HashMap;
use std::fs::write;
use std::path::{Path, PathBuf};

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
    pub properties: LFCProperties
}

impl LFCProperties {
    /// root points to root of project
    pub fn new(
        root: PathBuf,
        properties: HashMap<String, serde_json::Value>,
    ) -> LFCProperties {
        let mut src = root.clone();
        src.push(PathBuf::from("src"));

        let mut out = root.clone();
        out.push(PathBuf::from("src-gen"));

        LFCProperties {
            src,
            out,
            properties,
        }
    }

    pub fn write(&self, path: &Path) -> std::io::Result<()> {
        let json_string = serde_json::to_string(&self).unwrap();
        write(path, json_string)
    }
}

impl CodeGenerator {
    pub fn new(
        root: PathBuf,
        lfc: PathBuf,
        properties: HashMap<String, serde_json::Value>,
    ) -> CodeGenerator {
        CodeGenerator { lfc, properties: LFCProperties::new(root, properties)}
    }


    pub fn generate_code(self, app: &App) -> std::io::Result<()> {
        // path to the src-gen directory
        let mut src_gen_directory = app.root_path.clone();
        src_gen_directory.push(PathBuf::from("./src-gen"));

        match self.properties.write(&src_gen_directory) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("cannot write src-ge/lfc.json with error {:?}", &e);
            }
        }

        let mut command = Command::new("lfc");
        command.arg("--no-compile");
        command.arg("--output");
        command.arg("./");
        command.arg(format!(
            "{}/{}",
            &app.root_path.display(),
            &app.main_reactor
        ));
        match run_and_capture(&mut command) {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                eprintln!("error while generating code {:?}", e);
                Err(e)
            }
        }
    }
}

