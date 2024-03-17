use std::collections::HashMap;
use crate::package::{LIBRARY_DIRECTORY};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use crate::package::tree::{DependencyTree, DependencyTreeNode, PackageDetails, ProjectSource};


//TODO: we probably want a LockedDependency here with SHA-Hash and Revision

pub struct DependencyManager {
    tree: DependencyTree
}

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

impl PackageDetails {
    /// this function fetches the specified location and places it at the given location
    pub fn fetch(&self, library_path: &PathBuf) -> anyhow::Result<()> {
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
    pub fn new(dependencies: HashMap<String, PackageDetails>, target_path: &PathBuf) -> anyhow::Result<DependencyManager> {
        let library_path = target_path.join(LIBRARY_DIRECTORY);
        fs::create_dir_all(&library_path)?;

        let tree = DependencyTree::new(dependencies, target_path)?;

        Ok(DependencyManager {
            tree,
        })
    }

}
