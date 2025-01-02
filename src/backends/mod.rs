use log::error;
use rayon::prelude::*;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::args::{BuildSystem, Platform, TargetLanguage};
use crate::package::{
    management::DependencyManager, target_properties::MergeTargetProperties, App, Config,
    OUTPUT_DIRECTORY,
};
use crate::util::errors::{AnyError, BuildResult, LingoError};
use crate::{GitCloneAndCheckoutCap, RemoveFolderCap, WhichCapability};

pub mod cmake_c;
pub mod cmake_cpp;
pub mod lfc;
pub mod npm;
pub mod pnpm;

#[allow(clippy::single_match)] // there more options will be added to this match block
pub fn execute_command<'a>(
    command: &CommandSpec,
    config: &'a mut Config,
    which: WhichCapability,
    clone: GitCloneAndCheckoutCap,
    remove_dir_all: RemoveFolderCap,
) -> BatchBuildResults<'a> {
    let mut result = BatchBuildResults::new();
    let dependencies = Vec::from_iter(config.dependencies.clone());

    match command {
        CommandSpec::Build(_build) => {
            let manager = match DependencyManager::from_dependencies(
                dependencies.clone(),
                &PathBuf::from(OUTPUT_DIRECTORY),
                &clone,
            ) {
                Ok(value) => value,
                Err(e) => {
                    error!("failed to create dependency manager because of {e}");
                    return result;
                }
            };

            // enriching the apps with the target properties from the libraries
            let library_properties = manager.get_target_properties().expect("lib properties");

            // merging app with library target properties
            for app in &mut config.apps {
                if let Err(e) = app.properties.merge(&library_properties) {
                    error!("cannot merge properties from the libraries with the app. error: {e}");
                    return result;
                }
            }
        }
        CommandSpec::Clean => {
            let output_root = &config.apps[0].output_root;
            if let Err(e) = remove_dir_all(&output_root.display().to_string()) {
                error!("lingo was unable to delete build folder! {e}");
            }
        }
        _ => {}
    }

    // Group apps by build system
    let mut by_build_system = HashMap::<(BuildSystem, TargetLanguage), Vec<&App>>::new();
    for app in &config.apps {
        by_build_system
            .entry((app.build_system(&which), app.target))
            .or_default()
            .push(app);
    }

    for (build_system, apps) in by_build_system {
        let mut sub_res = BatchBuildResults::for_apps(&apps);

        sub_res.map(|app| {
            // TODO: Support using lingo as a thin wrapper around west
            if app.platform == Platform::Zephyr {
                Err(Box::new(LingoError::UseWestBuildToBuildApp))
            } else {
                Ok(())
            }
        });

        match build_system {
            (BuildSystem::CMake, TargetLanguage::Cpp) => {
                cmake_cpp::CmakeCpp.execute_command(command, &mut sub_res)
            }
            (BuildSystem::CMake, TargetLanguage::C) => {
                cmake_c::CmakeC.execute_command(command, &mut sub_res)
            }
            (BuildSystem::Npm, TargetLanguage::TypeScript) => {
                npm::Npm.execute_command(command, &mut sub_res)
            }
            (BuildSystem::Pnpm, TargetLanguage::TypeScript) => {
                pnpm::Pnpm.execute_command(command, &mut sub_res)
            }
            (BuildSystem::LFC, _) => lfc::LFC.execute_command(command, &mut sub_res),
            (BuildSystem::Cargo, _) => todo!(),
            _ => {
                error!("invalid combination of target and platform!");
                todo!()
            }
        };
        result.append(sub_res);
    }
    result
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum BuildProfile {
    /// Compile with optimizations.
    Release,
    /// Compile with debug info.
    Debug,
}

pub struct BuildCommandOptions {
    /// Build profile, mostly relevant for target compilation.
    pub profile: BuildProfile,
    /// Whether to compile the target code.
    pub compile_target_code: bool,
    /// Path to the LFC executable.
    pub lfc_exec_path: PathBuf,
    /// Max threads to use for compilation. A value of zero means
    /// that the number will be automatically determined.
    /// A value of one effectively disables parallel builds.
    pub max_threads: usize,
    /// if compilation should continue if one of the apps fails building
    pub keep_going: bool,
}

/// Description of a lingo command
pub enum CommandSpec {
    /// Compile generated code with the target compiler.
    Build(BuildCommandOptions),
    /// Update dependencies
    Update,
    /// Clean build artifacts
    Clean,
}

/// Implemented by specific build strategies, eg for specific build tools.
pub trait BatchBackend {
    /// Build all apps, possibly in parallel.
    fn execute_command(&mut self, command: &CommandSpec, results: &mut BatchBuildResults);
}

/// Collects build results by app.
pub struct BatchBuildResults<'a> {
    results: Vec<(&'a App, BuildResult)>,
    keep_going: bool,
}

impl<'a> BatchBuildResults<'a> {
    /// Create an empty result, only for use with `append`.
    fn new() -> Self {
        Self {
            results: Vec::new(),
            keep_going: false,
        }
    }

    /// Create a result with an entry for each app. This can
    /// then be used by combinators like map and such.
    fn for_apps(apps: &[&'a App]) -> Self {
        Self {
            results: apps.iter().map(|&a| (a, Ok(()))).collect(),
            keep_going: false,
        }
    }
    /// sets the keep going value
    fn keep_going(&mut self, value: bool) {
        self.keep_going = value
    }

    /// Print this result collection to standard output.
    pub fn print_results(&self) {
        for (app, b) in &self.results {
            match b {
                Ok(()) => {
                    log::info!("- {}: Success", &app.name);
                }
                Err(e) => {
                    log::error!("- {}: Error: {}", &app.name, e);
                }
            }
        }
    }

    /// Absorb some results into this vector. Apps are not deduplicated, so this
    /// is only ok if the other is disjoint from this result.
    fn append(&mut self, mut other: BatchBuildResults<'a>) {
        self.results.append(&mut other.results);
        self.results.sort_by_key(|(app, _)| &app.name);
    }

    // Note: the duplication of the bodies of the following functions is benign, and
    // allows the sequential map to be bounded more loosely than if we were to extract
    // a function to get rid of the dup.

    /// Map results sequentially. Apps that already have a failing result recorded
    /// are not fed to the mapping function.
    pub fn map<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&'a App) -> BuildResult,
    {
        self.results.iter_mut().for_each(|(app, res)| {
            if let Ok(()) = res {
                *res = f(app);

                if (*res).is_err() && !self.keep_going {
                    panic!(
                        "build step failed because of {} with main reactor {}!",
                        &app.name,
                        &app.main_reactor.display()
                    );
                }
            }
        });
        self
    }

    /// Map results in parallel. Apps that already have a failing result recorded
    /// are not fed to the mapping function.
    pub fn par_map<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&'a App) -> BuildResult + Send + Sync,
    {
        self.results.par_iter_mut().for_each(|(app, res)| {
            if let Ok(()) = res {
                *res = f(app);

                if (*res).is_err() && !self.keep_going {
                    panic!(
                        "build step failed with error {:?} because of app {} with main reactor {}!",
                        &res,
                        &app.name,
                        &app.main_reactor.display()
                    );
                }
            }
        });
        self
    }

    /// Execute a function of all the apps that have not failed.
    /// The returned result is set to all apps, ie, either they
    /// all succeed, or they all fail for the same reason.
    pub fn gather<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&Vec<&'a App>) -> BuildResult,
    {
        // collect all apps that have not yet failed.
        let vec: Vec<&'a App> = self
            .results
            .iter()
            .filter_map(|&(app, ref res)| res.as_ref().ok().map(|()| app))
            .collect();

        if vec.is_empty() {
            return self;
        }

        match f(&vec) {
            Ok(()) => { /* Do nothing, all apps have succeeded. */ }
            Err(e) => {
                if !self.keep_going {
                    panic!("build step failed!");
                }

                // Mark all as failed for the same reason.
                let shared: Arc<AnyError> = e.into();
                for (_app, res) in &mut self.results {
                    if let Ok(()) = res {
                        *res = Err(Box::new(LingoError::Shared(shared.clone())));
                    }
                }
            }
        }
        self
    }
}
