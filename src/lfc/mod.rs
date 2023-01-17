use serde::{Serialize, Deserialize};
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
    pub src: Path,
    pub out: Path,
    pub properties: Properties
}


impl CommunicationLFC {
    fn default() -> CommunicationLFC {
        CommunicationLFC {
            src: env::current_dir() + PathBuf::from("src-gen")
        }
    }

}


