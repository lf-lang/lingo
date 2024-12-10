use crate::util::sha1dir;
use colored::Colorize;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use versions::Versioning;

use log::error;
use serde::de::Error as DeserializationError;
use serde::ser::Error as SerializationError;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::GitCloneAndCheckoutCap;

use crate::package::management::copy_dir_all;
use crate::package::{
    deserialize_version, serialize_version,
    target_properties::{LibraryTargetProperties, MergeTargetProperties},
    tree::{DependencyTreeNode, PackageDetails, ProjectSource},
    ConfigFile,
};
use crate::util::errors::LingoError;

pub struct ParseLockSourceError {}

/// Different package sources types, available inside the lock file.
#[derive(PartialEq, Debug)]
pub enum PackageLockSourceType {
    REGISTRY,
    GIT,
    TARBALL,
    PATH,
}

/// Struct that saves the source uri string
#[derive(Debug)]
pub struct PackageLockSource {
    pub source_type: PackageLockSourceType,
    pub uri: String,
    pub rev: Option<String>,
}

// Tries to parse the enum value from given string
impl FromStr for PackageLockSourceType {
    type Err = ParseLockSourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "registry" => Ok(Self::REGISTRY),
            "git" => Ok(Self::GIT),
            "path" => Ok(Self::PATH),
            "tar" => Ok(Self::TARBALL),
            _ => Err(ParseLockSourceError {}),
        }
    }
}

/// generates the corresponding string based on enum value
impl Display for PackageLockSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::REGISTRY => "registry".to_string(),
            Self::GIT => "git".to_string(),
            Self::PATH => "path".to_string(),
            Self::TARBALL => "tar".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl From<ProjectSource> for PackageLockSourceType {
    fn from(value: ProjectSource) -> Self {
        match value {
            ProjectSource::Git(_) => Self::GIT,
            ProjectSource::TarBall(_) => Self::TARBALL,
            ProjectSource::Path(_) => Self::PATH,
        }
    }
}

/// Parses the whole source uri string of a package
/// the uri string follows the pattern <type>+<url>(#<git-rev>)
impl FromStr for PackageLockSource {
    type Err = ParseLockSourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("+") {
            Some((source_type_string, mut uri)) => {
                let source_type = PackageLockSourceType::from_str(source_type_string)?;

                let rev: Option<String> = match source_type {
                    PackageLockSourceType::GIT => match uri.split_once("#") {
                        Some((url, rev)) => {
                            uri = url;
                            Some(rev.to_string())
                        }
                        None => {
                            return Err(ParseLockSourceError {});
                        }
                    },
                    _ => None,
                };

                Ok(PackageLockSource {
                    source_type,
                    uri: uri.to_string(),
                    rev,
                })
            }
            None => Err(ParseLockSourceError {}),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PackageLock {
    pub name: String,
    #[serde(
        serialize_with = "serialize_version",
        deserialize_with = "deserialize_version"
    )]
    pub version: Versioning,
    pub source: PackageLockSource,
    pub checksum: String,
}

impl From<DependencyTreeNode> for PackageLock {
    fn from(value: DependencyTreeNode) -> Self {
        let uri = match &value.package.mutual_exclusive {
            ProjectSource::Git(git) => git.to_string(),
            ProjectSource::TarBall(tar) => tar.to_string(),
            ProjectSource::Path(path) => format!("{:?}", path),
        };

        PackageLock {
            name: value.name,
            version: value.version,
            source: PackageLockSource {
                source_type: PackageLockSourceType::from(value.package.mutual_exclusive),
                uri,
                rev: value.package.git_rev,
            },
            checksum: value.hash,
        }
    }
}

impl<'de> Deserialize<'de> for PackageLockSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match PackageLockSource::from_str(&s) {
            Ok(value) => Ok(value),
            Err(_) => Err(D::Error::custom("cannot parse package source string!")),
        }
    }
}

impl Serialize for PackageLockSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let source_type = self.source_type.to_string();
        let mut serialized_string = format!("{}+{}", source_type, self.uri);

        if self.source_type == PackageLockSourceType::GIT {
            if let Some(rev) = self.rev.clone() {
                serialized_string = format!("{}#{}", serialized_string, rev)
            } else {
                error!("expected and revision but got none during serialization of lock file!");
                return Err(S::Error::custom("expected revision but gone None"));
            }
        }

        serializer.serialize_str(&serialized_string)
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct DependencyLock {
    /// mapping from package name to location
    #[serde(flatten)]
    pub dependencies: HashMap<String, PackageLock>,

    /// this will be populated when the project is successfully loaded from the lock file
    #[serde(skip)]
    loaded_dependencies: Vec<DependencyTreeNode>,
}

impl DependencyLock {
    pub(crate) fn create(selected_dependencies: Vec<DependencyTreeNode>) -> DependencyLock {
        let mut map = HashMap::new();
        for dependency in &selected_dependencies {
            map.insert(
                dependency.name.clone(),
                PackageLock::from(dependency.clone()),
            );
        }
        Self {
            dependencies: map,
            loaded_dependencies: selected_dependencies,
        }
    }

    pub fn init(
        &mut self,
        lfc_include_folder: &Path,
        git_clone_and_checkout_cap: &GitCloneAndCheckoutCap,
    ) -> anyhow::Result<()> {
        for (_, lock) in self.dependencies.iter() {
            let temp = lfc_include_folder.join(&lock.name);
            // the Lingo.toml for this dependency doesnt exists, hence we need to fetch this package
            if !temp.join("Lingo.toml").exists() {
                let mut details = PackageDetails::try_from(&lock.source)?;

                details
                    .fetch(&temp, git_clone_and_checkout_cap)
                    .expect("cannot pull package");
            }

            let hash = sha1dir::checksum_current_dir(&temp, false);

            if hash.to_string() != lock.checksum {
                error!("checksum does not match aborting!");
            }

            let lingo_toml_text = fs::read_to_string(temp.join("Lingo.toml"))?;
            let read_toml = toml::from_str::<ConfigFile>(&lingo_toml_text)?.to_config(&temp);

            println!(
                "{} {} ... {}",
                "Reading".green().bold(),
                lock.name,
                read_toml.package.version
            );

            let lib = match read_toml.library {
                Some(value) => value,
                None => {
                    // error we expected a library here
                    return Err(LingoError::NoLibraryInLingoToml(
                        temp.join("Lingo.toml").display().to_string(),
                    )
                    .into());
                }
            };

            self.loaded_dependencies.push(DependencyTreeNode {
                name: read_toml.package.name.clone(),
                version: read_toml.package.version.clone(),
                package: PackageDetails {
                    version: Default::default(),
                    mutual_exclusive: ProjectSource::Path(PathBuf::new()),
                    git_tag: None,
                    git_rev: None,
                },
                location: temp.clone(),
                include_path: lib.location.clone(),
                hash: lock.checksum.clone(),
                dependencies: vec![],
                properties: lib.properties.clone(),
            });
        }

        Ok(())
    }

    pub fn create_library_folder(
        &self,
        source_path: &Path,
        target_path: &PathBuf,
    ) -> anyhow::Result<()> {
        fs::create_dir_all(target_path)?;
        for (_, dep) in self.dependencies.iter() {
            let local_source = source_path.join(&dep.checksum);
            let find_source = target_path.clone().join(&dep.name);
            fs::create_dir_all(&find_source)?;
            copy_dir_all(&local_source, &find_source)?;
        }

        Ok(())
    }

    pub fn aggregate_target_properties(&self) -> anyhow::Result<LibraryTargetProperties> {
        let mut i = LibraryTargetProperties::default();
        for tp in &self.loaded_dependencies {
            i.merge(&tp.properties)?;
        }

        Ok(i)
    }
}
