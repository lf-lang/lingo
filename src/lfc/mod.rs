use serde_derive::{Deserialize, Serialize};
use serde_json;
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
pub struct CommunicationLFC {
    pub src: PathBuf,
    pub out: PathBuf,
    pub properties: HashMap<String, serde_json::Value>,
}

impl CommunicationLFC {
    /// path points to root of project
    pub fn default(
        path: &PathBuf,
        properties: HashMap<String, serde_json::Value>,
    ) -> CommunicationLFC {
        let mut src = path.clone();
        src.push(PathBuf::from("src"));

        let mut out = path.clone();
        out.push(PathBuf::from("src-gen"));

        CommunicationLFC {
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
