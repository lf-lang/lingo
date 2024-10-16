pub mod lock;
pub mod management;
pub mod tree;

pub mod target_properties;

use serde::de::{Error, Visitor};
use serde::{Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use tempfile::tempdir;
use versions::Versioning;

use std::fs::{remove_dir_all, remove_file, write};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fmt, io};

use crate::args::{
    BuildSystem,
    BuildSystem::{CMake, LFC},
    InitArgs, Platform, TargetLanguage,
};
use crate::package::{
    target_properties::{
        AppTargetProperties, AppTargetPropertiesFile, LibraryTargetProperties,
        LibraryTargetPropertiesFile,
    },
    tree::PackageDetails,
};
use crate::util::{
    analyzer, copy_recursively,
    errors::{BuildResult, LingoError},
};
use crate::{FsReadCapability, GitCloneCapability, GitUrl, WhichCapability};

/// place where are the build artifacts will be dropped
pub const OUTPUT_DIRECTORY: &str = "build";
/// name of the folder inside the `OUTPUT_DIRECTORY` where libraries
/// will be loaded (cloned, extracted, copied) into for further processing.
pub const LIBRARY_DIRECTORY: &str = "libraries";

/// default folder for lf executable files
const DEFAULT_EXECUTABLE_FOLDER: &str = "src";

/// default folder for lf library files
const DEFAULT_LIBRARY_FOLDER: &str = "src/lib";

fn is_valid_location_for_project(path: &std::path::Path) -> bool {
    !path.join(DEFAULT_EXECUTABLE_FOLDER).exists()
        && !path.join(".git").exists()
        && !path.join(DEFAULT_LIBRARY_FOLDER).exists()
}

/// list of apps inside a toml file
#[derive(Deserialize, Serialize, Clone)]
pub struct AppVec {
    pub app: Vec<AppFile>,
}

/// The Lingo.toml format is defined by this struct
#[derive(Clone, Deserialize, Serialize)]
pub struct ConfigFile {
    /// top level package description
    pub package: PackageDescription,

    /// list of apps defined inside this package
    #[serde(rename = "app")]
    pub apps: Option<Vec<AppFile>>,

    /// library exported by this Lingo Toml
    #[serde(rename = "lib")]
    pub library: Option<LibraryFile>,

    /// Dependencies for required to build this Lingua-Franca Project
    pub dependencies: HashMap<String, PackageDetails>,
}

/// This struct is used after filling in all the defaults
#[derive(Clone)]
pub struct Config {
    /// top level package description
    pub package: PackageDescription,

    /// list of apps defined inside this package
    pub apps: Vec<App>,

    /// library exported by this package
    pub library: Option<Library>,

    /// Dependencies for required to build this Lingua-Franca Project
    pub dependencies: HashMap<String, PackageDetails>,
}

/// The Format inside the Lingo.toml under [lib]
#[derive(Clone, Deserialize, Serialize)]
pub struct LibraryFile {
    /// if not specified will default to value specified in the package description
    pub name: Option<String>,

    /// if not specified will default to ./lib
    pub location: Option<PathBuf>,

    /// target language of the library
    pub target: TargetLanguage,

    /// platform of this project
    pub platform: Option<Platform>,

    /// target properties of that lingua-franca app
    pub properties: LibraryTargetPropertiesFile,
}

#[derive(Clone)]
pub struct Library {
    /// if not specified will default to value specified in the package description
    pub name: String,

    /// if not specified will default to ./src
    pub location: PathBuf,

    /// target of the app
    pub target: TargetLanguage,

    /// platform of this project
    pub platform: Platform,

    /// target properties of that lingua-franca app
    pub properties: LibraryTargetProperties,

    /// Root directory where to place src-gen and other compilation-specifics stuff.
    pub output_root: PathBuf,
}

/// Schema of the configuration parsed from the Lingo.toml
#[derive(Clone, Deserialize, Serialize)]
pub struct AppFile {
    /// if not specified will default to value specified in the package description
    pub name: Option<String>,

    /// if not specified will default to main.lf
    pub main: Option<PathBuf>,

    /// target of the app
    pub target: TargetLanguage,

    /// platform of this project
    pub platform: Option<Platform>,

    /// target properties of that lingua-franca app
    pub properties: AppTargetPropertiesFile,
}

#[derive(Clone)]
pub struct App {
    /// Absolute path to the directory where the Lingo.toml file is located.
    pub root_path: PathBuf,
    /// Name of the app (and the final binary).
    pub name: String,
    /// Root directory where to place src-gen and other compilation-specifics stuff.
    pub output_root: PathBuf,
    /// Absolute path to the main reactor file.
    pub main_reactor: PathBuf,
    /// main reactor name
    pub main_reactor_name: String,
    /// target language of this lf program
    pub target: TargetLanguage,
    /// platform for which this program should be compiled
    pub platform: Platform,
    /// target properties of that lingua-franca app
    pub properties: AppTargetProperties,
}

impl AppFile {
    const DEFAULT_MAIN_REACTOR_RELPATH: &'static str = "src/Main.lf";
    pub fn convert(self, package_name: &str, path: &Path) -> App {
        let file_name: Option<String> = match self.main.clone() {
            Some(path) => path
                .file_stem()
                .to_owned()
                .and_then(|x| x.to_str())
                .map(|x| x.to_string()),
            None => None,
        };
        let name = self
            .name
            .unwrap_or(file_name.unwrap_or(package_name.to_string()).to_string());

        let mut abs = path.to_path_buf();
        abs.push(
            self.main
                .unwrap_or(Self::DEFAULT_MAIN_REACTOR_RELPATH.into()),
        );

        let temp = abs
            .clone()
            .file_name()
            .expect("cannot extract file name")
            .to_str()
            .expect("cannot convert path to string")
            .to_string();
        let main_reactor_name = &temp[..temp.len() - 3];

        App {
            root_path: path.to_path_buf(),
            name,
            output_root: path.join(OUTPUT_DIRECTORY),
            main_reactor: abs,
            main_reactor_name: main_reactor_name.to_string(),
            target: self.target,
            platform: self.platform.unwrap_or(Platform::Native),
            properties: self.properties.from(path),
        }
    }
}

impl LibraryFile {
    pub fn convert(self, package_name: &str, path: &Path) -> Library {
        let file_name: Option<String> = match self.location.clone() {
            Some(path) => path
                .file_stem()
                .to_owned()
                .and_then(|x| x.to_str())
                .map(|x| x.to_string()),
            None => None,
        };
        let name = self
            .name
            .unwrap_or(file_name.unwrap_or(package_name.to_string()).to_string());

        Library {
            name,
            location: {
                let mut abs = path.to_path_buf();
                abs.push(self.location.unwrap_or(DEFAULT_LIBRARY_FOLDER.into()));
                abs
            },
            target: self.target,
            platform: self.platform.unwrap_or(Platform::Native),
            properties: self.properties.from(path),
            output_root: path.join(OUTPUT_DIRECTORY),
        }
    }
}

impl App {
    pub fn build_system(&self, which: &WhichCapability) -> BuildSystem {
        match self.target {
            TargetLanguage::C => CMake,
            TargetLanguage::Cpp => CMake,
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

    pub fn src_dir_path(&self) -> Option<PathBuf> {
        for path in self.main_reactor.ancestors() {
            if path.ends_with("src") {
                return Some(path.to_path_buf());
            }
        }
        None
    }
}

fn serialize_version<S>(version: &Versioning, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&version.to_string())
}

struct VersioningVisitor;

impl<'de> Visitor<'de> for VersioningVisitor {
    type Value = Versioning;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an valid semantic version")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Versioning::from_str(v).map_err(|_| E::custom("not a valid version"))
    }
}

fn deserialize_version<'de, D>(deserializer: D) -> Result<Versioning, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(VersioningVisitor)
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PackageDescription {
    pub name: String,
    #[serde(
        serialize_with = "serialize_version",
        deserialize_with = "deserialize_version"
    )]
    pub version: Versioning,
    pub authors: Option<Vec<String>>,
    pub website: Option<String>,
    pub license: Option<String>,
    pub description: Option<String>,
}

impl ConfigFile {
    pub fn new_for_init_task(init_args: &InitArgs) -> io::Result<ConfigFile> {
        let src_path = Path::new(DEFAULT_EXECUTABLE_FOLDER);
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
                main: Some(spec.path),
                target: spec.target,
                platform: Some(init_args.platform),
                properties: Default::default(),
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
                version: Versioning::from_str("0.1.0").unwrap(),
                authors: None,
                website: None,
                license: None,
                description: None,
            },
            dependencies: HashMap::default(),
            apps: Some(app_specs),
            library: Option::default(),
        };
        Ok(result)
    }

    pub fn write(&self, path: &Path) -> io::Result<()> {
        let toml_string = toml::to_string(&self).expect("cannot serialize toml");
        write(path, toml_string)
    }

    pub fn from(path: &Path, fsr: FsReadCapability) -> io::Result<ConfigFile> {
        let contents = fsr(path);
        contents.and_then(|contents| {
            toml::from_str(&contents).map_err(|e| {
                io::Error::new(
                    ErrorKind::InvalidData,
                    format!("failed to convert string to toml: {}", e),
                )
            })
        })
    }

    // Sets up a standard LF project for "native" development and deployment
    pub fn setup_native(&self, target_language: TargetLanguage) -> BuildResult {
        std::fs::create_dir_all("./src")?;
        let hello_world_code: &'static str = match target_language {
            TargetLanguage::Cpp => include_str!("../../defaults/HelloCpp.lf"),
            TargetLanguage::C => include_str!("../../defaults/HelloC.lf"),
            TargetLanguage::Python => include_str!("../../defaults/HelloPy.lf"),
            TargetLanguage::TypeScript => include_str!("../../defaults/HelloTS.lf"),
            _ => panic!("Target langauge not supported yet"), //FIXME: Add support for Rust.
        };

        write(Path::new("./src/Main.lf"), hello_world_code)?;
        Ok(())
    }

    fn setup_template_repo(&self, url: &str, clone: &GitCloneCapability) -> BuildResult {
        let dir = tempdir()?;
        let tmp_path = dir.path();
        clone(GitUrl::from(url), tmp_path)?;
        // Copy the cloned template repo into the project directory
        copy_recursively(tmp_path, Path::new("."))?;
        // Remove temporary folder
        dir.close()?;
        Ok(())
    }

    // Sets up a LF project with Zephyr as the target platform.
    fn setup_zephyr(&self, clone: &GitCloneCapability) -> BuildResult {
        let url = "https://github.com/lf-lang/lf-west-template";
        self.setup_template_repo(url, clone)?;
        remove_file(".gitignore")?;
        remove_dir_all(Path::new(".git"))?;
        Ok(())
    }

    // Sets up a LF project with RP2040 MCU as the target platform.
    // Initializes a repo using the lf-pico-template
    fn setup_rp2040(&self, clone: &GitCloneCapability) -> BuildResult {
        let url = "https://github.com/lf-lang/lf-pico-template";
        // leave git artifacts
        self.setup_template_repo(url, clone)?;
        Ok(())
    }

    pub fn setup_example(
        &self,
        platform: Platform,
        target_language: TargetLanguage,
        git_clone_capability: &GitCloneCapability,
    ) -> BuildResult {
        if is_valid_location_for_project(Path::new(".")) {
            match platform {
                Platform::Native => self.setup_native(target_language),
                Platform::Zephyr => self.setup_zephyr(git_clone_capability),
                Platform::RP2040 => self.setup_rp2040(git_clone_capability),
            }
        } else {
            Err(Box::new(LingoError::InvalidProjectLocation(
                env::current_dir().expect("cannot fetch current working directory"),
            )))
        }
    }

    /// The `path` is the path to the directory containing the Lingo.toml file.
    pub fn to_config(self, path: &Path) -> Config {
        let package_name = &self.package.name;

        Config {
            //properties: self.properties,
            apps: self
                .apps
                .unwrap_or_default()
                .into_iter()
                .map(|app_file| app_file.convert(package_name, path))
                .collect(),
            package: self.package.clone(),
            library: self.library.map(|lib| lib.convert(package_name, path)),
            dependencies: self.dependencies,
        }
    }
}
