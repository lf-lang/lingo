use std::collections::HashMap;
use std::path::PathBuf;

use std::sync::Arc;

use rayon::prelude::*;

use crate::args::Platform;
use crate::util::errors::{AnyError, BuildResult, LingoError};
use crate::{args::BuildSystem, package::App};

pub mod cmake;
pub mod lfc;
pub mod npm;
pub mod pnpm;

pub fn execute_command<'a>(command: &CommandSpec, apps: &[&'a App]) -> BatchBuildResults<'a> {
    // Group apps by build system
    let mut by_build_system = HashMap::<BuildSystem, Vec<&App>>::new();
    for &app in apps {
        by_build_system
            .entry(app.build_system())
            .or_default()
            .push(app);
    }

    let mut result = BatchBuildResults::new();
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
            BuildSystem::LFC => lfc::LFC.execute_command(command, &mut sub_res),
            BuildSystem::CMake => cmake::Cmake.execute_command(command, &mut sub_res),
            BuildSystem::Cargo => todo!(),
            BuildSystem::Npm => npm::Npm.execute_command(command, &mut sub_res),
            BuildSystem::Pnpm => pnpm::Pnpm.execute_command(command, &mut sub_res)
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
}

impl<'a> BatchBuildResults<'a> {
    /// Create an empty result, only for use with `append`.
    fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Create a result with an entry for each app. This can
    /// then be used by combinators like map and such.
    fn for_apps(apps: &[&'a App]) -> Self {
        Self {
            results: apps.iter().map(|&a| (a, Ok(()))).collect(),
        }
    }

    /// Print this result collection to standard output.
    pub fn print_results(&self) {
        for (app, b) in &self.results {
            match b {
                Ok(()) => {
                    println!("- {}: Success", &app.name);
                }
                Err(e) => {
                    println!("- {}: Error: {}", &app.name, e);
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
