use std::collections::HashMap;

use std::path::PathBuf;

use crate::util::errors::{merge, BuildResult};
use crate::{args::BuildSystem, package::App};

pub mod cmake;
pub mod lfc;

pub fn execute_command(command: BatchLingoCommand) -> BuildResult {
    // Group apps by build system
    let mut by_build_system = HashMap::<BuildSystem, Vec<&App>>::new();
    for &app in &command.apps {
        by_build_system
            .entry(app.build_system())
            .or_default()
            .push(app);
    }

    let mut result = Ok(());
    for (bs, apps) in by_build_system {
        let command = command.with_apps(apps);
        let sub_res = match bs {
            BuildSystem::LFC => lfc::LFC.execute_command(command),
            BuildSystem::CMake => cmake::Cmake.execute_command(command),
            BuildSystem::Cargo => Ok(()),
        };
        result = merge(result, sub_res);
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

#[derive(Clone)]
pub struct BuildCommandOptions {
    /// Build profile, mostly relevant for target compilation.
    pub profile: BuildProfile,
    /// Whether to compile the target code.
    pub compile_target_code: bool,
    /// Path to the LFC executable.
    pub lfc_exec_path: PathBuf,
}

/// Description of a lingo command
#[derive(Clone)]
pub enum CommandSpec {
    /// Compile generated code with the target compiler.
    Build(BuildCommandOptions),
    /// Update dependencies
    Update,
    /// Clean build artifacts
    Clean,
}

/// Batch of apps to process, possibly in parallel.
pub struct BatchLingoCommand<'a> {
    /// List of apps to build.
    pub apps: Vec<&'a App>,
    /// Action to take.
    pub task: CommandSpec,
}

impl<'a> BatchLingoCommand<'a> {
    fn with_apps<'b>(&self, apps: Vec<&'b App>) -> BatchLingoCommand<'b> {
        BatchLingoCommand {
            apps,
            task: self.task.clone(),
        }
    }
}

/// trait that all different build backends need to implement
pub trait BatchBackend {
    /// Build all apps, possibly in parallel.
    fn execute_command(&mut self, command: BatchLingoCommand) -> BuildResult;
}
