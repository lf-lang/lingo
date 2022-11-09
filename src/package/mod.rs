use crate::analyzer;

use serde_derive::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs::{read_to_string, write};
use std::path::Path;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub package: Package,
    dependencies: HashMap<String, String>,
    libraries: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub language: String,
    pub main_reactor: Vec<String>,
    pub authors: Option<Vec<String>>,
    pub website: Option<String>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
}

impl Config {
    pub fn new() -> Config {
        let main_reactor = if !std::path::Path::new("./src").exists() {
            vec![String::from("Main")]
        } else {
            analyzer::search(Path::new("./src"))
        };

        Config {
            package: Package {
                name: std::env::current_dir()
                    .expect("error while reading current directory")
                    .as_path()
                    .file_name()
                    .expect("cannot get file name")
                    .to_string_lossy()
                    .to_string(),
                version: "0.1.0".to_string(),
                authors: None,
                language: "".to_string(),
                main_reactor,
                website: None,
                license: None,
                description: None,
                homepage: None,
            },
            dependencies: HashMap::new(),
            libraries: HashMap::new(),
        }
    }

    pub fn write(&self, path: &Path) {
        let toml_string = toml::to_string(&self).unwrap();
        write(path, &toml_string).expect("cannot write toml file");
    }

    pub fn from(path: &Path) -> Option<Config> {
        match read_to_string(path) {
            Ok(content) => toml::from_str(&content)
                .map_err(|_| println!("the Barrel.toml has an invalid format!"))
                .ok(),
            Err(_) => {
                println!("cannot read Barrel.toml does it exist?");
                None
            }
        }
    }

    pub fn setup_example() {
        if !std::path::Path::new("./src").exists() {
            std::fs::create_dir_all("./src").expect("Cannot create target directory");
            let hello_world_code: &'static str = include_str!("../../defaults/Main.lf");
            write(Path::new("./src/Main.lf"), hello_world_code)
                .expect("cannot write Main.lf file!");
        }
    }
}
