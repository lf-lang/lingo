use anyhow::Context;
use colored::Colorize;
use log::error;
use versions::{Requirement, Versioning};

use crate::util::sha1dir;
use crate::{GitCloneAndCheckoutCap, GitUrl};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;
use url::{ParseError, Url};

use crate::package::lock::{PackageLockSource, PackageLockSourceType};
use crate::package::{
    lock::DependencyLock,
    target_properties::LibraryTargetProperties,
    tree::{DependencyTreeNode, GitLock, PackageDetails, ProjectSource},
    ConfigFile, LIBRARY_DIRECTORY,
};
use crate::util::errors::LingoError;

#[derive(Default)]
pub struct DependencyManager {
    /// queue of packages that need processing
    pulling_queue: Vec<(String, PackageDetails)>,
    /// the flatten dependency tree with selected packages from the dependency tree
    lock: DependencyLock,
}

/// this copies all the files recursively from one location to another
pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

impl TryFrom<&PackageLockSource> for PackageDetails {
    type Error = ParseError;

    fn try_from(value: &PackageLockSource) -> Result<Self, Self::Error> {
        let url = &value.uri;
        Ok(PackageDetails {
            version: Default::default(),
            mutual_exclusive: match value.source_type {
                PackageLockSourceType::REGISTRY => {
                    todo!()
                }
                PackageLockSourceType::GIT => ProjectSource::Git(Url::from_str(url)?),
                PackageLockSourceType::TARBALL => ProjectSource::TarBall(Url::from_str(url)?),
                PackageLockSourceType::PATH => ProjectSource::Path(PathBuf::from(url)),
            },
            git_tag: value.rev.clone().map(GitLock::Rev),
            git_rev: value.rev.clone(),
        })
    }
}

impl PackageDetails {
    /// this function fetches the specified location and places it at the given location
    pub fn fetch(
        &mut self,
        library_path: &PathBuf,
        clone: &GitCloneAndCheckoutCap,
    ) -> anyhow::Result<()> {
        match &self.mutual_exclusive {
            ProjectSource::Path(path_buf) => {
                let src = fs::canonicalize(path_buf).with_context(|| {
                    format!("dependency path not found: {}", path_buf.display())
                })?;
                let dst = fs::canonicalize(library_path).with_context(|| {
                    format!("library path not found: {}", library_path.display())
                })?;
                Ok(copy_dir_all(src, dst)?)
            }
            ProjectSource::Git(git_url) => {
                self.git_rev = clone(
                    GitUrl::from(git_url.as_str()),
                    library_path,
                    self.git_tag.clone(),
                )?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl DependencyManager {
    pub fn from_dependencies(
        dependencies: Vec<(String, PackageDetails)>,
        target_path: &Path,
        git_clone_and_checkout_cap: &GitCloneAndCheckoutCap,
    ) -> anyhow::Result<DependencyManager> {
        println!(
            "{} resolving dependencies in {}",
            "Build step:".green().bold(),
            target_path.display()
        );
        // create library folder
        let library_path = target_path.join(LIBRARY_DIRECTORY);
        fs::create_dir_all(&library_path).with_context(|| {
            format!(
                "failed to create library directory: {}",
                library_path.display()
            )
        })?;

        let mut manager;
        let mut lock: DependencyLock;
        let lock_file = target_path.join("../Lingo.lock");

        // checks if a Lingo.lock file exists
        if lock_file.exists() {
            println!(
                "{} loading lock file {}",
                "Build step:".green().bold(),
                lock_file.display()
            );
            // reads and parses Lockfile
            lock =
                toml::from_str::<DependencyLock>(&fs::read_to_string(&lock_file).with_context(
                    || format!("failed to read lock file: {}", lock_file.display()),
                )?)
                .expect("cannot parse lock");

            // if a lock file is present it will load the dependencies from it and checks
            // integrity of the build directory
            println!(
                "{} validating lock dependencies in {}",
                "Build step:".green().bold(),
                target_path.join("lfc_include").display()
            );
            if let Ok(()) = lock.init(&target_path.join("lfc_include"), git_clone_and_checkout_cap)
            {
                return Ok(DependencyManager {
                    pulling_queue: vec![],
                    lock,
                });
            }
        }

        // creates a new dependency manager object
        manager = DependencyManager::default();

        // starts recursively pulling dependencies
        let root_nodes = manager.pull(
            dependencies.clone(),
            target_path,
            git_clone_and_checkout_cap,
        )?;

        // flattens the dependency tree and makes the package selection
        let selection = DependencyManager::flatten(root_nodes.clone())?;

        // creates a lock file struct from the selected packages
        lock = DependencyLock::create(selection);

        // writes the lock file down
        let lock_file_path = target_path.join("../Lingo.lock");
        let mut lock_file = File::create(&lock_file_path)
            .with_context(|| format!("failed to create lock file: {}", lock_file_path.display()))?;

        println!("{:?}", lock.dependencies);
        let serialized_toml = toml::to_string(&lock).expect("cannot generate toml");

        lock_file.write_all(serialized_toml.as_ref())?;

        // moves the selected packages into the include folder
        let include_folder = target_path.join("lfc_include");
        lock.create_library_folder(&library_path, &include_folder)
            .expect("creating lock folder failed");

        // saves the lockfile with the dependency manager
        manager.lock = lock;

        Ok(manager)
    }

    pub fn pull(
        &mut self,
        mut dependencies: Vec<(String, PackageDetails)>,
        root_path: &Path,
        git_clone_and_checkout_cap: &GitCloneAndCheckoutCap,
    ) -> anyhow::Result<Vec<DependencyTreeNode>> {
        let mut sub_dependencies = vec![];
        self.pulling_queue.append(&mut dependencies);
        let sub_dependency_path = root_path.join("libraries");
        //fs::remove_dir_all(&sub_dependency_path)?;
        fs::create_dir_all(&sub_dependency_path).with_context(|| {
            format!(
                "failed to create directory: {}",
                sub_dependency_path.display()
            )
        })?;

        while !self.pulling_queue.is_empty() {
            if let Some((package_name, package_details)) = self.pulling_queue.pop() {
                print!("{} {} ...", "Cloning".green().bold(), package_name);
                let node = match self.non_recursive_fetching(
                    &package_name,
                    package_details,
                    &sub_dependency_path,
                    git_clone_and_checkout_cap,
                ) {
                    Ok(value) => value,
                    Err(e) => {
                        return Err(e);
                    }
                };

                sub_dependencies.push(node);
            } else {
                break;
            }
        }

        //dependencies
        Ok(sub_dependencies)
    }

    pub(crate) fn non_recursive_fetching(
        &mut self,
        name: &str,
        mut package: PackageDetails,
        base_path: &Path,
        git_clone_and_checkout_cap: &GitCloneAndCheckoutCap,
    ) -> anyhow::Result<DependencyTreeNode> {
        // creating the directory where the library will be housed
        let library_path = base_path; //.join("libs");
                                      // place where to drop the source
        let temporary_path = library_path.join("temporary");
        let _ = fs::remove_dir_all(&temporary_path);
        let _ = fs::create_dir_all(&temporary_path);

        // directory where the dependencies will be dropped

        // creating the necessary directories
        fs::create_dir_all(library_path)?;
        fs::create_dir_all(&temporary_path)?;

        // cloning the specified package
        println!(
            "{} fetching dependency {} into {}",
            "Build step:".green().bold(),
            name,
            temporary_path.display()
        );
        package.fetch(&temporary_path, git_clone_and_checkout_cap)?;

        println!(
            "{} computing checksum for {}",
            "Build step:".green().bold(),
            temporary_path.display()
        );
        let checksum_started_at = Instant::now();
        let hash = sha1dir::checksum_current_dir(&temporary_path, false);
        println!(
            "{} checksum complete for {} in {:?}",
            "Build step:".green().bold(),
            temporary_path.display(),
            checksum_started_at.elapsed()
        );
        let include_path = library_path.join(hash.to_string());

        let lingo_toml_path = temporary_path.join("Lingo.toml");
        let lingo_toml_text = fs::read_to_string(&lingo_toml_path)
            .with_context(|| format!("failed to read {}", lingo_toml_path.display()))?;
        let read_toml = toml::from_str::<ConfigFile>(&lingo_toml_text)?.to_config(&temporary_path);

        println!(" {}", read_toml.package.version);

        let config = match read_toml.library {
            Some(value) => value,
            None => {
                // error we expected a library here
                return Err(
                    LingoError::NoLibraryInLingoToml(library_path.display().to_string()).into(),
                );
            }
        };

        if !package.version.matches(&read_toml.package.version) {
            error!("version mismatch between specified location and requested version requirement");
            return Err(LingoError::LingoVersionMismatch(format!(
                "requested version {} got version {}",
                package.version, read_toml.package.version
            ))
            .into());
        }

        let dependencies = vec![];

        for dep in read_toml.dependencies {
            self.pulling_queue.push(dep);
        }

        fs::create_dir_all(&include_path)?;
        copy_dir_all(&temporary_path, &include_path)?;

        Ok(DependencyTreeNode {
            name: name.to_string(),
            package: package.clone(),
            location: include_path.clone(),
            include_path: config.location.clone(),
            dependencies: dependencies.clone(),
            hash: hash.to_string(),
            version: read_toml.package.version.clone(),
            properties: config.properties,
        })
    }

    fn flatten(root_nodes: Vec<DependencyTreeNode>) -> anyhow::Result<Vec<DependencyTreeNode>> {
        // implementation idea:
        // 1.   we collect all the version requirements for packages => are the different
        //      constraints satisfiable ?
        // 2.   we collect all the different sources
        // 3.   finding the set of sources that satisfies the set of version constraints
        // 4.   pick the newest version from that set
        // TODO: later we can probably do this in one pass

        let mut constraints = HashMap::<&String, Vec<Requirement>>::new();
        let mut sources = HashMap::<&String, Vec<&DependencyTreeNode>>::new();

        // this basically flattens the
        let mut nodes = Vec::new();
        for node in root_nodes {
            let mut children = node.aggregate();
            nodes.append(&mut children);
        }

        for node in &nodes {
            let constraint = &node.package.version;

            constraints
                .entry(&node.name)
                .and_modify(|value| {
                    value.push(constraint.clone());
                })
                .or_insert(vec![constraint.clone()]);

            sources
                .entry(&node.name)
                .and_modify(move |value| {
                    value.push(node);
                })
                .or_insert(vec![&node]);
        }

        let merged: Vec<(&String, Vec<Requirement>, Vec<&DependencyTreeNode>)> = constraints
            .into_iter()
            .filter_map(move |(key, requirements)| {
                sources
                    .get_mut(&key)
                    .map(move |location| (key, requirements, location.clone()))
            })
            .collect();

        let mut selection = Vec::new();

        for (_, requirements, location) in merged {
            //TODO: replace this in the future by first merging all the requirements
            // (determine upper and lower bound)

            let mut filtered_results: Vec<&DependencyTreeNode> = location
                .into_iter()
                .filter(|location| {
                    let filter = |version: &Versioning| {
                        for requirement in &requirements {
                            if !requirement.matches(version) {
                                return false;
                            }
                        }
                        true
                    };

                    filter(&location.version)
                })
                .collect();

            if filtered_results.is_empty() {
                error!("no viable package was found that fulfills all the requirements");
            }

            filtered_results.sort_by_key(|value| value.version.clone());

            let package = filtered_results
                .last()
                .expect("There should be at least one viable package remaining!");

            selection.push((*package).clone());
        }

        Ok(selection)
    }

    pub fn get_target_properties(&self) -> anyhow::Result<LibraryTargetProperties> {
        self.lock.aggregate_target_properties()
    }
}
