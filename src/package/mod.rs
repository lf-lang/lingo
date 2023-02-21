use crate::analyzer;

use serde_derive::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Clone)]
pub struct AppVec {
    pub app: Vec<AppFile>,
}

/// the Lingo.toml format is defined by this struct
#[derive(Clone, Deserialize, Serialize)]
pub struct ConfigFile {
    /// top level package description
    pub package: PackageDescription,

    /// high level properties that are set for every app inside the package
    pub properties: HashMap<String, serde_json::Value>,

    /// list of apps defined inside this package
    #[serde(rename = "app")]
    pub apps: Vec<AppFile>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    /// top level package description
    pub package: PackageDescription,

    /// high level properties that are set for every app inside the package
    pub properties: HashMap<String, serde_json::Value>,

    /// list of apps defined inside this package
    #[serde(rename = "app")]
    pub apps: Vec<App>,
}

/// Schema of the configuration parsed from the Lingo.toml
#[derive(Clone, Deserialize, Serialize)]
pub struct AppFile {
    /// if not specified will default to value specified in the package description
    pub name: Option<String>,

    /// if not specified will default to main.lf
    pub main_reactor: Option<String>,

    /// target of the app
    pub target: String,

    pub dependencies: HashMap<String, DetailedDependency>,

    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct App {
    /// where the Lingo.toml is located in the filesystem
    pub root_path: PathBuf,

    pub name: String,
    pub main_reactor: String,
    pub target: String,

    dependencies: HashMap<String, DetailedDependency>,
    properties: HashMap<String, serde_json::Value>,
}

/// Simple or DetailedDependcy
#[derive(Clone, Deserialize, Serialize)]
pub enum FileDependcy {
    // the version string
    Simple(String),
    /// version string and source
    Advanced(DetailedDependency),
}

/// Dependcy with source and version
#[derive(Clone, Deserialize, Serialize)]
pub struct DetailedDependency {
    version: String,
    git: Option<String>,
    tarball: Option<String>,
    zip: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PackageDescription {
    pub name: String,
    pub version: String,
    pub authors: Option<Vec<String>>,
    pub website: Option<String>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
}

impl ConfigFile {
    pub fn new() -> ConfigFile {
        let _main_reactor = if !std::path::Path::new("./src").exists() {
            vec![String::from("Main")]
        } else {
            analyzer::search(Path::new("./src"))
        };

        ConfigFile {
            package: PackageDescription {
                name: std::env::current_dir()
                    .expect("error while reading current directory")
                    .as_path()
                    .file_name()
                    .expect("cannot get file name")
                    .to_string_lossy()
                    .to_string(),
                version: "0.1.0".to_string(),
                authors: None,
                website: None,
                license: None,
                description: None,
                homepage: None,
            },
            properties: HashMap::new(),
            apps: vec![AppFile {
                name: None,
                main_reactor: None,
                target: "cpp".to_string(),
                dependencies: HashMap::new(),
                properties: HashMap::new(),
            }],
        }
    }

    pub fn write(&self, path: &Path) {
        let toml_string = toml::to_string(&self).unwrap();
        write(path, toml_string).unwrap_or_else(|_| panic!("cannot write toml file {:?}", &path));
    }

    pub fn from(path: &Path) -> Option<ConfigFile> {
        match read_to_string(path) {
            Ok(content) => toml::from_str(&content)
                .map_err(|e| println!("the Lingo.toml has an invalid format! Error: {:?}", e))
                .ok(),
            Err(_) => {
                println!("cannot read Lingo.toml does it exist?");
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

    pub fn to_config(mut self, path: PathBuf) -> Config {
        Config {
            package: self.package.clone(),
            properties: self.properties,
            apps: self
                .apps
                .iter_mut()
                .map(|app| App {
                    root_path: path.clone(),
                    name: app.name.as_ref().unwrap_or(&self.package.name).clone(),
                    main_reactor: app
                        .main_reactor
                        .clone()
                        .unwrap_or("src/main.lf".to_string()),
                    target: app.target.clone(),
                    dependencies: app.dependencies.clone(),
                    properties: app.properties.clone(),
                })
                .collect(),
        }
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self::new()
    }
}
