use std::collections::HashMap;
use std::fmt::Display;
use std::fs::write;
use std::io;

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_derive::{Deserialize, Serialize};
use serde_json;

use crate::util::command_line::run_and_capture;
use crate::App;

///
/// taken from: https://www.lf-lang.org/docs/handbook/target-declaration?target=c
///
#[derive(Serialize, Deserialize, Clone)]
pub struct Properties {}

///
/// struct which is given to lfc for code generation
///
/// TODO all of this is contained in the App struct. Why do we need this?
///
#[derive(Serialize, Deserialize, Clone)]
pub struct LFCProperties {
    /// Path to the LF source file containing the main reactor.
    pub src: PathBuf,
    /// Path to the directory into which build artifacts like
    /// the src-gen and bin directory are generated.
    pub out: PathBuf,
    pub properties: HashMap<String, serde_json::Value>,
}

impl Display for LFCProperties {
    /// convert lfc properties to string
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let string = serde_json::to_string(&self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", string)
    }
}

impl LFCProperties {
    /// root points to root of project
    pub fn new(
        src: PathBuf,
        out: PathBuf,
        mut properties: HashMap<String, serde_json::Value>,
    ) -> LFCProperties {
        properties.insert("no-compile".to_string(), serde_json::Value::Bool(true));
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
}

pub fn invoke_code_generator(
    lfc_exec: &Path,
    properties: &LFCProperties,
    app: &App,
) -> io::Result<()> {
    // path to the src-gen directory
    let mut src_gen_directory = app.root_path.clone();
    src_gen_directory.push("src-gen");

    println!(
        "Invoking code-generator: `{} --json={}`",
        lfc_exec.display(),
        properties
    );

    let mut command = Command::new(lfc_exec);
    command.arg(format!("--json={}", properties));
    run_and_capture(&mut command).map(|_| ())
}
