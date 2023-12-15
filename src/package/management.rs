use std::path::PathBuf;
use serde_derive::{Deserialize, Serialize};
use url::Url;
use std::path::Path;
use crate::package::version::{Version, to_version_string, from_version_string};

#[derive(Clone, Deserialize, Serialize)]
enum ProjectSource {
    #[serde(rename="git")]
    Git(Url),
    #[serde(rename="tarball")]
    TarBall(Url),
    #[serde(rename="path")]
    Path(PathBuf)
}

/// Dependency with source and version
#[derive(Clone, Deserialize, Serialize)]
pub struct DetailedDependency {
    #[serde(deserialize_with="from_version_string", serialize_with="to_version_string")]
    version: Version,
    #[serde(flatten)]
    mutual_exclusive: ProjectSource
}

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
    fn fetch(&self, library_path: &PathBuf) -> anyhow::Result<()>{
        match &self.mutual_exclusive {
            ProjectSource::Path(path_buf) => {
                Ok(copy_dir_all(path_buf, library_path)?)
            },
            ProjectSource::Git(git_url ) => {
                git2::Repository::clone(git_url.as_str(), library_path)?;
                Ok(())
            },
            _ => todo!("Not Supported")
        }
    }
}