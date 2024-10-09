use serde::{Deserialize, Serialize};
use url::Url;
use versions::{Requirement, Versioning};

use std::path::PathBuf;

use crate::package::target_properties::LibraryTargetProperties;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum ProjectSource {
    #[serde(rename = "git")]
    Git(Url),
    #[serde(rename = "tarball")]
    TarBall(Url),
    #[serde(rename = "path")]
    Path(PathBuf),
    //#[serde(rename = "empty")]
    //Empty,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GitLock {
    #[serde(rename = "tag")]
    Tag(String),

    #[serde(rename = "branch")]
    Branch(String),

    #[serde(rename = "rev")]
    Rev(String),
}

/// Dependency with source and version
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PackageDetails {
    #[serde(
        deserialize_with = "Requirement::deserialize",
        serialize_with = "Requirement::serialize"
    )]
    pub(crate) version: Requirement,
    #[serde(flatten)]
    pub(crate) mutual_exclusive: ProjectSource,
    #[serde(flatten)]
    pub(crate) git_tag: Option<GitLock>,
    #[serde(skip)]
    pub(crate) git_rev: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DependencyTreeNode {
    /// Name of this Package
    pub(crate) name: String,
    /// version specified in the Lingo.toml
    pub(crate) version: Versioning,
    /// source of this package
    pub(crate) package: PackageDetails,
    /// location of where the packed has been cloned to
    pub(crate) location: PathBuf,
    /// relative path inside the package where the to include contents live
    pub(crate) include_path: PathBuf,
    /// hash of the package
    pub(crate) hash: String,
    /// required dependencies to build this package
    pub(crate) dependencies: Vec<DependencyTreeNode>,
    /// required dependencies to build this package
    pub(crate) properties: LibraryTargetProperties,
}

impl DependencyTreeNode {
    pub fn shallow_clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            package: self.package.clone(),
            location: self.location.clone(),
            include_path: self.include_path.clone(),
            hash: self.hash.clone(),
            dependencies: Vec::new(),
            properties: Default::default(),
        }
    }

    pub fn aggregate(&self) -> Vec<DependencyTreeNode> {
        let mut aggregator = vec![self.shallow_clone()];

        for dependency in &self.dependencies {
            let mut aggregation = dependency.aggregate();
            aggregator.append(&mut aggregation);
        }

        aggregator
    }
}
