use crate::args::{InitArgs, Platform, TargetLanguage};
use crate::util::{analyzer, copy_recursively};

use serde_derive::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs::{read_to_string, remove_dir_all, write, remove_file};
use std::path::{Path, PathBuf};


use git2::Repository;

fn is_valid_location_for_project(path: &std::path::Path) -> bool {
    !path.join("src").exists() && !path.join(".git").exists() && !path.join("application").exists()
}

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
    pub target: TargetLanguage,

    pub platform: Platform,

    pub dependencies: HashMap<String, DetailedDependency>,

    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct App {
    /// where the Lingo.toml is located in the filesystem
    pub root_path: PathBuf,

    pub name: String,
    pub main_reactor: String,
    pub target: TargetLanguage,
    pub platform: Platform,

    pub dependencies: HashMap<String, DetailedDependency>,
    pub properties: HashMap<String, serde_json::Value>,
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
    pub fn new(init_args: InitArgs) -> ConfigFile {
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
                target: init_args.language.unwrap_or_else(|| {
                    // Target langauge for Zephyr is C, else Cpp.
                    match init_args.platform {
                        Some(Platform::Zephyr) => TargetLanguage::C,
                        _ => TargetLanguage::Cpp,
                    }
                }),
                platform: init_args.platform.unwrap_or(Platform::Native),
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

    // Sets up a standard LF project for "native" development and deployment
    pub fn setup_native(&self) {
        std::fs::create_dir_all("./src").expect("Cannot create target directory");
        let hello_world_code: &'static str = match self.apps[0].target {
            TargetLanguage::Cpp => include_str!("../../defaults/HelloCpp.lf"),
            TargetLanguage::C => include_str!("../../defaults/HelloC.lf"),
            _ => panic!("Target langauge not supported yet"), // FIXME: Add examples for other programs
        };

        write(Path::new("./src/Main.lf"), hello_world_code).expect("cannot write Main.lf file!");
    }

    // Sets up a LF project with Zephyr as the target platform.
    pub fn setup_zephyr(&self) {
        // Clone lf-west-template into a temporary directory
        let tmp_path = Path::new("zephyr_tmp");
        if tmp_path.exists() {
            remove_dir_all(tmp_path).expect("Could not remove temporarily cloned repository");
        }
        let url = "https://github.com/lf-lang/lf-west-template";
        let _repo = match Repository::clone(url, tmp_path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e), // FIXME: How to handle errors?
        };

        // Copy the cloned template repo into the project directory
        copy_recursively(tmp_path, Path::new(".")).expect("Could not copy cloned repo");
        
        // Remove .git, .gitignore ad temporary folder
        remove_file(".gitignore").expect("Could not remove .gitignore");
        remove_dir_all(Path::new(".git")).expect("Could not remove .git directory");
        remove_dir_all(tmp_path).expect("Could not remove temporarily cloned repository");
    }

    pub fn setup_example(&self) {
        if is_valid_location_for_project(Path::new(".")) {
            match self.apps[0].platform {
                Platform::Native => self.setup_native(),
                Platform::Zephyr => self.setup_zephyr(),
            }
        } else {
            panic!("Failed to initilize project, invalid location"); // FIXME: Handle properly
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
                        .unwrap_or("src/Main.lf".to_string()), // FIXME: The default should be that it searches the `src` directory for a main reactor
                    target: app.target.clone(),
                    platform: app.platform.clone(),
                    dependencies: app.dependencies.clone(),
                    properties: app.properties.clone(),
                })
                .collect(),
        }
    }
}
