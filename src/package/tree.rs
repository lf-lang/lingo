use std::collections::HashMap;
use std::fmt::Error;
use std::fs;
use std::fs::read;
use std::hash::Hash;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use url::Url;
use versions::Requirement;
use log::error;
use run_script::run_or_exit;
use serde::de::Unexpected::Option;
use url::quirks::protocol;
use crate::package::{ConfigFile, LIBRARY_DIRECTORY};
use crate::util::errors::LingoError;

#[derive(Clone, Deserialize, Serialize)]
pub enum ProjectSource {
    #[serde(rename = "git")]
    Git(Url),
    #[serde(rename = "tarball")]
    TarBall(Url),
    #[serde(rename = "path")]
    Path(PathBuf),
}

/// Dependency with source and version
#[derive(Clone, Deserialize, Serialize)]
pub struct PackageDetails {
    #[serde(deserialize_with = "Requirement::deserialize", serialize_with = "Requirement::serialize")]
    version: Requirement,
    #[serde(flatten)]
    pub(crate) mutual_exclusive: ProjectSource,
}

pub struct DependencyTreeNode {
    /// Name of this Package
    name: String,
    /// source of this package
    package: PackageDetails,
    /// location of where the packed has been cloned to
    location: PathBuf,
    /// required dependencies to build this package
    dependencies: Vec<DependencyTreeNode>
}

impl DependencyTreeNode {
    fn recursive_fetching(name: &String, package: PackageDetails, base_path: &PathBuf) -> anyhow::Result<DependencyTreeNode> {
        // creating the directory where the library will be housed
        let library_path = base_path.clone().join(name);
        // place where to drop the source
        let self_path = library_path.clone().join("self");
        // directory where the dependencies will be dropped
        let dependency_path = library_path.clone().join("dependencies");

        // creating the necessary directories
        fs::create_dir_all(&library_path)?;
        fs::create_dir_all(&self_path)?;
        fs::create_dir_all(&dependency_path)?;

        // cloning the specified package
        package.fetch(&self_path)?;

        // now we need to read the lingo.toml in that project to figure out the next dependencies
        let lingo_toml_text = fs::read_to_string(&library_path.clone().join("Lingo.toml"))?;
        let read_toml = serde_json::from_str::<ConfigFile>(&*lingo_toml_text)?.to_config(&self_path);

        if read_toml.library.is_none() { // error we expected a library here
            return Err(LingoError::NoLibraryInLingoToml(library_path.display().to_string()).into())
        }

        let mut dependencies = vec![];

        for (package_name, package_details) in read_toml.dependencies {
            let sub_dependency_path = dependency_path.clone().join(&package_name);
            fs::create_dir_all(&sub_dependency_path)?;

            let node = match DependencyTreeNode::recursive_fetching(&package_name, package_details, &sub_dependency_path) {
                Ok(value) => value,
                Err(e) => {
                    error!("cannot fetch library {package_name} into path {:?}", sub_dependency_path);
                    return Err(e);
                }
            };

            dependencies.push(node);
        }

        Ok(DependencyTreeNode {
            name: name.clone(),
            package,
            location: base_path.clone(),
            dependencies,
        })
    }
}

pub struct DependencyTree {
    dependencies: Vec<DependencyTreeNode>
}

impl DependencyTree {
    pub fn new(toml_dependencies: HashMap<String, PackageDetails>, root_path: &PathBuf) -> anyhow::Result<DependencyTree>{
        let mut dependencies = vec![];
        for (package_name, package_details) in toml_dependencies {
            let sub_dependency_path = root_path.clone().join(&package_name);
            fs::create_dir_all(&sub_dependency_path)?;

            let node = match DependencyTreeNode::recursive_fetching(&package_name, package_details, &sub_dependency_path) {
                Ok(value) => value,
                Err(e) => {
                    error!("cannot fetch library {package_name} into path {:?}", sub_dependency_path);
                    return Err(e);
                }
            };

            dependencies.push(node);
        }

        Ok(DependencyTree{
            dependencies,
        })

    }

}