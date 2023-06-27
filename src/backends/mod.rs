pub mod cmake;
pub mod lfc;

use std::collections::HashMap;

use crate::{args::BuildSystem, package::App};

use std::error::Error;

use crate::util::errors::{BuildResult, LingoError, merge};

pub fn execute_command(command: BatchLingoCommand, ctx: &mut LingoCommandCtx) -> BuildResult {
    // Group apps by build system
    let mut by_build_system = HashMap::<BuildSystem, Vec<&App>>::new();
    for app in &command.apps {
        by_build_system
            .entry(app.build_system())
            .or_default()
            .push(app);
    }

    let mut result = Ok(());
    for (bs, apps) in by_build_system {
        let command = command.with_apps(apps);
        let sub_res = match bs {
            BuildSystem::LFC => lfc::LFC.execute_command(command, ctx),
            BuildSystem::CMake => cmake::Cmake.execute_command(command, ctx),
            BuildSystem::Cargo => Ok(()),
        };
        result = merge(result, sub_res);
    }
    result
}

pub struct LingoCommandCtx {
    fail_at_end: bool,
}

impl LingoCommandCtx {
    pub fn new(fail_at_end: bool) -> Self {
        Self { fail_at_end }
    }

    pub(self) fn notify_failed(&mut self, app: &App, error: &dyn Error) -> BuildResult {
        println!("Failed building app {}: {}", app.name, error);
        if self.fail_at_end {
            Ok(())
        } else {
            Err(Box::new(LingoError::RunAborted()))
        }
    }
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
    fn execute_command(
        &mut self,
        command: BatchLingoCommand,
        ctx: &mut LingoCommandCtx,
    ) -> BuildResult;
}
