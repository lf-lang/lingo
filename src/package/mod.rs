use crate::args::{BuildSystem, InitArgs, Platform, TargetLanguage};
use crate::util::{analyzer, copy_recursively};

use serde_derive::{Deserialize, Serialize};

use std::collections::HashMap;

use std::fs::{read_to_string, remove_dir_all, remove_file, write};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::{env, io};

use crate::args::BuildSystem::{CMake, Cargo, LFC};
use crate::util::errors::{BuildResult, LingoError};
use git2::Repository;
use tempfile::tempdir;
use which::which;

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

    pub platform: Option<Platform>,

    pub dependencies: HashMap<String, DetailedDependency>,

    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct App {
    /// Absolute path to the directory where the Lingo.toml file is located.
    pub root_path: PathBuf,

    /// Name of the app (and the final binary).
    pub name: String,
    /// Root directory where to place src-gen and other compilation-specifics stuff.
    pub output_root: PathBuf,

    /// Absolute path to the main reactor file.
    pub main_reactor: PathBuf,
    pub target: TargetLanguage,
    pub platform: Platform,

    pub dependencies: HashMap<String, DetailedDependency>,
    pub properties: HashMap<String, serde_json::Value>,
}

impl App {
    pub fn build_system(&self) -> BuildSystem {
        match self.target {
            TargetLanguage::C => LFC,
            TargetLanguage::Cpp => CMake,
            TargetLanguage::Rust => Cargo,
            TargetLanguage::TypeScript => {
                if which("pnpm").is_ok() {
                    BuildSystem::Pnpm
                } else {
                    BuildSystem::Npm
                }
            }
            _ => LFC,
        }
    }
    pub fn src_gen_dir(&self) -> PathBuf {
        self.output_root.join("src-gen")
    }
    pub fn executable_path(&self) -> PathBuf {
        let mut p = self.output_root.join("bin");
        if self.target == TargetLanguage::TypeScript {
            p.push(self.name.clone() + ".js")
        } else {
            p.push(&self.name);
        }
        p
    }
}

/// Simple or DetailedDependency
#[derive(Clone, Deserialize, Serialize)]
pub enum FileDependency {
    // the version string
    Simple(String),
    /// version string and source
    Advanced(DetailedDependency),
}

/// Dependency with source and version
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
                platform: Some(init_args.platform),
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

    pub fn write(&self, path: &Path) -> io::Result<()> {
        let toml_string = toml::to_string(&self).unwrap();
        write(path, toml_string)
    }

    pub fn from(path: &Path) -> io::Result<ConfigFile> {
        read_to_string(path).and_then(|contents| {
            toml::from_str(&contents)
                .map_err(|e| io::Error::new(ErrorKind::InvalidData, format!("{}", e)))
        })
    }

    // Sets up a standard LF project for "native" development and deployment
    pub fn setup_native(&self) -> BuildResult {
        std::fs::create_dir_all("./src")?;
        let hello_world_code: &'static str = match self.apps[0].target {
            TargetLanguage::Cpp => include_str!("../../defaults/HelloCpp.lf"),
            TargetLanguage::C => include_str!("../../defaults/HelloC.lf"),
            TargetLanguage::Python => include_str!("../../defaults/HelloPy.lf"),
            TargetLanguage::TypeScript => include_str!("../../defaults/HelloTS.lf"),
            _ => panic!("Target langauge not supported yet"), //FIXME: Add support for Rust.
        };

        write(Path::new("./src/Main.lf"), hello_world_code)?;
        Ok(())
    }

    fn setup_template_repo(&self, url: &str) -> BuildResult {
        let dir = tempdir()?;
        let tmp_path = dir.path();
        Repository::clone(url, tmp_path)?;
        // Copy the cloned template repo into the project directory
        copy_recursively(tmp_path, Path::new("."))?;
        // Remove temporary folder
        dir.close()?;
        Ok(())
    }

    // Sets up a LF project with Zephyr as the target platform.
    fn setup_zephyr(&self) -> BuildResult {
        let url = "https://github.com/lf-lang/lf-west-template";
        self.setup_template_repo(url)?;
        remove_file(".gitignore")?;
        remove_dir_all(Path::new(".git"))?;
        Ok(())
    }

    // Sets up a LF project with RP2040 MCU as the target platform.
    // Initializes a repo using the lf-pico-template
    fn setup_rp2040(&self) -> BuildResult {
        let url = "https://github.com/lf-lang/lf-pico-template";
        // leave git artifacts
        self.setup_template_repo(url)?;
        Ok(())
    }

    pub fn setup_example(&self) -> BuildResult {
        if is_valid_location_for_project(Path::new(".")) {
            match self.apps[0].platform {
                Some(Platform::Native) => self.setup_native(),
                Some(Platform::Zephyr) => self.setup_zephyr(),
                Some(Platform::RP2040) => self.setup_rp2040(),
                _ => Ok(()),
            }
        } else {
            Err(Box::new(LingoError::InvalidProjectLocation(
                env::current_dir().unwrap(),
            )))
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
                    output_root: path.join("target"),
                    main_reactor: {
                        let mut abs = path.to_path_buf();
                        abs.push(
                            app.main_reactor
                                .unwrap_or(Self::DEFAULT_MAIN_REACTOR_RELPATH.into()),
                        );
                        abs
                    },
                    target: app.target,
                    platform: app.platform.unwrap_or(Platform::Native),
                    dependencies: app.dependencies,
                    properties: app.properties,
                })
                .collect(),
            package: self.package,
        }
    }
}
