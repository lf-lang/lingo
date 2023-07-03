use crate::args::{InitArgs, Platform, TargetLanguage};
use crate::util::{analyzer, copy_recursively};

use serde_derive::{Deserialize, Serialize};

use std::collections::HashMap;

use std::fs::{read_to_string, remove_dir_all, remove_file, write};
use std::io;
use std::io::ErrorKind;
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
    pub main_reactor: Option<PathBuf>,

    /// target of the app
    pub target: TargetLanguage,

    pub platform: Platform,

    pub dependencies: HashMap<String, DetailedDependency>,

    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct App {
    /// Absolute path to the directory where the Lingo.toml file is located.
    pub root_path: PathBuf,

    /// Name of the app (and the final binary).
    pub name: String,

    /// Absolute path to the main reactor file.
    pub main_reactor: PathBuf,
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
    // FIXME: The default should be that it searches the `src` directory for a main reactor
    const DEFAULT_MAIN_REACTOR_RELPATH: &'static str = "src/Main.lf";

    pub fn new_for_init_task(init_args: InitArgs) -> io::Result<ConfigFile> {
        let src_path = Path::new("./src");
        let main_reactors = if src_path.exists() {
            analyzer::find_main_reactors(src_path)?
        } else {
            vec![analyzer::MainReactorSpec {
                name: "Main".into(),
                path: src_path.join("Main.lf"),
                target: init_args.get_target_language(),
            }]
        };
        let app_specs = main_reactors
            .into_iter()
            .map(|spec| AppFile {
                name: Some(spec.name),
                main_reactor: Some(spec.path),
                target: spec.target,
                platform: init_args.platform.unwrap_or(Platform::Native),
                dependencies: HashMap::new(),
                properties: HashMap::new(),
            })
            .collect::<Vec<_>>();

        let result = ConfigFile {
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
            apps: app_specs,
        };
        Ok(result)
    }

    pub fn write(&self, path: &Path) {
        let toml_string = toml::to_string(&self).unwrap();
        write(path, toml_string).unwrap_or_else(|_| panic!("cannot write toml file {:?}", &path));
    }

    pub fn from(path: &Path) -> io::Result<ConfigFile> {
        read_to_string(path).and_then(|contents| {
            toml::from_str(&contents)
                .map_err(|e| io::Error::new(ErrorKind::InvalidData, format!("{}", e)))
        })
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
            panic!("Failed to initialize project, invalid location"); // FIXME: Handle properly
        }
    }

    /// The `path` is the path to the directory containing the Lingo.toml file.
    pub fn to_config(self, path: &Path) -> Config {
        let package_name = &self.package.name;
        Config {
            properties: self.properties,
            apps: self
                .apps
                .into_iter()
                .map(|app| App {
                    root_path: path.to_path_buf(),
                    name: app.name.unwrap_or(package_name.clone()),
                    main_reactor: {
                        let mut abs = path.to_path_buf();
                        abs.push(
                            app.main_reactor
                                .unwrap_or(Self::DEFAULT_MAIN_REACTOR_RELPATH.into()),
                        );
                        abs
                    },
                    target: app.target,
                    platform: app.platform,
                    dependencies: app.dependencies,
                    properties: app.properties,
                })
                .collect(),
            package: self.package,
        }
    }
}
