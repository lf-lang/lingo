use crate::package::version::{from_version_string, to_version_string, Version};
use crate::package::{LIBRARY_DIRECTORY};
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use url::Url;

#[derive(Clone, Deserialize, Serialize)]
enum ProjectSource {
    #[serde(rename = "git")]
    Git(Url),
    #[serde(rename = "tarball")]
    TarBall(Url),
    #[serde(rename = "path")]
    Path(PathBuf),
}

/// Dependency with source and version
#[derive(Clone, Deserialize, Serialize)]
pub struct DetailedDependency {
    #[serde(
        deserialize_with = "from_version_string",
        serialize_with = "to_version_string"
    )]
    version: Version,
    #[serde(flatten)]
    mutual_exclusive: ProjectSource,
}

//TODO: we probably want a LockedDependency here with SHA-Hash and Revision

pub struct DependencyManager(Vec<(String, DetailedDependency)>);

/// this copies all the files recursively from one location to another
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

impl DetailedDependency {
    /// this function fetches the specified location and places it at the given location
    fn fetch(&self, library_path: &PathBuf) -> anyhow::Result<()> {
        match &self.mutual_exclusive {
            ProjectSource::Path(path_buf) => Ok(copy_dir_all(path_buf, library_path)?),
            ProjectSource::Git(git_url) => {
                git2::Repository::clone(git_url.as_str(), library_path)?;
                Ok(())
            }
            _ => todo!("Not Supported"),
        }
    }
}

impl DependencyManager {
    pub fn new(data: Vec<(String, DetailedDependency)>) -> DependencyManager {
        DependencyManager(data)
    }

    pub fn pull_dependencies(&self, target_path: &PathBuf) -> anyhow::Result<()> {
        let library_path = target_path.join(LIBRARY_DIRECTORY);

        fs::create_dir_all(&library_path)?;
        self.0
            .iter()
            .try_for_each(|(name, dependency)| dependency.fetch(&library_path.join(name)))?;

        Ok(())
    }
}
